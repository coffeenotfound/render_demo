use cgmath::{Vector3, Quaternion, One, Zero};
use crate::camera::CameraProjection;

pub struct Camera {
	pub translation: Vector3<f32>,
	pub rotation: Quaternion<f32>,
	pub viewport_size: (u32, u32),
	pub projection: Box<dyn CameraProjection>,
}

impl Camera {
	pub fn resize_viewport(&mut self, new_size: (u32, u32)) {
		self.viewport_size = new_size;
	}
	
	pub fn new/*<P>*/(projection: Box<dyn CameraProjection>) -> Camera /*where P: IntoBoxed<Boxed = dyn CameraProjection>*/ {
		Camera {
			translation: Vector3::zero(),
			rotation: Quaternion::one(),
			viewport_size: (0, 0),
			projection,
		}
	}
}
