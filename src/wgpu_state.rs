use std::sync::Arc;
use imgui::Context;
use wgpu_profiler::{GpuProfiler, GpuProfilerSettings};
use winit::window::Window;
use crate::gui::GUI;

pub struct WGPUState<'a> {
    window: Arc<winit::window::Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'a>,
    surface_format: wgpu::TextureFormat,
    surface_config: wgpu::SurfaceConfiguration,
}

impl<'a> WGPUState<'a> {
    pub async fn new(window: Arc<winit::window::Window>) -> WGPUState<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        // Check timestamp features.
        let features = adapter.features()
            & GpuProfiler::ALL_WGPU_TIMER_FEATURES;

        let (device, queue) = adapter
            .request_device(
            &wgpu::DeviceDescriptor {
                required_features: features,
                required_limits: wgpu::Limits::default(),
                label: Some("device"),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![surface_format.add_srgb_suffix()],
        };

        let mut wgpu_state = WGPUState {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            surface_config,
        };

        wgpu_state.configure_surface();

        wgpu_state
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn surface_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.surface_config
    }

    pub fn get_window(&self) -> Arc<Window> {
        self.window.clone()
    }

    fn configure_surface(&mut self) {
        self.surface_config.format = self.surface_format;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();
    }

    pub fn render(&self, gui: &mut GUI, 
                  display_pipeline: &wgpu::RenderPipeline, 
                  display_bind_group: &wgpu::BindGroup) {
        
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next surface texture");

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        let mut display_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment{
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        display_pass.set_pipeline(display_pipeline);
        display_pass.set_bind_group(0, display_bind_group, &[]);
        display_pass.draw(0..6, 0..1);

        drop(display_pass);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("UI RenderPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let gui_draw_data = Context::render(&mut gui.imgui);
        gui.imgui_renderer.render(gui_draw_data, &self.queue, &self.device, &mut pass).expect("Failed to render");
        drop(pass);

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }
}