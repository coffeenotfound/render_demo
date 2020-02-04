#![feature(str_strip)]
#![allow(deprecated)]

pub mod demo;
pub mod utils;
pub mod render;
pub mod camera;
pub mod model;
pub mod math;
pub mod asset;
pub mod structured_shader_language;

fn main() {
	demo::start();
}
