use std::process::{Command, Output, Stdio};
use std::io;

/// Run a shell command and capture output
pub fn run_command(cmd: &str) -> io::Result<Output> {
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
}

/// Run a shell command, streaming output to stdout/stderr
pub fn run_command_streaming(cmd: &str) -> io::Result<i32> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()?;

    Ok(status.code().unwrap_or(-1))
}

/// Check if a command exists on PATH
pub fn command_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
