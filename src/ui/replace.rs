enum Ty {
	Num,
	Str,
	Seq,
	Map
}

pub struct IncompleteScene {
	parameters: HashMap<String, Ty>,
	elements: Value,
	inputs: Value
}