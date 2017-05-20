use std::num::ParseIntError;
use std::num::ParseFloatError;

#[derive(Debug)]
pub enum Kind {
	Int { low: i64, high: i64 },
	Float { low: f64, high: f64 },
	String { max_length: usize }
}

#[derive(Debug)]
pub enum ParseError {
	IntRange(i64),
	FloatRange(f64),
	IntError(ParseIntError),
	FloatError(ParseFloatError),
	TooLong(usize)
}

impl Kind {
	pub fn parse(&self, data: &str) -> Result<Value, ParseError> {
		match *self {
			Kind::Int {low, high} 
			    => data.parse::<i64>()
					.map_err(ParseError::IntError)
					.and_then(|x| 
						if x >= low && x <= high {
							Ok(x)
						} else {
							Err(ParseError::IntRange(x))
						}
					).map(Value::Int),
			Kind::Float {low, high} 
				=> data.parse::<f64>()
					.map_err(ParseError::FloatError)
					.and_then(|x| 
						if x >= low && x <= high {
							Ok(x)
						} else {
							Err(ParseError::FloatRange(x))
						}
					).map(Value::Float),
			Kind::String { max_length } 
				=> if data.len() <= max_length { 
					Ok(data.to_string()).map(Value::String) 
				} else { 
					Err(ParseError::TooLong(data.len())) 
				}
		}
	}
}

#[derive(Debug)]
pub enum Value {
	Int(i64),
	Float(f64),
	String(String)
}