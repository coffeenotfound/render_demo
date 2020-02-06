
#[derive(Copy, Clone)]
pub enum TextureDimensionality {
	Zero,
	One,
	Two,
	Three,
}

pub struct BTexImageFormat<'a> {
	pub name: &'a str,
	pub compatible_1d: bool,
	pub compatible_2d: bool,
	pub compatible_3d: bool,
}

impl<'a> BTexImageFormat<'a> {
	pub fn is_dimensionality_compatible(&self, dimensionality: TextureDimensionality) -> bool {
		use TextureDimensionality::*;
		match dimensionality {
			Zero => true,
			One => self.compatible_1d,
			Two => self.compatible_2d,
			Three => self.compatible_3d,
		}
	}
}

pub struct BTexHeaderInfo {
	pub version: u32,
	pub header_length: u32,
	pub offset_table_length: u32,
}

pub struct BTexTextureInfo<'g> {
	pub dimensionality: TextureDimensionality,
	pub width: u32,
	pub height: u32,
	pub depth: u32,
	pub levels: u32,
	pub layers: u32,
	pub attributes: u32,
	pub image_format: &'g BTexImageFormat<'g>,
}

impl<'g> BTexTextureInfo<'g> {
	pub fn is_sparse(&self) -> bool {
		(self.attributes & 0x1 != 0)
	}
}

pub struct BTexOffsetTable {
	pub offsets: Vec<PixelDataOffset>,
}

pub struct PixelDataOffset {
	pub offset: u64,
	pub length: u64,
}
