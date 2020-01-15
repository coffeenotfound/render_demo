
pub trait IntoBoxed {
	type Boxed: ?Sized;
	fn into_boxed(self) -> Box<Self::Boxed>;
}

/*
impl<T: ?Sized> IntoBoxed for Box<T> {
	type Boxed = T;
	
	fn into_boxed(self) -> Box<Self::Boxed> {
		self
	}
}

impl<T: ?Sized> IntoBoxed for T {
	type Boxed = T;
	
	fn into_boxed(self) -> Box<Self::Boxed> {
		Box::new(self)
	}
}
*/
