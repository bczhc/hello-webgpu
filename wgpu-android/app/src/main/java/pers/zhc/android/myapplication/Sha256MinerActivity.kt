package pers.zhc.android.myapplication

import android.os.Bundle
import android.widget.ScrollView
import androidx.appcompat.app.AppCompatActivity
import pers.zhc.android.myapplication.databinding.Sha256MinerActivityMainBinding
class Sha256MinerActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val bindings = Sha256MinerActivityMainBinding.inflate(layoutInflater).also {
            setContentView(it.root)
        }

        val appendLog = { line: String ->
            bindings.tvConsoleLog.let {
                it.append(line)
                it.append("\n")
            }
            bindings.scrollView.post {
                bindings.scrollView.fullScroll(ScrollView.FOCUS_DOWN)
            }
        }

        bindings.btnStartMining.setOnClickListener {
            bindings.tvConsoleLog.setText("")
            appendLog("开始计算……")
            bindings.btnStartMining.isEnabled = false
            Thread {
                JNI.sha256Demo(
                    bindings.etWorkgroupSize.text!!.toString().toInt(),
                    bindings.etDispatchX.text!!.toString().toInt(),
                    bindings.etIterations.text!!.toString().toInt(),
                    bindings.etDifficulty.text!!.toString().toInt(),
                    object : JNI.LogCallback() {
                        override fun print(line: String) {
                            runOnUiThread {
                                appendLog(line)
                                appendLog("")
                            }
                        }
                    }
                )
                runOnUiThread {
                    bindings.btnStartMining.isEnabled = true
                    appendLog("----------------")
                }
            }.start()
        }
    }
}
