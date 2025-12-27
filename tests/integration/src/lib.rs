// Integration test harness for riscmon.
//
// This crate spawns a QEMU process and drives the riscmon REPL over
// its stdin/stdout to test behavior. It's not a member of a
// workspace so it is not run by default with cargo test.
//
// Prerequisites:
//   - `qemu-system-riscv64` must be installed and on PATH
//   - The debug binary must be built first (e.g. `make debug`)

#[cfg(test)]
mod harness;

#[cfg(test)]
mod tests;
