use std::env;
use std::sync::Arc;
use std::time::Instant;
use wgpu_playground::triangle_rotation::State;
use wgpu_playground::wgpu_instance_with_env_backend;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct App {
    state: Option<State>,
    window: Option<Arc<Window>>,
    start: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            window: None,
            state: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let wgpu_instance = wgpu_instance_with_env_backend();
        let surface = wgpu_instance.create_surface(Arc::clone(&window)).unwrap();
        let size = window.inner_size();

        pollster::block_on(async {
            let state = State::new(wgpu_instance, surface, (size.width, size.height)).await;
            self.state = Some(state);
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
                    state.update_elapsed(self.start.elapsed().as_secs_f32());
                    state.render(|| {
                        w.pre_present_notify();
                    });
                    w.request_redraw();
                }
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize((size.width, size.height));
            }
            WindowEvent::MouseInput {
                state: e_state,
                button,
                ..
            } => {
                // if e_state == ElementState::Pressed && button == MouseButton::Left {
                //     state.render();
                // }
            }
            _ => {}
        }
    }
}

fn main() {
    // wgpu uses `log` for all of our logging, so we initialize a logger with the `env_logger` crate.
    //
    // To change the log level, set the `RUST_LOG` environment variable. See the `env_logger`
    // documentation for more information.
    unsafe {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    // When the current loop iteration finishes, immediately begin a new
    // iteration regardless of whether or not new events are available to
    // process. Preferred for applications that want to render as fast as
    // possible, like games.
    event_loop.set_control_flow(ControlFlow::Poll);

    // When the current loop iteration finishes, suspend the thread until
    // another event arrives. Helps keeping CPU utilization low if nothing
    // is happening, which is preferred if the application might be idling in
    // the background.
    // event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
