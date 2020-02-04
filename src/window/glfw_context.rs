use glfw;
use std::sync::atomic::{AtomicBool, Ordering};
use std::error;
use std::fmt::{self, Debug, Display};
use glfw::Glfw;

static CONTEXT_ALREADY_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub struct GlfwContext {
	glfw_handle: Glfw
}

impl GlfwContext {
	pub fn glfw(&self) -> &Glfw {
		&self.glfw_handle
	}
	
	pub fn glfw_mut(&mut self) -> &mut Glfw {
		&mut self.glfw_handle
	}
	
	pub fn init() -> Result<GlfwContext, ContextInitError> {
		// Check if already globally initialized
		if let Ok(_) = CONTEXT_ALREADY_INITIALIZED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
			// Init glfw
			let glfw = glfw::init::<()>(None).map_err(|e| ContextInitError::GlfwInitError(e))?;
			
			// Make context and return
			Ok(GlfwContext {
				glfw_handle: glfw
			})
		}
		else {
			// Already initialized before, return error
			Err(ContextInitError::AlreadyInitialized)
		}
	}
}

pub enum ContextInitError {
	AlreadyInitialized,
	GlfwInitError(glfw::InitError)
}

impl GenericError for ContextInitError {
	fn fmt_error(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		match self {
			ContextInitError::AlreadyInitialized => write!(f, "AlreadyInitialized"),
			ContextInitError::GlfwInitError(err) => {
				write!(f, "GlfwInitError: ")?;
				Display::fmt(err, f)
			},
		}
	}
}

const DEFAULT_MESSAGE: &'static str = "";

pub trait GenericError: error::Error + Display + Debug {
	const MESSAGE: &'static str = DEFAULT_MESSAGE;
	
	fn fmt_error(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		write!(f, "{}", Self::MESSAGE)
	}
	
	fn fmt_error_debug(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		self.fmt_error(f)
	}
}

impl error::Error for ContextInitError {

}

impl Display for ContextInitError  {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		<Self as GenericError>::fmt_error(self, f)
	}
}

impl Debug for ContextInitError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		<Self as GenericError>::fmt_error_debug(self, f)
	}
}

pub enum Formatting<'s, E> where E: 's + GenericError {
	Static(&'s str),
	Dynamic(&'s fn(&'_ E, &'_ mut fmt::Formatter<'_>) -> Result<(), fmt::Error>)
}

//impl<T> error::Error for T where T: GenericError {
//	
//}
//
//impl<T> Display for T where T: GenericError {
//	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
//		<Self as GenericError>::fmt_error(self, f)
//	}
//}
//
//impl<T> Debug for T where T: GenericError {
//	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
//		<Self as GenericError>::fmt_error_debug(self, f)
//	}
//}
