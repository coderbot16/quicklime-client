use image::GrayAlphaImage;
use text::metrics::GlyphSize;

const SPACE_WIDTH: u32 = 3;

/// Characters included in `ascii.png` (previously known as `default.png`), a modified CP437.
pub static DEFAULT_CHARS: &str = "\
	ÀÁÂÈÊËÍÓÔÕÚßãõğİ\
	ıŒœŞşŴŵžȇ\0\0\0\0\0\0\0\
	\u{0020}!\"#$%&'()*+,-./\
	0123456789:;<=>?\
	@ABCDEFGHIJKLMNO\
	PQRSTUVWXYZ[\\]^_\
	`abcdefghijklmno\
	pqrstuvwxyz{|}~\0\
	ÇüéâäàåçêëèïîìÄÅ\
	ÉæÆôöòûùÿÖÜø£Ø×ƒ\
	áíóúñÑªº¿®¬½¼¡«»\
	░▒▓│┤╡╢╖╕╣║╗╝╜╛┐\
	└┴┬├─┼╞╟╚╔╩╦╠═╬╧\
	╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀\
	αβΓπΣσμτΦΘΩδ∞∅∈∩\
	≡±≥≤⌠⌡÷≈°∙·√ⁿ²■\0";
	

pub fn character_to_default(character: char) -> Option<u8> {
	return DEFAULT_CHARS.find(character).map(|x| x as u8)
}

#[derive(Debug)]
pub enum CalculateMetricsError {
	/// Image dimensions are not a power of 2, or zero.
	NotPowerOf2(u32, u32)
}

pub struct DefaultMetrics {
	widths: [u32; 256],
	dimensions: (u32, u32)
}

impl DefaultMetrics {
	
	// TODO: Make this use any alpha image.
	pub fn calculate(image: GrayAlphaImage) -> Result<Self, CalculateMetricsError> {
		let dimensions = image.dimensions();
		
		if dimensions.0.count_ones() != 1 || dimensions.1.count_ones() != 1 {
			return Err(CalculateMetricsError::NotPowerOf2(image.width(), image.height()))
		}
		
		// Pixels per character.
		let per_char = (dimensions.0 / 16, dimensions.1 / 16);
		
		let mut widths = [1; 256];
		
		'outer:
		for (default, width) in (0..256).zip(widths.iter_mut()) {
			let (atlas_x, atlas_y) = (default % 16, default / 16);
			
			for sub_x in (0..per_char.0).map(|x| per_char.0-x-1) {
				for sub_y in 0..per_char.1 {
					
					// Check if alpha channel is nonzero
					if image.get_pixel(atlas_x * per_char.0 + sub_x, atlas_y * per_char.1 + sub_y).data[1] != 0 {
						*width = sub_x + 1;
						
						// Next char
						continue 'outer;
					}
				}
			}
		}
		
		widths[0x20] = SPACE_WIDTH;
		
		Ok(DefaultMetrics { dimensions: dimensions, widths: widths })
	}
	
	pub fn size(&self, index: u8) -> GlyphSize {
		// TODO: GlyphSize doesn't support anything other than a 128x128 atlas yet.
		GlyphSize::from_default_width(self.widths[index as usize] as u8)
	}
}