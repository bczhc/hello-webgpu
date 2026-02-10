#![feature(decl_macro)]
#![feature(try_blocks)]

mod compute_demo;
mod sha256_miner;

use jni::objects::{JClass, JObject};
use jni::sys::{jint, jstring};
use jni::JNIEnv;
use log::{debug, info, LevelFilter};

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

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_initWgpu(
    env: JNIEnv,
    _c: JClass,
    surface: JObject,
) {
    debug!("initWgpu called");
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
