use std::env;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use wgpu_playground::default;

struct App {
    state: Option<render::State>,
    window: Option<Arc<Window>>,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            state: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let mut attributes = WindowAttributes::default();
        // attributes.inner_size = Some(dpi::Size::Physical(PhysicalSize::new(1024, 1024)));
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .unwrap(),
        );

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
                    state.render(|| {
                        w.pre_present_notify();
                    });
                    w.request_redraw();
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
    use std::sync::Arc;
    use bytemuck::bytes_of;
    use chrono::Local;
    use wgpu::{
        include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer,
        BufferBinding, BufferDescriptor, BufferUsages, Color, ColorTargetState, Device, FragmentState,
        Queue, RenderPipeline, RenderPipelineDescriptor, VertexAttribute,
        VertexBufferLayout, VertexFormat, VertexState,
    };
    use wgpu_playground::{default, wgpu_instance_with_env_backend, ColorExt};
    use winit::window::Window;

    pub struct State {
        device: wgpu::Device,
        queue: wgpu::Queue,
        size: (u32, u32),
        surface: wgpu::Surface<'static>,
        surface_format: wgpu::TextureFormat,
        pipeline: RenderPipeline,
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
                .request_device(&wgpu::DeviceDescriptor::default())
                .await?;

            let cap = surface.get_capabilities(&adapter);

            let surface_format = cap.formats[0].remove_srgb_suffix();

            let shader_module =
                device.create_shader_module(include_wgsl!("../shaders/hello-triangle.wgsl"));

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

            let state = State {
                device,
                queue,
                size: (size.width, size.height),
                surface,
                surface_format,
                pipeline,
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
            let surface_texture = self
                .surface
                .get_current_texture()
                .expect("failed to acquire next swapchain texture");
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

            let ts = Local::now().timestamp_nanos_opt().unwrap();
            pass.set_pipeline(&self.pipeline);
            pass.draw(0..3, 0..1);

            drop(pass);

            redraw_callback();
            self.queue.submit([encoder.finish()]);
            surface_texture.present();
        }
    }
}
