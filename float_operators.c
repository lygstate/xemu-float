#include <stdio.h>
#include <stdint.h>
#include <xmmintrin.h>
#include <emmintrin.h>
#include <stddef.h>
#include <intrin.h>

#include <windows.h>

struct FloatInstruction;

typedef void (*InstructionEvaluate)(struct FloatInstruction *instruction);

struct FloatInstruction
{
    double operands_f64[32];
    float operands_f32[64];
    uint32_t recover_mxcsr;
    uint32_t init_mxcsr;
    uint32_t current_mxcsr;
    InstructionEvaluate evaluate;
    uint64_t rip;
    uint64_t rsp;
};

inline unsigned long float_handle_exception(
    struct _EXCEPTION_POINTERS *ep,
    struct FloatInstruction *instruction)
{
    unsigned long code = ep->ExceptionRecord->ExceptionCode;
    int mask = 0;
    if (code == EXCEPTION_FLT_INEXACT_RESULT)
    {
        mask = 1 << 12;
        instruction->current_mxcsr |= 1 << 5;
    }
    else if (code == EXCEPTION_FLT_UNDERFLOW)
    {
        mask = 1 << 11;
        instruction->current_mxcsr |= 1 << 4;
    }
    else if (code == EXCEPTION_FLT_OVERFLOW)
    {
        mask = 1 << 10;
        instruction->current_mxcsr |= 1 << 3;
    }
    else if (code == EXCEPTION_FLT_DIVIDE_BY_ZERO)
    {
        mask = 1 << 9;
        instruction->current_mxcsr |= 1 << 2;
    }
    else if (code == EXCEPTION_FLT_DENORMAL_OPERAND)
    {
        mask = 1 << 8;
        instruction->current_mxcsr |= 1 << 1;
    }
    else
    {
        return EXCEPTION_CONTINUE_SEARCH;
    }
    instruction->current_mxcsr |= mask;
    _mm_setcsr(0x1F80);
    instruction->evaluate(instruction);
    _mm_setcsr(instruction->current_mxcsr);
    return EXCEPTION_EXECUTE_HANDLER;
}

void fdiv64_no_exception(struct FloatInstruction *instruction)
{
    __m128d ma = _mm_set1_pd(instruction->operands_f64[1]);
    __m128d mb = _mm_set1_pd(instruction->operands_f64[2]);
    __try
    {
        __m128d mc = _mm_div_pd(ma, mb);
        instruction->operands_f64[0] = _mm_cvtsd_f64(mc);
    }
    __except (float_handle_exception(GetExceptionInformation(), instruction))
    {
    }
}

void fdiv32_no_exception(struct FloatInstruction *instruction)
{
    __m128 ma = _mm_set1_ps(instruction->operands_f32[1]);
    __m128 mb = _mm_set1_ps(instruction->operands_f32[2]);

    __try
    {
        __m128 mc = _mm_div_ps(ma, mb);
        instruction->operands_f32[0] = _mm_cvtss_f32(mc);
    }
    __except (float_handle_exception(GetExceptionInformation(), instruction))
    {
    }
}
