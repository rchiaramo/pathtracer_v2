use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use crate::gui::GUI;
use crate::wgpu_state::WGPUState;

#[derive(Default)]
pub struct App<'a> {
    wgpu_state: Option<WGPUState<'a>>,
    gui_controller: Option<GUI>
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(LogicalSize::new(800.0, 600.0))
                        .with_title("It's WGPU time!"))
                .unwrap(),
        );

        let state = pollster::block_on(WGPUState::new(window.clone()));

        self.gui_controller = GUI::new(&window, &state.surface_config, &state.device, &state.queue);
        self.wgpu_state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let state = self.wgpu_state.as_mut().unwrap();
        let gui = self.gui_controller.as_mut().unwrap();
        let window = state.get_window();
        if window_id != window.id() { return; }

        let now = Instant::now();
        gui.imgui.io_mut().update_delta_time(now - gui.last_frame);
        gui.last_frame = now;

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
                state.resize(size)
            },

            WindowEvent::RedrawRequested => {
                gui.display_ui(&window);
                state.render(gui);
                state.get_window().request_redraw();
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
    
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let state = self.wgpu_state.as_mut().unwrap();
        let gui = self.gui_controller.as_mut().unwrap();
        let window = state.get_window();
        gui.platform
            .prepare_frame(gui.imgui.io_mut(), &window)
            .expect("WinitPlatform::prepare_frame failed");
    }
}
