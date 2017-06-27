use serde::{Serializer, Serialize, Deserializer, Deserialize};
use serde::de::{Error, Visitor};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

pub enum ParseColorError {
	/// String did not have required length. 7 bytes for Rgb, 9 bytes for Rgba.
	TooShort,
	/// String did not start with a hash.
	NoHash,
	/// Non hex characters in body of string.
	NotHex
}

impl Display for ParseColorError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match *self {
			ParseColorError::TooShort => write!(f, "must be 7 characters for rgb, 9 characters for rgba."),
			ParseColorError::NoHash => write!(f, "literal did not start with a hash character ('#')."),
			ParseColorError::NotHex => write!(f, "non hex values after hash character ('#')."),
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Rgb(u32);

impl Rgb {
	/// Creates a new RGB value, where the values are in the Srgb color set.
	/// The provided integer should be in the format `(r << 16) | (g << 8) | b`.
	pub fn from_rgb(rgb: u32) -> Self {
		Rgb(rgb & 0x00FFFFFF)
	}
	
	/// Creates a new RGB value, where the values are in the Srgb color set.
	pub fn new(r: u8, g: u8, b: u8) -> Self {
		Rgb(
			((r as u32) << 16) | 
			((g as u32) << 8) | 
			(b as u32)
		)
	}
	
	pub fn r(&self) -> u8 {
		(self.0 >> 16) as u8
	}
	
	pub fn g(&self) -> u8 {
		(self.0 >> 8) as u8
	}
	
	pub fn b(&self) -> u8 {
		self.0 as u8
	}
	
	/// Returns an integer with the format `(r << 16) | (g << 8) | b`.
	pub fn rgb(&self) -> u32 {
		self.0
	}
	
	pub fn to_srgb(&self) -> [f32; 3] {
		[
			(self.r() as f32) / 255.0,
			(self.g() as f32) / 255.0,
			(self.b() as f32) / 255.0
		]
	}
	
	pub fn to_linear(&self) -> [f32; 3] {
		let srgb = self.to_srgb();
		
		[
			srgb[0].powf(2.2),
			srgb[1].powf(2.2),
			srgb[2].powf(2.2)
		]
	}
	
	pub fn to_rgba(&self, alpha: u8) -> Rgba {
		Rgba(self.0 | ((alpha as u32) << 24))
	}
}

impl Display for Rgb {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "#{:6X}", self.rgb())
	}
}

impl FromStr for Rgb {
	type Err = ParseColorError;
	
	fn from_str(str: &str) -> Result<Self, Self::Err> {
		if str.len() < 7 {
			Err(ParseColorError::TooShort)
		} else if !str.starts_with('#') {
			Err(ParseColorError::NoHash)
		} else {
			u32::from_str_radix(&str[1..], 16).map(Rgb).map_err(|_| ParseColorError::NotHex)
		}
	}
}

impl<'de> Deserialize<'de> for Rgb {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		struct RgbVisitor;
		impl<'de> Visitor<'de> for RgbVisitor {
			type Value = Rgb;
			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		        formatter.write_str("a string literal in the form of \"#RRGGBB\", where RRGGBB are hex digits")
		    }
			
			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
				v.parse::<Rgb>().map_err(|e| E::custom(format!("malformed rgb literal: {}", v)))
			}
		}
		
		deserializer.deserialize_str(RgbVisitor)
	}
}

impl Serialize for Rgb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Rgba(u32);

impl Rgba {
	/// Creates a new RGB value, where the values are in the Srgb color set.
	pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
		Rgba(
			((a as u32) << 24) |
			((r as u32) << 16) | 
			((g as u32) << 8) | 
			(b as u32)
		)
	}
	
	pub fn r(&self) -> u8 {
		(self.0 >> 16) as u8
	}
	
	pub fn g(&self) -> u8 {
		(self.0 >> 8) as u8
	}
	
	pub fn b(&self) -> u8 {
		self.0 as u8
	}
	
	pub fn a(&self) -> u8 {
		(self.0 >> 24) as u8
	}
	
	/// Returns an integer with the format `(a << 24) | (r << 16) | (g << 8) | b`.
	pub fn rgba(&self) -> u32 {
		self.0
	}
	
	pub fn to_srgb(&self) -> [f32; 4] {
		[
			(self.r() as f32) / 255.0,
			(self.g() as f32) / 255.0,
			(self.b() as f32) / 255.0,
			(self.a() as f32) / 255.0
		]
	}
	
	pub fn to_linear(&self) -> [f32; 4] {
		let srgb = self.to_srgb();
		
		[
			srgb[0].powf(2.2),
			srgb[1].powf(2.2),
			srgb[2].powf(2.2),
			srgb[3]
		]
	}
	
	pub fn to_rgb(&self) -> Rgb {
		Rgb(self.0 & 0xFFFFFF)
	}
}

impl Display for Rgba {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "#{:8X}", self.rgba())
	}
}

impl FromStr for Rgba {
	type Err = ParseColorError;
	
	fn from_str(str: &str) -> Result<Self, Self::Err> {
		if str.len() < 9 {
			Err(ParseColorError::TooShort)
		} else if !str.starts_with('#') {
			Err(ParseColorError::NoHash)
		} else {
			u32::from_str_radix(&str[1..], 16).map(Rgba).map_err(|_| ParseColorError::NotHex)
		}
	}
}

impl<'de> Deserialize<'de> for Rgba {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		struct RgbaVisitor;
		impl<'de> Visitor<'de> for RgbaVisitor {
			type Value = Rgba;
			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		        formatter.write_str("a string literal in the form of \"#AARRGGBB\", where AARRGGBB are hex digits")
		    }
			
			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
				v.parse::<Rgba>().map_err(|e| E::custom(format!("malformed rgba literal: {}", v)))
			}
		}
		
		deserializer.deserialize_str(RgbaVisitor)
	}
}

impl Serialize for Rgba {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}