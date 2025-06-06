use crate::Result;
use crate::core::Process;

/// Process an input command for a given [`Process`].
pub fn handle_command(proc: &mut Process, input: &str) -> Result<()> {
    let mut parts = input.split(' ');

    let command = parts.next().unwrap_or("");

    if "continue".starts_with(command) {
        proc.resume()?;
        let reason = proc.wait_on_signal()?;
        reason.log_stop_reason(proc);
    } else {
        return Err(format!("unrecognized command '{command}'").into());
    }

    Ok(())
}
