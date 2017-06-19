use std::collections::HashMap;
use ui::lit::Lit;

#[derive(Serialize, Deserialize)]
pub struct TexmapBucket(pub HashMap<String, Texmap>);

#[derive(Serialize, Deserialize)]
pub struct Texmap(pub HashMap<String, TextureSelection>);

impl Texmap {
	pub fn new(name: String) -> Self {
		let mut map = HashMap::new();
		
		map.insert(name, TextureSelection::identity());
		
		Texmap(map)
	}
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
/// A selection in UV space: bottom left is [0.0, 0.0], top right is [1.0, 1.0]
pub struct TextureSelection {
	pub min: [Lit; 2],
	pub size: [Lit; 2]
}

impl TextureSelection {
	pub fn identity() -> Self {
		TextureSelection {
			min: [Lit::from_part(0.0), Lit::from_part(0.0)], 
			size: [Lit::from_part(1.0), Lit::from_part(1.0)]
		}
	}
}