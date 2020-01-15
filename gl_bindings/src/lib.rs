#[allow(non_camel_case_types)]
pub mod gl {
	include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
	
//	pub use crate::gl::types::*;
	
	pub type boolean = self::types::GLboolean;
	pub type char = self::types::GLchar;
	pub type int = self::types::GLint;
	pub type uint = self::types::GLuint;
	pub type float = self::types::GLfloat;
	pub type sizei = self::types::GLsizei;
	pub type enuma = self::types::GLenum;
	pub type intptr = self::types::GLintptr;
	pub type sizeiptr = self::types::GLsizeiptr;
	pub type bitfield = self::types::GLbitfield;
	pub type void = std::ffi::c_void;
}
