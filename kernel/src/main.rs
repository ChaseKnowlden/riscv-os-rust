#![no_main]
#![no_std]

use core::{arch::global_asm, panic::PanicInfo};
use riscvrust_arch::halt;
use riscvrust_kernel::boot::{self, BootInfo};

global_asm!(include_str!("entry.S"));

/// First Rust code entered after the assembly bootstrap completes.
#[unsafe(no_mangle)]
pub extern "C" fn rust_main(hart_id: usize, device_tree_address: usize) -> ! {
    if boot::initialize(BootInfo::new(hart_id, device_tree_address)).is_err() {
        halt();
    }

    if riscvrust_sbi::base::specification_version().is_err() {
        halt();
    }

    riscvrust_kernel::println!("riscvrust: early console {}", "online");

    halt()
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
