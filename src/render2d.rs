pub type Color = [f32; 3];

#[derive(Copy, Clone, Debug)]
pub struct Vertex2D {
	pub pos: [f32; 2],
	pub tex: [f32; 2],
	pub color: Color
}

pub struct Rect {
	min: Vertex2D,
	max: Vertex2D,
	plus_x_color: Color,
	plus_y_color: Color
}

impl Rect {
	pub fn solid(min: [f32; 2], max: [f32; 2], color: Color) -> Self {
		Rect {
			min: Vertex2D {pos: min, tex: [0.0, 0.0], color: color},
			max: Vertex2D {pos: max, tex: [0.0, 0.0], color: color},
			plus_x_color: color,
			plus_y_color: color
		}
	}
	
	pub fn textured(min: [f32; 2], max: [f32; 2], color: Color, min_tex: [f32; 2], max_tex: [f32; 2]) -> Self {
		Rect {
			min: Vertex2D {pos: min, tex: min_tex, color: color},
			max: Vertex2D {pos: max, tex: max_tex, color: color},
			plus_x_color: color,
			plus_y_color: color
		}
	}
	
	pub fn as_quad(&self) -> Quad {
		Quad([
			self.min,
			Vertex2D {pos: [self.max.pos[0], self.min.pos[1]], tex: [self.max.tex[0], self.min.tex[1]], color: self.plus_x_color },
			Vertex2D {pos: [self.min.pos[0], self.max.pos[1]], tex: [self.min.tex[0], self.max.tex[1]], color: self.plus_y_color },
			self.max
		])
	}
}

pub struct Quad (pub [Vertex2D; 4]);

impl Quad {
	pub fn slant(&mut self, factor: f32) {
		self.0[0].pos[0] -= factor;
		self.0[1].pos[0] -= factor;
		
		self.0[2].pos[0] += factor;
		self.0[3].pos[0] += factor;
	}
	
	pub fn as_triangles(&self) -> [Vertex2D; 6] {
		[
			self.0[0],
			self.0[1],
			self.0[3],
			self.0[0],
			self.0[2],
			self.0[3]
		]
	}
}