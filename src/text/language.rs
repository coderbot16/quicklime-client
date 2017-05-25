use text::formatter::{Kind, FormatCommand, Index};
use std::collections::HashMap;

struct Node {
	branch: Option<HashMap<String, Node>>,
	leaf: Option<Compiled>
}

impl Node {
	fn leaf(compiled: Compiled) -> Self {
		Node { branch: None, leaf: Some(compiled) }
	}
	
	fn branch(map: HashMap<String, Node>) -> Self {
		Node { branch: Some(map), leaf: None}
	}
	
	fn set_leaf(&mut self, compiled: Compiled) {
		// TODO: Use entry APIs when they are stabilized.
		if let Some(ref mut current) = self.leaf {
			*current = compiled
		} else {
			self.leaf = Some(compiled)
		}
	}
	
	fn insert(&mut self, key: &str, node: Node) {
		// TODO: Use entry APIs when they are stabilized.
		if let Some(ref mut br) = self.branch {
			br.insert(key.to_owned(), node);
		} else {
			let mut map = HashMap::new();
			map.insert(key.to_owned(), node);
			self.branch = Some(map);
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Error {
	Comment,
	NoValue
}

fn parse_line(line: &str) -> Result<(&str, &str), Error> {
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

struct SimpleFormatCmd {
	string_start: usize,
	arg_index: usize
}

struct CmdProcessor {
	head: usize,
	last: usize
}

impl CmdProcessor {
	fn new() -> Self {
		CmdProcessor {
			head: 0,
			last: 0
		}
	}
	
	fn process(&mut self, string_start: usize, cmd: FormatCommand) -> Option<SimpleFormatCmd> {
		if cmd.flags.any() | cmd.width.is_some() | cmd.precision.is_some() | (cmd.kind != Kind::String && cmd.kind != Kind::Decimal && cmd.kind != Kind::Float) {
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
				arg_index: current_idx
		}	
	}
}

struct Compiled {
	string: String,
	commands: Vec<SimpleFormatCmd>
}

impl Compiled {
	fn compile(source: &str) -> Option<Self> {
		let mut processor = CmdProcessor::new();
		let mut compiled = Compiled { string: String::new(), commands: Vec::new() };
		
		let mut next = 0;
		for (index, c) in source.char_indices() {
			if index < next { continue };
			
			match c {
				'%' => {
					if let Ok((size, cmd)) = FormatCommand::parse(&source[index..]) {
						next = index + size;
						
						if let Some(cmd) = processor.process(compiled.string.len(), cmd) {
							compiled.commands.push(cmd)
						} else {
							return None;
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

#[test]
fn test_parse_lines() {
	assert_eq!(Ok(("translation.test.none", "Hello, world!")), parse_line("translation.test.none=Hello, world!"));
	assert_eq!(Ok(("translation.test.none", "Hello, world!")), parse_line("translation.test.none=Hello, world!=whatever"));
	assert_eq!(Ok((" translation.test.none", "Hello, world! ")), parse_line(" translation.test.none=Hello, world! "));
	assert_eq!(Err(Error::Comment), parse_line("# This is an interesting comment."));
	assert_eq!(Err(Error::NoValue), parse_line("I'm a strong, independent key and ain't no value gonna mess with me."));
	assert_eq!(Err(Error::NoValue), parse_line(""));
}