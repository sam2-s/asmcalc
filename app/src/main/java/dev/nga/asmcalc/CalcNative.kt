package dev.nga.asmcalc

object CalcNative {

    init {
        System.loadLibrary("asmcalc")
    }

    external fun nativeCreate(): Long

    external fun nativeDestroy(handle: Long)

    external fun nativeKey(handle: Long, keycode: Int)

    external fun nativeFormat(handle: Long): String
}
