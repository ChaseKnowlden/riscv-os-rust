# riscvrust

`riscvrust` is a small 64-bit RISC-V operating system kernel written in Rust.
It currently boots as an OpenSBI supervisor payload on QEMU's `virt` machine,
sets up a bootstrap stack, and enters Rust code.

## Target platform

- ISA: RV64GC
- Rust target: `riscv64gc-unknown-none-elf`
- Machine: QEMU `virt`
- Firmware: OpenSBI
- Initial privilege mode: supervisor mode (S-mode)
- Kernel load address: `0x8020_0000`

## Prerequisites

Install the following tools:

- [Rustup](https://rustup.rs/) and Cargo. The repository pins Rust 1.95.0 and
  asks Rustup to install rustfmt, Clippy, and the RISC-V target automatically.
- GNU Make.
- QEMU with the `qemu-system-riscv64` executable and default RISC-V OpenSBI
  firmware.
- Optional: a GDB build that supports RISC-V, such as `gdb-multiarch` or
  `riscv64-unknown-elf-gdb`.

Package names vary by operating system. For example, QEMU can be installed
with Homebrew on macOS:

```sh
brew install qemu
```

On Debian or Ubuntu, the relevant packages are commonly installed with:

```sh
sudo apt install make qemu-system-misc opensbi gdb-multiarch
```

Confirm that the required commands are available:

```sh
rustup --version
make --version
qemu-system-riscv64 --version
```

The first Rust or Cargo command run inside the repository may download the
pinned toolchain and target.

## First boot

Build the kernel and launch it under QEMU:

```sh
make run
```

`cargo run -p riscvrust-kernel --bin riscvrust-kernel` is equivalent. QEMU
starts the bundled OpenSBI firmware, which loads the kernel at `0x8020_0000`
and enters it in S-mode.

At the current milestone, a successful boot prints the OpenSBI banner and
includes output similar to:

```text
Domain0 Next Address        : 0x0000000080200000
Domain0 Next Mode           : S-mode
Boot HART ID                : 0

riscvrust kernel v0.1.0
  hart ID:       0
  privilege:     supervisor (S-mode)
  device tree:   0x0000000087e00000
  memory layout:
    kernel      0x0000000080200000..0x0000000080209000 (36864 bytes)
    text (RX)   0x0000000080200000..0x0000000080202d4e (11598 bytes)
    ...
  device-tree discovery (6084 bytes):
    RAM   0x0000000080000000..0x0000000088000000 (134217728 bytes)
    CPUs (1): 0
    UART  0x0000000010000000..0x0000000010000100 (256 bytes)
      interrupt: Some(10)
    PLIC  0x000000000c000000..0x000000000c600000 (6291456 bytes)
      interrupt sources: Some(95)
    CLINT 0x0000000002000000..0x0000000002010000 (65536 bytes)
      CPU interrupt controllers: 1
```

The kernel banner is emitted through the SBI debug console and confirms that
execution reached Rust successfully. Exact section boundaries vary with the
build. The kernel then parks the hart, so no further output is expected yet.

If the kernel panics, it prints a `KERNEL PANIC` diagnostic containing the
source file, line, column, and panic message before parking the hart.

Press `Ctrl-A`, then `X`, to exit QEMU's non-graphical console.

## Build and test

Build the debug kernel ELF:

```sh
make build
```

The resulting image is:

```text
target/riscv64gc-unknown-none-elf/debug/riscvrust-kernel
```

Run hardware-independent workspace tests on the host machine:

```sh
make test
```

The Makefile detects the host Rust target explicitly because Cargo defaults to
the bare-metal RISC-V target in this repository.

## Debug with GDB

Start QEMU paused before its first instruction, with a GDB server listening on
TCP port 1234:

```sh
make debug
```

In another terminal, load the kernel symbols and attach a RISC-V-capable GDB:

```sh
gdb-multiarch target/riscv64gc-unknown-none-elf/debug/riscvrust-kernel
```

Then run these commands at the GDB prompt:

```text
target remote :1234
break rust_main
continue
```

If `gdb-multiarch` is unavailable, use the equivalent RISC-V GDB executable on
your system. Override the port or QEMU executable when needed:

```sh
make debug GDB_PORT=3333
make run QEMU=/path/to/qemu-system-riscv64
```

Additional QEMU options can be passed through `QEMU_ARGS`:

```sh
make run QEMU_ARGS="-d guest_errors"
```

## Repository layout

```text
kernel/       Freestanding kernel binary and linker script
crates/arch/  RISC-V architecture support
crates/sbi/   Supervisor Binary Interface support
scripts/      QEMU launch tooling
TASKS.md      Implementation roadmap
```
