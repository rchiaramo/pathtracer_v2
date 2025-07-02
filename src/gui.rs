use std::time::Instant;
use imgui::{FontSource, MouseCursor};
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;

#[derive(Debug)]
pub struct UserInput {
    key_pressed: bool,
    key_released: bool,
    key: imgui::Key,
    mouse_delta: [f32; 2],
    state_changed: bool,
}

impl Default for UserInput {
    fn default() -> Self {
        Self {
            key_pressed: false,
            key_released: false,
            key: imgui::Key::Slash,
            mouse_delta: [0.0; 2],
            state_changed: false,
        }
    }
}

impl UserInput {
    pub fn reset_state(&mut self) {
        self.state_changed = false;
    }
    
    pub fn state_changed(&self) -> bool {
        self.state_changed
    }
    
    pub fn key(&self) -> imgui::Key {
        self.key
    }
    
    pub fn key_pressed(&self) -> bool {
        self.key_pressed
    }
    
    pub fn set_key_pressed(&mut self, key: imgui::Key) {
        self.key_pressed = true;
        self.key = key;
        self.state_changed = true;
    }
    
    pub fn key_released(&self) -> bool {
        self.key_released
    }
    
    pub fn set_key_released(&mut self, key: imgui::Key) {
        self.key_released = true;
        self.key = key;
        self.state_changed = true;
    }
    
    pub fn mouse_delta(&self) -> [f32; 2] {
        self.mouse_delta
    }
    
    pub fn set_mouse_delta(&mut self, mouse_delta: [f32; 2]) {
        self.mouse_delta = mouse_delta;
        self.state_changed = true;
    }
}

pub struct GUI {
    pub platform: WinitPlatform,
    pub imgui: imgui::Context,
    pub imgui_renderer: Renderer,
    pub last_cursor: Option<MouseCursor>,
    pub last_frame: Instant,
}

impl GUI {
    pub fn new(window: &winit::window::Window, surface_cap: &wgpu::SurfaceConfiguration,
               device: &wgpu::Device, queue: &wgpu::Queue)
               -> Option<Self> {

        let mut imgui = imgui::Context::create();
        let mut platform = WinitPlatform::new(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );
        
        imgui.set_ini_filename(std::path::PathBuf::from("~/Documents/Rust/pathracer_v2/imgui.ini"));

        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let renderer_config = RendererConfig {
            texture_format: surface_cap.format,
            ..Default::default()
        };

        let imgui_renderer = Renderer::new(&mut imgui, &device, &queue, renderer_config);

        let last_frame = Instant::now();

        Some(Self {
            platform,
            imgui,
            imgui_renderer,
            last_cursor: None,
            last_frame,
        })
    }

    pub fn display_ui(&mut self, 
                      window: &winit::window::Window, 
                      user_input: &mut UserInput) {
        let ui = self.imgui.new_frame();
        self.platform.prepare_render(&ui, &window);
        
        // process user input
        // if the right mouse button is held down and we move the mouse, we can orient the camera
        let mouse_down = ui.io().mouse_down;
        if mouse_down[1] {
            let mouse_delta = ui.io().mouse_delta;
            if mouse_delta != [0.0, 0.0] {
                user_input.set_mouse_delta(mouse_delta);
            }
        }
        // move up/down
        if ui.is_key_pressed(imgui::Key::E) {
            user_input.set_key_pressed(imgui::Key::E);
        }
        if ui.is_key_released(imgui::Key::E) {
            user_input.set_key_released(imgui::Key::E);
        }
        if ui.is_key_pressed(imgui::Key::Q) {
            user_input.set_key_pressed(imgui::Key::Q);
        }
        if ui.is_key_released(imgui::Key::Q) {
            user_input.set_key_released(imgui::Key::Q);
        }
        // move forward/backwards
        if ui.is_key_pressed(imgui::Key::W) {
            user_input.set_key_pressed(imgui::Key::W);
        }
        if ui.is_key_released(imgui::Key::W) {
            user_input.set_key_released(imgui::Key::W);
        }
        if ui.is_key_pressed(imgui::Key::S) {
            user_input.set_key_pressed(imgui::Key::S);
        }
        if ui.is_key_released(imgui::Key::S) {
            user_input.set_key_released(imgui::Key::S);
        }
        // move left/right
        if ui.is_key_pressed(imgui::Key::D) {
            user_input.set_key_pressed(imgui::Key::D);
        }
        if ui.is_key_released(imgui::Key::D) {
            user_input.set_key_released(imgui::Key::D);
        }
        if ui.is_key_pressed(imgui::Key::A) {
            user_input.set_key_pressed(imgui::Key::A);
        }
        if ui.is_key_released(imgui::Key::A) {
            user_input.set_key_released(imgui::Key::A);
        }
        
        {
            let window = ui.window("Hello Imgui from WGPU!");
            window
                .size([400.0, 100.0], imgui::Condition::FirstUseEver)
                .position([0.0, 0.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    let ds = ui.io().display_size;
                    ui.text(format!(
                        "Display size {:?}",
                        ds
                    ));
                    ui.separator();

                    ui.text("Sampling parameters");
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!("Mouse: {:?}", mouse_pos)
                    );
                });
        }

        if self.last_cursor != ui.mouse_cursor() {
            self.last_cursor = ui.mouse_cursor();
            self.platform.prepare_render(&ui, &window);
        }
    }
}