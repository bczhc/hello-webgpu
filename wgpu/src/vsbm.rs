use crate::WgpuStateInitInfo;
use bytemuck::{Pod, Zeroable};
use std::iter;
use std::time::{Duration, Instant};

// --- Uniform 数据结构 (必须符合 WGSL 的 16 字节对齐) ---
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Uniforms {
    origin: [f32; 3],
    padding1: f32,
    right: [f32; 3],
    padding2: f32,
    up: [f32; 3],
    padding3: f32,
    forward: [f32; 3],
    padding4: f32,
    screen_size: [f32; 2],
    len: f32,
    padding5: f32,
}

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub size: (u32, u32),
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    elapsed: f32,
    texture_format: wgpu::TextureFormat,
}

impl State {
    pub fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.texture_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.texture_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.0,
            height: self.size.1,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoNoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    pub async fn new(info: WgpuStateInitInfo) -> Self {
        let instance = info.instance;
        let surface = info.surface;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let texture_format = surface_caps.formats[0];

        // --- 核心 WGSL 着色器 ---
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/vsbm.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: None,
                    write_mask: Default::default(),
                })],
            }),
            multiview_mask: None,
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
        });

        let state = Self {
            surface,
            device,
            queue,
            size: info.size,
            render_pipeline,
            uniform_buffer,
            uniform_bind_group,
            elapsed: 0f32,
            texture_format,
        };
        state.configure_surface();
        state
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    pub fn update(&mut self) {
        self.elapsed += 0.012;
        let ang1 = 2.8 + self.elapsed * 0.5; // 自动旋转
        let ang2: f32 = 0.4;
        let len = 1.6;

        let origin = [
            len * ang1.cos() * ang2.cos(),
            len * ang2.sin(),
            len * ang1.sin() * ang2.cos(),
        ];
        let right = [ang1.sin(), 0.0, -ang1.cos()];
        let up = [
            -ang2.sin() * ang1.cos(),
            ang2.cos(),
            -ang2.sin() * ang1.sin(),
        ];
        let forward = [
            -ang1.cos() * ang2.cos(),
            -ang2.sin(),
            -ang1.sin() * ang2.cos(),
        ];

        let cx = self.size.0 as f32;
        let cy = self.size.1 as f32;
        let sx = (cx.min(cy) / cx) * (cx / cx.max(cy));
        let sy = (cy.min(cx) / cy) * (cy / cx.max(cy));

        // 因为使用了 1:1 的 Viewport，这里 screen_size 直接给 1.0 即可
        let uniforms = Uniforms {
            origin,
            padding1: 0.0,
            right,
            padding2: 0.0,
            up,
            padding3: 0.0,
            forward,
            padding4: 0.0,
            screen_size: [1.0, 1.0],
            len,
            padding5: 0.0,
        };

        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn render(&self, before_submit_callback: impl FnOnce()) -> Result<(), wgpu::SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.texture_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.07,
                            g: 0.06,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            // --- 核心逻辑：设置居中的 1:1 正方形视口 ---
            let win_w = self.size.0 as f32;
            let win_h = self.size.1 as f32;
            let side = win_w.min(win_h); // 取短边
            let x_offset = (win_w - side) / 2.0;
            let y_offset = (win_h - side) / 2.0;

            render_pass.set_viewport(x_offset, y_offset, side, side, 0.0, 1.0);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        before_submit_callback();
        self.queue.submit(iter::once(encoder.finish()));
        surface_texture.present();
        Ok(())
    }
}
