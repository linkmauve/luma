//! ``gx::xf`` module of ``luma_core``.
//!
//! Contains functions for configuring the transform unit.

use core::mem;
use crate::gx::Gx;

#[repr(u32)]
pub enum Projection {
    Perspective(f32, f32, f32, f32, f32, f32),
    Orthographic(f32, f32, f32, f32, f32, f32),
}

pub struct Xf<'a>(&'a mut Gx);

impl<'a> Xf<'a> {
    pub fn new(gx: &mut Gx) -> Xf {
        Xf(gx)
    }

    #[inline(always)]
    fn write(&mut self, register: u16, values: &[u32]) {
        self.0.wp.write_xf(register, values);
    }

    /// Set the 3D viewport
    #[inline(always)]
    pub fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32, near: f32, far: f32) {
        let x0 = unsafe { mem::transmute(width / 2.) };
        let y0 = unsafe { mem::transmute(-height / 2.) };
        let x1 = unsafe { mem::transmute((x + (width / 2.)) + 342.) };
        let y1 = unsafe { mem::transmute((y + (height / 2.)) + 342.) };
        // XXX: maybe figure out a different interface for that.
        let fp = (far * 16777215.) as u32;
        let z = ((far - near) * 16777215.) as u32;
        self.write(0x101a, &[x0, y0, z, x1, y1, fp]);
    }

    /// Set the projection matrix
    #[inline(always)]
    pub fn set_projection(&mut self, projection: Projection) {
        self.write(0x1020, &match projection {
            Projection::Perspective(a, b, c, d, e, f) => [unsafe { mem::transmute(a) }, unsafe { mem::transmute(b) }, unsafe { mem::transmute(c) }, unsafe { mem::transmute(d) }, unsafe { mem::transmute(e) }, unsafe { mem::transmute(f) }, 0],
            Projection::Orthographic(a, b, c, d, e, f) => [unsafe { mem::transmute(a) }, unsafe { mem::transmute(b) }, unsafe { mem::transmute(c) }, unsafe { mem::transmute(d) }, unsafe { mem::transmute(e) }, unsafe { mem::transmute(f) }, 1],
        });
    }

    /// Set the number of colour channels
    #[inline(always)]
    pub fn set_num_texgens(&mut self, num: u32) {
        self.write(0x1009, &[num]);
    }

    /// Set the number of colour channels
    #[inline(always)]
    pub fn set_num_chan(&mut self, num: u32) {
        self.write(0x1009, &[num]);
    }

    /// TODO
    #[inline(always)]
    pub fn set_0000_matrix(&mut self, mat: &[f32; 12]) {
        let mat_u32: alloc::vec::Vec<u32> = mat.iter().map(|x| unsafe { mem::transmute(x) }).collect();
        self.write(0x0000, &mat_u32);
    }
}
