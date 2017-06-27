use text::style::{Style, StyleCommand, Color, PaletteColor};
use std::fmt::{self, Display};
use std::str::FromStr;
use serde::de::{Deserializer, Deserialize, Error, Visitor};
use serde::{Serializer, Serialize};
use std::iter::Peekable;
use std::slice;

// [PlainBuf] Overhead: 48 bytes for collections, 4 bytes per descriptor, random access
// [Encoded] Overhead: 24 bytes for collections, 2 to 12 bytes per descriptor

/// A buffer storing an unstyled string annotated with styles in a descriptor buffer.
#[derive(Debug)]
pub struct PlainBuf {
	/// The unstyled string
	string: String,
	/// The descriptors holding the styles.
	descriptors: Vec<(u8, Style)>
}

impl PlainBuf {
	/// Creates a new, empty PlainBuf.
	pub fn new() -> Self {
		PlainBuf {
			string: String::new(),
			descriptors: Vec::new()
		}
	}
	
	/// Creates a new PlainBuf where each buffer has a certain capacity.
	pub fn with_capacity(string: usize, descriptors: usize) -> Self {
		PlainBuf {
			string: String::with_capacity(string),
			descriptors: Vec::with_capacity(descriptors)
		}
	}
	
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
	
	pub fn push(&mut self, string: &str, style: Style) {
		self.string.push_str(string);
		
		self.break_descriptor(string.len(), style);
	}
	
	fn break_descriptor(&mut self, len: usize, style: Style) {
		let total_len = self.string.len();
		let mut string = &self.string[total_len - len ..];
		
		while string.len() > 0 {
			let mut chunk_len = u8::max_value();
			
			while !string.is_char_boundary(chunk_len as usize) {
				chunk_len -= 1;
			}
			
			self.descriptors.push((chunk_len, style));
			
			string = &string[chunk_len as usize ..];
		}
	}
	
	pub fn unstyled(&self) -> &str {
		&self.string
	}
	
	pub fn iter(&self) -> Iter {
		Iter {
			head: &self.string,
			descriptors: self.descriptors.iter().peekable()
		}
	}
	
	// TODO: Pop, Truncate
}

impl FromStr for PlainBuf {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, ()> {
		let mut reader = FormatReader::new();
	
		reader.append(s);
		Ok(reader.finish())
	}
}

impl Display for PlainBuf {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut writer = FormatWriter::new(f);
		let mut head = &self.string as &str;
		
		let mut offset = 0;
		
		for &(len, style) in &self.descriptors {
			let (part, rest) = head.split_at(len as usize);
			head = rest;
			
			writer.write(part, style)?;
		}
		
		Ok(())
	}
}

pub struct Iter<'a, 'b> {
	head: &'a str,
	descriptors: Peekable<slice::Iter<'b, (u8, Style)>>
}

impl<'a, 'b> Iter<'a, 'b> {
	pub fn empty() -> Self {
		Iter {
			head: "",
			descriptors: [].iter().peekable()
		}
	}
}

impl<'a, 'b> Iterator for Iter<'a, 'b> {
	type Item = (&'a str, Style);
	
	fn next(&mut self) -> Option<Self::Item> {
		if let Some(&(len, style)) = self.descriptors.next() {
			let mut len = len as usize;
			
			// Try to merge descriptors of the same style together.
			while let Some(&&(next_len, next_style)) = self.descriptors.peek() {
				if next_style != style {
					break;
				}
					
				len += next_len as usize;
				
				self.descriptors.next();
			}
			
			let (part, rest) = self.head.split_at(len);
			self.head = rest;
			
			Some((part, style))
		} else {
			None
		}
	}
}

pub struct FormatReader {
	target: PlainBuf,
	marker: char,
	expect_code: bool,
	style: Style,
	current_len: usize
}

impl FormatReader {
	pub fn new() -> Self {
		Self::with_marker('§')
	}
	
	pub fn with_marker(marker: char) -> Self {
		FormatReader {
			target: PlainBuf::new(),
			marker: marker,
			expect_code: false,
			style: Style::new(),
			current_len: 0
		}
	}
	
	pub fn extend(target: PlainBuf, marker: char) -> Self {
		FormatReader {
			target: target,
			marker: marker,
			expect_code: false,
			style: Style::new(),
			current_len: 0
		}
	}
	
	fn flush(&mut self) {
		if self.current_len != 0 {
			self.target.break_descriptor(self.current_len, self.style);
			self.current_len = 0;
		}
	}
	
	pub fn append(&mut self, string: &str) {
		let mut start = 0;
		
		for (index, char) in string.char_indices() {
			if self.expect_code {
				self.target.string.push_str(&string[start..start+self.current_len]);
				self.flush();
				
				self.style.process(&StyleCommand::from_code(char).unwrap_or(StyleCommand::Color(PaletteColor::White)));
				
				self.expect_code = false;
			} else if char == self.marker {
				self.expect_code = true;
			} else {
				if self.current_len == 0 {
					start = index;
				}
				
				self.current_len += utf8_len(char);
			}
		}
		
		self.target.string.push_str(&string[start..start+self.current_len]);
	}
	
	pub fn finish(mut self) -> PlainBuf {
		self.flush();
		self.target
	}
}

// TODO: Does the stdlib have a function like this?
fn utf8_len(c: char) -> usize {
	let c = c as u32;
	
	if c <= 0x7F {
		1
	} else if c <= 0x7FF {
		2
	} else if c <= 0xFFFF {
		3
	} else {
		4
	}
}

pub struct FormatWriter<'w, W> where W: fmt::Write, W: 'w {
	target: &'w mut W,
	current_style: Style,
	marker: char
}

impl<'w, W> FormatWriter<'w, W> where W: fmt::Write {
	pub fn new(target: &'w mut W) -> Self {
		FormatWriter { target, current_style: Style::new(), marker: '§' }
	}
	
	pub fn with_marker(target: &'w mut W, marker: char) -> Self {
		FormatWriter { target, current_style: Style::new(), marker }
	}
	
	pub fn write(&mut self, string: &str, style: Style) -> fmt::Result {
		for command in self.current_style.transition(style) {
			self.target.write_char(self.marker)?;
			self.target.write_char(command.as_code())?;
		}
		
		self.current_style = style;
		
		self.target.write_str(string)
	}
}

impl<'de> Deserialize<'de> for PlainBuf {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		struct PlainVisitor;
		impl<'de> Visitor<'de> for PlainVisitor {
			type Value = PlainBuf;
			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		        formatter.write_str("a string containing Minecraft formatting codes")
		    }
			
			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
				Ok(v.parse::<PlainBuf>().expect("Parsing a PlainBuf should never return an error!"))
			}
		}
		
		deserializer.deserialize_str(PlainVisitor)
	}
}

impl Serialize for PlainBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}