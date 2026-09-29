#include <jni.h>
#include <stddef.h>
#include <stdlib.h>
#include <stdint.h>

#include "calc_keys.inc"
#include "calc_state.h"

void calc_reset(calc_state *state);
void calc_key(calc_state *state, int keycode);
int calc_format(calc_state *state, char *out);

_Static_assert(sizeof(calc_state) == CALC_STATE_SIZE, "calc_state size must match the assembly layout");
_Static_assert(offsetof(calc_state, acc) == ST_ACC, "acc offset must match the assembly layout");
_Static_assert(offsetof(calc_state, entry) == ST_ENTRY, "entry offset must match the assembly layout");
_Static_assert(offsetof(calc_state, rhs) == ST_RHS, "rhs offset must match the assembly layout");
_Static_assert(offsetof(calc_state, acc_sign) == ST_ACC_SIGN, "acc_sign offset must match the assembly layout");
_Static_assert(offsetof(calc_state, entry_sign) == ST_ENTRY_SIGN, "entry_sign offset must match the assembly layout");
_Static_assert(offsetof(calc_state, rhs_sign) == ST_RHS_SIGN, "rhs_sign offset must match the assembly layout");
_Static_assert(offsetof(calc_state, dot) == ST_DOT, "dot offset must match the assembly layout");
_Static_assert(offsetof(calc_state, int_digits) == ST_INT_DIGITS, "int_digits offset must match the assembly layout");
_Static_assert(offsetof(calc_state, frac_digits) == ST_FRAC_DIGITS, "frac_digits offset must match the assembly layout");
_Static_assert(offsetof(calc_state, op) == ST_OP, "op offset must match the assembly layout");
_Static_assert(offsetof(calc_state, last_op) == ST_LAST_OP, "last_op offset must match the assembly layout");
_Static_assert(offsetof(calc_state, typing) == ST_TYPING, "typing offset must match the assembly layout");
_Static_assert(offsetof(calc_state, error) == ST_ERROR, "error offset must match the assembly layout");

JNIEXPORT jlong JNICALL
Java_dev_nga_asmcalc_CalcNative_nativeCreate(JNIEnv *env, jclass clazz) {
    calc_state *state = (calc_state *) calloc(1, sizeof(calc_state));
    if (state == NULL) {
        return 0;
    }
    calc_reset(state);
    return (jlong) (intptr_t) state;
}

JNIEXPORT void JNICALL
Java_dev_nga_asmcalc_CalcNative_nativeDestroy(JNIEnv *env, jclass clazz, jlong handle) {
    free((void *) (intptr_t) handle);
}

JNIEXPORT void JNICALL
Java_dev_nga_asmcalc_CalcNative_nativeKey(JNIEnv *env, jclass clazz, jlong handle, jint keycode) {
    calc_key((calc_state *) (intptr_t) handle, (int) keycode);
}

JNIEXPORT jstring JNICALL
Java_dev_nga_asmcalc_CalcNative_nativeFormat(JNIEnv *env, jclass clazz, jlong handle) {
    char buffer[CALC_FORMAT_BUFFER_SIZE];
    calc_state *state = (calc_state *) (intptr_t) handle;
    calc_format(state, buffer);
    return (*env)->NewStringUTF(env, buffer);
}
