# AsmCalc

An Android calculator whose entire engine is written in AArch64 ARM assembly.

The state machine, the fixed-point arithmetic, the decimal formatting and the error
handling all live in [`app/src/main/asm/calc_core.S`](app/src/main/asm/calc_core.S).
That file calls nothing: no libc, no runtime, no PLT entries. Every routine is a leaf
function that keeps its state in registers.

Everything outside the engine is deliberately thin. The only C in the project is a JNI
shim that forwards calls, and the only Kotlin is a keypad.

| Layer | Language | Size |
| --- | --- | --- |
| Calculator engine | AArch64 assembly | the whole `libasmcalc.so` is 7.6 KB |
| JNI bridge | C | pass-through only, no logic |
| UI | Kotlin | button grid, no arithmetic |

## Number representation

Values are **sign-magnitude fixed point scaled by 10000**, so `1` is `10000` and
`12.5` is `125000`. This is a deliberate trade:

- every operation stays unsigned, so no signed 128-bit division is needed
- `0.1 + 0.2` is exactly `0.3`, with no floating point drift
- overflow is detected with the carry out of `adds`, or with `umulh` for products
- the range is about 10^15, which is far more than the display can show

Division rounds to nearest, so `1/3` is `0.3333` and `2/3` is `0.6667`. Because
values are truncated to four decimals, `1/3 x 3` is `0.9999`. This is the price of
staying in integer assembly, and it is a price worth paying for an engine with no
floating-point runtime at all.

Multiplication and division both need an intermediate wider than 64 bits, so the core
contains a 128-bit divide ([`udiv128`](app/src/main/asm/calc_core.S)) that falls back to
a single `udiv` instruction whenever the high half is zero.

## Behaviour

- operators chain with immediate execution, like a pocket calculator: `2+3*4=` is `20`
- a repeated `=` repeats the last operation: `2+3==` is `8`
- dividing by zero or exceeding the magnitude limit latches `Error` until `C`
- entry is capped at 12 integer digits, and a fifth fractional digit shifts the window
  with rounding instead of being dropped
- hardware keyboards, including the numpad, drive the same keycodes

## Layout

```
app/src/main/asm/calc_core.S      the engine
app/src/main/asm/calc_keys.inc    keycodes, shared by assembly and C
app/src/main/asm/calc_state.h     the state layout, with C static assertions
app/src/main/cpp/jni_shim.c       JNI pass-through
app/src/main/java/...             keypad and display
app/src/test/asm/calc_tests.S     the test suite, run by the host harness
hosttest/                         freestanding runner for the suite
```

The assembly and the C shim share `calc_state.h`. The shim asserts every field offset
at compile time with `_Static_assert`, so the two views of the state cannot silently
drift apart.

## Building the APK

```sh
./gradlew :app:assembleDebug
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

Requires JDK 17 and the Android NDK. The project pins Gradle 9.6.1, AGP 9.2.1 and
CMake 3.22.1, and targets `arm64-v8a` only.

## Running the tests

The suite is a freestanding AArch64 binary: it has its own `_start` and uses raw
`write` and `exit` syscalls, with no libc at all. It is assembled with the NDK
toolchain and run under `qemu-aarch64`.

```sh
./hosttest/build.sh
```

```text
PASS harness: comparison and reporting primitives
PASS twelve times twelve is a hundred and forty four
PASS a tenth plus a fifth is exactly three tenths
PASS operators chain with immediate execution
...
cases: 29 failed: 0
all tests passed
```

The exit code is the number of failures, so the script works directly in CI.

The harness self-tests first. A test suite that cannot detect a failing comparison is
not worth much, so the framework verifies its own primitives before it trusts them.

## Continuous integration

- [`asm-test`](.github/workflows/asm-test.yml) builds and runs the assembly suite under
  qemu on every push
- [`android-apk`](.github/workflows/android-apk.yml) assembles the debug APK, uploads
  it as a build artifact, and publishes it as a release asset when a `v*` tag is pushed

## License

MIT. See [LICENSE](LICENSE).
