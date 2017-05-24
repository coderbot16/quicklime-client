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

// TODO: Format strings

#[test]
fn test_parse_lines() {
	assert_eq!(Ok(("translation.test.none", "Hello, world!")), parse_line("translation.test.none=Hello, world!"));
	assert_eq!(Ok(("translation.test.none", "Hello, world!")), parse_line("translation.test.none=Hello, world!=whatever"));
	assert_eq!(Ok((" translation.test.none", "Hello, world! ")), parse_line(" translation.test.none=Hello, world! "));
	assert_eq!(Err(Error::Comment), parse_line("# This is an interesting comment."));
	assert_eq!(Err(Error::NoValue), parse_line("I'm a strong, independent key and ain't no value gonna mess with me."));
}