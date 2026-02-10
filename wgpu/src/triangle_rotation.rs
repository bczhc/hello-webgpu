use crate::{ColorExt, WgpuStateInitInfo};
use bytemuck::checked::cast_slice;
use wgpu::{
    include_wgsl, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferDescriptor,
    BufferUsages, Color, ColorTargetState, Device, FragmentState, Queue,
    RenderPipeline, RenderPipelineDescriptor, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexState,
};

pub struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: (u32, u32),
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    uniform_buffer: Buffer,
}

#[rustfmt::skip]
static VERTICES_DATA: [f32; 15] = {
    // 使用 f32 常量计算
    const SQRT_3: f32 = 1.732050808;  // √3
    const SIDE: f32 = 1.0;
    const HALF_SIDE: f32 = SIDE / 2.0;
    const HEIGHT: f32 = SQRT_3 * HALF_SIDE;  // √3/2 * 边长

    [
        // 顶部顶点 (红色)
        0.0, HEIGHT * 2.0 / 3.0, 1.0, 0.0, 0.0,
        // 左下角顶点 (绿色)
        -HALF_SIDE, -HEIGHT / 3.0, 0.0, 1.0, 0.0,
        // 右下角顶点 (蓝色)
        HALF_SIDE, -HEIGHT / 3.0, 0.0, 0.0, 1.0,
    ]
};

impl State {
    pub async fn new(info: WgpuStateInitInfo) -> State {
        let instance = info.instance;
        let surface = info.surface;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let cap = surface.get_capabilities(&adapter);

        let surface_format = cap.formats[0];

        let shader_module =
            device.create_shader_module(include_wgsl!("shaders/triangle-rotation.wgsl"));

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            vertex: VertexState {
                module: &shader_module,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[
                    // slot 0
                    VertexBufferLayout {
                        array_stride: 5 * 4,
                        attributes: &[
                            // position 0: vertex
                            VertexAttribute {
                                format: VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            // position 1: color
                            VertexAttribute {
                                format: VertexFormat::Float32x3,
                                offset: 2 * 4,
                                shader_location: 1,
                            },
                        ],
                        step_mode: Default::default(),
                    },
                ],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: surface_format.add_srgb_suffix(),
                    blend: None,
                    write_mask: Default::default(),
                })],
            }),
            label: None,
            layout: None,
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = Self::create_vertex_buffer(&device, &queue, &VERTICES_DATA);
        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 1 * 4,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        let state = State {
            uniform_buffer: buffer,
            device,
            queue,
            size: info.size,
            surface,
            surface_format,
            pipeline,
            vertex_buffer,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    fn create_vertex_buffer(device: &Device, queue: &Queue, data: &[f32]) -> Buffer {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: data.len() as u64 * 4,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        queue.write_buffer(&buffer, 0, bytemuck::cast_slice(data));
        buffer
    }

    pub fn update_elapsed(&self, value: f32) {
        self.queue
            .write_buffer(&self.uniform_buffer, 0, cast_slice(&[value]));
    }

    pub fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.0,
            height: self.size.1,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    pub fn render(&self, redraw_callback: impl FnOnce()) {
        // Create texture view
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        // Renders a gray screen
        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color::from_vec4d([0.3, 0.3, 0.3, 1.0])),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(self.uniform_buffer.as_entire_buffer_binding()),
            }],
        });

        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..3, 0..1);

        drop(pass);

        // Submit the command in the queue to execute
        redraw_callback();
        self.queue.submit([encoder.finish()]);
        surface_texture.present();
    }
}
