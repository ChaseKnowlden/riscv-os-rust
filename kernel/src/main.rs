#![no_main]
#![no_std]

use core::{arch::global_asm, panic::PanicInfo};
use riscvrust_arch::halt;
use riscvrust_kernel::{
    boot::{self, BootInfo},
    memory::{self, Region},
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

    halt()
}

fn print_region(name: &str, region: Region) {
    riscvrust_kernel::println!(
        "    {name:<11} 0x{:016x}..0x{:016x} ({} bytes)",
        region.start(),
        region.end(),
        region.size(),
    );
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
