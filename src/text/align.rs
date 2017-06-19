pub enum Align {
	Left,
	Center,
	Right
}

impl Align {
	pub fn start_x(&self, x_min: f32, x_max: f32, width: f32) -> f32 {
		match *self {
			Align::Left => x_min,
			Align::Center => (x_min + x_max - width) / 2.0,
			Align::Right => x_max - width
		}
	}
}

#[test]
fn test_left() {
	let area_width = 128.0;
	let str_width = 32.0;
	
	let align = Align::Left;
	for x in 0..256 {
		let x = x as f32;
		
		let start_x = align.start_x(x, x+area_width, str_width);
		let x_max = x + area_width;
		
		assert_eq!(x, start_x);
		assert_eq!(x_max - start_x, area_width);
	}
}

#[test]
fn test_right() {
	let area_width = 128.0;
	let str_width = 32.0;
	
	let align = Align::Right;
	for x in 0..256 {
		let x = x as f32;
		
		let start_x = align.start_x(x, x+area_width, str_width);
		let x_max = x + area_width;
		
		assert_eq!(x_max - start_x, str_width);
		assert_eq!(start_x - x, area_width - str_width);
	}
}

#[test]
fn test_center() {
	let area_width = 128.0;
	let str_width = 32.0;
	
	let align = Align::Center;
	for x in 0..256 {
		let x = x as f32;
		
		let start_x = align.start_x(x, x+area_width, str_width);
		let x_max = x + area_width;
		
		assert_eq!(start_x - x, x_max - (start_x + str_width));
	}
}