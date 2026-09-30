package dev.nga.asmcalc

import android.app.Activity
import android.os.Bundle
import android.view.Gravity
import android.view.KeyEvent
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.TextView

/**
 * The calculator shell.
 *
 * Holds one native session and swaps the keypad underneath it. All arithmetic
 * happens in `libasmcalc.so`: either the AArch64 assembly kernel or the Rust
 * feature layer, depending on the tab.
 */
class MainActivity : Activity() {

    private lateinit var display: TextView
    private lateinit var expression: TextView
    private lateinit var status: TextView
    private lateinit var angleLabel: TextView

    private var handle = 0L
    private var mode = Mode.CALCULATOR

    /** Key looks, since the pads are built in code rather than from layout. */
    private object KeyLook {
        const val STANDARD = 0
        const val FUNCTION = 1
        const val MEMORY = 2
        const val EQUALS = 3
    }

    private enum class Mode(val tab: Int, val pad: Int, val native: Int) {
        CALCULATOR(R.id.tab_calculator, R.id.pad_calculator, CalcNative.Engine.FIXED),
        SCIENTIFIC(R.id.tab_scientific, R.id.pad_scientific, CalcNative.Engine.SCIENTIFIC),
        PROGRAMMER(R.id.tab_programmer, R.id.pad_programmer, CalcNative.Engine.PROGRAMMER),
        CONVERT(R.id.tab_convert, R.id.pad_convert, CalcNative.Engine.SCIENTIFIC),
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        handle = CalcNative.nativeCreate()

        display = findViewById(R.id.display)
        expression = findViewById(R.id.expression)
        status = findViewById(R.id.status)
        angleLabel = findViewById(R.id.angle_mode)

        buildCalculatorPad(findViewById(R.id.pad_calculator))
        buildScientificPad(findViewById(R.id.pad_scientific))
        buildProgrammerPad(findViewById(R.id.pad_programmer))
        buildConvertPad(findViewById(R.id.pad_convert))

        for (mode in Mode.values()) {
            findViewById<View>(mode.tab).setOnClickListener { select(mode) }
        }

        angleLabel.setOnClickListener { cycleAngleMode() }

        select(Mode.CALCULATOR)
    }

    override fun onDestroy() {
        CalcNative.nativeDestroy(handle)
        handle = 0
        super.onDestroy()
    }

    private fun select(mode: Mode) {
        this.mode = mode
        for (candidate in Mode.values()) {
            findViewById<View>(candidate.tab).isSelected = candidate == mode
            findViewById<View>(candidate.pad).visibility =
                if (candidate == mode) View.VISIBLE else View.GONE
        }
        CalcNative.nativeSetEngine(handle, mode.native)
        status.text = statusFor(mode)
        angleLabel.visibility = if (mode == Mode.CONVERT) View.GONE else View.VISIBLE
        refresh()
    }

    private fun statusFor(mode: Mode): String = when (mode) {
        Mode.CALCULATOR -> "AArch64 assembly kernel"
        Mode.SCIENTIFIC -> "Rust engine, f64"
        Mode.PROGRAMMER -> "Rust engine, integer"
        Mode.CONVERT -> "Rust engine, units"
    }

    private fun cycleAngleMode() {
        val next = when (CalcNative.nativeAngleMode(handle)) {
            "DEG" -> CalcNative.Angles.RADIANS
            "RAD" -> CalcNative.Angles.GRADIANS
            else -> CalcNative.Angles.DEGREES
        }
        CalcNative.nativeSetAngleMode(handle, next)
        refresh()
    }

    private fun refresh() {
        display.text = CalcNative.nativeDisplay(handle)
        expression.text = if (mode == Mode.PROGRAMMER) {
            CalcNative.nativeProgrammerBases(handle).replace("\\n", "   ")
        } else {
            CalcNative.nativeExpression(handle)
        }
        angleLabel.text = when (mode) {
            Mode.PROGRAMMER -> ""
            Mode.CONVERT -> ""
            else -> CalcNative.nativeAngleMode(handle)
        }
    }

    // --- keypad construction ----------------------------------------------

    private fun press(keycode: Int) {
        CalcNative.nativePress(handle, keycode)
        refresh()
    }

    /**
     * Build a keypad from rows of key specs.
     *
     * A spec is `text|label|keycode`, where the text is what the key shows and
     * the keycode is what the kernel receives. Labels differ from the key so the
     * pad can show `÷` while sending the divide keycode.
     */
    private fun buildPad(
        container: LinearLayout,
        rows: List<List<Pair<String, Int>>>,
        style: Int = KeyLook.STANDARD,
    ) {
        container.removeAllViews()
        for (row in rows) {
            val line = LinearLayout(this)
            line.orientation = LinearLayout.HORIZONTAL
            line.layoutParams = LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                0,
                1f,
            )
            for ((label, keycode) in row) {
                line.addView(makeKey(label, keycode, style))
            }
            container.addView(line)
        }
    }

    private fun makeKey(label: String, keycode: Int, style: Int): Button {
        val button = Button(this)
        button.text = label
        button.setTextColor(textColorFor(style))
        button.textSize = sizeFor(style)
        button.gravity = Gravity.CENTER
        button.isAllCaps = false
        button.setBackgroundResource(backgroundFor(style))
        button.layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.MATCH_PARENT, 1f)
        val margin = (5 * resources.displayMetrics.density).toInt()
        button.setPadding(margin, 0, margin, 0)
        button.setOnClickListener { onKey(label, keycode) }
        return button
    }

    /**
     * Route a key to whichever engine owns the current tab.
     *
     * Programmer mode has its own digit rules: the assembly kernel knows nothing
     * about number bases, so those keys go to the Rust programmer engine, which
     * rejects a digit the base cannot represent.
     */
    private fun onKey(label: String, keycode: Int) {
        when {
            mode == Mode.PROGRAMMER && keycode in 0..15 -> {
                when (keycode) {
                    -1 -> CalcNative.nativeProgrammerBackspace(handle)
                    -2 -> CalcNative.nativeProgrammerClear(handle)
                    else -> CalcNative.nativeProgrammerPush(handle, label)
                }
                refresh()
            }
            else -> press(keycode)
        }
    }

    private fun backgroundFor(style: Int): Int = when (style) {
        KeyLook.FUNCTION -> R.drawable.key_function_background
        KeyLook.MEMORY -> R.drawable.key_memory_background
        KeyLook.EQUALS -> R.drawable.key_equals_background
        else -> R.drawable.key_background
    }

    private fun textColorFor(style: Int): Int = when (style) {
        KeyLook.FUNCTION -> 0xFF9FD4FF.toInt()
        KeyLook.MEMORY -> 0xFFFFD08A.toInt()
        KeyLook.EQUALS -> 0xFFFFB300.toInt()
        else -> 0xFFFFFFFF.toInt()
    }

    private fun sizeFor(style: Int): Float = when (style) {
        KeyLook.FUNCTION -> 16f
        KeyLook.MEMORY -> 15f
        else -> 24f
    }

    private fun digits() = listOf(
        "7" to CalcNative.Key.DIGIT_0 + 7,
        "8" to CalcNative.Key.DIGIT_0 + 8,
        "9" to CalcNative.Key.DIGIT_0 + 9,
        "4" to CalcNative.Key.DIGIT_0 + 4,
        "5" to CalcNative.Key.DIGIT_0 + 5,
        "6" to CalcNative.Key.DIGIT_0 + 6,
        "1" to CalcNative.Key.DIGIT_0 + 1,
        "2" to CalcNative.Key.DIGIT_0 + 2,
        "3" to CalcNative.Key.DIGIT_0 + 3,
        "0" to CalcNative.Key.DIGIT_0,
    )

    private fun buildCalculatorPad(container: LinearLayout) {
        val rows = mutableListOf<List<Pair<String, Int>>>()
        rows.add(
            listOf(
                "C" to CalcNative.Key.CLEAR,
                "+/−" to CalcNative.Key.SIGN,
                "÷" to CalcNative.Key.DIVIDE,
                "×" to CalcNative.Key.MULTIPLY,
            )
        )
        val d = digits()
        rows.add(d.subList(0, 4))
        rows.add(d.subList(4, 7) + ("−" to CalcNative.Key.SUBTRACT))
        rows.add(d.subList(7, 10) + ("+" to CalcNative.Key.ADD))
        rows.add(
            listOf(
                "." to CalcNative.Key.DOT,
                "=" to CalcNative.Key.EQUALS,
            )
        )
        buildPad(container, rows)
    }

    private fun buildScientificPad(container: LinearLayout) {
        // A scientific pad is built from named function keys, which the Rust
        // engine turns into expressions; the kernel only knows digits and
        // arithmetic, so these route through submit rather than press.
        val container2 = container
        container2.removeAllViews()

        val functions = listOf(
            "sin", "cos", "tan",
            "asin", "acos", "atan",
            "ln", "log", "√",
            "x²", "xʸ", "1/x",
            "π", "e", "n!",
            "MC", "MR", "M+",
        )

        for (rowIndex in 0 until 3) {
            val line = LinearLayout(this)
            line.orientation = LinearLayout.HORIZONTAL
            line.layoutParams = LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                0,
                1f,
            )
            for (column in 0 until 3) {
                val name = functions[rowIndex * 3 + column]
                val button = Button(this)
                button.text = name
                button.isAllCaps = false
                button.textSize = 15f
                button.setTextColor(0xFF9FD4FF.toInt())
                button.setBackgroundResource(R.drawable.key_function_background)
                button.layoutParams =
                    LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.MATCH_PARENT, 1f)
                button.setOnClickListener { applyFunction(name) }
                line.addView(button)
            }
            container2.addView(line)
        }

        val d = digits()
        buildPad(
            container2,
            listOf(
                d.subList(0, 4),
                d.subList(4, 7) + ("(" to LEFT_PAREN),
                d.subList(7, 10) + (")" to RIGHT_PAREN),
                listOf(
                    "." to CalcNative.Key.DOT,
                    "C" to CalcNative.Key.CLEAR,
                    "÷" to CalcNative.Key.DIVIDE,
                ),
                listOf(
                    "×" to CalcNative.Key.MULTIPLY,
                    "−" to CalcNative.Key.SUBTRACT,
                    "+" to CalcNative.Key.ADD,
                    "=" to CalcNative.Key.EQUALS,
                ),
            ),
        )
    }

    private fun buildProgrammerPad(container: LinearLayout) {
        container.removeAllViews()

        // Base and word selectors, because a programmer calculator is mostly
        // about which base you are reading the number in.
        val selectors = LinearLayout(this)
        selectors.orientation = LinearLayout.HORIZONTAL
        selectors.layoutParams = LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            0,
            1f,
        )
        for ((label, base) in listOf(
            "HEX" to CalcNative.Base.HEXADECIMAL,
            "DEC" to CalcNative.Base.DECIMAL,
            "OCT" to CalcNative.Base.OCTAL,
            "BIN" to CalcNative.Base.BINARY,
        )) {
            val button = Button(this)
            button.text = label
            button.isAllCaps = true
            button.textSize = 13f
            button.setTextColor(0xFF9FD4FF.toInt())
            button.setBackgroundResource(R.drawable.key_function_background)
            button.layoutParams =
                LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.MATCH_PARENT, 1f)
            button.setOnClickListener {
                CalcNative.nativeSetBase(handle, base)
                refresh()
            }
            selectors.addView(button)
        }
        container.addView(selectors)

        // Bitwise row.
        val bitwise = LinearLayout(this)
        bitwise.orientation = LinearLayout.HORIZONTAL
        bitwise.layoutParams = LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            0,
            1f,
        )
        for ((label, operation) in listOf(
            "AND" to CalcNative.Bitwise.AND,
            "OR" to CalcNative.Bitwise.OR,
            "XOR" to CalcNative.Bitwise.XOR,
            "NOT" to CalcNative.Bitwise.NOT,
            "<<" to CalcNative.Bitwise.SHIFT_LEFT,
            ">>" to CalcNative.Bitwise.SHIFT_RIGHT,
        )) {
            val button = Button(this)
            button.text = label
            button.isAllCaps = true
            button.textSize = 12f
            button.setTextColor(0xFF9FD4FF.toInt())
            button.setBackgroundResource(R.drawable.key_function_background)
            button.layoutParams =
                LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.MATCH_PARENT, 1f)
            button.setOnClickListener {
                // A NOT is unary and applies straight away; the rest fold in
                // whatever is on the display as the second operand.
                if (operation == CalcNative.Bitwise.NOT) {
                    CalcNative.nativeProgrammerApply(handle, operation)
                } else {
                    CalcNative.nativeProgrammerRepeat(handle, operation)
                }
                refresh()
            }
            bitwise.addView(button)
        }
        container.addView(bitwise)

        // Hex digits and the numeric pad.
        val rows = mutableListOf<List<Pair<String, Int>>>()
        rows.add(
            listOf(
                "A" to 10,
                "B" to 11,
                "C" to 12,
                "D" to 13,
                "E" to 14,
                "F" to 15,
                "DEL" to -1,
                "C" to -2,
            )
        )
        val d = digits()
        rows.add(d.subList(0, 4))
        rows.add(d.subList(4, 7) + ("7" to 7))
        rows.add(d.subList(7, 10) + ("8" to 8))
        buildPad(container, rows)
    }

    private fun buildConvertPad(container: LinearLayout) {
        container.removeAllViews()

        val input = EditText(this)
        input.hint = "5 km to miles"
        input.setTextColor(0xFFFFFFFF.toInt())
        input.setHintTextColor(0xFF8A8A8A.toInt())
        input.textSize = 18f
        input.layoutParams = LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            ViewGroup.LayoutParams.WRAP_CONTENT,
        )
        container.addView(input)

        val result = TextView(this)
        result.text = ""
        result.setTextColor(0xFF9FD4FF.toInt())
        result.textSize = 20f
        result.gravity = Gravity.END
        result.layoutParams = LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            0,
            1f,
        )
        container.addView(result)

        val categories = listOf("Length", "Mass", "Temperature", "Area", "Volume", "Time", "Speed", "Data")
        for (rowIndex in 0 until 2) {
            val line = LinearLayout(this)
            line.orientation = LinearLayout.HORIZONTAL
            line.layoutParams = LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                0,
                1f,
            )
            for (column in 0 until 4) {
                val name = categories[rowIndex * 4 + column]
                val button = Button(this)
                button.text = name
                button.isAllCaps = false
                button.textSize = 13f
                button.setTextColor(0xFF9FD4FF.toInt())
                button.setBackgroundResource(R.drawable.key_function_background)
                button.layoutParams =
                    LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.MATCH_PARENT, 1f)
                button.setOnClickListener {
                    input.setText("1 $name to ")
                    input.setSelection(input.text.length)
                }
                line.addView(button)
            }
            container.addView(line)
        }
        input.addTextChangedListener(object : android.text.TextWatcher {
            override fun beforeTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
            override fun onTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
            override fun afterTextChanged(s: android.text.Editable?) {
                val text = s?.toString().orEmpty()
                if (text.isBlank()) {
                    result.text = ""
                } else {
                    result.text = CalcNative.nativeConvert(handle, text)
                }
            }
        })

        convertInput = input
        convertResult = result
    }

    private lateinit var convertInput: EditText
    private lateinit var convertResult: TextView

    // --- expression helpers -----------------------------------------------

    /**
     * Append a function to the current expression.
     *
     * The scientific pad shows named functions, but the kernel only speaks
     * digits and operators, so the function is woven into the expression text
     * and the whole thing is re-evaluated.
     */
    private fun applyFunction(name: String) {
        val current = CalcNative.nativeExpression(handle)
        val wrapped = when (name) {
            "√" -> "sqrt($current)"
            "x²" -> "($current)^2"
            "xʸ" -> "($current)^"
            "1/x" -> "1/($current)"
            "n!" -> "($current)!"
            "π" -> "pi"
            "e" -> "e"
            "MC" -> {
                CalcNative.nativeMemory(handle, 0, CalcNative.Memory.CLEAR)
                refresh()
                return
            }
            "MR" -> {
                val value = CalcNative.nativeMemory(handle, 0, CalcNative.Memory.RECALL)
                CalcNative.nativeSubmit(handle, value.toString())
                refresh()
                return
            }
            "M+" -> {
                CalcNative.nativeMemory(handle, 0, CalcNative.Memory.ADD)
                refresh()
                return
            }
            else -> "$name($current)"
        }
        CalcNative.nativeSubmit(handle, wrapped)
        refresh()
    }

    // Pseudo keycodes for keys the kernel does not define. The scientific pad
    // maps these onto expression text instead, so they never reach the kernel.
    private val LEFT_PAREN = 100
    private val RIGHT_PAREN = 101

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        val mapped = when (keyCode) {
            in KeyEvent.KEYCODE_0..KeyEvent.KEYCODE_9 -> keyCode - KeyEvent.KEYCODE_0
            KeyEvent.KEYCODE_PERIOD, KeyEvent.KEYCODE_NUMPAD_DOT -> CalcNative.Key.DOT
            KeyEvent.KEYCODE_EQUALS, KeyEvent.KEYCODE_NUMPAD_ENTER -> CalcNative.Key.EQUALS
            KeyEvent.KEYCODE_PLUS, KeyEvent.KEYCODE_NUMPAD_ADD -> CalcNative.Key.ADD
            KeyEvent.KEYCODE_MINUS, KeyEvent.KEYCODE_NUMPAD_SUBTRACT -> CalcNative.Key.SUBTRACT
            KeyEvent.KEYCODE_STAR, KeyEvent.KEYCODE_NUMPAD_MULTIPLY -> CalcNative.Key.MULTIPLY
            KeyEvent.KEYCODE_SLASH, KeyEvent.KEYCODE_NUMPAD_DIVIDE -> CalcNative.Key.DIVIDE
            KeyEvent.KEYCODE_CLEAR, KeyEvent.KEYCODE_DEL -> CalcNative.Key.CLEAR
            else -> return super.onKeyDown(keyCode, event)
        }
        press(mapped)
        return true
    }
}
