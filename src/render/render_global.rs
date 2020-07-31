use std::error;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Mutex;
use gl_bindings::gl;
use cgmath::{Matrix4, SquareMatrix, vec3, Point3, Rad, Vector3};
use crate::demo;
use crate::utils::lazy_option::Lazy;
use crate::render::{Framebuffer, FramebufferAttachment, AttachmentPoint, ImageFormat, RenderSubsystem, Texture};
use crate::render::separable_sss::SeparableSSSSubsystem;
use crate::render::shader::managed::ManagedProgram;
use crate::asset::AssetPathBuf;

#[derive(Copy, Clone, Debug)]
pub enum AntialiasingMode {
	None,
	MSAA {samples: u32},
	SDAA {coverage_samples: u32},
	SCAA {color_samples: u32, coverage_samples: u32},
}

pub struct RenderGlobal {
	current_configuration: Rc<RefCell<GraphicsConfiguration>>,
	current_resolution: (u32, u32),
	
	separable_sss_system: SeparableSSSSubsystem,
	
	framebuffer_scene_hdr_ehaa: Option<Rc<RefCell<Framebuffer>>>,
	
	program_ehaa_scene: ManagedProgram,
	program_post_composite: ManagedProgram,
	
	frametime_query_object_gl: gl::uint,
	
	queued_shader_reload: bool,
	
	enable_bary_tess: bool,
	antialiasing_mode: AntialiasingMode,
	
//	enable_sdaa: bool,
//	num_sdaa_samples: u32,
//	enable_nsaa: bool,
}

impl RenderGlobal {
	pub fn new() -> RenderGlobal {
		RenderGlobal {
			current_configuration: Rc::new(RefCell::new(GraphicsConfiguration::new())),
			current_resolution: (0, 0),
			
			separable_sss_system: SeparableSSSSubsystem::new(),
			
			framebuffer_scene_hdr_ehaa: None,
			
			program_ehaa_scene: ManagedProgram::new(Some(AssetPathBuf::from("/shaders/legacy/main_scene_forward.program"))),
			program_post_composite: ManagedProgram::new(Some(AssetPathBuf::from("/shaders/post_composite.program"))),
			
			frametime_query_object_gl: 0,
			
			queued_shader_reload: false,
			
			enable_bary_tess: false,
			
//			antialiasing_mode: AntialiasingMode::None,
			antialiasing_mode: AntialiasingMode::MSAA {samples: 4},
//			antialiasing_mode: AntialiasingMode::SDAA {coverage_samples: 4},
//			antialiasing_mode: AntialiasingMode::SCAA {color_samples: 2, coverage_samples: 4},
			
//			enable_sdaa: true,
//			num_sdaa_samples: 4,
//			num_color_samples: 4,
//			
//			enable_nsaa: false,
		}
	}
	
	pub fn initialize(&mut self, resolution: (u32, u32)) -> Result<(), Box<dyn error::Error>> {
		// Set initial resolution
		self.current_resolution = resolution;
		
		// Init subsystems
		self.separable_sss_system.initialize();
		
		// Do initial reconfiguration
		self.do_reconfigure_pipeline(self.current_resolution, false)?;
		
		Ok(())
	}
	
	pub fn do_reconfigure_pipeline(&mut self, new_resolution: (u32, u32), only_resize: bool) -> Result<(), Box<dyn error::Error>> {
		// Update state
		self.current_resolution = new_resolution;
		
		let config = RefCell::borrow(&self.current_configuration);
		let event = ReconfigureEvent {
			configuration: config.deref(),
			resolution: new_resolution,
			only_resize,
		};
		
		// Configure main fbo
		if let Some(t) = &mut self.framebuffer_scene_hdr_ehaa {
			let mut fbo = RefCell::borrow_mut(t);
			fbo.resize(event.resolution.0, event.resolution.1);
		}
		else {
			// Create fbo
			self.framebuffer_scene_hdr_ehaa = Some(Rc::new(RefCell::new({
				let mut fbo = Framebuffer::new(event.resolution.0, event.resolution.1);
				
				fn add_attachment(framebuffer: &mut Framebuffer, attachment_point: AttachmentPoint, internal_format: ImageFormat, samples: Option<u32>) {
					let texture = if let Some(n) = samples {
						Rc::new(RefCell::new(Texture::new_ms(0, 0, n, internal_format)))
					} else {
						Rc::new(RefCell::new(Texture::new(0, 0, 1, internal_format)))
					};
					framebuffer.add_attachment(FramebufferAttachment::from_texture(attachment_point, texture, 0));
				}
//				let color_samples = Some().filter(|n| *n > 1);
//				let depth_samples = self.enable_sdaa.then_some(self.num_sdaa_samples).filter(|n| *n > 1);
				
				use AntialiasingMode as AA;
				let (color_samples, depth_samples) = match &self.antialiasing_mode {
					AA::MSAA {samples} => (Some(*samples), Some(*samples)),
					AA::SDAA {coverage_samples} => (None, Some(*coverage_samples)),
					AA::SCAA {color_samples, coverage_samples} => (Some(*color_samples), Some(*coverage_samples)),
					AA::None | _ => (None, None),
				};
				
				add_attachment(&mut fbo, AttachmentPoint::Depth, ImageFormat::get(gl::DEPTH_COMPONENT24), depth_samples);
				add_attachment(&mut fbo, AttachmentPoint::Color(0), ImageFormat::get(gl::R11F_G11F_B10F), color_samples);
//				add_attachment(&mut fbo, AttachmentPoint::Color(1), ImageFormat::get(gl::R8UI), color_samples);
//				add_attachment(&mut fbo, AttachmentPoint::Color(2), ImageFormat::get(gl::RGBA8), color_samples);
				
//				add_attachment(&mut fbo, AttachmentPoint::Color(3), ImageFormat::get(gl::R11F_G11F_B10F), color_samples);
//				add_attachment(&mut fbo, AttachmentPoint::Color(4), ImageFormat::get(gl::R11F_G11F_B10F), color_samples);
//				add_attachment(&mut fbo, AttachmentPoint::Color(5), ImageFormat::get(gl::R11F_G11F_B10F), color_samples);
				
////				fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Depth, ImageFormat::get(gl::DEPTH_COMPONENT32F)));
//				if !self.enable_sdaa {fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Depth, ImageFormat::get(gl::DEPTH_COMPONENT24)));}
//				else {fbo.add_attachment(FramebufferAttachment::from_texture(AttachmentPoint::Depth, Rc::new(RefCell::new(Texture::new_ms(0, 0, self.num_sdaa_samples, ImageFormat::get(gl::DEPTH_COMPONENT24)))), 0));} // Multisampled depth attachment for SDAA
////				fbo.add_attachment(FramebufferAttachment::from_texture(AttachmentPoint::Depth, Rc::new(RefCell::new(Texture::new_ms(0, 0, self.num_sdaa_samples, ImageFormat::get(gl::DEPTH_COMPONENT32F)))), 0)); // Multisampled depth attachment for SDAA
//				if self.num_color_samples <= 1 {fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Color(0), ImageFormat::get(gl::R11F_G11F_B10F)));}
//				else {fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Color(0), ImageFormat::get(gl::R11F_G11F_B10F)));}
////				fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Color(0), ImageFormat::get(gl::R11F_G11F_B10F)));
////				fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Color(1), ImageFormat::get(gl::RGBA8UI)));
////				fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Color(1), ImageFormat::get(gl::R8UI)));
////				fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Color(2), ImageFormat::get(gl::RGB8)));
////				fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Color(1), ImageFormat::get(gl::RGBA8)));
////				fbo.add_attachment(FramebufferAttachment::from_new_texture(AttachmentPoint::Color(1), ImageFormat::get(gl::RG16_SNORM)));
				
				fbo.allocate();
				fbo
			})));
		}
		
		// DEBUG: Get main fbo completeness and stats
		unsafe {
			let main_fbo = self.framebuffer_scene_hdr_ehaa.need().borrow();
			let status = gl::CheckNamedFramebufferStatus(main_fbo.handle_gl(), gl::FRAMEBUFFER);
			if status != gl::FRAMEBUFFER_COMPLETE {
				panic!("Main framebuffer incomplete: {:?}", status);
			}
			else {
				println!("Main framebuffer complete: {:?}", status)
			}
		}
		
		// DEBUG: Setup framebuffer sample locations
		unsafe {
			let main_fbo = self.framebuffer_scene_hdr_ehaa.need().borrow();
			
			let enable_sample_locations = if let AntialiasingMode::SDAA {..} = &self.antialiasing_mode {true} else {false};
			gl::NamedFramebufferParameteri(main_fbo.handle_gl(), gl::FRAMEBUFFER_PROGRAMMABLE_SAMPLE_LOCATIONS_ARB, if enable_sample_locations {16} else {0});
			gl::NamedFramebufferParameteri(main_fbo.handle_gl(), gl::FRAMEBUFFER_SAMPLE_LOCATION_PIXEL_GRID_ARB, 0);
			
			let mut sample_locations = [0.5f32; 16];
			
//			{// 5 samples
//				sample_locations[0] = 0.5;
//				sample_locations[1] = 0.5;
//				
//				// Down
//				sample_locations[2] = 0.5+0.125;
//				sample_locations[3] = 0.0+0.125;
//				
//				// Up
//				sample_locations[4] = 0.5-0.125;
//				sample_locations[5] = 1.0-0.125;
//				
//				// Left
//				sample_locations[6] = 0.0+0.125;
//				sample_locations[7] = 0.5+0.125;
//				
//				// Right
//				sample_locations[8] = 1.0-0.125;
//				sample_locations[9] = 0.5-0.125;
//			}
			
			{
				// NOTE: Color sample to coverage sample association
				// is implementation dependent so we really are just
				// hoping for the best when defining the first two
				// samples as our color samples.
				
				// Southwest (down right)
//				sample_locations[0] = 0.5-0.25;
//				sample_locations[1] = 0.5-0.125;
				sample_locations[0] = 0.5;
				sample_locations[1] = 0.5;
				
				// Northeast (up right)
				sample_locations[2] = 0.5+0.25;
				sample_locations[3] = 0.5+0.125;
				
				// Northwest (up left)
				sample_locations[4] = 0.5-0.125;
				sample_locations[5] = 0.5+0.25;
				
				// Southeast (down right)
				sample_locations[6] = 0.5+0.125;
				sample_locations[7] = 0.5-0.25;
				
				// South (down)
				sample_locations[8] = 0.5-0.125;
				sample_locations[9] = 0.0+0.125;
				
				// North (up)
				sample_locations[10] = 0.5+0.125;
				sample_locations[11] = 1.0-0.125;
				
				// West (left)
				sample_locations[12] = 0.0+0.125;
				sample_locations[13] = 0.5+0.125;
				
				// East (right)
				sample_locations[14] = 1.0-0.125;
				sample_locations[15] = 0.5-0.125;
			}
			
//			{
//				// https://docs.microsoft.com/en-us/windows/win32/api/d3d11/ne-d3d11-d3d11_standard_multisample_quality_levels
//				// (sample 0 and 1 are swapped)
//				
//				// Color sample
//				sample_locations[0] = 0.5 + (-1.0 / 16.0);
//				sample_locations[1] = 0.5 + (3.0 / 16.0);
//				
//				// Up 1
//				sample_locations[2] = 0.5 + (1.0 / 16.0);
//				sample_locations[3] = 0.5 + (-3.0 / 16.0);
//				
//				// Right 1
//				sample_locations[4] = 0.5 + (5.0 / 16.0);
//				sample_locations[5] = 0.5 + (1.0 / 16.0);
//				
//				// Up 2 
//				sample_locations[6] = 0.5 + (-3.0 / 16.0);
//				sample_locations[7] = 0.5 + (-5.0 / 16.0);
//				
//				// Left 1
//				sample_locations[8] = 0.5 + (-5.0 / 16.0);
//				sample_locations[9] = 0.5 + (5.0 / 16.0);
//				
//				// Left 2
//				sample_locations[10] = 0.5 + (-7.0 / 16.0);
//				sample_locations[11] = 0.5 + (-1.0 / 16.0);
//				
//				// Down 1
//				sample_locations[12] = 0.5 + (3.0 / 16.0);
//				sample_locations[13] = 0.5 + (7.0 / 16.0);
//				
//				// Right 2
//				sample_locations[14] = 0.0 + (7.0 / 16.0);
//				sample_locations[15] = 0.5 + (-7.0 / 16.0);
//			}
			
			let mut sample_location_grid_buffer = vec![0f32; 4*4*8*2];
			sample_location_grid_buffer.chunks_exact_mut(8*2).for_each(|p| {
				p.copy_from_slice(&sample_locations);
			});
			
			gl::NamedFramebufferSampleLocationsfvARB(main_fbo.handle_gl(), 0, 16, &sample_locations as *const gl::float);
//			gl::NamedFramebufferSampleLocationsfvARB(main_fbo.handle_gl(), 0, 16, sample_location_grid_buffer.as_ptr() as *const gl::float);
		}
		
		// Configure coverage modulation table
		unsafe {
			// Query table size
			let mut modulation_table_size: gl::int = 0;
			gl::GetIntegerv(gl::COVERAGE_MODULATION_TABLE_SIZE_NV, &mut modulation_table_size);
			println!("COVERAGE_MODULATION_TABLE_SIZE_NV = {}", modulation_table_size);
			
			let mut modulation_table_buffer = vec![0f32; modulation_table_size as usize];
			
			for (i, v) in modulation_table_buffer.iter_mut().enumerate() {
				*v = i as f32 / modulation_table_size as f32;
//				*v = ((i / 4) * 4) as f32 / modulation_table_size as f32;
//				*v = if ((i + 1) as f32 / modulation_table_size as f32) < (4.0 / 16.0) {0.0} else {1.0};
			}
			
			println!("{:?}", modulation_table_buffer);
			
			gl::CoverageModulationTableNV(modulation_table_size, modulation_table_buffer.as_ptr());
		}
		
		// DEBUG: Get mixed_samples support
		unsafe {
			let mut max_raster_samples: gl::int = 0;
			gl::GetIntegerv(gl::MAX_RASTER_SAMPLES_EXT, &mut max_raster_samples);
			
			let mut sample_locations_table_size: gl::int = 0;
			gl::GetIntegerv(gl::PROGRAMMABLE_SAMPLE_LOCATION_TABLE_SIZE_ARB, &mut sample_locations_table_size);
			
			let (mut sample_grid_width, mut sample_grid_height): (gl::int, gl::int) = (0, 0);
			gl::GetIntegerv(gl::SAMPLE_LOCATION_PIXEL_GRID_WIDTH_ARB, &mut sample_grid_width);
			gl::GetIntegerv(gl::SAMPLE_LOCATION_PIXEL_GRID_HEIGHT_ARB, &mut sample_grid_height);
			
			println!("MAX_RASTER_SAMPLES_EXT = {}", max_raster_samples);
			println!("PROGRAMMABLE_SAMPLE_LOCATION_TABLE_SIZE_ARB = {}", sample_locations_table_size);
			println!("SAMPLE_LOCATION_PIXEL_GRID_WIDTH_ARB = {}", sample_grid_width);
			println!("SAMPLE_LOCATION_PIXEL_GRID_HEIGHT_ARB = {}", sample_grid_height);
		}
		
		// Reconfigure subsystems
		self.separable_sss_system.reconfigure(event);
		
		// Drop config for now
		drop(config);
		
		// Create query object
		if self.frametime_query_object_gl == 0 {
			self.frametime_query_object_gl = unsafe {
				let mut query: gl::uint = 0;
				gl::CreateQueries(gl::TIME_ELAPSED, 1, &mut query);
				query
			};
		}
		
		// Load shaders
		self.reload_shaders();
		
		// DEBUG: Check support for framebuffer_mixed_samples
		unsafe {
			let mut mixed_depth_samples_supported: gl::boolean = gl::FALSE;
			gl::GetBooleanv(gl::MIXED_DEPTH_SAMPLES_SUPPORTED_NV, &mut mixed_depth_samples_supported);
			
			if mixed_depth_samples_supported != gl::TRUE {
				panic!("GetBoolean reported MIXED_DEPTH_SAMPLES_SUPPORTED_NV as false");
			}
		}
		
		Ok(())
	}
	
	fn reload_shaders(&mut self) {
//		let asset_folder = demo::demo_instance().asset_folder.as_mut().unwrap();
		
		// Log
		println!("Reloading shaders!");
		
		// Reload shaders from asset
		self.program_ehaa_scene.reload_from_asset().expect("Failed to reload scene shader from asset");
		self.program_post_composite.reload_from_asset().expect("Failed to reload post composite shader from asset");
		
//		// Delete old shaders
//		if let Some(program) = self.program_ehaa_scene.take() {
//			let mut program = RefCell::borrow_mut(&program);
//			program.delete();
//		}
//		if let Some(program) = self.program_post_resolve.take() {
//			let mut program = RefCell::borrow_mut(&program);
//			program.delete();
//		}
		
		// Reload shader from assets
		
//		// Load shaders
//		self.program_ehaa_scene = Some({
//			let mut s = ShaderProgram::new_from_file(
//				&asset_folder.join("shaders/scene_ehaa.vert.glsl"),
//				&asset_folder.join("shaders/scene_ehaa.frag.glsl"),
//				Some(&asset_folder.join("shaders/scene_ehaa.tesseval.glsl"))
////				None
//			);
//			s.compile();
//			Rc::new(RefCell::new(s))
//		});
//		self.program_post_resolve = Some({
//			let mut s = ShaderProgram::new_from_file(
//				&asset_folder.join("shaders/post_resolve.vert.glsl"),
//				&asset_folder.join("shaders/post_resolve.frag.glsl"),
//				None
//			);
//			s.compile();
//			Rc::new(RefCell::new(s))
//		});
		
		// Set tessellation state
		unsafe {
			gl::PatchParameteri(gl::PATCH_VERTICES, 3);
			gl::PatchParameterfv(gl::PATCH_DEFAULT_OUTER_LEVEL, [1.0f32, 1.0f32, 1.0f32, 1.0f32].as_ptr());
			gl::PatchParameterfv(gl::PATCH_DEFAULT_INNER_LEVEL, [1.0f32, 1.0f32].as_ptr());
		}
		
		// Reload subsystem shaders
		self.separable_sss_system.reload_shaders();
	}
	
	pub fn do_render_frame(&mut self) {
		// Reload shaders if needed
		if self.queued_shader_reload {
			self.queued_shader_reload = false;
			self.reload_shaders();
		}
		
		// Update cam state
		// LATER: Do this when rendering a scene: Get active camera from scene, make CameraState, calc proj matrix, pass state along in functions
		let active_camera = demo::demo_instance().get_test_camera();
		let active_camera = if let Some(cam) = active_camera.upgrade() {
			cam
		} else {
			// No active camera, so don't render anything for now
			return;
		};
		
		let camera_fovy: Rad<f32>;
		let camera_near_z: f32;
		let camera_far_z: f32;
		
		let camera_pos: Vector3<f32>;
		
		let cam_state = {
			let cam = Mutex::lock(&active_camera).unwrap();
			let mut state = RenderCameraState::new();
			
			// Get camera pos
			camera_pos = cam.translation;
			
			// Get camera fovy
//			let projection: &dyn Any = cam.projection.as_ref();
//			let projection: &PerspectiveProjection = projection.downcast_ref::<PerspectiveProjection>().unwrap();
			
			camera_fovy = cam.projection.camera_fovy();
			let (near_z, far_z) = cam.projection.test_depth_planes();
			camera_near_z = near_z;
			camera_far_z = far_z;
			
			// Base matrix for our coordinate system (
			let base_matrix = Matrix4::look_at_dir(Point3 {x: 0.0, y: 0.0, z: 0.0}, vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 0.0)); // For some reason look_at_dir inverts the dir vector
			state.view_matrix = base_matrix * Matrix4::from(cam.rotation) * Matrix4::from_translation(-cam.translation);
			state.projection_matrix = cam.projection.projection_matrix(cam.viewport_size);
			state
		};
		
		let viewprojection_matrix = cam_state.projection_matrix * cam_state.view_matrix;
		
		// Recompile shaders
		if self.program_ehaa_scene.needs_recompile() {
			self.program_ehaa_scene.do_recompile();
		}
		if self.program_post_composite.needs_recompile() {
			self.program_post_composite.do_recompile();
		}
		
		unsafe {
			gl::Disable(gl::FRAMEBUFFER_SRGB);
			gl::Disable(gl::BLEND);
			
			gl::Enable(gl::CULL_FACE);
			gl::FrontFace(gl::CCW);
			gl::CullFace(gl::FRONT); // For some reason we need to cull FRONT. This might be due to reverse-z flipping the winding order?
			
			gl::Enable(gl::DEPTH_TEST);
			
			// Setup NDC z axis for reverse float depth
			gl::DepthFunc(gl::GREATER);
			gl::ClearDepth(0.0); // 0.0 is far with reverse z
			gl::ClipControl(gl::LOWER_LEFT, gl::ZERO_TO_ONE);
			gl::DepthRange(0.0, 1.0); // Standard (non-inversed) depth range, we use a reverse-z projection matrix instead
			
			// Use scene shader
			let scene_shader = self.program_ehaa_scene.program().unwrap();
			let scene_shader_gl = scene_shader.program_gl().unwrap();
			gl::UseProgram(scene_shader_gl);
			
			// Bind scene framebuffer
			let scene_fbo = RefCell::borrow(self.framebuffer_scene_hdr_ehaa.need());
			gl::BindFramebuffer(gl::FRAMEBUFFER, scene_fbo.handle_gl());
			
			// Set the viewport
			gl::Viewport(0, 0, self.current_resolution.0 as gl::sizei, self.current_resolution.1 as gl::sizei);
			
			gl::ClearColor(0.0, 0.0, 0.0, 0.0);
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
			
			{// Upload matrices
				let model_matrix = Matrix4::from_scale(1.0);
				
				let model_matrix_arr: [[f32; 4]; 4] = model_matrix.into();
				gl::UniformMatrix4fv(gl::GetUniformLocation(scene_shader_gl, "uMatrixModel\0".as_ptr() as *const gl::char), 1, gl::FALSE, model_matrix_arr.as_ptr() as *const gl::float);
				
				let view_matrix_arr: [[f32; 4]; 4] = cam_state.view_matrix.into();
				gl::UniformMatrix4fv(gl::GetUniformLocation(scene_shader_gl, "uMatrixView\0".as_ptr() as *const gl::char), 1, gl::FALSE, view_matrix_arr.as_ptr() as *const gl::float);
				
				let viewprojection_matrix_arr: [[f32; 4]; 4] = viewprojection_matrix.into();
				gl::UniformMatrix4fv(gl::GetUniformLocation(scene_shader_gl, "uMatrixViewProjection\0".as_ptr() as *const gl::char), 1, gl::FALSE, viewprojection_matrix_arr.as_ptr() as *const gl::float);
				
				gl::Uniform3f(gl::GetUniformLocation(scene_shader_gl, "uEyePosWorldspace\0".as_ptr() as *const gl::char), camera_pos.x, camera_pos.y, camera_pos.z);
			}
			
			let start_frametimer = {// Start frametime timer
				let mut elapsed_frametime: u64 = std::u64::MAX;
				gl::GetQueryObjectui64v(self.frametime_query_object_gl, gl::QUERY_RESULT_NO_WAIT, &mut elapsed_frametime);
				
				if elapsed_frametime != std::u64::MAX {
					let _float_frametime = (elapsed_frametime as f64) / 1e6;
					
//					let title = format!("EHAA Demo ~ Frametime {} ms", float_frametime);
//					self.window.need_mut().set_title(title.as_str());
					
					// Restart query
					gl::BeginQuery(gl::TIME_ELAPSED, self.frametime_query_object_gl);
					true
				}
				else {
					false
				}
			};
			
			gl::EnableVertexAttribArray(0);
//			gl::EnableVertexAttribArray(1);
//			gl::EnableVertexAttribArray(2);
			
			{// DEBUG: Enable mixed_samples
//				gl::Enable(gl::RASTER_MULTISAMPLE_EXT);
//				gl::RasterSamplesEXT(self.num_sdaa_samples as gl::uint, gl::TRUE);
				
//				gl::Enable(gl::COVERAGE_MODULATION_TABLE_NV);
//				gl::CoverageModulationNV(gl::ALPHA);
				
//				gl::Enable(gl::ALPHA_TEST);
//				gl::AlphaFunc();
				
				{
//					gl::Enablei(gl::BLEND, 0);
//					gl::BlendFunci(0, gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
//					
//					gl::Enablei(gl::BLEND, 3);
//					gl::BlendFunci(3, gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
//					
//					gl::Enablei(gl::BLEND, 4);
//					gl::BlendFunci(4, gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
//					
//					gl::Enablei(gl::BLEND, 5);
//					gl::BlendFunci(5, gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
				}
				
				if let AntialiasingMode::SDAA {..} = self.antialiasing_mode {
					gl::Enable(gl::BLEND);
					
					gl::Enablei(gl::BLEND, 0);
					gl::BlendFunci(0, gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
					
					gl::Enablei(gl::BLEND, 3);
					gl::BlendFunci(3, gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
					
					gl::Enablei(gl::BLEND, 4);
					gl::BlendFunci(4, gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
					
					gl::Enablei(gl::BLEND, 5);
					gl::BlendFunci(5, gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
					
//					gl::Enable(gl::FRAGMENT_COVERAGE_TO_COLOR_NV);
//					gl::FragmentCoverageColorNV(1);
				}
				
				if false/*self.enable_nsaa*/ {
					gl::Enablei(gl::BLEND, 0);
					gl::BlendFunci(0, gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
					
//					gl::Enablei(gl::BLEND, 1);
					
//					gl::Enable(gl::CONSERVATIVE_RASTERIZATION_NV);
//					gl::BlendFunci();
				}
				
//				gl::BlendFunc(gl::ONE, gl::ZERO);
//				gl::BlendEquation(gl::FUNC_ADD);
				
//				gl::Enable(gl::CONSERVATIVE_RASTERIZATION_NV);
			}
			
			/*
			{// Draw teapot
				let test_teapot_vbo = demo::demo_instance().test_teapot_vbo.need();
				
				gl::BindBuffer(gl::ARRAY_BUFFER, test_teapot_vbo.vbo_gl);
				gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, 0 as *const gl::void);
				
				gl::DrawArrays(gl::PATCHES, 0, (crate::render::teapot::TEAPOT_VERTEX_DATA.len() / 3) as gl::sizei);
			}
			*/
			
//			/*
			{// Draw head model
				let test_head_model = demo::demo_instance().test_head_model.need();
				
				// Bind textures
				gl::BindTextureUnit(1, test_head_model.tex_albedo.texture_gl());
				gl::BindTextureUnit(2, test_head_model.tex_normal.texture_gl());
				gl::BindTextureUnit(4, test_head_model.tex_transmission.texture_gl());
				
				gl::BindBuffer(gl::ARRAY_BUFFER, test_head_model.vertex_buffer_gl);
//				let stride = 8*4;
				let stride = 12*4;
				gl::EnableVertexAttribArray(0);
				gl::EnableVertexAttribArray(1);
				gl::EnableVertexAttribArray(2);
				gl::EnableVertexAttribArray(3);
				gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, 0 as *const gl::void); // vertex
				gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3*4 + 3*4) as *const gl::void); // texcoord
				gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, stride, (3*4) as *const gl::void); // normal
				gl::VertexAttribPointer(3, 4, gl::FLOAT, gl::FALSE, stride, (3*4 + 3*4 + 2*4) as *const gl::void); // tangent
				
				gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, test_head_model.index_buffer_gl);
				
				let prim_mode = if self.enable_bary_tess {gl::PATCHES} else {gl::TRIANGLES};
//				gl::DrawElements(gl::PATCHES, test_head_model.num_indices as gl::sizei, gl::UNSIGNED_INT, 0 as *const gl::void);
//				gl::DrawElements(gl::TRIANGLES, self.test_head_model.need().num_indices as gl::GLsizei, gl::UNSIGNED_INT, 0 as *const std::ffi::c_void);
				gl::DrawElementsInstanced(prim_mode, test_head_model.num_indices as gl::sizei, gl::UNSIGNED_INT, 0 as *const gl::void, 64);
				
				gl::DisableVertexAttribArray(0);
				gl::DisableVertexAttribArray(1);
				gl::DisableVertexAttribArray(2);
				gl::DisableVertexAttribArray(3);
			}
//			*/
			
			/*
			{// Draw debug triangles
				gl::Begin(gl::PATCHES);
//					gl::VertexAttrib3f(2, 1.0, 0.616, 0.984);
					
//					gl::VertexAttribI1ui(1, 0);
					gl::VertexAttrib3f(0, 0.0, 0.1, 0.0);
					
//					gl::VertexAttribI1ui(1, 1);
					gl::VertexAttrib3f(0, 0.5, 0.2, 0.0);
					
					let (mouse_x, mouse_y) = demo::demo_instance().window.need().get_cursor_pos();
					
//					gl::VertexAttribI1ui(1, 2);
					gl::VertexAttrib3f(0, (mouse_x / 1280.0) as f32 * 2.0 - 1.0, 1.0 - (mouse_y / 720.0) as f32 * 2.0, 0.0);
//					gl::Vertex3f(0.1, 0.6 + 0.2*(std::time::UNIX_EPOCH.elapsed().unwrap().as_secs_f32()).sin(), 0.0);
//					gl::Vertex3f(0.1, 0.6, 0.0);
					
//					gl::VertexAttrib3f(2, 0.153, 0.0, 1.0);
//					gl::VertexAttribI1ui(1, 0);
					gl::VertexAttrib3f(0, 0.0, 0.1, 0.0);
					
//					gl::VertexAttribI1ui(1, 1);
					gl::VertexAttrib3f(0, 0.2, 0.6, 0.0);
					
//					gl::VertexAttribI1ui(1, 2);
//					gl::VertexAttrib3f(0, (mouse_x / 1280.0) as f32 * 2.0 - 1.0, 1.0 - (mouse_y / 720.0) as f32 * 2.0, 0.0);
				gl::End();
			}
			*/
			
			// Evaluate depth values at programmable sample locations
			// (ensures correct values if depth values are stored in compressed form)
//			gl::EvaluateDepthValuesARB();
			
			// DEBUG: Disable mixed_samples
//			gl::Disable(gl::RASTER_MULTISAMPLE_EXT);
//			gl::Disable(gl::COVERAGE_MODULATION_TABLE_NV);
//			gl::Disable(gl::ALPHA_TEST);
			gl::Disable(gl::BLEND);
			gl::Disablei(gl::BLEND, 0);
			gl::Disablei(gl::BLEND, 1);
			gl::Disablei(gl::BLEND, 3);
			gl::Disablei(gl::BLEND, 4);
			gl::Disablei(gl::BLEND, 5);
			
			{// Resolve separable sss
				let main_fbo = RefCell::borrow(self.framebuffer_scene_hdr_ehaa.need());
				let scene_hdr_rt = RefCell::borrow(&main_fbo.get_attachment(AttachmentPoint::Color(0)).unwrap().texture);
				let scene_depth_rt = RefCell::borrow(&main_fbo.get_attachment(AttachmentPoint::Depth).unwrap().texture);
				
				// Render ssss
				self.separable_sss_system.do_resolve_sss(&scene_hdr_rt, &scene_depth_rt, camera_fovy, (camera_near_z, camera_far_z));
			}
			
			{// Do post composite pass
				let post_resolve_shader = self.program_post_composite.program().unwrap();
				
// 				// DEBUG: Blit framebuffer
//				gl::BlitNamedFramebuffer(self.framebuffer_scene_hdr_ehaa.need().handle_gl(), 0, 0, 0, 1280, 720, 0, 0, 1280, 720, gl::COLOR_BUFFER_BIT, gl::NEAREST);
				
				gl::Disable(gl::DEPTH_TEST);
				
				// Bind resolve shader
				gl::UseProgram(post_resolve_shader.program_gl().unwrap());
				
				// Bind shaders
				let main_fbo = RefCell::borrow(self.framebuffer_scene_hdr_ehaa.need());
//				gl::BindTextureUnit(0, RefCell::borrow(&main_fbo.get_attachment(AttachmentPoint::Color(0)).unwrap().texture).texture_gl());
 				gl::BindTextureUnit(0, RefCell::borrow(&self.separable_sss_system.fbo_resolve_final.get_attachment(AttachmentPoint::Color(0)).unwrap().texture).texture_gl());
//				gl::BindTextureUnit(1, RefCell::borrow(&main_fbo.get_attachment(AttachmentPoint::Color(1)).unwrap().texture).texture_gl());
//				gl::BindTextureUnit(2, RefCell::borrow(&main_fbo.get_attachment(AttachmentPoint::Color(2)).unwrap().texture).texture_gl());
				
				gl::BindTextureUnit(4, RefCell::borrow(&main_fbo.get_attachment(AttachmentPoint::Depth).unwrap().texture).texture_gl());
				
//				gl::BindTextureUnit(5, RefCell::borrow(&main_fbo.get_attachment(AttachmentPoint::Color(3)).unwrap().texture).texture_gl());
//				gl::BindTextureUnit(6, RefCell::borrow(&main_fbo.get_attachment(AttachmentPoint::Color(4)).unwrap().texture).texture_gl());
//				gl::BindTextureUnit(7, RefCell::borrow(&main_fbo.get_attachment(AttachmentPoint::Color(5)).unwrap().texture).texture_gl());
				
				// Bind back buffer
				gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
				
				// Draw oversized fullscreen triangles
				gl::DisableVertexAttribArray(0);
				gl::DisableVertexAttribArray(1);
				gl::DisableVertexAttribArray(2);
				gl::DrawArrays(gl::TRIANGLES, 0, 3);
			}
			
			// End frametimer query
			if start_frametimer {
				gl::EndQuery(gl::TIME_ELAPSED);
			}
		}
	}
	
	pub fn queue_shader_reload(&mut self) {
		self.queued_shader_reload = true;
	}
}

pub struct GraphicsConfiguration {
	
}

impl GraphicsConfiguration {
	pub fn new() -> GraphicsConfiguration {
		GraphicsConfiguration {}
	}
}

pub struct ReconfigureEvent<'a> {
	pub configuration: &'a GraphicsConfiguration,
	pub resolution: (u32, u32),
	pub only_resize: bool,
}

pub struct RenderCameraState {
	pub projection_matrix: Matrix4<f32>,
	pub view_matrix: Matrix4<f32>,
}

impl RenderCameraState {
	pub fn new() -> RenderCameraState {
		RenderCameraState {
			projection_matrix: Matrix4::identity(),
			view_matrix: Matrix4::identity(),
		}
	}
}
