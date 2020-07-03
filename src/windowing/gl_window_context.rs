use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{self};
use gl_bindings::gl;
use crate::utils::lazy_option::Lazy;
use crate::windowing::Window;

pub struct GLWindowContext {
	glfw_window: sync::Weak<parking_lot::Mutex<Window>>,
}

impl GLWindowContext {
	pub fn set_swap_interval(&mut self, interval: glfw::SwapInterval) -> bool {
//		if self.is_current() {
		if let Some(window) = self.glfw_window.upgrade() {
			let window_borrow = window.lock();
			let mut glfw_context = window_borrow.glfw_context.borrow_mut();
			let glfw = glfw_context.glfw_mut();
			glfw.set_swap_interval(interval);
			true
		} else {
			false
		}
	}
	
	pub fn make_current(&mut self) -> bool {
		if let Some(strong_ref) = self.glfw_window.upgrade() {
			let mut window = strong_ref.lock();
			
			// Borrow
			let glfw_context = Rc::clone(&window.glfw_context);
			let mut glfw_context = RefCell::borrow_mut(&glfw_context);
			let glfw_window = window.window_glfw.need_mut();
			
			// Make context current
			glfw_context.glfw_mut().make_context_current(Some(glfw_window));
//			drop(glfw_context);
			
			// Init gl
			gl::load_with(|s| glfw_window.get_proc_address(s) as *const _);
			
			true
		}
		else { // Window does not exist anymore
			false
		}
	}
	
//	pub fn is_current(&self) -> bool {
//		
//	}
	
	pub(super) fn from_window(window_ref: sync::Weak<parking_lot::Mutex<Window>>) -> Self {
		Self {
			glfw_window: window_ref,
		}
	}
}
