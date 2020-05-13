use gl_bindings::gl;
use crate::render::shader::{Shader, ShaderStage};
use std::mem;

pub struct ShaderProgram {
	vertex_shader: Option<Shader>,
	fragment_shader: Option<Shader>,
	tess_control_shader: Option<Shader>,
	tess_eval_shader: Option<Shader>,
	geometry_shader: Option<Shader>,
	compute_shader: Option<Shader>,
	
	program_gl: gl::uint,
}

impl ShaderProgram {
	pub fn is_linked(&self) -> bool {
		self.program_gl != 0
	}
	
	pub fn program_gl(&self) -> Option<gl::uint> {
		if self.program_gl != 0 {
			Some(self.program_gl)
		} else {
			None
		}
	}
	
	pub fn attached_shader(&self, stage: &ShaderStage) -> Option<&Shader> {
		self.shader_slot(stage).map_or(None, |s| s.as_ref())
	}
	
	pub fn attached_shader_mut(&mut self, stage: &ShaderStage) -> Option<&mut Shader> {
		self.shader_slot_mut(stage).map_or(None, |s| s.as_mut())
	}
	
	pub fn has_stage(&self, stage: &ShaderStage) -> bool {
		self.shader_slot(stage).is_none()
	}
	
	fn shader_slot(&self, stage: &ShaderStage) -> Option<&Option<Shader>> {
		use ShaderStage::*;
		Some(match stage {
			Vertex => &self.vertex_shader,
			Fragment => &self.fragment_shader,
			TessellationControl => &self.tess_control_shader,
			TessellationEval => &self.tess_eval_shader,
			Geometry => &self.geometry_shader,
			Compute => &self.compute_shader,
		})
	}
	
	fn shader_slot_mut(&mut self, stage: &ShaderStage) -> Option<&mut Option<Shader>> {
		use ShaderStage::*;
		Some(match stage {
			Vertex => &mut self.vertex_shader,
			Fragment => &mut self.fragment_shader,
			TessellationControl => &mut self.tess_control_shader,
			TessellationEval => &mut self.tess_eval_shader,
			Geometry => &mut self.geometry_shader,
			Compute => &mut self.compute_shader,
		})
	}
	
	pub fn attach_shader(&mut self, shader: Shader) -> bool {
		// Get the slot for the given stage
		let slot = if let Some(slot) = self.shader_slot_mut(&shader.shader_stage()) {
			slot
		} else {
			// Unavailable slot (this shouldn't ever happen)
			return false;
		};
		
		// Check if there is already a shader attached
		if let None = slot {
			// Put the shader in the slot
			*slot = Some(shader);
			true
		} else {
			false
		}
	}
	
	pub fn detach_shader(&mut self, _stage: &ShaderStage) -> bool {
		unimplemented!();
	}
	
	pub fn link(&mut self, options: &ProgramLinkOptions) -> ProgramLinkResult {
		let info_log: Option<String>;
		let successful: bool;
		
		unsafe {
			// Create program object
			self.program_gl = gl::CreateProgram();
			
			// Attach shaders
			for stage in ShaderStage::stages() {
				if let Some(shader) = self.attached_shader(stage) {
					if let Some(shader_gl) = shader.shader_gl() {
						gl::AttachShader(self.program_gl, shader_gl);
					}
					else {
						// Shader is avaialable but uncompiled
						return ProgramLinkResult::new(ProgramLinkStatus::UncompiledShader, None);
					}
				}
			}
			
			// Link program
			gl::LinkProgram(self.program_gl);
			
			// Get link status
			let mut link_status: gl::int = 0;
			gl::GetProgramiv(self.program_gl, gl::LINK_STATUS, &mut link_status);
			let successful_link = link_status == gl::TRUE as gl::int;
			successful = successful_link;
			
			// Query the info log
			let query_info_log = if successful_link {options.capture_success_info_log} else {options.capture_failure_info_log};
			
			info_log = if query_info_log {
				// Query info log length
				let mut info_log_buffer_size: gl::int = 0;
				gl::GetProgramiv(self.program_gl, gl::INFO_LOG_LENGTH, &mut info_log_buffer_size);
				
				// Allocate info log buffer
				let mut info_log_buffer = vec![0u8; info_log_buffer_size as usize];
				
				// Actually get the info log
				let mut info_log_string_length: gl::sizei = 0;
				gl::GetProgramInfoLog(self.program_gl, info_log_buffer_size as gl::sizei, &mut info_log_string_length, info_log_buffer.as_mut_ptr() as *mut gl::char);
				
				// Truncate the buffer to the actual length in chars (should be the same minus the null terminator, so minus 1)
				info_log_buffer.truncate(info_log_string_length as usize);
				
				// Make into string
				let info_log_string = String::from_utf8_unchecked(info_log_buffer);
				Some(info_log_string)
			} else {
				None
			};
			
			// Detach all shaders again
			for stage in ShaderStage::stages() {
				if let Some(shader) = self.attached_shader(stage) {
					if let Some(shader_gl) = shader.shader_gl() {
						gl::DetachShader(self.program_gl, shader_gl);
					}
				}
			}
		}
		
		// Make result object
		let status = if successful {ProgramLinkStatus::Success} else {ProgramLinkStatus::LinkageError};
		let result = ProgramLinkResult::new(status, info_log);
		result
	}
	
	/// Disposes this programs OpenGL program object by deleting it.
	pub fn dispose(&mut self) {
		if self.program_gl != 0 {
			unsafe {
				// Delete the program itself
				let name = mem::replace(&mut self.program_gl, 0);
				gl::DeleteProgram(name);
			}
		}
	}
	
	pub fn new() -> Self {
		Self {
			vertex_shader: None,
			fragment_shader: None,
			tess_control_shader: None,
			tess_eval_shader: None,
			geometry_shader: None,
			compute_shader: None,
			
			program_gl: 0,
		}
	}
}

impl Drop for ShaderProgram {
	fn drop(&mut self) {
		// Make sure the gl program gets deleted
		self.dispose()
	}
}

pub struct ProgramLinkOptions {
	pub capture_success_info_log: bool,
	pub capture_failure_info_log: bool,
}

impl ProgramLinkOptions {
	pub fn with_info_log(&mut self, capture_on_success: bool, capture_on_failure: bool) -> &mut Self {
		self.capture_success_info_log = capture_on_success;
		self.capture_failure_info_log = capture_on_failure;
		self
	}
}

impl Default for ProgramLinkOptions {
	fn default() -> Self {
		Self {
			capture_success_info_log: false,
			capture_failure_info_log: true,
		}
	}
}

pub struct ProgramLinkResult {
	pub info_log: Option<String>,
	pub status: ProgramLinkStatus,
}

impl ProgramLinkResult {
	pub fn status(&self) -> ProgramLinkStatus {
		self.status
	}
	
	pub fn info_log(&self) -> Option<&str> {
		self.info_log.as_ref().map(|l| l.as_str())
	}
	
	pub fn new(status: ProgramLinkStatus, info_log: Option<String>) -> Self {
		Self {
			status,
			info_log,
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum ProgramLinkStatus {
	Success,
	LinkageError,
	UncompiledShader,
}
