//! Defines error types for the library.

/// Convenient wrapper around `Result` for `rdbg::Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// Represents errors that can occur during the debugging.
pub type Error = Box<dyn std::error::Error>;
