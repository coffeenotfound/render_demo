use std::mem;
use std::io::{self, Write, Seek, Read};
use std::fmt;
use std::error::Error;
use byteorder::{LittleEndian, WriteBytesExt};
use crate::btex::{BTexTextureInfo};

pub struct BTexWriter<'w, W> where W: Write + Seek {
	writer: &'w mut W,
}

impl<'w, W> BTexWriter<'w, W> where W: Write + Seek {
//	pub fn write<'b, P, F>(&'b mut self, texture_info: &'b BTexTextureInfo, data_provider: F) -> Result<(), WriteError>
//			where P: 'b + PixelDataSource, F: Fn(ImageSourceIndex) -> Option<&'b mut P> {
	pub fn write<'b, R>(&'b mut self, texture_info: &'_ BTexTextureInfo, pixel_data_sources: &'_ mut [Option<&'_ mut PixelDataSource<'_, R>>]) -> Result<(), WriteError> where R: Read {
		fn conv_io_error<T>(result: Result<T, io::Error>) -> Result<T, WriteError> {
			result.map_err(|e| WriteError::IoError(e))
		}
		
		// Write magic number
		conv_io_error(self.writer.write_all(&[b'b', b't', b'e', b'x']))?;
		
		// Calc offset table length
		let num_images = texture_info.levels * texture_info.layers;
		let offset_table_length = num_images * 2 * mem::size_of::<u64>() as u32;
		
		// Write header info
		let header_version = 1;
		let header_length = 64;
		conv_io_error(self.writer.write_u32::<LittleEndian>(header_version))?; // version
		conv_io_error(self.writer.write_u32::<LittleEndian>(header_length))?; // header_length
		conv_io_error(self.writer.write_u32::<LittleEndian>(offset_table_length))?; // offset_table_length
		
		// Write texture info
		conv_io_error(self.writer.write_u32::<LittleEndian>(texture_info.attributes))?; // attributes
		conv_io_error(self.writer.write_u32::<LittleEndian>(texture_info.width))?; // width
		conv_io_error(self.writer.write_u32::<LittleEndian>(texture_info.height))?; // height
		conv_io_error(self.writer.write_u32::<LittleEndian>(texture_info.depth))?; // depth
		conv_io_error(self.writer.write_u32::<LittleEndian>(texture_info.levels))?; // levels
		conv_io_error(self.writer.write_u32::<LittleEndian>(texture_info.layers))?; // layers
		
		// Serialize image format
		let mut image_format_buffer = [0 as u8; 16];
		image_format_buffer.copy_from_slice(&texture_info.image_format.name.as_bytes()[0..8]);
		
		// Write image format str
		conv_io_error(self.writer.write_all(&image_format_buffer))?; // image_format
		
		// Write the offset table
		let is_sparse_texture = texture_info.is_sparse();
		let mut running_offset = (header_length + offset_table_length) as u64;
		
		for source in pixel_data_sources.iter_mut() {
			let data_length = if let Some(source) = source {
				source.data_length
			} else {
				if !is_sparse_texture {
					// Omitted image eventhough the texture is non-sparse
					return Err(WriteError::IllegallyOmittedImage);
				} else {
					// If image is legally omitted, it has size zero
					0
				}
			};
			
			// Serialize offset entry
			conv_io_error(self.writer.write_u64::<LittleEndian>(running_offset))?; // offset
			conv_io_error(self.writer.write_u64::<LittleEndian>(data_length))?; // length
			
			// Accumulate running offset
			running_offset += data_length;
		}
		
		// Write the image data
		for source in pixel_data_sources.iter_mut() {
			if let Some(source) = source {
				// Copy the data from the source to the writer
				let mut capped_reader = source.source.take(source.data_length);
				let written = io::copy(&mut capped_reader, &mut self.writer).map_err(|e| WriteError::PixelSourceIoError(e))?;
				
				// Check if enough was written
				if written < source.data_length {
					return Err(WriteError::NotEnoughPixelData);
				}
			} else {
				if !is_sparse_texture {
					// Omitted image eventhough the texture is non-sparse
					return Err(WriteError::IllegallyOmittedImage);
				} else {
					// Image is legally omitted, so write nothing
				}
			};
		}
		
		// Everything written successfully, return Ok
		Ok(())
	}
}

#[derive(Debug)]
pub enum WriteError {
	IoError(io::Error),
	PixelSourceIoError(io::Error),
	IllegallyOmittedImage,
	NotEnoughPixelData,
}

impl Error for WriteError {}

impl fmt::Display for WriteError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		<Self as fmt::Debug>::fmt(self, f)
	}
}

//#[derive(Copy, Clone)]
//pub struct ImageSourceIndex {
//	pub level: u32,
//	pub layer: u32,
//}

pub struct PixelDataSource<'a, R> where R: Read {
	pub data_length: u64,
	pub source: &'a mut R,
}
