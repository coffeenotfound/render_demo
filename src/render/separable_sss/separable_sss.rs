use gl_bindings::gl;
use std::cell::RefCell;
use cgmath::{Rad};
use crate::render::{RenderSubsystem, ReconfigureEvent, Framebuffer, Texture, AttachmentPoint, FramebufferAttachment, ImageFormat};
use crate::render::shader::{UniformLocationCache};
use crate::render::shader::managed::{ManagedProgram};
use crate::asset::AssetPathBuf;

pub struct SeparableSSSSubsystem {
	pub fbo_resolve_intermediate: Framebuffer,
	pub fbo_resolve_final: Framebuffer,
	
	pub program_sss_resolve: /*Rc<RefCell<*/ManagedProgram/*>>*/,
	pub uniforms_sss_resolve: UniformsSSSResolve,
}

impl SeparableSSSSubsystem {
	pub fn new() -> SeparableSSSSubsystem {
		SeparableSSSSubsystem {
			fbo_resolve_intermediate: Framebuffer::new(0, 0),
			fbo_resolve_final: Framebuffer::new(0, 0),
			
			program_sss_resolve: /*Rc::new(RefCell::new(*/ManagedProgram::new(Some(AssetPathBuf::from("/shaders/separable_sss_resolve.program")))/*))*/,
			uniforms_sss_resolve: UniformsSSSResolve {
				global_sss_width: UniformLocationCache::new("uGlobalSSSWidth"),
				separable_pass_dir: UniformLocationCache::new("uSeparablePassDir"),
				distance_to_projection_window: UniformLocationCache::new("uDistanceToProjectionWindow"),
				camera_depth_planes: UniformLocationCache::new("uCameraDepthPlanes"),
			}
		}
	}
	
	pub fn do_resolve_sss(&mut self, scene_hdr_rt: &Texture, scene_depth_rt: &Texture, camera_fovy: Rad<f32>, depth_planes: (f32, f32)) {
		unsafe {
			// Setup pipeline
			gl::Disable(gl::DEPTH_TEST);
			gl::Disable(gl::CULL_FACE);
			gl::Disable(gl::BLEND);
			
			gl::DisableVertexAttribArray(0);
			
			// Compile shader
			let resolve_shader = &mut self.program_sss_resolve;
			if resolve_shader.needs_recompile() {
				resolve_shader.do_recompile();
			}
			
			// Bind shader
			let resolve_shader = &resolve_shader.program().unwrap();
			let resolve_shader_gl = resolve_shader.program_gl().unwrap();
			gl::UseProgram(resolve_shader_gl);
			
			// Upload uniform params
//			gl::Uniform1f(self.uniforms_sss_resolve.distance_to_projection_window.get(resolve_shader_gl), camera_fovy.0);
			gl::Uniform1f(self.uniforms_sss_resolve.distance_to_projection_window.get(resolve_shader_gl).unwrap(), 1.0 / f32::tan(0.5 * camera_fovy.0));
//			gl::Uniform2f(gl::GetUniformLocation(resolve_shader.program_gl(), "uCameraDepthPlanes\0".as_ptr() as *const gl::char), depth_planes.0, depth_planes.1);
			gl::Uniform2f(self.uniforms_sss_resolve.camera_depth_planes.get(resolve_shader_gl).unwrap(), depth_planes.0, depth_planes.1);
			
//			let pass_dir_uniform_location = gl::GetUniformLocation(resolve_shader.program_gl(), "uSeperablePassDir\0".as_ptr() as *const gl::char);
			let pass_dir_uniform_location = self.uniforms_sss_resolve.separable_pass_dir.get(resolve_shader_gl).unwrap();
			
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
		// Reload shader from asset
		self.program_sss_resolve.reload_from_asset().expect("Failed to reload sss resolve shader from asset");
		
//		self.program_sss_resolve = Some({
//			let mut s = ShaderProgram::new_from_file(
//				&asset_folder.join("shaders/separable_sss_resolve.vert.glsl"),
//				&asset_folder.join("shaders/separable_sss_resolve.frag.glsl"),
//				None,
//			);
//			s.compile();
//			Rc::new(RefCell::new(s))
//		});
		
		// DEBUG: Log
		println!("Reloaded ssss resolve shader");
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
		
//		// Delete old shader
//		if let Some(program) = self.program_sss_resolve.take() {
//			let mut program = RefCell::borrow_mut(&program);
//			program.delete();
//		}
		
		// Load shaders
		self.reload_shaders();
	}
}

pub struct UniformsSSSResolve {
	pub global_sss_width: UniformLocationCache, // float
	pub separable_pass_dir: UniformLocationCache, // vec2
	pub distance_to_projection_window: UniformLocationCache, // float
	pub camera_depth_planes: UniformLocationCache, // vec2
}
