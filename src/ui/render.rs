use gfx::{self, Factory, Resources, PipelineState, Encoder, CommandBuffer};
use gfx::traits::FactoryExt;
use ui::managed::ManagedBuffer;
use gfx::handle::{ShaderResourceView, Sampler, RenderTargetView, DepthStencilView};
use gfx::texture;
use gfx::format::Formatted;

pub use self::textured_pipe as pipe;
pub use self::VERTEX_SHADER_TEX as VERTEX_SHADER;
pub use self::FRAGMENT_SHADER_TEX as FRAGMENT_SHADER;

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

struct SolidPipe<R> where R: Resources {
	buffer: ManagedBuffer<R>,
	state: PipelineState<R, solid_pipe::Meta>,
	data: solid_pipe::Data<R>
}

struct Context<R> where R: Resources {
	solid: SolidPipe<R>,
	textured: Vec<TexturedPipe<R>>
}

impl<R> Context<R> where R: Resources {
	fn create<F>(factory: &mut F, image: &[u8]) where F: Factory<R> + FactoryExt<R> {
		let textured = factory.create_pipeline_simple(
	        VERTEX_SHADER_TEX.as_bytes(),
	        FRAGMENT_SHADER_TEX.as_bytes(),
	        textured_pipe::new()
	    );
		
		let solid = factory.create_pipeline_simple(
	        VERTEX_SHADER_SOLID.as_bytes(),
	        FRAGMENT_SHADER_SOLID.as_bytes(),
	        solid_pipe::new()
	    );
		
		let buffer = ManagedBuffer::new(factory);
		
		let (_, texture_view) = factory.create_texture_immutable_u8::<(gfx::format::R8_G8_B8_A8, gfx::format::Srgb)>(
			texture::Kind::D2(256, 256, texture::AaMode::Single),
			&[image]
		).unwrap();

		let sampler = factory.create_sampler(texture::SamplerInfo::new(
			texture::FilterMethod::Scale,
			texture::WrapMode::Tile,
		));
    
	    /*let tex_data = textured_pipe::Data {
			vbuf: vertex_buffer,
			out: main_color,
			tex: (texture_view, sampler),
			// TODO: Depth
		};*/
    
	    /*let data = pipe::Data {
			tex: (texture_view, sampler),
			vbuf: vertex_buffer,
			out: main_color,
			// TODO: Depth
		};*/
	}
	
	fn render<F, C>(&self, factory: &mut F, encoder: &mut Encoder<R, C>) where F: Factory<R> + FactoryExt<R>, C: CommandBuffer<R> {
		
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

pub const VERTEX_SHADER_TEX: &str = "
#version 150 core

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

pub const FRAGMENT_SHADER_TEX: &str = "
#version 150 core

uniform sampler2D Texture;

in vec4 v_Color;
in vec2 v_Tex;

out vec4 Out;

void main() {
	Out = texture(Texture, v_Tex) * v_Color;
}
";

const VERTEX_SHADER_SOLID: &str = "
#version 150 core

in vec3 a_Pos;
in vec3 a_Color;

out vec4 v_Color;

void main() {
    v_Color = vec4(a_Color, 1.0);
    gl_Position = a_Pos;
}
";

const FRAGMENT_SHADER_SOLID: &str = "
#version 150 core

uniform sampler2D Texture;

in vec4 v_Color;

out vec4 Out;

void main() {
	Out = v_Color;
}
";