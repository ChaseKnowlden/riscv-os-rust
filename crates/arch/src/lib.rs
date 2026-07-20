#![no_std]

//! RISC-V architecture support for the kernel.

#[cfg(target_arch = "riscv64")]
use core::arch::asm;

/// Parks the current hart indefinitely.
///
/// `wfi` is permitted to return for implementation-defined reasons, so it is
/// issued in a loop rather than treated as a permanently sleeping instruction.
pub fn halt() -> ! {
    loop {
        #[cfg(target_arch = "riscv64")]
        // SAFETY: `wfi` does not access memory or the stack. Executing it in
        // supervisor mode is part of the selected OpenSBI platform contract.
        unsafe {
            asm!("wfi", options(nomem, nostack));
        }

        #[cfg(not(target_arch = "riscv64"))]
        core::hint::spin_loop();
    }
}
