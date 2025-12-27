HOST_TARGET := $(shell rustc -vV | sed -n 's|host: ||p')
TARGET := riscv64gc-unknown-none-elf

BINARY_NAME := riscmon

DEBUG_DIR := target/$(TARGET)/debug
RELEASE_DIR := target/$(TARGET)/release

QEMU := qemu-system-riscv64
QEMU_FLAGS := -machine virt -nographic -bios none

.PHONY: all debug release clean run run.release test.integration

all: debug

debug:
	cargo build --target $(TARGET)

release:
	cargo build --target $(TARGET) --release

clean:
	cargo clean

run: debug
	$(QEMU) $(QEMU_FLAGS) -kernel $(DEBUG_DIR)/$(BINARY_NAME)

run.release: release
	$(QEMU) $(QEMU_FLAGS) -kernel $(RELEASE_DIR)/$(BINARY_NAME)

test.integration: debug
	cargo test --manifest-path tests/integration/Cargo.toml --target $(HOST_TARGET)
