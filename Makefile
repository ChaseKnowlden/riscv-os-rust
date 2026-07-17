CARGO ?= cargo
QEMU ?= qemu-system-riscv64

TARGET := riscv64gc-unknown-none-elf
HOST_TARGET := $(shell rustc --print host-tuple)
KERNEL := target/$(TARGET)/debug/riscvrust-kernel

GDB_PORT ?= 1234
QEMU_ARGS ?=

.PHONY: build run debug test

build:
	$(CARGO) build -p riscvrust-kernel --bin riscvrust-kernel

run: build
	QEMU="$(QEMU)" scripts/qemu-virt $(KERNEL) $(QEMU_ARGS)

debug: build
	QEMU="$(QEMU)" GDB_PORT="$(GDB_PORT)" scripts/qemu-virt --debug $(KERNEL) $(QEMU_ARGS)

test:
	$(CARGO) test --workspace --target $(HOST_TARGET)
