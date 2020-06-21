use std::mem::{self, MaybeUninit};
use std::cell::UnsafeCell;
use std::ops::DerefMut;
use parking_lot::Mutex;

pub struct UnsafeLazy<T> {
	init_flag: Mutex<bool>,
	data: UnsafeCell<MaybeUninit<T>>,
}

impl<T> UnsafeLazy<T> {
	pub fn empty() -> Self {
		Self {
			init_flag: Mutex::new(false),
			data: UnsafeCell::new(MaybeUninit::zeroed()),
		}
	}
	
	pub fn new(data: T) -> Self {
		Self {
			init_flag: Mutex::new(true),
			data: UnsafeCell::new(MaybeUninit::new(T)),
		}
	}
	
	pub unsafe fn get(&self) -> &T {
		&*self.data.get()
	}
	
	pub fn get_safe(&self) -> Option<&T> {
		// SAFETY: Unwrap won't panic because the Mutex is
		// only used by us and can't get poisoned
		let guard = self.init_flag.lock().unwrap();
		
		match *guard {
			// SAFETY: Always safe since at this point we own
			// the mutex which guards access to the UnsafeCell
			true => unsafe {
				Some(self.get())
			}
			false => None
		}
	}
	
	pub fn set(&self, data: T) -> bool {
		// SAFETY: Unwrap won't panic because the Mutex is
		// only used by us and can't get poisoned
		let mut guard = self.init_flag.lock().unwrap();
		
		let update = !mem::replace(guard.deref_mut(), true);
		if update {
			unsafe {
				*self.data.get() = data;
			}
		}
		
		mem::drop(guard);
		update
	}
}

impl<T> Drop for UnsafeLazy<T> {
	fn drop(&mut self) {
		// Check if value is initialized
		if self.init_flag.get_mut() {
			// Get the data by swapping it out
			// SAFETY: Accessing the UnsafeCell is safe because
			// we own it and made sure the value is initialized
			let mut data = MaybeUninit::<T>::uninit();
			mem::swap(unsafe {&mut *self.data.get()}, &mut data);
			
			// Turn the MaybeUninit into a value and drop it
			// SAFETY: Safe because at this point the value is guarenteed to be valid
			mem::drop(unsafe {data.assume_init()});
		}
	}
}