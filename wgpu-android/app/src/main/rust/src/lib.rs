#![feature(decl_macro)]
#![feature(try_blocks)]

mod compute_demo;

use jni::objects::{JClass, JObject};
use jni::sys::jint;
use jni::JNIEnv;
use log::{debug, info, LevelFilter};

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

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_initWgpu(
    env: JNIEnv,
    _c: JClass,
    surface: JObject,
) {
    debug!("initWgpu called");

    let result = compute_demo::compute();
    let result = pollster::block_on(result).unwrap();
    info!("wgpu result: {:?}", result);
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
