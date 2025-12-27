pub(crate) use crate::{STACK_BOTTOM, STACK_TOP};

// -----------------------------------------------------------------------------
// Memory Map
// -----------------------------------------------------------------------------

pub(crate) const RAM_BASE: usize = 0x8000_0000;
pub(crate) const RAM_SIZE: usize = 128 * 1024 * 1024; // 128 MiB (QEMU "virt" default)
pub(crate) const RAM_END_INCLUSIVE: usize = RAM_BASE + RAM_SIZE - 1;
pub(crate) const BYTES_PER_LINE: usize = 16;

// -----------------------------------------------------------------------------
// Address Validation
// -----------------------------------------------------------------------------

/// Returns true if addr lies within the QEMU virt RAM window.
#[inline(always)]
pub(crate) fn is_in_ram(addr: usize) -> bool {
    (RAM_BASE..=RAM_END_INCLUSIVE).contains(&addr)
}

/// Determine whether a given address lies within riscmon’s own stack reservation.
#[inline(always)]
#[allow(dead_code)]
pub(crate) fn is_in_stack(addr: usize) -> bool {
    (STACK_BOTTOM..STACK_TOP).contains(&addr)
}

/// Check if an address is valid for “monitor state” operations (ex: `@ADDR`).
#[allow(dead_code)]
pub(crate) fn is_valid_monitor_address(addr: usize) -> bool {
    is_in_ram(addr) && !is_in_stack(addr)
}

/// Check whether two inclusive address ranges overlap.
#[allow(dead_code)]
pub(crate) fn ranges_overlap(a_start: usize, a_end: usize, b_start: usize, b_end: usize) -> bool {
    a_start <= b_end && b_start <= a_end
}
