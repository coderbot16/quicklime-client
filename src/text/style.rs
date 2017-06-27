use color::Rgb;
use std::iter;

const BOLD: u8 = 1;
const UNDERLINE: u8 = 2;
const ITALIC: u8 = 4;
const STRIKETHROUGH: u8 = 8;
const OBFUSCATE: u8 = 16;

/// Packs the 5 style properties into a single byte.
/// Format: [UNUSEDx3][RANDOM][STRIKE][ITALIC][UNDERLINE][BOLD]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[must_use]
pub struct StyleFlags(u8);

impl StyleFlags {
	pub fn none() -> Self {
		StyleFlags(0)
	}

	pub fn all() -> Self {
		StyleFlags(BOLD | UNDERLINE | ITALIC | STRIKETHROUGH | OBFUSCATE)
	}
	
	pub fn set_bold(self, bold: bool) -> Self {
		StyleFlags((self.0 & !BOLD) | (if bold {BOLD} else {0}))
	}

	pub fn bold(&self) -> bool {
		(self.0 & BOLD) == BOLD
	}
	
	pub fn set_underline(self, underline: bool) -> Self {
		StyleFlags((self.0 & !UNDERLINE) | (if underline {UNDERLINE} else {0}))
	}
	
	pub fn underline(&self) -> bool {
		(self.0 & UNDERLINE) == UNDERLINE
	}
	
	pub fn set_italic(self, italic: bool) -> Self {
		StyleFlags((self.0 & !ITALIC) | (if italic {ITALIC} else {0}))
	}
	
	pub fn italic(&self) -> bool {
		(self.0 & ITALIC) == ITALIC
	}
	
	pub fn set_strikethrough(self, strikethrough: bool) -> Self {
		StyleFlags((self.0 & !STRIKETHROUGH) | (if strikethrough {STRIKETHROUGH} else {0}))
	}
	
	pub fn strikethrough(&self) -> bool {
		(self.0 & STRIKETHROUGH) == STRIKETHROUGH
	}
	
	pub fn set_obfuscate(self, obfuscate: bool) -> Self{
		StyleFlags((self.0 & !OBFUSCATE) | (if obfuscate {OBFUSCATE} else {0}))
	}
	
	pub fn obfuscate(&self) -> bool {
		(self.0 & OBFUSCATE) == OBFUSCATE
	}
	
	pub fn or(&self, other: Self) -> Self {
		StyleFlags(self.0 | other.0)
	}
	
	pub fn process(self, cmd: &StyleCommand) -> Self {
		match *cmd {
			StyleCommand::Color(_)		=> Self::none(),
			StyleCommand::Reset 		=> Self::none(),
			StyleCommand::Bold			=> self.set_bold(true),
			StyleCommand::Underline 	=> self.set_underline(true),
			StyleCommand::Italic 		=> self.set_italic(true),
			StyleCommand::Strikethrough => self.set_strikethrough(true),
			StyleCommand::Obfuscate 	=> self.set_obfuscate(true),
		}
	}
	
	pub fn commands(&self) -> Commands {
		Commands {
			flags: *self,
			flag: 0
		}
	}
	
	pub fn delta_commands(&self, other: StyleFlags) -> DeltaCommands {
		DeltaCommands {
			flags: other,
			other: *self,
			flag: 0
		}
	}
}

pub struct Commands {
	flags: StyleFlags,
	flag: u8
}

impl Commands {
	fn try_next(&mut self) -> Option<Option<StyleCommand>> {
		if self.flag == 6 {return None};
		self.flag += 1;
		
		Some(match self.flag {
			1 => if self.flags.bold() 			{Some(StyleCommand::Bold)} else {None},
			2 => if self.flags.underline() 		{Some(StyleCommand::Underline)} else {None},
			3 => if self.flags.italic() 		{Some(StyleCommand::Italic)} else {None},
			4 => if self.flags.strikethrough() 	{Some(StyleCommand::Strikethrough)} else {None},
			5 => if self.flags.obfuscate() 		{Some(StyleCommand::Obfuscate)} else {None},
			_ => unreachable!()
		})
	}
}

impl Iterator for Commands {
	type Item = StyleCommand;
	
	fn next(&mut self) -> Option<Self::Item> {
		while let Some(result) = self.try_next() {
			if result.is_some() {
				return result
			}
		}
		
		None
	}
}

pub struct DeltaCommands {
	flags: StyleFlags,
	other: StyleFlags,
	flag: u8
}

impl DeltaCommands {
	fn try_next(&mut self) -> Option<Option<StyleCommand>> {
		if self.flag == 6 {return None};
		self.flag += 1;
		
		Some(match self.flag {
			1 => if !self.other.bold()          && self.flags.bold() 			{Some(StyleCommand::Bold)} else {None},
			2 => if !self.other.underline()     && self.flags.underline() 		{Some(StyleCommand::Underline)} else {None},
			3 => if !self.other.italic()        && self.flags.italic() 			{Some(StyleCommand::Italic)} else {None},
			4 => if !self.other.strikethrough() && self.flags.strikethrough() 	{Some(StyleCommand::Strikethrough)} else {None},
			5 => if !self.other.obfuscate()     && self.flags.obfuscate() 		{Some(StyleCommand::Obfuscate)} else {None},
			_ => unreachable!()
		})
	}
}

impl Iterator for DeltaCommands {
	type Item = StyleCommand;
	
	fn next(&mut self) -> Option<Self::Item> {
		while let Some(result) = self.try_next() {
			if result.is_some() {
				return result
			}
		}
		
		None
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Style {
	pub color: Color,
	/// The style of the text.
	pub flags: StyleFlags,
}

impl Style {
	pub fn new() -> Self {
		Style {
			flags: StyleFlags::none(),
			color: Color::Default
		}
	}
	
	pub fn process(&mut self, cmd: &StyleCommand) {
		self.flags = self.flags.process(&cmd);
		
		match *cmd {
			StyleCommand::Reset => self.color = Color::Default,
			StyleCommand::Color(color) => self.color = Color::Palette(color),
			_ => ()
		}
	}
	
	pub fn will_reset(&self, other: Self) -> bool {
		// If any style flags are unset, we have to reencode the color as well.
		self.color != other.color || self.flags.or(other.flags) != other.flags
	}
	
	pub fn transition(&self, other: Style) -> Transition {
		if *self == other {
			Transition::None
		} else if self.will_reset(other) {
			Transition::Reset(iter::once(other.color.command()).chain(other.flags.commands()))
		} else {
			/*for command in self.flags.delta_commands(other.flags).filter_map(|x| x) {
				self.write_command(command)?;
			}*/
			
			unimplemented!()
		}
	}
}

pub enum Transition {
	Reset(iter::Chain<iter::Once<StyleCommand>, Commands>),
	Delta(DeltaCommands),
	None
}

impl Iterator for Transition {
	type Item = StyleCommand;
	
	fn next(&mut self) -> Option<Self::Item> {
		match *self {
			Transition::Reset(ref mut iter) => iter.next(),
			Transition::Delta(ref mut delta) => delta.next(),
			Transition::None => None
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Color {
	Default,
	Palette(PaletteColor),
	//Rgb(u8, u8, u8)
}

impl Color {
	pub fn command(&self) -> StyleCommand {
		match *self {
			Color::Default => StyleCommand::Reset,
			Color::Palette(pal) => StyleCommand::Color(pal)
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PaletteColor {
	Black,
	DarkBlue,
	DarkGreen,
	DarkAqua,
	DarkRed,
	DarkPurple,
	Gold,
	Gray,
	DarkGray,
	Blue,
	Green,
	Aqua,
	Red,
	LightPurple,
	Yellow,
	White
}

impl PaletteColor {
	pub fn foreground(&self) -> Rgb {
		Rgb::from_rgb(match *self {
			PaletteColor::Black 		=> 0x000000,
			PaletteColor::DarkBlue 	=> 0x0000AA,
			PaletteColor::DarkGreen	=> 0x00AA00,
			PaletteColor::DarkAqua 	=> 0x00AAAA,
			PaletteColor::DarkRed 		=> 0xAA0000,
			PaletteColor::DarkPurple 	=> 0xAA00AA,
			PaletteColor::Gold 		=> 0xFFAA00,
			PaletteColor::Gray 		=> 0xAAAAAA,
			PaletteColor::DarkGray 	=> 0x555555,
			PaletteColor::Blue 		=> 0x5555FF,
			PaletteColor::Green 		=> 0x55FF55,
			PaletteColor::Aqua 		=> 0x55FFFF,
			PaletteColor::Red 			=> 0xFF5555,
			PaletteColor::LightPurple 	=> 0xFF55FF,
			PaletteColor::Yellow 		=> 0xFFFF55,
			PaletteColor::White 		=> 0xFFFFFF
		})
	}
	
	pub fn background(&self) -> Rgb {
		Rgb::from_rgb(match *self {
			PaletteColor::Black 		=> 0x000000,
			PaletteColor::DarkBlue 	=> 0x00002A,
			PaletteColor::DarkGreen	=> 0x002A00,
			PaletteColor::DarkAqua 	=> 0x002A2A,
			PaletteColor::DarkRed 		=> 0x2A0000,
			PaletteColor::DarkPurple 	=> 0x2A002A,
			PaletteColor::Gold 		=> 0x2A2A00,
			PaletteColor::Gray 		=> 0x2A2A2A,
			PaletteColor::DarkGray 	=> 0x151515,
			PaletteColor::Blue 		=> 0x15153F,
			PaletteColor::Green 		=> 0x153F15,
			PaletteColor::Aqua 		=> 0x153F3F,
			PaletteColor::Red 			=> 0x3F1515,
			PaletteColor::LightPurple 	=> 0x3F153F,
			PaletteColor::Yellow 		=> 0x3F3F15,
			PaletteColor::White 		=> 0x3F3F3F
		})
	}
	
	fn from_code(code: char) -> Option<Self> {
		Some(match code {
			'0' => PaletteColor::Black,
			'1' => PaletteColor::DarkBlue,
			'2' => PaletteColor::DarkGreen,
			'3' => PaletteColor::DarkAqua,
			'4' => PaletteColor::DarkRed,
			'5' => PaletteColor::DarkPurple,
			'6' => PaletteColor::Gold,
			'7' => PaletteColor::Gray,
			'8' => PaletteColor::DarkGray,
			'9' => PaletteColor::Blue,
			'a' => PaletteColor::Green,
			'b' => PaletteColor::Aqua,
			'c' => PaletteColor::Red,
			'd' => PaletteColor::LightPurple,
			'e' => PaletteColor::Yellow,
			'f' => PaletteColor::White,
			_ => return None
		})
	}
	
	fn as_code(&self) -> char {
		match *self {
			PaletteColor::Black 		=> '0',
			PaletteColor::DarkBlue 	=> '1',
			PaletteColor::DarkGreen	=> '2',
			PaletteColor::DarkAqua 	=> '3',
			PaletteColor::DarkRed 		=> '4',
			PaletteColor::DarkPurple 	=> '5',
			PaletteColor::Gold 		=> '6',
			PaletteColor::Gray 		=> '7',
			PaletteColor::DarkGray 	=> '8',
			PaletteColor::Blue 		=> '9',
			PaletteColor::Green 		=> 'a',
			PaletteColor::Aqua 		=> 'b',
			PaletteColor::Red 			=> 'c',
			PaletteColor::LightPurple 	=> 'd',
			PaletteColor::Yellow 		=> 'e',
			PaletteColor::White 		=> 'f'
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StyleCommand {
	Color(PaletteColor),
	Reset,
	Bold,
	Underline,
	Italic,
	Strikethrough,
	Obfuscate
}

impl StyleCommand {
	pub fn from_code(code: char) -> Option<Self> {
		Some(match code {
			'r' => StyleCommand::Reset,
			'l' => StyleCommand::Bold,
			'n' => StyleCommand::Underline,
			'o' => StyleCommand::Italic,
			'm' => StyleCommand::Strikethrough,
			'k' => StyleCommand::Obfuscate,
			code => return PaletteColor::from_code(code).map(StyleCommand::Color)
		})
	}
	
	pub fn as_code(&self) -> char {
		match *self {
			StyleCommand::Color(ref c)  => c.as_code(),
			StyleCommand::Reset 		=> 'r',
			StyleCommand::Bold 			=> 'l',
			StyleCommand::Underline 	=> 'n',
			StyleCommand::Italic 		=> 'o',
			StyleCommand::Strikethrough => 'm',
			StyleCommand::Obfuscate 	=> 'k'
		}
	}
	
	pub fn affected_flags(&self) -> StyleFlags {
		match *self {
			StyleCommand::Color(_)		=> StyleFlags::all(),
			StyleCommand::Reset 		=> StyleFlags::all(),
			StyleCommand::Bold			=> StyleFlags::none().set_bold(true),
			StyleCommand::Underline 	=> StyleFlags::none().set_underline(true),
			StyleCommand::Italic 		=> StyleFlags::none().set_italic(true),
			StyleCommand::Strikethrough => StyleFlags::none().set_strikethrough(true),
			StyleCommand::Obfuscate 	=> StyleFlags::none().set_obfuscate(true)
		}
	}
}