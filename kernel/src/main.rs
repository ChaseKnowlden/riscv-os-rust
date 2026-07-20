#![no_main]
#![no_std]

use core::{arch::global_asm, panic::PanicInfo};
use riscvrust_arch::halt;
use riscvrust_kernel::{
    boot::{self, BootInfo},
    memory::{self, Region},
    platform::Platform,
};

global_asm!(include_str!("entry.S"));

/// First Rust code entered after the assembly bootstrap completes.
#[unsafe(no_mangle)]
pub extern "C" fn rust_main(hart_id: usize, device_tree_address: usize) -> ! {
    if boot::initialize(BootInfo::new(hart_id, device_tree_address)).is_err() {
        halt();
    }

    if let Err(error) = riscvrust_sbi::base::specification_version() {
        panic!("failed to query the SBI specification version: {error:?}");
    }

    let Some(boot_info) = boot::get() else {
        panic!("firmware boot information was not preserved");
    };
    let layout = memory::kernel_layout();

    riscvrust_kernel::println!();
    riscvrust_kernel::println!("riscvrust kernel v{}", env!("CARGO_PKG_VERSION"));
    riscvrust_kernel::println!("  hart ID:       {}", boot_info.hart_id());
    riscvrust_kernel::println!("  privilege:     supervisor (S-mode)");
    riscvrust_kernel::println!(
        "  device tree:   0x{:016x}",
        boot_info.device_tree_address()
    );
    riscvrust_kernel::println!("  memory layout:");
    print_region("kernel", layout.kernel);
    print_region("text (RX)", layout.text);
    print_region("rodata (R)", layout.read_only_data);
    print_region("data (RW)", layout.data);
    print_region("bss (RW)", layout.bss);
    print_region("boot stack", layout.boot_stack);

    // SAFETY: OpenSBI passes a resident, immutable FDT pointer in a1 according
    // to the selected platform boot contract.
    let platform = unsafe { Platform::from_pointer(boot_info.device_tree_address()) }
        .unwrap_or_else(|error| panic!("invalid firmware device tree: {error:?}"));
    platform
        .validate()
        .unwrap_or_else(|error| panic!("required platform device missing: {error:?}"));

    riscvrust_kernel::println!(
        "  device-tree discovery ({} bytes):",
        platform.device_tree_size()
    );
    platform.for_each_memory_region(|ram| {
        print_address_range("RAM", ram.base, ram.size);
    });

    let cpu_count = platform.cpu_count();
    riscvrust_kernel::print!("    CPUs ({cpu_count}):");
    platform.for_each_cpu_hart(|hart_id| {
        riscvrust_kernel::print!(" {hart_id}");
    });
    riscvrust_kernel::println!();

    let uart = platform.uart().expect("validated UART disappeared");
    print_address_range("UART", uart.registers.base, uart.registers.size);
    riscvrust_kernel::println!("      interrupt: {:?}", uart.interrupt);

    let plic = platform.plic().expect("validated PLIC disappeared");
    print_address_range("PLIC", plic.registers.base, plic.registers.size);
    riscvrust_kernel::println!("      interrupt sources: {:?}", plic.interrupt_sources);

    let clint = platform.clint().expect("validated CLINT disappeared");
    print_address_range("CLINT", clint.base, clint.size);
    riscvrust_kernel::println!(
        "      CPU interrupt controllers: {}",
        platform.cpu_interrupt_controller_count()
    );

    riscvrust_kernel::println!("  shutdown: requesting SBI system power-off");
    match riscvrust_sbi::system_reset::shutdown() {
        Ok(never) => match never {},
        Err(error) => panic!("SBI system shutdown failed: {error:?}"),
    }
}

fn print_region(name: &str, region: Region) {
    riscvrust_kernel::println!(
        "    {name:<11} 0x{:016x}..0x{:016x} ({} bytes)",
        region.start(),
        region.end(),
        region.size(),
    );
}

fn print_address_range(name: &str, base: usize, size: Option<usize>) {
    match size {
        Some(size) => riscvrust_kernel::println!(
            "    {name:<5} 0x{base:016x}..0x{:016x} ({size} bytes)",
            base + size
        ),
        None => riscvrust_kernel::println!("    {name:<5} 0x{base:016x} (size unspecified)"),
    }
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    match info.location() {
        Some(location) => riscvrust_kernel::println!(
            "\nKERNEL PANIC at {}:{}:{}\n  {}",
            location.file(),
            location.line(),
            location.column(),
            info.message(),
        ),
        None => {
            riscvrust_kernel::println!("\nKERNEL PANIC at <unknown location>\n  {}", info.message())
        }
    }

    halt()
}
