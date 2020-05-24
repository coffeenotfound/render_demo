use gl_bindings::gl;
use crate::render::ImageFormat;
use std::path::Path;
use std::error;
use std::fs::OpenOptions;
use ktx::KtxInfo;

pub struct Texture {
	width: u32,
	height: u32,
	levels: u32,
	image_format: ImageFormat,
	
	handle_gl: gl::uint,
}

impl Texture {
	pub fn resize(&mut self, width: u32, height: u32) -> bool {
		if self.handle_gl == 0 {
			self.width = width;
			self.height = height;
			true
		}
		else {false}
	}
	
	pub fn allocate(&mut self) -> bool {
		if self.handle_gl == 0 {
			// Allocate texture
			self.handle_gl = unsafe {
				let mut tex: gl::uint = 0;
				gl::CreateTextures(gl::TEXTURE_2D, 1, &mut tex);
				gl::TextureParameteri(tex, gl::TEXTURE_MIN_FILTER, gl::LINEAR as gl::int);
				gl::TextureParameteri(tex, gl::TEXTURE_MAG_FILTER, gl::LINEAR as gl::int);
				gl::TextureParameteri(tex, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as gl::int);
				gl::TextureParameteri(tex, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as gl::int);
				
				gl::TextureStorage2D(tex, self.levels as gl::sizei, self.image_format.as_gl_enum(), self.width as gl::sizei, self.height as gl::sizei);
				tex
			};
			true
		}
		else {
			false
		}
	}
	
	pub fn is_allocated(&self) -> bool {
		self.handle_gl != 0
	}
	
	pub fn texture_gl(&self) -> gl::uint {
		self.handle_gl
	}
	
	pub fn new(width: u32, height: u32, levels: u32, image_format: ImageFormat) -> Texture {
		Texture {
			width,
			height,
			levels,
			image_format,
			
			handle_gl: 0,
		}
	}
	
	#[deprecated]
	pub fn load_ktx_from_path(path: &Path, image_format: ImageFormat) -> Result<Texture, Box<dyn error::Error>> {
		// Open file
		let ktx_decoder = ktx::Decoder::new(OpenOptions::new().read(true).open(path)?)?;
		
		// Get header
//		let header = ktx_decoder.header();
		let (img_width, img_height) = (ktx_decoder.pixel_width(), ktx_decoder.pixel_height());
		
		// Read texture data
		let base_level_data = ktx_decoder.read_textures().next().unwrap();
		
		// Allocate texture
		let mut texture = Texture::new(img_width as u32, img_height as u32, 8, image_format);
		texture.allocate();
		
		// Upload compressed image data
		unsafe {
			gl::CompressedTextureSubImage2D(texture.handle_gl, 0, 0, 0, img_width as gl::sizei, img_height as gl::sizei, image_format.as_gl_enum(), base_level_data.len() as gl::sizei, base_level_data.as_ptr() as *const gl::void);
		}
		
//		// Generate mipmaps
//		unsafe {
//			gl::TextureParameteri(texture.texture_gl(), gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as gl::int);
//			gl::TextureParameteri(texture.texture_gl(), gl::TEXTURE_MAG_FILTER, gl::LINEAR as gl::int);
//			
//			gl::GenerateTextureMipmap(texture.texture_gl());
//		}
		
		Ok(texture)
	}
	
	#[deprecated]
	pub fn load_png_from_path(path: &Path, image_format: ImageFormat) -> Result<Texture, Box<dyn error::Error>> {
		// Load texture
		let result = lodepng::decode32_file(path)?;
		
		let (img_width, img_height) = (result.width, result.height);
		
		// Allocate texture
		//let mut texture = Texture::new(img_width as u32, img_height as u32, 8, ImageFormat::get(gl::SRGB8_ALPHA8));
		let mut texture = Texture::new(img_width as u32, img_height as u32, 8, image_format);
		texture.allocate();
		
		// Upload image data
		unsafe {
			let data_buffer_ptr = result.buffer.as_ptr();
			gl::TextureSubImage2D(texture.handle_gl, 0, 0, 0, img_width as gl::sizei, img_height as gl::sizei, gl::RGBA, gl::UNSIGNED_BYTE, data_buffer_ptr as *const gl::void);
		}
		
		// Generate mipmaps
		unsafe {
			gl::TextureParameteri(texture.texture_gl(), gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as gl::int);
			gl::TextureParameteri(texture.texture_gl(), gl::TEXTURE_MAG_FILTER, gl::LINEAR as gl::int);
			
			gl::GenerateTextureMipmap(texture.texture_gl());
		}
		
		Ok(texture)
	}
	
	/*
	pub fn load_png_from_path(path: &Path) -> Result<Texture, Box<dyn error::Error>> {
		let file = OpenOptions::new().read(true).open(path)?;
		
		// Read header
		let png_decoder = png::Decoder::new(file);
		let (img_info, mut png_reader) = png_decoder.read_info()?;
		
		let info = png_reader.info();
		if info.bit_depth != png::BitDepth::Eight || info.color_type != png::ColorType::RGB {
			panic!("Unsupported image format");
		}
		
		// Read image data
		let buffer_size = info.width * info.height * 3;
		let mut data_buffer = vec![0u8; buffer_size as usize];
		png_reader.next_frame(data_buffer.as_mut_slice())?;
		
		// Allocate texture
		let mut texture = Texture::new(img_info.width, img_info.height, 1, ImageFormat::get(gl::RGB8));
		texture.allocate();
		
		// Upload image data
		unsafe {
			gl::TextureSubImage2D(texture.handle_gl, 0, 0, 0, img_info.width as gl::sizei, img_info.height as gl::sizei, gl::RGB, gl::UNSIGNED_BYTE, data_buffer.as_ptr() as *const gl::void);
		}
		
		Ok(texture)
	}
	*/
}
