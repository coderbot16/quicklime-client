use language::Directory;

#[derive(Copy, Clone)]
enum State {
	Main,
	Translating(usize),
	End
}

struct Processor<'a, I> where I: Iterator<Item=Component<'a>> {
	lang: Option<Directory>,
	source: I,
	state: Vec<State>
}

/*impl<'a, I> Iterator for Processor<'a, I> where I: Iterator<Item=Component<'a>> {
	type Item = Result<Component<'a>, ComponentBuf>;
	
	fn next(&mut self) -> Option<Self::Item> {
		Some(match self.state {
			State::Main => {
				match self.source.next() {
					Some(component) => {
						match component.kind {
							Kind::Text => Ok(component),
							Kind::Translate => 
						}
					},
					None => {self.state = State::End; return None}
				}
			},
			State::Translating => unimplemented!(),
			State::End => return None
		})
	}
}*/