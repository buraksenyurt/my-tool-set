//! Starts, watches and stops the processes the shell launches.
//!
//! Two kinds of runs: a **tool** (a GUI app the user opened) and a **build**
//! (a local tool's build command). Each run keeps the last lines of its
//! stdout/stderr and reports changes through an [`EventSink`], so the UI can
//! follow along without polling.
//!
//! [`pty`] runs CLI tools in a pseudo terminal for the embedded terminal.
//!
//! Stopping is gentle first and forceful after a grace period, and always
//! covers the whole process tree: stopping `npm run build` must also stop the
//! `cargo` it started.

mod ansi;
mod kill;
mod log;
pub mod pty;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

pub use log::{LogBuffer, LogLine, Stream};

pub type RunId = u64;

/// How long a stopped process gets to exit before it is killed.
pub const GRACE: Duration = Duration::from_secs(4);
/// Lines kept per run.
pub const LOG_LINES: usize = 5000;
/// Finished runs kept for the UI (running ones are always kept).
pub const KEEP_FINISHED: usize = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunKind {
    Tool,
    Build,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Program {
    /// An executable with arguments, started directly.
    Exe { path: PathBuf, args: Vec<String> },
    /// A command line for the platform shell (`cmd /C` or `sh -c`).
    Shell { command: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub tool: String,
    pub kind: RunKind,
    pub program: Program,
    pub cwd: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunState {
    Running,
    /// `code` is `None` when the process was ended by a signal.
    Exited {
        code: Option<i32>,
        stopped: bool,
    },
    /// Could not be started at all.
    Failed {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunInfo {
    pub id: RunId,
    pub tool: String,
    pub kind: RunKind,
    /// What was run, for display.
    pub command: String,
    pub cwd: PathBuf,
    pub pid: Option<u32>,
    /// Milliseconds since the Unix epoch.
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub state: RunState,
}

/// Where run events go (the Tauri layer forwards them to the webview).
pub trait EventSink: Send + Sync + 'static {
    fn run_changed(&self, run: &RunInfo);
    fn log(&self, run: RunId, lines: &[LogLine]);
}

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("{0} zaten çalışıyor")]
    AlreadyRunning(String),
    #[error("Çalışma bulunamadı: {0}")]
    UnknownRun(RunId),
    #[error("Çalışma zaten bitmiş")]
    NotRunning,
}

struct Run {
    info: RunInfo,
    log: LogBuffer,
    stop_requested: bool,
}

#[derive(Default)]
struct Inner {
    next_id: RunId,
    runs: VecDeque<Run>,
}

#[derive(Clone)]
pub struct Runner {
    inner: Arc<Mutex<Inner>>,
    sink: Arc<dyn EventSink>,
}

impl Runner {
    pub fn new(sink: impl EventSink) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::default())),
            sink: Arc::new(sink),
        }
    }

    /// Starts `spec` on the current Tokio runtime. A tool (or build) that is
    /// already running is not started twice. A program that cannot be started
    /// still becomes a run, in the `Failed` state, so the UI can show why.
    pub fn start(&self, spec: Spec) -> Result<RunInfo, RunError> {
        let mut inner = self.inner.lock().unwrap();
        if inner.runs.iter().any(|r| {
            r.info.tool == spec.tool
                && r.info.kind == spec.kind
                && r.info.state == RunState::Running
        }) {
            return Err(RunError::AlreadyRunning(spec.tool));
        }
        inner.next_id += 1;
        let id = inner.next_id;

        let mut command = build_command(&spec);
        let mut info = RunInfo {
            id,
            tool: spec.tool.clone(),
            kind: spec.kind,
            command: describe(&spec.program),
            cwd: spec.cwd.clone(),
            pid: None,
            started_at: now_ms(),
            ended_at: None,
            state: RunState::Running,
        };

        let child = command.spawn();
        let mut child = match child {
            Ok(child) => child,
            Err(e) => {
                info.state = RunState::Failed {
                    message: format!("Başlatılamadı: {e}"),
                };
                info.ended_at = Some(now_ms());
                push(
                    &mut inner,
                    Run {
                        info: info.clone(),
                        log: LogBuffer::new(LOG_LINES),
                        stop_requested: false,
                    },
                );
                drop(inner);
                self.sink.run_changed(&info);
                return Ok(info);
            }
        };
        info.pid = child.id();

        let (tx, rx) = mpsc::unbounded_channel::<LogLine>();
        if let Some(out) = child.stdout.take() {
            tokio::spawn(read_lines(out, Stream::Stdout, tx.clone()));
        }
        if let Some(err) = child.stderr.take() {
            tokio::spawn(read_lines(err, Stream::Stderr, tx.clone()));
        }
        drop(tx);

        push(
            &mut inner,
            Run {
                info: info.clone(),
                log: LogBuffer::new(LOG_LINES),
                stop_requested: false,
            },
        );
        drop(inner);
        self.sink.run_changed(&info);

        let runner = self.clone();
        tokio::spawn(async move { runner.forward_logs(id, rx).await });

        let runner = self.clone();
        tokio::spawn(async move {
            let status = child.wait().await;
            runner.finish(id, status.ok().and_then(|s| s.code()));
        });

        Ok(info)
    }

    /// Asks the run's process tree to exit, then kills it after [`GRACE`].
    pub fn stop(&self, id: RunId) -> Result<(), RunError> {
        let pid = {
            let mut inner = self.inner.lock().unwrap();
            let run = inner
                .runs
                .iter_mut()
                .find(|r| r.info.id == id)
                .ok_or(RunError::UnknownRun(id))?;
            if run.info.state != RunState::Running {
                return Err(RunError::NotRunning);
            }
            run.stop_requested = true;
            run.info.pid
        };
        let Some(pid) = pid else {
            return Err(RunError::NotRunning);
        };
        kill::terminate_tree(pid);
        let runner = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(GRACE).await;
            if runner.is_running(id) {
                kill::kill_tree(pid);
            }
        });
        Ok(())
    }

    pub fn is_running(&self, id: RunId) -> bool {
        self.inner
            .lock()
            .unwrap()
            .runs
            .iter()
            .any(|r| r.info.id == id && r.info.state == RunState::Running)
    }

    /// Newest first.
    pub fn runs(&self) -> Vec<RunInfo> {
        self.inner
            .lock()
            .unwrap()
            .runs
            .iter()
            .rev()
            .map(|r| r.info.clone())
            .collect()
    }

    pub fn log(&self, id: RunId) -> Option<Vec<LogLine>> {
        self.inner
            .lock()
            .unwrap()
            .runs
            .iter()
            .find(|r| r.info.id == id)
            .map(|r| r.log.lines())
    }

    /// Stops every running build (on shell exit). Tools are left running:
    /// they are the user's apps, not the shell's.
    pub fn stop_builds(&self) {
        let pids: Vec<u32> = self
            .inner
            .lock()
            .unwrap()
            .runs
            .iter()
            .filter(|r| r.info.kind == RunKind::Build && r.info.state == RunState::Running)
            .filter_map(|r| r.info.pid)
            .collect();
        for pid in pids {
            kill::kill_tree(pid);
        }
    }

    async fn forward_logs(&self, id: RunId, mut rx: mpsc::UnboundedReceiver<LogLine>) {
        let mut batch = Vec::new();
        while let Some(first) = rx.recv().await {
            batch.push(first);
            // Collect what else is already there, then give the UI one event
            // per ~50 ms instead of one per line.
            tokio::time::sleep(Duration::from_millis(50)).await;
            while batch.len() < 1000 {
                match rx.try_recv() {
                    Ok(line) => batch.push(line),
                    Err(_) => break,
                }
            }
            {
                let mut inner = self.inner.lock().unwrap();
                if let Some(run) = inner.runs.iter_mut().find(|r| r.info.id == id) {
                    for line in &batch {
                        run.log.push(line.clone());
                    }
                }
            }
            self.sink.log(id, &batch);
            batch.clear();
        }
    }

    fn finish(&self, id: RunId, code: Option<i32>) {
        let info = {
            let mut inner = self.inner.lock().unwrap();
            let Some(run) = inner.runs.iter_mut().find(|r| r.info.id == id) else {
                return;
            };
            run.info.state = RunState::Exited {
                code,
                stopped: run.stop_requested,
            };
            run.info.ended_at = Some(now_ms());
            run.info.clone()
        };
        self.sink.run_changed(&info);
    }
}

fn push(inner: &mut Inner, run: Run) {
    inner.runs.push_back(run);
    // Drop the oldest finished runs beyond the limit.
    let finished = inner
        .runs
        .iter()
        .filter(|r| r.info.state != RunState::Running)
        .count();
    let mut excess = finished.saturating_sub(KEEP_FINISHED);
    inner.runs.retain(|r| {
        if excess > 0 && r.info.state != RunState::Running {
            excess -= 1;
            false
        } else {
            true
        }
    });
}

async fn read_lines(
    reader: impl AsyncRead + Unpin,
    stream: Stream,
    tx: mpsc::UnboundedSender<LogLine>,
) {
    let mut reader = BufReader::new(reader);
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match reader.read_until(b'\n', &mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let text = String::from_utf8_lossy(&buf);
                // Progress bars redraw with \r; keep only the last state.
                let text = text.trim_end_matches(['\n', '\r']);
                let text = text.rsplit('\r').next().unwrap_or(text);
                let line = LogLine {
                    stream,
                    text: ansi::strip(text),
                };
                if tx.send(line).is_err() {
                    break;
                }
            }
        }
    }
}

fn build_command(spec: &Spec) -> Command {
    let mut command = match &spec.program {
        Program::Exe { path, args } => {
            let mut c = Command::new(path);
            c.args(args);
            c
        }
        Program::Shell { command } => shell(command),
    };
    command
        .current_dir(&spec.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(false);
    kill::own_process_group(&mut command, spec.kind == RunKind::Build);
    command
}

#[cfg(windows)]
fn shell(line: &str) -> Command {
    let mut c = Command::new("cmd");
    // raw_arg: cmd has its own quoting rules; pass the line untouched.
    c.arg("/D").arg("/S").arg("/C");
    c.raw_arg(format!("\"{line}\""));
    c
}

#[cfg(not(windows))]
fn shell(line: &str) -> Command {
    let mut c = Command::new("sh");
    c.arg("-c").arg(line);
    c
}

fn describe(program: &Program) -> String {
    match program {
        Program::Exe { path, args } => {
            let mut s = path.display().to_string();
            for a in args {
                s.push(' ');
                if a.contains(' ') {
                    s.push('"');
                    s.push_str(a);
                    s.push('"');
                } else {
                    s.push_str(a);
                }
            }
            s
        }
        Program::Shell { command } => command.clone(),
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;

    #[derive(Default)]
    struct Recorder {
        changes: StdMutex<Vec<RunInfo>>,
        lines: StdMutex<Vec<LogLine>>,
    }

    impl EventSink for Arc<Recorder> {
        fn run_changed(&self, run: &RunInfo) {
            self.changes.lock().unwrap().push(run.clone());
        }
        fn log(&self, _: RunId, lines: &[LogLine]) {
            self.lines.lock().unwrap().extend_from_slice(lines);
        }
    }

    fn shell_spec(tool: &str, kind: RunKind, command: &str) -> Spec {
        Spec {
            tool: tool.into(),
            kind,
            program: Program::Shell {
                command: command.into(),
            },
            cwd: std::env::temp_dir(),
        }
    }

    async fn wait_until(runner: &Runner, id: RunId) -> RunInfo {
        for _ in 0..200 {
            let info = runner.runs().into_iter().find(|r| r.id == id).unwrap();
            if info.state != RunState::Running {
                // Let the log forwarder flush its last batch.
                tokio::time::sleep(Duration::from_millis(120)).await;
                return info;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("run {id} did not finish");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn captures_output_streams_and_exit_code() {
        let rec = Arc::new(Recorder::default());
        let runner = Runner::new(rec.clone());
        let info = runner
            .start(shell_spec(
                "t",
                RunKind::Build,
                "echo bir; echo iki >&2; printf '\\033[32myeşil\\033[0m\\n'; exit 3",
            ))
            .unwrap();
        assert_eq!(info.state, RunState::Running);
        assert!(info.pid.is_some());

        let done = wait_until(&runner, info.id).await;
        assert_eq!(
            done.state,
            RunState::Exited {
                code: Some(3),
                stopped: false
            }
        );
        assert!(done.ended_at.is_some());

        let log = runner.log(info.id).unwrap();
        let texts: Vec<_> = log.iter().map(|l| (l.stream, l.text.as_str())).collect();
        assert!(texts.contains(&(Stream::Stdout, "bir")), "{texts:?}");
        assert!(texts.contains(&(Stream::Stderr, "iki")), "{texts:?}");
        assert!(
            texts.contains(&(Stream::Stdout, "yeşil")),
            "ANSI not stripped: {texts:?}"
        );
        assert_eq!(rec.lines.lock().unwrap().len(), 3);

        let changes = rec.changes.lock().unwrap();
        assert_eq!(changes.first().unwrap().state, RunState::Running);
        assert!(matches!(
            changes.last().unwrap().state,
            RunState::Exited { .. }
        ));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn one_run_per_tool_and_kind() {
        let runner = Runner::new(Arc::new(Recorder::default()));
        let first = runner
            .start(shell_spec("t", RunKind::Tool, "sleep 5"))
            .unwrap();
        assert!(matches!(
            runner.start(shell_spec("t", RunKind::Tool, "sleep 5")),
            Err(RunError::AlreadyRunning(_))
        ));
        // A build of the same tool is a different thing.
        let build = runner
            .start(shell_spec("t", RunKind::Build, "true"))
            .unwrap();
        runner.stop(first.id).unwrap();
        wait_until(&runner, first.id).await;
        wait_until(&runner, build.id).await;
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stop_ends_the_whole_process_tree() {
        let runner = Runner::new(Arc::new(Recorder::default()));
        let marker = std::env::temp_dir().join(format!("mts-runner-{}", std::process::id()));
        let _ = std::fs::remove_file(&marker);
        // The grandchild would write the marker after 2 s if it survived.
        let command = format!("sh -c 'sleep 2; touch {}' & wait", marker.display());
        let info = runner
            .start(shell_spec("t", RunKind::Build, &command))
            .unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        runner.stop(info.id).unwrap();
        let done = wait_until(&runner, info.id).await;
        assert!(
            matches!(done.state, RunState::Exited { stopped: true, .. }),
            "{done:?}"
        );
        tokio::time::sleep(Duration::from_millis(2500)).await;
        assert!(!marker.exists(), "grandchild survived the stop");
        assert!(matches!(runner.stop(info.id), Err(RunError::NotRunning)));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stubborn_processes_are_killed_after_the_grace_period() {
        let runner = Runner::new(Arc::new(Recorder::default()));
        let info = runner
            .start(shell_spec(
                "t",
                RunKind::Tool,
                "trap '' TERM; while true; do sleep 0.1; done",
            ))
            .unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        runner.stop(info.id).unwrap();
        let started = std::time::Instant::now();
        let done = wait_until(&runner, info.id).await;
        assert!(started.elapsed() >= GRACE - Duration::from_millis(500));
        assert_eq!(
            done.state,
            RunState::Exited {
                code: None,
                stopped: true
            }
        );
    }

    #[tokio::test]
    async fn missing_program_is_a_failed_run_not_an_error() {
        let rec = Arc::new(Recorder::default());
        let runner = Runner::new(rec.clone());
        let info = runner
            .start(Spec {
                tool: "t".into(),
                kind: RunKind::Tool,
                program: Program::Exe {
                    path: "/definitely/not/here".into(),
                    args: vec![],
                },
                cwd: std::env::temp_dir(),
            })
            .unwrap();
        assert!(matches!(info.state, RunState::Failed { .. }));
        assert!(!runner.is_running(info.id));
        // And the tool can be started again right away.
        assert!(runner.start(shell_spec("t", RunKind::Tool, "true")).is_ok());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn keeps_a_bounded_history() {
        let runner = Runner::new(Arc::new(Recorder::default()));
        let mut last = 0;
        for _ in 0..(KEEP_FINISHED + 5) {
            last = runner
                .start(shell_spec("t", RunKind::Build, "true"))
                .unwrap()
                .id;
            wait_until(&runner, last).await;
        }
        let runs = runner.runs();
        assert!(runs.len() <= KEEP_FINISHED + 1, "{}", runs.len());
        assert_eq!(runs.first().unwrap().id, last);
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn windows_captures_output_streams_and_exit_code() {
        let runner = Runner::new(Arc::new(Recorder::default()));
        let info = runner
            .start(shell_spec(
                "t",
                RunKind::Build,
                "(echo bir)& (echo iki)1>&2 & exit /b 3",
            ))
            .unwrap();
        let done = wait_until(&runner, info.id).await;
        assert_eq!(
            done.state,
            RunState::Exited {
                code: Some(3),
                stopped: false
            }
        );
        let log = runner.log(info.id).unwrap();
        let texts: Vec<_> = log.iter().map(|l| (l.stream, l.text.trim())).collect();
        assert!(texts.contains(&(Stream::Stdout, "bir")), "{texts:?}");
        assert!(texts.contains(&(Stream::Stderr, "iki")), "{texts:?}");
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn windows_stop_ends_the_process_tree() {
        let runner = Runner::new(Arc::new(Recorder::default()));
        // cmd -> ping: stopping must take the child down too.
        let info = runner
            .start(shell_spec("t", RunKind::Build, "ping -n 60 127.0.0.1 >NUL"))
            .unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
        runner.stop(info.id).unwrap();
        let done = wait_until(&runner, info.id).await;
        assert!(
            matches!(done.state, RunState::Exited { stopped: true, .. }),
            "{done:?}"
        );
        let out = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq PING.EXE", "/NH"])
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
        assert!(!text.contains("ping.exe"), "ping survived: {text}");
    }

    #[test]
    fn describes_programs_for_display() {
        assert_eq!(
            describe(&Program::Exe {
                path: "/x/easy-git".into(),
                args: vec!["--repo".into(), "a b".into()]
            }),
            "/x/easy-git --repo \"a b\""
        );
    }
}
