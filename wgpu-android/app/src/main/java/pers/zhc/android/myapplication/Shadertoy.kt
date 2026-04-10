package pers.zhc.android.myapplication

import android.net.Uri
import android.os.Bundle
import android.view.Choreographer
import android.view.SurfaceHolder
import androidx.appcompat.app.AppCompatActivity
import pers.zhc.android.myapplication.databinding.FullscreenSurfaceBinding
import java.io.Serializable

class Shadertoy : AppCompatActivity(), SurfaceHolder.Callback {
    private var addr = 0L
    private lateinit var shadertoyCode: String

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val bindings = FullscreenSurfaceBinding.inflate(layoutInflater).also {
            setContentView(it.root)
        }

        val data: Uri? = intent.data
        if (data != null) {
            // 说明是从外部文件打开的
            val codeText = readTextFromUri(data)
            initShader(codeText)
        } else {
            // 说明是应用内跳转，走你原来的 Extra 逻辑
            val info = intent.getSerializableExtra(EXTRA_KEY) as ExtraInfo?
            info?.let { initShader(it.code) }
        }

        bindings.surfaceView.holder.addCallback(this)
    }

    private fun initShader(code: String) {
        shadertoyCode = code
    }

    private fun readTextFromUri(uri: Uri): String {
        return contentResolver.openInputStream(uri)?.use { inputStream ->
            inputStream.bufferedReader().use { it.readText() }
        } ?: ""
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        val surface = holder.surface
        addr = JNI.initWgpu(surface, JNI.Animations.SHADERTOY.id, shadertoyCode)

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
