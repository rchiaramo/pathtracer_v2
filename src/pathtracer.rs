use wgpu::{BindGroupDescriptor};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgt::BufferDescriptor;
use crate::utilities::u8cast::{any_as_u8_slice, vec_as_u8_slice};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GPUFrameBuffer {
    width: u32,
    height: u32,
    frame: u32,
    accumulated_samples: u32
}

impl GPUFrameBuffer {
    pub fn new(width: u32, height: u32, frame: u32, accumulated_samples: u32) -> Self {
        Self {
            width,
            height,
            frame,
            accumulated_samples
        }
    }
}

pub struct PathTracer {
    image_buffer: wgpu::Buffer,
    frame_buffer: wgpu::Buffer,
    display_bind_group: wgpu::BindGroup,
    display_pipeline: wgpu::RenderPipeline,
}

impl PathTracer {
    pub fn new(device: &wgpu::Device, max_window_size: u32) -> Option<Self> {
        let image = vec![[0.1f32, 0.2, 0.3]; max_window_size as usize];
        let image_bytes = unsafe {
            vec_as_u8_slice(&image)
        };

        let image_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Image Buffer"),
            contents: image_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        
        let frame_buffer = device.create_buffer(&BufferDescriptor{
            label: Some("Frame Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let image_buffer_layout = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        
        let image_buffer_binding = wgpu::BindGroupEntry { binding: 0, resource: image_buffer.as_entire_binding() };
        
        let frame_buffer_layout = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let frame_buffer_binding = wgpu::BindGroupEntry { binding: 1, resource: frame_buffer.as_entire_binding() };
        
        let display_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Display Bind Group Layout"),
            entries: &[
                image_buffer_layout,
                frame_buffer_layout,
            ],
        });
        
        let display_bind_group = device.create_bind_group(&BindGroupDescriptor{
            label: Some("Display Bind Group"),
            layout: &display_bind_group_layout,
            entries: &[
                image_buffer_binding,
                frame_buffer_binding,
            ],
        });
        
        let shader = device.create_shader_module(
            wgpu::include_wgsl!("../screen_shader.wgsl")
        );
        
        let display_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Display Pipeline Layout"),
            bind_group_layouts: &[&display_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let display_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Display Pipeline"),
            layout: Some(&display_pipeline_layout), 
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState{
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });
        
        Some(
            Self {
                image_buffer,
                frame_buffer,
                display_bind_group,
                display_pipeline
            }
        )
    }
    
    pub fn frame_buffer(&self) -> &wgpu::Buffer {
        &self.frame_buffer
    }
    
    pub fn display_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.display_pipeline
    }
    
    pub fn display_bind_group(&self) -> &wgpu::BindGroup {
        &self.display_bind_group
    }
}