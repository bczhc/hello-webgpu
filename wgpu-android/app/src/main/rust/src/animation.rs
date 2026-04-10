use wgpu::util::RenderEncoder;

pub mod jni_exports {
    use crate::animator::{Animate, RotatingTriangleAnimator, ShadertoyAnimator, VsbmAnimator};
    use crate::{animator, default, AndroidWindow};
    use jni::objects::{JClass, JObject, JString};
    use jni::sys::{jfloat, jint, jlong};
    use jni::JNIEnv;
    use log::{debug, error, info, trace};
    use std::sync::{Arc, Mutex};
    use wgpu::{BackendOptions, Backends, Instance, InstanceDescriptor};
    use wgpu_playground::WgpuStateInitInfo;

    struct Wrapper {
        animator: Box<dyn Animate>,
        window: Arc<AndroidWindow>,
    }

    fn create_init_info_from_window(android_window: Arc<AndroidWindow>) -> WgpuStateInitInfo {
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::VULKAN,
            ..default!()
        });
        let size = android_window.size;
        let surface = instance
            .create_surface(Arc::clone(&android_window))
            .unwrap();

        let init_info = WgpuStateInitInfo {
            instance,
            surface,
            size,
        };
        init_info
    }

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_initWgpu(
        mut env: JNIEnv,
        _c: JClass,
        surface: JObject,
        animation_id: jint,
        extra_code: JString,
    ) -> jlong {
        info!("initWgpu called");

        unsafe {
            let window_ptr =
                ndk_sys::ANativeWindow_fromSurface(env.get_native_interface(), surface.as_raw());
            let width = ndk_sys::ANativeWindow_getWidth(window_ptr);
            let height = ndk_sys::ANativeWindow_getHeight(window_ptr);
            // let size = (width as u32, height as u32);
            let size = (1024, 1024);
            info!("Surface size: {:?}", size);

            if window_ptr.is_null() {
                error!("window_ptr is null");
                return 0;
            }

            let android_window = AndroidWindow {
                native_window: window_ptr,
                size,
            };
            let android_window = Arc::new(android_window);

            let result = pollster::block_on(async {
                let result: anyhow::Result<jlong> = try {
                    let shadertoy_code = if extra_code.is_null() {
                        None
                    } else {
                        Some(
                            env.get_string(&extra_code)?
                                .to_str()
                                .expect("UTF-8 error")
                                .to_string(),
                        )
                    };

                    let init_info = create_init_info_from_window(Arc::clone(&android_window));
                    let animator = create_animator(
                        animation_id,
                        init_info,
                        shadertoy_code.as_ref().map(|x| x.as_str()),
                    )?;

                    let wrapper = Wrapper {
                        animator,
                        window: android_window,
                    };
                    Box::into_raw(Box::new(wrapper)) as jlong
                };
                result
            });
            result.unwrap_or_else(|e| {
                error!("JNI error: {:?}", e);
                0
            })
        }
    }

    fn create_animator(
        animation_id: jint,
        init_info: WgpuStateInitInfo,
        extra_code: Option<&str>,
    ) -> anyhow::Result<Box<dyn Animate>> {
        let animator: Box<dyn Animate> = match animation_id {
            0 => Box::new(RotatingTriangleAnimator::new(init_info)?),
            1 => Box::new(VsbmAnimator::new(init_info)?),
            2 => Box::new(ShadertoyAnimator::new(
                init_info,
                extra_code.expect("Code is missing"),
            )?),
            _ => {
                error!("Unknown animation id");
                panic!();
            }
        };
        Ok(animator)
    }

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_resize(
        _env: JNIEnv,
        _c: JClass,
        addr: jlong,
        width: jint,
        height: jint,
    ) {
        info!("resize called");
        let wrapper = unsafe { &mut *(addr as *mut Wrapper) };
        // wrapper.animator.resize((width as _, height as _)).unwrap();
    }

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_cleanup(
        _env: JNIEnv,
        _c: JClass,
        addr: jlong,
    ) {
        info!("cleanup called");
        unsafe {
            let wrapper = Box::from_raw(addr as *mut Wrapper);
            let animator = wrapper.animator;
            drop(animator);
            debug!("strong: {}", Arc::strong_count(&wrapper.window));
            let Ok(w) = Arc::try_unwrap(wrapper.window) else {
                panic!("Should have only one reference here");
            };
            ndk_sys::ANativeWindow_release(w.native_window);
        }
    }

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_frame(
        env: JNIEnv,
        _c: JClass,
        addr: jlong,
    ) {
        trace!("update called");
        let wrapper = unsafe { &mut *(addr as *mut Wrapper) };
        wrapper.animator.frame().unwrap();
    }

    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub extern "system" fn Java_pers_zhc_android_myapplication_JNI_changeAnimation(
        env: JNIEnv,
        _c: JClass,
        addr: jlong,
        animation_id: jint,
    ) -> jlong {
        info!("changeAnimation called");
        let wrapper = unsafe { Box::from_raw(addr as *mut Wrapper) };
        // destroy the old animator and take android_window out
        drop(wrapper.animator);
        let window_arc = wrapper.window;
        let init_info = create_init_info_from_window(Arc::clone(&window_arc));

        let animator = create_animator(animation_id, init_info, None).unwrap();
        let wrapper = Wrapper {
            window: window_arc,
            animator,
        };
        info!("changeAnimation end");
        Box::into_raw(Box::new(wrapper)) as jlong
    }
}
