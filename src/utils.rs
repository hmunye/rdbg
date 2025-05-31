//! Shared helper utilities.

use std::fmt;

/// Logs errors to standard error in a structured format.
pub fn log_err<E>(program: &str, err: E)
where
    E: fmt::Display,
{
    eprintln!("\x1b[1m{program}\x1b[0m: \x1b[1;91merror\x1b[0m: {err}");
}
