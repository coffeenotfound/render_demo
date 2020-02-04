
pub trait OptionOverwrite<T> {
	fn overwrite(&mut self, value: T) -> &mut T;
}

impl<T> OptionOverwrite<T> for Option<T> {
	fn overwrite(&mut self, value: T) -> &mut T {
		*self = Some(value);
		self.as_mut().unwrap()
	}
}
