# RISCMON

A minimal machine monitor for RISC-V systems, inspired by WozMon.

## About

Like Steve Wozniak's 256-byte monitor for the Apple I, Riscmon provides nothing
but the essentials:

- Memory inspection
- Memory modification
- Jump to address and execute

No filesystem. No processes. No abstractions. Raw access to the machine.

### Features

- [x] UART serial I/O REPL
- [x] Memory Read/Write
- [x] Memory Dump (with formatting options)
- [ ] REPL Command History
- [ ] Jump and Execute
- [ ] Debugging Capabilities

#### Core Commands

| Command | Description |
| --- | --- |
| `@` | Print current address |
| `@ADDR` | Set current address (hex) |
| `ADDR.ADDR` | Dump memory range (inclusive) |
| `ADDR+OFF` | Dump `OFF` bytes starting at `ADDR` |
| `ADDR.ADDR.as_str` | Dump range as ASCII (non-printables shown as `.`) |
| `ADDR+OFF.as_str` | Dump `OFF` bytes as ASCII |
| `ADDR: XX YY ...` | Write up to 32 bytes starting at `ADDR` (tokens are hex, and may be `aa` or `0xaa`) |

## Development

- **Language**: Rust (`no_std`) with minimal RISC-V assembly for boot
- **Target**: RISC-V 64-bit (`riscv64gc-unknown-none-elf`)
- **Platform**: QEMU `virt` machine (real hardware later)
- **Interface**: Serial console (UART)

> **Warn**: ONLY a 64bit RISC-V QEMU system is supported currently.

### Building

```console
# Install RISC-V target
rustup target add riscv64gc-unknown-none-elf

# Build
cargo build --release

# Run in QEMU
qemu-system-riscv64 -machine virt -nographic -bios none -kernel target/riscv64gc-unknown-none-elf/release/riscmon
```

Or, simply run:

```console
make run
```

## Philosophy

Riscmon was built as an exercise, for fun. It answers the question: *what is the
absolute minimum a computer needs to be programmable?*

The answer: memory access and a jump instruction.

In theory you could do all kinds of things with Riscmon, but practically
speaking it's probably best used as a debugger for other freestanding RISC-V
programs.

## Contributions

Fixes are welcome, but for new features (and generally large changes) create a
discussion so we can talk it over first.

## License

MIT
