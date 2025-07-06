mod app;
mod wgpu_state;
mod gui;
mod pathtracer;
mod utilities;
mod camera;
mod sampling_parameters;
mod frames_per_second;

use winit::event_loop::{ControlFlow, EventLoop};
use crate::app::App;

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
