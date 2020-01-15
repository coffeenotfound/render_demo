use cgmath::{Matrix4, Rad, SquareMatrix, Zero};

pub trait CameraProjection {
	fn projection_matrix(&self, viewport_size: (u32, u32)) -> Matrix4<f32>;
}

pub struct PerspectiveProjection {
	pub fovy: Rad<f32>,
	pub near_z: f32,
	pub far_z: f32,
	pub depth_zero_to_one: bool,
	pub inverse_depth: bool,
}

impl PerspectiveProjection {
	pub fn new(fovy: Rad<f32>, near_z: f32, far_z: f32, depth_zero_to_one: bool, inverse_depth: bool) -> PerspectiveProjection {
		PerspectiveProjection {
			fovy,
			near_z,
			far_z,
			depth_zero_to_one,
			inverse_depth,
		}
	}
}

impl CameraProjection for PerspectiveProjection {
	/// From https://github.com/JOML-CI/JOML/blob/master/src/org/joml/Matrix4f.java#L9933
	#[allow(unused_parens)]
	fn projection_matrix(&self, viewport_size: (u32, u32)) -> Matrix4<f32> {
		let aspect = viewport_size.0 as f32 / viewport_size.1 as f32;
		let z_near = if self.inverse_depth {self.far_z} else {self.near_z};
		let z_far = if self.inverse_depth {self.near_z} else {self.far_z};
		
		let mut mat = Matrix4::from_value(0.0);
		
		let h = f32::tan(self.fovy.0 * 0.5);
		mat[0][0] = 1.0 / (h * aspect);
		mat[1][1] = 1.0 / h;
		
		let far_inf = (z_far > 0.0 && f32::is_infinite(z_far));
		let near_inf = (z_near > 0.0 && f32::is_infinite(z_near));
		
		if far_inf {
			// See: "Infinite Projection Matrix" (http://www.terathon.com/gdc07_lengyel.pdf)
			let e = 1e-6f32;
			mat[2][2] = e - 1.0;
			mat[3][2] = (e - if self.depth_zero_to_one {1.0} else {2.0}) * z_near;
		}
		else if near_inf {
			let e = 1e-6f32;
			mat[2][2] = if self.depth_zero_to_one {0.0} else {1.0} - e;
			mat[3][2] = (if self.depth_zero_to_one {1.0} else {2.0} - e) * z_far;
		}
		else {
			mat[2][2] = if self.depth_zero_to_one {z_far} else {z_far + z_near} / (z_near - z_far);
			mat[3][2] = if self.depth_zero_to_one {z_far} else {z_far + z_far} * z_near / (z_near - z_far);
		}
		mat[2][3] = -1.0;
		
		// Inverse depth
		if self.inverse_depth {
			let mut depth_reversal_matrix = Matrix4::<f32>::zero();
			depth_reversal_matrix[0][0] = 1.0;
			depth_reversal_matrix[1][1] = 1.0;
			depth_reversal_matrix[2][2] = -1.0;
			depth_reversal_matrix[3][2] = 1.0;
			depth_reversal_matrix[3][3] = 1.0;

			mat = mat * depth_reversal_matrix;
		}
		mat
	}
}
