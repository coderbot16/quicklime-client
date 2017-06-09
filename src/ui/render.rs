use gfx::{self, Factory, Resources, PipelineState, Encoder, CommandBuffer};
use gfx::traits::FactoryExt;
use ui::managed::ManagedBuffer;
use gfx::handle::{ShaderResourceView, Sampler, RenderTargetView, DepthStencilView};
use gfx::texture;
use gfx::format::Formatted;
use resource::atlas::{Texmap, TextureSelection};
use std::collections::HashMap;

use ColorFormat;
use DepthFormat;

// TODO: Multitexture-per-PSO
pub struct TexturedPipe<R> where R: Resources {
	buffer: ManagedBuffer<R>,
	state: PipelineState<R, textured_pipe::Meta>,
	data: textured_pipe::Data<R>,
}

impl<R> TexturedPipe<R> where R: Resources {
	pub fn create<F>(factory: &mut F, image: &[u8], out: RenderTargetView<R, ColorFormat>, out_depth: DepthStencilView<R, DepthFormat>) -> Self where F: Factory<R> + FactoryExt<R> {
		let state = factory.create_pipeline_simple(
	        VERTEX_SHADER_TEX.as_bytes(),
	        FRAGMENT_SHADER_TEX.as_bytes(),
	        textured_pipe::new()
	    ).unwrap();
		
		let buffer = ManagedBuffer::new(factory);
		
		let (_, view) = factory.create_texture_immutable_u8::<(gfx::format::R8_G8_B8_A8, gfx::format::Srgb)>(
			texture::Kind::D2(256, 256, texture::AaMode::Single),
			&[image]
		).unwrap();

		let sampler = factory.create_sampler(texture::SamplerInfo::new(
			texture::FilterMethod::Scale,
			texture::WrapMode::Tile,
		));
		
	    let data = textured_pipe::Data {
			buffer: buffer.remote().clone(),
			out: out,
			out_depth: out_depth,
			texture: (view, sampler),
		};
	    
	    TexturedPipe {
	    	buffer: buffer,
	    	state: state,
	    	data: data
	    }
	}
	
	pub fn buffer_mut(&mut self) -> &mut ManagedBuffer<R> {
		&mut self.buffer
	}
	
	pub fn render<F, C>(&mut self, factory: &mut F, encoder: &mut Encoder<R, C>) where F: Factory<R> + FactoryExt<R>, C: CommandBuffer<R> {
		self.buffer.update(factory, encoder);
		self.data.buffer = self.buffer.remote().clone();
		
		encoder.draw(&self.buffer.slice(), &self.state, &self.data);
	}
}

pub struct SolidPipe<R> where R: Resources {
	buffer: ManagedBuffer<R>,
	state: PipelineState<R, solid_pipe::Meta>,
	data: solid_pipe::Data<R>
}

impl<R> SolidPipe<R> where R: Resources {
	pub fn create<F>(factory: &mut F, out: RenderTargetView<R, ColorFormat>, out_depth: DepthStencilView<R, DepthFormat>) -> Self where F: Factory<R> + FactoryExt<R> {
		let state = factory.create_pipeline_simple(
	        VERTEX_SHADER_SOLID.as_bytes(),
	        FRAGMENT_SHADER_SOLID.as_bytes(),
	        solid_pipe::new()
	    ).unwrap();
		
		let buffer = ManagedBuffer::new(factory);
		
	    let data = solid_pipe::Data {
			buffer: buffer.remote().clone(),
			out: out,
			out_depth: out_depth
		};
	    
	    SolidPipe {
	    	buffer: buffer,
	    	state: state,
	    	data: data
	    }
	}
	
	pub fn buffer_mut(&mut self) -> &mut ManagedBuffer<R> {
		&mut self.buffer
	}
	
	pub fn render<F, C>(&mut self, factory: &mut F, encoder: &mut Encoder<R, C>) where F: Factory<R> + FactoryExt<R>, C: CommandBuffer<R> {
		self.buffer.update(factory, encoder);
		self.data.buffer = self.buffer.remote().clone();
		
		encoder.draw(&self.buffer.slice(), &self.state, &self.data);
	}
}

pub struct Context<R> where R: Resources {
	out: RenderTargetView<R, ColorFormat>,
	out_depth: DepthStencilView<R, DepthFormat>,
	solid: SolidPipe<R>,
	textured: Vec<TexturedPipe<R>>,
	textures: HashMap<String, (usize, TextureSelection)>
}

impl<R> Context<R> where R: Resources {
	pub fn create<F>(factory: &mut F, out: RenderTargetView<R, ColorFormat>, out_depth: DepthStencilView<R, DepthFormat>) -> Self where F: Factory<R> + FactoryExt<R> {
		Context {
			out: out.clone(),
			out_depth: out_depth.clone(),
			solid: SolidPipe::create(factory, out.clone(), out_depth.clone()),
			textured: Vec::new(),
			textures: HashMap::new()
		}
	}
	
	pub fn new_zone(&mut self) -> usize {
		let zone = self.solid.buffer_mut().new_zone();
		
		for pipe in &mut self.textured {
			pipe.buffer_mut().new_zone();
		}
		
		zone
	}
	
	pub fn extend_zone<I>(&mut self, iter: I, texture: Option<&str>) -> bool where I: IntoIterator<Item=Vertex> {
		if let Some(texture) = texture {
			if let Some(&(index, selection)) = self.textures.get(texture) {
				Self::extend_textured(&mut self.textured[index], iter, selection);
				
				true
			} else {
				false
			}
		} else {
			self.solid.buffer_mut().extend(iter);
			
			true
		}
	}
	
	fn extend_textured<I>(pipe: &mut TexturedPipe<R>, iter: I, selection: TextureSelection) where I: IntoIterator<Item=Vertex> {
		pipe.buffer_mut().extend(iter.into_iter().map(|v| {let v = Vertex { 
			pos: v.pos, 
			color: v.color, 
			tex: [
				(selection.min[0].to_part(0.0) + v.tex[0] * selection.size[0].to_part(0.0) + 1.0) / 2.0,
				(selection.min[1].to_part(0.0) + v.tex[1] * selection.size[1].to_part(0.0) + 1.0) / 2.0
			]
		}; println!("{:?}", v); v}))
	}
	
	pub fn add_texture<F>(&mut self, factory: &mut F, texmap: &Texmap, texture: &[u8]) where F: Factory<R> + FactoryExt<R> {
		let index = self.textured.len();
		self.textured.push(TexturedPipe::create(factory, texture, self.out.clone(), self.out_depth.clone()));
		
		for (k, v) in texmap.0.iter() {
			self.textures.insert(k.clone(), (index, *v));
		}
	}
	
	pub fn render<F, C>(&mut self, factory: &mut F, encoder: &mut Encoder<R, C>) where F: Factory<R> + FactoryExt<R>, C: CommandBuffer<R> {
		self.solid.render(factory, encoder);
		
		for pipe in &mut self.textured {
			pipe.render(factory, encoder);
		}
	}
}

gfx_defines!{
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        color: [f32; 3] = "a_Color",
        tex: [f32; 2] = "a_Tex",
    }

    pipeline textured_pipe {
        buffer: gfx::VertexBuffer<Vertex> = (),
        out: gfx::BlendTarget<ColorFormat> = ("Out", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
        texture: gfx::TextureSampler<[f32; 4]> = "Texture",
    }
    
    pipeline solid_pipe {
        buffer: gfx::VertexBuffer<Vertex> = (),
        out: gfx::BlendTarget<ColorFormat> = ("Out", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

const VERTEX_SHADER_TEX: &str = "
#version 130

in vec3 a_Pos;
in vec3 a_Color;
in vec2 a_Tex;

out vec4 v_Color;
out vec2 v_Tex;

void main() {
	v_Tex = a_Tex;
    v_Color = vec4(a_Color, 1.0);
    gl_Position = vec4(a_Pos, 1.0);
}
";

const FRAGMENT_SHADER_TEX: &str = "
#version 130

uniform sampler2D Texture;

in vec4 v_Color;
in vec2 v_Tex;

out vec4 Out;

void main() {
	Out = texture(Texture, v_Tex) * v_Color;
}
";

const VERTEX_SHADER_SOLID: &str = "
#version 130

in vec3 a_Pos;
in vec3 a_Color;

out vec4 v_Color;

void main() {
    v_Color = vec4(a_Color, 1.0);
    gl_Position = vec4(a_Pos, 1.0);
}
";

const FRAGMENT_SHADER_SOLID: &str = "
#version 130

uniform sampler2D Texture;

in vec4 v_Color;

out vec4 Out;

void main() {
	Out = v_Color;
}
";