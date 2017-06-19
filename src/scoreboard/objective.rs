use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use text::flat::ChatBuf;
use scoreboard::{Identifier, EntityIdentifier};

type Objective = Rc<RefCell<ObjectiveData>>;

struct ObjectiveManager {
	objectives:  HashMap<Identifier, Objective>,
	player_list: Option<Objective>,
	sidebar:     Option<Objective>,
	below_name:  Option<Objective>
}

struct ObjectiveData {
	identifier: Identifier,
	name: ChatBuf,
	display: DisplayType,
	scores: HashMap<EntityIdentifier, i32>
}

impl ObjectiveData {
	fn score(&self, id: &EntityIdentifier) -> Option<i32> {
		self.scores.get(id).map(|v| *v)
	}
	
	fn update(&mut self, id: EntityIdentifier, value: i32) -> Option<i32> {
		self.scores.insert(id, value)
	}
}

enum DisplayType {
	Integer,
	Hearts
}