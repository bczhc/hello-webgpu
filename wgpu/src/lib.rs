#![feature(decl_macro)]

pub mod triangle_rotation;
pub mod vsbm;

use std::env;
use std::time::{Duration, Instant};
use wgpu::{Backends, Color, Instance, Surface};

pub fn set_up_logger() {
    unsafe {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
}

pub macro default() {
    Default::default()
}

pub fn wgpu_instance_with_env_backend() -> Instance {
    let backends = Backends::from_env().unwrap_or(Backends::all());
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    desc.backends = backends;
    Instance::new(desc)
}

pub trait ColorExt {
    fn from_vec4d(x: [f64; 4]) -> Self;
}

impl ColorExt for Color {
    fn from_vec4d(x: [f64; 4]) -> Self {
        Self {
            r: x[0],
            g: x[1],
            b: x[2],
            a: x[3],
        }
    }
}

pub fn random_color() -> [f32; 3] {
    [
        rand::random::<f32>(),
        rand::random::<f32>(),
        rand::random::<f32>(),
    ]
}

pub struct WgpuStateInitInfo {
    pub instance: Instance,
    pub surface: Surface<'static>,
    pub size: (u32, u32),
}

pub struct FpsCounter {
    instant: Instant,
    counter: usize,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            instant: Instant::now(),
            counter: 0,
        }
    }

    pub fn hint_and_get(&mut self) -> (Duration, f32) {
        self.counter += 1;
        let duration = self.instant.elapsed();
        (
            duration,
            (self.counter as f64 / duration.as_secs_f64()) as f32,
        )
    }
}

#[cfg(not(target_os = "android"))]
pub mod winit_extensions {
    use winit::dpi::PhysicalSize;

    pub struct WindowSizeWrapper((u32, u32));

    impl From<PhysicalSize<u32>> for WindowSizeWrapper {
        fn from(value: PhysicalSize<u32>) -> Self {
            Self((value.width, value.height))
        }
    }
}
