# PathTracer v2
This will be a new GPU-based path tracer that will have glTF model loading
capabilities.  

This will be written with the latest versions of WGPU, winit, and imgui.
All three crates appear to be working well together now.  In fact, any future ray
tracers that need the basic wgpu-winit-imgui framework can start by taking the source code
from main.rs, app.rs, gui.rs, and wgpu_state.rs.

## To-do list
- Implement OBJ and glTF loaders
- Create a BVH for more complex models