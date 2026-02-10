use crate::{default, AndroidWindow};
use bytemuck::cast_slice;
use log::debug;
use wgpu::{include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferBinding, BufferDescriptor, BufferSlice, BufferUsages, Color, ColorTargetState, Device, FragmentState, Instance, LoadOp, Operations, PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, StoreOp, Surface, SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState};
use wgpu::util::RenderEncoder;

pub struct State {
    surface: Surface<'static>,
    pipeline: RenderPipeline,
    texture_format: TextureFormat,
    device: Device,
    queue: Queue,
    size: (u32, u32),
    uniform: Buffer,
    bind_group: BindGroup,
    pub vertex_buffer: Buffer,
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
    pub async fn new(window: AndroidWindow) -> anyhow::Result<Self> {
        let instance = Instance::default();

        let window_size = (window.width, window.height);

        let adapter = instance.request_adapter(&default!()).await?;
        let (device, queue) = adapter.request_device(&default!()).await?;

        let surface = instance.create_surface(window)?;
        let cap = surface.get_capabilities(&adapter);
        let texture_format = cap.formats[0];

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: VERTICES_DATA.len() as u64 * 4,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex_buffer, 0, cast_slice(&VERTICES_DATA));

        let shader_module = device.create_shader_module(include_wgsl!("./hello_triangle.wgl"));
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: VertexState {
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
                entry_point: None,
                module: &shader_module,
                compilation_options: default!(),
            },
            primitive: default!(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: texture_format,
                    blend: None,
                    write_mask: default!(),
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let uniform = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 1 * 4,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &uniform,
                    size: None,
                    offset: 0,
                }),
            }],
        });

        let state = Self {
            uniform,
            pipeline,
            texture_format,
            surface,
            device,
            queue,
            size: window_size,
            bind_group,
            vertex_buffer,
        };
        state.configure_surface();
        Ok(state)
    }

    pub fn configure_surface(&self) {
        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: self.texture_format,
                width: self.size.0,
                height: self.size.1,
                present_mode: PresentMode::AutoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: Default::default(),
                view_formats: vec![self.texture_format.add_srgb_suffix()],
            },
        );
    }

    pub fn update_uniform(&self, value: f32) {
        self.queue
            .write_buffer(&self.uniform, 0, cast_slice(&[value]));
    }

    pub fn update_size(&mut self, window_size: (u32, u32)) {
        self.size = window_size;
    }

    pub fn render(&self) -> anyhow::Result<()> {
        let surface_texture = self.surface.get_current_texture()?;
        let view = surface_texture.texture.create_view(&TextureViewDescriptor {
            format: Some(self.texture_format.add_srgb_suffix()),
            ..default!()
        });

        let mut encoder = self.device.create_command_encoder(&default!());
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::WHITE),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);

        drop(pass);
        let command_buffer = encoder.finish();

        self.queue.submit([command_buffer]);
        surface_texture.present();
        Ok(())
    }
}

pub mod jni_exports {
    use crate::hello_triangle::State;
    use crate::{default, AndroidWindow};
    use jni::objects::{JClass, JObject};
    use jni::sys::{jfloat, jint};
    use jni::JNIEnv;
    use log::{error, info};
    use once_cell::sync::Lazy;
    use std::sync::Mutex;
    use std::thread::spawn;
    use std::time::Instant;

    static STATE: Lazy<Mutex<Option<State>>> = Lazy::new(|| default!());

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_initWgpu(
        env: JNIEnv,
        _c: JClass,
        surface: JObject,
    ) {
        info!("initWgpu called");

        unsafe {
            let window_ptr =
                ndk_sys::ANativeWindow_fromSurface(env.get_native_interface(), surface.as_raw());
            let width = ndk_sys::ANativeWindow_getWidth(window_ptr);
            let height = ndk_sys::ANativeWindow_getHeight(window_ptr);

            if window_ptr.is_null() {
                error!("window_ptr is null");
                return; // 或者抛出 Java 异常
            }

            let android_window = AndroidWindow {
                native_window: window_ptr,
                width: width as _,
                height: height as _,
            };

            pollster::block_on(async {
                let result: anyhow::Result<()> = try {
                    let state = State::new(android_window).await?;
                    *STATE.lock().unwrap() = Some(state);
                };
                if let Err(e) = result {
                    error!("JNI error: {:?}", e);
                    return;
                }
            });
        }
    }

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_resize(
        _env: JNIEnv,
        _c: JClass,
        width: jint,
        height: jint,
    ) {
        info!("resize called");
        let mut guard = STATE.lock().unwrap();
        let state = guard.as_mut().unwrap();
        state.update_size((width as _, height as _));
        state.configure_surface();
    }

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_cleanup(
        _env: JNIEnv,
        _c: JClass,
    ) {
        info!("cleanup called");
        let mut guard = STATE.lock().unwrap();
        *guard = None;
    }

    // #[unsafe(no_mangle)]
    // #[allow(non_snake_case)]
    // pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_frame(env: JNIEnv, _c: JClass) {
    //     info!("update called");
    //     let guard = STATE.lock().unwrap();
    //     let state = guard.as_ref().unwrap();
    //     state.render().unwrap();
    // }

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_startAnimationThread(
        _env: JNIEnv,
        _c: JClass,
        increment: jfloat,
    ) {
        info!("startAnimationThread called");
        spawn(move || {
            let mut elapsed = 0_f32;
            loop {
                let guard = STATE.lock().unwrap();
                let Some(state) = guard.as_ref() else {
                    break;
                };

                elapsed += increment;
                state.update_uniform(elapsed);
                state.render().unwrap();
            }
        });
    }
}
