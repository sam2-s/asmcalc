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

    /** Number bases, mirroring `Base` in Rust. */
    object Base {
        const val DECIMAL = 0
        const val HEXADECIMAL = 1
        const val OCTAL = 2
        const val BINARY = 3
    }

    /** Word sizes, mirroring `WordSize` in Rust. */
    object Word {
        const val BITS_8 = 0
        const val BITS_16 = 1
        const val BITS_32 = 2
        const val BITS_64 = 3
    }

    /** Bitwise operations, mirroring `Bitwise` in Rust. */
    object Bitwise {
        const val AND = 0
        const val OR = 1
        const val XOR = 2
        const val NOT = 3
        const val SHIFT_LEFT = 4
        const val SHIFT_RIGHT = 5
        const val MODULO = 6
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

    // Programmer mode.

    external fun nativeSetBase(handle: Long, base: Int)

    external fun nativeGetBase(handle: Long): Int

    external fun nativeSetWordSize(handle: Long, word: Int)

    /** Returns true when the digit is valid in the current base. */
    external fun nativeProgrammerPush(handle: Long, digit: String): Boolean

    external fun nativeProgrammerClear(handle: Long)

    external fun nativeProgrammerBackspace(handle: Long)

    external fun nativeProgrammerApply(handle: Long, operation: Int)

    external fun nativeProgrammerRepeat(handle: Long, operation: Int)

    /** Newline separated `HEX ...`, `DEC ...`, `OCT ...`, `BIN ...`. */
    external fun nativeProgrammerBases(handle: Long): String

    // Unit conversion.

    /** Run a request such as `5 km to miles`, returning a description. */
    external fun nativeConvert(handle: Long, request: String): String
}
