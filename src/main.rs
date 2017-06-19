#[macro_use]
extern crate serde_derive;

extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate glutin;
extern crate num;

mod input;
mod text;
mod ui;
mod render2d;
mod directory;
mod color;
mod segment;

use color::Rgba;
//mod scoreboard;

use std::fs::File;
use glutin::Event;
use std::io::BufReader;
use ui::render::Context;
use resource::atlas::{self, Texmap};
mod resource;

use resource::atlas::TexmapBucket;
use std::collections::HashMap;

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

	let file = File::open("assets/minecraft/font/glyph_sizes.bin").unwrap();
	let glyph_metrics = text::metrics::GlyphMetrics::from_file(&file).unwrap();
	let metrics = text::metrics::Metrics::unicode(glyph_metrics);
	
	let page_0_file = File::open("assets/minecraft/textures/font/unicode_page_00.png").unwrap();
	let widgets_file = File::open("assets/minecraft/textures/gui/widgets.png").unwrap();
	
	let page_0 = image::load(BufReader::new(page_0_file), ImageFormat::PNG).expect("failed to load image").flipv().to_rgba().into_raw();
	let widgets = image::load(BufReader::new(widgets_file), ImageFormat::PNG).expect("failed to load image").flipv().to_rgba().into_raw();
	
	let builder = glutin::WindowBuilder::new()
        .with_title("quicklime-client [Minecraft 1.10.2]".to_string())
        //.with_dimensions(1920, 1080)
        .with_fullscreen(glutin::get_primary_monitor())
        .with_vsync()
        .with_gl(glutin::GlRequest::GlThenGles {
        	opengl_version: (2, 1),
        	opengles_version: (2, 1)	
        });
    let (window, mut device, mut factory, main_color, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
        
	let mut last_half_size = (960.0, 540.0);
	
	let scale_factor = 2.0;
	let scale = ((1.0 / last_half_size.0) * scale_factor, (1.0 / last_half_size.1) * scale_factor);
	
	let test_file = File::open("resources/scenes/Button.json").unwrap();
	let test_multiple_file = File::open("resources/scenes/Main.json").unwrap();
	
	let test_org = serde_json::from_reader::<File, ::ui::replace::IncompleteScene>(test_file).unwrap();
	let mut test_multiple = serde_json::from_reader::<File, ::ui::replace::IncompleteScene>(test_multiple_file).unwrap().complete(&HashMap::new()).unwrap();
	
	let mut scenes = HashMap::new();
	scenes.insert("Button".to_owned(), test_org);
	
	test_multiple.bake_all(&scenes).unwrap();
	
	let bucket_file = File::open("resources/texmaps/gui.json").unwrap();
	let bucket = serde_json::from_reader::<_, TexmapBucket>(bucket_file).unwrap();
	
	let mut encoder: Encoder<_, _> = factory.create_command_buffer().into();
	
	// TODO: Transparency sorting.
	let mut context = Context::create(&mut factory, main_color.clone(), main_depth.clone());
	context.add_texture(&mut factory, bucket.0.get("minecraft:textures/gui/widgets.png").unwrap(), &widgets);
	context.add_texture(&mut factory, &Texmap::new("unicode_page_00".to_owned()), &page_0);
	
	for element in test_multiple.elements.values_mut() {
		element.default.push_to((0.0, 0.0), scale, (1.0, 1.0), &mut context, &metrics);
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
	
	/*let name = "assets/minecraft/lang/en_US.lang";
	let mut read = File::open(name).unwrap();
	let mut dir = language::Directory::load(&mut read, name).unwrap();
	
	print_helper(None, dir.root(), -1);*/
}