use cgmath::{Matrix4, Rad, SquareMatrix};

/// From https://github.com/JOML-CI/JOML/blob/master/src/org/joml/Matrix4f.java#L9933
#[allow(unused_parens)]
#[deprecated]
pub fn make_perspective_mat4(fovy: Rad<f32>, aspect: f32, z_near: f32, z_far: f32, zero_to_one: bool) -> Matrix4<f32> {
	let mut mat = Matrix4::from_value(0.0);
	
	let h = f32::tan(fovy.0 * 0.5);
	mat[0][0] = 1.0 / (h * aspect);
	mat[1][1] = 1.0 / h;
	
	let far_inf = (z_far > 0.0 && f32::is_infinite(z_far));
	let near_inf = (z_near > 0.0 && f32::is_infinite(z_near));
	
	if far_inf {
		// See: "Infinite Projection Matrix" (http://www.terathon.com/gdc07_lengyel.pdf)
		let e = 1e-6f32;
		mat[2][2] = e - 1.0;
		mat[3][2] = (e - if zero_to_one {1.0} else {2.0}) * z_near;
	}
	else if near_inf {
		let e = 1e-6f32;
		mat[2][2] = if zero_to_one {0.0} else {1.0} - e;
		mat[3][2] = (if zero_to_one {1.0} else {2.0} - e) * z_far;
	}
	else {
		mat[2][2] = if zero_to_one {z_far} else {z_far + z_near} / (z_near - z_far);
		mat[3][2] = if zero_to_one {z_far} else {z_far + z_far} * z_near / (z_near - z_far);
	}
	mat[2][3] = -1.0;
	mat
}
