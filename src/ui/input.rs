#[derive(Serialize, Deserialize)]
struct InputState {
	name: String,
	center: (Lit, Lit),
	extents: (Lit, Lit),
	events: Vec<Handler>
}

#[derive(Serialize, Deserialize)]
struct Handler {
	event: Event,
	action: Action
}

#[derive(Serialize, Deserialize)]
enum Event {
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