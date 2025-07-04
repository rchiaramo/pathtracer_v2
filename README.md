# PathTracer v2
This will be a new GPU-based path tracer that will have glTF model loading
capabilities.  

This will be written with the latest versions of WGPU, winit, and imgui.
All three crates ***appear*** to be working well together now.  In fact, any future ray
tracers that need the basic wgpu-winit-imgui framework can start by taking the source code
from main.rs, app.rs, gui.rs, and wgpu_state.rs.

## To-do list
- Create compute shaders to update the image buffer 
  - first create a ray generation compute shader that takes camera input to generate a buffer of rays
  - time the ray gen compute shader to compare it with a mega-kernel approach
  - create a mega kernel that also takes camera input, but implements simple ray tracing
- work main loop to properly put in frame update, render progress, etc
- make gui display actually show relevant values
- add camera
- add gui controls for camera
- Implement OBJ and glTF loaders
- Create a BVH for more complex models

## Accomplished
- Basic window up with clear color using winit and wgpu
- imgui window displaying in the upper left corner, tracking mouse movement
- basic image buffer (filled with constant color) being displayed with a display shader