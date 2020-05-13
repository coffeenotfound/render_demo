use glfw::{WindowMode, WindowHint, FlushedMessages, WindowEvent, Context, MouseButton};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::windowing::{GlfwContext, GLWindowContext};
use std::sync::mpsc::Receiver;
use crate::utils::bool_cmpxchg::BoolCompareExchange;
use crate::utils::lazy_option::Lazy;
use glfw::ffi::{glfwGetPrimaryMonitor, glfwGetVideoMode};

pub struct Window {
	pub(in super) glfw_context: Rc<RefCell<GlfwContext>>,
	self_reference: Weak<RefCell<Self>>,
	
	visible: bool,
	width: u32,
	height: u32,
	title: String,
	
	visibility_dirty: bool,
	size_dirty: bool,
	title_dirty: bool,
	
	pub(in super) window_glfw: Option<glfw::Window>,
	message_channel: Option<Receiver<(f64, glfw::WindowEvent)>>,
	
	gl_context: Option<Rc<RefCell<GLWindowContext>>>,
}

impl Window {
	pub fn init(&mut self) -> bool {
		if self.window_glfw.is_none() {
			// Borrow glfw context
			let mut context = RefCell::borrow_mut(&mut self.glfw_context);
			let glfw = context.glfw_mut();
			
			// Set glfw window hints
			glfw.default_window_hints();
			glfw.window_hint(WindowHint::Resizable(false));
			
			// Create glfw handle
			let (window, channel) = glfw.create_window(self.width, self.height, &self.title, WindowMode::Windowed).expect("Failed to init glfw window");
			self.window_glfw = Some(window);
			self.message_channel = Some(channel);
			let window_glfw = self.window_glfw.need_mut();
			
			// Enable key and mouse input callbacks
			window_glfw.set_key_polling(true);
			window_glfw.set_scroll_polling(true);
			
			// Set initial window color to avoid artifacts (only on windows for now)
			if cfg!(windows) {
				set_initial_background_color_win32(&window_glfw, 0x000000);
			}
			
			// Create the gl context object
			self.gl_context = Some(Rc::new(RefCell::new(GLWindowContext::from_window(Weak::clone(&self.self_reference)))));
			
			true
		}
		else {
			false
		}
	}
	
	pub fn is_initialized(&self) -> bool {
		self.window_glfw.is_some()
	}
	
	pub fn make_visible(&mut self, visible: bool) {
		if self.is_initialized() {
			// Set flag and mark dirty
			self.visible = visible;
			self.visibility_dirty = true;
		}
	}
	
	pub fn resize(&mut self, width: u32, height: u32) {
		self.width = width;
		self.height = height;
		self.size_dirty = true;
	}
	
	pub fn get_size(&self) -> (u32, u32) {
		(self.width, self.height)
	}
	
	pub fn set_title(&mut self, title: String) {
		self.title = title;
		self.title_dirty = true;
	}
	
	pub fn should_close(&self) -> bool {
		self.window_glfw.as_ref().map_or(false, |w| w.should_close())
	}
	
	pub fn get_cursor_pos(&self) -> (f64, f64) {
		self.window_glfw.need().get_cursor_pos()
	}
	
	pub fn get_mouse_button(&self, button: MouseButton) -> bool { 
		(self.window_glfw.need().get_mouse_button(button) == glfw::Action::Press)
	}
	
	pub fn gl_context(&self) -> Option<&Rc<RefCell<GLWindowContext>>> {
		self.gl_context.as_ref()
	}
	
	pub fn swap_buffers(&mut self) {
		if let Some(window) = &mut self.window_glfw {
			window.swap_buffers();
		}
	}
	
	/// This is a super bad abstraction because it does not
	/// properly account for multiple monitors but it's
	/// fine for our test porpuses right now.
	/// 
	/// Currently only works while the window is actually created.
	/// Yeah it's bad. Completely redo this in future.
	pub fn center_on_screen(&mut self) {
		if let Some(window) = &mut self.window_glfw {
			unsafe {
				// Get the primary monitor
				let primary_monitor = glfwGetPrimaryMonitor();
				
				// Get the current video mode of the primary monitor
				let video_mode = glfwGetVideoMode(primary_monitor);
				
				// Move our window
				let x_pos = ((*video_mode).width - self.width as i32) / 2i32;
				let y_pos = ((*video_mode).height - self.height as i32) / 2i32;
				window.set_pos(x_pos, y_pos);
			}
		}
	}
	
	pub fn update(&mut self) {
		if let Some(window) = &mut self.window_glfw {
			if self.size_dirty.compare_exchange(true, false) {
				window.set_size(self.width as i32, self.height as i32);
			}
			if self.title_dirty.compare_exchange(true, false) {
				window.set_title(&self.title);
			}
			if self.visibility_dirty.compare_exchange(true, false) {
				if self.visible {
					window.show();
				}
				else {
					window.hide();
				}
			}
		}
	}
	
	pub fn poll_messages(&mut self) -> FlushedMessages<(f64, WindowEvent)> {
		// Update window state
		self.update();
		
		{// Poll glfw messages
			let mut context_borrow = RefCell::borrow_mut(&mut self.glfw_context);
			context_borrow.glfw_mut().poll_events();
		}
		
		// Flush messages out of queue
		glfw::flush_messages(self.message_channel.need())
	}
	
	pub fn new(glfw_context: Rc<RefCell<GlfwContext>>) -> Rc<RefCell<Self>> {
		let shared_ref = Rc::new(RefCell::new(Self {
			glfw_context,
			self_reference: Weak::new(), // Empty for now, replaced soon
			
			visible: false,
			width: 256,
			height: 256,
			title: String::from("Window"),
			
			visibility_dirty: true,
			size_dirty: true,
			title_dirty: true,
			
			window_glfw: None,
			message_channel: None,
			
			gl_context: None,
		}));
		
		// Store our (weak) self reference
		let weak_ref = Rc::downgrade(&shared_ref);
		shared_ref.borrow_mut().self_reference = weak_ref;
		
		// Finally, return as shared ref
		shared_ref
	}
}

#[cfg(windows)]
fn set_initial_background_color_win32(window_glfw: &glfw::Window, color_bgr: u32) {
	use winapi::shared::windef;
	use winapi::um::winuser;
	use winapi::um::wingdi;
	
	// Get glfw window hwnd
	let hwnd = window_glfw.get_win32_window() as windef::HWND;
	
	// Set initial window background color (by setting win32 window background brush)
	unsafe {
		let black_background_brush = wingdi::CreateSolidBrush(color_bgr as windef::COLORREF);
		winuser::SetClassLongPtrA(hwnd, winuser::GCLP_HBRBACKGROUND, black_background_brush as winapi::shared::basetsd::LONG_PTR);
	}
}
