# AsmCalc

An Android calculator with an AArch64 assembly arithmetic kernel and a Rust
feature layer on top of it.

```
Kotlin UI  ──JNI──▶  Rust cdylib  ──C ABI──▶  AArch64 assembly kernel
 (tabs)              expressions,              fixed point, no libc,
                     transcendentals,          no runtime calls
                     units, bases,
                     bignum, matrices
```

One native library, two engines. The assembly is not a legacy fallback that
nothing calls: it is the arithmetic path, and it is what runs when you open the
app.

## The assembly kernel

[`app/src/main/asm/calc_core.S`](app/src/main/asm/calc_core.S) holds the state
machine, the arithmetic, the decimal formatting and the error handling. It calls
nothing. No libc, no PLT entries, every routine a leaf function.

Values are **sign-magnitude fixed point scaled by 10000**, so `1` is `10000` and
`12.5` is `125000`. Every operation therefore stays unsigned, which is why the
core needs no signed 128-bit division. `0.1 + 0.2` is exactly `0.3`, overflow is
caught by the carry out of `adds` and by `umulh` for products, and division rounds
to nearest so `2/3` is `0.6667`.

Multiplication and division both need an intermediate wider than 64 bits, so the
core contains a 128-bit divide ([`udiv128`](app/src/main/asm/calc_core.S)) that
falls back to a single `udiv` whenever the high half is zero.

## The Rust layer

Rust owns everything that cannot be a four-decimal fixed-point engine:

| Module | What it does |
| --- | --- |
| `expr` | Recursive descent parser, real precedence, `^` right associative |
| `expr::number` | Exact decimals on `num-bigint`: `1/3` is `0.333…` and `20!` is exact |
| `programmer` | DEC/HEX/OCT/BIN, bitwise, two's complement, 8 to 64 bit words |
| `units` | Eight categories, with `5 km to miles` request parsing |
| `algebra` | Matrix add/multiply/determinant/inverse, rectangular complex |
| `engine` | The session: memory registers and the history tape |

The **exact** tower is a second parser, not a mode flag on the first. `sqrt(2)`
has no exact decimal answer and `1/3` has no exact `f64` answer, so each engine
answers what it can answer exactly and reports the rest rather than approximating
in both directions.

## Testing

Two suites, and the second one exists to keep the first honest.

**The assembly suite** ([`hosttest/`](hosttest)) is a freestanding AArch64
binary: its own `_start`, raw `write` and `exit` syscalls, no libc at all. It is
assembled with the NDK and run under `qemu-aarch64`.

```sh
./hosttest/build.sh
```

```text
PASS harness: comparison and reporting primitives
PASS twelve times twelve is a hundred and forty four
PASS a tenth plus a fifth is exactly three tenths
...
cases: 29 failed: 0
```

The exit code is the failure count, so it drops straight into CI. The harness
self-tests first, because a suite that cannot detect a failing comparison is not
worth much.

**The Rust suite** has 70 tests and runs natively, which is fast. The
interesting part is `kernel_parity`, which replays all 28 calculator cases
*through the FFI against the real assembly*:

```sh
./rusttest.sh
```

It cross compiles to a static `aarch64-unknown-linux-musl` binary so qemu can run
it. musl is used because an Android binary needs `/system/bin/linker64`, which
qemu cannot provide.

On the host there is no AArch64, so `kernel_engine` supplies a Rust transcription
of the kernel. The parity suite is what holds the two to the same answers.

## Continuous integration

| Workflow | What it does |
| --- | --- |
| `asm-test` | Assembles and runs the assembly suite under qemu |
| `rust-test` | Native Rust tests, the FFI parity suite, and clippy with `-D warnings` |
| `android-apk` | Builds the APK, uploads it, and releases it on a `v*` tag |

## Building

```sh
./gradlew :app:assembleDebug
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

Pinned: Gradle 9.6.1, AGP 9.2.1, CMake 3.22.1, NDK r27b, `arm64-v8a` only.

The native library is built by cargo and copied in by CMake, so there is still
only one build system with one output. Two details are worth knowing if you build
this on a new machine:

- The project expects a JDK 17+ on `PATH` or `JAVA_HOME`.
- Your `~/.cargo/config.toml` matters. It normally sets `-static`, which is
  correct for a static executable and wrong for a cdylib, so the Gradle build
  passes `RUSTFLAGS` explicitly to override it.

## License

MIT. See [LICENSE](LICENSE).
