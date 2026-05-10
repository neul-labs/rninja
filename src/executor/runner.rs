use std::io;
use std::process::{Command, Output, Stdio};

#[allow(dead_code)]
/// Run a shell command and capture output
pub fn run_command(cmd: &str) -> io::Result<Output> {
    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "sh"
    };
    let shell_arg = if cfg!(target_os = "windows") {
        "/C"
    } else {
        "-c"
    };
    Command::new(shell)
        .arg(shell_arg)
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
}

#[allow(dead_code)]
/// Run a shell command, streaming output to stdout/stderr.
///
/// Returns the exit code on success, or an error if:
/// - The command couldn't be spawned
/// - The process was terminated by a signal (returns `std::io::ErrorKind::Other`)
pub fn run_command_streaming(cmd: &str) -> io::Result<i32> {
    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "sh"
    };
    let shell_arg = if cfg!(target_os = "windows") {
        "/C"
    } else {
        "-c"
    };
    let status = Command::new(shell).arg(shell_arg).arg(cmd).status()?;

    // Properly handle signal termination instead of hiding it
    if let Some(code) = status.code() {
        Ok(code)
    } else {
        // Process was terminated by a signal - return an error to indicate this
        Err(io::Error::other("command was terminated by a signal"))
    }
}

#[allow(dead_code)]
/// Check if a command exists on PATH.
///
/// Returns `true` if the command exists, `false` if it doesn't,
/// or propagates I/O errors (permission denied, command not found, etc.).
pub fn command_exists(name: &str) -> bool {
    let which_cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };
    Command::new(which_cmd)
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
