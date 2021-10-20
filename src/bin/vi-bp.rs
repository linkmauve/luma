#![no_std]
#![feature(asm)]

extern crate luma_core;
extern crate luma_runtime;

use luma_core::gx::{bp::CopyFlag, Gx};
use luma_core::vi::{Vi, Xfb};

const EEVEE_RGB: &[u8] = include_bytes!("eevee.rgb");

fn main() {
    // Setup the video interface.
    let xfb = Xfb::allocate(640, 480);
    let (width, height) = (xfb.width(), xfb.height());
    let mut vi = Vi::setup(xfb);

    // Setup the GPU.
    let mut gx = Gx::setup();
    let efb = gx.efb();

    // Draw to the EFB using pokes.
    for y in 0..height {
        for x in 0..width {
            let r = EEVEE_RGB[((y * width) + x) as usize * 3] as u32;
            let g = EEVEE_RGB[((y * width) + x) as usize * 3 + 1] as u32;
            let b = EEVEE_RGB[((y * width) + x) as usize * 3 + 2] as u32;

            // EFB is apparently in XRGB8888.
            let pixel = (r << 16) | (g << 8) | b;
            efb.poke(x, y, pixel);
        }
    }

    // Copy the EFB to the XFB using BP.
    {
        let mut bp = gx.bp();
        bp.set_efb_coord(0, 0);
        bp.set_efb_size(width as u32, height as u32);
        bp.set_output(vi.xfb());
        bp.set_copy_clear_color(255, 0, 0, 0);
        bp.set_copy_clear_depth(0);
        bp.set_filter([0x666666, 0x666666, 0x666666, 0x666666]);
        bp.set_vertical_filter([0x00, 0x00, 0x15, 0x16, 0x15, 0x00, 0x00]);
        bp.do_copy(CopyFlag::CLEAR | CopyFlag::TO_XFB);
        bp.flush();
    }

    loop {}
}
