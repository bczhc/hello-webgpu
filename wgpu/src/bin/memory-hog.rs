use std::io::stdin;
use std::mem;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use wgpu::{BufferDescriptor, BufferUsages};
use wgpu_playground::wgpu_instance_with_env_backend;

fn main() -> anyhow::Result<()> {
    let instance = wgpu_instance_with_env_backend();
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default());
    let adapter = pollster::block_on(adapter)?;
    let x = adapter.request_device(&wgpu::DeviceDescriptor::default());
    let (device, queue) = pollster::block_on(x)?;

    let error_handler = |e: wgpu::Error| {
        println!("{}", e);
        sleep(Duration::from_secs_f32(0.1f32));
    };

    device.on_uncaptured_error(Arc::new(error_handler));

    loop {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 1048576,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        queue.write_buffer(&buffer, 0, &[]);
        mem::forget(buffer);
    }
}
