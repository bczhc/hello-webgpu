use bytemuck::cast_slice;
use bytemuck::checked::cast_slice_mut;
use wgpu::wgt::PollType;
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer,
    BufferBinding, BufferDescriptor, BufferUsages, ComputePipeline, ComputePipelineDescriptor, Device,
    Instance, MapMode, PipelineCompilationOptions, Queue,
};
use crate::default;

const WORKGROUP_SIZE: usize = 1;

struct State {
    device: Device,
    queue: Queue,
    pipeline: ComputePipeline,
    work_buffer: Buffer,
    result_buffer: Buffer,
    bind_group: BindGroup,
}

impl State {
    async fn new(work_buffer_size: u64) -> anyhow::Result<Self> {
        let instance = Instance::default();
        let adapter = instance.request_adapter(&default!()).await?;
        let (device, queue) = adapter.request_device(&default!()).await?;

        let shader_module = device.create_shader_module(include_wgsl!("compute_demo.wgsl"));
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &shader_module,
            entry_point: None,
            compilation_options: PipelineCompilationOptions {
                constants: &[("WORKGROUP_SIZE", WORKGROUP_SIZE as f64)],
                zero_initialize_workgroup_memory: false,
            },
            cache: None,
        });

        let work_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: work_buffer_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let result_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: work_buffer_size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &work_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Ok(Self {
            queue,
            device,
            pipeline,
            work_buffer,
            bind_group,
            result_buffer,
        })
    }

    fn write_work_buffer(&self, buf: &[u8]) {
        self.queue.write_buffer(&self.work_buffer, 0, buf);
    }

    fn compute_dispatch(&self, workgroups: (u32, u32, u32)) {
        let mut encoder = self.device.create_command_encoder(&default!());

        let mut pass = encoder.begin_compute_pass(&default!());
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, default!());
        pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
        drop(pass);

        encoder.copy_buffer_to_buffer(&self.work_buffer, 0, &self.result_buffer, 0, None);

        let command_buffer = encoder.finish();
        self.queue.submit([command_buffer]);
    }

    async fn read_result(&self, to: &mut [u8]) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.result_buffer.map_async(MapMode::Read, .., |e| {
            tx.send(e).unwrap();
        });
        self.device.poll(PollType::Wait {
            submission_index: None,
            timeout: None,
        })?;
        rx.await??;

        to[..(self.result_buffer.size() as usize)]
            .copy_from_slice(cast_slice(&*self.result_buffer.get_mapped_range(..)));
        self.result_buffer.unmap();
        Ok(())
    }
}

pub async fn compute() -> anyhow::Result<[u32; 3]> {
    let input = [1_u32, 2, 3];
    let mut result = [0_u32; 3];
    let state = State::new(3 * 4).await?;
    state.write_work_buffer(cast_slice(&input));
    state.compute_dispatch((3, 1, 1));
    state.read_result(cast_slice_mut(&mut result)).await?;
    Ok(result)
}

pub mod jni_exports {
    use jni::JNIEnv;
    use jni::objects::JClass;
    use jni::sys::jstring;
    use crate::compute_demo;

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_simpleCompute(
        env: JNIEnv,
        _c: JClass,
    ) -> jstring {
        let result = compute_demo::compute();
        let result = pollster::block_on(result).unwrap();
        env.new_string(format!("{:?}", result)).unwrap().into_raw()
    }
}
