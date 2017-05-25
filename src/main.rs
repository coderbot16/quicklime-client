#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate glutin;

//mod console;
//use console::{Kind, Value};

mod input;
mod text;
mod ui;
mod render2d;
use render2d::Rect;

use text::render::Command;
use std::fs::File;
use input::Screen;
use glutin::Event;
use std::io::BufReader;
use ui::{Scene, State, Element, Kind, Coloring};
use ui::lit::Lit;
//mod resource;

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;

use gfx::{Device, Encoder};
use gfx::traits::{FactoryExt, Factory};
use gfx::texture;

extern crate memmap;
extern crate image;
use image::ImageFormat;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        color: [f32; 3] = "a_Color",
        tex: [f32; 2] = "at_tex_coord",
    }

    constant Transform {
        transform: [[f32; 4];4] = "u_Transform",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::ConstantBuffer<Transform> = "Transform",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
        tex: gfx::TextureSampler<[f32; 4]> = "s_texture",
    }
}

const VERTEX_SHADER: &str = "
#version 150 core

in vec4 a_Pos;
in vec3 a_Color;
in vec2 at_tex_coord;

uniform Transform {
    mat4 u_Transform;
};

out vec4 v_Color;
out vec2 v_tex_coord;

void main() {
	v_tex_coord = at_tex_coord;
    v_Color = vec4(a_Color, 1.0);
    gl_Position = a_Pos * u_Transform;
}
";

const FRAGMENT_SHADER: &str = "
#version 150 core

uniform sampler2D s_texture;

in vec4 v_Color;
in vec2 v_tex_coord;

out vec4 Target0;

void main() {
	vec4 tex_color = texture(s_texture, v_tex_coord);
	
	if(v_tex_coord == vec2(0, 0)) {
		// TODO: this should be a seperate shader.
		Target0 = v_Color;
	} else {
		Target0 = tex_color * v_Color;
	}
}
";

/*const TRIANGLE: [Vertex; 3] = [
	   /* Vertex { pos: [ -0.5, -0.5, -0.5, 1.0], color: [1.0, 0.0, 0.0] },
	    Vertex { pos: [  0.5, -0.5, -0.5, 1.0], color: [0.0, 1.0, 0.0] },
	    Vertex { pos: [  0.0,  0.5, -0.5, 1.0 ], color: [0.0, 0.0, 1.0] }*/
	];*/

fn main() {
	println!("Starting quicklime-client version 0.0.1");
	
	/*let sv_cheats = Kind::Int {low: 0, high: 1};
	let ok = "1";
	let bad = "100";
	
	println!("{:?}", sv_cheats.parse(ok));
	println!("{:?}", sv_cheats.parse(bad));*/
	
	/*let mut screen = Screen::new();
	//let sp_slice = ScreenSlice {x_min: -0.2, x_max: 0.2, y_min: -0.2, y_max: 0.2};
	//let mp_slice = ScreenSlice {x_min: -0.2, x_max: 0.2, y_min: -0.5, y_max: -0.3};

	let file = File::open("/home/coderbot/eclipseRust/workspace/quicklime-client/assets/minecraft/font/glyph_sizes.bin").unwrap();
	let glyph_metrics = text::metrics::GlyphMetrics::from_file(&file).unwrap();
	let metrics = text::metrics::Metrics::unicode(glyph_metrics);
	
	let ctxt = text::render::RenderingContext::new(&metrics);
	let mut style = text::style::Style::new(text::style::Color::Palette(text::style::PaletteColor::White));
	// TODO: Bold doesn't work.
	style.flags = style.flags.set_italic(true);
	
	let page_0_file = File::open("/home/coderbot/eclipseRust/workspace/quicklime-client/assets/minecraft/textures/font/unicode_page_00.png").unwrap();
	let page_0 = image::load(BufReader::new(page_0_file), ImageFormat::PNG).expect("failed to load image").flipv();
	let rgba = page_0.to_rgba();
	
	let mut page_0_raw = Vec::with_capacity(256*256);
	
	for (x, y, pixel) in rgba.enumerate_pixels() {
		let data = if pixel.data[3] == 0 {
			[0, 0, 0, 0]
		} else {
			pixel.data
		};
		
		page_0_raw.push(data);
		
		if x >= 128 && x < 144 && y >= 64 && y < 80 {
			println!("{:?}", data);
		}
	}
	
	//let vertex_data: Vec<Vertex> = 
	//	sp_slice.as_triangles(-0.5).into_iter()
	//	.chain(mp_slice.as_triangles(-0.5).into_iter())
	//	.map(|pos| Vertex {pos: *pos, color: [1.0, 1.0, 1.0]} )
	//	.collect(); 

	for m in glutin::get_available_monitors() {
		println!("Monitor name: {:?}", m.get_name())
	}

	//screen.add_interact_area(Area { id: "singleplayer".to_string(), slice: sp_slice });
	//screen.add_interact_area(Area { id: "multiplayer".to_string(), slice: mp_slice });
	
	let builder = glutin::WindowBuilder::new()
        .with_title("quicklime-client [Minecraft 1.10.2]".to_string())
        //.with_dimensions(1920, 1080)
        .with_fullscreen(glutin::get_primary_monitor())
        .with_vsync();
    let (window, mut device, mut factory, main_color, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
        
	let mut last_half_size = (960.0, 540.0);
	let scale_factor = 2.0;
	
	let scale = ((1.0 / last_half_size.0) * scale_factor, (1.0 / last_half_size.1) * scale_factor);
	
	let mut vertex_data: Vec<Vertex> = Vec::new();
	
	for command in ctxt.render(0.0, 0.0, "Hello World!".chars(), &style, false).filter_map(|x| x) {
		println!("{:?}", command);
		match command {
			Command::Char( ref draw_command ) => {
				vertex_data.extend (
					draw_command
					.to_quad(scale, [1.0, 1.0, 1.0])
					.as_triangles()
					.iter()
					.map(|vertex| Vertex { pos: [vertex.pos[0], vertex.pos[1], -0.5, 1.0], color: vertex.color, tex: vertex.tex })
				);
			},
			Command::CharDefault { .. } => panic!("Can't draw default chars"),
			Command::Rect { x, y, width, height } => {
				let (x, y) = (x * scale.0, y * scale.1);
				let (width, height) = (width * scale.0, height * scale.1);
				
				vertex_data.extend (
					Rect::solid([x, y], [x + width, y + height], [1.0, 1.0, 1.0])
					.as_quad()
					.as_triangles()
					.iter()
					.map(|vertex| Vertex { pos: [vertex.pos[0], vertex.pos[1], -0.5, 1.0], color: vertex.color, tex: vertex.tex })
				)
			},
		}
	}
	
	let pso = factory.create_pipeline_simple(
        VERTEX_SHADER.as_bytes(),
        FRAGMENT_SHADER.as_bytes(),
        pipe::new()
    ).unwrap();
	
	let mut encoder: Encoder<_, _> = factory.create_command_buffer().into();
	
	// Identity matrix	
	const TRANSFORM: Transform = Transform {
	    transform: [[1.0, 0.0, 0.0, 0.0],
	                [0.0, 1.0, 0.0, 0.0],
	                [0.0, 0.0, 1.0, 0.0],
	                [0.0, 0.0, 0.0, 1.0]]
	};
	
	let (_, texture_view) = factory.create_texture_immutable::<ColorFormat>(
        texture::Kind::D2(256, 256, texture::AaMode::Single),
        &[&page_0_raw as &[[u8; 4]]]
        ).unwrap();

    let sampler = factory.create_sampler(texture::SamplerInfo::new(
        texture::FilterMethod::Scale,
        texture::WrapMode::Tile,
    ));
	
	let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, ());
	let transform_buffer = factory.create_constant_buffer(1);
	let data = pipe::Data {
		tex: (texture_view, sampler),
	    vbuf: vertex_buffer,
	    transform: transform_buffer,
	    out: main_color,
	};
	
	encoder.update_buffer(&data.transform, &[TRANSFORM], 0).expect("Failed to update transform buffer");
	
	'main: loop {
		for event in window.poll_events() {
			match event {
				Event::Closed => break 'main,
				Event::Resized(x, y) => {println!("New window size: {}, {}", x, y); last_half_size = (x as f32 / 2.0, y as f32 / 2.0)},
				Event::MouseMoved(x, y) => screen.position((x as f32) / (last_half_size.0 as f32) - 1.0, 1.0 - (y as f32) / (last_half_size.1 as f32)),
				Event::MouseInput(state, button) => screen.mouse_click(state, button),
				_ => println!("ev: {:?}", event)
			}
		}
		
		// 9bc6fe
		encoder.clear(&data.out, [(0x9b as f32) / 255.0, (0xc6 as f32) / 255.0, (0xfe as f32) / 255.0, 1.0]);
		encoder.draw(&slice, &pso, &data);
		encoder.flush(&mut device);
		
		window.swap_buffers().unwrap();
		device.cleanup();
	}*/
	
	/*use text::flat::{Component, ChatBuf, Kind, Mode};
	use text::style::Style;
	
	let mut buf = ChatBuf::new();
	
	let hello = Component::new("Hello World!", Kind::Text, Style::new(), Mode::Level, false);
	let space = Component::new(" ", Kind::Text, Style::new(), Mode::Deeper, false);
	let end = Component::new("FooBarBaz", Kind::Text, Style::new(), Mode::Level, false);
	
	buf.push(hello.unwrap());
	buf.push(space.unwrap());
	buf.push(end.unwrap());
	
	println!("{}", ::std::mem::size_of::<ChatBuf>());
	println!("{:?}", buf);
	
	println!("{:?}", ChatBuf::from_formatted("Hello World!"));
	
	println!("{:?}", ChatBuf::from_formatted("§4§lMinecraft §6§lServer"));
	
	for component in buf.components() {
		println!("{:?}", component);
	}
	
	let redstone_creations = ChatBuf::from_formatted("§1R§2e§3d§4s§5t§6o§7n§8e §9C§ar§be§ca§dt§ei§fo§1n§2s");
	println!("{:?}", redstone_creations);
	for component in redstone_creations.components() {
		println!("{:?}", component);
	}*/
	
	/*let mut scene = Scene::new();
	let state = State {
		name: "default".to_owned(),
		center: (Lit::new(0.0, 0), Lit::new(0.0, 0)),
		extents: (Lit::new(0.5, 0), Lit::new(0.5, 0)),
		color: Coloring::Solid([1.0, 0.0, 1.0]),
		kind: Kind::Nodraw
	};
	
	let element = Element {
		current: None,
		default: state,
		states: vec![]
	};
	
	scene.elements.insert("test".to_owned(), element);
	
	println!("{}", serde_json::to_string(&scene).unwrap());*/
	
	
}