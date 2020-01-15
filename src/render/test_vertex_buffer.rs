use gl_bindings::gl;

pub struct TestVertexBuffer {
	pub vbo_gl: gl::uint,
}

impl TestVertexBuffer {
	pub fn allocate(&mut self, data: &[u8]) {
		if self.vbo_gl == 0 {
			// Create gl vbo object
			self.vbo_gl = unsafe {
				let mut buffer: gl::uint = 0;
				gl::CreateBuffers(1, &mut buffer);
				buffer
			};
			
			// Upload vertex data
			unsafe {
				gl::NamedBufferData(self.vbo_gl, data.len() as isize, data.as_ptr() as *const gl::void, gl::STATIC_DRAW);
			}
		}
	}
	
	pub fn new() -> TestVertexBuffer {
		TestVertexBuffer {
			vbo_gl: 0
		}
	}
}
