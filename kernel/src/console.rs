//! Console output available during early kernel initialization.

pub use riscvrust_sbi::Error;

/// Writes one byte through the SBI debug console extension.
pub fn write_byte(byte: u8) -> Result<(), Error> {
    riscvrust_sbi::debug_console::write_byte(byte)
}

/// Writes a UTF-8 string through the early console.
///
/// Line feeds are preceded by carriage returns for terminals attached to
/// QEMU's serial console.
pub fn write_str(text: &str) -> Result<(), Error> {
    for byte in text.bytes() {
        if byte == b'\n' {
            write_byte(b'\r')?;
        }
        write_byte(byte)?;
    }

    Ok(())
}
