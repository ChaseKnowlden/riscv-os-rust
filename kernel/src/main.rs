#![no_main]
#![no_std]

use core::{arch::global_asm, panic::PanicInfo};
use riscvrust_kernel::boot::{self, BootInfo};

global_asm!(include_str!("entry.S"));

/// First Rust code entered after the assembly bootstrap completes.
#[unsafe(no_mangle)]
pub extern "C" fn rust_main(hart_id: usize, device_tree_address: usize) -> ! {
    if boot::initialize(BootInfo::new(hart_id, device_tree_address)).is_err() {
        halt();
    }

    halt()
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    halt()
}

fn halt() -> ! {
    loop {
        core::hint::spin_loop();
    }
}
