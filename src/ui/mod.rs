pub mod lit;
pub mod render;
pub mod managed;
pub mod input;
pub mod replace;

use color::Rgb;
use gfx::Resources;
use render2d::{Vertex2D, Rect};
use resource::atlas::TextureSelection;
use serde::de::{Error, Deserializer, Deserialize, Visitor, MapAccess};
use serde_json::Value;
use std::borrow::Borrow;
use std::cmp::max;
use std::collections::HashMap;
use std::fmt;
use text::align::Align;
use text::metrics::Metrics;
use text::pages::get_page;
use text::render::{RenderingContext, Command};
use text::repr::plain::{PlainBuf, Iter};
use ui::input::{Input, InputEvent};
use ui::lit::Lit;
use ui::render::Context;
use ui::replace::IncompleteScene;

pub use self::render::Vertex as Vertex;

#[derive(Serialize, Deserialize, Default)]
pub struct Scene {
	#[serde(skip_serializing, skip_deserializing)]
	cursor: Option<(f32, f32)>,
	pub elements: HashMap<String, Element>,
	pub inputs: HashMap<String, Input>
}

impl Scene {
	/// Resolves all of the contained imports to incomplete scenes and preforms replacement of the parameters contained in the incomplete scenes.
	/// If an incomplete scene contains further imports, those will be handled properly as well.
	/// Failure to call this function or an error while resolving will result in a panic on a subsequent call to Element::push_to.
	pub fn bake_all(&mut self, scenes: &HashMap<String, IncompleteScene>) -> Result<(), replace::Error> {
		for value in self.elements.values_mut() {
			for state in &mut value.states {
				state.bake(scenes)?;
			}
			
			value.default.bake(scenes)?;
		}
		
		Ok(())
	}
	
	pub fn handle_event(&mut self, event: &InputEvent) {
		match event {
			_ => ()
		}
	}
	
	pub fn max_level(&self) -> u32 {
		max(
			self.elements.values().map(|elem| elem.max_level()).fold(0, |accum, val| max(accum, val)),
			self.inputs.values().map(|input| input.max_level()).fold(0, |accum, val| max(accum, val))
		)
	}
	
	/// Compute the value that is multiplied by the level to get a value between 0.0 and 1.0.
	pub fn z_stride(&self) -> f32 {
		println!("max_level+1: {}", self.max_level() + 1);
		
		1.0 / (self.max_level() + 1) as f32
	}
}

/// An element, which contains at least one state, the default state. An element may only be in one state at any time. 
#[derive(Serialize, Deserialize)]
pub struct Element {
	#[serde(skip_serializing, skip_deserializing)]
	pub current: Option<usize>,
	pub default: State,
	#[serde(default = "Vec::new")]
	pub states: Vec<State>
}

impl Element {
	/// Gets a reference to the current state of this element.
	fn state(&self) -> &State {
		match self.current {
			None => &self.default,
			Some(idx) => &self.states[idx]
		}
	}
	
	/// Gets a mutable reference to the current state of this element.
	fn state_mut(&mut self) -> &mut State {
		match self.current {
			None => &mut self.default,
			Some(idx) => &mut self.states[idx]
		}
	}
	
	/// Gets the maximum Z level of all of the states.
	fn max_level(&self) -> u32 {
		max(
			self.default.level,
			self.states.iter().map(|state| state.level).fold(0, |accum, val| max(accum, val))
		)
	}
}

/// A single state of an element. Contains all of the properties related to rendering of the element.
#[derive(Serialize, Deserialize)]
pub struct State {
	#[serde(default = "default_name")]
	pub name: String,
	pub center: (Lit, Lit),
	pub extents: (Lit, Lit),
	#[serde(default = "Coloring::white")]
	pub color: Coloring,
	pub kind: Kind,
	pub level: u32,
	#[serde(skip_serializing, skip_deserializing)]
	pub zone_id: Option<usize>,
}

fn default_name() -> String {
	"default".to_string()
}

impl State {
	/// Bakes this element if it is an Import, and recursively bakes it's children. Baking involves resolving import references and preforming replacement of parameters.
	pub fn bake(&mut self, scenes: &HashMap<String, IncompleteScene>) -> Result<(), replace::Error> {
		let replacement = if let Kind::Import { ref scene, ref with } = self.kind {
			let incomplete = scenes.get(scene).ok_or_else(|| replace::Error::SceneLookupFailed(scene.to_owned()))?;
			
			let mut complete = incomplete.complete(with)?;
			complete.bake_all(scenes)?;
			
			Some(Kind::Baked (complete))
		} else {
			None
		};
		
		if let Some(replacement) = replacement {
			self.kind = replacement;
		}
		
		Ok(())
	}
	
	/// Pushes the raw vertex data representing this element to a context.
	pub fn push_to<R>(&mut self, offset: (f32, f32, f32), scale: (f32, f32), viewport_scale: (f32, f32), z_stride: f32, context: &mut Context<R>, metrics: &Metrics) where R: Resources {
		let z_offset = offset.2 + (self.level as f32 * z_stride);
		// Subtract the level in unit form from 1, to properly transform into normalized depth. In level form, 1.0 is the closest, while 0.0 is the closest in normalized depth.
		let depth = 1.0 - z_offset;
		println!("z: offset={}, stride={}, depth={}", z_offset, z_stride, depth);
		assert!(depth >= 0.0, depth <= 1.0);
		
		match self.kind {
			Kind::Rect (Image { ref texture, ref slice }) => {
				println!("Rect {{ texture: {:?}, slice: {:?} }}", texture, slice);
				
				self.zone_id = Some(context.new_zone());
				
				let extents = (self.extents.0.to_part(scale.0) * viewport_scale.0, self.extents.1.to_part(scale.1) * viewport_scale.1);
				let min = [slice.min[0].to_part(0.0), slice.min[1].to_part(0.0)];
				
				let rect = Rect {
					min: Vertex2D { pos: [self.center.0.to_part(scale.0) - extents.0, self.center.1.to_part(scale.1) - extents.1], 
						tex: min, color: self.color.bottom_left().to_linear()},
					max: Vertex2D { pos: [self.center.0.to_part(scale.0) + extents.0, self.center.1.to_part(scale.1) + extents.1],
						tex: [min[0] + slice.size[0].to_part(0.0), min[1] + slice.size[1].to_part(0.0)], color: self.color.top_right().to_linear()},
					plus_y_color: self.color.top_left().to_linear(),
					plus_x_color: self.color.bottom_right().to_linear()
				};
				
				let tris = 
					rect
					.as_quad()
					.as_triangles();
					
				let vertices = 
					tris	
					.iter()
					.map(|vertex| Vertex { pos: [vertex.pos[0] + offset.0, vertex.pos[1] + offset.1, depth], color: vertex.color, tex: vertex.tex });
				
				context.extend_zone(vertices, texture.as_ref().map(Borrow::borrow));
			},
			Kind::Text (Text { ref string, shadow, ref align }) => {
				println!("{:?}", string);
				
				self.zone_id = Some(context.new_zone());
				let ctxt = RenderingContext::new(&metrics);
				
				let width = metrics.advance(string.iter()).total().expect("string contained characters outside BMP");
				let start = align.start_x(
					self.center.0.to_px(scale.0) * viewport_scale.0 - self.extents.0.to_px(scale.0) * viewport_scale.0, 
					self.center.0.to_px(scale.0) * viewport_scale.0 + self.extents.0.to_px(scale.0) * viewport_scale.0, 
					width as f32
				);
				
				let y_center = self.center.1.to_px(scale.1);
				
				let color = self.color.solid();
				let shadow_iter = ctxt.render(start, y_center - 4.5, if shadow {string.iter()} else {Iter::empty()}, true, color).filter_map(|x| x);
				
				for command in shadow_iter.chain(
						ctxt.render(start, y_center - 4.5, string.iter(), false, color).filter_map(|x| x)
					) {
						
						// TODO: Proper Z values for text with shadow.
					println!("{:?}", command);
					match command {
						Command::Char( ref draw_command ) => {
							// TODO: Scaled text
							let (quad, atlas) = draw_command.to_quad((scale.0, scale.1)); 
							
							context.extend_zone (
								quad
								.as_triangles()
								.iter()
								.map(|vertex| Vertex { pos: [vertex.pos[0] + offset.0, vertex.pos[1] + offset.1, depth], color: vertex.color, tex: vertex.tex }),
								Some(get_page(atlas))
							);
						},
						Command::Rect { x, y, width, height, color } => {
							let (x, y) = (x * scale.0, y * scale.1);
							
							// TODO: Scaled text
							let (width, height) = (width * scale.0, height * scale.1);
							
							context.extend_zone (
								Rect::solid([x, y], [x + width, y + height], color)
								.as_quad()
								.as_triangles()
								.iter()
								.map(|vertex| Vertex { pos: [vertex.pos[0] + offset.0, vertex.pos[1] + offset.1, depth - z_stride/2.0], color: vertex.color, tex: vertex.tex }), None
							);
						},
					}
				}
			},
			Kind::Baked (ref mut scene) => {
				println!("BakedScene");
				let z_stride = z_stride * scene.z_stride();
				let offset = (self.center.0.to_part(scale.0) + offset.0, self.center.1.to_part(scale.1) + offset.1, z_offset);
				let viewport_scale = (viewport_scale.0 * self.extents.0.to_part(scale.0), viewport_scale.1 * self.extents.1.to_part(scale.1));
				
				
				for element in scene.elements.values_mut() {
					// TODO: Obey coloring.
					
					element.default.push_to(offset, scale, viewport_scale, z_stride, context, metrics);
				}
			},
			Kind::Import {..} => panic!("Tried to push an unbaked state to context, did you forget to check the return value of Scene::bake_all?"),
			Kind::Nodraw => ()
		}
	}
	
	/*pub fn z_span(&self) -> u32 {
		match self.kind {
			Kind::Rect (Image { .. }) => 1,
			Kind::Text (Text { shadow, .. }) => 2 + if shadow {2} else {0},
			Kind::Baked (ref mut scene) => scene.z_span(),
			Kind::Import {..} => panic!("Tried to get the width of an unbaked state, did you forget to check the return value of Scene::bake_all?"),
			Kind::Nodraw => 0
		}
	}*/
}

impl<'de> Deserialize<'de> for Kind {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		deserializer.deserialize_map(KindVisitor)
	}
}

struct KindVisitor;
impl<'de> Visitor<'de> for KindVisitor {
	type Value = Kind;
	
	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("a map of the structure {\"variant\": {/*map data*/}} or \"variant\"")
	}
	
	fn visit_str<E>(self, key: &str) -> Result<Self::Value, E> where E: Error {
		Ok(match key {
			"rect" => Kind::Rect(Image { texture: None, slice: TextureSelection::identity() }),
			"text" => return Err(E::custom("Text element must have associated data")),
			"import" => unimplemented!(),
			"baked" => return Err(E::custom("Baked element must have associated data")),
			"nodraw" => Kind::Nodraw,
			key => Kind::Import {scene: key.to_owned(), with: HashMap::new() }
		})
	}
	
	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
		let key: String = map.next_key()?.ok_or_else( || A::Error::custom("Enum structure must have at least one key") )?;
		
		// This custom deserialization implementation allows Import statements to be written like builtin kinds.
		
		Ok(match &key as &str {
			"rect" => Kind::Rect(map.next_value()?),
			"text" => Kind::Text(map.next_value()?),
			"import" => unimplemented!(),
			"baked" => Kind::Baked(map.next_value()?),
			"nodraw" => Kind::Nodraw,
			key => Kind::Import {scene: key.to_owned(), with: map.next_value()? }
		})
	}
}

#[derive(Serialize, Deserialize)]
pub struct Image {
	#[serde(default = "default_texture")]
	texture: Option<String>,
	#[serde(default = "TextureSelection::identity")]
	slice: TextureSelection
}

#[derive(Serialize, Deserialize)]
pub struct Text {
	string: PlainBuf,
	#[serde(default = "default_shadow")]
	shadow: bool,
	#[serde(default = "default_align")]
	align: Align
}

fn default_shadow() -> bool {
	false
}

fn default_texture() -> Option<String> {
	None
}

fn default_align() -> Align {
	Align::Center
}

#[derive(Serialize)]
pub enum Kind {
	/// A colored rectangle that can be textured.
	Rect(Image),
	/// A piece of text.
	Text(Text), // TODO: Formatting with ChatBuf
	/// An unresolved import, as found in unbaked scenes.
	Import { scene: String, with: HashMap<String, Value> },
	/// A baked scene.
	Baked(Scene),
	/// Nothing - results in no vertices. Can be used to "hide" things.
	Nodraw
}

/// A coloring of an element. May be a solid color, all vertices are colored equally, or a color varying at each corner.
#[derive(Serialize, Deserialize)]
pub enum Coloring {
	#[serde(rename="solid")]
	Solid(Rgb),
	#[serde(rename="corners")]
	Corners {
		top_left: Rgb,
		top_right: Rgb,
		bottom_left: Rgb,
		bottom_right: Rgb
	}
}

impl Coloring {
	/// Returns the white color, #ffffff
	fn white() -> Self {
		Coloring::Solid(Rgb::new(255, 255, 255))
	}
	
	/// If this a solid coloring, returns the color. Otherwise, takes the average of the 4 colors.
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