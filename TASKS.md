# Rust RISC-V OS Tasks

Roadmap for a small 64-bit RISC-V operating system written primarily in Rust.

## Initial target

- Architecture: RISC-V RV64GC
- Platform: QEMU `virt`
- Firmware interface: SBI/OpenSBI
- Kernel: freestanding Rust (`#![no_std]`, `#![no_main]`)
- First execution mode: supervisor mode (S-mode)
- Initial console: SBI legacy console or UART 16550
- Initial filesystem: in-memory or embedded initramfs

Change these assumptions here before implementation if the project targets real hardware or M-mode instead.

## Milestone 0 — Project foundation

- [x] Create a Cargo workspace for the kernel and supporting crates
- [x] Select and pin a Rust toolchain
- [x] Add the RISC-V compilation target and target configuration
- [x] Configure panic behavior, linker arguments, and kernel build profiles
- [x] Add a linker script defining the kernel memory layout
- [x] Add assembly entry code that initializes the stack and calls Rust
- [ ] Add `make`, `just`, or Cargo aliases for build, run, debug, and test
- [ ] Add a QEMU launch configuration for the `virt` machine
- [ ] Document prerequisites and first-boot instructions in `README.md`
- [ ] Add formatting, linting, and build checks to CI

**Exit criterion:** QEMU loads the kernel and reaches a Rust entry function reproducibly.

## Milestone 1 — Boot and diagnostics

- [ ] Clear `.bss` during early boot
- [ ] Preserve the hart ID and device-tree pointer passed by firmware
- [ ] Implement an SBI call wrapper
- [ ] Implement early console output
- [ ] Add `print!` and `println!` macros
- [ ] Add a panic handler that prints the panic location and halts
- [ ] Print kernel version, hart ID, privilege mode, and memory layout at boot
- [ ] Parse enough of the device tree to discover RAM, CPUs, UART, and interrupt devices
- [ ] Implement a clean shutdown path for QEMU

**Exit criterion:** The kernel prints a boot banner and useful panic diagnostics, then can shut down QEMU.

## Milestone 2 — Traps, exceptions, and interrupts

- [ ] Define the supervisor trap-frame layout
- [ ] Add trap entry and return assembly
- [ ] Configure `stvec` and supervisor status registers
- [ ] Decode and report synchronous exceptions
- [ ] Handle supervisor timer interrupts
- [ ] Configure the platform interrupt controller (PLIC or AIA, as applicable)
- [ ] Handle external UART interrupts
- [ ] Add fault tests for illegal instructions, breakpoints, and page faults
- [ ] Ensure nested or unexpected traps fail with actionable diagnostics

**Exit criterion:** Timer and UART interrupts work, and exceptions produce reliable trap reports.

## Milestone 3 — Physical memory management

- [ ] Read usable RAM ranges from the device tree
- [ ] Reserve firmware, kernel, device, and boot-data regions
- [ ] Implement page-aligned address and range types
- [ ] Implement a physical frame allocator
- [ ] Add allocator invariants and allocation statistics
- [ ] Test exhaustion, reuse, alignment, and reserved-range handling
- [ ] Add an early boot allocator if initialization requires one

**Exit criterion:** The kernel can safely allocate and free physical page frames without touching reserved memory.

## Milestone 4 — Virtual memory

- [ ] Select Sv39 as the initial paging mode
- [ ] Define page-table entry flags and address translation helpers
- [ ] Implement page-table creation, mapping, unmapping, and lookup
- [ ] Map kernel text, read-only data, writable data, stacks, and MMIO with appropriate permissions
- [ ] Enable paging and switch to the kernel virtual address space
- [ ] Implement TLB invalidation with `sfence.vma`
- [ ] Add guard pages around kernel stacks
- [ ] Handle page faults and print the faulting virtual address
- [ ] Add mapping and permission tests

**Exit criterion:** The kernel runs with paging enabled and enforces section permissions.

## Milestone 5 — Kernel memory allocation

- [ ] Implement a global heap allocator
- [ ] Initialize the heap from mapped physical frames
- [ ] Enable the Rust `alloc` crate
- [ ] Validate `Box`, `Vec`, `String`, and collection usage
- [ ] Define behavior for allocation failure
- [ ] Add heap accounting and stress tests

**Exit criterion:** Dynamic allocation is stable under repeated allocation and deallocation workloads.

## Milestone 6 — Time and task scheduling

- [ ] Implement monotonic time using the RISC-V timer interface
- [ ] Program periodic or tickless timer interrupts
- [ ] Define kernel task and context structures
- [ ] Implement context switching
- [ ] Implement a simple round-robin scheduler
- [ ] Add task states, run queues, sleep, and wake-up
- [ ] Add synchronization primitives suitable for interrupt context
- [ ] Create an idle task per active hart
- [ ] Add scheduler tracing and fairness tests

**Exit criterion:** Multiple kernel tasks run, yield, sleep, and wake independently.

## Milestone 7 — User mode and system calls

- [ ] Define the user virtual address-space layout
- [ ] Create and switch process page tables
- [ ] Enter RISC-V user mode (U-mode) safely
- [ ] Define a system-call ABI and syscall numbers
- [ ] Implement syscall dispatch and argument validation
- [ ] Implement initial syscalls: `write`, `exit`, `yield`, `sleep`, and memory growth
- [ ] Safely copy data between user and kernel address spaces
- [ ] Isolate process faults from the kernel and other processes
- [ ] Track process IDs, exit status, and parent/child relationships

**Exit criterion:** An isolated user program prints to the console and exits through system calls.

## Milestone 8 — Program loading and processes

- [ ] Parse 64-bit little-endian RISC-V ELF executables
- [ ] Validate ELF headers, segments, sizes, and permissions
- [ ] Map loadable segments into a fresh process address space
- [ ] Create user stacks with arguments and environment data
- [ ] Implement process creation and program replacement
- [ ] Implement waiting for child processes
- [ ] Reclaim all process resources on exit
- [ ] Add malformed-ELF and resource-leak tests

**Exit criterion:** The kernel loads and runs more than one user-space ELF program.

## Milestone 9 — Device drivers and block I/O

- [ ] Introduce MMIO register helpers with volatile access
- [ ] Implement a UART 16550 driver
- [ ] Implement VirtIO discovery on QEMU `virt`
- [ ] Implement a VirtIO block driver
- [ ] Add a block-device abstraction
- [ ] Handle descriptor queues, interrupts, and memory barriers correctly
- [ ] Add bounded timeouts and error reporting for device operations
- [ ] Test multi-block reads, writes, and invalid requests

**Exit criterion:** The kernel reliably reads and writes a QEMU VirtIO block device.

## Milestone 10 — Filesystem and shell

- [ ] Define VFS traits for files, directories, and mounted filesystems
- [ ] Implement an initial read-only initramfs or simple in-memory filesystem
- [ ] Add path parsing and directory traversal
- [ ] Implement file descriptors and per-process descriptor tables
- [ ] Add file syscalls: `open`, `close`, `read`, `write`, and directory operations
- [ ] Mount a persistent filesystem backed by VirtIO block storage
- [ ] Implement a minimal user-space shell
- [ ] Add basic utilities such as `echo`, `cat`, `ls`, and `help`

**Exit criterion:** A user can boot into a shell and interact with files and programs.

## Milestone 11 — Multicore support

- [ ] Discover available harts from the device tree
- [ ] Start secondary harts through the SBI HSM extension
- [ ] Allocate per-hart stacks and local state
- [ ] Make the frame allocator, heap, scheduler, and drivers concurrency-safe
- [ ] Implement inter-processor interrupts
- [ ] Implement remote TLB shootdowns
- [ ] Add lock-ordering rules and deadlock diagnostics
- [ ] Stress-test scheduling and allocation across multiple harts

**Exit criterion:** Workloads run reliably on at least four emulated harts.

## Milestone 12 — Hardening and maintainability

- [ ] Minimize and document every `unsafe` block
- [ ] Add architecture and subsystem documentation
- [ ] Establish a kernel error type and consistent error propagation
- [ ] Add structured logging with configurable levels
- [ ] Add unit tests for hardware-independent crates on the host
- [ ] Add QEMU integration tests with deterministic pass/fail exit codes
- [ ] Test debug and release builds in CI
- [ ] Run Clippy and rustfmt in CI
- [ ] Add timeouts to boot tests so CI cannot hang indefinitely
- [ ] Audit integer overflow, pointer arithmetic, and user-provided lengths
- [ ] Document supported SBI, QEMU, Rust, and RISC-V versions

**Exit criterion:** Core functionality has automated regression coverage and documented safety invariants.

## Later capabilities

- [ ] Signals and asynchronous process events
- [ ] Pipes, polling, and richer process IPC
- [ ] Networking through VirtIO net
- [ ] Additional filesystems
- [ ] Copy-on-write process creation
- [ ] Demand paging and memory-mapped files
- [ ] Kernel preemption
- [ ] Load balancing across harts
- [ ] Real-time clock support
- [ ] Random-number support
- [ ] Real hardware target and board-specific drivers
- [ ] GDB scripts, stack unwinding, and crash dumps

## Definition of done for each task

A task is complete when:

- The implementation builds without warnings under the pinned toolchain.
- Relevant invariants and `unsafe` assumptions are documented.
- Hardware-independent logic has unit tests where practical.
- Hardware-dependent behavior has a reproducible QEMU test or manual test procedure.
- Failure paths return an error or produce a useful diagnostic instead of silently hanging.
- User-facing behavior is reflected in project documentation.

## Current focus

Start with **Milestone 0**. Keep only one milestone as the primary focus, and move deferred ideas to **Later capabilities** instead of expanding the active scope.
