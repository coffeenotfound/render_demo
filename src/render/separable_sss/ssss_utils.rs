use cgmath::{Vector3, Vector4, ElementWise, vec3};

pub const DEFAULT_HUMAN_SKIN_FALLOFF_FACTORS: SubsurfaceFalloffFactors = SubsurfaceFalloffFactors::new(1.0, 0.37, 0.30);
pub const DEFAULT_HUMAN_SKIN_STRENGTH_FACTORS: SubsurfaceStrengthFactors = SubsurfaceStrengthFactors::new(0.48, 0.41, 0.28);

//#[derive(Clone, Copy)]
pub struct SubsurfaceFalloffFactors {
	pub rgb: Vector3<f32>,
}

impl SubsurfaceFalloffFactors {
	pub const fn new(r: f32, g: f32, b: f32) -> SubsurfaceFalloffFactors {
		SubsurfaceFalloffFactors {
			rgb: vec3(r, g, b)
		}
	}
}

//#[derive(Clone, Copy)]
pub struct SubsurfaceStrengthFactors {
	pub rgb: Vector3<f32>, 
}

impl SubsurfaceStrengthFactors {
	pub const fn new(r: f32, g: f32, b: f32) -> SubsurfaceStrengthFactors {
		SubsurfaceStrengthFactors {
			rgb: vec3(r, g, b)
		}
	}
}

pub struct SubsurfaceKernel {
	coefficients: Vec<Vector4<f32>>,
}

impl SubsurfaceKernel {
	pub fn as_slice(&self) -> &[Vector4<f32>] {
		self.coefficients.as_slice()
	}
	
	pub fn as_mut_slice(&mut self) -> &mut [Vector4<f32>] {
		self.coefficients.as_mut_slice()
	}
	
	pub fn kernel_size(&self) -> u32 {
		self.coefficients.len() as u32
	}
	
	pub fn new(coefficients: Vec<Vector4<f32>>) -> SubsurfaceKernel {
		SubsurfaceKernel {
			coefficients
		}
	}
}

pub struct SubsurfaceKernelGenerator {
	kernel_size: u32,
	falloff_factors: SubsurfaceFalloffFactors,
	strength_factors: SubsurfaceStrengthFactors,
}

impl SubsurfaceKernelGenerator {
	pub fn generate_kernel(&mut self) -> SubsurfaceKernel {
		// Allocate coeffs vec
		let mut coeffs = Vec::<Vector4<f32>>::with_capacity(self.kernel_size as usize);
		unsafe {coeffs.set_len(coeffs.capacity())};
		let kernel_slice = coeffs.as_mut_slice();
		
		fn gaussian(variance: f32, r: f32, falloff_factors: &SubsurfaceFalloffFactors) -> Vector3<f32> {
			let mut v = vec3::<f32>(0.0, 0.0, 0.0);
			for i in 0..3 {
				let rr = r / (0.001 + falloff_factors.rgb[i]);
				v[i] = f32::exp((-(rr * rr)) / (2.0 * variance)) / (2.0 * std::f32::consts::PI *  variance);
			}
			v
		}
		
		fn profile(r: f32, falloff_factors: &SubsurfaceFalloffFactors) -> Vector3<f32> { 
			gaussian(0.0484, r, falloff_factors) * 0.100 +
			gaussian(0.1870, r, falloff_factors) * 0.118 +
			gaussian(0.5670, r, falloff_factors) * 0.113 +
			gaussian(1.9900, r, falloff_factors) * 0.358 +
			gaussian(7.4100, r, falloff_factors) * 0.078
		}
		
		let range: f32 = if self.kernel_size > 20 {3.0} else {2.0};
		let exponent: f32 = 2.0;
		
		// Calculate the kernel offset
		let step = 2.0f32 * range / (self.kernel_size - 1) as f32;
		for i in 0..self.kernel_size as usize {
			let o = -range + (i as f32) * step;
			let sign = if o < 0.0 {-1.0} else {1.0};
			let dist = range * sign * f32::abs(f32::powf(o, exponent)) / f32::powf(range, exponent);
			
			kernel_slice[i] = Vector4 {x: 0.0, y: 0.0, z: 0.0, w: dist};
		}
		
		// Calculate the kernel weights
		for i in 0..self.kernel_size as usize {
			let w0 = if i > 0 {f32::abs(kernel_slice[i].w - kernel_slice[i - 1].w)} else {0.0};
			let w1 = if i < (self.kernel_size as usize - 1) {f32::abs(kernel_slice[i].w - kernel_slice[i + 1].w)} else {0.0};
			let area = (w0 + w1) / 2.0;
			
			let ww = kernel_slice[i].w;
			let t: Vector3<f32> = profile(ww, &self.falloff_factors) * area;
			
			kernel_slice[i] = t.extend(ww);
		}
		
		// We want the offset 0.0 to come first
		let t = kernel_slice[self.kernel_size as usize / 2];
		for i in (1..(self.kernel_size as usize / 2)).rev() {
			kernel_slice[i] = kernel_slice[i - 1];
		}
		kernel_slice[0] = t;
		
		// Calculate the sum of the weights, we will need to normalize them below
		let mut sum = vec3::<f32>(0.0, 0.0, 0.0);
		for i in 0..self.kernel_size as usize {
			sum += kernel_slice[i].truncate();
		}
		
		// Normalize the weights
		for i in 0..self.kernel_size as usize {
			kernel_slice[i] = kernel_slice[i].div_element_wise(sum.extend(1.0));
		}
		
		// Tweak the first sample using the desired strength (lerp(1.0, kernel[0].rgb, strength))
//		let w = kernel_slice[0].w;
//		kernel_slice[0] = ((vec3::<f32>(0.0, 0.0, 0.0) - self.strength_factors.rgb) + self.strength_factors.rgb * kernel_slice[0].truncate()).extend(w);
		kernel_slice[0].x = (1.0 - self.strength_factors.rgb.x) + self.strength_factors.rgb.x * kernel_slice[0].x;
		kernel_slice[0].y = (1.0 - self.strength_factors.rgb.y) + self.strength_factors.rgb.y * kernel_slice[0].y;
		kernel_slice[0].z = (1.0 - self.strength_factors.rgb.z) + self.strength_factors.rgb.z * kernel_slice[0].z;
		
		// Tweak the other samples (lerp(0.0, kernel[0].rgb, strength))
		for i in 1..self.kernel_size as usize {
			kernel_slice[i].x *= self.strength_factors.rgb.x;
			kernel_slice[i].y *= self.strength_factors.rgb.y;
			kernel_slice[i].z *= self.strength_factors.rgb.z;
		}
		
		// Return kernel
		SubsurfaceKernel::new(coeffs)
	}
	
	pub fn new(kernel_size: u32, falloff_factors: SubsurfaceFalloffFactors, strength_factors: SubsurfaceStrengthFactors) -> SubsurfaceKernelGenerator {
		SubsurfaceKernelGenerator {
			kernel_size,
			falloff_factors,
			strength_factors
		}
	}
}
