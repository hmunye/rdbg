//! `rdbg` is a native debugger, written in Rust, targeting x86-64 Linux systems.

#![deny(missing_docs, missing_debug_implementations, unreachable_pub)]
#![warn(rust_2018_idioms)]

pub mod core;
pub mod utils;

mod config;
pub use config::Config;

mod error;
pub use error::{Error, Result};
