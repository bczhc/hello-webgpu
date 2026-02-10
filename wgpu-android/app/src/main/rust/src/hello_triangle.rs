use wgpu::util::RenderEncoder;

pub mod jni_exports {
    use crate::{default, AndroidWindow};
    use jni::objects::{JClass, JObject};
    use jni::sys::{jfloat, jint};
    use jni::JNIEnv;
    use log::{error, info};
    use once_cell::sync::Lazy;
    use std::sync::{Arc, Mutex};
    use std::thread::spawn;
    use std::time::Instant;
    use wgpu::Instance;
    use wgpu_playground::triangle_rotation::State;

    static STATE: Lazy<Mutex<Option<State>>> = Lazy::new(|| default!());

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_initWgpu(
        env: JNIEnv,
        _c: JClass,
        surface: JObject,
    ) {
        info!("initWgpu called");

        unsafe {
            let window_ptr =
                ndk_sys::ANativeWindow_fromSurface(env.get_native_interface(), surface.as_raw());
            let width = ndk_sys::ANativeWindow_getWidth(window_ptr);
            let height = ndk_sys::ANativeWindow_getHeight(window_ptr);

            if window_ptr.is_null() {
                error!("window_ptr is null");
                return; // 或者抛出 Java 异常
            }

            let android_window = AndroidWindow {
                native_window: window_ptr,
                size: (width as _, height as _),
            };
            let android_window = Arc::new(android_window);

            pollster::block_on(async {
                let result: anyhow::Result<()> = try {
                    let instance = Instance::default();
                    let size = android_window.size;
                    let surface = instance.create_surface(android_window)?;
                    let state = State::new(instance, surface, size).await;
                    *STATE.lock().unwrap() = Some(state);
                };
                if let Err(e) = result {
                    error!("JNI error: {:?}", e);
                    return;
                }
            });
        }
    }

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_resize(
        _env: JNIEnv,
        _c: JClass,
        width: jint,
        height: jint,
    ) {
        info!("resize called");
        let mut guard = STATE.lock().unwrap();
        let state = guard.as_mut().unwrap();
        state.resize((width as _, height as _));
    }

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_cleanup(
        _env: JNIEnv,
        _c: JClass,
    ) {
        info!("cleanup called");
        let mut guard = STATE.lock().unwrap();
        *guard = None;
    }

    // #[unsafe(no_mangle)]
    // #[allow(non_snake_case)]
    // pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_frame(env: JNIEnv, _c: JClass) {
    //     info!("update called");
    //     let guard = STATE.lock().unwrap();
    //     let state = guard.as_ref().unwrap();
    //     state.render().unwrap();
    // }

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_startAnimationThread(
        _env: JNIEnv,
        _c: JClass,
        increment: jfloat,
    ) {
        info!("startAnimationThread called");
        spawn(move || {
            let mut elapsed = 0_f32;
            loop {
                let guard = STATE.lock().unwrap();
                let Some(state) = guard.as_ref() else {
                    break;
                };

                elapsed += increment;
                state.update_elapsed(elapsed);
                state.render(|| {});
            }
        });
    }
}
