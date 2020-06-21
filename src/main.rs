#![feature(str_strip)]
#![allow(deprecated)]
#![allow(unused_parens)]

pub mod demo;
pub mod windowing;
pub mod utils;
pub mod render;
pub mod camera;
pub mod model;
pub mod math;
pub mod asset;
pub mod structured_shader_language;
pub mod btex;
pub mod world;
pub mod garbagecs;
pub mod game;

fn main() {
//	// DEBUG:
//	ehaa_test_stuff::calc_edge_coverage_mask_lut();
	
	demo::start();
}

mod ehaa_test_stuff {
	use cgmath::{Vector2};
	use num::Integer;
	
	pub fn calc_edge_coverage_mask_lut() {
		const num_angle_slices: usize = 32;
		const num_dist_slices: usize = 32;
		const dist_range: f64 = 0.55;
		
		let mut lut = [0u16; num_angle_slices * num_dist_slices];
		
		for angle_index in 0..num_angle_slices {
			for distance_index in 0..num_dist_slices {
				let a = (angle_index as f64 / num_angle_slices as f64) * 2.0 * std::f64::consts::PI;
				let d = (distance_index as f64 / (num_dist_slices - 1) as f64) * dist_range;
				
				let normal: Vector2<f64> = Vector2::new(f64::sin(a), -f64::cos(a));
				let scaled_normal = normal * d;
				let tangent = Vector2::new(-normal.y, normal.x);
				
				let edge_end1 = Vector2::new(0.5f64, 0.5f64) + scaled_normal + tangent;
				let edge_end2 = Vector2::new(0.5f64, 0.5f64) + scaled_normal - tangent;
				
				let mut coverage_mask: u16 = 0;
				
				for sample_index in 0..16 {
					let x = (sample_index.mod_floor(&4) as f64 + 0.5) / 4.0;
					let y = (sample_index.div_floor(&4) as f64 + 0.5) / 4.0;
					let sample_pos = Vector2::new(x, y);
					
//					const EDGE_EPS: f64 = 0.001;
					let side_distance = (sample_pos.x - edge_end1.x) * (edge_end2.y - edge_end1.y) - (sample_pos.y - edge_end1.y) * (edge_end2.x - edge_end1.x);
					let covered = (side_distance > 0.0);
					
					if covered {
						coverage_mask |= (0x1 << sample_index);
					}
				}
				
				let lut_index = angle_index * num_dist_slices + distance_index;
				lut[lut_index] = coverage_mask;
			}
		}
		
		// Print
		println!("const uint16_t EDGE_TO_COVERGE_MARK_LUT[{}*{}] = {{", num_dist_slices, num_angle_slices);
		for row in lut.chunks_exact(num_dist_slices) {
			print!("\t");
			for v in row {
				print!("{:#06X}us, ", v);
			}
			println!();
		}
		println!("}};");
	}
}
