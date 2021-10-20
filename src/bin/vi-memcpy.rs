#![no_std]
#![feature(asm)]

extern crate luma_core;
extern crate luma_runtime;

use luma_core::vi::{Vi, Xfb};

const EEVEE_YUYV: &[u8] = include_bytes!("eevee.yuyv");

unsafe fn dcbz(address: *mut u8) {
    asm!("dcbz 0,{0}",
        in(reg) address);
}

unsafe fn small_memcpy(src: *const u8, dst: *mut u8, len: usize) {
    /*
    asm!(
        "mtxer {2};
        lswx r5,{0};
        stswx r5,{1};",
        in(reg) src,
        in(reg) dst,
        in(reg) len);
    */
}

unsafe fn my_memcpy(src: *const u8, dst: *mut u8, len: usize) {
    let mut lines = len >> 5;
    if ((dst as u32) & 0x1f) != 0 {
        small_memcpy(src, dst, len);
    }
    let (src, dst) = if ((src as u32) & 0x7) != 0 {
        let mut src = src as *const u32;
        let mut dst = dst as *mut u32;
        while lines > 0 {
            let r0 = *src.offset(0);
            let r1 = *src.offset(1);
            let r2 = *src.offset(2);
            let r3 = *src.offset(3);
            let r4 = *src.offset(4);
            let r5 = *src.offset(5);
            let r6 = *src.offset(6);
            let r7 = *src.offset(7);
            dcbz(dst as *mut u8);
            *dst.offset(0) = r0;
            *dst.offset(1) = r1;
            *dst.offset(2) = r2;
            *dst.offset(3) = r3;
            *dst.offset(4) = r4;
            *dst.offset(5) = r5;
            *dst.offset(6) = r6;
            *dst.offset(7) = r7;
            src = src.offset(8);
            dst = dst.offset(8);
            lines -= 1;
        }
        (src as *const u8, dst as *mut u8)
    } else {
        let mut src = src as *const f64;
        let mut dst = dst as *mut f64;
        while lines > 0 {
            let f0 = *src.offset(0);
            let f1 = *src.offset(1);
            let f2 = *src.offset(2);
            let f3 = *src.offset(3);
            dcbz(dst as *mut u8);
            *dst.offset(0) = f0;
            *dst.offset(1) = f1;
            *dst.offset(2) = f2;
            *dst.offset(3) = f3;
            src = src.offset(4);
            dst = dst.offset(4);
            lines -= 1;
        }
        (src as *const u8, dst as *mut u8)
    };
    let len = len & 0x1f;
    if len > 0 {
        core::ptr::copy(src, dst, len);
    }
}

fn main() {
    // Setup the video interface.
    let xfb = Xfb::allocate(640, 480);
    let mut vi = Vi::setup(xfb);

    // Draw to the XFB using the CPU.
    let xfb = vi.xfb();
    // unsafe { core::ptr::copy(EEVEE_YUYV.as_ptr(), xfb.as_mut_ptr(), EEVEE_YUYV.len()) };

    unsafe { my_memcpy(EEVEE_YUYV.as_ptr(), xfb.as_mut_ptr(), EEVEE_YUYV.len()) };

    loop {}
}
