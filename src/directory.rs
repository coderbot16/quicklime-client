use std::collections::HashMap;
use std::str::Split;
use std::ops::FnMut;
use std::collections::hash_map::Iter;

#[derive(Debug)]
pub struct Directory<V> {
	root: Node<V>
}

impl<V> Directory<V> {
	pub fn new() -> Self {
		Directory { root: Node::dummy() }
	}
	
	pub fn get(&self, key: &str) -> Option<&V> {
		let mut node = &self.root;
		
		for part in key.split(".") {
			if let Some(deeper) = node.deeper(part) {
				node = deeper;
			} else {
				return None;
			}
		}
		
		node.get()
	}
	
	pub fn insert(&mut self, key: &str, value: V) {
		Self::insert_helper(&mut self.root, key.split('.'), value)
	}
	
	pub fn root(&mut self) -> &Node<V> {
		&self.root
	}
	
	// Avoid pissing off the borrow checker
	fn insert_helper<'a, I>(node: &mut Node<V>, mut current: I, value: V) where I: Iterator<Item=&'a str> {
		if let Some(part) =  current.next() {
			if let Some(deeper) = node.deeper_mut(part) {
				Self::insert_helper(deeper, current, value);
				return;
			}
			
			node.insert(part, Node::dummy());
			Self::insert_helper(node.deeper_mut(part).unwrap(), current, value)
		} else {
			node.set_leaf(value)
		}
	}
}

#[derive(Debug)]
pub struct Node<V> {
	branch: Option<HashMap<String, Node<V>>>,
	leaf: Option<V>
}

impl<V> Node<V> {
	fn dummy() -> Self {
		Node { branch: None, leaf: None }
	}
	
	fn leaf(value: V) -> Self {
		Node { branch: None, leaf: Some(value) }
	}
	
	fn branch(map: HashMap<String, Node<V>>) -> Self {
		Node { branch: Some(map), leaf: None}
	}
	
	fn set_leaf(&mut self, value: V) {
		// TODO: Use entry APIs when they are stabilized.
		if let Some(ref mut current) = self.leaf {
			*current = value
		} else {
			self.leaf = Some(value)
		}
	}
	
	fn insert(&mut self, key: &str, node: Node<V>) {
		// TODO: Use entry APIs when they are stabilized.
		if let Some(ref mut br) = self.branch {
			br.insert(key.to_owned(), node);
		} else {
			let mut map = HashMap::new();
			map.insert(key.to_owned(), node);
			self.branch = Some(map);
		}
	}
	
	pub fn deeper(&self, key: &str) -> Option<&Node<V>> {
		if let Some(ref map) = self.branch {
			map.get(key)
		} else {
			None
		}
	}
	
	fn deeper_mut(&mut self, key: &str) -> Option<&mut Node<V>> {
		if let Some(ref mut map) = self.branch {
			map.get_mut(key)
		} else {
			None
		}
	}
	
	pub fn get(&self) -> Option<&V> {
		self.leaf.as_ref()
	}
	
	fn get_mut(&mut self) -> Option<&mut V> {
		self.leaf.as_mut()
	}
	
	pub fn iter(&self) -> Option<Iter<String, Node<V>>> {
		if let Some(ref br) = self.branch {
			Some(br.iter())
		} else {
			None
		}
	}
}