use std::mem;

//#[deprecated]
pub enum Lazy<T> {
	Later,
	Now(T),
}

impl<T> Lazy<T> {
	pub fn get(&self) -> Option<&T> {
		if let Lazy::Now(d) = self {
			Some(d)
		}
		else {
			None
		}
	}
	
	pub fn get_mut(&mut self) -> Option<&mut T> {
		if let Lazy::Now(d) = self {
			Some(d)
		}
		else {
			None
		}
	}
	
	pub fn need(&self) -> &T {
		self.get().unwrap()
	}
	
	pub fn need_mut(&mut self) -> &mut T {
		self.get_mut().unwrap()
	}
	
	pub fn take(&mut self) -> Option<T> {
		if let Lazy::Now(a) = mem::replace(self, Self::default()) {
			Some(a)
		}
		else {
			None
		}
	}
}

impl<T> Default for Lazy<T> {
	fn default() -> Self {
		Lazy::Later
	}
}
