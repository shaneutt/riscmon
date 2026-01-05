use crate::hex;

// -----------------------------------------------------------------------------
// UART register offsets and bitfields (16550-compatible)
// -----------------------------------------------------------------------------

// QEMU virt machine UART base address (memory-mapped I/O).
//
// QEMU exposes a 16550-compatible UART at this address.
const UART_BASE: usize = 0x1000_0000;

// Offset (in bytes) from UART_BASE for the RHR/THR register.
//
// 16550 quirk: offset 0 is *multiplexed*.
// - read  @ +0 => RHR (Receive Holding Register: next received byte)
// - write @ +0 => THR (Transmit Holding Register: byte to send)
const UART_RHR_THR: usize = 0x00;

// Offset (in bytes) from UART_BASE for the LSR (Line Status Register).
//
// We poll LSR bits to implement blocking RX/TX without interrupts.
const UART_LSR: usize = 0x05;

// LSR bit mask: DR (Data Ready). When set, RHR has at least one byte to read.
const LSR_DATA_READY: u8 = 1 << 0;

// LSR bit mask: THRE (THR Empty). When set, THR can accept the next TX byte.
const LSR_THR_EMPTY: u8 = 1 << 5;

// -----------------------------------------------------------------------------
// UART I/O functions
// -----------------------------------------------------------------------------

/// Read a single byte from UART (blocking).
///
/// This is a polled implementation: we spin until LSR indicates RX data is ready.
pub(crate) fn getc() -> u8 {
    while unsafe { ((UART_BASE + UART_LSR) as *const u8).read_volatile() } & LSR_DATA_READY == 0 {
        core::hint::spin_loop();
    }

    unsafe { ((UART_BASE + UART_RHR_THR) as *const u8).read_volatile() }
}

/// Write a single byte to UART (blocking).
///
/// We spin until LSR indicates THR is empty, then write the byte to THR.
pub(crate) fn putc(c: u8) {
    while unsafe { ((UART_BASE + UART_LSR) as *const u8).read_volatile() } & LSR_THR_EMPTY == 0 {
        core::hint::spin_loop();
    }

    let uart = (UART_BASE + UART_RHR_THR) as *mut u8;
    unsafe {
        // Volatile is required for MMIO: the compiler must not optimize this away.
        uart.write_volatile(c);
    }
}

/// Print a string to UART.
///
/// We iterate bytes to avoid allocation/formatting machinery.
pub(crate) fn print(s: &str) {
    for b in s.bytes() {
        putc(b);
    }
}

/// Print a string followed by newline (CRLF).
///
/// Many serial terminals expect CRLF (`\r\n`) rather than just LF (`\n`).
pub(crate) fn println(s: &str) {
    print(s);
    print("\r\n");
}

/// Print a byte as two lowercase hex digits (no prefix).
pub(crate) fn print_hex_u8(v: u8) {
    putc(hex::hex_digit((v >> 4) & 0x0f));
    putc(hex::hex_digit(v & 0x0f));
}

/// Print a 32-bit value as eight lowercase hex digits (no prefix).
pub(crate) fn print_hex_u32(v: u32) {
    for shift in (0..32).step_by(4).rev() {
        let nibble = ((v >> shift) & 0x0f) as u8;
        putc(hex::hex_digit(nibble));
    }
}

/// Clear the user's terminal via ANSI escape sequences.
pub(crate) fn clear_screen() {
    print("\x1b[2J\x1b[H");
}
