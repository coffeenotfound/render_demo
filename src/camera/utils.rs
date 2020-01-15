use cgmath::{Angle, BaseFloat};

pub fn fovx_to_fovy<N, R>(fovx: R, aspect: N) -> R where N: BaseFloat, f32: Into<N>, R: Angle<Unitless = N> {
	R::atan(R::tan(fovx * 0.5.into()) * (1.0.into() / aspect)) * 2.0.into()
}

pub fn fovy_to_fovx<N, R>(fovy: R, aspect: N) -> R where N: BaseFloat, f32: Into<N>, R: Angle<Unitless = N> {
	R::atan(R::tan(fovy * 0.5.into()) * aspect) * 2.0.into()
}
