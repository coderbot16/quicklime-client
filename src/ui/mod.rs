pub mod lit;

use input::ScreenSlice;
use std::rc::Rc;
use std::collections::HashMap;
use text::flat::ChatBuf;
use render2d::{Vertex2D, Color, Rect, Quad};
use ui::lit::Lit;

#[derive(Serialize, Deserialize)]
pub struct Scene {
	pub elements: HashMap<String, Element>
}

impl Scene {
	pub fn new() -> Self {
		Scene {
			elements: HashMap::new()
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct Element {
	#[serde(skip_serializing, skip_deserializing)]
	pub current: Option<usize>,
	pub default: State,
	pub states: Vec<State>
}

#[derive(Serialize, Deserialize)]
pub struct State {
	pub name: String,
	pub center: (Lit, Lit),
	pub extents: (Lit, Lit),
	pub color: Coloring,
	pub kind: Kind
}

impl State {
	fn try_to_quad(&self, scale: (f32, f32)) -> Option<Quad> {
		match self.kind {
			Kind::Rect => {
				Some(Rect {
					min: Vertex2D { pos: [self.center.0.to_part(scale.0) - self.extents.0.to_part(scale.0), self.center.1.to_part(scale.1) - self.extents.1.to_part(scale.1)], 
						tex: [0.0, 0.0], color: self.color.bottom_left()},
					max: Vertex2D { pos: [self.center.0.to_part(scale.0) + self.extents.0.to_part(scale.0), self.center.1.to_part(scale.1) + self.extents.1.to_part(scale.1)],
						tex: [0.0, 0.0], color: self.color.top_right()},
					plus_y_color: self.color.top_left(),
					plus_x_color: self.color.bottom_right()
				}.as_quad())
			},
			Kind::Image { ref slice, .. } => {
				Some(Rect {
					min: Vertex2D { pos: [self.center.0.to_part(scale.0) - self.extents.0.to_part(scale.0), self.center.1.to_part(scale.1) - self.extents.1.to_part(scale.1)], 
						tex: [slice.x_min, slice.y_min], color: self.color.bottom_left()},
					max: Vertex2D { pos: [self.center.0.to_part(scale.0) + self.extents.0.to_part(scale.0), self.center.1.to_part(scale.1) + self.extents.1.to_part(scale.1)],
						tex: [slice.x_max, slice.y_max], color: self.color.top_right()},
					plus_y_color: self.color.top_left(),
					plus_x_color: self.color.bottom_right()
				}.as_quad())
			},
			Kind::Text {..} => None,
			Kind::Nodraw => None
		}
	}
}

#[derive(Serialize, Deserialize)]
pub enum Kind {
	Rect,
	Image { path: String, slice: ScreenSlice },
	//Text { buf: ChatBuf }
	Text { string: String },
	Nodraw
}

#[derive(Serialize, Deserialize)]
pub enum Coloring {
	Solid(Color),
	Corners {
		top_left: Color,
		top_right: Color,
		bottom_left: Color,
		bottom_right: Color
	}
}

impl Coloring {
	fn bottom_left(&self) -> Color {
		match *self {
			Coloring::Solid(c) => c,
			Coloring::Corners { bottom_left, .. } => bottom_left
		}
	}
	
	fn bottom_right(&self) -> Color {
		match *self {
			Coloring::Solid(c) => c,
			Coloring::Corners { bottom_right, .. } => bottom_right
		}
	}
	
	fn top_left(&self) -> Color {
		match *self {
			Coloring::Solid(c) => c,
			Coloring::Corners { top_left, .. } => top_left
		}
	}
	
	fn top_right(&self) -> Color {
		match *self {
			Coloring::Solid(c) => c,
			Coloring::Corners { top_right, .. } => top_right
		}
	}
}