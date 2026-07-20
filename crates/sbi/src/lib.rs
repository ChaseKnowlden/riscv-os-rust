#![no_std]

//! Supervisor Binary Interface support for the kernel.

#[cfg(target_arch = "riscv64")]
use core::arch::asm;

/// The standard SBI base extension ID.
pub const BASE_EXTENSION_ID: i32 = 0x10;

/// The standard SBI debug console extension ID (`"DBCN"`).
pub const DEBUG_CONSOLE_EXTENSION_ID: i32 = 0x4442_434e;

/// Standard errors returned by SBI functions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Failed,
    NotSupported,
    InvalidParameter,
    Denied,
    InvalidAddress,
    AlreadyAvailable,
    AlreadyStarted,
    AlreadyStopped,
    NoSharedMemory,
    InvalidState,
    BadRange,
    Timeout,
    InputOutput,
    DeniedLocked,
    Unknown(isize),
}

impl Error {
    const fn from_code(code: isize) -> Self {
        match code {
            -1 => Self::Failed,
            -2 => Self::NotSupported,
            -3 => Self::InvalidParameter,
            -4 => Self::Denied,
            -5 => Self::InvalidAddress,
            -6 => Self::AlreadyAvailable,
            -7 => Self::AlreadyStarted,
            -8 => Self::AlreadyStopped,
            -9 => Self::NoSharedMemory,
            -10 => Self::InvalidState,
            -11 => Self::BadRange,
            -12 => Self::Timeout,
            -13 => Self::InputOutput,
            -14 => Self::DeniedLocked,
            code => Self::Unknown(code),
        }
    }
}

/// Raw pair returned by an SBI call in registers `a0` and `a1`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use]
pub struct Return {
    error: isize,
    value: usize,
}

impl Return {
    /// Returns the signed error code from `a0` without interpreting it.
    pub const fn error_code(self) -> isize {
        self.error
    }

    /// Returns the raw value from `a1`.
    pub const fn value(self) -> usize {
        self.value
    }

    /// Converts the raw SBI return into a result.
    ///
    /// The value in `a1` is ignored on failure because the SBI specification
    /// leaves it unspecified unless an individual function says otherwise.
    pub const fn into_result(self) -> Result<usize, Error> {
        if self.error == 0 {
            Ok(self.value)
        } else {
            Err(Error::from_code(self.error))
        }
    }
}

/// Invokes an SBI v0.2-or-newer function with up to six XLEN-sized arguments.
///
/// Extension and function IDs are signed 32-bit values per the SBI encoding;
/// converting through `isize` sign-extends them to the native register width.
///
/// # Safety
///
/// The caller must satisfy every function-specific requirement for the given
/// IDs and arguments, including the validity, accessibility, lifetime, and
/// alignment of any physical memory ranges exposed to firmware. The caller
/// must also account for all function-specific platform side effects.
#[cfg(target_arch = "riscv64")]
pub unsafe fn call(extension_id: i32, function_id: i32, arguments: [usize; 6]) -> Return {
    let mut a0 = arguments[0];
    let mut a1 = arguments[1];

    // SAFETY: The caller upholds the function-specific SBI contract. The
    // register assignment follows the ratified SBI binary encoding. Omitting
    // `nomem` is intentional because firmware may access caller-owned memory.
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") a0,
            inlateout("a1") a1,
            in("a2") arguments[2],
            in("a3") arguments[3],
            in("a4") arguments[4],
            in("a5") arguments[5],
            in("a6") function_id as isize as usize,
            in("a7") extension_id as isize as usize,
            options(nostack),
        );
    }

    Return {
        error: a0 as isize,
        value: a1,
    }
}

/// Functions from the mandatory SBI base extension.
pub mod base {
    #[cfg(target_arch = "riscv64")]
    use super::{BASE_EXTENSION_ID, Error};

    #[cfg(target_arch = "riscv64")]
    const GET_SPEC_VERSION: i32 = 0;

    /// Returns the SBI specification version implemented by the firmware.
    #[cfg(target_arch = "riscv64")]
    pub fn specification_version() -> Result<usize, Error> {
        // SAFETY: This base query accepts no arguments, exposes no memory, and
        // has no platform side effects beyond returning the version number.
        unsafe { super::call(BASE_EXTENSION_ID, GET_SPEC_VERSION, [0; 6]) }.into_result()
    }
}

/// Functions from the SBI debug console extension.
pub mod debug_console {
    use super::Error;

    #[cfg(target_arch = "riscv64")]
    use super::DEBUG_CONSOLE_EXTENSION_ID;

    #[cfg(target_arch = "riscv64")]
    const CONSOLE_WRITE_BYTE: i32 = 2;

    /// Writes one byte to the firmware-provided debug console.
    ///
    /// This operation does not expose kernel memory to firmware and is safe to
    /// use before page tables or a memory allocator have been initialized.
    pub fn write_byte(byte: u8) -> Result<(), Error> {
        #[cfg(target_arch = "riscv64")]
        {
            // SAFETY: The byte-write function receives its value directly in
            // a0 and has no pointer, lifetime, or alignment requirements.
            unsafe {
                super::call(
                    DEBUG_CONSOLE_EXTENSION_ID,
                    CONSOLE_WRITE_BYTE,
                    [usize::from(byte), 0, 0, 0, 0, 0],
                )
            }
            .into_result()
            .map(|_| ())
        }

        #[cfg(not(target_arch = "riscv64"))]
        {
            let _ = byte;
            Err(Error::NotSupported)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_success() {
        let returned = Return {
            error: 0,
            value: 0x1234,
        };

        assert_eq!(returned.into_result(), Ok(0x1234));
    }

    #[test]
    fn decodes_all_standard_errors() {
        let cases = [
            (-1, Error::Failed),
            (-2, Error::NotSupported),
            (-3, Error::InvalidParameter),
            (-4, Error::Denied),
            (-5, Error::InvalidAddress),
            (-6, Error::AlreadyAvailable),
            (-7, Error::AlreadyStarted),
            (-8, Error::AlreadyStopped),
            (-9, Error::NoSharedMemory),
            (-10, Error::InvalidState),
            (-11, Error::BadRange),
            (-12, Error::Timeout),
            (-13, Error::InputOutput),
            (-14, Error::DeniedLocked),
        ];

        for (code, expected) in cases {
            let returned = Return {
                error: code,
                value: usize::MAX,
            };
            assert_eq!(returned.into_result(), Err(expected));
        }
    }

    #[test]
    fn retains_unknown_error_codes() {
        let returned = Return {
            error: -99,
            value: usize::MAX,
        };

        assert_eq!(returned.into_result(), Err(Error::Unknown(-99)));
    }
}
