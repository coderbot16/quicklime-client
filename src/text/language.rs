use text::formatter::{Kind, FormatCommand, Index, ParseFormatError};
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
	
	Ok((
		items.next().expect("A split iterator should yield at least one element! Go home Rust, you're drunk."), 
		try!(items.next().ok_or(Error::NoValue))
	))
}

#[derive(Debug)]
struct SimpleFormatCmd {
	string_start: usize,
	arg_index: usize,
	upper: bool
}

impl Display for SimpleFormatCmd {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		if self.upper {
			write!(f, "{{{}::to_uppercase}}", self.arg_index)
		} else {
			write!(f, "{{{}}}", self.arg_index)
		}
	}
}

// TODO: For specific formatting codes, we will want to make a Transformation system that transforms incoming primitives into strings.
// TODO: This should support JSON values, as that is what is can be used in `with`.

#[derive(Debug)]
pub enum ProcessError {
	UnsupportedFlags,
	UnsupportedWidth,
	UnsupportedPrecision,
	UnsupportedKind,
	Parse(ParseFormatError),
	NoPreviousArgument
}

struct CmdProcessor {
	head: usize,
	last: Option<usize>
}

impl CmdProcessor {
	fn new() -> Self {
		CmdProcessor {
			head: 1,
			last: None
		}
	}
	
	fn process(&mut self, string_start: usize, cmd: FormatCommand) -> Result<SimpleFormatCmd, ProcessError> {
		if cmd.kind == Kind::Decimal || cmd.kind == Kind::Float {
			// Plain decimal/float format codes are replaced with string format codes, for some reason. But why?
			self.last = Some(1);
			Ok(SimpleFormatCmd { string_start: string_start, arg_index: 0, upper: false })
			
		} else if cmd.flags.any() {
			Err(ProcessError::UnsupportedFlags)
		} else if cmd.width.is_some() {
			Err(ProcessError::UnsupportedWidth)
		} else if cmd.precision.is_some() {
			Err(ProcessError::UnsupportedPrecision)
		} else if cmd.kind != Kind::String {
			Err(ProcessError::UnsupportedKind)
		} else {
			self.process_lossy(string_start, cmd)
		}		
	}
	
	fn process_lossy(&mut self, string_start: usize, cmd: FormatCommand) -> Result<SimpleFormatCmd, ProcessError> {
		let current_idx = match cmd.index {
			Index::Previous => if let Some(last) = self.last {last} else {return Err(ProcessError::NoPreviousArgument)},
			Index::Exact(idx) => idx,
			Index::Next => self.head
		};
		
		if cmd.index == Index::Next {
			self.head += 1;
		}
		
		self.last = Some(current_idx);
		
		Ok(SimpleFormatCmd {
				string_start: string_start,
				arg_index: current_idx - 1, // Format string indices count from 1, but array indices count from 0.
				upper: cmd.upper
		})	
	}
}

#[derive(Debug)]
pub struct Compiled {
	string: String,
	commands: Vec<SimpleFormatCmd>
}

impl Compiled {
	pub fn compile(source: &str) -> Result<Self, ProcessError> {
		let mut processor = CmdProcessor::new();
		let mut compiled = Compiled { string: String::new(), commands: Vec::new() };
		
		let mut next = 0;
		
		for (index, c) in source.char_indices() {
			if index < next { continue };
			
			match c {
				'%' => {
					let (size, cmd) = try!(FormatCommand::parse(&source[index..]).map_err(ProcessError::Parse)); 
					
					next = index + size;
					
					if cmd.kind == Kind::Newline {
						compiled.string.push('\n');
					} else if cmd.kind == Kind::Percent {
						compiled.string.push('%');
					} else {
						compiled.commands.push(try!(processor.process(compiled.string.len(), cmd)));
					}
				},
				c => compiled.string.push(c)
			}
		}
		
		Ok(compiled)
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