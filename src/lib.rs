//! `rdbg` is a native debugger, written in Rust, targeting x86-64 Linux systems.

#![deny(missing_docs, missing_debug_implementations, unreachable_pub)]
#![warn(rust_2018_idioms)]

mod config;
mod error;

pub mod utils;

pub use config::Config;
pub use error::{Error, Result};
