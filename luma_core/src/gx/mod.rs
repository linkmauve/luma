//! ``gx`` module of ``luma_core``.
//!
//! Contains functions for basic GX access.

use core::arch::asm;
use crate::io::{read32, write32};
use crate::{mfspr, mtspr};
use crate::allocate::{ptr_as_pinned_array, alloc_array_aligned};
use crate::PointerExt as _;
use alloc::boxed::Box;
use core::pin::Pin;
use core::ptr;

pub mod bp;

const CP: *mut u16 = 0xcc00_0000 as *mut u16;
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

    /// Write a value into the write-gather pipe
    #[inline(always)]
    pub fn write<T>(&mut self, value: T) {
        let address = self.data.as_mut_ptr() as *mut T;
        // No need for eieio here, synchronisation is handled by the processor.
        unsafe { ptr::write_volatile(address, value) };
    }

    /*
    /// Write something for XF into the write-gather pipe
    #[inline(always)]
    pub fn write_xf(&mut self, register: u16, values: &[u32]) {
        self.write(0x10_u8);
        self.write((values.len() - 1) as u16);
        self.write(register);
        for value in values {
            self.write(*value);
        }
    }
    */

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
    #[inline(always)]
    fn addr(&self, x: usize, y: usize) -> u32 {
        assert!(x < 640);
        assert!(y < 528);
        let stride = 1024;
        let addr = (y * stride) + x;
        unsafe { self.data.as_ptr().offset(addr as isize) as u32 }
    }

    /// Read the pixel value at coordinate (x, y), in XRGB8888 format.
    #[inline(always)]
    pub fn peek(&self, x: usize, y: usize) -> u32 {
        read32(self.addr(x, y))
    }

    /// Write to the pixel at coordinate (x, y), in XRGB8888 format.
    #[inline(always)]
    pub fn poke(&mut self, x: usize, y: usize, pixel: u32) {
        write32(self.addr(x, y), pixel);
    }
}

/// Main struct to access the GX, which is Flipper/Hollywood’s main GPU and Latte’s secondary GPU.
pub struct Gx {
    wp: Wp,
    efb: Efb,
    fifo: Pin<Box<[u8; FIFO_SIZE]>>,
}

impl Gx {
    /// Initialises both the CPU-side and GPU-side, the write-gather pipe, the EFB, and the FIFO.
    // TODO: this still doesn’t work on hardware…
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

    /// Obtain read only access to the EFB
    pub fn efb(&self) -> &Efb {
        &self.efb
    }

    /// Obtain read/write access to the EFB
    pub fn efb_mut(&mut self) -> &mut Efb {
        &mut self.efb
    }

    /// Initialise the BP, and get access to its commands
    pub fn bp(&mut self) -> bp::Bp {
        bp::Bp::new(self)
    }
}
