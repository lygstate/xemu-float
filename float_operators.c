#include <stdint.h>
#include <xmmintrin.h>
#include <emmintrin.h>

struct FloatInstruction;

typedef void (*InstructionEvaluate)(struct FloatInstruction *instruction);

struct FloatInstruction
{
    double operands_f64[32];
    float operands_f32[32];
    InstructionEvaluate evaluate;
    uint32_t init_mxcsr;
    uint32_t current_mxcsr;
};

void fdiv64_no_exception(struct FloatInstruction *instruction)
{
    __m128d ma = _mm_set1_pd(instruction->operands_f64[1]);
    __m128d mb = _mm_set1_pd(instruction->operands_f64[2]);
    __m128d mc = _mm_div_pd(ma, mb);
    instruction->operands_f64[0] = _mm_cvtsd_f64(mc);
}

void fdiv32_no_exception(struct FloatInstruction *instruction)
{
    __m128 ma = _mm_set1_ps(instruction->operands_f32[1]);
    __m128 mb = _mm_set1_ps(instruction->operands_f32[2]);
    __m128 mc = _mm_div_ps(ma, mb);
    instruction->operands_f32[0] = _mm_cvtss_f32(mc);
}
