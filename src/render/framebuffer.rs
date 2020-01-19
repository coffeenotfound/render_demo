use gl_bindings::gl;
use crate::render::{ImageFormat, Texture};
use crate::bool_cmpxchg::BoolCompareExchange;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Framebuffer {
	width: u32,
	height: u32,
	is_allocated: bool,
	
	handle_gl: gl::uint,
	color_attachments: [Option<FramebufferAttachment>; 16],
	depth_attachment: Option<FramebufferAttachment>,
}

impl Framebuffer {
	pub fn size(&self) -> (u32, u32) {
		(self.width, self.height)
	}
	
	pub fn is_allocated(&self) -> bool {
		self.is_allocated
	}
	
	pub fn handle_gl(&self) -> gl::uint {
		self.handle_gl
	}
	
	// generic_error_here!("Failed to add attachment")
	
	pub fn add_attachment(&mut self, attachment: FramebufferAttachment) -> bool {
		use AttachmentPoint as A;
		
		let slot = match attachment.attachment_point {
			A::Depth => {
				&mut self.depth_attachment
			}
			A::Color(index) if index < 16 => {
				&mut self.color_attachments[index as usize]
			}
			_ => {
				return false;
			}
		};
		
		if let None = slot {
			// Create attachment
			*slot = Some(attachment);
			true
		}
		else {
			false
		}
	}
	
	pub fn get_attachment(&self, attachment_point: AttachmentPoint) -> Option<&FramebufferAttachment> {
		use AttachmentPoint as A;
		match attachment_point {
			A::Depth => self.depth_attachment.as_ref(),
			A::Color(index) if index < 16 => self.color_attachments[index as usize].as_ref(),
			_ => None
		}
	}
	
	pub fn allocate(&mut self) -> bool {
		if self.is_allocated.compare_exchange(false, true) {
			// Gen gl framebuffer object
			self.handle_gl = unsafe {
				let mut buffer: gl::uint = 0;
				gl::CreateFramebuffers(1, &mut buffer);
				buffer
			};
			
			let mut draw_buffer_table: [gl::enuma; 16] = [gl::NONE; 16];
			
			// Create depth attachment
			if let Some(depth_attachment) = &mut self.depth_attachment {
				// Allocate texture
				depth_attachment.allocate(self.width, self.height);
				
				// Attach texture to framebuffer
				unsafe {
					gl::NamedFramebufferTexture(self.handle_gl, AttachmentPoint::Depth.as_gl_enum().unwrap(), depth_attachment.texture.borrow().texture_gl(), depth_attachment.level as gl::int);
				}
			}
			
			// Create color attachment
			for attachment in self.color_attachments.iter_mut() {
				if let Some(attachment) = attachment.as_mut() {
					// Allocate color texture
					attachment.allocate(self.width, self.height);
					
					// Attach texture to framebuffer
					unsafe {
						gl::NamedFramebufferTexture(self.handle_gl, attachment.attachment_point.as_gl_enum().unwrap(), attachment.texture.borrow().texture_gl(), attachment.level as gl::int);
					}
					
					// Update draw buffer table
					let index = match attachment.attachment_point {
						AttachmentPoint::Color(index) => index,
						_ => panic!(),
					};
					draw_buffer_table[index as usize] = attachment.attachment_point.as_gl_enum().unwrap();
				}
			}
			
			// Set drawbuffers
			unsafe {
//				gl::NamedFramebufferDrawBuffers(self.handle_gl, draw_buffer_table.len() as gl::GLsizei, draw_buffer_table.as_ptr());
				// Somehow the above doesn't work and generate an GL_INVALID_VALUE: Draw buffer is invalid error
				let debug_draw_buffers = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];
				gl::NamedFramebufferDrawBuffers(self.handle_gl, 2, debug_draw_buffers.as_ptr());
			}
			
			// Check framebuffer status
			let status = unsafe {
				gl::CheckNamedFramebufferStatus(self.handle_gl, gl::FRAMEBUFFER)
			};
			if status != gl::FRAMEBUFFER_COMPLETE {
				eprintln!("Framebuffer is incomplete: {}", status);
			}
			return true;
		}
		else {
			return false;
		}
	}
	
	pub fn resize(&mut self, new_width: u32, new_height: u32) -> bool {
		fn resize_attachment(attachment: &mut FramebufferAttachment, new_width: u32, new_height: u32) {
			attachment.resize(new_width, new_height);
		}
		
		// Resize depth attachment
		if let Some(depth) = &mut self.depth_attachment {
			resize_attachment(depth, new_width, new_height);
		}
		
		// Resize color attachments
		for attachment in &mut self.color_attachments.iter_mut() {
			if let Some(color) = attachment.as_mut() {
				resize_attachment(color, new_width, new_height);
			}
		}
		
		true
	}
	
	pub fn new(width: u32, height: u32) -> Framebuffer {
		Framebuffer {
			width,
			height,
			handle_gl: 0,
			color_attachments: Default::default(),
			depth_attachment: None,
			is_allocated: false,
		}
	}
}

pub struct FramebufferAttachment {
	pub attachment_point: AttachmentPoint,
	pub texture: Rc<RefCell<Texture>>,
	pub level: u32,
}

impl FramebufferAttachment {
	pub fn allocate(&mut self, width: u32, height: u32) { 
		let mut tex = self.texture.borrow_mut();
		tex.resize(width, height);
		tex.allocate();
	}
	
	pub fn resize(&mut self, width: u32, height: u32) {
		// Delete old texture
		let tex = RefCell::borrow_mut(&mut self.texture);
		
		if tex.texture_gl() != 0 {
			let tex_gl: gl::uint = tex.texture_gl(); 
			unsafe {
				gl::DeleteTextures(1, &tex_gl);
			}
		}
		drop(tex);
		
		// Reallocate
		self.allocate(width, height);
	}
	
	pub fn from_texture(attachment_point: AttachmentPoint, texture: Rc<RefCell<Texture>>, level: u32) -> FramebufferAttachment {
		FramebufferAttachment {
			attachment_point,
			texture,
			level
		}
	}
	
	pub fn from_new_texture(attachment_point: AttachmentPoint, image_format: ImageFormat) -> FramebufferAttachment {
		let texture = Texture::new(0, 0, 1, image_format);
		Self::from_texture(attachment_point, Rc::new(RefCell::new(texture)), 0)
	}
}

#[derive(Copy, Clone)]
pub enum AttachmentPoint {
	Depth,
	Color(u32),
}

impl AttachmentPoint {
	pub fn as_gl_enum(&self) -> Option<gl::enuma> {
		match self {
			AttachmentPoint::Depth => Some(gl::DEPTH_ATTACHMENT),
			AttachmentPoint::Color(index) if *index < 32 => Some(gl::COLOR_ATTACHMENT0 + *index as gl::enuma),
			_ => None
		}
	}
}
