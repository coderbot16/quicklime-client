use text::formatter::{Kind, FormatCommand, Index};
use directory;
use std::fmt::{self, Formatter, Display};

pub type Directory = directory::Directory<Compiled>;
pub type Node = directory::Node<Compiled>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
	Comment,
	NoValue
}

pub fn parse_line(line: &str) -> Result<(&str, &str), Error> {
	if line.starts_with("#") {
		// Comment
		return Err(Error::Comment);
	}
	
	let mut items = line.split("=");
	
	// TODO: Formatting
	// TODO: Formatting: Any Decimal or Float format code is replaced with a format code equivalent to %1s
	
	Ok((
		items.next().expect("A split iterator should yield at least one element!"), 
		try!(items.next().ok_or(Error::NoValue))
	))
}

#[derive(Debug)]
struct SimpleFormatCmd {
	string_start: usize,
	arg_index: usize
}

impl Display for SimpleFormatCmd {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		write!(f, "{{{}}}", self.arg_index)
	}
}

struct CmdProcessor {
	head: usize,
	last: usize
}

impl CmdProcessor {
	fn new() -> Self {
		CmdProcessor {
			head: 1,
			last: 1
		}
	}
	
	fn process(&mut self, string_start: usize, cmd: FormatCommand) -> Option<SimpleFormatCmd> {
		if cmd.flags.any() | cmd.width.is_some() | cmd.precision.is_some() | (cmd.kind != Kind::String && cmd.kind != Kind::Decimal && cmd.kind != Kind::Float) || cmd.upper {
			None
		} else {
			Some(self.process_lossy(string_start, cmd))
		}		
	}
	
	fn process_lossy(&mut self, string_start: usize, cmd: FormatCommand) -> SimpleFormatCmd {
		let current_idx = match cmd.index {
			Index::Previous => self.last,
			Index::Exact(idx) => idx,
			Index::Next => self.head
		};
		
		if cmd.index == Index::Next {
			self.head += 1;
		}
		
		self.last = current_idx;
		
		SimpleFormatCmd {
				string_start: string_start,
				arg_index: current_idx - 1 // Format string indices count from 1, but array indices count from 0.
		}	
	}
}

#[derive(Debug)]
pub struct Compiled {
	string: String,
	commands: Vec<SimpleFormatCmd>
}

impl Compiled {
	pub fn compile(source: &str) -> Option<Self> {
		let mut processor = CmdProcessor::new();
		let mut compiled = Compiled { string: String::new(), commands: Vec::new() };
		
		let mut next = 0;
		
		for (index, c) in source.char_indices() {
			if index < next { continue };
			
			match c {
				'%' => {
					if let Ok((size, cmd)) = FormatCommand::parse(&source[index..]) {
						next = index + size;
						
						if cmd.kind == Kind::Newline {
							compiled.string.push('\n');
						} else if cmd.kind == Kind::Percent {
							compiled.string.push('%');
						} else {
							if let Some(cmd) = processor.process(compiled.string.len(), cmd) {
								compiled.commands.push(cmd)
							} else {
								return None;
							}
						}
					} else {
						return None;
					}
				},
				c => compiled.string.push(c)
			}
		}
		
		Some(compiled)
	}
}

impl Display for Compiled {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		let mut base = 0;
		let mut string = self.string.clone();
		
		for cmd in self.commands.iter() {
			let command = format!("{}", cmd);
			string.insert_str(base + cmd.string_start, &command);
			base += command.len();
		}
		
		write!(f, "\"{}\"", string)
	}
}

#[test]
fn test_parse_lines() {
	assert_eq!(Ok(("translation.test.none", "Hello, world!")), parse_line("translation.test.none=Hello, world!"));
	assert_eq!(Ok(("translation.test.none", "Hello, world!")), parse_line("translation.test.none=Hello, world!=whatever"));
	assert_eq!(Ok((" translation.test.none", "Hello, world! ")), parse_line(" translation.test.none=Hello, world! "));
	assert_eq!(Err(Error::Comment), parse_line("# This is an interesting comment."));
	assert_eq!(Err(Error::NoValue), parse_line("I'm a strong, independent key and ain't no value gonna mess with me."));
	assert_eq!(Err(Error::NoValue), parse_line(""));
}