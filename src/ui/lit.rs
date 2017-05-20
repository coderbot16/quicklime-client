use std::fmt::{self, Formatter, Display};
use std::str::FromStr;
use std::num::{ParseIntError, ParseFloatError};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Lit {
	part: f32,
	px: i32
}

impl Lit {
	pub fn new(part: f32, px: i32) -> Self {
		Lit {
			part: part,
			px: px
		}
	}
	
	pub fn px(&self) -> i32 {
		self.px
	}
	
	pub fn part(&self) -> f32 {
		self.part
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseLitError {
	ParseInt(ParseIntError),
	ParseFloat(ParseFloatError)
}

impl FromStr for Lit {
	type Err = ParseLitError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut lit = Lit { part: 0.0, px: 0 };
		
		for term in s.split('+').map(str::trim) {
			if term.ends_with("px") {
				// Remove prefix
				let term = &term[..term.len() - 2];
				
				lit.px += try!(term.parse::<i32>().map_err(ParseLitError::ParseInt));
			} else {
				lit.part += try!(term.parse::<f32>().map_err(ParseLitError::ParseFloat));
			}
		}
		
		Ok(lit)
	}
}

impl Display for Lit {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		write!(f, "{} + {}px", self.part, self.px)
	}
}

#[test]
fn test_parse_lit() {
	assert_eq!(Ok(Lit::new(2.0, 0)), "1 + 1".parse::<Lit>());
	assert_eq!(Ok(Lit::new(1.0, 1)), "1 + 1px".parse::<Lit>());
	assert_eq!(Ok(Lit::new(1.0, 1)), "1px + 1".parse::<Lit>());
	assert_eq!(Ok(Lit::new(1.0, 1)), "1+1px".parse::<Lit>());
	assert_eq!(Ok(Lit::new(1.0, 1)), "1px+1".parse::<Lit>());
	"1 + 1.0px".parse::<Lit>().unwrap_err();
}