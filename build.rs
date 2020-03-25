
fn main() {
	// Add glfw to link path
	println!(r"cargo:rustc-link-search={}", env!("GLFW3_MINGW_LIBS"));
}
