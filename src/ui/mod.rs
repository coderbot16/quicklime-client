pub mod lit;
pub mod render;
pub mod managed;
pub use self::render::Vertex as Vertex;
pub mod input;
pub mod replace;

use input::ScreenSlice;
use std::rc::Rc;
use std::collections::HashMap;
use text::flat::ChatBuf;
use render2d::{Vertex2D, Color, Rect, Quad};
use ui::lit::Lit;
use ui::render::Context;
use gfx::Resources;
use text::render::{RenderingContext, Command};
use text::metrics::Metrics;
use text::style::{StyleFlags, Style};
use text::pages::PAGES;
use text::align::Align;
use color::{Rgb, Rgba};
use self::input::Input;
use serde_json::Value;
use ui::replace::IncompleteScene;
use std::fmt;
use serde::de::{Error, Deserializer, Deserialize, Visitor, MapAccess};

#[derive(Serialize, Deserialize)]
pub struct Scene {
	pub elements: HashMap<String, Element>,
	pub inputs: HashMap<String, Input>
}

impl Scene {
	pub fn new() -> Self {
		Scene {
			elements: HashMap::new(),
			inputs: HashMap::new()
		}
	}
	
	pub fn bake_all(&mut self, scenes: &HashMap<String, IncompleteScene>) -> Result<(), replace::Error> {
		for value in self.elements.values_mut() {
			for state in &mut value.states {
				state.bake(scenes)?;
			}
			
			value.default.bake(scenes)?;
		}
		
		Ok(())
	}
}

#[derive(Serialize, Deserialize)]
pub struct Element {
	#[serde(skip_serializing, skip_deserializing)]
	pub current: Option<usize>,
	pub default: State,
	#[serde(default = "Vec::new")]
	pub states: Vec<State>
}

impl Element {
	fn state(&self) -> &State {
		match self.current {
			None => &self.default,
			Some(idx) => &self.states[idx]
		}
	}
	
	fn state_mut(&mut self) -> &mut State {
		match self.current {
			None => &mut self.default,
			Some(idx) => &mut self.states[idx]
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct State {
	#[serde(default = "default_name")]
	pub name: String,
	pub center: (Lit, Lit),
	pub extents: (Lit, Lit),
	#[serde(default = "Coloring::white")]
	pub color: Coloring,
	pub kind: KindWrapper,
	pub z: f32,
	#[serde(skip_serializing, skip_deserializing)]
	pub zone_id: Option<usize>
}

fn default_name() -> String {
	"default".to_string()
}

impl State {
	pub fn bake(&mut self, scenes: &HashMap<String, IncompleteScene>) -> Result<(), replace::Error> {
		let replacement = if let Kind::Import { ref scene, ref with } = self.kind.0 {
			let incomplete = scenes.get(scene).ok_or_else(|| replace::Error::SceneLookupFailed(scene.to_owned()))?;
			
			let mut complete = incomplete.complete(with)?;
			complete.bake_all(scenes)?;
			
			Some(Kind::Baked (complete))
		} else {
			None
		};
		
		if let Some(replacement) = replacement {
			self.kind = KindWrapper(replacement);
		}
		
		Ok(())
	}
	
	pub fn push_to<R>(&mut self, offset: (f32, f32), scale: (f32, f32), viewport_scale: (f32, f32), context: &mut Context<R>, metrics: &Metrics) where R: Resources {
		match self.kind.0 {
			Kind::Rect => {
				self.zone_id = Some(context.new_zone());
				
				let extents = (self.extents.0.to_part(scale.0) * viewport_scale.0, self.extents.1.to_part(scale.1) * viewport_scale.1);
				
				context.extend_zone(Rect {
					min: Vertex2D { pos: [self.center.0.to_part(scale.0) - extents.0, self.center.1.to_part(scale.1) - extents.1], 
						tex: [0.0, 0.0], color: self.color.bottom_left().to_linear()},
					max: Vertex2D { pos: [self.center.0.to_part(scale.0) + extents.0, self.center.1.to_part(scale.1) + extents.1],
						tex: [0.0, 0.0], color: self.color.top_right().to_linear()},
					plus_y_color: self.color.top_left().to_linear(),
					plus_x_color: self.color.bottom_right().to_linear()
				}.as_quad().as_triangles().iter().map(|vertex| Vertex { pos: [vertex.pos[0] + offset.0, vertex.pos[1] + offset.1, self.z], color: vertex.color, tex: vertex.tex }), None);
			},
			Kind::Image (Image { ref texture, ref slice }) => {
				self.zone_id = Some(context.new_zone());
				
				let extents = (self.extents.0.to_part(scale.0) * viewport_scale.0, self.extents.1.to_part(scale.1) * viewport_scale.1);
				
				context.extend_zone(Rect {
					min: Vertex2D { pos: [self.center.0.to_part(scale.0) - extents.0, self.center.1.to_part(scale.1) - extents.1], 
						tex: [slice.x_min, slice.y_min], color: self.color.bottom_left().to_linear()},
					max: Vertex2D { pos: [self.center.0.to_part(scale.0) + extents.0, self.center.1.to_part(scale.1) + extents.1],
						tex: [slice.x_max, slice.y_max], color: self.color.top_right().to_linear()},
					plus_y_color: self.color.top_left().to_linear(),
					plus_x_color: self.color.bottom_right().to_linear()
				}.as_quad().as_triangles().iter().map(|vertex| Vertex { pos: [vertex.pos[0] + offset.0, vertex.pos[1] + offset.1, self.z], color: vertex.color, tex: vertex.tex }), Some(texture));
			},
			Kind::Text (Text { ref string }) => {
				self.zone_id = Some(context.new_zone());
				let ctxt = RenderingContext::new(&metrics);
				let style = Style::new();
				
				let color = self.color.solid();
				let shadow = Rgb::new(color.r() / 4, color.g() / 4, color.b() / 4).to_linear();
				let color = color.to_linear();
				
				let align = Align::Center;
				let width = metrics.advance(string.chars(), &StyleFlags::none()).total().expect("wtf?");
				
				let start = align.start_x(
					self.center.0.to_px(scale.0) - self.extents.0.to_px(scale.0), 
					self.center.0.to_px(scale.0) + self.extents.0.to_px(scale.0), 
					width as f32
				);
				
				// TODO: Positioning
				
				let y_center = self.center.1.to_px(scale.1);
				
				println!("y_center: {}, x_start: {}", y_center, start);
				
				for command in ctxt.render(start + 1.0, y_center - 5.5, string.chars(), &style, true, shadow).filter_map(|x| x).chain(
						ctxt.render(start, y_center - 4.5, string.chars(), &style, false, color).filter_map(|x| x)
					) {
					println!("{:?}", command);
					match command {
						Command::Char( ref draw_command ) => {
							// TODO: Scaled text
							let (quad, atlas) = draw_command.to_quad((scale.0, scale.1)); 
							
							context.extend_zone (
								quad
								.as_triangles()
								.iter()
								.map(|vertex| Vertex { pos: [vertex.pos[0] + offset.0, vertex.pos[1] + offset.1, self.z], color: vertex.color, tex: [vertex.tex[0] * 2.0 - 1.0, vertex.tex[1] * 2.0 - 1.0] }),
								Some(PAGES[atlas as usize])
							);
						},
						Command::CharDefault { .. } => panic!("Can't draw default chars"),
						Command::Rect { x, y, width, height } => {
							let (x, y) = (x * scale.0, y * scale.1);
							
							// TODO: Scaled text
							let (width, height) = (width * scale.0, height * scale.1);
							
							context.extend_zone (
								Rect::solid([x, y], [x + width, y + height], [1.0, 1.0, 1.0])
								.as_quad()
								.as_triangles()
								.iter()
								.map(|vertex| Vertex { pos: [vertex.pos[0] + offset.0, vertex.pos[1] + offset.1, self.z], color: vertex.color, tex: vertex.tex }), None
							);
						},
					}
				}
			},
			Kind::Baked (ref mut scene) => {
				let mut depth = 1.0;
				
				for (name, element) in &mut scene.elements {
					// TODO
					
					element.default.push_to(
						(self.center.0.to_part(scale.0) + offset.0, self.center.1.to_part(scale.1) + offset.1), 
						scale,
						(viewport_scale.0 * self.extents.0.to_part(scale.0), viewport_scale.1 * self.extents.1.to_part(scale.1)), context, metrics);
					depth /= 2.0;
				}
			},
			Kind::Import {..} => panic!("Tried to push an unbaked state to context"),
			Kind::Nodraw => ()
		}
	}
}

#[derive(Serialize)]
pub struct KindWrapper(pub Kind);

impl<'de> Deserialize<'de> for KindWrapper {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		deserializer.deserialize_map(KindVisitor)
	}
}

struct KindVisitor;
impl<'de> Visitor<'de> for KindVisitor {
	type Value = KindWrapper;
	
	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("a map of the structure {\"variant\": {/*map data*/}}")
	}
	
	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
		let key: String = map.next_key()?.ok_or_else( || A::Error::custom("Enum structure must have at least one key") )?;
		
		Ok(KindWrapper(match &key as &str {
			"Rect" => Kind::Rect,
			"Image" => Kind::Image(map.next_value()?),
			"Text" => Kind::Text(map.next_value()?),
			"Import" => unimplemented!(),
			"Baked" => Kind::Baked(map.next_value()?),
			"Nodraw" => Kind::Nodraw,
			key => Kind::Import {scene: key.to_owned(), with: map.next_value()? }
		}))
	}
}

#[derive(Serialize, Deserialize)]
struct Image {
	texture: String,
	slice: ScreenSlice
}

#[derive(Serialize, Deserialize)]
struct Text {
	string: String
}

#[derive(Serialize, Deserialize)]
pub enum Kind {
	Rect,
	Image(Image),
	//Text { buf: ChatBuf }
	Text(Text),
	Import { scene: String, with: HashMap<String, Value> },
	Baked(Scene),
	Nodraw
}

#[derive(Serialize, Deserialize)]
pub enum Coloring {
	Solid(Rgb),
	Corners {
		top_left: Rgb,
		top_right: Rgb,
		bottom_left: Rgb,
		bottom_right: Rgb
	}
}

impl Coloring {
	fn white() -> Self {
		Coloring::Solid(Rgb::new(255, 255, 255))
	}
	
	fn solid(&self) -> Rgb {
		match *self {
			Coloring::Solid(c) => c,
			Coloring::Corners { ref top_left, ref top_right, ref bottom_left, ref bottom_right } => {
				Rgb::new(
					(((top_left.r() as u32) + (top_right.r() as u32) + (bottom_left.r() as u32) + (bottom_right.r() as u32)) / 4) as u8,
					(((top_left.g() as u32) + (top_right.g() as u32) + (bottom_left.g() as u32) + (bottom_right.g() as u32)) / 4) as u8,
					(((top_left.b() as u32) + (top_right.b() as u32) + (bottom_left.b() as u32) + (bottom_right.b() as u32)) / 4) as u8
				)
			}
		}
	}
	
	fn bottom_left(&self) -> Rgb {
		match *self {
			Coloring::Solid(c) => c,
			Coloring::Corners { bottom_left, .. } => bottom_left
		}
	}
	
	fn bottom_right(&self) -> Rgb {
		match *self {
			Coloring::Solid(c) => c,
			Coloring::Corners { bottom_right, .. } => bottom_right
		}
	}
	
	fn top_left(&self) -> Rgb {
		match *self {
			Coloring::Solid(c) => c,
			Coloring::Corners { top_left, .. } => top_left
		}
	}
	
	fn top_right(&self) -> Rgb {
		match *self {
			Coloring::Solid(c) => c,
			Coloring::Corners { top_right, .. } => top_right
		}
	}
}