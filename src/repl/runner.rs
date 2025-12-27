use crate::{
    INFO_BANNER, memory,
    repl::commands::handle_command,
    uart,
    uart::{print, println},
};
use core::sync::atomic::{AtomicUsize, Ordering};

// -----------------------------------------------------------------------------
// REPL State
// -----------------------------------------------------------------------------

// A "known bytes" marker, to find in memory.
#[allow(dead_code)]
#[used]
#[unsafe(no_mangle)]
pub(crate) static FINDME: [u8; 8] = *b"foundme!";

// The monitor keeps a "current address".
static CURRENT_ADDR: AtomicUsize = AtomicUsize::new(memory::RAM_BASE);

// Set the monitor's “current address” (used as an implicit base for some workflows).
pub(crate) fn set_current_addr(addr: usize) {
    CURRENT_ADDR.store(addr, Ordering::Relaxed);
}

// Get the monitor's current address.
pub(crate) fn get_current_addr() -> usize {
    CURRENT_ADDR.load(Ordering::Relaxed)
}

// -----------------------------------------------------------------------------
// REPL Runner
// -----------------------------------------------------------------------------

// Run the REPL loop.
pub(crate) fn run() -> ! {
    println(INFO_BANNER);

    let mut line_buf = [0u8; LINE_BUF_CAP];
    loop {
        prompt();
        let n = read_line(&mut line_buf);
        handle_command(&line_buf[..n]);
    }
}

// -----------------------------------------------------------------------------
// REPL Line Editor
// -----------------------------------------------------------------------------

const LINE_BUF_CAP: usize = 128;

// Print the REPL prompt.
fn prompt() {
    print("> ");
}

// Read a line of input into a buffer.
//
// Returns the number of bytes read (excluding newline). Will return
// 0 if the line was cancelled via Ctrl+C, or should otherwise be ignored.
fn read_line(buf: &mut [u8]) -> usize {
    let mut len = 0;

    loop {
        let b = uart::getc();
        match b {
            0x03 => {
                // Matched: Ctrl+C (ETX).
                //
                // Cancel the current input line and return 0 to treat it as an
                // empty command.
                print("^C\r\n");
                return 0;
            }
            b'\r' | b'\n' => {
                // Matched: Enter (CR or LF).
                //
                // Finish editing, print a newline, and return the number of
                // bytes currently in the buffer.
                print("\r\n");
                return len;
            }
            0x08 | 0x7f => {
                // Matched: Backspace (0x08) or Delete (0x7f).
                //
                // Delete one byte from the input (if any) and echo the erase.
                if len > 0 {
                    len -= 1;
                    print("\x08 \x08");
                }
            }
            _ => {
                // Matched: any other input byte.
                //
                // Accept printable ASCII into the line buffer (with echo);
                // ignore control bytes and ignore extra bytes once the buffer is full.
                if b >= 0x20 && b <= 0x7e {
                    if len < buf.len() {
                        buf[len] = b;
                        len += 1;
                        uart::putc(b);
                    }
                }
            }
        }
    }
}
