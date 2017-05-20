//! This is the implementation of the qlc GUI.
//! There are several distinct 

use std::mem;
use glutin::{ElementState, MouseButton};

type Identifier = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenSlice {
	pub x_min: f32,
	pub x_max: f32,
	pub y_min: f32,
	pub y_max: f32
}

impl ScreenSlice {
	fn intersect(&self, x: f32, y: f32) -> bool {
		x >= self.x_min && x <= self.x_max && y >= self.y_min && y <= self.y_max
	}
	
	pub fn as_triangles(&self, depth: f32) -> [[f32; 4]; 6] {
		[
			[self.x_min, self.y_min, depth, 1.0],
			[self.x_min, self.y_max, depth, 1.0],
			[self.x_max, self.y_max, depth, 1.0],
			[self.x_min, self.y_min, depth, 1.0],
			[self.x_max, self.y_min, depth, 1.0],
			[self.x_max, self.y_max, depth, 1.0],
		]
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Area {
	pub id: Identifier,
	pub slice: ScreenSlice
}

#[derive(Debug)]
enum Message {
	MouseEnter { x: f32, y: f32 },
	MouseLeave,
	MouseMove { x: f32, y: f32 },
	MouseInput { state: ElementState, button: MouseButton },
	MouseWheel,
	KeyboardInput
}

enum Action {
	LeaveEnter { left: String },
	Move,
	Enter
}

pub struct Screen {
	interact: Vec<Area>,
	latest_position: Option<(f32, f32)>,
	latest_interact: Option<Identifier>,
}

impl Screen {
	pub fn new() -> Self {
		Screen {
			interact: Vec::new(),
			latest_position: None,
			latest_interact: None,
		}
	}
	
	fn send(&self, id: &str, msg: Message) {
		println!("{} <- {:?}", id, msg);
	}
	
	pub fn add_interact_area(&mut self, area: Area) {
		self.interact.push(area);
		
		if let Some((x, y)) = self.latest_position {
			self.position(x, y);
		}
	}
	
	/// This function should be called when the position of the mouse or the interact list changes. 
	/// It is automatically called when an element is added to the interact list, but needs to be called on every recieved mouse event.
	pub fn position(&mut self, x: f32, y: f32) {
		self.latest_position = Some((x, y));
		
		// Traverse the list from the tail, so that the most recent added elements at the top of the screen take precedence.
		let mut range = 0..self.interact.len();
		while let Some(index) = range.next_back() {
			let area = &self.interact[index];
			
			if area.slice.intersect(x, y) {
				let action = if let Some(ref mut id) = self.latest_interact {
					// If there was already a latest interact, we need to make sure it isn't still in the same area.
					
					if &area.id != id {
						// Replace the value of the latest_interact with this area while receiving the last area into the other variable.
						let mut other = area.id.clone();
						mem::swap(id, &mut other);
						
						Action::LeaveEnter { left: other }
					} else {
						Action::Move
					}
				} else {
					self.latest_interact = Some(area.id.clone());
					Action::Enter
				};
				
				match action {
					Action::LeaveEnter {left} => {
						self.send(&left, Message::MouseLeave); 
						self.send(&area.id, Message::MouseEnter { x: x, y: y })
					},
					Action::Move => self.send(&area.id, Message::MouseMove { x: x, y: y }),
					Action::Enter => self.send(&area.id, Message::MouseEnter { x: x, y: y } )
				};
				
				return
			}
		}
		
		//println!("MISS: {}, {}", x, y);
		if let Some(id) = self.latest_interact.take() {
			self.send(&id, Message::MouseLeave);
		}
	}
	
	pub fn mouse_click(&self, state: ElementState, button: MouseButton) {
		let target = if let Some(ref id) = self.latest_interact {
			id
		} else {
			return
		};
		
		self.send(target, Message::MouseInput { state: state, button: button });
	}
}