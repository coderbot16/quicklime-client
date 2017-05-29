use text::formatter::{Kind, FormatCommand, Index, ParseFormatError};
use directory;
use std::fmt::{self, Formatter, Display};
use std::io::{self, Read, BufReader, BufRead, Write};
use resource::Asset;

pub type Directory = directory::Directory<Result<Compiled, LoadError>>;
pub type Node = directory::Node<Result<Compiled, LoadError>>;

impl Asset for Directory {
	type Err = Error;
	type WErr = ();
	
	fn load<R>(read: &mut R, name: &str) -> Result<Self, Self::Err> where R: Read {
		let read = BufReader::new(read);
		
		let mut dir = Directory::new();
		let mut line_number = 0;
		
		for line in read.lines() {
			let line = try!(line.map_err(Error::Io));
			
			if let Some((key, raw)) = try!(parse_line(&line)) {
				dir.insert(key, Compiled::compile(raw).map_err(
					|(index, err)| LoadError {
						err: err, 
						file: name.to_owned(), 
						line: line_number,
						text: line.clone(), 
						index: index + key.len() + 1
					}
				))
			};
			
			line_number += 1;
		}
		
		Ok(dir)
	}
	
	fn save<W>(&self, write: &mut W) -> Result<(), Self::WErr> where W: Write {
		unimplemented!()
	}
}

pub struct LoadError {
	err: ProcessError,
	file: String,
	line: usize,
	index: usize,
	text: String
}

impl Display for LoadError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		try!(writeln!(f, "error: {}", self.err));
		
		try!(writeln!(f, "  --> {}:{}:{}", "assets/minecraft/lang/en_US.lang", self.line+1, self.index+1));
				
		let num = format!("{}", self.line+1);
				
		for _ in 0..num.len() {try!(write!(f, " "))};
		try!(writeln!(f, " |"));
					
		try!(writeln!(f, "{} | {}", num, self.text));
					
		for _ in 0..num.len() {try!(write!(f, " "))};
		try!(write!(f, " | "));
		for _ in 0..self.index {
			try!(write!(f, " "))
		}
						
		try!(writeln!(f, "^"));
		
		for _ in 0..num.len() {try!(write!(f, " "))};
		try!(writeln!(f, " |"));
		
		if let Some(help) = self.err.help() {
			for _ in 0..num.len() {try!(write!(f, " "))};
			try!(writeln!(f, " = help: {}", help))
		}
		
		Ok(())
	}
}

#[derive(Debug)]
pub enum Error {
	Io(io::Error),
	NoValue
}

pub fn parse_line(line: &str) -> Result<Option<(&str, &str)>, Error> {
	if line.is_empty() || line.starts_with("#") {
		// Comment
		return Ok(None);
	}
	
	let mut items = line.split("=");
	
	Ok(Some((
		items.next().expect("A split iterator should yield at least one element! Go home Rust, you're drunk."), 
		try!(items.next().ok_or(Error::NoValue))
	)))
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
	UnsupportedKind(Kind, bool),
	Parse(ParseFormatError),
	NoPreviousArgument
}

impl ProcessError {
	pub fn help(&self) -> Option<&'static str> {
		match self {
			&ProcessError::UnsupportedFlags 	 => Some("the formatting system does not support this feature yet"),
			&ProcessError::UnsupportedWidth 	 => Some("the formatting system does not support this feature yet"),
			&ProcessError::UnsupportedPrecision  => Some("the formatting system does not support this feature yet"),
			&ProcessError::UnsupportedKind(k, _) => Some("the formatting system does not support this feature yet"),
			&ProcessError::Parse(ref p) 			 => p.help(),
			&ProcessError::NoPreviousArgument 	 => Some("please use relative indexing (%s) or exact indexing (%1$s) for this format code")
		}
	}
}

impl Display for ProcessError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		match self {
			&ProcessError::UnsupportedFlags => write!(f, "flags are not currently supported"),
			&ProcessError::UnsupportedWidth => write!(f, "width is not currently supported"),
			&ProcessError::UnsupportedPrecision => write!(f, "precision is not currently supported"),
			&ProcessError::UnsupportedKind(k, upper) => write!(f, "kind {:?} [%{}] is not currently supported", k, k.character(upper)),
			&ProcessError::Parse(ref p) => write!(f, "while parsing format: {}", p),
			&ProcessError::NoPreviousArgument => write!(f, "previous argument flag used ('<'), but there is no previous argument")
		}
	}
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
			Err(ProcessError::UnsupportedKind(cmd.kind, cmd.upper))
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
	pub fn compile(source: &str) -> Result<Self, (usize, ProcessError)> {
		let mut processor = CmdProcessor::new();
		let mut compiled = Compiled { string: String::new(), commands: Vec::new() };
		
		let mut next = 0;
		
		for (index, c) in source.char_indices() {
			if index < next { continue };
			
			match c {
				'%' => {
					let (size, cmd) = try!(FormatCommand::parse(&source[index..]).map_err(|e| (index, ProcessError::Parse(e)))); 
					
					next = index + size;
					
					if cmd.kind == Kind::Newline {
						compiled.string.push('\n');
					} else if cmd.kind == Kind::Percent {
						compiled.string.push('%');
					} else {
						compiled.commands.push(try!(processor.process(compiled.string.len(), cmd).map_err(|e| (index, e))));
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