mod lit;

use std::rc::Rc;
use std::collections::HashMap;
use text::chat::Container;
use render2d::Color;
use ui::lit::Lit;

struct Element {
	current: Rc<State>,
	default: Rc<State>,
	others: HashMap<String, Rc<State>>
}

struct State {
	center: (Lit, Lit),
	bottom_left: (Lit, Lit),
	top_right: (Lit, Lit),
	color: Coloring,
	kind: Option<Kind>
}

enum Kind {
	Rect,
	Image { path: String },
	Text { container: Container }
}

enum Coloring {
	Solid(Color),
	Corners {
		top_left: Color,
		top_right: Color,
		bottom_left: Color,
		bottom_right: Color
	}
}