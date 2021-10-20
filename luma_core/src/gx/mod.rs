//! ``gx`` module of ``luma_core``.
//!
//! Contains functions for basic GX access.

use crate::io::{read32, write32};
use crate::{mfspr, mtspr};
use crate::allocate::{ptr_as_pinned_array, alloc_array_aligned};
use crate::PointerExt as _;
use alloc::boxed::Box;
use core::pin::Pin;
use core::ptr;
use core::mem;

pub mod bp;
pub mod cp;
pub mod xf;

const CP: *mut u16 = 0xcc00_0000 as *mut u16;
const PE: u32 = 0xcc00_1000;
const PI: *mut u32 = 0xcc00_3000 as *mut u32;
const BP: u8 = 0x61;

const WPAR_SIZE: usize = 128;
const FIFO_SIZE: usize = 4096;
const EFB_SIZE: usize = 1024 * 1024;

struct Wp {
    data: Pin<Box<[u8; WPAR_SIZE]>>,
}

impl Wp {
    const ADDRESS: *mut u8 = 0xcc00_8000 as *mut u8;

    /// Not public because there can be only one set for GX
    fn new() -> Wp {
        // Store to WPAR.
        mtspr!(Wp::ADDRESS.as_phys(), 921);
        // Reenable write gather pipe in HID2.  TODO: fix Dolphin to use this value.
        let hid2 = mfspr!(920);
        mtspr!(hid2 | 0x4000_0000, 920);
        let data = unsafe { ptr_as_pinned_array::<u8, WPAR_SIZE>(Wp::ADDRESS) };
        Wp { data }
    }

    /// Write a byte into the write-gather pipe
    #[inline(always)]
    pub fn write8(&mut self, value: u8) {
        let address = self.data.as_mut_ptr();
        // No need for eieio here, synchronisation is handled by the processor.
        unsafe { ptr::write_volatile(address, value) };
    }

    /// Write a half-word into the write-gather pipe
    #[inline(always)]
    pub fn write16(&mut self, value: u16) {
        let address = self.data.as_mut_ptr() as *mut u16;
        // No need for eieio here, synchronisation is handled by the processor.
        unsafe { ptr::write_volatile(address, value) };
    }

    /// Write a word into the write-gather pipe
    #[inline(always)]
    pub fn write32(&mut self, value: u32) {
        let address = self.data.as_mut_ptr() as *mut u32;
        // No need for eieio here, synchronisation is handled by the processor.
        unsafe { ptr::write_volatile(address, value) };
    }

    /// Write a word into the write-gather pipe
    #[inline(always)]
    pub fn writef32(&mut self, value: f32) {
        self.write32(unsafe { mem::transmute(value) });
    }

    /// Write something into the write-gather pipe
    #[inline(always)]
    pub fn write_xf(&mut self, register: u16, values: &[u32]) {
        self.write8(0x10);
        self.write16((values.len() - 1) as u16);
        self.write16(register);
        for value in values {
            self.write32(*value);
        }
    }

    /// Flush the write-gather pipe
    #[inline(always)]
    pub fn flush(&mut self) {
        let address = self.data.as_mut_ptr() as *mut u32;
        unsafe {
            ptr::write_volatile(address, 0u32);
            ptr::write_volatile(address, 0u32);
            ptr::write_volatile(address, 0u32);
            ptr::write_volatile(address, 0u32);
            ptr::write_volatile(address, 0u32);
            ptr::write_volatile(address, 0u32);
            ptr::write_volatile(address, 0u32);
            ptr::write_volatile(address, 0u32);
        }
    }
}

pub struct Efb {
    data: Pin<Box<[u32; EFB_SIZE]>>,
}

impl Efb {
    const ADDRESS: *mut u32 = 0x0800_0000 as *mut u32;

    /// Not public because there can be only one set for GX
    fn new() -> Efb {
        let data = unsafe { ptr_as_pinned_array::<u32, EFB_SIZE>(Efb::ADDRESS) };
        Efb { data }
    }

    /// Get the address of a pixel in EFB memory.
    fn addr(&self, x: usize, y: usize) -> u32 {
        assert!(x < 1024);
        assert!(y < 1024);
        let stride = 1024;
        let addr = (y * stride) + x;
        unsafe { self.data.as_ptr().offset(addr as isize) as u32 }
    }

    /// Read the pixel value at coordinate (x, y), in XRGB8888 format.
    pub fn peek(&self, x: usize, y: usize) -> u32 {
        read32(self.addr(x, y))
    }

    /// Write to the pixel at coordinate (x, y), in XRGB8888 format.
    pub fn poke(&mut self, x: usize, y: usize, pixel: u32) {
        write32(self.addr(x, y), pixel);
    }
}

pub struct Gx {
    wp: Wp,
    efb: Efb,
    fifo: Pin<Box<[u8; FIFO_SIZE]>>,
}

impl Gx {
    pub fn setup() -> Gx {
        let wp = Wp::new();
        let efb = Efb::new();

        let mut fifo = alloc_array_aligned::<FIFO_SIZE>();
        unsafe {
            let fifo_start = fifo.as_mut_ptr().as_phys();
            let fifo_end = fifo.as_mut_ptr().offset(FIFO_SIZE as isize).as_phys();

            // Setup the FIFO, first on the CPU…
            ptr::write(PI.offset(3), fifo_start);
            ptr::write(PI.offset(4), fifo_end);
            ptr::write(PI.offset(5), fifo_start);

            // … then on the GPU.
            ptr::write(CP.offset(0x01), 1 << 4);
            ptr::write(CP.offset(0x10), (fifo_start & 0xffff) as u16);
            ptr::write(CP.offset(0x11), (fifo_start >> 16) as u16);
            ptr::write(CP.offset(0x12), (fifo_end & 0xffff) as u16);
            ptr::write(CP.offset(0x13), (fifo_end >> 16) as u16);
            ptr::write(CP.offset(0x1a), (fifo_start & 0xffff) as u16);
            ptr::write(CP.offset(0x1b), (fifo_start >> 16) as u16);
            ptr::write(CP.offset(0x1c), (fifo_start & 0xffff) as u16);
            ptr::write(CP.offset(0x1d), (fifo_start >> 16) as u16);

            // And now enable it.
            ptr::write(CP.offset(0x01), (1 << 4) | (1 << 0));
        }

        Gx { wp, efb, fifo }
    }

    /// Obtain read/write access to the EFB
    pub fn efb(&mut self) -> &mut Efb {
        &mut self.efb
    }

    /// Initialise the BP, and get access to its commands
    pub fn bp(&mut self) -> bp::Bp {
        bp::Bp::new(self)
    }

    /// Initialise the CP, and get access to its commands
    pub fn cp(&mut self) -> cp::Cp {
        cp::Cp::new(self)
    }

    /// Initialise the XF, and get access to its commands
    pub fn xf(&mut self) -> xf::Xf {
        xf::Xf::new(self)
    }
    /// Send vertices
    #[inline(always)]
    pub fn invalidate_vertex_cache(&mut self) {
        self.wp.write8(0x48);
    }

    /// Send vertices
    #[inline(always)]
    pub fn send_vertices(&mut self) {
        self.wp.write8(0x90);
        self.wp.write16(3);

        self.wp.write8(0);
        self.wp.write8(0);

        self.wp.write8(1);
        self.wp.write8(1);

        self.wp.write8(2);
        self.wp.write8(2);

        /*
        self.wp.write8(0x80);
        self.wp.write16(4);

        self.wp.write32(0x4282_0000);
        self.wp.write32(0);
        self.wp.writef32(286.);
        self.wp.write32(0xdc00_00ff);
        self.wp.writef32(0.);
        self.wp.writef32(0.);

        self.wp.write32(0x4282_0000);
        self.wp.write32(0);
        self.wp.writef32(158.);
        self.wp.write32(0xdc00_00ff);
        self.wp.writef32(0.);
        self.wp.writef32(1.);

        self.wp.write32(0x4410_4000);
        self.wp.write32(0);
        self.wp.writef32(286.);
        self.wp.write32(0xdc00_00ff);
        self.wp.writef32(1.);
        self.wp.writef32(0.);

        self.wp.write32(0x4410_4000);
        self.wp.write32(0);
        self.wp.writef32(158.);
        self.wp.write32(0xdc00_00ff);
        self.wp.writef32(1.);
        self.wp.writef32(1.);
        */

        //42820000 00000000 438f0000 dc0000ff 0000000000000000
        //                  ↑ 286.
        //42820000 00000000 431e0000 dc0000ff 000000003f800000
        //                  ↑ 158.
        //44104000 00000000 438f0000 dc0000ff 3f80000000000000
        //44104000 00000000 431e0000 dc0000ff 3f8000003f800000
    }

    /// Flush the write-gather pipe
    #[inline(always)]
    pub fn flush(&mut self) {
        self.wp.flush();
    }
}
