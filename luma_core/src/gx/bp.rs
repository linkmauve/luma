//! ``gx::bp`` module of ``luma_core``.
//!
//! Contains functions for doing XFB copies and clearing EFB.

use crate::gx::{Gx, BP};
use crate::vi::Xfb;
use crate::PointerExt;

const ENABLE_ADDRESS: *mut u16 = 0xcc00_0002 as *mut u16;

bitflags::bitflags! {
    pub struct CopyFlag: u32 {
        const TO_XFB = 1 << 14;
        const CLEAR = 1 << 11;
    }
}

pub struct Bp<'a>(&'a mut Gx);

impl<'a> Drop for Bp<'a> {
    fn drop(&mut self) {
        unsafe {
            let value = core::ptr::read(ENABLE_ADDRESS);
            core::ptr::write(ENABLE_ADDRESS, value & !(1 << 5));
        }
    }
}

impl<'a> Bp<'a> {
    pub fn new(gx: &mut Gx) -> Bp {
        unsafe {
            let value = core::ptr::read(ENABLE_ADDRESS);
            core::ptr::write(ENABLE_ADDRESS, value | (1 << 5));
        }
        Bp(gx)
    }

    #[inline(always)]
    fn write(&mut self, register: u8, value: u32) {
        assert!(value <= 0x00ffffff);
        self.0.wp.write(BP);
        self.0.wp.write(((register as u32) << 24) | value);
    }

    /// TODO
    #[inline(always)]
    pub fn flush(&mut self) {
        self.0.wp.flush();
    }

    /// Set the src coordinates for the EFB copy
    #[inline(always)]
    pub fn set_efb_coord(&mut self, x: u32, y: u32) {
        let register = 0x49;
        assert!(x < 1024);
        assert!(y < 1024);
        let value = (y << 10) | x;
        self.write(register, value);
    }

    /// Set the src size for the EFB copy
    #[inline(always)]
    pub fn set_efb_size(&mut self, width: u32, height: u32) {
        let register = 0x4a;
        assert!(width > 0);
        assert!(height > 0);
        assert!(width < 1024);
        assert!(height < 1024);
        let value = ((height - 1) << 10) | (width - 1);
        self.write(register, value);
    }

    /// Set the address of the XFB in main RAM
    #[inline(always)]
    fn set_xfb_addr(&mut self, addr: *mut u8) {
        let register = 0x4b;
        let value = (addr.as_phys() as u32) >> 5;
        self.write(register, value);
    }

    /// Set the stride of the XFB, in bytes
    #[inline(always)]
    fn set_xfb_stride(&mut self, stride: u32) {
        let register = 0x4d;
        assert_eq!(stride & 0x1f, 0);
        let stride_in_cachelines = stride >> 5;
        assert!(stride_in_cachelines < 32768);
        self.write(register, stride_in_cachelines);
    }

    /// Set the scale of the XFB, must be 256 usually.
    #[inline(always)]
    fn set_xfb_scale(&mut self, scale: u32) {
        let register = 0x4e;
        assert!(scale < 1024);
        self.write(register, scale);
    }

    /// Define the output framebuffer for the copy.
    #[inline(always)]
    pub fn set_output(&mut self, xfb: &mut Xfb) {
        self.set_xfb_addr(xfb.as_mut_ptr());
        self.set_xfb_stride(xfb.stride() as u32);
        self.set_xfb_scale(256);
    }

    /// Set the clear color when EFB clearing is enabled
    #[inline(always)]
    pub fn set_copy_clear_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        let high = 0x4f;
        let low = 0x50;
        self.write(high, ((a as u32) << 8) | (r as u32));
        self.write(low, ((g as u32) << 8) | (b as u32));
    }

    /// Set the clear depth value when EFB clearing is enabled
    #[inline(always)]
    pub fn set_copy_clear_depth(&mut self, depth: u32) {
        let register = 0x51;
        assert!(depth < 16777216);
        self.write(register, depth);
    }

    /// Set the clear depth value when EFB clearing is enabled
    #[inline(always)]
    pub fn set_filter(&mut self, filters: [u32; 4]) {
        for (i, filter) in filters.iter().enumerate() {
            let register = (i as u8) + 1;
            self.write(register, *filter);
        }
    }

    /// Set the clear depth value when EFB clearing is enabled
    #[inline(always)]
    pub fn set_vertical_filter(&mut self, filters: [u8; 7]) {
        let high = 0x53;
        let low = 0x54;
        for f in filters {
            assert!(f < 64);
        }
        let value =
              ((filters[3] as u32) << 18)
            | ((filters[2] as u32) << 12)
            | ((filters[1] as u32) << 6)
            | (filters[0] as u32);
        self.write(high, value);
        let value =
              ((filters[6] as u32) << 12)
            | ((filters[5] as u32) << 6)
            | (filters[4] as u32);
        self.write(low, value);
    }

    /// Set the clear depth value when EFB clearing is enabled
    #[inline(always)]
    pub fn do_copy(&mut self, flags: CopyFlag) {
        let register = 0x52;
        self.write(register, flags.bits);
    }

    /// TODO
    #[inline(always)]
    pub fn do_stuff(&mut self) {
        let register = 0x40;
        let value = 0x17;
        self.write(register, value);
        let register = 0x41;
        let value = 0x4bc;
        self.write(register, value);
        let register = 0x43;
        let value = 0x40;
        self.write(register, value);
        self.flush();
    }

    /// TODO
    #[inline(always)]
    pub fn do_stuff2(&mut self) {
        let register = 0x0f;
        let value = 0;
        self.write(register, value);
        let register = 0x66;
        let value = 0x1000;
        self.write(register, value);
        let value = 0x1100;
        self.write(register, value);
        let register = 0x0f;
        let value = 0;
        self.write(register, value);
    }
}
