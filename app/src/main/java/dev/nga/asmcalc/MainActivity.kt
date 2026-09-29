package dev.nga.asmcalc

import android.app.Activity
import android.os.Bundle
import android.view.KeyEvent
import android.view.View
import android.widget.TextView

class MainActivity : Activity() {

    private var state = 0L

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        state = CalcNative.nativeCreate()
        display = findViewById(R.id.display)

        bind(R.id.key_0, CalcKey.DIGIT_0)
        bind(R.id.key_1, CalcKey.DIGIT_1)
        bind(R.id.key_2, CalcKey.DIGIT_2)
        bind(R.id.key_3, CalcKey.DIGIT_3)
        bind(R.id.key_4, CalcKey.DIGIT_4)
        bind(R.id.key_5, CalcKey.DIGIT_5)
        bind(R.id.key_6, CalcKey.DIGIT_6)
        bind(R.id.key_7, CalcKey.DIGIT_7)
        bind(R.id.key_8, CalcKey.DIGIT_8)
        bind(R.id.key_9, CalcKey.DIGIT_9)
        bind(R.id.key_dot, CalcKey.DOT)
        bind(R.id.key_equals, CalcKey.EQUALS)
        bind(R.id.key_plus, CalcKey.ADD)
        bind(R.id.key_minus, CalcKey.SUBTRACT)
        bind(R.id.key_multiply, CalcKey.MULTIPLY)
        bind(R.id.key_divide, CalcKey.DIVIDE)
        bind(R.id.key_clear, CalcKey.CLEAR)
        bind(R.id.key_sign, CalcKey.SIGN)

        refresh()
    }

    private lateinit var display: TextView

    override fun onDestroy() {
        CalcNative.nativeDestroy(state)
        state = 0
        super.onDestroy()
    }

    private fun bind(viewId: Int, keycode: Int) {
        findViewById<View>(viewId).setOnClickListener { press(keycode) }
    }

    private fun press(keycode: Int) {
        CalcNative.nativeKey(state, keycode)
        refresh()
    }

    private fun refresh() {
        display.text = CalcNative.nativeFormat(state)
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        val mapped = when (keyCode) {
            in KeyEvent.KEYCODE_0..KeyEvent.KEYCODE_9 -> keyCode - KeyEvent.KEYCODE_0
            KeyEvent.KEYCODE_PERIOD, KeyEvent.KEYCODE_NUMPAD_DOT -> CalcKey.DOT
            KeyEvent.KEYCODE_EQUALS, KeyEvent.KEYCODE_NUMPAD_ENTER -> CalcKey.EQUALS
            KeyEvent.KEYCODE_PLUS, KeyEvent.KEYCODE_NUMPAD_ADD -> CalcKey.ADD
            KeyEvent.KEYCODE_MINUS, KeyEvent.KEYCODE_NUMPAD_SUBTRACT -> CalcKey.SUBTRACT
            KeyEvent.KEYCODE_STAR, KeyEvent.KEYCODE_NUMPAD_MULTIPLY -> CalcKey.MULTIPLY
            KeyEvent.KEYCODE_SLASH, KeyEvent.KEYCODE_NUMPAD_DIVIDE -> CalcKey.DIVIDE
            KeyEvent.KEYCODE_CLEAR, KeyEvent.KEYCODE_DEL -> CalcKey.CLEAR
            else -> return super.onKeyDown(keyCode, event)
        }
        press(mapped)
        return true
    }
}
