cmake_minimum_required(VERSION 3.22.1)

# The native library is built by cargo: the Rust feature layer plus the
# AArch64 assembly kernel, which build.rs assembles with the NDK toolchain.
#
# AGP only packages a .so that a real CMake target produces, so the cargo
# build is wrapped in a tiny C stub library. The stub is never compiled on its
# own terms; its only job is to exist so Gradle has something to package, while
# its build step produces the actual library.

set(ASM_CALC_RUST_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../../../rust)

# AGP exports ANDROID_NDK_ROOT but not ANDROID_NDK_HOME, and cargo's [env] block
# in .cargo/config.toml does not override a variable that is merely *set* to the
# empty string. Normalise it here so build.rs always sees a real path.
# AGP names the NDK directory ANDROID_NDK and does not export either
# ANDROID_NDK_HOME or ANDROID_NDK_ROOT, which is what cargo's build.rs looks
# for. Resolve it here, falling back to the SDK tree, so the path is never
# merely set-but-empty (cargo's [env] block cannot override an empty value).
if(NOT ANDROID_NDK_HOME OR ANDROID_NDK_HOME STREQUAL "")
    if(ANDROID_NDK)
        set(ANDROID_NDK_HOME "${ANDROID_NDK}")
    elseif(ANDROID_NDK_ROOT)
        set(ANDROID_NDK_HOME "${ANDROID_NDK_ROOT}")
    else()
        file(GLOB ASM_CALC_NDKS "${ANDROID_SDK_ROOT}/ndk/*")
        list(SORT ASM_CALC_NDKS)
        list(REVERSE ASM_CALC_NDKS)
        list(GET ASM_CALC_NDKS 0 ASM_CALC_NDK)
        set(ANDROID_NDK_HOME "${ASM_CALC_NDK}")
    endif()
endif()
message(STATUS "AsmCalc: ANDROID_NDK_HOME=${ANDROID_NKD_HOME}")

# AGP names the ABI `arm64-v8a`; Rust calls the same target `aarch64`.
if(ANDROID_ABI STREQUAL "arm64-v8a")
    set(ASM_CALC_RUST_TARGET aarch64-linux-android)
else()
    set(ASM_CALC_RUST_TARGET ${ANDROID_ABI}-linux-android)
endif()

set(ASM_CALC_SO
    ${ASM_CALC_RUST_DIR}/target/${ASM_CALC_RUST_TARGET}/release/libasmcalc.so)

file(GLOB ASM_CALC_STUB_SOURCES ${CMAKE_CURRENT_SOURCE_DIR}/stub/*.c)

add_library(asmcalc SHARED ${ASM_CALC_STUB_SOURCES})

# Force the cargo build to run before anything tries to package the .so.
add_custom_command(
    TARGET asmcalc PRE_BUILD
    COMMAND ${CMAKE_COMMAND} -E env
        RUSTFLAGS=${ASM_CALC_RUSTFLAGS}
        ANDROID_NDK_HOME=${ANDROID_NDK_HOME}
        cargo build
            --offline
            --manifest-path ${ASM_CALC_RUST_DIR}/Cargo.toml
            --target ${ASM_CALC_RUST_TARGET}
            --release
    COMMENT "Building libasmcalc.so with cargo (Rust feature layer plus the AArch64 kernel)"
    VERBATIM
)

# Replace whatever CMake just stubbed out with the cargo artefact.
add_custom_command(
    TARGET asmcalc POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy ${ASM_CALC_SO} $<TARGET_FILE:asmcalc>
    COMMENT "Installing the cargo built libasmcalc.so"
    VERBATIM
)
