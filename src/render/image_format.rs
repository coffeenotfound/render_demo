use gl_bindings::gl;

#[derive(Copy, Clone)]
pub struct ImageFormat {
	format_gl: gl::enuma,
}

impl ImageFormat {
	pub fn get(format_gl: gl::enuma) -> ImageFormat {
		ImageFormat {
			format_gl
		}
	}
	
	pub fn as_gl_enum(&self) -> gl::enuma {
		self.format_gl
	}
}

impl PartialEq for ImageFormat {
	fn eq(&self, other: &Self) -> bool {
		(self.format_gl == other.format_gl)
	}
}
impl Eq for ImageFormat {}
