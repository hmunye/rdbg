//! Error types for the debugger.

/// Convenient wrapper around `Result` for [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during the debugging process.
pub type Error = Box<dyn std::error::Error>;
