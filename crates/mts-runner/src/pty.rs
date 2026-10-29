//! Interactive terminal sessions for CLI tools (GitDash, Docky).
//!
//! A session is a program running in a pseudo terminal (ConPTY on Windows,
//! a Unix pty elsewhere). Its output is handed over as raw bytes, VT escape
//! sequences and all, for a terminal emulator (xterm.js) to draw; keystrokes
//! go back the same way.
//!
//! On Windows the session runs under `cmd` with the UTF-8 code page (65001):
//! the default console code page (857 in Turkey) would garble the UTF-8
//! characters CLI tools print.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

pub type SessionId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtySpec {
    pub tool: String,
    /// The executable, or a command line for the platform shell.
    pub program: PtyProgram,
    pub cwd: PathBuf,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtyProgram {
    Exe {
        path: PathBuf,
        args: Vec<String>,
    },
    /// A command line; `args` are quoted and appended to it.
    Shell {
        command: String,
        args: Vec<String>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum PtyError {
    #[error("Terminal açılamadı: {0}")]
    Open(String),
    #[error("Program başlatılamadı: {0}")]
    Spawn(String),
    #[error("Terminal oturumu bulunamadı: {0}")]
    Unknown(SessionId),
    #[error("Terminal oturumu: {0}")]
    Io(String),
}

struct Session {
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
}

#[derive(Clone, Default)]
pub struct PtyManager {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Default)]
struct Inner {
    next_id: SessionId,
    sessions: HashMap<SessionId, Session>,
}

/// What the program to run looks like on this platform, for display.
pub fn command_line(program: &PtyProgram) -> String {
    let (head, args) = match program {
        PtyProgram::Exe { path, args } => (quote(&path.display().to_string()), args),
        PtyProgram::Shell { command, args } => (command.clone(), args),
    };
    let mut line = head;
    for a in args {
        line.push(' ');
        line.push_str(&quote(a));
    }
    line
}

impl PtyManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts `spec`. `on_data` gets output bytes from a reader thread;
    /// `on_exit` gets the session id and exit code once the program ends (the
    /// session is removed by then).
    pub fn open(
        &self,
        spec: &PtySpec,
        on_data: impl Fn(&[u8]) + Send + 'static,
        on_exit: impl FnOnce(SessionId, Option<u32>) + Send + 'static,
    ) -> Result<SessionId, PtyError> {
        let pair = native_pty_system()
            .openpty(size(spec.cols, spec.rows))
            .map_err(|e| PtyError::Open(e.to_string()))?;

        let mut cmd = build(&spec.program);
        cmd.cwd(&spec.cwd);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::Spawn(e.to_string()))?;
        // The child owns the slave end now; dropping ours lets EOF reach the
        // reader when the program exits.
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyError::Open(e.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyError::Open(e.to_string()))?;

        let id = {
            let mut inner = self.inner.lock().unwrap();
            inner.next_id += 1;
            let id = inner.next_id;
            inner.sessions.insert(
                id,
                Session {
                    writer,
                    master: pair.master,
                    child,
                },
            );
            id
        };

        let manager = self.clone();
        std::thread::Builder::new()
            .name(format!("pty-{id}"))
            .spawn(move || {
                let mut buf = [0u8; 16 * 1024];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => on_data(&buf[..n]),
                    }
                }
                on_exit(id, manager.reap(id));
            })
            .map_err(|e| PtyError::Io(e.to_string()))?;

        Ok(id)
    }

    pub fn write(&self, id: SessionId, data: &[u8]) -> Result<(), PtyError> {
        let mut inner = self.inner.lock().unwrap();
        let session = inner.sessions.get_mut(&id).ok_or(PtyError::Unknown(id))?;
        session
            .writer
            .write_all(data)
            .and_then(|_| session.writer.flush())
            .map_err(|e| PtyError::Io(e.to_string()))
    }

    pub fn resize(&self, id: SessionId, cols: u16, rows: u16) -> Result<(), PtyError> {
        let inner = self.inner.lock().unwrap();
        let session = inner.sessions.get(&id).ok_or(PtyError::Unknown(id))?;
        session
            .master
            .resize(size(cols, rows))
            .map_err(|e| PtyError::Io(e.to_string()))
    }

    /// Ends the program (the reader thread then reports the exit).
    pub fn close(&self, id: SessionId) -> Result<(), PtyError> {
        let mut inner = self.inner.lock().unwrap();
        let session = inner.sessions.get_mut(&id).ok_or(PtyError::Unknown(id))?;
        session
            .child
            .kill()
            .map_err(|e| PtyError::Io(e.to_string()))
    }

    pub fn close_all(&self) {
        let mut inner = self.inner.lock().unwrap();
        for session in inner.sessions.values_mut() {
            let _ = session.child.kill();
        }
    }

    pub fn is_open(&self, id: SessionId) -> bool {
        self.inner.lock().unwrap().sessions.contains_key(&id)
    }

    /// Removes the session and waits for its exit code.
    fn reap(&self, id: SessionId) -> Option<u32> {
        let session = self.inner.lock().unwrap().sessions.remove(&id)?;
        let mut child = session.child;
        child.wait().ok().map(|s| s.exit_code())
    }
}

fn size(cols: u16, rows: u16) -> PtySize {
    PtySize {
        cols: cols.max(2),
        rows: rows.max(1),
        pixel_width: 0,
        pixel_height: 0,
    }
}

#[cfg(windows)]
fn build(program: &PtyProgram) -> CommandBuilder {
    // UTF-8 code page first, then the program. The command line travels in
    // an environment variable: portable-pty escapes quotes inside arguments
    // as \" (C runtime style), which cmd does not understand, so a path with
    // spaces would arrive broken. cmd expands %MTS_COMMAND% and then parses
    // it with its own quoting rules, which command_line() follows.
    let mut cmd = CommandBuilder::new("cmd.exe");
    cmd.args(["/D", "/S", "/C", "chcp 65001 >NUL & %MTS_COMMAND%"]);
    cmd.env("MTS_COMMAND", command_line(program));
    cmd
}

#[cfg(not(windows))]
fn build(program: &PtyProgram) -> CommandBuilder {
    match program {
        PtyProgram::Exe { path, args } => {
            let mut cmd = CommandBuilder::new(path);
            cmd.args(args);
            cmd
        }
        PtyProgram::Shell { .. } => {
            let mut cmd = CommandBuilder::new("sh");
            cmd.arg("-c");
            cmd.arg(command_line(program));
            cmd
        }
    }
}

/// Quotes one argument for the platform shell when it needs it.
#[cfg(windows)]
pub fn quote(arg: &str) -> String {
    if !arg.is_empty() && !arg.contains([' ', '\t', '"', '&', '|', '<', '>', '^', '(', ')']) {
        return arg.to_string();
    }
    format!("\"{}\"", arg.replace('"', "\"\""))
}

#[cfg(not(windows))]
pub fn quote(arg: &str) -> String {
    if !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_alphanumeric() || "-_./:=@%+,".contains(c))
    {
        return arg.to_string();
    }
    format!("'{}'", arg.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    type Output = Arc<Mutex<Vec<u8>>>;

    fn collect() -> (impl Fn(&[u8]) + Send + 'static, Output) {
        let out: Output = Arc::new(Mutex::new(Vec::new()));
        let sink = out.clone();
        (
            move |b: &[u8]| sink.lock().unwrap().extend_from_slice(b),
            out,
        )
    }

    fn wait_for(out: &Output, needle: &str) -> bool {
        for _ in 0..100 {
            if String::from_utf8_lossy(&out.lock().unwrap()).contains(needle) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        false
    }

    fn spec(program: PtyProgram, cols: u16, rows: u16) -> PtySpec {
        PtySpec {
            tool: "t".into(),
            program,
            cwd: std::env::temp_dir(),
            cols,
            rows,
        }
    }

    fn shell(command: &str, args: &[&str]) -> PtyProgram {
        PtyProgram::Shell {
            command: command.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[cfg(unix)]
    #[test]
    fn runs_a_program_in_a_terminal_and_reports_its_exit() {
        let manager = PtyManager::new();
        let (on_data, out) = collect();
        let (tx, rx) = mpsc::channel();
        let id = manager
            .open(
                &spec(
                    shell(
                        "test -t 1 && echo tty; printf 'ğüşİ \\033[31mkırmızı\\033[0m\\n'; exit 7",
                        &[],
                    ),
                    80,
                    24,
                ),
                on_data,
                move |_, code| tx.send(code).unwrap(),
            )
            .unwrap();
        assert_eq!(rx.recv_timeout(Duration::from_secs(10)).unwrap(), Some(7));
        let text = String::from_utf8_lossy(&out.lock().unwrap()).to_string();
        assert!(text.contains("tty"), "stdout is not a terminal: {text:?}");
        // Escape sequences are passed through untouched for xterm.js.
        assert!(text.contains("ğüşİ \u{1b}[31mkırmızı"), "{text:?}");
        assert!(!manager.is_open(id));
    }

    #[cfg(unix)]
    #[test]
    fn input_resize_and_close() {
        let manager = PtyManager::new();
        let (on_data, out) = collect();
        let (tx, rx) = mpsc::channel();
        let id = manager
            .open(
                &spec(
                    shell("stty size; read line; echo \"geldi:$line\"; sleep 30", &[]),
                    100,
                    30,
                ),
                on_data,
                move |_, code| tx.send(code).unwrap(),
            )
            .unwrap();
        assert!(wait_for(&out, "30 100"), "initial size");
        manager.write(id, b"merhaba\r").unwrap();
        assert!(wait_for(&out, "geldi:merhaba"));
        manager.resize(id, 120, 40).unwrap();
        manager.close(id).unwrap();
        rx.recv_timeout(Duration::from_secs(10))
            .expect("exit reported");
        assert!(matches!(manager.write(id, b"x"), Err(PtyError::Unknown(_))));
    }

    #[cfg(unix)]
    #[test]
    fn exe_arguments_are_passed_as_is() {
        let manager = PtyManager::new();
        let (on_data, out) = collect();
        let (tx, rx) = mpsc::channel();
        manager
            .open(
                &spec(
                    PtyProgram::Exe {
                        path: "/bin/echo".into(),
                        args: vec!["bir iki".into(), "'üç'".into()],
                    },
                    80,
                    24,
                ),
                on_data,
                move |_, c| tx.send(c).unwrap(),
            )
            .unwrap();
        rx.recv_timeout(Duration::from_secs(10)).unwrap();
        assert!(wait_for(&out, "bir iki 'üç'"));
    }

    #[cfg(unix)]
    #[test]
    fn shell_programs_get_quoted_arguments() {
        let manager = PtyManager::new();
        let (on_data, out) = collect();
        let (tx, rx) = mpsc::channel();
        manager
            .open(
                &spec(shell("printf '[%s]'", &["a b", "it's", "$HOME"]), 80, 24),
                on_data,
                move |_, c| tx.send(c).unwrap(),
            )
            .unwrap();
        rx.recv_timeout(Duration::from_secs(10)).unwrap();
        assert!(wait_for(&out, "[a b][it's][$HOME]"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_terminal_speaks_utf8() {
        let manager = PtyManager::new();
        let (on_data, out) = collect();
        let (tx, rx) = mpsc::channel();
        manager
            .open(
                &spec(shell("echo ğüşİ & exit /b 7", &[]), 80, 24),
                on_data,
                move |_, c| tx.send(c).unwrap(),
            )
            .unwrap();
        assert_eq!(rx.recv_timeout(Duration::from_secs(20)).unwrap(), Some(7));
        assert!(
            wait_for(&out, "ğüşİ"),
            "{:?}",
            String::from_utf8_lossy(&out.lock().unwrap())
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_paths_and_arguments_with_spaces() {
        let dir = std::env::temp_dir().join(format!("mts pty {}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let script = dir.join("yankı aracı.cmd");
        std::fs::write(&script, "@echo [%~1][%~2]\r\n").unwrap();
        let manager = PtyManager::new();
        let (on_data, out) = collect();
        let (tx, rx) = mpsc::channel();
        manager
            .open(
                &spec(
                    PtyProgram::Exe {
                        path: script,
                        args: vec!["C:\\My Repos\\proje".into(), "60".into()],
                    },
                    80,
                    24,
                ),
                on_data,
                move |_, c| tx.send(c).unwrap(),
            )
            .unwrap();
        rx.recv_timeout(Duration::from_secs(20)).unwrap();
        assert!(
            wait_for(&out, "[C:\\My Repos\\proje][60]"),
            "{:?}",
            String::from_utf8_lossy(&out.lock().unwrap())
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn quoting() {
        #[cfg(unix)]
        {
            assert_eq!(quote("plain-arg_1.txt"), "plain-arg_1.txt");
            assert_eq!(quote("a b"), "'a b'");
            assert_eq!(quote("it's"), "'it'\\''s'");
            assert_eq!(quote(""), "''");
        }
        #[cfg(windows)]
        {
            assert_eq!(quote("C:\\x\\y.exe"), "C:\\x\\y.exe");
            assert_eq!(
                quote("C:\\Program Files\\x.exe"),
                "\"C:\\Program Files\\x.exe\""
            );
            assert_eq!(quote("a\"b"), "\"a\"\"b\"");
        }
    }
}
