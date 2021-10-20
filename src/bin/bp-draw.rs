//! This is an example of how to draw things to the screen using Luma.
//!
//! The drawing has been ported from Weston’s clients/simple-shm.c

#![no_std]

extern crate luma_core;
extern crate luma_runtime;

use luma_core::gx::{bp::CopyFlag, Gx, Efb};
use luma_core::vi::{Vi, Xfb};
use core::fmt::Write;

/// Fill the padding with white pixels.
#[inline(never)]
fn fill_padding(efb: &mut Efb, width: usize, height: usize) {
    for y in 0..height {
        if y < 20 || y >= height - 20 {
            for x in 0..width {
                efb.poke(x, y, 0x00ffffff);
            }
        } else {
            for x in 0..20 {
                efb.poke(x, y, 0x00ffffff);
                efb.poke(x + width - 20, y, 0x00ffffff);
            }
        }
    }
}

/// Ported from Weston’s clients/simple-shm.c
fn paint_pixels(efb: &mut Efb, padding: usize, width: usize, height: usize, time: usize) {
    let halfh = padding + (height - padding * 2) / 2;
    let halfw = padding + (width - padding * 2) / 2;

    // Squared radii thresholds
    let mut or = (if halfw < halfh { halfw } else { halfh }) - 8;
    let mut ir = or - 32;
    or *= or;
    ir *= ir;

    for y in padding..(height - padding) {
        let y2 = (y - halfh) * (y - halfh);

        for x in padding..(width - padding) {
            let v;

            /* squared distance from center */
            let r2 = (x - halfw) * (x - halfw) + y2;

            if r2 < ir {
                v = (r2 / 32 + time / 4) * 0x0080401;
            } else if r2 < or {
                v = (y + time / 2) * 0x0080401;
            } else {
                v = (x + time) * 0x0080401;
            }

            // TODO: avoid using EFB pokes, these are slow.  Instead, use a textured draw.
            efb.poke(x, y, v as u32);
        }
    }
}

fn main() {
    // Setup the video interface.
    let xfb = Xfb::allocate(640, 480);
    let (width, height) = (xfb.width(), xfb.height());
    let mut vi = Vi::setup(xfb);

    // Setup the GPU.
    let mut gx = Gx::setup();

    fill_padding(gx.efb_mut(), 640, 480);

    let mut i = 0;
    loop {
        {
            let efb = gx.efb_mut();
            paint_pixels(efb, 20, 640, 480, i);
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

        i += 1;
    }
}
