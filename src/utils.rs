//! Helper utilities.

use std::fmt;

// Macro for appending current `errno` value to a message.
macro_rules! errno {
    ($($arg:tt)*) => {
        format!("{}: {}", format!($($arg)*), std::io::Error::last_os_error()).into()
    };
}

pub(crate) use errno;

/// Logs errors to standard error in a structured format.
pub fn log_err<E>(program: &str, err: E)
where
    E: fmt::Display,
{
    eprintln!("\x1b[1m{program}\x1b[0m: \x1b[1;91merror\x1b[0m: {err}");
}
