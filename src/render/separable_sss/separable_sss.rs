use gl_bindings::gl;
use std::cell::RefCell;
use std::rc::Rc;
use cgmath::{Rad, Deg};
use crate::render::{RenderSubsystem, ReconfigureEvent, Framebuffer, Texture, ShaderProgram, AttachmentPoint, FramebufferAttachment, ImageFormat};
use crate::demo::demo_instance;
use crate::utils::lazy_option::Lazy;

pub struct SeparableSSSSubsystem {
	pub fbo_resolve_intermediate: Framebuffer,
	pub fbo_resolve_final: Framebuffer,
	
	pub program_sss_resolve: Option<Rc<RefCell<ShaderProgram>>>,
}

impl SeparableSSSSubsystem {
	pub fn new() -> SeparableSSSSubsystem {
		SeparableSSSSubsystem {
			fbo_resolve_intermediate: Framebuffer::new(0, 0),
			fbo_resolve_final: Framebuffer::new(0, 0),
			
			program_sss_resolve: None,
		}
	}
	
	pub fn do_resolve_sss(&mut self, scene_hdr_rt: &Texture, scene_depth_rt: &Texture, camera_fovy: Rad<f32>, depth_planes: (f32, f32)) {
		unsafe {
			// Setup pipeline
			gl::Disable(gl::DEPTH_TEST);
			gl::Disable(gl::CULL_FACE);
			gl::Disable(gl::BLEND);
			
			gl::DisableVertexAttribArray(0);
			
			// Bind shader
			let resolve_shader = RefCell::borrow(self.program_sss_resolve.need());
			gl::UseProgram(resolve_shader.program_gl());
			
			// Upload uniform params
			gl::Uniform1f(gl::GetUniformLocation(resolve_shader.program_gl(), "uCameraFovyRad\0".as_ptr() as *const gl::char), camera_fovy.0);
			gl::Uniform2f(gl::GetUniformLocation(resolve_shader.program_gl(), "uCameraDepthPlanes\0".as_ptr() as *const gl::char), depth_planes.0, depth_planes.1);
			
			let pass_dir_uniform_location = gl::GetUniformLocation(resolve_shader.program_gl(), "uSeperablePassDir\0".as_ptr() as *const gl::char);
			
			// Bind depth texture
			gl::BindTextureUnit(1, scene_depth_rt.texture_gl());
			
			{// Render first resolve pass
				// Bind fbo
				gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo_resolve_intermediate.handle_gl());
				
				// Bind source textures
				gl::BindTextureUnit(0, scene_hdr_rt.texture_gl());
				
				// Upload pass dir uniform
				gl::Uniform2f(pass_dir_uniform_location, 1.0, 0.0);
				
				// Render screen trianble
				gl::DrawArrays(gl::TRIANGLES, 0, 3);
			}
			
			{// Render second resolve pass
				// Bind fbo
				gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo_resolve_final.handle_gl());
				
				// Bind source textures
				let intermediate_scene_rt = RefCell::borrow(&self.fbo_resolve_intermediate.get_attachment(AttachmentPoint::Color(0)).unwrap().texture);
				gl::BindTextureUnit(0, intermediate_scene_rt.texture_gl());
				
				// Upload pass dir uniform
				gl::Uniform2f(pass_dir_uniform_location, 0.0, 1.0);
				
				// Render screen trianble
				gl::DrawArrays(gl::TRIANGLES, 0, 3);
			}
		}
	}
	
	pub fn reload_shaders(&mut self) {
		// DEBUG: Log
		println!("Reloaded ssss resolve shader");
		
		// DEBUG: Get asset folder
		let asset_folder = demo_instance().asset_folder.as_ref().unwrap().as_path();
		
		// Load shaders
		self.program_sss_resolve = Some({
			let mut s = ShaderProgram::new_from_file(
				&asset_folder.join("shaders/separable_sss_resolve.vert.glsl"),
				&asset_folder.join("shaders/separable_sss_resolve.frag.glsl"),
				None,
			);
			s.compile();
			Rc::new(RefCell::new(s))
		});
	}
}

impl RenderSubsystem for SeparableSSSSubsystem {
	fn initialize(&mut self) {
		// Create fbo
		self.fbo_resolve_intermediate = {
			let mut fbo = Framebuffer::new(0, 0);
			fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Depth, ImageFormat::get(gl::DEPTH_COMPONENT32F)));
			fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Color(0), ImageFormat::get(gl::R11F_G11F_B10F)));
			fbo
		};
		self.fbo_resolve_final = {
			let mut fbo = Framebuffer::new(0, 0);
			fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Depth, ImageFormat::get(gl::DEPTH_COMPONENT32F)));
			fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Color(0), ImageFormat::get(gl::R11F_G11F_B10F)));
			fbo
		};
	}
	
	fn deinitialize(&mut self) {
		// Do nothing
	}
	
	fn reconfigure(&mut self, event: ReconfigureEvent<'_>) {
		// Allocate framebuffers
		let fbo = &mut self.fbo_resolve_intermediate;
		fbo.resize(event.resolution.0, event.resolution.1);
		fbo.allocate();
		
		let fbo = &mut self.fbo_resolve_final;
		fbo.resize(event.resolution.0, event.resolution.1);
		fbo.allocate();
		
		// Delete old shader
		if let Some(program) = self.program_sss_resolve.take() {
			let mut program = RefCell::borrow_mut(&program);
			program.delete();
		}
		
		// Load shaders
		self.reload_shaders();
	}
}
