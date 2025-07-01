use std::time::Instant;
use imgui::{FontSource, MouseCursor};
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;

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

    pub fn display_ui(&mut self, window: &winit::window::Window){
        let ui = self.imgui.frame();
        self.platform.prepare_render(&ui, &window);
        {
            let window = ui.window("Hello Imgui from WGPU!");
            window
                .size([400.0, 100.0], imgui::Condition::FirstUseEver)
                .position([0.0, 0.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    ui.text(format!(
                        "Avg compute time: {:.3}ms, render progress: {:.1} %",
                        10.0,
                        // fps_counter.average_fps(),
                        0.5 * 100.0
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