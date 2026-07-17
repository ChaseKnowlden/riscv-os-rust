CARGO ?= cargo
QEMU ?= qemu-system-riscv64

TARGET := riscv64gc-unknown-none-elf
HOST_TARGET := $(shell rustc --print host-tuple)
KERNEL := target/$(TARGET)/debug/riscvrust-kernel

GDB_PORT ?= 1234
QEMU_ARGS ?=
QEMU_BASE_ARGS := \
	-machine virt \
	-cpu rv64 \
	-smp 1 \
	-m 128M \
	-bios default \
	-kernel $(KERNEL) \
	-nographic \
	-no-reboot

.PHONY: build run debug test

build:
	$(CARGO) build -p riscvrust-kernel --bin riscvrust-kernel

run: build
	$(QEMU) $(QEMU_BASE_ARGS) $(QEMU_ARGS)

debug: build
	$(QEMU) $(QEMU_BASE_ARGS) -S -gdb tcp::$(GDB_PORT) $(QEMU_ARGS)

test:
	$(CARGO) test --workspace --target $(HOST_TARGET)
