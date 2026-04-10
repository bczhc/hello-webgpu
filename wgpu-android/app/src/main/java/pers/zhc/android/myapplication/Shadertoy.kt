package pers.zhc.android.myapplication

import android.os.Bundle
import android.view.Choreographer
import android.view.SurfaceHolder
import androidx.appcompat.app.AppCompatActivity
import pers.zhc.android.myapplication.databinding.FullscreenSurfaceBinding
import java.io.Serializable

class Shadertoy : AppCompatActivity(), SurfaceHolder.Callback {
    private var addr = 0L
    private lateinit var extraInfo: ExtraInfo

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val bindings = FullscreenSurfaceBinding.inflate(layoutInflater).also {
            setContentView(it.root)
        }

        extraInfo = intent.getSerializableExtra(EXTRA_KEY) as ExtraInfo

        bindings.surfaceView.holder.addCallback(this)
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        val surface = holder.surface
        addr = JNI.initWgpu(surface, JNI.Animations.SHADERTOY.id, extraInfo.code)

        Choreographer.getInstance().postFrameCallback(object : Choreographer.FrameCallback {
            override fun doFrame(frameTimeNanos: Long) {
                if (addr != 0L) {
                    JNI.frame(addr)
                }
                Choreographer.getInstance().postFrameCallback(this)
            }
        })
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        JNI.resize(addr, width, height)
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        JNI.cleanup(addr)
        addr = 0
    }

    data class ExtraInfo(
        val code: String,
    ) : Serializable

    companion object {
        const val EXTRA_KEY = "extra info"
    }
}
