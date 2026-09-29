cmake_minimum_required(VERSION 3.22.1)

project(asmcalc C ASM)

add_library(asmcalc SHARED
    ${CMAKE_CURRENT_SOURCE_DIR}/../asm/calc_core.S
    jni_shim.c
)

target_include_directories(asmcalc PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../asm)
