// -----------------------------------------------------------------------------
// Memory-Mapped Input/Output (MMIO)
// -----------------------------------------------------------------------------

// QEMU "virt" exposes a "finisher" MMIO device used to end emulation.
// Writing 0x5555 makes QEMU exit ("power off").
const QEMU_FINISHER: usize = 0x0010_0000;

/// Power off the system.
pub(crate) fn poweroff() -> ! {
    unsafe {
        (QEMU_FINISHER as *mut u32).write_volatile(0x5555);
    }
    loop {
        core::hint::spin_loop();
    }
}
