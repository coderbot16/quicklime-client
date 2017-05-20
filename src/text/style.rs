const BOLD: u8 = 1;
const UNDERLINE: u8 = 2;
const ITALIC: u8 = 4;
const STRIKETHROUGH: u8 = 8;
const OBFUSCATE: u8 = 16;

/// Packs the 5 style properties into a single byte.
/// Format: [UNUSEDx3][RANDOM][STRIKE][ITALIC][UNDERLINE][BOLD]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
	
	/// Returns a pair of StyleFlags: one describing with the result of the style command, and one containing a bitmask of the affected flags.
	pub fn process(self, cmd: &StyleCommand) -> (StyleFlags, StyleFlags) {
		match *cmd {
			StyleCommand::Color(_)		=> (Self::none(), 						Self::none()),
			StyleCommand::Reset 		=> (Self::none(), 						Self::none()),
			StyleCommand::Bold			=> (StyleFlags(self.0 | BOLD), 			Self::none().set_bold(true)),
			StyleCommand::Underline 	=> (StyleFlags(self.0 | UNDERLINE), 	Self::none().set_underline(true)),
			StyleCommand::Italic 		=> (StyleFlags(self.0 | ITALIC), 		Self::none().set_italic(true)),
			StyleCommand::Strikethrough => (StyleFlags(self.0 | STRIKETHROUGH), Self::none().set_strikethrough(true)),
			StyleCommand::Obfuscate 	=> (StyleFlags(self.0 | OBFUSCATE), 	Self::none().set_obfuscate(true))
		}
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
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Color {
	Default,
	Palette(PaletteColor),
	//Rgb(u8, u8, u8)
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
	fn foreground(&self) -> u32 {
		match *self {
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
		}
	}
	
	fn background(&self) -> u32 {
		match *self {
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
		}
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
}