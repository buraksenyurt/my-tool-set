//! Process groups and tree termination, per platform.

use tokio::process::Command;

/// Puts the child in its own process group so the whole tree can be
/// signalled, and so a Ctrl+C in the shell's console does not reach it.
/// Builds get no console window on Windows.
#[cfg(unix)]
pub fn own_process_group(command: &mut Command, _hide_window: bool) {
    command.process_group(0);
}

#[cfg(windows)]
pub fn own_process_group(command: &mut Command, hide_window: bool) {
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut flags = CREATE_NEW_PROCESS_GROUP;
    if hide_window {
        flags |= CREATE_NO_WINDOW;
    }
    command.creation_flags(flags);
}

/// Asks politely: SIGTERM to the group, or WM_CLOSE to the tree's windows.
#[cfg(unix)]
pub fn terminate_tree(pid: u32) {
    // The child was started with process_group(0): its pgid is its pid.
    unsafe {
        libc::killpg(pid as libc::pid_t, libc::SIGTERM);
    }
}

#[cfg(unix)]
pub fn kill_tree(pid: u32) {
    unsafe {
        libc::killpg(pid as libc::pid_t, libc::SIGKILL);
    }
}

#[cfg(windows)]
pub fn terminate_tree(pid: u32) {
    taskkill(pid, false);
}

#[cfg(windows)]
pub fn kill_tree(pid: u32) {
    taskkill(pid, true);
}

#[cfg(windows)]
fn taskkill(pid: u32, force: bool) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut c = std::process::Command::new("taskkill");
    c.args(["/PID", &pid.to_string(), "/T"]);
    if force {
        c.arg("/F");
    }
    let _ = c
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}
