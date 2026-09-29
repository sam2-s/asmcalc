package dev.nga.asmcalc

import android.app.Activity
import android.os.Bundle
import android.view.KeyEvent
import android.view.View
import android.widget.TextView
import dev.nga.asmcalc.CalcNative.Angles
import dev.nga.asmcalc.CalcNative.Engine
import dev.nga.asmcalc.CalcNative.Key
import dev.nga.asmcalc.CalcNative.Memory

class MainActivity : Activity() {

    private var state = 0L

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        state = CalcNative.nativeCreate()
        display = findViewById(R.id.display)

        bind(R.id.key_0, Key.DIGIT_0)
        bind(R.id.key_1, Key.DIGIT_0 + 1)
        bind(R.id.key_2, Key.DIGIT_0 + 2)
        bind(R.id.key_3, Key.DIGIT_0 + 3)
        bind(R.id.key_4, Key.DIGIT_0 + 4)
        bind(R.id.key_5, Key.DIGIT_0 + 5)
        bind(R.id.key_6, Key.DIGIT_0 + 6)
        bind(R.id.key_7, Key.DIGIT_0 + 7)
        bind(R.id.key_8, Key.DIGIT_0 + 8)
        bind(R.id.key_9, Key.DIGIT_0 + 9)
        bind(R.id.key_dot, Key.DOT)
        bind(R.id.key_equals, Key.EQUALS)
        bind(R.id.key_plus, Key.ADD)
        bind(R.id.key_minus, Key.SUBTRACT)
        bind(R.id.key_multiply, Key.MULTIPLY)
        bind(R.id.key_divide, Key.DIVIDE)
        bind(R.id.key_clear, Key.CLEAR)
        bind(R.id.key_sign, Key.SIGN)

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
        CalcNative.nativePress(state, keycode)
        refresh()
    }

    private fun refresh() {
        display.text = CalcNative.nativeDisplay(state)
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        val mapped = when (keyCode) {
            in KeyEvent.KEYCODE_0..KeyEvent.KEYCODE_9 -> keyCode - KeyEvent.KEYCODE_0
            KeyEvent.KEYCODE_PERIOD, KeyEvent.KEYCODE_NUMPAD_DOT -> Key.DOT
            KeyEvent.KEYCODE_EQUALS, KeyEvent.KEYCODE_NUMPAD_ENTER -> Key.EQUALS
            KeyEvent.KEYCODE_PLUS, KeyEvent.KEYCODE_NUMPAD_ADD -> Key.ADD
            KeyEvent.KEYCODE_MINUS, KeyEvent.KEYCODE_NUMPAD_SUBTRACT -> Key.SUBTRACT
            KeyEvent.KEYCODE_STAR, KeyEvent.KEYCODE_NUMPAD_MULTIPLY -> Key.MULTIPLY
            KeyEvent.KEYCODE_SLASH, KeyEvent.KEYCODE_NUMPAD_DIVIDE -> Key.DIVIDE
            KeyEvent.KEYCODE_CLEAR, KeyEvent.KEYCODE_DEL -> Key.CLEAR
            else -> return super.onKeyDown(keyCode, event)
        }
        press(mapped)
        return true
    }
}
