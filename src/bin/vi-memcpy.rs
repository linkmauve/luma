#![no_std]
#![feature(asm)]

extern crate luma_core;
extern crate luma_runtime;

use luma_core::vi::{Vi, Xfb};

const EEVEE_YUYV: &[u8] = include_bytes!("eevee.yuyv");

fn main() {
    // Setup the video interface.
    let xfb = Xfb::allocate(640, 480);
    let mut vi = Vi::setup(xfb);

    // Draw to the XFB using the CPU.
    let xfb = vi.xfb();
    unsafe { core::ptr::copy(EEVEE_YUYV.as_ptr(), xfb.as_mut_ptr(), EEVEE_YUYV.len()) };

    loop {}
}
