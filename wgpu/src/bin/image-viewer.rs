use bytemuck::{bytes_of, cast_slice_mut};
use clap::Parser;
use image::GenericImageView;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use log::info;
use wgpu::{
    include_wgsl, BindGroupDescriptor, BindGroupEntry, BufferDescriptor, BufferUsages,
    ColorTargetState, Device, FragmentState, Instance, LoadOp, LoadOpDontCare, Operations,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, StoreOp,
    Surface, SurfaceConfiguration, TextureUsages, TextureViewDescriptor, TextureViewDimension,
    VertexState,
};
use wgpu_playground::{default, set_up_logger, wgpu_instance_with_env_backend};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey, SmolStr};
use winit::window::{Window, WindowId};

struct App {
    state: Option<State>,
    window: Option<Arc<Window>>,
    config: Config,
}

struct Config {
    image_path: PathBuf,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(default!()).unwrap();
        let window = Arc::new(window);

        let instance = wgpu_instance_with_env_backend();
        let surface = instance.create_surface(window.clone()).unwrap();
        let window_size = window.inner_size();

        let state = State::new(
            instance,
            surface,
            (window_size.width, window_size.height),
            &self.config.image_path,
        )
        .unwrap();
        self.state = Some(state);
        self.window = Some(Arc::clone(&window));

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };
        let Some(window) = &mut self.window else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state
                    .render(|| {
                        window.pre_present_notify();
                    })
                    .unwrap();
            }
            WindowEvent::Resized(size) => {
                state.resize((size.width as _, size.height as _));
            }
            WindowEvent::KeyboardInput {event, ..} => {
                if event.logical_key == Key::Character("q".into()) {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

fn open_image(path: impl AsRef<Path>) -> anyhow::Result<(u32, u32, Vec<u8>)> {
    let img = image::open(path)?;
    let width = img.width();
    let height = img.height();

    let mut rgb888_buf = Vec::with_capacity(width as usize * height as usize * 3);
    for x in img.pixels() {
        rgb888_buf.push(x.2[0]);
        rgb888_buf.push(x.2[1]);
        rgb888_buf.push(x.2[2]);
    }

    Ok((width, height, rgb888_buf))
}

#[derive(Parser)]
struct Args {
    /// Path of image
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    set_up_logger();

    let args = Args::parse();

    let el = EventLoop::new()?;
    el.set_control_flow(ControlFlow::Wait);

    let mut app = App {
        state: None,
        window: None,
        config: Config {
            image_path: args.path,
        },
    };
    el.run_app(&mut app)?;
    Ok(())
}

struct State {
    device: Device,
    pipeline: RenderPipeline,
    queue: wgpu::Queue,
    surface_format: wgpu::TextureFormat,
    surface: Surface<'static>,
    size: (u32, u32),
    bind_group: wgpu::BindGroup,
}

impl State {
    fn new(
        instance: Instance,
        surface: Surface<'static>,
        init_size: (u32, u32),
        image_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let (image_width, image_height, image_buf) = open_image(image_path)?;

        let adapter = pollster::block_on(instance.request_adapter(&default!()))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&default!()))?;

        let caps = surface.get_capabilities(&adapter);
        // disable auto gamma encoding
        let surface_format = caps.formats[0].remove_srgb_suffix();

        let module = device.create_shader_module(include_wgsl!("../shaders/image-viewer.wgsl"));

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: VertexState {
                module: &module,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &module,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: Default::default(),
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let uniform = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 1 * 4,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        queue.write_buffer(&uniform, 0, bytes_of(&[image_width]));

        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: image_width as u64 * image_height as u64 * 3 * 4,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        let mut range = buffer.get_mapped_range_mut(..);
        let view = &mut *range;
        let view: &mut [u32] = cast_slice_mut(view);
        for (i, x) in image_buf.iter().enumerate() {
            view[i] = *x as u32;
        }
        drop(range);
        buffer.unmap();

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });

        let mut state = Self {
            device,
            pipeline,
            queue,
            surface_format,
            surface,
            size: init_size,
            bind_group,
        };

        // run an initial surface configuration
        state.reconfigure_surface();
        Ok(state)
    }

    fn resize(&mut self, size: (u32, u32)) {
        self.size = size;
        self.reconfigure_surface();
    }

    fn reconfigure_surface(&mut self) {
        info!("Configure surface: width {}, height {}", self.size.0, self.size.1);
        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                width: self.size.0,
                height: self.size.1,
                present_mode: Default::default(),
                desired_maximum_frame_latency: 0,
                alpha_mode: Default::default(),
                view_formats: vec![self.surface_format],
            },
        );
    }

    fn render(&self, pre_present_op: impl FnOnce()) -> anyhow::Result<()> {
        let mut encoder = self.device.create_command_encoder(&default!());

        let texture = self.surface.get_current_texture()?;
        let texture_view = texture.texture.create_view(&TextureViewDescriptor {
            label: None,
            format: Some(self.surface_format),
            dimension: Some(TextureViewDimension::D2),
            usage: Some(TextureUsages::RENDER_ATTACHMENT),
            aspect: Default::default(),
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::DontCare(LoadOpDontCare::default()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..6, 0..1);
        drop(pass);

        let command_buffer = encoder.finish();

        self.queue.submit([command_buffer]);
        pre_present_op();
        texture.present();

        Ok(())
    }
}
