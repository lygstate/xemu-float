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
    uint32_t saved_mxcsr;
    uint32_t previous_mxcsr;
    uint32_t current_mxcsr;
    InstructionEvaluate evaluate;
    uint64_t rip;
    uint64_t rsp;
};

inline unsigned long float_handle_exception(unsigned long code)
{
    if (code >= STATUS_FLOAT_DENORMAL_OPERAND && code <= STATUS_FLOAT_UNDERFLOW)
    {
        return EXCEPTION_EXECUTE_HANDLER;
    }
    else
    {
        return EXCEPTION_CONTINUE_SEARCH;
    }
}

inline double fdiv64_no_exception(double a, double b)
{
    __m128d ma = _mm_set1_pd(a);
    __m128d mb = _mm_set1_pd(b);
    __m128d mc = _mm_div_pd(ma, mb);
    return _mm_cvtsd_f64(mc);
}

inline float fdiv32_no_exception(float a, float b)
{
    __m128 ma = _mm_set1_ps(a);
    __m128 mb = _mm_set1_ps(b);
    __m128 mc = _mm_div_ps(ma, mb);
    return _mm_cvtss_f32(mc);
}

inline void float_save_and_prepare_csr(struct FloatInstruction *instruction)
{
    instruction->saved_mxcsr = _mm_getcsr();
    _mm_setcsr(0x1F80);
}

inline void float_update_and_restore_csr(struct FloatInstruction *instruction)
{
    instruction->previous_mxcsr = instruction->current_mxcsr;
    uint32_t diff = _mm_getcsr() ^ 0x1F80;
    instruction->current_mxcsr |= diff << 7 | diff;
    _mm_setcsr(instruction->current_mxcsr);
}

void test_double_div64_c(
    struct FloatInstruction *instruction,
    const double *a,
    const double *b,
    double *c,
    uint32_t *csr,
    size_t len,
    size_t count)
{
    for (int j = 0; j < count; ++j)
    {
        int i = 0;
        for (;;)
        {
            {
                for (i; i < len;)
                {
                    // NOTE: Directly calling to function is faster than function pointer.
                    c[i] = fdiv64_no_exception(a[i], b[i]);
                    // csr[i] = instruction->current_mxcsr;
                    ++i;
                }
                break;
            }
#if 0
            __except (float_handle_exception(GetExceptionCode()))
            {
                float_save_and_prepare_csr(instruction);
                c[i] = fdiv64_no_exception(a[i], b[i]);
                float_update_and_restore_csr(instruction);
                csr[i] = instruction->current_mxcsr;
                ++i;
            }
#endif
        }
    }
}

void test_double_div32_c(
    struct FloatInstruction *instruction,
    const float *a,
    const float *b,
    float *c,
    uint32_t *csr,
    size_t len,
    size_t count)
{
    for (int j = 0; j < count; ++j)
    {
        int i = 0;
        for (;;)
        {
            {
                for (i; i < len;)
                {
                    c[i] = fdiv32_no_exception(a[i], b[i]);
                    // csr[i] = instruction->current_mxcsr;
                    ++i;
                }
                break;
            }
#if 0
            __except (float_handle_exception(GetExceptionCode()))
            {
                float_save_and_prepare_csr(instruction);
                c[i] = fdiv32_no_exception(a[i], b[i]);
                float_update_and_restore_csr(instruction);
                csr[i] = instruction->current_mxcsr;
                ++i;
            }
#endif
        }
    }
}
