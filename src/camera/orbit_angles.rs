use cgmath::{Vector3, BaseFloat, Euler, Angle, Rad, Zero};
use num_traits::FromPrimitive;

pub struct OrbitAngles<N = f32, R = Rad<N>> where N: BaseFloat + FromPrimitive, R: Angle<Unitless = N> {
	pub up_vector: Vector3<N>,
	pub forward_vector: Vector3<N>,
	pub center: Vector3<N>,
	pub angles: Euler<R>,
	pub distance: N,
}

impl<N, R> OrbitAngles<N, R> where N: BaseFloat + FromPrimitive, R: Angle<Unitless = N> {
	pub fn new_zero(up_vector: Vector3<N>, forward_vector: Vector3<N>) -> OrbitAngles<N, R> {
		OrbitAngles {
			up_vector,
			forward_vector,
			center: Vector3::zero(),
			angles: Euler::new(R::zero(), R::zero(), R::zero()),
			distance: N::from_f32(0.0).unwrap(),
		}
	}
	
//	/// Calculates and returns a non translated view matrix
//	/// that represents this orbit's transform.
//	pub fn view_matrix(&self) -> Matrix4<N> {
//		
//	}
}
