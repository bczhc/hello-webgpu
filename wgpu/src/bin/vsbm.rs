#![feature(try_blocks)]

//! https://cznull.github.io/vsbm wgpu port
//!
//! At 1024x1024 surface dimension, DX12 on Windows 10 has ~5 fps higher than
//! Vulkan on Windows 10 & Vulkan on Linux. Test hardware: NVIDIA GeForce RTX 3060 Mobile / Max-Q.

use chrono::Local;
use clap::Parser;
use std::env;
use std::sync::Arc;
use wgpu_playground::vsbm::{Config, State};
use wgpu_playground::{default, wgpu_instance_with_env_backend, WgpuStateInitInfo};
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short = 'i', long, default_value = "5")]
    kernel_iterations: u32,
}

#[derive(Default)]
struct App {
    pub state: Option<State>,
    pub window: Option<Arc<Window>>,
    pub last_frame_time: u64,
    pub animation_config: Config,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        pollster::block_on(async {
            let result: anyhow::Result<()> = try {
                // let size = window.inner_size();
                let size = (1024, 1024);
                let instance = wgpu_instance_with_env_backend();
                let surface = instance.create_surface(Arc::clone(&window))?;
                let state = State::new(
                    WgpuStateInitInfo {
                        instance,
                        size,
                        surface,
                    },
                    Config {
                        kernel_iterations: self.animation_config.kernel_iterations,
                    },
                )
                .await;
                self.state = Some(state);
            };
            result
        })
        .unwrap();

        window.request_redraw();
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => state.resize((1024, 1024)),
            WindowEvent::RedrawRequested => {
                let Some(w) = &self.window else {
                    return;
                };

                state.update();
                match state.render(|| w.pre_present_notify()) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    Err(e) => eprintln!("{:?}", e),
                }

                // print the FPS
                let ts = Local::now().timestamp_nanos_opt().unwrap() as u64;
                let fps = 1_000_000_000.0 / (ts - self.last_frame_time) as f64;
                println!("FPS: {:.2}", fps.floor());
                self.last_frame_time = ts;

                w.request_redraw();
            }
            _ => {}
        }
    }
}

pub fn main() {
    let args = Args::parse();

    unsafe {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        animation_config: Config {
            kernel_iterations: args.kernel_iterations,
        },
        ..default!()
    };
    event_loop.run_app(&mut app).unwrap();
}
