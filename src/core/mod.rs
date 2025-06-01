//! Core library implementation of the debugger.

mod command;
mod pipe;
mod process;

pub use command::handle_command;
pub use pipe::Pipe;
pub use process::{Process, StopReason};
