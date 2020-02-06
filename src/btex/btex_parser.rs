use std::io::{self, Read, Seek, Error, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::btex::{HashMapFormatRegistry, BTexHeaderInfo, BTexTextureInfo, BTexOffsetTable, BTexFormatRegistry, TextureDimensionality, PixelDataOffset};

pub struct BTexParser<'r, 'g, R, G = HashMapFormatRegistry<'g>> where R: Read + Seek, G: BTexFormatRegistry<'g> {
	reader: &'r mut R,
	format_registry: &'g G,
}

impl<'r, 'g, R, G> BTexParser<'r, 'g, R, G> where R: Read + Seek, G: BTexFormatRegistry<'g> {
	pub fn parse(&mut self) -> Result<(BTexHeaderInfo, BTexTextureInfo<'g>, BTexOffsetTable), ParseError> {
		fn conv_io_error<T>(result: Result<T, io::Error>) -> Result<T, ParseError> {
			result.map_err(|e| ParseError::IoError(e))
		}
		
		// Check the magic number
		let mut magic_number = [0 as u8; 4];
		conv_io_error(self.reader.read_exact(&mut magic_number))?;
		
		if magic_number != [b'b', b't', b'e', b'x'] {
			return Err(ParseError::InvalidMagicNumber);
		}
		
		// Read header info
		let version = conv_io_error(self.reader.read_u32::<LittleEndian>())?;
		let header_length = conv_io_error(self.reader.read_u32::<LittleEndian>())?;
		let offset_table_length = conv_io_error(self.reader.read_u32::<LittleEndian>())?;
		
		// Make header info object
		let header_info = BTexHeaderInfo {
			version,
			header_length,
			offset_table_length,
		};
		
		// Read texture info
		let texture_attribs = conv_io_error(self.reader.read_u32::<LittleEndian>())?;
		let width = conv_io_error(self.reader.read_u32::<LittleEndian>())?;
		let height = conv_io_error(self.reader.read_u32::<LittleEndian>())?;
		let depth = conv_io_error(self.reader.read_u32::<LittleEndian>())?;
		let levels = conv_io_error(self.reader.read_u32::<LittleEndian>())?;
		let layers = conv_io_error(self.reader.read_u32::<LittleEndian>())?;
		
		fn calc_num_levels(size: u32) -> u32 {
			(32 - u32::leading_zeros(size))
		}
		
		// Determine the dimensionality
		let dimensionality = (if width == 0 && height == 0 && depth == 0 {
			if layers > 0 || levels > 0 {
				Err(ParseError::InvalidTextureLayout)
			}
			else {
				Ok(TextureDimensionality::Zero)
			}
		}
		else if width > 0 && height == 0 && depth == 0 {
			if levels == 0 || levels > calc_num_levels(width) {
				Err(ParseError::InvalidTextureLayout)
			}
			else {
				Ok(TextureDimensionality::One)
			}
		}
		else if width > 0 && height > 0 && depth == 0 {
			if levels == 0 || levels > u32::max(calc_num_levels(width), calc_num_levels(height)) {
				Err(ParseError::InvalidTextureLayout)
			}
			else {
				Ok(TextureDimensionality::Two)
			}
		}
		else if width > 0 && height > 0 && depth > 0 {
			if levels == 0 || levels > u32::max(calc_num_levels(width), u32::max(calc_num_levels(width), calc_num_levels(depth))) {
				Err(ParseError::InvalidTextureLayout)
			}
			else {
				Ok(TextureDimensionality::Three)
			}
		}
		else {
			Err(ParseError::InvalidTextureLayout)
		})?;
		
		// TODO: Parse format str as ascii not utf8
		// Read the image format
		let mut raw_format = [0 as u8; 16];
		conv_io_error(self.reader.read_exact(&mut raw_format))?;
		
		let format_str = std::str::from_utf8(&raw_format).map_err(|_| ParseError::UnknownImageFormat(String::from("????")))?;
		
		// Lookup image format
		let image_format = (if let Some(format) = self.format_registry.lookup_format(format_str) {
			Ok(format)
		} else {
			Err(ParseError::UnknownImageFormat(String::from(format_str)))
		})?;
		
		// Check image format compatibility
		if !image_format.is_dimensionality_compatible(dimensionality) {
			return Err(ParseError::IncompatibleImageFormat(String::from(format_str), dimensionality))
		}
		
		// Make texture info object
		let texture_info = BTexTextureInfo {
			dimensionality,
			width,
			height,
			depth,
			levels,
			layers,
			image_format,
			attributes: texture_attribs,
		};
		
		// Parse the offset table
		let num_images: u32 = (match u32::checked_mul(levels, layers) {
			Some(num) => Ok(num),
			None => Err(ParseError::TooManyImages),
		})?;
		
		let mut data_offsets = Vec::<PixelDataOffset>::with_capacity(num_images as usize);
		
		// Read the offset entries
		for _ in 0..num_images {
			let offset = conv_io_error(self.reader.read_u64::<LittleEndian>())?;
			let length = conv_io_error(self.reader.read_u64::<LittleEndian>())?;
			
			data_offsets.push(PixelDataOffset {offset, length});
		}
		
		// Make offset table object
		let offset_table = BTexOffsetTable {
			offsets: data_offsets,
		};
		
		// Return parsed data
		Ok((header_info, texture_info, offset_table))
	}
	
	pub fn pixel_data_reader<'a>(&'a mut self, data_offset: &'a PixelDataOffset) -> Result<PixelDataReader<'a, R>, io::Error> {
		let mut reader = PixelDataReader::from_parser(self.reader, data_offset);
		reader.seek_data()?;
		Ok(reader)
	}
	
	pub fn read_pixel_data<'a>(&'a mut self, data_offset: &'a PixelDataOffset, buffer: &mut [u8]) -> Result<u64, io::Error> {
		let mut pixel_reader = self.pixel_data_reader(data_offset)?;
		let read_length = usize::min(buffer.len(), data_offset.length as usize);
		
		pixel_reader.read_exact(&mut buffer[..read_length]).map(|_| data_offset.length)
	}
	
	pub fn new(reader: &'r mut R, format_registry: &'g G) -> Self {
		Self {
			reader,
			format_registry,
		}
	}
}

pub struct PixelDataReader<'a, R> where R: Read + Seek {
	reader: &'a mut R,
	data_offset: &'a PixelDataOffset,
	bytes_left: u64,
}

impl<'a, R> PixelDataReader<'a, R> where R: Read + Seek {
	fn seek_data(&mut self) -> Result<(), io::Error> {
		// Seek in reader
		self.reader.seek(SeekFrom::Start(self.data_offset.offset)).map(|_| ())
	}
	
	fn from_parser(reader: &'a mut R, data_offset: &'a PixelDataOffset) -> Self {
		Self {
			reader,
			data_offset,
			bytes_left: data_offset.length,
		}
	}
}

impl<'a, R> Read for PixelDataReader<'a, R> where R: Read + Seek {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
		// Read only as much as fits into the buffer and as much as we have left
		let num_bytes = usize::min(self.bytes_left as usize, buf.len());
		
		let result = self.reader.read(&mut buf[..num_bytes]);
		if let Ok(num) = &result {
			self.bytes_left -= *num as u64;
		}
		result
	}
}

pub enum ParseError {
	IoError(io::Error),
	InvalidMagicNumber,
	InvalidTextureLayout,
	UnknownImageFormat(String),
	IncompatibleImageFormat(String, TextureDimensionality),
	IllegallyOmittedImage,
	TooManyImages,
}
