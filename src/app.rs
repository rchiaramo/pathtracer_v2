use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use crate::gui::{UserInput, GUI};
use crate::pathtracer::PathTracer;
use crate::wgpu_state::WGPUState;

#[derive(Default)]
pub struct App<'a> {
    gui_controller: Option<GUI>,
    path_tracer: Option<PathTracer<'a>>,
    user_input: UserInput
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // println!("Resumed: {:?}", Instant::now());
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(LogicalSize::new(1200.0, 675.0))
                        .with_title("It's WGPU time!"))
                .unwrap(),
        );

        let wgpu_state = pollster::block_on(WGPUState::new(window.clone()));

        self.gui_controller = GUI::new(&window, &wgpu_state.surface_config, &wgpu_state.device, &wgpu_state.queue);
        self.path_tracer = PathTracer::new(wgpu_state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let path_tracer = self.path_tracer.as_mut().unwrap();
        let window = path_tracer.wgpu_state.get_window();
        if window_id != window.id() { return; }

        let gui = self.gui_controller.as_mut().unwrap();
        let now = Instant::now();
        gui.imgui.io_mut().update_delta_time(now - gui.last_frame);
        gui.last_frame = now;
        // println!("window event: {:}", (Instant::now() - gui.last_frame).as_nanos());

        match event {
            WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    ..
                },
                ..
            } => {
                event_loop.exit();
            },

            WindowEvent::Resized(size) => {
                // let logical_size = size.to_logical(window.scale_factor());
                // let logical_size = gui.platform.scale_size_from_winit(window, logical_size);
                // gui.imgui.io_mut().display_size = [logical_size.width as f32, logical_size.height as f32];
                path_tracer.wgpu_state.resize(size);
            },

            WindowEvent::RedrawRequested => {
                gui.display_ui(&window, &mut self.user_input);
                if self.user_input.state_changed() {
                    println!("user_input {:?}", self.user_input);
                }
                path_tracer.process_user_input(&mut self.user_input);
                path_tracer.run_path_tracer(gui);
                window.request_redraw();
            },

            _ => {
                let generic_event: winit::event::Event<WindowEvent> = winit::event::Event::WindowEvent {
                    window_id,
                    event,
                };
                gui.platform.handle_event(gui.imgui.io_mut(), &window, &generic_event);
                window.request_redraw();
            },
        }

    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let window = self.path_tracer.as_mut().unwrap().wgpu_state.get_window();
        let gui = self.gui_controller.as_mut().unwrap();
        
        // let now = Instant::now();
        // gui.last_frame = now;
        // println!("about to wait: {:}", (Instant::now() - gui.last_frame).as_nanos());
        
        gui.platform
            .prepare_frame(gui.imgui.io_mut(), &window)
            .expect("WinitPlatform::prepare_frame failed");
    }
}
