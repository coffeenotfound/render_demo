# Render Demo

<img src="https://raw.githubusercontent.com/coffeenotfound/render_demo/master/.repo/example_screenshot_subsurface_head2.jpg">

# Builing

Building is done through cargo as normal but you need atleast rustc 1.43.0-nightly.

On Windows make sure you have (pre-)compiled GLFW3.3+ libs installed and
an environment variable named `GLFW_MINGW_LIBS` pointed to the folder
containing the right `libglfw3.a` (for mingw-w64 if using the GNU toolchain).

Linux is currently not supported but it should be easy to get it working.
Just make sure you have the proper compiled glfw libs.
