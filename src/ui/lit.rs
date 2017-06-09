use std::fmt::{self, Formatter, Display};
use std::str::FromStr;
use std::num::{ParseIntError, ParseFloatError};
use serde::de::{Deserialize, Deserializer, Visitor, Error};
use serde::ser::{Serialize, Serializer};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Lit {
	part: f32,
	px: i32,
	tx: i32
}

impl Lit {
	pub fn from_part(part: f32) -> Self {
		Self::new(part, 0, 0)
	}
	
	pub fn new(part: f32, px: i32, tx: i32) -> Self {
		Lit {
			part: part,
			px: px,
			tx: tx
		}
	}
	
	pub fn px(&self) -> i32 {
		self.px
	}
	
	pub fn tx(&self) -> i32 {
		self.tx
	}
	
	pub fn part(&self) -> f32 {
		self.part
	}
	
	pub fn to_part(&self, scale: f32) -> f32 {
		self.part + (self.px as f32)*scale + (self.tx as f32)/128.0
	}
	
	pub fn to_px(&self, scale: f32) -> f32 {
		let part = self.part + (self.tx as f32)/128.0;
		
		part / scale + (self.px as f32)
	}
	
	fn handle_term(&mut self, op: char, term: &str) -> Result<(), ParseLitError> {
		println!("term: {}, op: {}", term, op);
		
		if term.ends_with("px") {
			let term = &term[..term.len() - 2];
			let val = term.parse::<i32>().map_err(ParseLitError::ParseInt)?;
		
			self.px += if op=='+' {val} else {-val};
		} else if term.ends_with("tx") {
			let term = &term[..term.len() - 2];
			let val = term.parse::<i32>().map_err(ParseLitError::ParseInt)?;
		
			self.tx += if op=='+' {val} else {-val};
		} else {
			let val =  term.parse::<f32>().map_err(ParseLitError::ParseFloat)?;
			
			self.part += if op=='+' {val} else {-val};
		};
		
		Ok(())
	}
}

impl<'de> Deserialize<'de> for Lit {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		struct LitVisitor;
		impl<'de> Visitor<'de> for LitVisitor {
			type Value = Lit;
			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		        formatter.write_str("a string literal with the format \"X[.Z] [+ Ypx]\"")
		    }
			
			fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E> where E: Error {
				Ok(Lit { part: v, px: 0, tx: 0 })
			}
			
			fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> where E: Error {
				Ok(Lit { part: v as f32, px: 0, tx: 0 })
			}
			
			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
				v.parse::<Lit>().map_err(|e| E::custom(format!("malformed literal: {}", v)))
			}
		}
		
		deserializer.deserialize_str(LitVisitor)
	}
}

impl Serialize for Lit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseLitError {
	ParseInt(ParseIntError),
	ParseFloat(ParseFloatError),
	DoubleOperator
}

impl FromStr for Lit {
	type Err = ParseLitError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut lit = Lit { part: 0.0, px: 0, tx: 0 };
		let mut start = Some(0);
		let mut last_op = '+';
		
		for (index, char) in s.char_indices() {
			if char=='+' || char== '-' {
				let term = if let Some(start) = start {
					&s[start..index]
				} else {
					return Err(ParseLitError::DoubleOperator)
				};
				
				lit.handle_term(last_op, term.trim())?;
				last_op = char;
				
				start = None;
			} else if start.is_none() {
				start = Some(index);
			}
		}
		
		// Handle the last term.
		if let Some(start) = start {
			let term = &s[start..];
			lit.handle_term(last_op, term.trim())?;
		}
		
		Ok(lit)
	}
}

impl Display for Lit {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		write!(f, "{} + {}px + {}tx", self.part, self.px, self.tx)
	}
}

#[test]
fn test_parse_lit() {
	assert_eq!(Ok(Lit::new(2.0, 0, 0)), "1 + 1".parse::<Lit>());
	assert_eq!(Ok(Lit::new(1.0, 1, 0)), "1 + 1px".parse::<Lit>());
	assert_eq!(Ok(Lit::new(1.0, 1, 0)), "1px + 1".parse::<Lit>());
	assert_eq!(Ok(Lit::new(1.0, 1, 0)), "1+1px".parse::<Lit>());
	assert_eq!(Ok(Lit::new(1.0, 1, 0)), "1px+1".parse::<Lit>());
	assert_eq!(Ok(Lit::new(-1.0, 1, 0)), "1px-1".parse::<Lit>());
	assert_eq!(Ok(Lit::new(0.0, 0, 0)), "1 - 1".parse::<Lit>());
	assert_eq!(Ok(Lit::new(1.0, 0, -23)), "1 - 23tx".parse::<Lit>());
	"1 + 1.0px".parse::<Lit>().unwrap_err();
}