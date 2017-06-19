/*use std::ops::{Add, Sub};
use num::Num;

pub struct SegmentedString<L, S> where L: Num + Into<usize> {
	buffer: String,
	segments: Vec<(L, S)>
}

impl<L, S> SegmentedString<L, S> where L: Num + Into<usize> {
	pub fn push(&mut self, string: &str, segment: S) {
		let len = string.len() as L;
		
		self.buffer.push_str(string);
		self.segments.push((len, segment));
	}
}*/