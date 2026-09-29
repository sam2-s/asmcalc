package dev.nga.asmcalc

/**
 * The native calculator.
 *
 * Backed by libasmcalc.so, which is two engines in one library: the AArch64
 * assembly kernel and the Rust feature layer. A handle is created once per
 * Activity and owns the state on the native side.
 */
object CalcNative {

    /** Mirrors `dev.nga.asmcalc.engine.Engine` in Rust. */
    object Engine {
        /** The assembly kernel: four decimals, no libc. */
        const val FIXED = 0

        /** Expressions and transcendentals in f64. */
        const val SCIENTIFIC = 1

        /** Integers, multiple bases, bitwise operations. */
        const val PROGRAMMER = 2
    }

    /** Mirrors `AngleMode` in Rust. */
    object Angles {
        const val DEGREES = 0
        const val RADIANS = 1
        const val GRADIANS = 2
    }

    /** Keycodes, matching `app/src/main/asm/calc_keys.inc`. */
    object Key {
        const val DIGIT_0 = 0
        const val DOT = 10
        const val EQUALS = 11
        const val ADD = 12
        const val SUBTRACT = 13
        const val MULTIPLY = 14
        const val DIVIDE = 15
        const val CLEAR = 16
        const val SIGN = 17
    }

    /** Memory register operations, passed to [nativeMemory]. */
    object Memory {
        const val ADD = 0
        const val SUBTRACT = 1
        const val RECALL = 2
        const val RECALL_INTO_DISPLAY = 3
        const val STORE = 4
        const val CLEAR = 5
    }

    init {
        System.loadLibrary("asmcalc")
    }

    external fun nativeCreate(): Long

    external fun nativeDestroy(handle: Long)

    external fun nativePress(handle: Long, keycode: Int)

    external fun nativeDisplay(handle: Long): String

    external fun nativeExpression(handle: Long): String

    external fun nativeSetEngine(handle: Long, engine: Int)

    external fun nativeGetEngine(handle: Long): Int

    external fun nativeSetAngleMode(handle: Long, mode: Int)

    external fun nativeAngleMode(handle: Long): String

    external fun nativeSubmit(handle: Long, expression: String): String

    external fun nativeSubmitExact(handle: Long, expression: String): String

    external fun nativeHistorySize(handle: Long): Int

    external fun nativeHistoryEntry(handle: Long, index: Int): String

    external fun nativeRecall(handle: Long, index: Int)

    external fun nativeMemory(handle: Long, slot: Int, operation: Int): Double
}
