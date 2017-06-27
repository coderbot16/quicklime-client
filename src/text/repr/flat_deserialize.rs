enum Field {
	Bold,
	Italic,
	Underlined,
	Strikethrough,
	Obfuscated,
	Color,
	Insertion,
	ClickEvent,
	HoverEvent,
	Extra,
	Text,
	Translate,
	With,
	Score,
	Selector,
	Keybind
}

impl Field {
	fn from_str(key: &str) -> Option<Self> {
		Some(match key {
			"bold" => Field::Bold,
			"italic" => Field::Italic,
			"underlined" => Field::Underlined,
			"strikethrough" => Field::Strikethrough,
			"obfuscated" => Field::Obfuscated,
			"color" => Field::Color,
			"insertion" => Field::Insertion,
			"clickEvent" => Field::ClickEvent,
			"hoverEvent" => Field::HoverEvent,
			"extra" => Field::Extra,
			"text" => Field::Text,
			"translate" => Field::Translate,
			"with" => Field::With,
			"score" => Field::Score,
			"selector" => Field::Selector,
			"keybind" => Field::Keybind,
			_ => return None
		})
	}
	
	fn to_str(&self) -> &'static str {
		match *self {
			Field::Bold => "bold",
			Field::Italic => "italic",
			Field::Underlined => "underlined",
			Field::Strikethrough => "strikethrough",
			Field::Obfuscated => "obfuscated",
			Field::Color => "color",
			Field::Insertion => "insertion",
			Field::ClickEvent => "clickEvent",
			Field::HoverEvent => "hoverEvent",
			Field::Extra => "extra",
			Field::Text => "text",
			Field::Translate => "translate",
			Field::With => "with",
			Field::Score => "score",
			Field::Selector => "selector",
			Field::Keybind => "keybind",
		}
	}
}

enum EventField {
	Action,
	Value
}

impl EventField {
	fn from_str(key: &str) -> Option<Self> {
		Some(match key {
			"action" => EventField::Action,
			"value" => EventField::Value,
			_ => return None
		})
	}
	
	fn to_str(&self) -> &'static str {
		match *self {
			EventField::Action => "action",
			EventField::Value => "value"
		}
	}
}

enum ScoreField {
	Name,
	Objective,
	Value
}

impl ScoreField {
	fn from_str(key: &str) -> Option<Self> {
		Some(match key {
			"name" => ScoreField::Name,
			"objective" => ScoreField::Objective,
			"value" => ScoreField::Value,
			_ => return None
		})
	}
	
	fn to_str(&self) -> &'static str {
		match *self {
			ScoreField::Name => "name",
			ScoreField::Objective => "objective",
			ScoreField::Value => "value"
		}
	}
}