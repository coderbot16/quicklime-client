use super::default::character_to_default;
use text::style::Style;
use text::metrics::{GlyphSize, Metrics};
use render2d::{Color, Quad, Rect};

const STRIKE_LEVEL: f32 = 5.0;
const UNDER_LEVEL: f32 = 0.0;
const AVOID_FP_ROUNDING: f32 = 0.01;

#[derive(Debug)]
pub enum Command {
	Char(DrawChar),
	CharDefault { x: f32, y: f32, italic: bool, character: u8, size: GlyphSize },
	Rect {x: f32, y: f32, width: f32, height: f32}
}

impl Command {
	fn decide(x: f32, y: f32, italic: bool, character: char, force_unicode: bool, size: GlyphSize, color: Color) -> Self {
		let default_index = character_to_default(character);
		
		if let Some(default) = default_index {
			if !force_unicode {
				return Command::CharDefault { x: x, y: y, italic: italic, character: default, size: size }
			}
		}
		
		Command::Char (DrawChar { x: x, y: y, italic: italic, character: character, size: size, color: color })
	}
}

#[derive(Debug)]
pub struct DrawChar { 
	pub x: f32, 
	pub y: f32, 
	pub italic: bool, 
	pub character: char,
	pub size: GlyphSize,
	pub color: Color
}

impl DrawChar {
	pub fn to_quad(&self, scale: (f32, f32)) -> Quad {
		let left = (self.size.left() as f32) / 256.0;
		let add = (self.size.right() as f32 + 1.0) / 256.0;
		
		let width = (self.size.width() as f32 / 2.0 - AVOID_FP_ROUNDING) * scale.0;
					
		let (x, y) = (self.x*scale.0, self.y*scale.1);
		let character = self.character as u32;
					
		let tex_x = ((character % 16) as f32) / 16.0;
		let tex_y = 1.0 - (((character % 256) / 16) as f32 + 1.0) / 16.0;
		
		// Vanilla doesn't AVOID_FP_ROUNDING with the minimum x position, but we encountered a bug with it and do it.
		let mut quad = Rect::textured(
			[x, y + 1.0 * scale.1], [x + width, y + (9.0-AVOID_FP_ROUNDING) * scale.1], 
			self.color, 
			[tex_x + left + AVOID_FP_ROUNDING*scale.0, tex_y], [tex_x + add, tex_y + (16.0 - 2.0*AVOID_FP_ROUNDING)/256.0]
		).as_quad();
		
		quad.slant(if self.italic {scale.0} else {0.0});
		
		quad
	}
}

pub struct RenderingContext<'a> {
	metrics: &'a Metrics
}

impl<'a> RenderingContext<'a> {
	pub fn new(metrics: &'a Metrics) -> Self {
		RenderingContext {
			metrics: metrics, 
		}
	}
	
	pub fn render<I>(&self, x: f32, y: f32, run: I, style: &Style, shadow: bool, color: Color) -> Render<I> where I: Iterator<Item=char> {
		Render {
			metrics: self.metrics,
			source: run,
			style: *style,
			shadow: shadow,
			start: (x, y),
			color: color,
			
			advance: 0.0,
			bonus: 0.0,
			state: RenderState::NextChar
		}
	}
}

#[derive(Eq, PartialEq)]
enum RenderState {
	NextChar,
	/// Rendering a character. The bold flags allows this to happen twice for bold chars.
	Main(char, bool),
	/// Rendering the strikethrough
	Strike,
	/// Rendering the underline
	Under,
	/// Everything rendered.
	End
}

/// An iterator that turns a series of chars into a series of rendering commands using a state machine.
pub struct Render<'a, I> where I: Iterator<Item=char> {
	// Data provided by original call
	metrics: &'a Metrics,
	source: I,
	style: Style,
	shadow: bool,
	start: (f32, f32),
	color: Color,
	
	// Data changed from iterations
	advance: f32,
	bonus: f32,
	state: RenderState,
}

impl<'a, I> Iterator for Render<'a, I> where I: Iterator<Item=char> {
	type Item = Option<Command>;
	
	fn next(&mut self) -> Option<Self::Item> {
		let (x, y) = (self.start.0, self.start.1);
		
		if self.state == RenderState::NextChar {
			self.state = if let Some(character) = self.source.next() {
				RenderState::Main(character, false)
			} else {
				if self.style.flags.strikethrough() {
					RenderState::Strike
				} else if self.style.flags.underline() {
					RenderState::Under
				} else {
					RenderState::End
				}
			};
		}
		
		Some(match self.state {
			RenderState::Main(c, bold) => {
				let size = if let Some(sz) = self.metrics.size(c) {
					sz
				} else {
					// This character has no corresponding glyph.
					self.state = RenderState::NextChar;
					return Some(None)
				};
				
				let x = x + self.advance;
				self.advance += size.advance().floor();
				self.advance += if self.style.flags.bold() {1.0} else {0.0};
				self.bonus = size.advance().fract();
				
				if c == ' ' {
					// We don't render space characters, but rendering them wouldn't hurt anything. This is just an optimization.
					self.state = RenderState::NextChar;
					return Some(None)
				}
				
				let default_index = character_to_default(c);
				
				// TODO: Make obsfucate work in some way.
				
				let offset = if self.metrics.always_unicode() {0.5} else {1.0};
				let shadow_and_unicode = (c == '\0' || default_index.is_none() || self.metrics.always_unicode()) && self.shadow;
				let char_offset = if shadow_and_unicode {-offset} else {0.0};
				
				if bold {
					let bold_offset = offset + char_offset;
					self.state = RenderState::NextChar;
					
					Some(Command::decide(x + bold_offset, y + char_offset, self.style.flags.italic(), c, self.metrics.always_unicode(), size, self.color))
				} else {
					self.state = if self.style.flags.bold() {
						RenderState::Main(c, true)
					} else {
						RenderState::NextChar
					};
					
					Some(Command::decide(x + char_offset, y + char_offset, self.style.flags.italic(), c, self.metrics.always_unicode(), size, self.color))
				}
			},
			RenderState::Strike => {
				self.state = if self.style.flags.underline() {RenderState::Under} else {RenderState::End};
				Some(Command::Rect { x: x, y: y + STRIKE_LEVEL, width: self.advance + self.bonus, height: 1.0})
			},
			RenderState::Under => {
				self.state = RenderState::End;
				Some(Command::Rect { x: x - 1.0, y: y + UNDER_LEVEL, width: self.advance + self.bonus + 1.0, height: 1.0})
			},
			RenderState::End => return None,
			RenderState::NextChar => unreachable!()
		})
	}
}
