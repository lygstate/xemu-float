#![feature(asm)]
#![feature(core_intrinsics)]

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use std::time::Instant;
/*
rustup default nightly-x86_64-pc-windows-msvc
rustup default nightly-x86_64-pc-windows-gnu
rustup default nightly-i686-pc-windows-msvc
del -Recurse .\target\
cargo run --release
cargo rustc --release -- --emit asm
 */

/*
    1111 10100000

10000000 01000000
00011111 10000000 // init value
00001111 10000000 // init value wihtout inexact
54321098 76543210
01100000 00000000

00000000 00000000  // init value enable all exception, default round mode, Disable `Flush To Zero`  and disable Denormals Are Zero
*/

/*
Pnemonic	Bit Location	Description
FZ	bit 15	Flush To Zero

R+	bit 14	Round Positive
R-	bit 13	Round Negative
RZ	bits 13 and 14	Round To Zero
RN	bits 13 and 14 are 0	Round To Nearest

PM	bit 12	Precision Mask
UM	bit 11	Underflow Mask
OM	bit 10	Overflow Mask
ZM	bit 9	Divide By Zero Mask
DM	bit 8	Denormal Mask
IM	bit 7	Invalid Operation Mask
DAZ	bit 6	Denormals Are Zero
PE	bit 5	Precision Flag
UE	bit 4	Underflow Flag
OE	bit 3	Overflow Flag
ZE	bit 2	Divide By Zero Flag
DE	bit 1	Denormal Flag
IE	bit 0	Invalid Operation Flag
*/

type InstructionEvaluate = unsafe extern "C" fn(instruction: &mut FloatInstruction);

#[repr(C)]
pub struct FloatInstruction {
    pub operands_f64: [f64; 32],
    pub operands_f32: [f32; 64],
    pub recover_mxcsr: u32,
    pub init_mxcsr: u32,
    pub current_mxcsr: u32,
    pub evaluate: Option<InstructionEvaluate>,
    rip: u64,
    rsp: u64,
}

impl Default for FloatInstruction {
    fn default() -> FloatInstruction {
        FloatInstruction {
            operands_f64: [0.0; 32],
            operands_f32: [0.0; 64],
            recover_mxcsr: 0,
            init_mxcsr: 0,
            current_mxcsr: 0,
            evaluate: None,
            rip: 0,
            rsp: 0,
        }
    }
}
extern "C" {
    pub fn fdiv64_no_exception(instruction: &mut FloatInstruction);
    pub fn fdiv32_no_exception(instruction: &mut FloatInstruction);
}

#[no_mangle]
pub extern "C" fn fdiv64(instruction: &mut FloatInstruction) {
    unsafe {
        _mm_setcsr(instruction.init_mxcsr);
        let ma = _mm_set1_pd(instruction.operands_f64[1]);
        let mb = _mm_set1_pd(instruction.operands_f64[2]);
        let mc = _mm_div_pd(ma, mb);
        instruction.operands_f64[0] = _mm_cvtsd_f64(mc);
        instruction.current_mxcsr = _mm_getcsr();
    }
}

#[no_mangle]
pub extern "C" fn fdiv32(instruction: &mut FloatInstruction) {
    unsafe {
        _mm_setcsr(instruction.init_mxcsr);
        let ma = _mm_set1_ps(instruction.operands_f32[1]);
        let mb = _mm_set1_ps(instruction.operands_f32[2]);
        let mc = _mm_div_ps(ma, mb);
        instruction.operands_f32[0] = _mm_cvtss_f32(mc);
        instruction.current_mxcsr = _mm_getcsr();
    }
}

fn test_double_div64(
    instruction: &mut FloatInstruction,
    a: &[f64],
    b: &[f64],
    c: &mut [f64],
    csr: &mut [u32],
    count: usize,
) {
    let now = Instant::now();
    let ops = a.len();
    unsafe {
        instruction.current_mxcsr = instruction.init_mxcsr;
        _mm_setcsr(instruction.init_mxcsr);
    }
    if let Some(f) = instruction.evaluate {
        for _ in 0..count {
            for i in 0..ops {
                unsafe {
                    _mm_setcsr(instruction.init_mxcsr);
                    let ma = _mm_set1_pd(a[i]);
                    let mb = _mm_set1_pd(b[i]);
                    let mc = _mm_div_pd(ma, mb);
                    c[i] = _mm_cvtsd_f64(mc);
                    csr[i] = _mm_getcsr();
                }
            }
        }
    }
    unsafe {
        _mm_setcsr(instruction.recover_mxcsr);
    }
    let mops = (ops * count) as f64 / 1024.0 / 1024.8;
    let mflops = mops / now.elapsed().as_secs_f64();
    println!("recover_mxcsr: {}", instruction.recover_mxcsr);
    println!(
        "final f64 csr: {} mflops:{} c:{}",
        instruction.current_mxcsr,
        mflops,
        c[1024 * 1024]
    );
}

fn test_double_div32(
    instruction: &mut FloatInstruction,
    a: &[f32],
    b: &[f32],
    c: &mut [f32],
    csr: &mut [u32],
    count: usize,
) {
    let now = Instant::now();
    let ops = a.len();
    unsafe {
        instruction.current_mxcsr = instruction.init_mxcsr;
        _mm_setcsr(instruction.init_mxcsr);
    }
    if let Some(f) = instruction.evaluate {
        for _ in 0..count {
            for i in 0..ops {
                unsafe {
                    _mm_setcsr(instruction.init_mxcsr);
                    let ma = _mm_set1_ps(a[i]);
                    let mb = _mm_set1_ps(b[i]);
                    let mc = _mm_div_ps(ma, mb);
                    c[i] = _mm_cvtss_f32(mc);
                    csr[i] = _mm_getcsr();
                }
            }
        }
    }
    unsafe {
        _mm_setcsr(instruction.recover_mxcsr);
    }
    let mops = (ops * count) as f64 / 1024.0 / 1024.8;
    let mflops = mops / now.elapsed().as_secs_f64();
    println!("recover_mxcsr: {}", instruction.recover_mxcsr);
    println!(
        "final f32 csr: {} mflops:{} c:{}",
        instruction.current_mxcsr,
        mflops,
        c[1024 * 1024]
    );
}

fn main() {
    let mut instruction: FloatInstruction = Default::default();
    unsafe {
        instruction.recover_mxcsr = _mm_getcsr();
        println!("recover_mxcsr:{}", instruction.recover_mxcsr);
        asm! {
            "fninit"
        };
        instruction.init_mxcsr = 0x80;
        println!("use mxcsr:{}", instruction.init_mxcsr);
    }
    let ops = 1024 * 1024 * 64;
    let mut a64 = vec![6.0f64; ops];
    let mut b64 = vec![5.0f64; ops];
    let mut c64 = vec![0.0f64; ops];
    a64[1024 * 1024] = 7.0f64;
    b64[1024 * 1024] = 9.0f64;

    let mut a32 = vec![6.0f32; ops];
    let mut b32 = vec![6.0f32; ops];
    let mut c32 = vec![0.0f32; ops];
    a32[1024 * 1024] = 7.0f32;
    b32[1024 * 1024] = 9.0f32;

    let mut csr = vec![0u32; ops];
    /*
    for vi in 0..ops {
        a64[vi] = vi as f64;
    }
    */

    for _ in 0..8 {
        println!("no mxcsr");
        instruction.init_mxcsr = 0x1F80;
        instruction.evaluate = Some(fdiv64_no_exception);
        test_double_div64(
            &mut instruction,
            a64.as_slice(),
            b64.as_slice(),
            c64.as_mut_slice(),
            csr.as_mut_slice(),
            8,
        );
        instruction.init_mxcsr = 0x1F80;
        instruction.evaluate = Some(fdiv32_no_exception);
        test_double_div32(
            &mut instruction,
            a32.as_slice(),
            b32.as_slice(),
            c32.as_mut_slice(),
            csr.as_mut_slice(),
            8,
        );

        println!("set/get mxcsr");
        instruction.init_mxcsr = 0x1F80;
        instruction.evaluate = Some(fdiv64);
        test_double_div64(
            &mut instruction,
            a64.as_slice(),
            b64.as_slice(),
            c64.as_mut_slice(),
            csr.as_mut_slice(),
            8,
        );
        instruction.init_mxcsr = 0x1F80;
        instruction.evaluate = Some(fdiv32);
        test_double_div32(
            &mut instruction,
            a32.as_slice(),
            b32.as_slice(),
            c32.as_mut_slice(),
            csr.as_mut_slice(),
            8,
        );
    }
}
