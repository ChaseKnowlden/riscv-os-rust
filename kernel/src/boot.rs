//! Boot information supplied by the platform firmware.

use core::sync::atomic::{AtomicUsize, Ordering};

const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

static STATE: AtomicUsize = AtomicUsize::new(UNINITIALIZED);
static HART_ID: AtomicUsize = AtomicUsize::new(0);
static DEVICE_TREE_ADDRESS: AtomicUsize = AtomicUsize::new(0);

/// Values placed in `a0` and `a1` by OpenSBI before entering the kernel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootInfo {
    hart_id: usize,
    device_tree_address: usize,
}

impl BootInfo {
    /// Creates boot information from the firmware-provided register values.
    pub const fn new(hart_id: usize, device_tree_address: usize) -> Self {
        Self {
            hart_id,
            device_tree_address,
        }
    }

    /// Returns the ID of the hart selected by the firmware to boot the kernel.
    pub const fn hart_id(self) -> usize {
        self.hart_id
    }

    /// Returns the physical address of the flattened device tree.
    pub const fn device_tree_address(self) -> usize {
        self.device_tree_address
    }
}

/// Indicates that boot information was already being or had been initialized.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AlreadyInitialized;

/// Preserves the firmware values for use by later kernel initialization.
///
/// The state transition publishes both values together, preventing readers on
/// other harts from observing a partially initialized [`BootInfo`].
pub fn initialize(info: BootInfo) -> Result<(), AlreadyInitialized> {
    STATE
        .compare_exchange(
            UNINITIALIZED,
            INITIALIZING,
            Ordering::Relaxed,
            Ordering::Relaxed,
        )
        .map_err(|_| AlreadyInitialized)?;

    HART_ID.store(info.hart_id(), Ordering::Relaxed);
    DEVICE_TREE_ADDRESS.store(info.device_tree_address(), Ordering::Relaxed);
    STATE.store(INITIALIZED, Ordering::Release);

    Ok(())
}

/// Returns the preserved boot information once initialization is complete.
pub fn get() -> Option<BootInfo> {
    if STATE.load(Ordering::Acquire) != INITIALIZED {
        return None;
    }

    Some(BootInfo::new(
        HART_ID.load(Ordering::Relaxed),
        DEVICE_TREE_ADDRESS.load(Ordering::Relaxed),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_firmware_values_once() {
        let expected = BootInfo::new(3, 0x87e0_0000);

        assert_eq!(initialize(expected), Ok(()));
        assert_eq!(get(), Some(expected));
        assert_eq!(initialize(expected), Err(AlreadyInitialized));
    }
}
