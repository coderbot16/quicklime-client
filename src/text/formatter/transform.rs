use text::formatter::{FormatCommand, Kind, Flag};

#[derive(Debug, Clone, PartialEq)]
enum Prim {
	Bool(bool),
	Null,
	Undefined,
	Integer(i64),
	Float(f64),
	String(String)
}

enum PaddingStrategy {
	/// [  -123]
	RightAlign,
	/// [-123  ]
	LeftAlign,
	/// [-00123]
	ZeroPad
}

enum Sign {
	/// For negative numbers: "-num". For positive numbers: "num"
	Minus,
	/// For negative numbers: "(num)". For positive numbers: "num"
	Surround,
	/// For negative numbers: "-num". For positive numbers: "+num"
	PositivePlus,
	/// For negative numbers: "-num". For positive numbers: " num"
	PositiveSpace
}

struct NumericRep {
	sign: Sign,
	group: bool
}

struct Target {
	min_width: usize,
	precision: Option<usize>,
	strategy: PaddingStrategy,
	repr: NumericRep,
	trans: Trans
}

enum Error {
	ConflictingFlags(Flag, Flag),
	UnsupportedKind(Kind),
	NoAlternate,
	Escape
}

impl Target {
	fn from_unchecked(cmd: &FormatCommand) -> Result<Self, Error> {
		// TODO: Alternate
		
		Ok(Target {
			min_width: cmd.width.unwrap_or(0),
			precision: cmd.precision,
			strategy: match (cmd.flags.zero_pad(), cmd.flags.left_justify()) {
				(true, false) => PaddingStrategy::ZeroPad,
				(false, true) => PaddingStrategy::LeftAlign,
				(false, false) => PaddingStrategy::RightAlign,
				(true, true) => return Err(Error::ConflictingFlags(Flag::ZeroPad, Flag::LeftJustify))
			},
			repr: NumericRep { 
				sign: {
					if cmd.flags.parentheses() {
						Sign::Surround
					} else if cmd.flags.plus() {
						if cmd.flags.leading_space() {
							return Err(Error::ConflictingFlags(Flag::Plus, Flag::LeadingSpace))
						}
						
						Sign::PositivePlus
					} else if cmd.flags.leading_space() {
						Sign::PositiveSpace
					} else {
						Sign::Minus
					}
				}, 
				group: cmd.flags.group() 
			},
			trans: match (cmd.kind, cmd.flags.alternate()) {
				(Kind::Bool, false) => Trans::Bool,
				(Kind::Bool, true) => return Err(Error::NoAlternate),
				// TODO: Kind::HexHashCode: NoAlternate
				// TODO: Kind::String: NoAlternate
				// TODO: Kind::Character: NoAlternate
				(Kind::Decimal, false) => Trans::Decimal,
				(Kind::Decimal, true) => return Err(Error::NoAlternate),
				(Kind::Octal, flag) => Trans::Octal { has_radix: flag },
				(Kind::Hex, flag) => Trans::Hex { has_radix: flag },
				(Kind::SciNot, false) => Trans::SciNot,
				(Kind::SciNot, true) => return Err(Error::NoAlternate),
				(Kind::Float, flag) => Trans::Float { force_decimal: flag },
				(Kind::CompSciNot, flag) => Trans::CompSciNot { force_decimal: flag },
				(Kind::Hexfloat, flag) => Trans::Hexfloat { force_decimal: flag },
				// TODO: Time
				(Kind::Percent, _) => return Err(Error::Escape),
				(Kind::Newline, _) => return Err(Error::Escape),
				_ => return Err(Error::UnsupportedKind(cmd.kind))
			}
		})
	}
	
	// TODO: precision for Kind::Bool, Kind::HexHashCode, Kind::String is max_chars
	// TODO: precision for [all FP] is TODO
	// TODO: precision for Kind::Unciode, Decimal, Octal, Hex is invalid
	
	
	fn from(cmd: &FormatCommand) -> Result<Self, Error> {
		let target = Self::from_unchecked(cmd)?;
		
		// check kind is Octal, Hex, [all FP] if Alt
		// check kind is Decimal, [all FP] if Sign::PositivePlus or Sign::PositiveSpace
		// check kind is [all Numeric] if PaddingStrategy::ZeroPad,
		// check kind is Decimal, SciNot, Float, CompSciNot if group==true or Sign::Surround
		// Kind::Octal / Kind::Hex: Reject sign if non Sign::Minus, reject if group==true.
		// TODO: Floats and Doubles, Time
		
		Ok(target)
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Trans {
	Bool,
	// NUMERIC
	Decimal,
	Octal { has_radix: bool },
	Hex { has_radix: bool },
	// NUMERIC-FP
	SciNot,
	Float { force_decimal: bool },
	CompSciNot { force_decimal: bool },
	Hexfloat { force_decimal: bool },
}

fn trans(transform: &Trans, prim: &Prim) -> &'static str {
	match (transform, prim) {
		(&Trans::Bool, &Prim::Bool(b)) => if b {"true"} else {"false"},
		(&Trans::Bool, &Prim::Null) => "false",
		(&Trans::Bool, &Prim::Undefined) => "false",
		(&Trans::Bool, _) => "true",
		_ => panic!()
	}
}