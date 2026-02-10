#![feature(decl_macro)]

pub mod triangle_rotation;

use std::env;
use wgpu::{Backends, Color, Instance, InstanceDescriptor};

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
    let instance = Instance::new(&InstanceDescriptor {
        backends: Backends::from_env().unwrap_or(default!()),
        ..default!()
    });
    instance
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
