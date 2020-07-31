use gl_bindings::gl;
use std::mem;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ShaderStage {
	Vertex,
	Fragment,
	TessellationControl,
	TessellationEval,
	Geometry,
	Compute,
}

impl ShaderStage {
	pub fn as_gl_enum(&self) -> gl::enuma {
		use ShaderStage::*;
		match self {
			Vertex => gl::VERTEX_SHADER,
			Fragment => gl::FRAGMENT_SHADER,
			TessellationControl => gl::TESS_CONTROL_SHADER,
			TessellationEval => gl::TESS_EVALUATION_SHADER,
			Geometry => gl::GEOMETRY_SHADER,
			Compute => gl::COMPUTE_SHADER,
		}
	}
	
	pub fn stages() -> &'static [ShaderStage] {
		use ShaderStage::*;
		&[Vertex, Fragment, TessellationControl, TessellationEval, Geometry, Compute]
	}
}

pub struct Shader {
	shader_stage: ShaderStage,
	
	attached_source: Option<ShaderCode>,
	
	/// The gl name of the gl shader object, or `0` if this
	/// shader is not currently compiled.
	shader_gl: gl::uint,
}

impl Shader {
	pub fn shader_stage(&self) -> ShaderStage {
		self.shader_stage
	}
	
	pub fn shader_gl(&self) -> Option<gl::uint> {
		if self.shader_gl != 0 {
			Some(self.shader_gl)
		} else {
			None
		}
	}
	
	pub fn is_compiled(&self) -> bool {
		self.shader_gl != 0
	}
	
	pub fn attach_source(&mut self, source: ShaderCode) {
		self.attached_source = Some(source);
	}
	
	pub fn drop_source(&mut self) -> Option<ShaderCode> {
		mem::replace(&mut self.attached_source, None)
	}
	
	pub fn attached_source(&self) -> Option<&ShaderCode> {
		self.attached_source.as_ref()
	}
	
	pub fn compile(&mut self, options: &ShaderCompileOptions) -> ShaderCompileResult {
		let info_log: Option<String>;
		let successful: bool;
		
		unsafe {
			// Create shader object
			self.shader_gl = gl::CreateShader(self.shader_stage.as_gl_enum());
			
			// Get code
			let code = if let Some(code) = &self.attached_source {
				code
			} else {
				// We don't have source code, so return a missing code error
				return ShaderCompileResult::new(ShaderCompileStatus::MissingSource, None);
			};
			
			// Add source code
			let code_ptr = code.code.as_ptr() as *const gl::char;
			let code_length = code.code.len() as gl::int;
			gl::ShaderSource(self.shader_gl, 1, &code_ptr, &code_length);
			
			// Compile
			gl::CompileShader(self.shader_gl);
			
			// Check compile status
			let mut compile_status: gl::int = 0;
			gl::GetShaderiv(self.shader_gl, gl::COMPILE_STATUS, &mut compile_status);
			successful = compile_status == gl::TRUE as gl::int;
			
			// Query the info log
			let query_info_log = if successful {options.capture_success_info_log} else {options.capture_failure_info_log};
			
			info_log = if query_info_log {
				// Query info log length
				let mut info_log_buffer_size: gl::int = 0;
				gl::GetShaderiv(self.shader_gl, gl::INFO_LOG_LENGTH, &mut info_log_buffer_size);
				
				// Allocate info log buffer
				let mut info_log_buffer = vec![0u8; info_log_buffer_size as usize];
				
				// Actually get the info log
				let mut info_log_string_length: gl::sizei = 0;
				gl::GetShaderInfoLog(self.shader_gl, info_log_buffer_size as gl::sizei, &mut info_log_string_length, info_log_buffer.as_mut_ptr() as *mut gl::char);
				
				// Truncate the buffer to the actual length in chars (should be the same minus the null terminator, so minus 1)
				info_log_buffer.truncate(info_log_string_length as usize);
				
				// Make into string
				let info_log_string = String::from_utf8_unchecked(info_log_buffer);
				Some(info_log_string)
			} else {
				None
			};
		}
		
		// Make result object
		let compile_result = ShaderCompileResult::new(ShaderCompileStatus::Success, info_log);
		compile_result
	}
	
	pub fn dispose(&mut self) {
		if self.shader_gl != 0 {
			unsafe {
				let name = mem::replace(&mut self.shader_gl, 0);
				gl::DeleteShader(name);
			}
		}
	}
	
	pub fn new(stage: ShaderStage) -> Self {
		Self {
			shader_stage: stage,
			
			attached_source: None,
			
			shader_gl: 0,
		}
	}
	
	/*
	pub fn load_from_path(shader_type: ShaderType, path: &Path) -> Result<Shader, io::Error> {
		// Read file
		let code = fs::read_to_string(path)?;
		
		Ok(Shader {
			shader_type,
			source_code: Some(code),
			shader_object_gl: 0
		})
	}
	*/
}

pub struct ShaderCode {
	pub code: Vec<u8>,
}

impl ShaderCode {
	pub fn new(code: Vec<u8>) -> Self {
		Self {
			code,
		}
	}
}

pub struct ShaderCompileOptions {
	pub capture_success_info_log: bool,
	pub capture_failure_info_log: bool,
}

impl ShaderCompileOptions { 
	pub fn with_info_log(&mut self, capture_on_success: bool, capture_on_failure: bool) -> &mut Self {
		self.capture_success_info_log = capture_on_success;
		self.capture_failure_info_log = capture_on_failure;
		self
	}
}

impl Default for ShaderCompileOptions {
	fn default() -> ShaderCompileOptions {
		ShaderCompileOptions {
			capture_success_info_log: false,
			capture_failure_info_log: true,
		}
	}
}

pub struct ShaderCompileResult {
	pub status: ShaderCompileStatus,
	pub info_log: Option<String>,
}

impl ShaderCompileResult { 
	pub fn new(status: ShaderCompileStatus, info_log: Option<String>) -> Self {
		Self {
			status,
			info_log,
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum ShaderCompileStatus {
	Success,
	MissingSource,
	CompileError,
}
