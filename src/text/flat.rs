use text::style::{StyleFlags, Style, StyleCommand, Color, PaletteColor};
use std::borrow::{Borrow, ToOwned};
use std::slice::Iter;

pub const MAX_INDUVIDUAL_LEN: usize = 65535;
pub const MAX_NESTING: Level = 127;
type Level = u8;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Mode {
	Level,
	Deeper,
	Shallower(Level)
}

#[derive(Debug, Copy, Clone)]
pub enum Kind {
	Text,
	Translate,
	// TODO: Are scores styled?
	ScoreName,
	ScoreObjective,
	ScoreValue,
	Selector
}

// The interaction is seperated from the text because they are unrelated for most purposes.

// COMPACTED

#[derive(Debug)]
pub struct ChatBuf {
	string: String,
	descriptors: Vec<Descriptor>,
	level: Level
}

impl ChatBuf {
	pub fn new() -> Self {
		println!("{}", ::std::mem::size_of::<Descriptor>());
		
		ChatBuf {
			string: String::new(),
			descriptors: Vec::new(),
			level: 0
		}
	}
	
	pub fn with_capacity(string: usize, descriptors: usize) -> Self {
		ChatBuf {
			string: String::with_capacity(string),
			descriptors: Vec::with_capacity(descriptors),
			level: 0
		}
	}
	
	// TODO: from_formatted[_lossy]
	
	pub fn from_formatted(s: &str, ampersand: bool) -> Self {
		let marker = if ampersand {'&'} else {'§'};
		
		// TODO: Better size calculation
		let mut buf = Self::new();
		
		let mut expect_code = false;
		let mut style = Style::new();
		let mut current_len = 0;
		
		for char in s.chars() {
			match char {
				code if expect_code => {
					expect_code = false;
					
					if current_len != 0 {
						buf.descriptors.push(Descriptor::new(current_len, 0, false, Kind::Text, style));
						current_len = 0;
					}
					
					let cmd = StyleCommand::from_code(code).unwrap_or(StyleCommand::Color(PaletteColor::White));
					
					style.flags.process(&cmd);
					
					if cmd == StyleCommand::Reset {
						style.color = Color::Default;
					} else if let StyleCommand::Color(color) = cmd {
						style.color = Color::Palette(color);
					}
				},
				marker => expect_code = true,
				char => {
					buf.string.push(char);
					current_len += 1;
				}
			}
		}
		
		if current_len != 0 {
			buf.descriptors.push(Descriptor::new(current_len, 0, false, Kind::Text, style));
		}
		
		buf
	}
	
	// TODO: from_json[_lenient]
	// TODO: into_formatted
	// TODO: into_json
	// TODO: into_json_flattened
	// TODO: into_json_bestfit
	
	pub fn capacity(&self) -> (usize, usize) {
		(self.string.capacity(), self.descriptors.capacity())
	}
	
	pub fn reserve(&mut self, string: usize, descriptors: usize) {
		self.string.reserve(string);
		self.descriptors.reserve(string);
	}
	
	pub fn reserve_exact(&mut self, string: usize, descriptors: usize) {
		self.string.reserve_exact(string);
		self.descriptors.reserve_exact(string);
	}
	
	pub fn shrink_to_fit(&mut self) {
		self.string.shrink_to_fit();
		self.descriptors.shrink_to_fit();
	}
	
	pub fn push(&mut self, component: Component) {
		match component.mode {
			Mode::Level => (),
			Mode::Deeper => self.level += 1,
			Mode::Shallower(reduction) => self.level -= ::std::cmp::min(self.level, reduction)
		};
		
		if self.level > MAX_NESTING {
			panic!("Nesting too deep! Chat components may only be nested to level {}", MAX_NESTING);
		}
		
		self.string.push_str(component.text);
		self.descriptors.push(Descriptor::new(component.text.len(), self.level, component.meta, component.kind, component.style));
	}
	
	// TODO: Pop, Truncate
	
	pub fn components(&self) -> Components {
		Components {
			head: &self.string,
			descriptors: self.descriptors.iter(),
			level: 0
		}
	}
}

pub struct Components<'a> {
	head: &'a str,
	descriptors: Iter<'a, Descriptor>,
	level: Level
}

impl<'a> Iterator for Components<'a> {
	type Item = Component<'a>;
	
	fn next(&mut self) -> Option<Self::Item> {
		self.descriptors.next().map(|descriptor| {
			let (part, rest) = self.head.split_at(descriptor.len as usize);
			self.head = rest;
			
			let mode = if self.level > descriptor.level() {
				Mode::Shallower(descriptor.level() - self.level)
			} else if self.level == descriptor.level() {
				Mode::Level
			} else {
				Mode::Deeper
			};
			
			self.level = descriptor.level();
			
			Component {
				text: part,
				kind: descriptor.kind,
				style: descriptor.style,
				mode: mode,
				meta: descriptor.meta()
			}
		})
	}
}

// 8 bytes
#[derive(Debug)]
struct Descriptor {
	len: u16,
	lvl: Level,
	kind: Kind,
	style: Style,
}

impl Descriptor {
	fn new(string_len: usize, level: Level, meta: bool, kind: Kind, style: Style) -> Self {
		Descriptor {
			len: string_len as u16,
			lvl: (level & 127) | if meta {128} else {0},
			kind: kind,
			style: style
		}
	}
	
	fn level(&self) -> Level {
		self.lvl & 127
	}
	
	fn meta(&self) -> bool {
		self.lvl >= 128
	}
}

// INTER

// TODO: Make sure that text is no longer than 2^16 - 1 
#[derive(Debug)]
pub struct Component<'a> {
	text: &'a str,
	kind: Kind,
	style: Style,
	mode: Mode,
	meta: bool
}

impl<'a> Component<'a> {
	pub fn new(text: &'a str, kind: Kind, style: Style, mode: Mode, meta: bool) -> Option<Self> {
		if text.len() > MAX_INDUVIDUAL_LEN {
			None
		} else {
			Some(Component {
				text: text,
				kind: kind,
				style: style,
				mode: mode,
				meta: meta
			})
		}
	}
	
	pub fn to_owned(&self) -> ComponentBuf {
		ComponentBuf {
			text: self.text.to_owned(),
			kind: self.kind,
			style: self.style,
			mode: self.mode,
			meta: self.meta
		}
	}
}

#[derive(Debug)]
pub struct ComponentBuf {
	text: String,
	kind: Kind,
	style: Style,
	mode: Mode,
	meta: bool
}

impl ComponentBuf {
	pub fn borrow(&self) -> Component {
		Component {
			text: self.text.borrow(),
			kind: self.kind,
			style: self.style,
			mode: self.mode,
			meta: self.meta
		}
	}
}