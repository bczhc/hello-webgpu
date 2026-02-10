package pers.zhc.android.myapplication

import android.os.Bundle
import android.view.SurfaceHolder
import androidx.appcompat.app.AppCompatActivity
import pers.zhc.android.myapplication.databinding.ActivityMainBinding

class MainActivity : AppCompatActivity(), SurfaceHolder.Callback {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val bindings = ActivityMainBinding.inflate(layoutInflater).also { setContentView(it.root) }

        bindings.surfaceView.holder.addCallback(this)
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        val surface = holder.surface
        JNI.initWgpu(surface)
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
    }
}
