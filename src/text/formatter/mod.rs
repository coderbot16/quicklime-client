mod transform;
use std::fmt::{self, Formatter, Display};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Kind {
	Bool,
	HexHashCode,
	String,
	Unicode,
	// NUMERIC
	Decimal,
	Octal,
	Hex,
	// NUMERIC-FP
	CompSciNot,
	Float,
	SciNot,
	Hexfloat,
	// TIME
	Time(TimeKind),
	// ESCAPE
	Percent,
	Newline
}

impl Kind {
	fn from_character(character: char, other: Option<char>) -> Option<(Self, bool)> {
		Some(match character {
			'b' => (Kind::Bool, 		false),
			'B' => (Kind::Bool, 		true),
			'h' => (Kind::HexHashCode, 	false),
			'H' => (Kind::HexHashCode, 	true),
			's' => (Kind::String, 		false),
			'S' => (Kind::String, 		true),
			'c' => (Kind::Unicode, 		false),
			'C' => (Kind::Unicode, 		true),
			'd' => (Kind::Decimal, 		false),
			'o' => (Kind::Octal, 		false),
			'x' => (Kind::Hex, 			false),
			'X' => (Kind::Hex, 			true),
			'e' => (Kind::CompSciNot, 	false),
			'E' => (Kind::CompSciNot, 	true),
			'f' => (Kind::Float, 		false),
			'g' => (Kind::SciNot, 		false),
			'G' => (Kind::SciNot, 		true),
			'a' => (Kind::Hexfloat, 	false),
			'A' => (Kind::Hexfloat, 	true),
			't' => return other.and_then(TimeKind::from_character).map(|tk| (Kind::Time(tk), false)),
			'T' => return other.and_then(TimeKind::from_character).map(|tk| (Kind::Time(tk), true)),
			'%' => (Kind::Percent, 		false),
			'n' => (Kind::Newline, 		false),
			_ => return None
		})
	}
	
	pub fn character(&self, upper: bool) -> char {
		match *self {
			Kind::Bool 			=> if upper {'B'} else {'b'},
			Kind::HexHashCode 	=> if upper {'H'} else {'h'},
			Kind::String 		=> if upper {'S'} else {'s'},
			Kind::Unicode 		=> if upper {'C'} else {'c'},
			Kind::Decimal 		=> 'd',
			Kind::Octal 		=> 'o',
			Kind::Hex 			=> if upper {'X'} else {'x'},
			Kind::CompSciNot 	=> if upper {'E'} else {'e'},
			Kind::Float 		=> 'f',
			Kind::SciNot 		=> if upper {'G'} else {'g'},
			Kind::Hexfloat 		=> if upper {'A'} else {'a'},
			Kind::Time(_) 		=> if upper {'T'} else {'t'},
			Kind::Percent 		=> '%',
			Kind::Newline 		=> 'n'
		}
	}
	
	fn second_character(&self) -> Option<char> {
		match *self {
			Kind::Time(tk) => Some(tk.character()),
			_ => None
		}
	}
	
	fn honors_uppercase(&self) -> bool {
		match *self {
			Kind::Bool 			=> true,
			Kind::HexHashCode 	=> true,
			Kind::String 		=> true,
			Kind::Unicode 		=> true,
			Kind::Decimal 		=> false,
			Kind::Octal 		=> false,
			Kind::Hex 			=> true,
			Kind::SciNot 		=> true,
			Kind::Float 		=> false,
			Kind::CompSciNot 	=> true,
			Kind::Hexfloat 		=> true,
			Kind::Time(_) 		=> true,
			Kind::Percent 		=> false,
			Kind::Newline 		=> false
		}
	}
}

/// Unless otherwise specified, each of these is padded
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TimeKind {
	/// Hour of the 24 hour clock. Padded with zeros if neccesary (00 - 23)
	Hour24,
	/// Hour of the 12 hour clock. Padded with zeros if neccesary (01 - 12)
	Hour12,
	/// Hour of the 24 hour clock (0 - 23).
	UnpaddedHour24,
	/// Hour of the 12 hour clock (0 - 12).
	UnpaddedHour12,
	Minute,
	Second,
	Milli,
	Nano,
	/// Marker for morning or afternoon, such as AM/PM.
	Marker,
	TzOffset,
	TzAbbrev,
	EpochSecond,
	EpochMilli
}

impl TimeKind {
	fn from_character(character: char) -> Option<Self> {
		Some(match character {
			'H' => TimeKind::Hour24,
			'I' => TimeKind::Hour12,
			'k' => TimeKind::UnpaddedHour24,
			'l' => TimeKind::UnpaddedHour12,
			'M' => TimeKind::Minute,
			'S' => TimeKind::Second,
			'L' => TimeKind::Milli,
			'N' => TimeKind::Nano,
			'p' => TimeKind::Marker,
			'z' => TimeKind::TzOffset,
			'Z' => TimeKind::TzAbbrev,
			's' => TimeKind::EpochSecond,
			'Q' => TimeKind::EpochMilli,
			_ => return None
		})
	}
	
	fn character(&self) -> char {
		match *self {
			TimeKind::Hour24 => 'H',
			TimeKind::Hour12 => 'I',
			TimeKind::UnpaddedHour24 => 'k',
			TimeKind::UnpaddedHour12 => 'l',
			TimeKind::Minute => 'M',
			TimeKind::Second => 'S',
			TimeKind::Milli => 'L',
			TimeKind::Nano => 'N',
			TimeKind::Marker => 'p',
			TimeKind::TzOffset => 'z',
			TimeKind::TzAbbrev => 'Z',
			TimeKind::EpochSecond => 's',
			TimeKind::EpochMilli => 'Q',
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Flag {
	/// All: Left justify the result. Width must be provided.
	LeftJustify,
	/// TODO: TODO
	Alternate,
	/// [Decimal, all FP]: Indicates that if the number is non-negative and finite, a '+' should be prepended. Redundant on Hexfloat.
	Plus,
	/// [Decimal, all FP]: Indicates that if the number is non-negative, a ' ' should be prepended.
	LeadingSpace,
	/// Numeric: Indicates to pad the output with zeroes. Width must be provided. Mutually exclusive with LeftJustify.
	ZeroPad,
	/// [Decimal, SciNot, Float, CompSciNot]: Indicates to use a locale-specific grouping seperator. (',' in en_US to format the number 10000 as 10,000)
	Group,
	/// [Decimal, SciNot, Float, CompSciNot]: Indicates to surround the value with '(' and ')' if it is negative.
	Parentheses,
	
	/// Meta: Indicator to argument index picker to use the last index. Meta-flag.
	PreviousIndex
}

impl Flag {
	fn from_character(character: char) -> Option<Self> {
		Some(match character {
			'-' => Flag::LeftJustify,
			'#' => Flag::Alternate,
			'+' => Flag::Plus,
			' ' => Flag::LeadingSpace,
			'0' => Flag::ZeroPad,
			',' => Flag::Group,
			'(' => Flag::Parentheses,
			'<' => Flag::PreviousIndex,
			_ => return None
		})
	}
	
	fn character(&self) -> char {
		match *self {
			Flag::LeftJustify 	=> '-',
			Flag::Alternate 	=> '#',
			Flag::Plus 			=> '+',
			Flag::LeadingSpace 	=> ' ',
			Flag::ZeroPad 		=> '0',
			Flag::Group 		=> ',',
			Flag::Parentheses 	=> '(',
			Flag::PreviousIndex => '<'
		}
	}
	
	fn bit(&self) -> u8 {
		match *self {
			Flag::LeftJustify 	=> 1,
			Flag::Alternate 	=> 2,
			Flag::Plus 			=> 4,
			Flag::LeadingSpace 	=> 8,
			Flag::ZeroPad 		=> 16,
			Flag::Group 		=> 32,
			Flag::Parentheses 	=> 64,
			Flag::PreviousIndex => 0
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Index {
	Next,
	Exact(usize),
	Previous
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Flags(u8);

impl Flags {
	pub fn handle(&mut self, flag: Flag) -> bool {
		if self.0 & flag.bit() != 0 {
			false
		} else {
			self.0 |= flag.bit();
			true
		}
	}
	
	pub fn any(&self) -> bool {
		self.0 != 0
	}
	
	pub fn left_justify(&self) -> bool {
		self.0 & Flag::LeftJustify.bit() != 0
	}
	
	pub fn alternate(&self) -> bool {
		self.0 & Flag::Alternate.bit() != 0
	}
	
	pub fn plus(&self) -> bool {
		self.0 & Flag::Plus.bit() != 0
	}
	
	pub fn leading_space(&self) -> bool {
		self.0 & Flag::LeadingSpace.bit() != 0
	}
	
	pub fn zero_pad(&self) -> bool {
		self.0 & Flag::ZeroPad.bit() != 0
	}
	
	pub fn group(&self) -> bool {
		self.0 & Flag::Group.bit() != 0
	}
	
	pub fn parentheses(&self) -> bool {
		self.0 & Flag::Parentheses.bit() != 0
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormatCommand {
	pub index: Index,
	pub flags: Flags,
	pub width: Option<usize>,
	pub precision: Option<usize>,
	pub kind: Kind,
	pub upper: bool
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParseFormatError {
	NotFormat,
	ExpectedPrecision,
	BadConversion(char),
	NoConversion,
	DuplicateFlag(Flag)
}

impl ParseFormatError {
	pub fn help(&self) -> Option<&'static str> {
		match *self {
			ParseFormatError::NotFormat => None,
			ParseFormatError::ExpectedPrecision => None,
			ParseFormatError::BadConversion(_) => Some("are you sure you don't have any invalid characters in the format code, as that would result in this error as well?"),
			ParseFormatError::NoConversion => None,
			ParseFormatError::DuplicateFlag(_) => None
		}
	}
}

impl Display for ParseFormatError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		match *self {
			ParseFormatError::NotFormat => write!(f, "tried to parse a format code, but it does not start with '%' - the caller has a bug"),
			ParseFormatError::ExpectedPrecision => write!(f, "found a '.', but there was no precision number afterwards"),
			ParseFormatError::BadConversion(c) => write!(f, "unsupported conversion character: {}", c),
			ParseFormatError::NoConversion => write!(f, "didn't find a conversion character"),
			ParseFormatError::DuplicateFlag(flag) => write!(f, "flag specified multiple times: {:?} ['{}']", flag, flag.character())
		}
	}
}

impl FormatCommand {
	/// Parses a format command of the form %[argument_index$][flags][width][.precision]conversion[date_conversion] into an object.
	/// Returns the amount of bytes consumed from the buffer along with the parsed format command.
	pub fn parse(s: &str) -> Result<(usize, Self), ParseFormatError> {
		// Record the starting length of the string, so that the length of the format code can be found later.
		let start_len = s.len();
		
		// All format codes start with a %
		if !s.starts_with('%') {
			return Err(ParseFormatError::NotFormat)
		}
		
		// Get the first number, which may either be the argument index or the width. If there is not number here, then it was either flags, start of precision, or the conversion.
		let (num, s) = get_num(&s[1..]);
		let was_index = s.starts_with('$');
		
		// Try to parse the numeric value.
		let value = if !num.is_empty() {
			Some(num.parse::<usize>().expect("parsing an integer from a digits-only string should never fail"))
		} else {
			None
		};
		
		let mut flags = Flags(0);
		
		let (s, width, arg_idx) = if was_index || num.is_empty() {
			// Remove the '$' character, if present.
			let s = &s[if was_index {1} else {0}..];
			let mut num_flags = 0;
			let mut use_prev = false;
			
			let mut iter = s.chars().map(Flag::from_character);
			while let Some(Some(flag)) = iter.next() {
				match flag {
					Flag::PreviousIndex => use_prev = true,
					flag => if !flags.handle(flag) { return Err(ParseFormatError::DuplicateFlag(flag)) }
				}
				
				num_flags += 1;
			}
			
			let index = match (use_prev, value) {
				(true, _)        => Index::Previous,
				(false, None)    => Index::Next,
				(false, Some(0)) => Index::Next,
				(false, Some(x)) => Index::Exact(x)
			};
			
			// While num_flags is a character count and not a byte count, this is still valid as all flags are single-byte characters.
			let (num, s) = get_num(&s[num_flags..]);
		
			let width = if !num.is_empty() {
				Some(num.parse::<usize>().expect("parsing an integer from a digits-only string should never fail"))
			} else {
				None
			};
			
			(s, width, index)
		} else {
			(s, value, Index::Next)
		};
		
		let (s, precision) = if s.starts_with('.') {
			let (num, s) = get_num(&s[1..]);
		
			(s, if !num.is_empty() {
				Some(num.parse::<usize>().expect("parsing an integer from a digits-only string should never fail"))
			} else {
				return Err(ParseFormatError::ExpectedPrecision)
			})
		} else {
			(s, None)
		};
		
		let (kind, upper) =
			s.chars().nth(0)
			.and_then(|first| Kind::from_character(first, s.chars().nth(1)))
			.ok_or( match s.chars().nth(0) {
				Some(c) => ParseFormatError::BadConversion(c),
				None => ParseFormatError::NoConversion
			})?;
		
		Ok((start_len - s.len() + 1, FormatCommand {
			index: arg_idx,
			flags: flags,
			width: width,
			precision: precision,
			kind: kind,
			upper: upper
		}))
	}
}

fn get_num(s: &str) -> (&str, &str) {
	let mut digits = 0;
	
	for c in s.chars() {
		match c {
			'0'...'9' => digits += 1,
			_ => break
		}
	}
	
	s.split_at(digits)
}

#[test]
fn test_parse() {
	assert_eq!(Ok((2, FormatCommand { index: Index::Next, flags: Flags(0), width: None, precision: None, kind: Kind::Decimal, upper: false})), 		   FormatCommand::parse("%d"));
	assert_eq!(Ok((5, FormatCommand { index: Index::Exact(42), flags: Flags(0), width: None, precision: None, kind: Kind::Decimal, upper: false})),     FormatCommand::parse("%42$d"));
	assert_eq!(Ok((4, FormatCommand { index: Index::Next, flags: Flags(0), width: Some(43), precision: None, kind: Kind::Decimal, upper: false})),      FormatCommand::parse("%43d"));
	assert_eq!(Ok((7, FormatCommand { index: Index::Exact(42), flags: Flags(0), width: Some(43), precision: None, kind: Kind::Decimal, upper: false})), FormatCommand::parse("%42$43d"));
	assert_eq!(Ok((7, FormatCommand { index: Index::Next, flags: Flags(0), width: Some(42), precision: Some(43), kind: Kind::Float, upper: false})),    FormatCommand::parse("%42.43f"));
	assert_eq!(Ok((10, FormatCommand { index: Index::Exact(41), flags: Flags(0), width: Some(42), precision: Some(43), kind: Kind::Float, upper: false})),FormatCommand::parse("%41$42.43f"));
}