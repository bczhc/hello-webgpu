#![feature(decl_macro)]
#![feature(try_blocks)]

mod compute_demo;
mod hello_triangle;
mod sha256_miner;

use jni::objects::{JClass, JObject};
use jni::sys::{jint, jstring};
use jni::JNIEnv;
use log::{debug, error, info, LevelFilter};
use raw_window_handle::{
    AndroidNdkWindowHandle, DisplayHandle, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};
use std::ptr::NonNull;
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
            .with_max_level(LevelFilter::Trace),
    );
    info!("Android logger initialized.");
}

/// 模拟一个持有 NativeWindow 的结构体
pub struct AndroidWindow {
    native_window: *mut ndk_sys::ANativeWindow,
    width: u32,
    height: u32,
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

unsafe impl Sync for AndroidWindow {}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_initWgpu(
    env: JNIEnv,
    _c: JClass,
    surface: JObject,
) {
    debug!("initWgpu called");

    unsafe {
        // 1. 获取原生窗口指针
        // 注意：这里需要确保 ndk_sys 的版本与你的 NDK 对应
        let window_ptr =
            ndk_sys::ANativeWindow_fromSurface(env.get_native_interface(), surface.as_raw());
        let width = ndk_sys::ANativeWindow_getWidth(window_ptr);
        let height = ndk_sys::ANativeWindow_getHeight(window_ptr);

        if window_ptr.is_null() {
            error!("window_ptr is null");
            return; // 或者抛出 Java 异常
        }

        // 2. 封装为 wgpu 可识别的 Surface
        // wgpu 现在通常配合 raw-window-handle 使用
        // 你需要保存这个 window_ptr，并在之后使用 wgpu::Instance::create_surface

        let android_window = AndroidWindow {
            native_window: window_ptr,
            width: width as _,
            height: height as _,
        };

        pollster::block_on(async {
            hello_triangle::show(android_window).await.unwrap();
        });

        // 这里的逻辑通常是：
        // a. 将 window_ptr 封装进一个自定义的结构体
        // b. 使用 wgpu::SurfaceTarget::from_native_and_custom_window 创建 surface

        // ⚠️ 重要：当你不再需要 surface 时，必须执行：
        // ndk_sys::ANativeWindow_release(window_ptr);
    }
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_resize(
    env: JNIEnv,
    _c: JClass,
    width: jint,
    height: jint,
) {
    debug!("resize called");
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_cleanup(env: JNIEnv, _c: JClass) {
    debug!("cleanup called");
}

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
