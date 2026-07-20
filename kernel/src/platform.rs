//! Hardware discovery from the firmware-provided flattened device tree.

use fdt::{Fdt, FdtError, node::FdtNode};

/// A physical memory or MMIO range described by a `reg` property.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AddressRange {
    pub base: usize,
    pub size: Option<usize>,
}

/// A discovered 16550-compatible UART.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Uart {
    pub registers: AddressRange,
    pub interrupt: Option<usize>,
}

/// A discovered platform-level interrupt controller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Plic {
    pub registers: AddressRange,
    pub interrupt_sources: Option<usize>,
}

/// Errors encountered while locating required QEMU `virt` hardware.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryError {
    MissingRam,
    MissingCpu,
    MissingUart,
    MissingPlic,
    MissingClint,
}

/// Zero-copy view of platform hardware described by an FDT.
#[derive(Clone, Copy, Debug)]
pub struct Platform<'a> {
    fdt: Fdt<'a>,
}

impl<'a> Platform<'a> {
    /// Parses a flattened device tree from a byte slice.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, FdtError> {
        Fdt::new(bytes).map(|fdt| Self { fdt })
    }

    /// Parses the firmware-owned flattened device tree at `address`.
    ///
    /// # Safety
    ///
    /// `address` must be nonzero and point to a complete, readable FDT that
    /// remains resident and immutable for the returned platform's lifetime.
    #[cfg(target_arch = "riscv64")]
    pub unsafe fn from_pointer(address: usize) -> Result<Platform<'static>, FdtError> {
        assert_ne!(address, 0, "firmware supplied a null device-tree pointer");

        // SAFETY: The caller guarantees that the firmware-provided pointer and
        // the total size encoded in its FDT header describe readable memory.
        unsafe { Fdt::from_ptr(address as *const u8) }.map(|fdt| Platform { fdt })
    }

    /// Ensures all hardware required by the initial QEMU target is present.
    pub fn validate(self) -> Result<(), DiscoveryError> {
        if self.memory_region_count() == 0 {
            return Err(DiscoveryError::MissingRam);
        }
        if self.cpu_count() == 0 {
            return Err(DiscoveryError::MissingCpu);
        }
        if self.uart().is_none() {
            return Err(DiscoveryError::MissingUart);
        }
        if self.plic().is_none() {
            return Err(DiscoveryError::MissingPlic);
        }
        if self.clint().is_none() {
            return Err(DiscoveryError::MissingClint);
        }

        Ok(())
    }

    /// Visits every physical RAM range.
    pub fn for_each_memory_region(self, mut visitor: impl FnMut(AddressRange)) {
        let memory = self.fdt.memory();
        for region in memory.regions() {
            visitor(AddressRange {
                base: region.starting_address.addr(),
                size: region.size,
            });
        }
    }

    /// Returns the number of physical RAM ranges.
    pub fn memory_region_count(self) -> usize {
        let memory = self.fdt.memory();
        memory.regions().count()
    }

    /// Visits every CPU hart ID listed by the FDT.
    pub fn for_each_cpu_hart(self, mut visitor: impl FnMut(usize)) {
        for cpu in self.fdt.cpus() {
            for hart_id in cpu.ids().all() {
                visitor(hart_id);
            }
        }
    }

    /// Returns the number of CPU hart IDs listed by the FDT.
    pub fn cpu_count(self) -> usize {
        self.fdt.cpus().map(|cpu| cpu.ids().all().count()).sum()
    }

    /// Finds the first NS16550-compatible UART.
    pub fn uart(self) -> Option<Uart> {
        let node = self.fdt.find_compatible(&["ns16550a", "ns16550"])?;
        Some(Uart {
            registers: node_address_range(node)?,
            interrupt: node.interrupts().and_then(|mut values| values.next()),
        })
    }

    /// Finds the RISC-V platform-level interrupt controller.
    pub fn plic(self) -> Option<Plic> {
        let node = self
            .fdt
            .find_compatible(&["sifive,plic-1.0.0", "riscv,plic0"])?;
        Some(Plic {
            registers: node_address_range(node)?,
            interrupt_sources: node
                .property("riscv,ndev")
                .and_then(|value| value.as_usize()),
        })
    }

    /// Finds the RISC-V core-local interruptor used by QEMU `virt`.
    pub fn clint(self) -> Option<AddressRange> {
        let node = self
            .fdt
            .find_compatible(&["sifive,clint0", "riscv,clint0"])?;
        node_address_range(node)
    }

    /// Counts per-hart CPU interrupt-controller nodes.
    pub fn cpu_interrupt_controller_count(self) -> usize {
        self.fdt
            .all_nodes()
            .filter(|node| {
                node.compatible()
                    .is_some_and(|compatible| compatible.all().any(|name| name == "riscv,cpu-intc"))
            })
            .count()
    }

    /// Returns the validated total FDT size in bytes.
    pub fn device_tree_size(self) -> usize {
        self.fdt.total_size()
    }
}

fn node_address_range(node: FdtNode<'_, '_>) -> Option<AddressRange> {
    let region = node.reg()?.next()?;
    Some(AddressRange {
        base: region.starting_address.addr(),
        size: region.size,
    })
}
