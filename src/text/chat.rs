use text::style::Style;

pub enum Container {
	/// A single component.
	Single { component: Component, extra: Option<Vec<Container>> },
	/// Array of components. The first component acts as a parent for others, as if the first component was an object and the rest of the components were members of it's extra array.
	Array(Vec<Container>),
	/// A JSON primitive, rendered as a string.
	Primitive(String)
}

pub struct Component {
	segment: Segment,
	interact: Interaction
}

pub struct Segment {
	style: Style,
	payload: Payload
}

pub struct Interaction {
	insert: Option<String>,
	click: Option<ClickEvent>,
	hover: Option<HoverEvent>
}

// When decoding, try to decode in the following order:
// Text
// Translation
// Score
// Selector
// (return an error)

enum Payload {
	Text(String),
	Translation { translate: String, with: Vec<Container> },
	Score { name: String, objective: String, value: Option<String> },
	Selector(String)
}

enum ClickEvent {
	/// Same as OpenFile, but protocol must be http/https.
	OpenUrl(String),
	/// The string is actually a URL. Any protocol is accepted.
	OpenFile(String),
	/// Causes the client to send the string as a chat message.
	RunCommand(String),
	/// Removed in Minecraft 1.8. Opens a GUI scene with the twitch user info corresponding to the provided username.
	TwitchUserInfo(String),
	/// Replaces the contents of the chat box with the text.
	SuggestCommand(String),
	/// Only used in books, changes to the page.
	ChangePage(f64),
	Unsupported { action: String, value: String }
}

enum HoverEvent {
	ShowText(Box<Container>),
	/// ItemStack NBT data in Mojangson format.
	ShowItem(String),
	/// Entity NBT data in Mojangson format. Only uses 3 values: id (Entity UUID), type (string, minecraft:whatever), and name (the entity's custom name).
	ShowEntity(String),
	/// String is the id of the achievement.
	ShowAchievement(String),
	Unsupported { action: String, value: String }
}