use std::time::{Duration, Instant};
use imgui::{FontSource, MouseCursor};
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use crate::frames_per_second::FramesPerSecond;


pub struct RenderStats {
    progress: f32,
    frames_per_second: FramesPerSecond
}

impl Default for RenderStats {
    fn default() -> Self {
        Self {
            progress: 0.0,
            frames_per_second: FramesPerSecond::new()
        }
    }
}

impl RenderStats {
    pub fn update_progress(&mut self, progress: f32, dt: Duration) {
        self.progress = progress;
        self.frames_per_second.update(dt);
    }
}

#[derive(Debug)]
pub struct UserInput {
    key_pressed: bool,
    key_released: bool,
    key: imgui::Key,
    mouse_delta: [f32; 2],
    vfov: f32,
    defocus_angle: f32,
    focus_distance: f32,
    samples_per_frame: u32,
    samples_per_pixel: u32,
    number_of_bounces: u32,
    state_changed: bool,
}

impl Default for UserInput {
    fn default() -> Self {
        Self {
            key_pressed: false,
            key_released: false,
            key: imgui::Key::Slash,
            mouse_delta: [0.0; 2],
            vfov: 90.0f32,
            defocus_angle: 0.0,
            focus_distance: 10.0,
            samples_per_frame: 1,
            samples_per_pixel: 50,
            number_of_bounces: 1,
            state_changed: true,
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
    
    pub fn vfov(&self) -> f32 {
        self.vfov
    }
    
    fn set_vfov(&mut self, deg: f32) {
        self.vfov = deg;
        self.state_changed = true;
    }
    
    pub fn defocus_angle(&self) -> f32 {
        self.defocus_angle
    }
    
    fn set_defocus_angle(&mut self, defocus_angle: f32) {
        self.defocus_angle = defocus_angle;
        self.state_changed = true;
    }
    
    pub fn focus_distance(&self) -> f32 {
        self.focus_distance
    }
    
    fn set_focus_distance(&mut self, focus_distance: f32) {
        self.focus_distance = focus_distance;
        self.state_changed = true;
    }

    pub fn samples_per_frame(&self) -> u32 {
        self.samples_per_frame
    }

    fn set_samples_per_frame(&mut self, samples_per_frame: u32) {
        self.samples_per_frame = samples_per_frame;
        self.state_changed = true;
    }

    pub fn samples_per_pixel(&self) -> u32 {
        self.samples_per_pixel
    }

    fn set_samples_per_pixel(&mut self, samples_per_pixel: u32) {
        self.samples_per_pixel = samples_per_pixel;
        self.state_changed = true;
    }

    pub fn number_of_bounces(&self) -> u32 {
        self.number_of_bounces
    }

    fn set_number_of_bounces(&mut self, number_of_bounces: u32) {
        self.number_of_bounces = number_of_bounces;
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
                      user_input: &mut UserInput, 
                      render_stats: &RenderStats) {
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
                .size([400.0, 250.0], imgui::Condition::FirstUseEver)
                .position([0.0, 0.0], imgui::Condition::FirstUseEver)
                // .collapsed(true, imgui::Condition::FirstUseEver)
                .build(|| {
                    let ds = ui.io().display_size;
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!("Display size {:?} Mouse: {:?}", ds, mouse_pos)
                    );
                    ui.separator();
                    ui.text(format!("Progress: {:.2}%  Average FPS: {:.1}s", 
                                    render_stats.progress, render_stats.frames_per_second.get_avg_fps()));
                    ui.separator();

                    ui.text("Camera parameters");
                    
                    let mut fov = user_input.vfov();
                    if ui.slider(
                        "vfov",
                        30.0,
                        120.0,
                        &mut fov,
                    ) {
                        user_input.set_vfov(fov);
                    };

                    let mut defocus_angle = user_input.defocus_angle();
                    if ui.slider(
                        "defocus angle",
                        0.0,
                        1.0,
                        &mut defocus_angle,
                    ) {
                        user_input.set_defocus_angle(defocus_angle);
                    };

                    let mut focus_distance = user_input.focus_distance();
                    if ui.slider(
                        "focus distance",
                        5.0,
                        20.0,
                        &mut focus_distance,
                    ) {
                        user_input.set_focus_distance(focus_distance);
                    };
                    
                    ui.separator();

                    ui.text("Sampling parameters");

                    let mut spf = user_input.samples_per_frame();
                    if ui.slider(
                        "Samples per frame",
                        1,
                        10,
                        &mut spf,
                    ) {
                        user_input.set_samples_per_frame(spf);
                    };

                    let mut spp = user_input.samples_per_pixel();
                    if ui.slider(
                        "Samples per pixel",
                        1,
                        1000,
                        &mut spp,
                    ){
                        user_input.set_samples_per_pixel(spp);
                    };

                    let mut nb = user_input.number_of_bounces();
                    if ui.slider(
                        "num bounces",
                        1,
                        100,
                        &mut nb,
                    ){
                        user_input.set_number_of_bounces(nb);
                    };
                });
        }

        if self.last_cursor != ui.mouse_cursor() {
            self.last_cursor = ui.mouse_cursor();
            self.platform.prepare_render(&ui, &window);
        }
    }
}