//! A demonstration to https://github.com/niri-wm/niri/issues/3567.

use std::env;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

const STRIPE_WIDTH: f32 = 100f32;

struct App {
    state: Option<render::State>,
    window: Option<Arc<Window>>,
    delta: f32,
    paused: bool,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            state: None,
            delta: 8.,
            paused: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = WindowAttributes::default();
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
        let window = self.window.as_ref().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(w) = &self.window {
                    state.left += self.delta;
                    if state.left >= state.size.0 as f32 - STRIPE_WIDTH {
                        self.delta = -self.delta;
                    }
                    if state.left <= 0.0 {
                        self.delta = -self.delta;
                    }

                    if !self.paused {
                        state.render(|| {
                            // w.pre_present_notify();
                        });
                        w.request_redraw();
                    }
                }
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                println!("{:?}", size);
                state.resize((size.width, size.height));
                state.left = 0.0;
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Released {
                    return;
                }
                if event.logical_key == Key::Named(NamedKey::Space) {
                    self.paused = !self.paused;
                    println!("Animation paused: {}", self.paused);
                    if !self.paused {
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}

mod render {
    use crate::STRIPE_WIDTH;
    use bytemuck::bytes_of;
    use chrono::Local;
    use std::sync::Arc;
    use wgpu::util::RenderEncoder;
    use wgpu::{
        include_wgsl, Buffer, BufferDescriptor, BufferUsages, Color, ColorTargetState,
        CurrentSurfaceTexture, Features, FragmentState, IndexFormat, PipelineLayoutDescriptor,
        RenderPipeline, RenderPipelineDescriptor, VertexState,
    };
    use wgpu_playground::{wgpu_instance_with_env_backend, ColorExt};
    use winit::window::Window;

    pub struct State {
        device: wgpu::Device,
        queue: wgpu::Queue,
        pub size: (u32, u32),
        surface: wgpu::Surface<'static>,
        surface_format: wgpu::TextureFormat,
        pipeline: RenderPipeline,
        ibo: Buffer,
        pub left: f32,
    }

    impl State {
        pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
            let instance = wgpu_instance_with_env_backend();
            let size = window.inner_size();
            let surface = instance.create_surface(Arc::clone(&window))?;

            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                .await?;
            let (device, queue) = adapter
                .request_device(&{
                    let mut d = wgpu::DeviceDescriptor::default();
                    d.required_features = Features::IMMEDIATES;
                    d.required_limits.max_immediate_size = 8;
                    d
                })
                .await?;

            let cap = surface.get_capabilities(&adapter);

            let surface_format = cap.formats[0].remove_srgb_suffix();

            let shader_module =
                device.create_shader_module(include_wgsl!("../shaders/burst-animation.wgsl"));

            let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                immediate_size: 8,
            });

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
                layout: Some(&pipeline_layout),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            });

            let indices = [0_u32, 1, 2, 0, 2, 3];
            let ibo = device.create_buffer(&BufferDescriptor {
                label: None,
                size: (indices.len() * 4) as _,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&ibo, 0, bytes_of(&indices));

            let state = State {
                device,
                queue,
                size: (size.width, size.height),
                surface,
                surface_format,
                pipeline,
                ibo,
                left: 0.0,
            };

            // Configure surface for the first time
            state.configure_surface();

            Ok(state)
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
            self.configure_surface();
        }

        pub fn render(&self, redraw_callback: impl FnOnce()) {
            // Create texture view
            let surface_texture = match self.surface.get_current_texture() {
                CurrentSurfaceTexture::Success(t) => t,
                _ => {
                    self.configure_surface();
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

            pass.set_index_buffer(self.ibo.slice(..), IndexFormat::Uint32);
            pass.set_pipeline(&self.pipeline);
            pass.set_immediates(0, bytes_of(&[self.left, self.left + STRIPE_WIDTH]));
            pass.draw_indexed(0..6, 0, 0..1);

            drop(pass);

            redraw_callback();
            self.queue.submit([encoder.finish()]);
            surface_texture.present();
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
