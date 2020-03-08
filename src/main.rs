#![feature(str_strip)]
#![allow(deprecated)]
#![allow(unused_parens)]

pub mod demo;
pub mod window;
pub mod utils;
pub mod render;
pub mod camera;
pub mod model;
pub mod math;
pub mod asset;
pub mod structured_shader_language;
pub mod btex;

/// This should force NVIDIA Optimus to switch to
/// the discrete GPU on notebooks.
/// But it doesn't seem to work or maybe my notebook
/// just doesn't support Optimus.
#[no_mangle]
#[used]
#[export_name = "NvOptimusEnablement"]
pub static NvOptimusEnablement: u32 = 0x1;

fn main() {
	demo::start();
}
