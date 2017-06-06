#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Rgb(u32);

impl Rgb {
	/// Creates a new RGB value, where the values are in the Srgb color set.
	pub fn new(r: u8, g: u8, b: u8) -> Self {
		Rgb(
			((r as u32) << 16) | 
			((g as u32) << 8) | 
			(b as u32)
		)
	}
	
	pub fn r(&self) -> u8 {
		(self.0 >> 16) as u8
	}
	
	pub fn g(&self) -> u8 {
		(self.0 >> 8) as u8
	}
	
	pub fn b(&self) -> u8 {
		self.0 as u8
	}
	
	pub fn to_srgb(&self) -> [f32; 3] {
		[
			(self.r() as f32) / 255.0,
			(self.g() as f32) / 255.0,
			(self.b() as f32) / 255.0
		]
	}
	
	pub fn to_linear(&self) -> [f32; 3] {
		let srgb = self.to_srgb();
		
		[
			srgb[0].powf(2.2),
			srgb[1].powf(2.2),
			srgb[2].powf(2.2)
		]
	}
	
	pub fn to_rgba(&self, alpha: u8) -> Rgba {
		Rgba(self.0 | ((alpha as u32) << 24))
	}
}

#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Rgba(u32);

impl Rgba {
	/// Creates a new RGB value, where the values are in the Srgb color set.
	pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
		Rgba(
			((a as u32) << 24) |
			((r as u32) << 16) | 
			((g as u32) << 8) | 
			(b as u32)
		)
	}
	
	pub fn r(&self) -> u8 {
		(self.0 >> 16) as u8
	}
	
	pub fn g(&self) -> u8 {
		(self.0 >> 8) as u8
	}
	
	pub fn b(&self) -> u8 {
		self.0 as u8
	}
	
	
	pub fn a(&self) -> u8 {
		(self.0 >> 24) as u8
	}
	
	pub fn to_srgb(&self) -> [f32; 4] {
		[
			(self.r() as f32) / 255.0,
			(self.g() as f32) / 255.0,
			(self.b() as f32) / 255.0,
			(self.a() as f32) / 255.0
		]
	}
	
	pub fn to_linear(&self) -> [f32; 4] {
		let srgb = self.to_srgb();
		
		[
			srgb[0].powf(2.2),
			srgb[1].powf(2.2),
			srgb[2].powf(2.2),
			srgb[3]
		]
	}
	
	pub fn to_rgb(&self) -> Rgb {
		Rgb(self.0 & 0xFFFFFF)
	}
}