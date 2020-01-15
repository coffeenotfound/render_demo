
pub trait Lazy {
	type Type;
	
	fn need(&self) -> &Self::Type;
	fn need_mut(&mut self) -> &mut Self::Type;
}

impl<T> Lazy for Option<T> {
	type Type = T;
	
	fn need(&self) -> &Self::Type {
		self.as_ref().unwrap()
	}
	
	fn need_mut(&mut self) -> &mut Self::Type {
		self.as_mut().unwrap()
	}
}
