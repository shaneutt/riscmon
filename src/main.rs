#![no_std]
#![no_main]

mod hex;
mod memory;
mod repl;
mod system;
mod uart;

use core::arch::naked_asm;
use core::panic::PanicInfo;

// -----------------------------------------------------------------------------
// Stack
// -----------------------------------------------------------------------------

pub(crate) const STACK_TOP: usize = 0x8010_0000;
pub(crate) const STACK_SIZE: usize = 16 * 1024; // 16 KiB reserved for riscmon's stack
pub(crate) const STACK_BOTTOM: usize = STACK_TOP - STACK_SIZE;

// -----------------------------------------------------------------------------
// Main
// -----------------------------------------------------------------------------

/// Riscmon information banner.
pub(crate) const INFO_BANNER: &str = concat!("riscmon v", env!("CARGO_PKG_VERSION"));

/// Entry point.
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub(crate) unsafe extern "C" fn _start() {
    naked_asm!(
        "li sp, {stack_top}", // set stack pointer
        "call main", // jump to our rust main
        "1: j 1b", // halt if main ever returns (it shouldn't)
        stack_top = const STACK_TOP,
    );
}

/// Main entry point (called from _start after stack setup).
///
/// Returns `!` because the monitor REPL loops until poweroff.
#[unsafe(no_mangle)]
pub(crate) extern "C" fn main() -> ! {
    repl::run()
}

// -----------------------------------------------------------------------------
// Handlers
// -----------------------------------------------------------------------------

// Panic handler.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart::print("PANIC!\n");
    loop {}
}
