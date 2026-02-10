#![feature(decl_macro)]
#![feature(try_blocks)]

mod compute_demo;
mod hello_triangle;
mod sha256_miner;

use jni::objects::{JClass, JObject};
use jni::sys::{jint, jstring};
use jni::JNIEnv;
use log::{debug, error, info, LevelFilter};
use once_cell::sync::Lazy;
use raw_window_handle::{
    AndroidNdkWindowHandle, DisplayHandle, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};
use std::ptr::NonNull;
use std::sync::Mutex;
use wgpu::{Instance, SurfaceTarget};

pub macro default() {
    Default::default()
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_initLogger(
    _env: JNIEnv,
    _c: JClass,
) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("Android wgpu demo")
            .with_max_level(LevelFilter::Info),
    );
    info!("Android logger initialized.");
}

pub struct AndroidWindow {
    native_window: *mut ndk_sys::ANativeWindow,
    size: (u32 ,u32),
}

impl HasWindowHandle for AndroidWindow {
    fn window_handle(&self) -> Result<WindowHandle, raw_window_handle::HandleError> {
        let mut handle = AndroidNdkWindowHandle::new(
            NonNull::new(self.native_window as *mut _).expect("Window handle is null"),
        );
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::AndroidNdk(handle)) })
    }
}

impl HasDisplayHandle for AndroidWindow {
    fn display_handle(&self) -> Result<DisplayHandle, raw_window_handle::HandleError> {
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawDisplayHandle::Android(
                raw_window_handle::AndroidDisplayHandle::new(),
            ))
        })
    }
}

unsafe impl Send for AndroidWindow {}
unsafe impl Sync for AndroidWindow {}
