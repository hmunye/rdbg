//! Core library implementation of the debugger.

mod command;
mod process;

pub use command::handle_command;
pub use process::{Process, StopReason};
