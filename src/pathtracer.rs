use glam::{Mat4, Vec3};
use wgpu::{BindGroupDescriptor, BindGroupLayoutDescriptor};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgt::BufferDescriptor;
use crate::camera::CameraController;
use crate::gui::{UserInput, GUI};
use crate::utilities::u8cast::{any_as_u8_slice, vec_as_u8_slice};
use crate::wgpu_state::WGPUState;

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

pub struct PathTracer<'a> {
    pub wgpu_state: WGPUState<'a>,
    image_buffer: wgpu::Buffer,
    frame_buffer: wgpu::Buffer,
    inv_projection_buffer: wgpu::Buffer,
    view_transform_buffer: wgpu::Buffer,
    image_bind_group: wgpu::BindGroup,
    view_proj_bind_group: wgpu::BindGroup,
    display_bind_group: wgpu::BindGroup,
    compute_shader_pipeline: wgpu::ComputePipeline,
    display_pipeline: wgpu::RenderPipeline,
    camera_controller: CameraController
}

impl<'a> PathTracer<'a> {
    pub fn new(wgpu_state: WGPUState<'a>) -> Option<Self> {
        let window = wgpu_state.get_window();
        let max_window_size = window
            .available_monitors()
            .map(|monitor| -> u32 {
                let viewport = monitor.size();
                let size = (viewport.width, viewport.height);
                size.0 * size.1
            })
            .max()
            .expect("must have at least one monitor");
        
        let device = &wgpu_state.device;
        
        let image = vec![[0.1f32, 0.2, 0.3]; max_window_size as usize];
        let image_bytes = unsafe {
            vec_as_u8_slice(&image)
        };

        let image_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Image Buffer"),
            contents: image_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let mut image_buffer_layout = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let image_buffer_binding = wgpu::BindGroupEntry { binding: 0, resource: image_buffer.as_entire_binding() };

        let frame_buffer = device.create_buffer(&BufferDescriptor{
            label: Some("Frame Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut frame_buffer_layout = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let frame_buffer_binding = wgpu::BindGroupEntry { binding: 1, resource: frame_buffer.as_entire_binding() };

        // group image and frame buffers into image bind group
        let image_bind_group_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor{
                label: Some("image bind group layout"),
                entries: &[image_buffer_layout,
                    frame_buffer_layout
                ],
            });

        let image_bind_group = device.create_bind_group(&BindGroupDescriptor{
            label: Some("image bind group"),
            layout: &image_bind_group_layout,
            entries: &[image_buffer_binding, frame_buffer_binding],
        });

        // set up the buffers for the inverse projection and view matrices
        let inv_projection_buffer = device.create_buffer(&BufferDescriptor{
            label: Some("Inverse Projection Matrix Buffer"),
            size: 16 * 4,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let inv_projection_buffer_layout = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let inv_projection_buffer_binding = wgpu::BindGroupEntry {
            binding: 0, resource: inv_projection_buffer.as_entire_binding()
        };

        let view_transform_buffer = device.create_buffer(&BufferDescriptor{
            label: Some("View Transform Buffer"),
            size: 16 * 4,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let view_transform_buffer_layout = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let view_transform_buffer_binding = wgpu::BindGroupEntry {
            binding: 1, resource: view_transform_buffer.as_entire_binding()
        };

        let view_proj_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("View Projection Bind Group Layout"),
            entries: &[
                inv_projection_buffer_layout,
                view_transform_buffer_layout,
            ],
        });

        let view_proj_bind_group = device.create_bind_group(&BindGroupDescriptor{
            label: Some("View Projection Bind Group"),
            layout: &view_proj_bind_group_layout,
            entries: &[
                inv_projection_buffer_binding,
                view_transform_buffer_binding,
            ],
        });

        // create the compute pipeline
        let path_tracer_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("compute shader pipeline layout"),
                bind_group_layouts: &[
                    &image_bind_group_layout,
                    &view_proj_bind_group_layout
                ],
                push_constant_ranges: &[],
            }
        );

        let mut shader = device.create_shader_module(
            wgpu::include_wgsl!("../shaders/compute_megakernel.wgsl")
        );

        // if I want to pass in override values, I can do it here:
        // let mut id:HashMap<String, f64> = HashMap::new();
        // id.insert("stackSize".to_string(), (bvh_tree.nodes.len() - 1) as f64);
        let compute_shader_pipeline = device.create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: Some("compute shader pipeline"),
                layout: Some(&path_tracer_pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                // PipelineCompilationOptions {
                //     constants: None, //&id,
                //     zero_initialize_workgroup_memory: false,
                //     vertex_pulling_transform: false,
                // },
                cache: None,
            }
        );

        // now create the pipeline for the display shader
        // we need to reset the layout for VERTEX_FRAGMENT rather than COMPUTE
        // and for the image buffer, read_only has to be true
        image_buffer_layout.visibility = wgpu::ShaderStages::VERTEX_FRAGMENT;
        image_buffer_layout.ty = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        };

        frame_buffer_layout.visibility = wgpu::ShaderStages::VERTEX_FRAGMENT;

        let image_buffer_binding = wgpu::BindGroupEntry { binding: 0, resource: image_buffer.as_entire_binding() };
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

        shader = device.create_shader_module(
            wgpu::include_wgsl!("../shaders/screen_shader.wgsl")
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

        let camera_controller = CameraController::new(
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 0.0),
            90.0,
            0.0,
            10.0,
            0.1,
            100.0,
            4.0,
            0.1
        );
        
        Some(
            Self {
                wgpu_state,
                image_buffer,
                frame_buffer,
                inv_projection_buffer,
                view_transform_buffer,
                image_bind_group,
                view_proj_bind_group,
                display_bind_group,
                display_pipeline,
                compute_shader_pipeline,
                camera_controller
            }
        )
    }
    
    fn frame_buffer(&self) -> &wgpu::Buffer {
        &self.frame_buffer
    }

    fn inv_projection_buffer(&self) -> &wgpu::Buffer {
        &self.inv_projection_buffer
    }

    fn view_transform_buffer(&self) -> &wgpu::Buffer {
        &self.view_transform_buffer
    }
    
    pub fn display_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.display_pipeline
    }
    
    pub fn display_bind_group(&self) -> &wgpu::BindGroup {
        &self.display_bind_group
    }

    pub fn process_user_input(&mut self, user_input: &mut UserInput) {
        self.camera_controller.process_user_input(user_input);
    }

    fn update_buffers(&mut self, dt: f32) {
        let size = self.wgpu_state.get_window().inner_size();
        let ar = size.width as f32 / size.height as f32;
        if self.camera_controller.updated() {
            self.camera_controller.update_camera(dt);
            let proj_mat = self.camera_controller.get_inv_projection_matrix(ar);
            let view_mat = self.camera_controller.get_view_transform();
            unsafe {
                self.wgpu_state.queue.write_buffer(self.inv_projection_buffer(), 0, any_as_u8_slice(&proj_mat));
                self.wgpu_state.queue.write_buffer(self.view_transform_buffer(), 0, any_as_u8_slice(&view_mat));
            }
            self.camera_controller.reset();
        }
        let frame = GPUFrameBuffer::new(size.width, size.height, 1, 1);
        let frame_data = unsafe { any_as_u8_slice(&frame) };
        self.wgpu_state.queue.write_buffer(self.frame_buffer(), 0, frame_data);
    }

    fn run_compute_kernel(&mut self) {
        let size = self.wgpu_state.get_window().inner_size();
        let mut encoder = self.wgpu_state.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("compute kernel encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute pass"),
                timestamp_writes: None,
                // Some(ComputePassTimestampWrites {
                //     query_set: &queries.set,
                //     beginning_of_pass_write_index: Some(queries.next_unused_query),
                //     end_of_pass_write_index: Some(queries.next_unused_query + 1),
                // })
            });
            // queries.next_unused_query += 2;
            compute_pass.set_pipeline(&self.compute_shader_pipeline);
            compute_pass.set_bind_group(0, &self.image_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.view_proj_bind_group, &[]);
            compute_pass.dispatch_workgroups(size.width / 4, size.height / 4, 1);

        }
        // queries.resolve(&mut encoder);
        self.wgpu_state.queue.submit(Some(encoder.finish()));
    }

    pub fn run_path_tracer(&mut self, gui: &mut GUI) {
        let dt = gui.imgui.io().delta_time;
        self.update_buffers(dt);

        self.run_compute_kernel();
        self.wgpu_state.render(gui, &self.display_pipeline, &self.display_bind_group);
    }
}