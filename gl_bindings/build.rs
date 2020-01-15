use std::env;
use std::fs::File;
use std::path::Path;
use gl_generator::{Registry, Api, Profile, Fallbacks, GlobalGenerator};

fn main() {
	// Generate gl bindings
	let gl_dest = env::var("OUT_DIR").unwrap();
	let mut gl_file = File::create(&Path::new(&gl_dest).join("bindings.rs")).unwrap();
	
	let exts: &[&str] = &[
		"GL_ARB_direct_state_access",
		"GL_ARB_clip_control",
		"GL_KHR_debug",
	];
	Registry::new(Api::Gl, (4, 5), Profile::Compatibility, Fallbacks::All, exts)
		.write_bindings(GlobalGenerator, &mut gl_file)
		.expect("Failed to generate opengl bindings");
}
