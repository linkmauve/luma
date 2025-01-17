//! ``luma_runtime`` is the runtime module of ``luma``.
//!
//! This module implements runtime functions and allocators required for ``no_std`` on the Wii.
//! This module also includes a crt0 implementation for bootstrapping the program.
//!
//! **NOTE**: This is currently in a very experimental state and is subject to change.
#![no_std]
#![feature(global_asm, lang_items, llvm_asm, alloc_error_handler)]

use core::{alloc::Layout, panic::PanicInfo};
use linked_list_allocator::LockedHeap;
#[allow(unused_imports)]
use luma_core::cache::*;

// Import linker symbols for allocator initialization.
extern "C" {
    pub static __stack_addr: usize;
    pub static __stack_end: usize;
}

// Global Allocator based on ``linked_list_allocator``.
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// crt0 Implementation
global_asm!(include_str!("../asm/crt0.S"));
global_asm!(include_str!("../asm/runtime.S"));
global_asm!(include_str!("../asm/system.S"));

/// This is the executable start function, which directly follows the entry point.
#[cfg_attr(not(test), lang = "start")]
#[cfg(not(test))]
fn start<T>(user_main: fn(), _argc: isize, _argv: *const *const u8) -> isize
where
    T: Termination,
{
    // Coerce the linker symbols to usize for allocator init.
    let stack_addr = unsafe { &__stack_addr } as *const _ as usize;
    let stack_end = unsafe { &__stack_end } as *const _ as usize;

    // Subtract the top of the stack from the bottom to get size.
    let out_size = stack_addr - stack_end;

    // Setup the allocator before the user_main is called.
    unsafe {
        ALLOCATOR
            .lock()
            .init(stack_addr, 24 * 1024 * 1024 - out_size);
    }

    // Jump to user defined main function.
    let user_main: fn() -> T = unsafe { core::mem::transmute(user_main) };
    user_main();

    panic!("main() cannot return");
}

/// Termination trait required for the start function.
#[cfg_attr(not(test), lang = "termination")]
trait Termination {}

/// This implementation does the bare minimum to satisfy the executable start function.
impl Termination for () {}

/// This function is called on panic.
#[cfg_attr(not(test), panic_handler)]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// This function is called when the allocator produces an error.
#[cfg_attr(not(test), alloc_error_handler)]
fn alloc_error_handler(_layout: Layout) -> ! {
    loop {}
}

/// Error handler personality language item (current no-op, to satisfy clippy).
#[cfg_attr(not(test), lang = "eh_personality")]
#[no_mangle]
extern "C" fn rust_eh_personality() {}
