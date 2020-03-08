
fn main() {
	// Add glfw to link path
	println!(r"cargo:rustc-link-search={}", env!("GLFW_MINGW_LIBS"));
}
