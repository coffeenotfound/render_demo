use std::cell::RefCell;
use std::error;
use std::ffi::CStr;
use std::fs::OpenOptions;
use std::panic;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{self, Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use byte_slice_cast::*;
use byteorder::{ByteOrder, LittleEndian};
use cgmath::{Deg, InnerSpace, Quaternion, Rad, Rotation, vec2, vec3, Vector2, Vector3};
use gl_bindings::gl;
use glfw::{SwapInterval, WindowEvent};
use crate::asset::{ASSET_MANAGER_INSTANCE, AssetPath};
use crate::camera::{Camera, OrbitAngles, PerspectiveProjection};
use crate::camera::utils::fovx_to_fovy;
use crate::model::ply::{PlyMeshLoader, PlyReadError, PullEvent};
use crate::render::{ImageFormat, RenderGlobal, TestVertexBuffer, Texture};
use crate::render::separable_sss::{DEFAULT_HUMAN_SKIN_FALLOFF_FACTORS, DEFAULT_HUMAN_SKIN_STRENGTH_FACTORS, SubsurfaceKernelGenerator};
use crate::utils::lazy_option::Lazy;
use crate::utils::option_overwrite::OptionOverwrite;
use crate::windowing::{GlfwContext, Window};

pub static mut DEMO_INSTANCE: Option<Demo> = None;

pub fn start() {
	// Init the demo object
	let demo = Demo::init().expect("Failed to init demo");
	unsafe {
		DEMO_INSTANCE = Some(demo);
	}
	
	// Run the demo
	demo_instance().run();
}

pub fn demo_instance() -> &'static mut Demo {
	unsafe {&mut DEMO_INSTANCE}.as_mut().expect("Demo instance not initialized yet (how the in the hel-")
}

pub struct Demo {
	#[deprecated]
	pub asset_folder: PathBuf,
	
	pub glfw_context: Rc<RefCell<GlfwContext>>,
	pub main_window: Option<Rc<RefCell<Window>>>,
//	pub window: Option<glfw::Window>,
//	pub window_channel: Option<Receiver<(f64, WindowEvent)>>,
	
	pub render_global: RenderGlobal,
	
	pub test_teapot_vbo: Option<TestVertexBuffer>,
	pub test_head_model: Option<TestHeadModel>,
	
	pub test_active_camera: Option<Arc<Mutex<Camera>>>,
	pub test_camera_orbit: OrbitAngles,
	pub test_camera_carousel_state: DemoCameraCarouselState,
}

impl Demo {
	pub fn init() -> Result<Demo, Box<dyn error::Error>> {
		let asset_folder;
		{// Resolve asset folder
			let current_dir = PathBuf::from(std::env::current_dir().unwrap().as_os_str().to_os_string().into_string().unwrap().replace("\\", "/"));
			
			asset_folder = current_dir.join("assets/");
			println!("Asset folder '{}'", asset_folder.as_path().display());
			
			unsafe {
				ASSET_MANAGER_INSTANCE.init(asset_folder.clone());
			}
		}
		
		// Init glfw
		let glfw_context = GlfwContext::init()?;
		
		// Make instance and return
		Ok(Self {
			asset_folder,
			
			glfw_context: Rc::new(RefCell::new(glfw_context)),
			main_window: None,
//			window: None,
//			window_channel: None,
			
			render_global: RenderGlobal::new(),
			
			test_teapot_vbo: None,
			test_head_model: None,
			
			test_active_camera: None,
			test_camera_orbit: {let mut a = OrbitAngles::new_zero(vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, -1.0)); a.distance = 6.0; a.center = vec3(0.0, 1.75, 0.0); a},
			test_camera_carousel_state: DemoCameraCarouselState::new(),
		})
	}
	
	pub fn run(&mut self) {
		//let resolution = (1280, 720);
		let resolution = (1600, 900);
		
		{// Create the main window
			let window = Window::new(Rc::clone(&self.glfw_context));
			let window = self.main_window.overwrite(window);
			let mut window = RefCell::borrow_mut(window);
			
			window.init();
			window.set_title(String::from("Render Demo"));
			window.resize(resolution.0, resolution.1);
			
			// Center window
			window.center_on_screen();
			
			// Make window visible
			window.make_visible(true);
			window.update();
			
			// Swap buffers to clear any artifacts in the backbuffer
			window.swap_buffers();
			
			// Create gl
			let gl_context = Rc::clone(window.gl_context().unwrap());
			let mut gl_context = RefCell::borrow_mut(&gl_context);
			
			// Drop window RefMut
			drop(window);
			
			// Make context current
			gl_context.make_current();
			gl_context.set_swap_interval(SwapInterval::Sync(1));
		}
		
//		// Init glfw
//		let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("Failed to init glfw");
//		
//		// Create glfw window
//		let window_glfw = {
//			let (window_glfw, window_channel) = glfw.create_window(resolution.0, resolution.1, "Render Demo", WindowMode::Windowed).expect("Failed to create glfw window");
//			self.window = Some(window_glfw);
//			self.window_channel = Some(window_channel);
//			
//			self.window.need_mut()
//		};
//		
//		// Open window
//		window_glfw.show();
		
		{// Print opengl implementation info
			fn get_gl_string(token: gl::enuma) -> String {
				let raw_ptr = unsafe {gl::GetString(token)};
				String::from(unsafe {CStr::from_ptr(raw_ptr as *const std::os::raw::c_char)}.to_string_lossy())
			}
			
			let vendor = get_gl_string(gl::VENDOR);
			let renderer = get_gl_string(gl::RENDERER);
			let version = get_gl_string(gl::VERSION);
			let glsl_version = get_gl_string(gl::SHADING_LANGUAGE_VERSION);
			
			println!("GL Implementation Info:");
			println!("  VENDOR: \"{}\"", vendor);
			println!("  RENDERER: \"{}\"", renderer);
			println!("  VERSION: \"{}\"", version);
			println!("  GLSL_VERSION: \"{}\"", glsl_version);
		}
		
		// Setup gl debug output
		unsafe {
			gl::Enable(gl::DEBUG_OUTPUT);
			gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
			gl::DebugMessageCallback(Some(gl_debug_callback), std::ptr::null());
		};
		
		{// DEBUG: Test ssss kernel gen
			let mut kernel_gen = SubsurfaceKernelGenerator::new(21, 2.5, DEFAULT_HUMAN_SKIN_FALLOFF_FACTORS, DEFAULT_HUMAN_SKIN_STRENGTH_FACTORS);
			let ssss_kernel = kernel_gen.generate_kernel();
			
			println!("#define SSSS_KERNEL_RANGE {:.2}", ssss_kernel.kernel_range);
			println!("#define SSSS_KERNEL_NUM_SAMPLES {}", ssss_kernel.num_samples());
			println!("vec4 SSSS_KERNEL[] = {{");
			for i in 0..ssss_kernel.num_samples() { 
				let sample = ssss_kernel.as_slice()[i as usize];
				
				println!("	vec4({:.6}, {:.6}, {:.6}, {:.6}),", sample.x, sample.y, sample.z, sample.w);
			}
			println!("}};");
		}
		
		{// Load test model (lee head)
			// Log
			println!("Loading lee head model");
			
//			let mut file = OpenOptions::new().read(true).open(r"C:\Users\Jan\Desktop\Lee Head\Lee Head.ply").expect("Failed to load test lee head model");
			let mut file = OpenOptions::new().read(true).open(unsafe {&ASSET_MANAGER_INSTANCE}.resolve_asset_fs_path(&AssetPath::from_str("models/free_head/head.ply"))).expect("Failed to load test free head model");
			
			let loader = PlyMeshLoader::new(&mut file);
			let mut puller = loader.parse_header().unwrap();
			
			// Get vertex and index num
			let mut num_vertices = 0u32;
			let mut num_indices = 0u32;
			
			for e in &puller.header().elements {
				let name = e.name.as_str();
				if name.eq("vertex") {
					num_vertices = e.num_entries;
				}
				else if name.eq("face") {
					num_indices = e.num_entries * 3;
				}
			}
			
			// Allocate data buffers
			let mut vertex_data_buffer = vec![0u8; (num_vertices as usize) * 4*8];
			let mut index_data_buffer = vec![0u8; (num_indices as usize) * 4];
			
			loop {
//				let mut borrowed_puller = puller.borrow_mut();
				match puller.next_event() {
					PullEvent::Element(mut parser) => {
						let elem_name = parser.element_descriptor().name.as_str();
						if elem_name.eq("vertex") {
							let mut read_buffer = [0u8; 32];
							
							let mut vertex_data_pos = 0usize;
							'vertex_entry_loop: loop {
								let res = parser.read_entry(&mut read_buffer);
								
								if let Err(PlyReadError::BufferTooSmall {min_buffer_size}) = res {
									panic!("Buffer too small! (min {})", min_buffer_size);
								}
								else if let Err(PlyReadError::NoMoreEntries) = res {
									break 'vertex_entry_loop;
								}
								else if let Ok(_) = res {
									// Copy into vertex buffer
									let final_pos = vertex_data_pos + 4 * 8;
									vertex_data_buffer[vertex_data_pos..final_pos].copy_from_slice(&read_buffer);
									vertex_data_pos = final_pos;
								}
							}
						}
						else if elem_name.eq("face") {
							let mut read_buffer = [0u8; 1+3*4];
							
							let mut index_data_pos = 0usize;
							'index_entry_loop: loop {
								let res = parser.read_entry(&mut read_buffer);
								
								if let Err(PlyReadError::BufferTooSmall {min_buffer_size}) = res {
									panic!("Buffer too small! (min {})", min_buffer_size);
								}
								else if let Err(PlyReadError::NoMoreEntries) = res {
									break 'index_entry_loop;
								}
								else if let Ok(_) = res {
									// Copy into index buffer
									index_data_buffer[index_data_pos+0..index_data_pos+4].copy_from_slice(&read_buffer[1..5]);
									index_data_buffer[index_data_pos+4..index_data_pos+8].copy_from_slice(&read_buffer[5..9]);
									index_data_buffer[index_data_pos+8..index_data_pos+12].copy_from_slice(&read_buffer[9..13]);
									index_data_pos += 3*4;
								}
							}
						}
					}
					PullEvent::End => break,
				}
			}
			
			// Generate tangents
			let (index_data_buffer, vertex_data_buffer) = calculate_mesh_tangents(num_indices, index_data_buffer, num_vertices, vertex_data_buffer);
			
			// Allocate vertex buffers
			let (vertex_buffer_gl, index_buffer_gl) = unsafe {
				let mut buffers = [0 as gl::uint, 2];
				gl::CreateBuffers(2 as gl::sizei, buffers.as_mut_ptr());
				(buffers[0], buffers[1])
			};
			
			// Upload data
			unsafe {
				gl::NamedBufferStorage(vertex_buffer_gl, vertex_data_buffer.len() as gl::sizeiptr, vertex_data_buffer.as_ptr() as *const gl::void, 0);
				gl::NamedBufferStorage(index_buffer_gl, index_data_buffer.len() as gl::sizeiptr, index_data_buffer.as_ptr() as *const gl::void, 0);
			}
			
			// Load textures
			println!("Loading lee head textures");
			
//			let tex_albedo = Texture::load_png_from_path(Path::new(r"C:/Users/Jan/Desktop/Lee Head/lee_head_albedo.png"), ImageFormat::get(gl::SRGB8_ALPHA8)).expect("Failed to load albedo texture");
//			let tex_normal = Texture::load_png_from_path(Path::new(r"C:/Users/Jan/Desktop/Lee Head/lee_head_normal.png"), ImageFormat::get(gl::RGBA8)).expect("Failed to load normal texture");
//			let tex_transmission = Texture::load_png_from_path(Path::new(r"C:/Users/Jan/Desktop/Lee Head/lee_head_transmission.png"), ImageFormat::get(gl::RGBA8)).expect("Failed to load transmission texture");
			
			let tex_albedo = Texture::load_ktx_from_path(unsafe {&ASSET_MANAGER_INSTANCE}.resolve_asset_fs_path(&AssetPath::from_str("models/free_head/head_albedo_bc7.ktx")).as_path(), ImageFormat::get(gl::COMPRESSED_SRGB_ALPHA_BPTC_UNORM_ARB)).expect("Failed to load albedo texture");
			let tex_normal = Texture::load_ktx_from_path(unsafe {&ASSET_MANAGER_INSTANCE}.resolve_asset_fs_path(&AssetPath::from_str("models/free_head/head_normal_bc1.ktx")).as_path(), ImageFormat::get(gl::COMPRESSED_RGB_S3TC_DXT1_EXT)).expect("Failed to load normal texture");
//			let tex_albedo = Texture::load_png_from_path(unsafe {&ASSET_MANAGER_INSTANCE}.resolve_asset_fs_path(&AssetPath::from_str("models/free_head/head_albedo.png")).as_path(), ImageFormat::get(gl::SRGB8_ALPHA8)).expect("Failed to load albedo texture");
//			let tex_normal = Texture::load_png_from_path(unsafe {&ASSET_MANAGER_INSTANCE}.resolve_asset_fs_path(&AssetPath::from_str("models/free_head/head_normal.png")).as_path(), ImageFormat::get(gl::RGBA8)).expect("Failed to load normal texture");
			let tex_transmission = Texture::load_png_from_path(unsafe {&ASSET_MANAGER_INSTANCE}.resolve_asset_fs_path(&AssetPath::from_str("models/free_head/head_translucency.png")).as_path(), ImageFormat::get(gl::RGBA8)).expect("Failed to load transmission texture");
			
//			let tex_albedo = Texture::new(16, 16, 1, ImageFormat::get(gl::RGBA8));
//			let tex_normal = Texture::new(16, 16, 1, ImageFormat::get(gl::RGBA8));
//			let tex_transmission = Texture::new(16, 16, 1, ImageFormat::get(gl::RGBA8));
			
			self.test_head_model = Some(TestHeadModel {
				vertex_buffer_gl,
				index_buffer_gl,
				num_indices,
				tex_albedo,
				tex_normal,
				tex_transmission,
			});
		}
		
		// Create test teapot vbo
		self.test_teapot_vbo = Some({
			let mut buffer = TestVertexBuffer::new();
			buffer.allocate(&crate::render::teapot::TEAPOT_VERTEX_DATA.as_byte_slice());
			buffer
		});
		
		// Create main camera
		self.test_active_camera = Some(Arc::new(Mutex::new({
			let fovy = fovx_to_fovy(Rad::from(Deg(65.0)), resolution.0 as f32 /resolution.1 as f32);
			let projection = PerspectiveProjection::new(fovy, 1.0/256.0, 4096.0, true, true);
			let mut cam = Camera::new(Box::new(projection));
			cam.translation = Vector3 {x: 0.0, y: 1.0, z: 4.0};
			cam
		})));
		
		// Initialize render global
		self.render_global.initialize(resolution).expect("Failed to init render global");
		
		// Main loop
		'main_loop: loop {
			{// Update window and poll messages
				let mut window_borrow = self.main_window.need().borrow_mut();
				
				// Poll window events
				for (_, event) in window_borrow.poll_messages() {
					match event {
						WindowEvent::Scroll(_scroll_x, scroll_y) => {
							self.test_camera_orbit.distance = f32::min(f32::max(self.test_camera_orbit.distance - ((scroll_y as f32 * 0.1) * self.test_camera_orbit.distance), 2.0), 16.0);
						}
						WindowEvent::Key(key, _scancode, action, _) => {
							if key == glfw::Key::R && action == glfw::Action::Press {
								// Reload shaders
								self.render_global.queue_shader_reload();
							}
						}
						_ => {},
					}
				}
				
				// Check if window should close
				if window_borrow.should_close() {
					break 'main_loop;
				}
				
				// Ensure window RefMut is dropped
				drop(window_borrow);
			}
			
			// Tick frame
			self.do_tick_frame();
			
			// Render frame
			self.render_global.do_render_frame();
			
			{// Swap buffers
				let mut window_borrow = self.main_window.need().borrow_mut();
				window_borrow.swap_buffers();
				
				// Ensure window RefMut is dropped
				drop(window_borrow);
			}
		}
		
//		// Close window
//		self.window.take().unwrap().close();
	}
	
	pub fn do_tick_frame(&mut self) {
		{// Tick demo camera carousel
			let carousel = &mut self.test_camera_carousel_state;
			
			let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
			if carousel.last_tick_time == 0.0 {
				carousel.last_tick_time = current_time;
			}
			
			// Get tick delta time, clamped from 0 to 1
			let delta_time = f32::min(f32::max((current_time - carousel.last_tick_time) as f32, 0.0), 1.0);
			
			let mouse_pos = self.main_window.need().borrow().get_cursor_pos();
			
			let button_state = self.main_window.need().borrow().get_mouse_button(glfw::MouseButtonLeft);
			if button_state {
				// Deaccelerate spin
				carousel.current_spin_speed = f32::max(carousel.current_spin_speed - carousel.spin_deacceleration_per_sec * delta_time, 0.0);
				
				// Reset override timeout
				carousel.override_timeout_left = carousel.override_timeout_length_secs;
				
				// Spin camera by mouse delta
				let mouse_delta = (mouse_pos.0 - carousel.last_mouse_pos.0) as f32;
				
				// Apply drag spin
				self.test_camera_orbit.angles.y += Deg(mouse_delta * carousel.drag_spin_sensitivity).into();
				
				// Vertical movement
				self.test_camera_orbit.center.y += (mouse_pos.1 - carousel.last_mouse_pos.1) as f32 * 0.001;
			}
			else {
				if carousel.current_spin_speed > 0.0 {
					carousel.override_timeout_left = 0.0;
				}
				
				if carousel.override_timeout_left > 0.0 {
					carousel.override_timeout_left = f32::max(0.0, carousel.override_timeout_left - delta_time);
				}
				else {
					carousel.current_spin_speed = f32::min(carousel.current_spin_speed + carousel.spin_acceleration_per_sec * delta_time, carousel.max_spin_speed);
				}
			}
			
			// Set last time and mouse pos
			carousel.last_tick_time = current_time;
			carousel.last_mouse_pos = mouse_pos;
			
			// Actually rotate camera orbit
			self.test_camera_orbit.angles.y += Deg(carousel.current_spin_speed).into();
		}
		
		{// Update cam
			let mut cam = self.test_active_camera.need().lock().unwrap();
			
			// Update camera viewport size
			let window = self.main_window.need().borrow();
			let window_size = window.get_size();
			cam.viewport_size = (window_size.0 as u32, window_size.1 as u32);
			
			// Update camera transform
			let orbit = &self.test_camera_orbit;
//			orbit.center = vec3(0.0, 0.5, 0.0);
//			orbit.center = vec3(0.0, 1.75, 0.0);
			
			let rotation = Quaternion::<f32>::from(orbit.angles);
			cam.rotation = rotation.clone().invert();
			cam.translation = orbit.center + (&rotation * vec3::<f32>(0.0, 0.0, -1.0) * -orbit.distance);
		}
	}
	
	pub fn get_test_camera(&mut self) -> sync::Weak<Mutex<Camera>> {
		match &self.test_active_camera {
			Some(cam) => Arc::downgrade(cam),
			None => sync::Weak::new(),
		}
	}
}

pub extern "system" fn gl_debug_callback(_source: gl::enuma, _message_type: gl::enuma, _id: gl::uint, _severity: gl::enuma, length: gl::sizei, raw_message: *const gl::char, _user_param: *mut gl::void) {
	// Print message
	let message = unsafe {String::from_raw_parts(raw_message as *mut u8, length as usize, length as usize)};
	println!("[GL DEBUG] {}", message);
	
	// Since the string isn't actually ours this prevents
	// heap corruption when rust tries to free the String's data
	std::mem::forget(message);
}

pub struct TestHeadModel {
	pub vertex_buffer_gl: gl::uint,
	pub index_buffer_gl: gl::uint,
	pub num_indices: u32,
	
	pub tex_albedo: Texture,
	pub tex_normal: Texture,
	pub tex_transmission: Texture,
//	pub tex_roughness: Texture,
}

pub fn calculate_mesh_tangents(num_indices: u32, index_data: Vec<u8>, num_vertices: u32, vertex_data: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
	// https://www.marti.works/calculating-tangents-for-your-mesh/
	
	let mut tangent_list = vec![vec3(0.0, 0.0, 0.0); num_vertices as usize];
	let mut bitangent_list = tangent_list.clone();
	
	fn read_vec3_f32(buffer: &[u8]) -> Vector3<f32> {
		vec3(LittleEndian::read_f32(&buffer[0..4]), LittleEndian::read_f32(&buffer[4..8]), LittleEndian::read_f32(&buffer[8..12]))
	}
	fn read_vec2_f32(buffer: &[u8]) -> Vector2<f32> {
		vec2(LittleEndian::read_f32(&buffer[0..4]), LittleEndian::read_f32(&buffer[4..8]))
	}
	
	for i in (0..num_indices as usize).step_by(3) {
		let index_base = i*4;
		let index0 = LittleEndian::read_u32(&index_data[(index_base)..(index_base+4)]) as usize;
		let index1 = LittleEndian::read_u32(&index_data[(index_base+4)..(index_base+8)]) as usize;
		let index2 = LittleEndian::read_u32(&index_data[(index_base+8)..(index_base+12)]) as usize;
		
		let vertex0 = read_vec3_f32(&vertex_data[(index0*32)..(index0*32+12)]);
		let vertex1 = read_vec3_f32(&vertex_data[(index1*32)..(index1*32+12)]);
		let vertex2 = read_vec3_f32(&vertex_data[(index2*32)..(index2*32+12)]);
		
		let uv0 = read_vec2_f32(&vertex_data[(index0*32+24)..(index0*32+32)]);
		let uv1 = read_vec2_f32(&vertex_data[(index1*32+24)..(index1*32+32)]);
		let uv2 = read_vec2_f32(&vertex_data[(index2*32+24)..(index2*32+32)]);
		
		let edge1: Vector3<f32> = vertex1 - vertex0;
		let edge2: Vector3<f32> = vertex2 - vertex0;
		
		let uv_edge1: Vector2<f32> = uv1 - uv0;
		let uv_edge2: Vector2<f32> = uv2 - uv0;
		
		let r = 1.0 / (uv_edge1.x * uv_edge2.y - uv_edge1.y * uv_edge2.x);
		
		let tangent = vec3(
			((edge1.x * uv_edge2.y) - (edge2.x * uv_edge1.y)) * r,
			((edge1.y * uv_edge2.y) - (edge2.y * uv_edge1.y)) * r,
			((edge1.z * uv_edge2.y) - (edge2.z * uv_edge1.y)) * r
		);
		let bitangent = vec3(
			((edge1.x * uv_edge2.x) - (edge2.x * uv_edge1.x)) * r,
			((edge1.y * uv_edge2.x) - (edge2.y * uv_edge1.x)) * r,
			((edge1.z * uv_edge2.x) - (edge2.z * uv_edge1.x)) * r
		);
		
		tangent_list[index0] += tangent;
		tangent_list[index1] += tangent;
		tangent_list[index2] += tangent;
		
		bitangent_list[index0] += bitangent;
		bitangent_list[index1] += bitangent;
		bitangent_list[index2] += bitangent;
	}
	
	let mut new_vertex_data = vec![0; num_vertices as usize * 48];
	
	for i in 0..num_vertices as usize {
		let n = read_vec3_f32(&vertex_data[(i*32+12)..(i*32+24)]);
		let t0 = tangent_list[i];
		let t1 = bitangent_list[i];
		
		let t = Vector3::normalize(t0 - (n * Vector3::dot(n, t0)));
		
		let c = Vector3::cross(n, t0);
		
		// Calculate handedness: Needed for calculating the binormal in the right direction
		let w = if Vector3::dot(c, t1) < 0.0 {-1.0} else {1.0};
		
		let final_tangent = t.extend(w);
		
		new_vertex_data[i*48..i*48+32].copy_from_slice(&vertex_data[i*32..i*32+32]);
		
		let mut tangent_buffer = [0u8; 16];
		LittleEndian::write_f32(&mut tangent_buffer[0..4], final_tangent.x);
		LittleEndian::write_f32(&mut tangent_buffer[4..8], final_tangent.y);
		LittleEndian::write_f32(&mut tangent_buffer[8..12], final_tangent.z);
		LittleEndian::write_f32(&mut tangent_buffer[12..16], final_tangent.w);
		
		new_vertex_data[i*48+32..i*48+48].copy_from_slice(&tangent_buffer);
	}
	
	(index_data, new_vertex_data)
}

/*
#[allow(unused)]
fn test_model_load(asset_folder: &Path) {
	use fbxcel::pull_parser::any::AnyParser;
	use fbxcel::pull_parser::v7400::{Event};
	
	let model_path = asset_folder.join(Path::new("models/9mm.fbx"));
	let model_file = File::open(model_path).unwrap();
	
	let parser = fbxcel::pull_parser::any::from_seekable_reader(model_file).unwrap();
	let mut parser = if let AnyParser::V7400(p) = parser {p}
	else {panic!()};
	
	println!("-- Start model dump --");
	loop {
		match parser.next_event().unwrap() {
			Event::StartNode(node) => {
				let a = node.name().to_string();
				println!("{}{}", "  ".repeat(parser.current_depth()), a);
			},
			Event::EndNode => {},
			Event::EndFbx(_footer) => break,
		}
	}
	println!("-- End model dump --");
}
*/

pub struct DemoCameraCarouselState {
	pub spin_acceleration_per_sec: f32,
	pub spin_deacceleration_per_sec: f32,
	pub max_spin_speed: f32,
	pub current_spin_speed: f32,
	pub override_timeout_length_secs: f32,
	pub override_timeout_left: f32,
	
	pub last_tick_time: f64,
	pub last_mouse_pos: (f64, f64),
	pub drag_spin_sensitivity: f32,
}

impl DemoCameraCarouselState {
	pub fn new() -> DemoCameraCarouselState {
		DemoCameraCarouselState {
			spin_acceleration_per_sec: 0.15,
			spin_deacceleration_per_sec: 0.8,
			//max_spin_speed: 0.15,
			max_spin_speed: 0.0,
			current_spin_speed: 0.15,
			override_timeout_length_secs: 0.5,
			override_timeout_left: 0.0,
			
			last_tick_time: 0.0,
			last_mouse_pos: (0.0, 0.0),
			drag_spin_sensitivity: 0.1,
		}
	}
}
