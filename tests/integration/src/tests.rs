// Integration tests for riscmon.
//
// Each test is isolated by spawning its own QEMU process
//
// The debug binary must be built first (e.g. `make debug`).

use crate::harness::QemuHarness;

// Get the path to the kernel; Assumes relative path from Cargo.toml
fn kernel_path() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!(
        "{}/../../target/riscv64gc-unknown-none-elf/debug/riscmon",
        manifest_dir
    )
}

#[test]
fn test_boot_prompt() {
    println!("starting QEMU");
    let mut q = QemuHarness::spawn(&kernel_path());
    println!("sending empty command to confirm REPL is responsive");
    q.send("");
    let out = q.receive();
    let _ = out; // successful response
    println!("kernel booted");
}

#[test]
fn test_help_command() {
    println!("starting QEMU");
    let mut q = QemuHarness::spawn(&kernel_path());
    println!("sending 'help' command");
    q.send("help");
    let out = q.receive();
    println!("checking output contains expected header line");
    assert!(
        out.contains("memory management commands:"),
        "unexpected help output:\n{out}"
    );
}

#[test]
fn test_addr_get_default() {
    println!("starting QEMU");
    let mut q = QemuHarness::spawn(&kernel_path());
    println!("sending '@' with no argument to query the current address");
    q.send("@");
    let out = q.receive();
    println!("checking output contains default address 80000000");
    assert!(
        out.contains("80000000"),
        "expected default address 80000000 in output, got:\n{out}"
    );
}

#[test]
fn test_addr_set_and_get() {
    println!("starting QEMU");
    let mut q = QemuHarness::spawn(&kernel_path());
    println!("sending '@80200000' to set the current address");
    q.send("@80200000");
    let out = q.receive();
    println!("checking output echoes back 80200000");
    assert!(
        out.contains("80200000"),
        "expected address 80200000 in output, got:\n{out}"
    );
}

#[test]
fn test_write_and_dump() {
    println!("starting QEMU");
    let mut q = QemuHarness::spawn(&kernel_path());
    println!("writing 4 bytes (de ad be ef) at 80200000");
    q.send("80200000: de ad be ef");
    let _write_out = q.receive();
    println!("dumping bytes back from 80200000..80200003");
    q.send("80200000.80200003");
    let dump_out = q.receive();
    println!("checking dump output contains written bytes");
    assert!(
        dump_out.contains("de ad be ef"),
        "expected 'de ad be ef' in dump output, got:\n{dump_out}"
    );
}

#[test]
fn test_unknown_command() {
    println!("starting QEMU");
    let mut q = QemuHarness::spawn(&kernel_path());
    println!("sending bad command 'zzz'");
    q.send("zzz");
    let out = q.receive();
    println!("checking output contains 'unknown command' error");
    assert!(
        out.contains("unknown command"),
        "expected 'unknown command' in output, got:\n{out}"
    );
}
