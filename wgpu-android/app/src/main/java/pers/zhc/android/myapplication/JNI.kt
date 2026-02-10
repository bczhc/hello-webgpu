package pers.zhc.android.myapplication

import android.view.Surface

object JNI {
    init {
        System.loadLibrary("app_jni")

        initLogger()
    }

    private external fun initLogger()
    external fun initWgpu(surface: Surface, animationId: Int): Long
    external fun resize(addr: Long, width: Int, height: Int)
    external fun cleanup(addr: Long)
    external fun frame(addr: Long)
    external fun changeAnimation(addr: Long, animationId: Int): Long

    enum class Animations(val id: Int) {
        ROTATING_TRIANGLE(0),
        VSBM(1),
    }

    external fun simpleCompute(): String

    external fun sha256Demo(
        workgroupSize: Int,
        dispatchX: Int,
        iterations: Int,
        difficulty: Int,
        logCallback: LogCallback,
    )

    abstract class LogCallback {
        abstract fun print(line: String)
    }
}
