#![feature(str_strip)]
#![allow(deprecated)]
#![allow(unused_parens)]

#![allow(dead_code)]

pub mod engine;
pub mod windowing;
pub mod utils;
pub mod render;
pub mod camera;
pub mod model;
pub mod math;
pub mod asset;
pub mod structured_shader_language;
pub mod btex;

fn main() {
	engine::start();
}
