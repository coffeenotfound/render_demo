use std::path::Path;
use std::io;
use std::fs;
use gl_bindings::gl;

pub enum ShaderType {
	Vertex,
	Fragment,
	TessellationControl,
	TessellationEval,
}

impl ShaderType {
	pub fn as_gl_shader_type(&self) -> gl::enuma {
		match self {
			ShaderType::Vertex => gl::VERTEX_SHADER,
			ShaderType::Fragment => gl::FRAGMENT_SHADER,
			ShaderType::TessellationControl => gl::TESS_CONTROL_SHADER,
			ShaderType::TessellationEval => gl::TESS_EVALUATION_SHADER,
		}
	}
}

pub struct ShaderProgram {
	program_gl: gl::uint,
	vertex_shader: Shader,
	fragment_shader: Shader,
	tess_eval_shader: Option<Shader>,
}

impl ShaderProgram {
	pub fn compile(&mut self) {
		// Compile shaders
		self.vertex_shader.compile();
		self.fragment_shader.compile();
		
		if let Some(shader) = &mut self.tess_eval_shader {
			shader.compile();
		}
		
		unsafe {
			// Generate shader program
			self.program_gl = gl::CreateProgram();
			
			// Add shaders to program
			gl::AttachShader(self.program_gl, self.vertex_shader.shader_object_gl);
			gl::AttachShader(self.program_gl, self.fragment_shader.shader_object_gl);
			if let Some(shader) = &mut self.tess_eval_shader {
				gl::AttachShader(self.program_gl, shader.shader_object_gl);
			}
			
			// Link program
			gl::LinkProgram(self.program_gl);
			
			// Detach and delete shader objects (they aren't necessary anymore after linking)
			gl::DetachShader(self.program_gl, self.vertex_shader.shader_object_gl);
			gl::DetachShader(self.program_gl, self.fragment_shader.shader_object_gl);
			self.vertex_shader.delete();
			self.fragment_shader.delete();
			
			// Check link status
			let mut link_status: gl::int = 0;
			gl::GetProgramiv(self.program_gl, gl::LINK_STATUS, &mut link_status);
			
			if link_status != gl::TRUE as gl::int {
				let mut info_log_length: gl::int = 0;
				gl::GetProgramiv(self.program_gl, gl::INFO_LOG_LENGTH, &mut info_log_length);
				
				let mut info_log_buffer = Vec::<u8>::with_capacity(info_log_length as usize);
				info_log_buffer.resize(info_log_length as usize, 0);
				gl::GetProgramInfoLog(self.program_gl, info_log_length, std::ptr::null_mut(), info_log_buffer.as_mut_ptr() as *mut gl::char);
				
				eprintln!("Failed to link shader program:");
				eprintln!("{}", String::from_utf8_unchecked(info_log_buffer));
			}
		}
	}
	
	pub fn delete(&mut self) {
		if self.program_gl != 0 {
			unsafe {
				gl::DeleteProgram(self.program_gl);
			}
			self.program_gl = 0;
		}
	}
	
	pub fn program_gl(&self) -> gl::uint {
		self.program_gl
	}
	
	pub fn new_from_file(vertex_file: &Path, fragment_file: &Path, tess_eval_file: Option<&Path>) -> ShaderProgram {
		let vertex_shader = Shader::load_from_path(ShaderType::Vertex, vertex_file).unwrap();
		let fragment_shader = Shader::load_from_path(ShaderType::Fragment, fragment_file).unwrap();
		
		let tess_eval_shader = if let Some(file) = tess_eval_file {
			Some(Shader::load_from_path(ShaderType::TessellationEval, file).unwrap())
		}
		else {None};
		
		ShaderProgram {
			vertex_shader,
			fragment_shader,
			tess_eval_shader,
			program_gl: 0
		}
	}
	
//	pub fn attach_shader(&mut self, shader_type: ShaderType, shader: Shader) {}
}

pub struct Shader {
	shader_type: ShaderType,
	source_code: Option<String>,
	shader_object_gl: gl::uint,
}

impl Shader {
	pub fn compile(&mut self) {
		unsafe {
			// Create shader object
			self.shader_object_gl = gl::CreateShader(self.shader_type.as_gl_shader_type());
			
			// Add source code
			let source_ptr = self.source_code.as_ref().unwrap().as_ptr() as *const gl::char;
			let length_ptr = self.source_code.as_ref().unwrap().len() as gl::int;
			gl::ShaderSource(self.shader_object_gl, 1, &source_ptr, &length_ptr);
			
			// Compile
			gl::CompileShader(self.shader_object_gl);
			
			// Check compile status
			let mut compile_status: gl::int = 0;
			gl::GetShaderiv(self.shader_object_gl, gl::COMPILE_STATUS, &mut compile_status);
			
			if compile_status != gl::TRUE as gl::int {
				let mut info_log_length: gl::int = 0;
				gl::GetShaderiv(self.shader_object_gl, gl::INFO_LOG_LENGTH, &mut info_log_length);
				
				let mut info_log_buffer = Vec::<u8>::with_capacity(info_log_length as usize);
				info_log_buffer.resize(info_log_length as usize, 0);
				gl::GetShaderInfoLog(self.shader_object_gl, info_log_length, std::ptr::null_mut(), info_log_buffer.as_mut_ptr() as *mut gl::char);
				
				println!("Failed to compile shader:");
				println!("{}", String::from_utf8_unchecked(info_log_buffer));
				
				// DEBUG: panic for now
				panic!();
			}
		}
	}
	
	pub fn delete(&mut self) {
		if self.shader_object_gl != 0 {
			unsafe {
				gl::DeleteShader(self.shader_object_gl);
			}
			self.shader_object_gl = 0;
		}
	}
	
	pub fn load_from_path(shader_type: ShaderType, path: &Path) -> Result<Shader, io::Error> {
		// Read file
		let code = fs::read_to_string(path)?;
		
		Ok(Shader {
			shader_type,
			source_code: Some(code),
			shader_object_gl: 0
		})
	}
}
