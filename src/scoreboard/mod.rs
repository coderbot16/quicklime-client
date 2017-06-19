mod team;
mod objective;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use scoreboard::team::Team;

type Identifier = Rc<String>;

enum DisplaySlot {
	PlayerList,
	Sidebar,
	BelowName
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum EntityIdentifier {
	Player(String),
	Entity([u8; 16])
}

enum Mode {
	Never,
	Enemy,
	Friendly,
	Always
}

// TODO: FriendlyFlags, NameTagVis, Collision

struct TeamId(usize);

struct Manager {
	
	teams: HashMap<Rc<String>, Rc<RefCell<Team>>>,
	entities: HashMap<EntityIdentifier, (Rc<String>, Rc<RefCell<Team>>)>,
}