//! Kernel image boundaries exported by the linker script.

/// A half-open virtual address range within the kernel image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Region {
    start: usize,
    end: usize,
}

impl Region {
    /// Creates a half-open range.
    ///
    /// # Panics
    ///
    /// Panics if `end` precedes `start`.
    pub const fn new(start: usize, end: usize) -> Self {
        assert!(start <= end, "region end precedes its start");
        Self { start, end }
    }

    /// Returns the inclusive start address.
    pub const fn start(self) -> usize {
        self.start
    }

    /// Returns the exclusive end address.
    pub const fn end(self) -> usize {
        self.end
    }

    /// Returns the size of the range in bytes.
    pub const fn size(self) -> usize {
        self.end - self.start
    }
}

/// Linker-defined memory layout of the loaded kernel image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelLayout {
    pub kernel: Region,
    pub text: Region,
    pub read_only_data: Region,
    pub data: Region,
    pub bss: Region,
    pub boot_stack: Region,
}

#[cfg(target_arch = "riscv64")]
unsafe extern "C" {
    static __kernel_start: u8;
    static __kernel_end: u8;
    static __text_start: u8;
    static __text_end: u8;
    static __rodata_start: u8;
    static __rodata_end: u8;
    static __data_start: u8;
    static __data_end: u8;
    static __bss_start: u8;
    static __bss_end: u8;
    static __boot_stack_bottom: u8;
    static __boot_stack_top: u8;
}

/// Returns the kernel layout described by the linker-provided symbols.
#[cfg(target_arch = "riscv64")]
pub fn kernel_layout() -> KernelLayout {
    // Raw-address expressions do not read or create references to the extern
    // statics; the linker script guarantees that each symbol is defined.
    KernelLayout {
        kernel: Region::new(
            (&raw const __kernel_start).addr(),
            (&raw const __kernel_end).addr(),
        ),
        text: Region::new(
            (&raw const __text_start).addr(),
            (&raw const __text_end).addr(),
        ),
        read_only_data: Region::new(
            (&raw const __rodata_start).addr(),
            (&raw const __rodata_end).addr(),
        ),
        data: Region::new(
            (&raw const __data_start).addr(),
            (&raw const __data_end).addr(),
        ),
        bss: Region::new(
            (&raw const __bss_start).addr(),
            (&raw const __bss_end).addr(),
        ),
        boot_stack: Region::new(
            (&raw const __boot_stack_bottom).addr(),
            (&raw const __boot_stack_top).addr(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::Region;

    #[test]
    fn region_uses_half_open_bounds() {
        let region = Region::new(0x1000, 0x1800);

        assert_eq!(region.start(), 0x1000);
        assert_eq!(region.end(), 0x1800);
        assert_eq!(region.size(), 0x800);
    }

    #[test]
    #[should_panic(expected = "region end precedes its start")]
    fn region_rejects_reversed_bounds() {
        let _ = Region::new(0x1800, 0x1000);
    }
}
