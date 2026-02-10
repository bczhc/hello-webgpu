use crate::{default, AndroidWindow};
use wgpu::{include_wgsl, Color, ColorTargetState, FragmentState, Instance, LoadOp, Operations, PresentMode, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, StoreOp, SurfaceConfiguration, TextureUsages, TextureViewDescriptor, VertexState};

pub async fn show(window: AndroidWindow) -> anyhow::Result<()> {
    let instance = Instance::default();

    let adapter = instance.request_adapter(&default!()).await?;
    let (device, queue) = adapter.request_device(&default!()).await?;

    let surface = instance.create_surface(&window)?;
    let cap = surface.get_capabilities(&adapter);
    let texture_format = cap.formats[0];
    surface.configure(&device, &SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: texture_format,
        width: window.width,
        height: window.height,
        present_mode: PresentMode::AutoVsync,
        desired_maximum_frame_latency: 2,
        alpha_mode: Default::default(),
        view_formats: vec![texture_format.add_srgb_suffix()],
    });

    let shader_module = device.create_shader_module(include_wgsl!("./hello_triangle.wgl"));
    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: VertexState {
            buffers: &[],
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
            targets: &[
                Some(ColorTargetState {
                    format: texture_format,
                    blend: None,
                    write_mask: default!(),
                })
            ],
        }),
        multiview_mask: None,
        cache: None,
    });

    let surface_texture = surface.get_current_texture()?;
    let view = surface_texture.texture.create_view(&TextureViewDescriptor {
        format: Some(texture_format.add_srgb_suffix()),
        ..default!()
    });

    let mut encoder = device.create_command_encoder(&default!());
    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: None,
        color_attachments: &[
            Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::WHITE),
                    store: StoreOp::Store,
                },
            })
        ],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    pass.set_pipeline(&pipeline);
    pass.draw(0..3, 0..1);

    drop(pass);
    let command_buffer = encoder.finish();

    queue.submit([command_buffer]);
    surface_texture.present();

    Ok(())
}
