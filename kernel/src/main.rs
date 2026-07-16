#![no_main]
#![no_std]

use core::{arch::global_asm, panic::PanicInfo};

global_asm!(include_str!("entry.S"));

/// First Rust code entered after the assembly bootstrap completes.
#[unsafe(no_mangle)]
pub extern "C" fn rust_main(_hart_id: usize, _device_tree: usize) -> ! {
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
