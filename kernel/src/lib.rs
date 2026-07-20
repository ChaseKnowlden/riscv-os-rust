#![no_std]

//! Core kernel functionality.

pub mod boot;
pub mod console;
pub mod memory;

/// Prints formatted text through the early kernel console.
#[macro_export]
macro_rules! print {
    ($($argument:tt)*) => {{
        $crate::console::_print(core::format_args!($($argument)*));
    }};
}

/// Prints formatted text followed by a newline through the early console.
#[macro_export]
macro_rules! println {
    () => {{
        $crate::console::_print(core::format_args!("\n"));
    }};
    ($($argument:tt)*) => {{
        $crate::console::_print(core::format_args!(
            "{}\n",
            core::format_args!($($argument)*)
        ));
    }};
}
