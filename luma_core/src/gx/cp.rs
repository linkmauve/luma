//! ``gx::cp`` module of ``luma_core``.
//!
//! Contains functions for configuring the command processor.

use crate::gx::Gx;

pub struct Cp<'a>(&'a mut Gx);

impl<'a> Cp<'a> {
    pub fn new(gx: &mut Gx) -> Cp {
        Cp(gx)
    }

    #[inline(always)]
    fn write(&mut self, addr: u8, value: u32) {
        self.0.wp.write8(0x08);
        self.0.wp.write8(addr);
        self.0.wp.write32(value);
    }

    /// Set the 3D viewport
    #[inline(always)]
    pub fn set_vcd_lo(&mut self, a: u32) {
        let low = (1 << 9) | (1 << 13);
        let high = 0;
        self.write(0x50, low);
        self.write(0x60, high);
    }
}
