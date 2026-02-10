#![feature(try_blocks)]

use std::env;
use std::sync::Arc;
use wgpu_playground::vsbm::State;
use wgpu_playground::{wgpu_instance_with_env_backend, FpsCounter, WgpuStateInitInfo};
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};
#[derive(Default)]
struct App {
    state: Option<State>,
    window: Option<Arc<Window>>,
    fps_counter: Option<FpsCounter>,
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
                let size = window.inner_size();
                let instance = wgpu_instance_with_env_backend();
                let surface = instance.create_surface(Arc::clone(&window))?;
                let state = State::new(WgpuStateInitInfo {
                    instance,
                    size: size.into(),
                    surface,
                })
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
            WindowEvent::Resized(physical_size) => state.resize(physical_size.into()),
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

                // calculate the FPS
                if let Some(f) = &mut self.fps_counter {
                    let (d, fps) = f.hint_and_get();
                    if d.as_secs_f64() > 1.0 {
                        println!("FPS: {}", fps);
                        self.fps_counter = Some(FpsCounter::new());
                    }
                } else {
                    self.fps_counter = Some(FpsCounter::new());
                }
                
                w.request_redraw();
            }
            _ => {}
        }
    }
}

pub fn main() {
    unsafe {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
