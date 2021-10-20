//! ``luma_core`` is the core module of ``luma``.
//!
//! This module contains core processor features.
//!
//! **NOTE**: This is currently in a very experimental state and is subject to change.
#![no_std]
#![allow(unused_attributes)]
#![feature(global_asm, asm, box_into_boxed_slice, allocator_api)]

extern crate alloc;

trait PointerExt {
    fn as_cached(self) -> Self;
    fn as_uncached(self) -> Self;
    fn as_phys(self) -> u32;
}

impl<T> PointerExt for *const T {
    fn as_cached(self) -> Self {
        (self.as_phys() | 0x8000_0000) as *const T
    }

    fn as_uncached(self) -> Self {
        (self.as_phys() | 0xc000_0000) as *const T
    }

    fn as_phys(self) -> u32 {
        (self as u32) & 0x1fff_ffff
    }
}

impl<T> PointerExt for *mut T {
    fn as_cached(self) -> Self {
        (self.as_phys() | 0x8000_0000) as *mut T
    }

    fn as_uncached(self) -> Self {
        (self.as_phys() | 0xc000_0000) as *mut T
    }

    fn as_phys(self) -> u32 {
        (self as u32) & 0x1fff_ffff
    }
}

// Broadway Processor Utilities
pub mod processor;

// Broadway Register Utilities
pub mod register;

// Broadway Integer Utilities
pub mod integer;

// Broadway Load and Store Utilities
pub mod loadstore;

// Broadway I/O Utilities
pub mod io;

// Broadway Cache Subsystem
pub mod cache;

// Helper functions to allocate aligned memory on the heap
pub mod allocate;

// VI Subsystem
pub mod vi;
