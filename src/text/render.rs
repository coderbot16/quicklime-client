use super::default::character_to_default;
use text::style::{self, Style};
use text::metrics::{GlyphSize, Metrics};
use render2d::{Color, Quad, Rect};
use color::Rgb;

const STRIKE_LEVEL: f32 = 5.0;
const UNDER_LEVEL: f32 = 0.0;
const AVOID_FP_ROUNDING: f32 = 0.01;

#[derive(Debug)]
pub enum Command {
	Char(DrawChar),
	Rect {x: f32, y: f32, width: f32, height: f32, color: Color }
}

impl Command {
	fn draw_char(x: f32, y: f32, italic: bool, character: char, force_unicode: bool, size: GlyphSize, color: Color) -> Self {
		Command::Char (DrawChar { x: x, y: y, italic: italic, character: CharKind::decide(character, force_unicode), size: size, color: color })
	}
}

#[derive(Debug)]
pub enum CharKind {
	Unicode(char),
	Default(u8)
}

impl CharKind {
	fn decide(character: char, force_unicode: bool) -> Self {
		match character_to_default(character) {
			Some(default_index) if !force_unicode => CharKind::Default(default_index),
			Some(_) | None => CharKind::Unicode(character)
		}
	}
	
	fn atlas_index(&self) -> u8 {
		match *self {
			CharKind::Unicode(c) => ((c as u32) % 256) as u8,
			CharKind::Default(d) => d
		}
	}
	
	fn atlas(&self) -> Option<u32> {
		match *self {
			CharKind::Unicode(c) => Some((c as u32) / 256),
			CharKind::Default(_) => None
		}
	}
}

#[derive(Debug)]
pub struct DrawChar {
	pub x: f32, 
	pub y: f32, 
	pub italic: bool, 
	pub character: CharKind,
	pub size: GlyphSize,
	pub color: Color
}

impl DrawChar {
	pub fn to_quad(&self, scale: (f32, f32)) -> (Quad, Option<u32>) {
		let left = (self.size.left() as f32) / 256.0;
		let add = (self.size.right() as f32 + 1.0) / 256.0;
		
		let width = (self.size.width() as f32 / 2.0 - AVOID_FP_ROUNDING) * scale.0;
					
		let (x, y) = (self.x*scale.0, self.y*scale.1);
		let atlas_index = self.character.atlas_index();
					
		let tex_x = ((atlas_index % 16) as f32) / 16.0;
		let tex_y = 1.0 - ((atlas_index / 16) as f32 + 1.0) / 16.0;
		
		// Vanilla doesn't AVOID_FP_ROUNDING with the minimum x position, but we encountered a bug with it and do it.
		let mut quad = Rect::textured(
			[x, y + 1.0 * scale.1], [x + width, y + (9.0-AVOID_FP_ROUNDING) * scale.1], 
			self.color, 
			[tex_x + left, tex_y], [tex_x + add, tex_y + 1.0 / 16.0 - 0.5 / 256.0]
		).as_quad();
		
		quad.slant(if self.italic {scale.0} else {0.0});
		
		(quad, self.character.atlas())
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
	
	/// Renders text. Don't add the position for shadow, this does it for you. The Y coordinate is the coordinate of the baseline of the text. Coordinates are on a pixel scale.
	pub fn render<'b, I>(&self, x: f32, y: f32, text: I, shadow: bool, color: Rgb) -> Render<'a, 'b, I> where I: Iterator<Item=(&'b str, Style)> {
		let mut render = Render {
			metrics: self.metrics,
			source: text,
			shadow: shadow,
			start: (x, y),
			color: color,
			
			current: None
		};
		
		render.init();
		
		render
	}
	
	fn render_run<I>(&self, x: f32, y: f32, run: I, style: &Style, shadow: bool, color: Rgb) -> RenderRun<I> where I: Iterator<Item=char> {
		RenderRun {
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

pub struct Render<'a, 'b, I> where I: Iterator<Item=(&'b str, Style)> {
	source: I,
	current: Option<RenderRun<'a, ::std::str::Chars<'b>>>,
	metrics: &'a Metrics,
	shadow: bool,
	start: (f32, f32),
	color: Rgb,
}

// TODO: Remove code duplication from borrow checker stupidity.
impl<'a, 'b, I> Render<'a, 'b, I> where I: Iterator<Item=(&'b str, Style)> {
	fn init(&mut self) -> bool{
		if let Some((next_run, style)) = self.source.next() {
			self.current = Some(RenderRun {
				metrics: self.metrics,
				source: next_run.chars(),
				style: style,
				shadow: self.shadow,
				start: self.start,
				color: self.color,
						
				advance: 0.0,
				bonus: 0.0,
				state: RenderState::NextChar
			});
			
			true
		} else {
			false
		}
	}
}

impl<'a, 'b, I> Iterator for Render<'a, 'b, I> where I: Iterator<Item=(&'b str, Style)> {
	type Item = Option<Command>;
	
	fn next(&mut self) -> Option<Self::Item> {
		if let Some(ref mut current) = self.current {
			if let Some(next) = current.next() {
				Some(next)
			} else {
				if let Some((next_run, style)) = self.source.next() {
					*current = RenderRun {
						metrics: self.metrics,
						source: next_run.chars(),
						style: style,
						shadow: self.shadow,
						start: self.start,
						color: self.color,
						
						advance: current.advance,
						bonus: current.bonus,
						state: RenderState::NextChar
					};
					Some(None) // TODO
				} else {
					None
				}
			}
		} else {
			None
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
pub struct RenderRun<'a, I> where I: Iterator<Item=char> {
	// Data provided by original call
	metrics: &'a Metrics,
	source: I,
	style: Style,
	shadow: bool,
	start: (f32, f32),
	color: Rgb,
	
	// Data changed from iterations
	advance: f32,
	bonus: f32,
	state: RenderState,
}

impl<'a, I> Iterator for RenderRun<'a, I> where I: Iterator<Item=char> {
	type Item = Option<Command>;
	
	fn next(&mut self) -> Option<Self::Item> {
		let (x, y) = self.start;
		
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
		
		let color = if let style::Color::Palette(pal) = self.style.color {
			if self.shadow {
				pal.background()
			} else {
				pal.foreground()
			}
		} else {
			if self.shadow {
				Rgb::new(self.color.r() / 4, self.color.g() / 4, self.color.b() / 4)
			} else {
				self.color
			}
		}.to_linear();
		
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
				
				if !self.style.flags.bold() | (self.style.flags.bold() && bold) {
					self.advance += size.advance().floor();
					self.advance += if self.style.flags.bold() {1.0} else {0.0};
					self.bonus = size.advance().fract();
				}
				
				if c == ' ' {
					if !bold && self.style.flags.bold() {
						self.state = RenderState::Main(' ', true);
					} else {
						self.state = RenderState::NextChar;
					}
					// We don't render space characters, but rendering them wouldn't hurt anything. This is just an optimization.
					
					return Some(None)
				}
				
				let default_index = character_to_default(c);
				
				// TODO: Make obsfucate work in some way.
				
				// We consider a one pixel offset to be 0.5 for unicode, as the characters are 16x16 there, and 1.0 for default, as the characters are 8x8 there.
				// This differs from default to fix unicode bugs.
				let offset = if default_index.is_none() || self.metrics.always_unicode() {0.5} else {1.0};
				
				// We fix MC-14502 and MC-76356 here because Mojang is too busy adding useless parrots to fix bugs that are 2 years old.
				let (x_offset, y_offset) = match (self.shadow, bold) {
					(false, false) => (0.0, 0.0),
					(false, true)  => (offset, 0.0),
					(true, false)  => (offset, -offset),      // fixed MC-14502 - hidden shadows
					(true, true)   => (offset * 2.0, -offset) // fixed MC-76356 - doubled up bold characters
				};
				
				if bold {
					self.state = RenderState::NextChar;
				} else {
					self.state = if self.style.flags.bold() {
						RenderState::Main(c, true)
					} else {
						RenderState::NextChar
					};
				}
				
				Some(Command::draw_char(x + x_offset, y + y_offset, self.style.flags.italic(), c, self.metrics.always_unicode(), size, color))
			},
			RenderState::Strike => {
				let (x, y) = if self.shadow {(x + 1.0, y - 1.0)} else {(x, y)};
				
				self.state = if self.style.flags.underline() {RenderState::Under} else {RenderState::End};
				Some(Command::Rect { x: x, y: y + STRIKE_LEVEL, width: self.advance + self.bonus, height: 1.0, color})
			},
			RenderState::Under => {
				let (x, y) = if self.shadow {(x + 1.0, y - 1.0)} else {(x, y)};
				
				self.state = RenderState::End;
				Some(Command::Rect { x: x - 1.0, y: y + UNDER_LEVEL, width: self.advance + self.bonus + 1.0, height: 1.0, color})
			},
			RenderState::End => return None,
			RenderState::NextChar => unreachable!()
		})
	}
}
