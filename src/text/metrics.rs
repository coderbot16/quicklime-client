use memmap::{Protection, Mmap};
use std::fs::File;
use std::io::{ErrorKind, Error};
use std::fmt::{self, Display, Formatter};
use text::style::{Style, StyleFlags};
use text::default::{DefaultMetrics, character_to_default};

// Each glyph takes up to 9x9 pixels.

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GlyphSize(u8);
impl GlyphSize {
	pub fn empty() -> Self {
		GlyphSize(0)
	}
	
	pub fn new(left: u8, right: u8) -> Self {
		GlyphSize((left << 4) | (right & 15))
	}
	
	// Incoming: an ASCII width from 0 to 8.
	pub fn from_default_width(width: u8) -> Self {
		if width == 0 {
			GlyphSize::empty()
		} else {
			GlyphSize::new(0, (width * 2) - 1)
		}
	}
	
	/// Returns the left side of the glyph, the X position of the leftmost column of pixels. This is from 0 to 15.
	pub fn left(&self) -> u8 {
		self.0 >> 4
	}
	
	/// Returns the right side of the glyph, the X position of the rightmost column of pixels. This is from 0 to 15.
	pub fn right(&self) -> u8 {
		self.0 & 15
	}
	
	/// Returns the width of the glyph on a texture map of size 256x256, with 256 characters total (16x16). The height of a glyph on the texture map is always 16.
	pub fn width(&self) -> u8 {
		self.right() + 1 - self.left()
	}
	
	/// Returns the advance width of the specified character in this font. The advance is the distance from the leftmost point to the rightmost point on the character's baseline.
	/// In MC fonts, this is 0.0 to 9.0.
	/// Note that this function returns the exact advance after rendering, while Minecraft char sizing may overestimate when calculating centering/trim/etc. 
	/// Use `advance_overestimated` to emulate this.
	pub fn advance(&self) -> f32 {
		if self.left() == 0 && self.right() == 0 {
			0.0
		} else {
			((self.width() as f32) / 2.0) + 1.0
		}
	}
	
	/// Overestimates the advance for some characters. This mimics the behavior of the vanilla size functions, but is incorrect for other purposes.
	/// In addition, this should not be used for Default characters, as the vanilla code does not overestimate for default characters.
	pub fn advance_overestimated(&self) -> f32 {
		if self.right() > 7 {
			9.0
		} else {
			self.advance()
		}
	}
}

impl Display for GlyphSize {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		write!(f, "{{ left: {}, right: {}}}", self.left(), self.right())
	}
}

pub struct GlyphMetrics {
	map: Mmap
}

impl GlyphMetrics {
	pub fn from_file(file: &File) -> Result<Self, Error> {
		Mmap::open(file, Protection::Read)
			.and_then(|map| if map.len() < 65536 {
					Err(Error::new(ErrorKind::UnexpectedEof, "Glyph size map is too short, much be at least 65536 bytes"))
				} else {
					Ok(map)
				} 
			).map(GlyphMetrics::new)
	}
	
	pub fn new(mmap: Mmap) -> Self {
		GlyphMetrics { map: mmap }
	}
	
	pub fn size(&self, value: u16) -> GlyphSize {
		// This accesses 1 byte, so it should be ok.
		GlyphSize(unsafe { self.map.as_slice()[value as usize] })
	}
}

pub struct Metrics {
	default: Option<DefaultMetrics>,
	unicode: Option<GlyphMetrics>
}

impl Metrics {
	pub fn dual(default: DefaultMetrics, unicode: GlyphMetrics) -> Self {
		Metrics {
			default: Some(default),
			unicode: Some(unicode)
		}
	}
	
	pub fn unicode(unicode: GlyphMetrics) -> Self {
		Metrics {
			default: None,
			unicode: Some(unicode)
		}
	}
	
	pub fn always_unicode(&self) -> bool {
		self.default.is_none() && self.unicode.is_some()
	}
	
	pub fn size(&self, value: char) -> Option<GlyphSize> {
		if value == '\0' {
			return Some(GlyphSize::empty())
		} else if value == ' ' {
			return Some(GlyphSize::from_default_width(3))
		};
		
		if let Some(ref default_metrics) = self.default {
			if let Some(default) = character_to_default(value) {
				return Some(default_metrics.size(default))
			}
		}
		
		if let Some(ref unicode_metrics) = self.unicode {
			if value < '\u{10000}' {
				return Some(unicode_metrics.size(value as u16))
			}
		}
		
		None
	}
	
	pub fn advance<'a, S, I>(&'a self, iter: S, style: &StyleFlags) -> Advance<'a, I> where S: IntoIterator<Item=char, IntoIter=I>, I: Iterator<Item=char> {
		Advance { iter: iter.into_iter(), bold: style.bold(), metrics: &self }
	}
}


pub struct Advance<'a, I> where I: Iterator<Item=char> {
	iter: I,
	bold: bool,
	metrics: &'a Metrics
}

impl<'a, I> Advance<'a, I> where I: Iterator<Item=char> {
	pub fn total(self) -> Option<usize> {
		let mut accumulator = 0;
		
		for adv in self {
			accumulator += match adv {
				Some(advance) => advance as usize,
				None => return None
			};
		}
		
		Some(accumulator)
	}
}

impl<'a, I> Iterator for Advance<'a, I> where I: Iterator<Item=char> {
	type Item = Option<u8>;
	
	fn next(&mut self) -> Option<Option<u8>> {
		self.iter
			.next()
			.map(|value| self.metrics
				.size(value)
				.map(|x| x.advance().floor() as u8)
				.map(|x| x + if self.bold {1} else {0} )
			)
	}
}

fn unicode_page_location(block: u32) -> String {
	format!("textures/font/unicode_page_{}_.png", block)
}
// TODO: Wrap, Trim, 