#![feature(asm)]
use core::arch::x86_64::*;
use std::time::Instant;

/*
00011111 10100000
00011111 10000000
54321098 76543210
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

fn test_double_div(a: f64, b:f64, init_csr: u32, ops: u32) {
    let mut final_csr: u32 = 0;
    let mut count = 0;
    let mut c: f64 = 0.0;
    let now = Instant::now();
    unsafe {
        for _i in 0..ops {
            final_csr = _mm_getcsr();
            if final_csr != init_csr {
                count += 1;
                _mm_setcsr(init_csr);
            }
            let ma = _mm_set1_pd(a);
            let mb = _mm_set1_pd(b);
            let mc = _mm_div_pd(ma, mb);
            c +=  _mm_cvtsd_f64(mc);
        }
    }
    let m_opts = ops as f64 / 1024.0/1024.8;

    println!("final csr: {} flops:{} count:{} c:{}",
    final_csr, m_opts / now.elapsed().as_secs_f64(), count, c);
}

fn main() {
    let mut init_csr: u32;
    unsafe {
        println!("first csr:{}", _mm_getcsr());
        _mm_setcsr(_mm_getcsr() & !0x8040);
        init_csr = _mm_getcsr();
        init_csr = init_csr | 0b100000;
        println!("init_csr:{}", init_csr);
    }
    let ops = 256 * 1024 * 1024;
    for _ in 1..5 {
        test_double_div(6.0f64, 6.0f64, init_csr, ops);
        test_double_div(6.0f64, 5.0f64, init_csr, ops);
        test_double_div(6.0f64, 6.0f64, init_csr, ops);
        test_double_div(6.0f64, 7.0f64, init_csr, ops);
    }
}
