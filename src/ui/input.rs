use glutin::{WindowEvent, ElementState, MouseScrollDelta, TouchPhase, MouseButton};
use std::cmp::max;
use std::path::PathBuf;
use ui::lit::Lit;

#[derive(Serialize, Deserialize)]
pub struct Input {
	#[serde(skip_serializing, skip_deserializing)]
	pub current: Option<usize>,
	default: InputState,
	states: Vec<InputState>
}

impl Input {
	fn state(&self) -> &InputState {
		match self.current {
			Some(idx) => &self.states[idx],
			None => &self.default
		}
	}
	
	pub fn hit(&self, pos: (f32, f32), scale: (f32, f32)) -> bool {
		let state = self.state();
		let center = (state.center.0.to_part(scale.0), state.center.1.to_part(scale.1));
		let extents = (state.extents.0.to_part(scale.0), state.extents.1.to_part(scale.1));
		
		// Test if the distance from the center on each axis is less than or equal to the extents.
		(pos.0 - center.0).abs() <= extents.0 && (pos.1 - center.1).abs() <= extents.1
	}
	
	/// Gets the maximum Z level of all of the states.
	pub fn max_level(&self) -> u32 {
		self.states.iter().map(|state| state.level).fold(0, |accum, val| max(accum, val))
	}
}

#[derive(Serialize, Deserialize)]
pub struct InputState {
	name: String,
	center: (Lit, Lit),
	extents: (Lit, Lit),
	level: u32,
	events: Vec<Handler>
}

#[derive(Serialize, Deserialize)]
struct Handler {
	event: AEvent,
	action: Action
}

#[derive(Serialize, Deserialize)]
enum AEvent {
	Enter,
	Leave,
	LeftClick,
	RightClick,
	MiddleClick
}

#[derive(Serialize, Deserialize)]
enum Action {
	SetElementState { element: String, state: String },
	SetInputState { input: String, state: String }
}

pub enum InputEvent {
	DroppedFile(PathBuf),
	ReceivedCharacter(char),
	// TODO: KbdInput
	MouseMoved(f32, f32),
	MouseEntered,
	MouseLeft,
	MouseWheel(MouseScrollDelta, TouchPhase),
	MouseInput(ElementState, MouseButton),
	// TODO: Touchpad?
}

impl InputEvent {
	pub fn from_glutin(glutin: WindowEvent, scale: (f32, f32)) -> Option<InputEvent> {
		Some(match glutin {
			WindowEvent::DroppedFile(buf) => InputEvent::DroppedFile(buf),
			WindowEvent::ReceivedCharacter(char) => InputEvent::ReceivedCharacter(char),
			WindowEvent::MouseMoved(x, y) => InputEvent::MouseMoved(x as f32 * scale.0, y as f32 * scale.1),
			WindowEvent::MouseEntered => InputEvent::MouseEntered,
			WindowEvent::MouseLeft => InputEvent::MouseLeft,
			WindowEvent::MouseWheel(delta, phase) => InputEvent::MouseWheel(delta, phase),
			WindowEvent::MouseInput(state, button) => InputEvent::MouseInput(state, button),
			_ => return None
		})
	}
}