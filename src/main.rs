#[macro_use]
extern crate serde_derive;

extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate glutin;

//mod console;
//use console::{Kind, Value};

mod input;
mod text;
mod ui;
mod render2d;
mod directory;
mod color;

use color::{Rgb, Rgba};
//mod scoreboard;

use render2d::Rect;

use text::language;
use std::fs::File;
use input::Screen;
use glutin::Event;
use std::io::BufReader;
use ui::{Vertex, Scene, State, Element, Kind, Coloring};
use ui::lit::Lit;
use ui::render::Context;
use resource::atlas::{self, Texmap};
mod resource;

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;

use gfx::{Device, Encoder};

extern crate memmap;
extern crate image;
use image::ImageFormat;

pub type ColorFormat = (gfx::format::R8_G8_B8_A8, gfx::format::Srgb);
pub type DepthFormat = gfx::format::DepthStencil;

fn main() {
	// minimum width: 320x240
	
	println!("Starting quicklime-client version 0.0.1");

	let file = File::open("/home/coderbot/eclipseRust/workspace/quicklime-client/assets/minecraft/font/glyph_sizes.bin").unwrap();
	let glyph_metrics = text::metrics::GlyphMetrics::from_file(&file).unwrap();
	let metrics = text::metrics::Metrics::unicode(glyph_metrics);
	
	let ctxt = text::render::RenderingContext::new(&metrics);
	let mut style = text::style::Style::new();
	style.color = text::style::Color::Palette(text::style::PaletteColor::White);
	
	let page_0_file = File::open("/home/coderbot/eclipseRust/workspace/quicklime-client/assets/minecraft/textures/font/unicode_page_00.png").unwrap();
	let widgets_file = File::open("assets/minecraft/textures/gui/widgets.png").unwrap();
	
	let page_0 = image::load(BufReader::new(page_0_file), ImageFormat::PNG).expect("failed to load image").flipv().to_rgba().into_raw();
	let widgets = image::load(BufReader::new(widgets_file), ImageFormat::PNG).expect("failed to load image").flipv().to_rgba().into_raw();
	
	let builder = glutin::WindowBuilder::new()
        .with_title("quicklime-client [Minecraft 1.10.2]".to_string())
        //.with_dimensions(1920, 1080)
        .with_fullscreen(glutin::get_primary_monitor())
        .with_vsync();
    let (window, mut device, mut factory, main_color, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
        
	let mut last_half_size = (960.0, 540.0);
	
	let scale_factor = 1.0;
	let scale = ((1.0 / last_half_size.0) * scale_factor, (1.0 / last_half_size.1) * scale_factor);
	
	let mut rect_data: Vec<Vertex> = Vec::new();
	
	let mut sfc = 0.0;
	let texmap: atlas::TexmapBucket = serde_json::from_reader(File::open("resources/texmaps/gui.json").unwrap()).unwrap();
	if let Some(texmap) = texmap.0.get("minecraft:textures/gui/widgets.png") {
		for (k, v) in texmap.0.iter() {
			let min = [v.min[0].to_part(0.0), v.min[1].to_part(0.0)];
			let size = [v.size[0].to_part(0.0), v.size[1].to_part(0.0)];
			sfc += size[0] * size[1] * 16384.0;
			
			let color = {
				[0.25 + (min[0]*16.0)%0.5, 0.25 + (min[1]*16.0)%0.5, 0.25 + (min[0]*16.0)%0.5]
			};
			
			let min = [min[0] * 0.5625, min[1]];
			let size = [size[0] * 0.5625, size[1]];
			let max = [min[0] + size[0], min[1] + size[1]];
			
			let width = scale.0 * 2.0;
			let height = scale.1 * 2.0;
			
			let rects = [
				Rect::solid(min, [min[0] + width, max[1]], color),
				Rect::solid(min, [max[0], min[1] + height], color),
				Rect::solid([max[0] - width, min[1]], [max[0], min[1] + size[1]], color),
				Rect::solid([min[0], max[1] - height], [min[0] + size[0], max[1]], color),
			];
			
			/*for r in rects.iter() {
				rect_data.extend (
					r
					.as_quad()
					.as_triangles()
					.iter()
					.map(|vertex| Vertex { pos: [vertex.pos[0], vertex.pos[1], 0.2], color: vertex.color, tex: vertex.tex })
				);
			}*/
			
			println!("{}: {:?}", k, v);
		}
	}
	
	println!("SFC: {} / 65536.0 ({}%)", sfc, (sfc/65536.0)*100.0);
	
	let test_file = File::open("resources/test.json").unwrap();
	let mut test = serde_json::from_reader::<File, Scene>(test_file).unwrap();
	
	let mut encoder: Encoder<_, _> = factory.create_command_buffer().into();
	
	// TODO: Transparency sorting.
	let mut context = Context::create(&mut factory, main_color.clone(), main_depth.clone());
	context.add_texture(&mut factory, texmap.0.get("minecraft:textures/gui/widgets.png").unwrap(), &widgets);
	context.add_texture(&mut factory, &Texmap::new("unicode_page_00".to_owned()), &page_0);
	context.new_zone();
	context.extend_zone(rect_data.iter().map(|x| *x), None);
	
	let mut depth = 1.0;
	
	for (name, element) in &mut test.elements {
		element.default.push_to(scale, depth, &mut context, &metrics);
		depth /= 2.0;
	}
	
	'main: loop {
		for event in window.poll_events() {
			match event {
				Event::Closed => break 'main,
				Event::Resized(x, y) => {println!("New window size: {}, {}", x, y); last_half_size = (x as f32 / 2.0, y as f32 / 2.0)},
				Event::MouseMoved(x, y) => /*screen.position((x as f32) / (last_half_size.0 as f32) - 1.0, 1.0 - (y as f32) / (last_half_size.1 as f32))*/(),
				//Event::MouseInput(state, button) => screen.mouse_click(state, button),
				_ => println!("ev: {:?}", event)
			}
		}
		
		let sky = Rgba::new(0x9b, 0xc6, 0xfe, 0xff);
		
		encoder.clear(&main_color, sky.to_linear());
		encoder.clear_depth(&main_depth, 1.0);
		context.render(&mut factory, &mut encoder);
		
		encoder.flush(&mut device);
		
		// clear depth
		
		window.swap_buffers().unwrap();
		device.cleanup();
	}
	
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
	
	/*let name = "assets/minecraft/lang/en_US.lang";
	let mut read = File::open(name).unwrap();
	let mut dir = language::Directory::load(&mut read, name).unwrap();
	
	print_helper(None, dir.root(), -1);*/
}