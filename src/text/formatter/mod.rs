use std::str::FromStr;
use std::num::ParseIntError;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Kind {
	Bool,
	HexHashCode,
	String,
	Unicode,
	Decimal,
	Octal,
	Hex,
	SciNot,
	Float,
	CompSciNot,
	Hexfloat,
	Date,
	Percent,
	Newline
}

impl Kind {
	fn from_character(character: char) -> Option<(Self, bool)> {
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
			'e' => (Kind::SciNot, 		false),
			'E' => (Kind::SciNot, 		true),
			'f' => (Kind::Float, 		false),
			'g' => (Kind::CompSciNot, 	false),
			'G' => (Kind::CompSciNot, 	true),
			'a' => (Kind::Hexfloat, 	false),
			'A' => (Kind::Hexfloat, 	true),
			't' => (Kind::Date, 		false),
			'T' => (Kind::Date, 		true),
			'%' => (Kind::Percent, 		false),
			'n' => (Kind::Newline, 		false),
			_ => return None
		})
	}
	
	fn character(&self, upper: bool) -> char {
		match *self {
			Kind::Bool 			=> if upper {'B'} else {'b'},
			Kind::HexHashCode 	=> if upper {'H'} else {'h'},
			Kind::String 		=> if upper {'S'} else {'s'},
			Kind::Unicode 		=> if upper {'C'} else {'c'},
			Kind::Decimal 		=> 'd',
			Kind::Octal 		=> 'o',
			Kind::Hex 			=> if upper {'X'} else {'x'},
			Kind::SciNot 		=> if upper {'E'} else {'e'},
			Kind::Float 		=> 'f',
			Kind::CompSciNot 	=> if upper {'G'} else {'g'},
			Kind::Hexfloat 		=> if upper {'A'} else {'a'},
			Kind::Date 			=> if upper {'T'} else {'t'},
			Kind::Percent 		=> '%',
			Kind::Newline 		=> 'n'
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
			Kind::Date 			=> true,
			Kind::Percent 		=> false,
			Kind::Newline 		=> false
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Flag {
	LeftJustify,
	Alternate,
	Plus,
	LeadingSpace,
	ZeroPad,
	Group,
	Parentheses,
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
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Index {
	Next,
	Exact(usize),
	Previous
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Flags(u8);

#[derive(Debug, Clone, Eq, PartialEq)]
struct FormatCommand {
	index: Index,
	flags: Flags,
	width: Option<usize>,
	precision: Option<usize>,
	kind: Kind,
	upper: bool
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ParseFormatError {
	NotFormat,
	ParseInt(ParseIntError),
	ExpectedPrecision,
	BadConversion
}

impl FormatCommand {
	fn parse(s: &str) -> Result<(usize, Self), ParseFormatError> {
		let start_len = s.len();
		
		if !s.starts_with('%') {
			return Err(ParseFormatError::NotFormat)
		}
		
		let s = &s[1..];
		let (num, s) = get_num(s);
		let was_index = s.starts_with('$');
		
		let value = if !num.is_empty() {
			Some(try!(num.parse::<usize>().map_err(ParseFormatError::ParseInt)))
		} else {
			None
		};
		
		let s = &s[if was_index {1} else {0}..];
		
		let (s, flags, width, arg_idx) = if was_index || num.is_empty() {
			let mut num_flags = 0;
			
			let mut use_prev = false;
			let mut flags = Flags(0);
			let mut iter = s.chars().map(Flag::from_character);
			while let Some(Some(flag)) = iter.next() {
				if flag == Flag::PreviousIndex {
					use_prev = true;
				} else {
					// TODO: Handle flag
					unimplemented!();
				}
				
				num_flags += 1;
			}
			
			let (num, s) = get_num(s);
		
			let width = if !num.is_empty() {
				Some(try!(num.parse::<usize>().map_err(ParseFormatError::ParseInt)))
			} else {
				None
			};
			
			let index = if use_prev {
				Index::Previous
			} else {
				if let Some(value) = value {
					if value != 0 {
						Index::Exact(value)
					} else {
						Index::Next
					}
				} else {
					Index::Next
				}
			};
			
			(s, flags, width, index)
		} else {
			(s, Flags(0), value, Index::Next)
		};
		
		let has_precision =  s.starts_with('.');
		let s = &s[if has_precision {1} else {0}..];
		
		let (s, precision) = if has_precision {
			let (num, s) = get_num(s);
		
			(s, if !num.is_empty() {
				Some(try!(num.parse::<usize>().map_err(ParseFormatError::ParseInt)))
			} else {
				return Err(ParseFormatError::ExpectedPrecision)
			})
		} else {
			(s, None)
		};
		
		let (kind, upper) = if let Some(v) = s.chars().nth(0).and_then(Kind::from_character) {
			v
		} else {
			return Err(ParseFormatError::BadConversion)
		};
		
		Ok((start_len - s.len(), FormatCommand {
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
	assert_eq!(Ok(FormatCommand { index: Index::Next, flags: Flags(0), width: None, precision: None, kind: Kind::Decimal, upper: false}), 		   FormatCommand::parse("%d"));
	assert_eq!(Ok(FormatCommand { index: Index::Exact(42), flags: Flags(0), width: None, precision: None, kind: Kind::Decimal, upper: false}),     FormatCommand::parse("%42$d"));
	assert_eq!(Ok(FormatCommand { index: Index::Next, flags: Flags(0), width: Some(43), precision: None, kind: Kind::Decimal, upper: false}),      FormatCommand::parse("%43d"));
	assert_eq!(Ok(FormatCommand { index: Index::Exact(42), flags: Flags(0), width: Some(43), precision: None, kind: Kind::Decimal, upper: false}), FormatCommand::parse("%42$43d"));
	assert_eq!(Ok(FormatCommand { index: Index::Next, flags: Flags(0), width: Some(42), precision: Some(43), kind: Kind::Float, upper: false}),    FormatCommand::parse("%42.43f"));
	assert_eq!(Ok(FormatCommand { index: Index::Exact(41), flags: Flags(0), width: Some(42), precision: Some(43), kind: Kind::Float, upper: false}),FormatCommand::parse("%41$42.43f"));
}