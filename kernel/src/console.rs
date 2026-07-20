//! Console output available during early kernel initialization.

use core::fmt::{self, Write};

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

/// Writes preformatted arguments through the early console.
pub fn write_fmt(arguments: fmt::Arguments<'_>) -> Result<(), Error> {
    let mut writer = ConsoleWriter { error: None };

    match fmt::write(&mut writer, arguments) {
        Ok(()) => Ok(()),
        Err(_) => Err(writer.error.unwrap_or(Error::Failed)),
    }
}

#[doc(hidden)]
pub fn _print(arguments: fmt::Arguments<'_>) {
    let _ = write_fmt(arguments);
}

struct ConsoleWriter {
    error: Option<Error>,
}

impl Write for ConsoleWriter {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        crate::console::write_str(text).map_err(|error| {
            self.error = Some(error);
            fmt::Error
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn formatting_macros_accept_arguments() {
        let value = 42;

        crate::print!("value={value}");
        crate::println!(" {value:#x}");
        crate::println!();
    }
}
