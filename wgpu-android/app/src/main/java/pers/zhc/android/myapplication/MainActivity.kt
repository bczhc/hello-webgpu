package pers.zhc.android.myapplication

import android.content.Intent
import android.os.Bundle
import android.view.Choreographer
import android.view.SurfaceHolder
import android.widget.EditText
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import pers.zhc.android.myapplication.databinding.ActivityMainBinding

class MainActivity : AppCompatActivity(), SurfaceHolder.Callback {
    private lateinit var tvFps: TextView
    private lateinit var appendLog: (line: String) -> Unit

    // address of the underlying JNI object
    private var addr: Long = 0

    private val defaultAnimation = JNI.Animations.ROTATING_TRIANGLE

    private var lastFrameTimeNanos: Long = 0

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val bindings = ActivityMainBinding.inflate(layoutInflater).also { setContentView(it.root) }
        appendLog = { line: String ->
            bindings.logTv.apply {
                append(line)
                append("\n")
            }
        }

        appendLog("simpleCompute result: ${JNI.simpleCompute()}")

        bindings.surfaceView.holder.addCallback(this)

        bindings.sha256MinerBtn.setOnClickListener {
            startActivity(Intent(this, Sha256MinerActivity::class.java))
        }
        tvFps = bindings.tvFps
        bindings.btnSelectAnimation.setOnClickListener {
            val animations = JNI.Animations.entries.toTypedArray()
            val items = animations.map { it.name }.toTypedArray()

            MaterialAlertDialogBuilder(this)
                .setTitle("选择动画模式")
                .setItems(items) { d, which ->
                    val selected = animations[which]
                    val oldAddr = addr
                    // set addr=0 to pause Choreographer's rendering
                    addr = 0
                    // then change animation
                    val newAddr = JNI.changeAnimation(oldAddr, selected.id)
                    addr = newAddr
                }
                .show()
        }
        bindings.fullscreenBtn.setOnClickListener {
            // 1. 创建输入框
            val input = EditText(this).apply {
                typeface = android.graphics.Typeface.MONOSPACE
                gravity = android.view.Gravity.TOP
                background = null // 去掉自带的下划线，让样式更整洁
                hint = "请输入代码..."
                // 允许输入框内部滚动
                isVerticalScrollBarEnabled = true
            }

// 2. 将输入框包裹在 NestedScrollView 中
            val scrollView = androidx.core.widget.NestedScrollView(this).apply {
                // 限制最大高度（例如 300dp），防止它无限撑大
                val maxPx = (300 * resources.displayMetrics.density).toInt()
                layoutParams = android.widget.FrameLayout.LayoutParams(
                    android.view.ViewGroup.LayoutParams.MATCH_PARENT,
                    maxPx
                )
                setPadding(60, 20, 60, 20) // 设置合适的内边距
                addView(input)
            }

// 3. 构建弹窗
            MaterialAlertDialogBuilder(this)
                .setTitle("输入代码")
                .setView(scrollView) // 这里传入的是带滚动的容器
                .setPositiveButton("确认") { _, _ ->
                    val codeText = input.text.toString()
                    if (codeText.isNotBlank()) {
                        val intent = Intent(this, Shadertoy::class.java).apply {
                            putExtra(Shadertoy.EXTRA_KEY, Shadertoy.ExtraInfo(codeText))
                        }
                        startActivity(intent)
                    }
                }
                .setNegativeButton("取消", null)
                .show()
        }
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        val surface = holder.surface
        addr = JNI.initWgpu(surface, defaultAnimation.id, null)

        Choreographer.getInstance().postFrameCallback(object : Choreographer.FrameCallback {
            override fun doFrame(frameTimeNanos: Long) {
                if (addr != 0L) {
                    JNI.frame(addr)

                    if (lastFrameTimeNanos != 0L) {
                        val diffNanos = frameTimeNanos - lastFrameTimeNanos
                        // 1s = 1,000,000,000ns
                        val fps = 1_000_000_000.0 / diffNanos
                        tvFps.text = String.format("FPS: %.1f", fps)
                    }
                    lastFrameTimeNanos = frameTimeNanos
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
}
