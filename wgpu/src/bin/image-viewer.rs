use bytemuck::{bytes_of, cast_slice_mut, Pod, Zeroable};
use chrono::Local;
use clap::Parser;
use cosmic_text::Family;
use image::GenericImageView;
use log::{error, info};
use static_assertions::{assert_eq_size, const_assert_eq};
use std::ffi::OsStr;
use std::fs;
use std::num::NonZeroU32;
use std::ops::IndexMut;
use std::path::{Path, PathBuf};
use std::slice::SliceIndex;
use std::sync::Arc;
use wgpu::wgt::strict_assert_eq;
use wgpu::{
    include_wgsl, AddressMode, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer,
    BufferDescriptor, BufferUsages, ColorTargetState, Device, Extent3d, FilterMode, FragmentState, Instance,
    LoadOp, LoadOpDontCare, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, SamplerDescriptor, StoreOp, Surface,
    SurfaceConfiguration, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension,
    VertexState,
};
use wgpu_playground::{default, set_up_logger, wgpu_instance_with_env_backend};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey, SmolStr};
use winit::monitor::MonitorHandle;
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::{Fullscreen, Window, WindowAttributes, WindowId};

struct InfoState {
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    size: (u32, u32),
    text_renderer: TextRenderer,
}

impl InfoState {
    fn new(window: Arc<Window>) -> Self {
        let context = softbuffer::Context::new(Arc::clone(&window)).unwrap();
        let mut surface = softbuffer::Surface::new(&context, window).unwrap();

        Self {
            surface,
            size: (0, 0),
            text_renderer: TextRenderer::new(),
        }
    }

    fn resize(&mut self, size: (u32, u32)) {
        self.size = size;
        self.surface
            .resize(size.0.try_into().unwrap(), size.1.try_into().unwrap())
            .unwrap();
    }

    fn present_text(&mut self, text: &str) {
        if self.size == (0, 0) {
            return;
        }
        let mut buffer = self.surface.buffer_mut().unwrap();
        buffer.fill(0x000000);
        self.text_renderer.render(buffer.as_mut(), self.size, text);
        buffer.present().unwrap();
    }
}

struct App<'a> {
    state: Option<State>,
    window: Option<Arc<Window>>,
    info_window: Option<Arc<Window>>,
    info_state: Option<InfoState>,
    image_list: Vec<PathBuf>,
    image_index: usize,
    args: &'a Args,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_title("WGPU Image Viewer")
            .with_inner_size(LogicalSize::new(640.0f32, 480.0f32))
            .with_visible(true);
        let window = event_loop.create_window(attributes).unwrap();
        let window = Arc::new(window);

        let instance = wgpu_instance_with_env_backend();
        let surface = instance.create_surface(window.clone()).unwrap();
        let window_size = window.inner_size();

        let mut state = State::new(
            instance,
            surface,
            (window_size.width, window_size.height),
            self.args.no_scale,
        )
        .unwrap();
        state.set_image(&self.image_list[self.image_index]).unwrap();

        self.state = Some(state);
        self.window = Some(Arc::clone(&window));
        window.request_redraw();

        let attributes = WindowAttributes::default()
            .with_title("WGPU Image Viewer")
            .with_inner_size(LogicalSize::new(640.0f32, 480.0f32))
            .with_visible(true);
        let info_window = event_loop.create_window(attributes).unwrap();
        let info_window = Arc::new(info_window);

        let info_state = InfoState::new(Arc::clone(&info_window));
        self.info_state = Some(info_state);
        self.info_window = Some(info_window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };
        let Some(window) = &mut self.window else {
            return;
        };
        let Some(info_state) = &mut self.info_state else {
            return;
        };
        let info_window_id = info_state.surface.window().id();
        let main_window_id = window.id();

        if window_id == main_window_id {
            match &event {
                WindowEvent::RedrawRequested => {
                    let render_result = state.render(|| {
                        window.pre_present_notify();
                    });
                    match render_result {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.reconfigure_surface();
                        }
                        Err(e) => {
                            error!("Render error: {:?}", e);
                            event_loop.exit();
                        }
                    }
                    window.request_redraw();
                }
                WindowEvent::Resized(size) => {
                    state.resize((size.width as _, size.height as _));
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if !event.state.is_pressed() {
                        return;
                    }
                    match &event.logical_key {
                        Key::Character(x) if x == "q" => {
                            event_loop.exit();
                        }
                        Key::Character(x) if x == "f" => {
                            self.toggle_fullscreen();
                        }
                        Key::Named(NamedKey::ArrowLeft) => {
                            self.previous_image();
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            self.next_image();
                        }
                        Key::Character(x) if x == "s" => {
                            state.uniform_data.no_scale.flip();
                        }
                        Key::Character(x) if x == "r" => {
                            state.uniform_data.proportional.flip();
                        }
                        _ => {}
                    }
                }
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_x, y),
                    ..
                } if window_id == main_window_id => {
                    if *y == -1.0 {
                        self.next_image();
                    } else if *y == 1.0 {
                        self.previous_image();
                    }
                }
                _ => {}
            }
        } else if window_id == info_window_id {
            match &event {
                WindowEvent::Resized(x) => {
                    info_state.resize((x.width, x.height));
                }
                WindowEvent::RedrawRequested => {
                    let path_text = format!("{}", self.image_list[self.image_index].display());
                   info_state.present_text(&path_text);
                   info_state.surface.window().request_redraw();
                }
                _ => {}
            }
        }

        // common
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}

struct TextRenderer {
    swash_cache: cosmic_text::SwashCache,
    font_system: cosmic_text::FontSystem,
}

impl TextRenderer {
    fn new() -> Self {
        use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};

        // A FontSystem provides access to detected system fonts, create one per application
        let mut font_system = FontSystem::new();
        font_system.db_mut().load_system_fonts();

        // A SwashCache stores rasterized glyphs, create one per application
        let swash_cache = SwashCache::new();
        Self {
            swash_cache,
            font_system,
        }
    }

    fn render(&mut self, out_buffer: &mut [u32], buffer_size: (u32, u32), text: &str) {
        // Text metrics indicate the font size and line height of a buffer
        let metrics = cosmic_text::Metrics::new(24.0, 24.0);

        // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
        let mut buffer = cosmic_text::Buffer::new(&mut self.font_system, metrics);

        // Borrow buffer together with the font system for more convenient method calls
        let mut buffer = buffer.borrow_with(&mut self.font_system);

        // Attributes indicate what font to choose
        let attrs = cosmic_text::Attrs::new().family(Family::Name("Noto Sans CJK SC"));

        // Set size and text
        buffer.set_size(Some(buffer_size.0 as _), Some(buffer_size.1 as _));
        buffer.set_text(text, &attrs, cosmic_text::Shaping::Advanced, None);

        // Create a default text color
        let text_color = cosmic_text::Color::rgb(0xFF, 0xFF, 0xFF);

        // Draw the buffer (for performance, instead use SwashCache directly)
        buffer.draw(&mut self.swash_cache, text_color, |x, y, w, h, color| {
            for w_i in 0..w {
                for h_i in 0..h {
                    if x < 0 || y < 0 {
                        continue;
                    }
                    let x0 = x as u32 + w_i;
                    let y0 = y as u32 + h_i;
                    let index = buffer_size.0 as usize * y0 as usize + x0 as usize;
                    if let Some(x) = out_buffer.get_mut(index) {
                        let rgba = color.as_rgba();
                        let mut c: u32 =
                            rgba[2] as u32 + ((rgba[1] as u32) << 8) + ((rgba[0] as u32) << 16);
                        if rgba[3] == 0 {
                            c = 0;
                        }
                        *x = c;
                    }
                }
            }
        });
    }
}

impl App<'_> {
    fn previous_image(&mut self) {
        let Some(ref mut state) = self.state else {
            return;
        };
        if self.image_index == 0 {
            self.image_index = self.image_list.len() - 1;
        } else {
            self.image_index -= 1;
        }
        state.set_image(&self.image_list[self.image_index]).unwrap();
    }

    fn next_image(&mut self) {
        let Some(ref mut state) = self.state else {
            return;
        };
        if self.image_index == self.image_list.len() - 1 {
            self.image_index = 0;
        } else {
            self.image_index += 1;
        }
        state.set_image(&self.image_list[self.image_index]).unwrap();
    }

    fn toggle_fullscreen(&self) {
        let Some(ref window) = self.window else {
            return;
        };
        let state = window.fullscreen();
        match state {
            None => {
                let monitor_handle = window.current_monitor();
                window.set_fullscreen(Some(Fullscreen::Borderless(monitor_handle)))
            }
            Some(_x) => {
                window.set_fullscreen(None);
            }
        }
    }
}

/// Returns rgba8888
fn open_image(path: impl AsRef<Path>) -> anyhow::Result<(u32, u32, Vec<u8>)> {
    let img = image::open(path)?;
    let width = img.width();
    let height = img.height();

    let mut out_buf = Vec::with_capacity(width as usize * height as usize * 3);
    for x in img.pixels() {
        out_buf.push(x.2[0]);
        out_buf.push(x.2[1]);
        out_buf.push(x.2[2]);
        // alpha
        out_buf.push(255);
    }

    Ok((width, height, out_buf))
}

#[derive(Parser)]
struct Args {
    /// Path of image file/a folder
    path: PathBuf,
    /// Do not scale the image (use `textureLoad` in the shader)
    #[arg(short = 's', long)]
    no_scale: bool,
}

fn main() -> anyhow::Result<()> {
    set_up_logger();

    let args = Args::parse();

    let mut image_list = Vec::new();

    let dir_path: PathBuf = if !args.path.is_dir() {
        let parent = args
            .path
            .parent()
            .expect("Can't get the parent of the given file path");
        parent.into()
    } else {
        args.path.clone()
    };

    let dir = fs::read_dir(dir_path)?;
    for x in dir {
        let entry = x?;
        let path = entry.path();
        if path.extension() == Some(OsStr::new("jpg"))
            || path.extension() == Some(OsStr::new("png"))
        {
            image_list.push(path);
        }
    }
    image_list.sort();

    // display the given image first
    let mut image_index = 0_usize;
    if !args.path.is_dir() {
        let position = image_list
            .iter()
            .position(|x| x.canonicalize().unwrap() == args.path.canonicalize().unwrap())
            .unwrap();
        image_index = position;
    }

    if image_list.is_empty() {
        eprintln!("There's no image in the folder");
        return Ok(());
    }

    let el = EventLoop::new()?;
    el.set_control_flow(ControlFlow::Wait);

    let mut app = App {
        state: None,
        window: None,
        image_list,
        image_index,
        args: &args,
        info_window: None,
        info_state: None,
    };
    el.run_app(&mut app)?;
    Ok(())
}

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Uniform {
    image_size: [u32; 2],
    out_size: [u32; 2],
    uv_offset: [f32; 2],
    _pad1: [u32; 2],
    no_scale: WgpuBool,
    proportional: WgpuBool,
    _pad2: u32,
    _pad3: u32,
}

#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(transparent)]
struct WgpuBool(u32);

impl WgpuBool {
    fn flip(&mut self) {
        let new_value = match self.0 {
            0 => 1,
            1 => 0,
            x => x,
        };
        *self = Self(new_value);
    }
}

impl From<bool> for WgpuBool {
    fn from(value: bool) -> Self {
        Self(value.into())
    }
}

assert_eq_size!(Uniform, [u32; 12]);

struct State {
    device: Device,
    pipeline: RenderPipeline,
    queue: wgpu::Queue,
    surface_format: TextureFormat,
    surface: Surface<'static>,
    size: (u32, u32),
    bind_group: Option<wgpu::BindGroup>,
    uniform: Buffer,
    uniform_data: Uniform,
    sampler: wgpu::Sampler,
}

impl State {
    fn new(
        instance: Instance,
        surface: Surface<'static>,
        init_size: (u32, u32),
        no_scale: bool,
    ) -> anyhow::Result<Self> {
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
            size: size_of::<Uniform>() as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            ..default!()
        });

        let mut state = Self {
            device,
            pipeline,
            queue,
            surface_format,
            surface,
            size: init_size,
            bind_group: None,
            uniform,
            uniform_data: Uniform::zeroed(),
            sampler,
        };
        state.uniform_data.no_scale = no_scale.into();
        state.write_uniform();

        // run an initial surface configuration
        state.reconfigure_surface();
        Ok(state)
    }

    fn set_image(&mut self, file: impl AsRef<Path>) -> anyhow::Result<()> {
        let file = file.as_ref();
        info!("Set image: {}", file.display());
        let (width, height, image_buf) = open_image(file)?;

        let texture = self.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            sample_count: 1,
            mip_level_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            view_formats: &[TextureFormat::Rgba8UnormSrgb],
        });

        self.queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            &image_buf,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4 * 1),
                rows_per_image: None,
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.uniform_data.image_size = [width, height];
        self.write_uniform();

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.uniform.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(
                        &texture.create_view(&TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        self.bind_group = Some(bind_group);

        Ok(())
    }

    fn write_uniform(&self) {
        self.queue
            .write_buffer(&self.uniform, 0, bytes_of(&self.uniform_data));
    }

    fn resize(&mut self, size: (u32, u32)) {
        self.size = size;
        self.reconfigure_surface();
    }

    fn reconfigure_surface(&mut self) {
        info!(
            "Configure surface: width {}, height {}",
            self.size.0, self.size.1
        );
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

    fn render(&mut self, pre_present_op: impl FnOnce()) -> Result<(), wgpu::SurfaceError> {
        if self.bind_group.is_none() {
            // The image texture is not present. Skip this rendering.l
            return Ok(());
        }

        self.uniform_data.out_size = self.size.into();

        // let subsec = Local::now().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
        // let offset = subsec * 1000000.0;
        // self.uniform_data.uv_offset = [offset as u32 as f32, offset as u32 as f32];
        self.write_uniform();

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
