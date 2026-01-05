use crate::{
    INFO_BANNER, STACK_BOTTOM, STACK_TOP, hex, memory,
    repl::{
        meminfo::{
            print_memory_dump, print_memory_dump_as_ascii, print_stack_range,
            print_valid_address_ranges,
        },
        runner::{FINDME, get_current_addr, set_current_addr},
    },
    system,
    uart::{clear_screen, print, print_hex_u32, println},
};

// -----------------------------------------------------------------------------
// Command Handler
// -----------------------------------------------------------------------------

// Parse and execute a single command line.
pub(crate) fn handle_command(line: &[u8]) {
    let Ok(s) = core::str::from_utf8(line) else {
        println("error: non-utf8 input");
        return;
    };

    match parse_command(s.trim()) {
        None => {}
        Some(Command::Help) => cmd_help(),
        Some(Command::Info) => cmd_info(),
        Some(Command::Clear) => cmd_clear(),
        Some(Command::Poweroff) => cmd_poweroff(),
        Some(Command::AddrGet) => cmd_addr_get(),
        Some(Command::AddrSet { addr }) => cmd_addr_set(addr),
        Some(Command::Write { start, bytes, len }) => cmd_write(start, &bytes[..len]),
        Some(Command::Dump { start, end, ascii }) => cmd_dump(start, end, ascii),
        Some(Command::Noop) => {}
        Some(Command::Unknown) => println("unknown command (try 'help')"),
    }
}

// -----------------------------------------------------------------------------
// Command Parser
// -----------------------------------------------------------------------------

#[derive(Copy, Clone)]
enum Command {
    Help,
    Info,
    Clear,
    Poweroff,
    AddrGet,
    AddrSet {
        addr: usize,
    },
    Write {
        start: usize,
        bytes: [u8; MAX_WRITE_BYTES],
        len: usize,
    },
    Dump {
        start: usize,
        end: usize,
        ascii: bool,
    },
    Noop,
    Unknown,
}

fn parse_command(cmd: &str) -> Option<Command> {
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return None;
    }

    match cmd {
        "help" => Some(Command::Help),
        "info" => Some(Command::Info),
        "clear" | "reset" => Some(Command::Clear),
        "poweroff" | "q" => Some(Command::Poweroff),
        _ => parse_address_cmd(cmd)
            .or_else(|| parse_write_cmd(cmd))
            .or_else(|| parse_dump_cmd(cmd))
            .or(Some(Command::Unknown)),
    }
}

fn parse_address_cmd(cmd: &str) -> Option<Command> {
    match cmd.strip_prefix('@') {
        None => None,
        Some(rest) => {
            let rest = rest.trim();
            match rest {
                "" => Some(Command::AddrGet),
                _ => match hex::parse_hex_usize(rest) {
                    Some(addr) => Some(Command::AddrSet { addr }),
                    None => {
                        println("error: invalid address");
                        Some(Command::Noop)
                    }
                },
            }
        }
    }
}

fn parse_write_cmd(cmd: &str) -> Option<Command> {
    let (addr_s, data_s) = cmd.split_once(':')?;
    let addr_s = addr_s.trim();
    let data_s = data_s.trim();

    if addr_s.is_empty() {
        println("error: invalid address");
        return Some(Command::Noop);
    }

    if data_s.is_empty() {
        println("error: no data");
        return Some(Command::Noop);
    }

    let Some(start) = hex::parse_hex_usize(addr_s) else {
        println("error: invalid address");
        return Some(Command::Noop);
    };

    let Ok((bytes, len)) = parse_write_bytes(data_s) else {
        return Some(Command::Noop);
    };

    Some(Command::Write { start, bytes, len })
}

fn parse_dump_cmd(cmd: &str) -> Option<Command> {
    let (ascii, core) = match cmd.strip_suffix(".as_str") {
        Some(prefix) => (true, prefix),
        None => (false, cmd),
    };

    match core.split_once('+') {
        Some((start_s, off_s)) => {
            let start = match hex::parse_hex_usize(start_s) {
                Some(v) => v,
                None => {
                    println("error: invalid start address");
                    return Some(Command::Noop);
                }
            };
            let off = match hex::parse_hex_usize(off_s) {
                Some(v) => v,
                None => {
                    println("error: invalid offset");
                    return Some(Command::Noop);
                }
            };
            let (start, end) = match range_from_len(start, off) {
                Ok(r) => r,
                Err(()) => return Some(Command::Noop),
            };
            Some(Command::Dump { start, end, ascii })
        }
        None => match core.split_once('.') {
            Some((start_s, end_s)) => {
                let start = match hex::parse_hex_usize(start_s) {
                    Some(v) => v,
                    None => {
                        println("error: invalid start address");
                        return Some(Command::Noop);
                    }
                };
                let end = match hex::parse_hex_usize(end_s) {
                    Some(v) => v,
                    None => {
                        println("error: invalid end address");
                        return Some(Command::Noop);
                    }
                };

                match validate_range(start, end) {
                    Ok(()) => Some(Command::Dump { start, end, ascii }),
                    Err(()) => Some(Command::Noop),
                }
            }
            None => None,
        },
    }
}

// -----------------------------------------------------------------------------
// Commands
// -----------------------------------------------------------------------------

fn cmd_help() {
    println("shell commands:");
    println("  help          - show help information");
    println("  info          - show monitor info");
    println("  clear (reset) - clear the terminal");
    println("  poweroff (q)  - power off the system");
    println("");
    println("memory management commands:");
    println("  @         - get current address");
    println("  @ADDR     - set current address (e.g. @80001000)");
    println("  ADDR.ADDR - dump memory range (e.g. 80002004.80002007)");
    println("  ADDR+OFF  - dump OFF bytes from ADDR (e.g. 80002004+4)");
    println("  ADDR.ADDR.as_str - dump range as ASCII (e.g. 80001000.8000103f.as_str)");
    println("  ADDR+OFF.as_str  - dump as ASCII (e.g. 80001000+40.as_str)");
    println("  ADDR: XX YY .. - write up to 32 bytes (e.g. 80001000: 48 69 21)");
}

fn cmd_info() {
    println(INFO_BANNER);

    print("stack: ");
    print_hex_u32(STACK_BOTTOM as u32);
    print(".. ");
    print_hex_u32(STACK_TOP as u32);
    println("");

    print("current: ");
    print_hex_u32(get_current_addr() as u32);
    println("");

    print("FINDME @ ");
    let addr = (&FINDME as *const [u8; 8]) as usize;
    print_hex_u32(addr as u32);
    println("");
}

fn cmd_clear() {
    clear_screen();
}

fn cmd_poweroff() -> ! {
    system::poweroff()
}

fn cmd_addr_get() {
    print_hex_u32(get_current_addr() as u32);
    println("");
}

fn cmd_addr_set(addr: usize) {
    if !memory::is_in_ram(addr) || memory::is_in_stack(addr) {
        println("error: invalid memory address");
        print_valid_address_ranges();
        return;
    }
    set_current_addr(addr);
    cmd_addr_get();
}

fn cmd_write(start: usize, bytes: &[u8]) {
    let len = bytes.len();
    if len == 0 {
        println("error: no data");
        return;
    }

    let Some(end) = start.checked_add(len - 1) else {
        println("error: address overflow");
        return;
    };

    if !memory::is_in_ram(start) || !memory::is_in_ram(end) {
        println("error: address out of range");
        print_valid_address_ranges();
        return;
    }

    if memory::ranges_overlap(start, end, STACK_BOTTOM, STACK_TOP) {
        println("error: write into riscmon stack not allowed");
        print_stack_range();
        return;
    }

    for (i, &b) in bytes.iter().enumerate() {
        unsafe {
            ((start + i) as *mut u8).write_volatile(b);
        }
    }

    if end < memory::RAM_END_INCLUSIVE {
        set_current_addr(end + 1);
    }

    cmd_addr_get();
}

fn cmd_dump(start: usize, end: usize, ascii: bool) {
    if let Err(()) = validate_range(start, end) {
        return;
    }

    match ascii {
        true => print_memory_dump_as_ascii(start, end),
        false => print_memory_dump(start, end),
    }
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

const MAX_DUMP_BYTES: usize = 256;
const MAX_WRITE_BYTES: usize = 32;

fn parse_write_bytes(data_s: &str) -> Result<([u8; MAX_WRITE_BYTES], usize), ()> {
    let mut bytes = [0u8; MAX_WRITE_BYTES];
    let mut n = 0usize;

    for token in data_s.split_whitespace() {
        if n == bytes.len() {
            println("error: too many bytes (max 32)");
            return Err(());
        }

        let Some(b) = hex::parse_hex_u8_token(token) else {
            println("error: invalid byte (use two hex digits like 0a)");
            return Err(());
        };

        bytes[n] = b;
        n += 1;
    }

    if n == 0 {
        println("error: no data");
        return Err(());
    }

    Ok((bytes, n))
}

fn range_from_len(start: usize, off: usize) -> Result<(usize, usize), ()> {
    if off == 0 {
        println("error: offset is 0");
        return Err(());
    }

    if off > MAX_DUMP_BYTES {
        println("error: range too large (max 256 bytes)");
        return Err(());
    }

    let Some(end) = start.checked_add(off - 1) else {
        println("error: address overflow");
        return Err(());
    };

    validate_range(start, end)?;

    Ok((start, end))
}

fn validate_range(start: usize, end: usize) -> Result<(), ()> {
    if end < start {
        println("error: end < start");
        return Err(());
    }

    if !memory::is_in_ram(start) || !memory::is_in_ram(end) {
        println("error: address out of range");
        print_valid_address_ranges();
        return Err(());
    }

    let Some(len) = end.checked_sub(start).and_then(|d| d.checked_add(1)) else {
        println("error: address overflow");
        return Err(());
    };

    if len > MAX_DUMP_BYTES {
        println("error: range too large (max 256 bytes)");
        return Err(());
    }

    Ok(())
}
