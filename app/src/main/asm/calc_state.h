#pragma once

#include <stdint.h>

#define ST_ACC 0
#define ST_ENTRY 8
#define ST_RHS 16
#define ST_ACC_SIGN 24
#define ST_ENTRY_SIGN 25
#define ST_RHS_SIGN 26
#define ST_DOT 27
#define ST_INT_DIGITS 28
#define ST_FRAC_DIGITS 29
#define ST_OP 30
#define ST_LAST_OP 31
#define ST_TYPING 32
#define ST_ERROR 33
#define CALC_STATE_SIZE 40
#define CALC_FORMAT_BUFFER_SIZE 64

typedef struct {
    uint64_t acc;
    uint64_t entry;
    uint64_t rhs;
    uint8_t acc_sign;
    uint8_t entry_sign;
    uint8_t rhs_sign;
    uint8_t dot;
    uint8_t int_digits;
    uint8_t frac_digits;
    uint8_t op;
    uint8_t last_op;
    uint8_t typing;
    uint8_t error;
    uint8_t reserved[6];
} calc_state;
