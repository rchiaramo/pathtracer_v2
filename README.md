# PathTracer v2
This will be a new GPU-based path tracer that will have glTF model loading
capabilities.  

This will be written with the latest versions of WGPU, winit, and imgui.
All three crates ***appear*** to be working well together now.  In fact, any future ray
tracers that need the basic wgpu-winit-imgui framework can start by taking the source code
from main.rs, app.rs, gui.rs, and wgpu_state.rs.

## To-do list
- update compute shader to ray trace two simple spheres hard-coded into kernel
- compare mega kernel vs wavefront approaches
  - create a ray generation compute shader that takes camera input to generate a buffer of rays
  - create hit kernel to determine ray-object intersection
  - create basic shader kernel
  - time the ray gen compute shader to compare it with the mega kernel approach
- Create a BVH for more complex models
- Implement OBJ and glTF loaders

## Accomplished
- Basic window up with clear color using winit and wgpu
- imgui window displaying in the upper left corner, tracking mouse movement
- basic image buffer (filled with constant color) being displayed with a display shader
- added camera 
- added gui controls for camera (wasd qe plus mouse pitch/yaw with right button held)
- added basic compute shader than can modify the image_buffer
  - mega kernel takes inv_projection and view transform matrices 
- main loop accounts for frame count and accumulated samples
  - counters reset with any user input
- render data output variable now sends render progress and avg FPS to the gui for display
- used wgpu-profiler crate to get timestamp queries running and displaying to gui