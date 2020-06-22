use gl_bindings::gl;
use std::ffi::CString;

pub struct UniformLocationCache {
	uniform_name: CString,
	
	cached_program_gl: gl::uint,
	cached_location: gl::int,
}

impl UniformLocationCache {
	pub fn get(&mut self, program_gl: gl::uint) -> Option<gl::int> {
		if self.cached_program_gl != program_gl || self.cached_location == 0 {
			self.cached_program_gl = program_gl;
			self.cached_location = unsafe {
				let name_ptr = self.uniform_name.as_ptr() as *const gl::char;
				gl::GetUniformLocation(program_gl, name_ptr)
			};
		}
		
		if self.cached_location != 0 {
			Some(self.cached_location)
		} else {
			None
		}
	}
	
	pub fn new(uniform_name: &str) -> Self {
		Self {
			uniform_name: CString::new(uniform_name).unwrap(),
			
			cached_program_gl: 0,
			cached_location: 0,
		}
	}
}

//pub struct Uniform<T: UniformValue> {
//	
//}
