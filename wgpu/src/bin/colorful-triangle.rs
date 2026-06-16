use std::env;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

struct App {
    state: Option<render::State>,
    window: Option<Arc<Window>>,
    frame_counter: usize,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            state: None,
            frame_counter: 0,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let attributes = WindowAttributes::default();
        // attributes.inner_size = Some(dpi::Size::Physical(PhysicalSize::new(1024, 1024)));
        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        pollster::block_on(async {
            let _a = event_loop.owned_display_handle();
            self.state = Some(render::State::new(Arc::clone(&window)).await.unwrap());
        });

        window.request_redraw();
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(w) = &self.window {
                    if self.frame_counter > usize::MAX {
                        event_loop.exit();
                    }
                    state.render(|| {
                        w.pre_present_notify();
                    });
                    w.request_redraw();
                    self.frame_counter += 1;
                }
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                println!("{:?}", size);
                state.resize((size.width, size.height));
            }
            _ => {}
        }
    }
}

fn main() {
    unsafe {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}

mod render {
    use bytemuck::bytes_of;
    use chrono::{Local, Timelike};
    use log::{error, info};
    use palette::{FromColor, Srgb};
    use std::sync::Arc;
    use wgpu::{
        BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer,
        BufferDescriptor, BufferUsages, Color, ColorTargetState, CurrentSurfaceTexture,
        FragmentState, RenderPipeline, RenderPipelineDescriptor, VertexState, include_wgsl,
    };
    use wgpu_playground::{ColorExt, wgpu_instance_with_env_backend};
    use winit::window::Window;

    pub struct State {
        device: wgpu::Device,
        queue: wgpu::Queue,
        size: (u32, u32),
        surface: wgpu::Surface<'static>,
        surface_format: wgpu::TextureFormat,
        pipeline: RenderPipeline,
        uniform: Buffer,
        bind_group: BindGroup,
    }

    impl State {
        pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
            let instance = wgpu_instance_with_env_backend();
            let size = window.inner_size();
            let surface = instance
                .create_surface(Arc::clone(&window))
                .map_err(anyhow::Error::msg)?;

            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                .await?;
            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor::default())
                .await?;

            let cap = surface.get_capabilities(&adapter);

            let surface_format = cap.formats[0].remove_srgb_suffix();

            let shader_module =
                device.create_shader_module(include_wgsl!("../shaders/colorful-triangle.wgsl"));

            let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                vertex: VertexState {
                    module: &shader_module,
                    entry_point: None,
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(FragmentState {
                    module: &shader_module,
                    entry_point: None,
                    compilation_options: Default::default(),
                    targets: &[Some(ColorTargetState {
                        format: surface_format,
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

            let uniform = device.create_buffer(&BufferDescriptor {
                label: None,
                size: 4 * 4,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                }],
            });

            let state = State {
                device,
                queue,
                size: (size.width, size.height),
                surface,
                surface_format,
                pipeline,
                uniform,
                bind_group,
            };
            state.update_uniform(bytes_of(&[1.0_f32, 0.0, 0.0, 1.0]));

            // Configure surface for the first time
            info!(
                "Initial surface configuration; size: {:?}",
                (size.width, size.height)
            );
            state.configure_surface();

            Ok(state)
        }

        fn update_uniform(&self, data: &[u8]) {
            self.queue.write_buffer(&self.uniform, 0, data);
        }

        pub fn configure_surface(&self) {
            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                view_formats: vec![self.surface_format],
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
            info!("Resize and reconfigure; size: {:?}", self.size);
            self.configure_surface();
        }

        pub fn render(&self, pre_present_callback: impl FnOnce()) {
            info!("Render; size: {:?}", self.size);

            // Create texture view
            let surface_texture = match self.surface.get_current_texture() {
                CurrentSurfaceTexture::Success(texture) => texture,
                CurrentSurfaceTexture::Suboptimal(texture) => {
                    self.configure_surface();
                    texture
                }
                CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => {
                    return;
                }
                CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                    self.configure_surface();
                    return;
                }
                CurrentSurfaceTexture::Validation => {
                    error!("Validation error in get_current_texture");
                    return;
                }
            };
            let texture_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    format: Some(self.surface_format),
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

            let ts = Local::now().nanosecond() as f64 / 1_000_000_000f64;
            let bg_color = palette::Hsv::new_srgb(ts * 360.0, 1.0, 1.0);
            let bg_color = Srgb::from_color(bg_color);
            self.update_uniform(bytes_of(&[
                bg_color.red as f32,
                bg_color.green as f32,
                bg_color.blue as f32,
            ]));
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);

            drop(pass);

            self.queue.submit([encoder.finish()]);
            pre_present_callback();
            surface_texture.present();
        }
    }
}
