package pers.zhc.android.myapplication

import android.view.Surface

object JNI {
    init {
        System.loadLibrary("app_jni")

        initLogger()
    }

    external fun initLogger()
    external fun initWgpu(surface: Surface)
    external fun resize(width: Int, height: Int)
    external fun cleanup()
}
