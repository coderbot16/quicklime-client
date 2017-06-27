use serde_json::Value;
use serde_json::map::Map;
use std::collections::HashMap;
use ui::Scene;
use serde_json;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Ty {
	Num,
	Str,
	Seq,
	Map
}

#[derive(Serialize, Deserialize)]
pub struct IncompleteScene {
	parameters: HashMap<String, Ty>,
	elements: Value,
	inputs: Value
}

impl IncompleteScene {
	pub fn complete(&self, data: &HashMap<String, Value>) -> Result<Scene, Error> {
		Ok(Scene {
			cursor: None,
			elements: complete(&self.elements, data, &self.parameters).and_then(|v| serde_json::from_value(v).map_err(Error::Json))?,
			inputs: complete(&self.inputs, data, &self.parameters).and_then(|v| serde_json::from_value(v).map_err(Error::Json))?
		})
	}
}

#[derive(Debug)]
pub enum Error {
	BadRef(String),
	LookupFailed(String),
	SceneLookupFailed(String),
	TyMismatch { name: String, expected: Ty },
	BadInsertTy { string: String, got: Ty, expected: Ty },
	Json (serde_json::Error)
}

pub fn complete(value: &Value, data: &HashMap<String, Value>, tys: &HashMap<String, Ty>) -> Result<Value, Error> {
	Ok(match *value {
		Value::Null => Value::Null,
		Value::Bool(b) => Value::Bool(b),
		Value::Number(ref n) => Value::Number(n.clone()),
		Value::String(ref s) => {
			if s.len() >= 2 && s.starts_with('%') && s.ends_with('%') {
				// Replacement
				let key = &s[1..s.len()-1];
				
				if key.len() == 0 {
					return Ok(Value::String("%".to_owned()));
				} else if key.len() >= 2 && key.starts_with('%') && s.ends_with('%') {
					return Ok(Value::String(key.to_owned()));
				}
				
				let ty = tys.get(key).ok_or_else(|| Error::BadRef(key.to_owned()))?;
				let replacement = data.get(key).ok_or_else(|| Error::LookupFailed(key.to_owned()))?;
				
				// TODO: TypeCheck
				
				replacement.clone()
			} else if s.contains('$') {
				let mut in_flag = false;
				let mut new = String::with_capacity(s.len());
				
				for term in s.split('$') {
					if in_flag {
						if term.is_empty() {
							new.push_str("$");
							in_flag = false;
							continue;
						}
						
						match tys.get(term) {
							Some(&Ty::Str) | Some(&Ty::Num) => match data.get(term) {
								Some(v) => new.push_str(&format!("{}", v)),
								None => return Err(Error::LookupFailed(term.to_owned()))
							},
							Some(&other) => return Err(Error::BadInsertTy { string: s.to_owned(), got: other, expected: Ty::Str}),
							None => return Err(Error::BadRef(term.to_owned()))
						}
					} else {
						new.push_str(term)
					}
					
					in_flag = !in_flag;
				}
				Value::String(new)
			} else {
				Value::String(s.clone())
			}
		},
		Value::Array(ref vec) => {
			let mut modified = Vec::with_capacity(vec.len());
			
			for entry in vec {
				modified.push(if let &Value::String(ref string) = entry {
					if string.len() >= 2 && string.starts_with('$') && string.ends_with('$') {
						let key = &string[1..string.len()-1];
						
						if key.len() == 0 {
							Value::String("$".to_owned())
						} else if key.len() >= 2 && key.starts_with('$') && string.ends_with('$') {
							Value::String(key.to_owned())
						} else if key.contains('$') {
							complete(entry, data, tys)?
						} else {
							let ty = tys.get(key).ok_or_else(|| Error::BadRef(key.to_owned()))?;
							let insert = data.get(key).ok_or_else(|| Error::LookupFailed(key.to_owned()))?;
				
							// TODO: TypeCheck
							unimplemented!()
						}
					} else {
						complete(entry, data, tys)?
					}
				} else {
					complete(entry, data, tys)?
				})
			}
			
			Value::Array(modified)
		},
		Value::Object(ref map) => {
			let mut modified = Map::with_capacity(map.len());
			
			for (key, entry) in map {
				modified.insert(key.to_owned(), if let &Value::String(ref string) = entry {
					// TODO: Make this not break on strings like "$bill$ + $bob$"
					if string.len() >= 2 && string.starts_with('$') && string.ends_with('$') {
						let key = &string[1..string.len()-1];
						
						if key.len() == 0 {
							Value::String("$".to_owned())
						} else if key.len() >= 2 && key.starts_with('$') && string.ends_with('$') {
							Value::String(key.to_owned())
						} else if key.contains('$') {
							complete(entry, data, tys)?
						} else {
							let ty = tys.get(key).ok_or_else(|| Error::BadRef(key.to_owned()))?;
							let insert = data.get(key).ok_or_else(|| Error::LookupFailed(key.to_owned()))?;
				
							// TODO: TypeCheck
							unimplemented!()
						}
					} else {
						complete(entry, data, tys)?
					}
				} else {
					complete(entry, data, tys)?
				});
			}
			
			Value::Object(modified)
		}
	})
}

#[test]
fn test() {
	let mut data = HashMap::new();
	data.insert("str".to_owned(), Value::String("Test String".to_owned()));
	data.insert("num".to_owned(), Value::Number(21343.into()));
	data.insert("bool".to_owned(), Value::Bool(true));
	data.insert("null".to_owned(), Value::Null);
	data.insert("array".to_owned(), Value::Array(vec![1.into(), 2.into(), 3.into(), 4.into()]));
	
	let mut tys = HashMap::new();
	tys.insert("str".to_owned(), Ty::Str);
	tys.insert("num".to_owned(), Ty::Num);
	tys.insert("bool".to_owned(), Ty::Str);
	tys.insert("null".to_owned(), Ty::Str);
	tys.insert("array".to_owned(), Ty::Seq);
	
	assert_eq!(Value::Number(21343.into()), complete(&Value::String("%num%".to_owned()), &data, &tys).unwrap());
	assert_eq!(Value::Array(vec![1.into(), 2.into(), 3.into(), 4.into()]), complete(&Value::String("%array%".to_owned()), &data, &tys).unwrap());

	let subject = json! ([
		"An innocent value",
		"%num%",
		"Pls no replace kthxbye",
		"%array%",
		"%null%"
	]);
	
	let result = json!([
		"An innocent value",
		21343,
		"Pls no replace kthxbye",
		[1, 2, 3, 4],
		null
	]);
	
	assert_eq!(Ok(result), complete(&subject, &data, &tys));
}

#[test]
fn test_escape() {
	let data = HashMap::new();
	let tys = HashMap::new();
	
	assert_eq!(Value::String("%escape%".to_owned()), complete(&Value::String("%%escape%%".to_owned()), &data, &tys).unwrap());
	assert_eq!(Value::String("%".to_owned()), complete(&Value::String("%%".to_owned()), &data, &tys).unwrap());
	assert_eq!(Value::String("an $escaped$ string $.".to_owned()), complete(&Value::String("an $$escaped$$ string $$.".to_owned()), &data, &tys).unwrap());
}