package pers.zhc.android.myapplication

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import pers.zhc.android.myapplication.databinding.ActivityMainBinding

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val bindings = ActivityMainBinding.inflate(layoutInflater)

        setContentView(bindings.root)
    }
}
