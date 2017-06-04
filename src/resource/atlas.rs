use std::collections::HashMap;
use ui::lit::Lit;

#[derive(Serialize, Deserialize)]
pub struct TexmapBucket(pub HashMap<String, Texmap>);

#[derive(Serialize, Deserialize)]
pub struct Texmap(pub HashMap<String, TextureSelection>);

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureSelection {
	pub min: [Lit; 2],
	pub size: [Lit; 2]
}