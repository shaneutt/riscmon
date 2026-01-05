use crate::{
    memory,
    uart::{print, print_hex_u8, print_hex_u32, println, putc},
};

// -----------------------------------------------------------------------------
// Memory Information
// -----------------------------------------------------------------------------

/// Print the address ranges the monitor considers “safe/valid”.
pub(crate) fn print_valid_address_ranges() {
    print("valid RAM: ");
    print_hex_u32(memory::RAM_BASE as u32);
    print("..");
    print_hex_u32(memory::RAM_END_INCLUSIVE as u32);
    println("");
}

// Print the monitor's own stack reservation.
pub(crate) fn print_stack_range() {
    print("monitor stack: ");
    print_hex_u32(memory::STACK_BOTTOM as u32);
    print("..");
    print_hex_u32(memory::STACK_TOP as u32);
    println("");
}

// -----------------------------------------------------------------------------
// Memory Dumps
// -----------------------------------------------------------------------------

/// How a memory dump should render each byte.
#[derive(Copy, Clone)]
enum DumpFormat {
    Hex,
    Ascii,
}

/// Dump raw bytes as hex, 16 bytes per line.
///
/// Output format: `AAAAAAAA: xx xx xx ...`
pub(crate) fn print_memory_dump(start: usize, end: usize) {
    dump_memory(start, end, DumpFormat::Hex)
}

/// Dump bytes as lossy ASCII, 16 bytes per line.
///
/// Printable bytes are shown directly; everything else becomes `.`.
pub(crate) fn print_memory_dump_as_ascii(start: usize, end: usize) {
    dump_memory(start, end, DumpFormat::Ascii)
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

// Core memory dump implementation.
fn dump_memory(start: usize, end: usize, format: DumpFormat) {
    let mut addr = start;

    while addr <= end {
        // Print the line prefix (address).
        print_hex_u32(addr as u32);
        print(": ");

        // Compute this line’s inclusive end.
        let line_end = core::cmp::min(end, addr + (memory::BYTES_PER_LINE - 1));

        // Emit each byte in the chosen format.
        let mut a = addr;
        while a <= line_end {
            let b = unsafe { (a as *const u8).read_volatile() };

            match format {
                DumpFormat::Hex => {
                    print_hex_u8(b);
                    if a != line_end {
                        putc(b' ');
                    }
                }
                DumpFormat::Ascii => {
                    // ASCII printable range: space (0x20) through tilde (0x7e).
                    if (0x20..=0x7e).contains(&b) {
                        putc(b);
                    } else {
                        putc(b'.');
                    }
                }
            }

            a += 1;
        }

        println("");

        // Move to the next line start.
        addr = line_end.saturating_add(1); // saturating_add to avoid overflow mistakes
    }
}
