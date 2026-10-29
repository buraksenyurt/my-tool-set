//! Starting tools and builds through mts-runner, and forwarding run events to
//! the webview (`run-changed`, `run-log`).

use mts_catalog::settings::{LocalStatus, local_status};
use mts_core::{Delivery, HostMode, Platform};
use mts_runner::{EventSink, LogLine, Program, RunInfo, RunKind, RunState, Spec, Stream};
use tauri::{AppHandle, Emitter, State};

use crate::dto::{LogLineDto, RunDto, RunKindDto, RunLogEvent, RunStateDto, StreamDto};
use crate::error::{AppError, ErrorKind};
use crate::state::AppState;

pub const RUN_CHANGED: &str = "run-changed";
pub const RUN_LOG: &str = "run-log";

pub struct TauriSink(pub AppHandle);

impl EventSink for TauriSink {
    fn run_changed(&self, run: &RunInfo) {
        let _ = self.0.emit(RUN_CHANGED, run_dto(run));
    }

    fn log(&self, run: u64, lines: &[LogLine]) {
        let _ = self.0.emit(
            RUN_LOG,
            RunLogEvent {
                run_id: run,
                lines: lines.iter().map(line_dto).collect(),
            },
        );
    }
}

// start/build/stop are async on purpose: Tauri runs sync commands on the main
// thread, outside the Tokio runtime that mts-runner spawns its tasks on.

/// Opens a tool in its own window: the installed release, or a local tool's
/// executable (or its `run.dev` command).
#[tauri::command]
pub async fn start_tool(id: String, state: State<'_, AppState>) -> Result<RunDto, AppError> {
    let spec = {
        let catalog = state.catalog.read().unwrap();
        let entry = catalog.get(&id).ok_or_else(|| {
            AppError::new(ErrorKind::UnknownTool, format!("Katalogda \"{id}\" yok"))
        })?;
        let m = &entry.manifest;
        match m.run.host {
            HostMode::External => {}
            HostMode::Terminal => {
                return Err(AppError::new(
                    ErrorKind::NotApplicable,
                    format!(
                        "{} gömülü terminalde çalışır; bu özellik bir sonraki fazda geliyor",
                        m.name
                    ),
                ));
            }
            HostMode::Module => {
                return Err(AppError::new(
                    ErrorKind::NotApplicable,
                    "Modül araçları MVP'de yok",
                ));
            }
        }
        match m.delivery() {
            Delivery::Release => {
                let installed = state.installer.installed(&id);
                let current = installed.first().ok_or_else(|| {
                    AppError::new(
                        ErrorKind::NotApplicable,
                        format!("{} kurulu değil; önce kur", m.name),
                    )
                })?;
                Spec {
                    tool: id.clone(),
                    kind: RunKind::Tool,
                    program: Program::Exe {
                        path: current.entry.clone(),
                        args: m.run.args.clone(),
                    },
                    cwd: current.folder.clone(),
                }
            }
            Delivery::Local => {
                let settings = state.settings.read().unwrap();
                let os = Platform::current().map(|p| p.os).ok_or_else(|| {
                    AppError::new(ErrorKind::NoBuildForPlatform, "Platform bilinmiyor")
                })?;
                match local_status(m, &settings, os) {
                    LocalStatus::Ready { folder, exe } => Spec {
                        tool: id.clone(),
                        kind: RunKind::Tool,
                        program: Program::Exe {
                            path: exe,
                            args: m.run.args.clone(),
                        },
                        cwd: folder,
                    },
                    LocalStatus::FromSource { folder } => Spec {
                        tool: id.clone(),
                        kind: RunKind::Tool,
                        program: Program::Shell {
                            command: m.run.dev.clone().unwrap_or_default(),
                        },
                        cwd: folder,
                    },
                    LocalStatus::NotBuilt { .. } => {
                        return Err(AppError::new(
                            ErrorKind::NotApplicable,
                            format!("{} henüz derlenmemiş; önce derle", m.name),
                        ));
                    }
                    _ => {
                        return Err(AppError::new(
                            ErrorKind::NotApplicable,
                            format!("{} için yerel klasör ayarlanmamış", m.name),
                        ));
                    }
                }
            }
        }
    };
    start(&state, spec)
}

/// Runs a local tool's `[build] command` in its folder.
#[tauri::command]
pub async fn build_tool(id: String, state: State<'_, AppState>) -> Result<RunDto, AppError> {
    let spec = {
        let catalog = state.catalog.read().unwrap();
        let entry = catalog.get(&id).ok_or_else(|| {
            AppError::new(ErrorKind::UnknownTool, format!("Katalogda \"{id}\" yok"))
        })?;
        let m = &entry.manifest;
        let command = m.build.as_ref().map(|b| b.command.clone()).ok_or_else(|| {
            AppError::new(
                ErrorKind::NotApplicable,
                format!("{} için derleme komutu yok", m.name),
            )
        })?;
        let settings = state.settings.read().unwrap();
        let os = Platform::current()
            .map(|p| p.os)
            .ok_or_else(|| AppError::new(ErrorKind::NoBuildForPlatform, "Platform bilinmiyor"))?;
        let folder = match local_status(m, &settings, os) {
            LocalStatus::Ready { folder, .. }
            | LocalStatus::NotBuilt { folder }
            | LocalStatus::FromSource { folder } => folder,
            _ => {
                return Err(AppError::new(
                    ErrorKind::NotApplicable,
                    format!("{} için yerel klasör ayarlanmamış", m.name),
                ));
            }
        };
        Spec {
            tool: id,
            kind: RunKind::Build,
            program: Program::Shell { command },
            cwd: folder,
        }
    };
    start(&state, spec)
}

#[tauri::command]
pub async fn stop_run(run_id: u64, state: State<'_, AppState>) -> Result<(), AppError> {
    state
        .runner()
        .stop(run_id)
        .map_err(|e| AppError::new(ErrorKind::NotApplicable, e.to_string()))
}

/// Current and recent runs, newest first.
#[tauri::command]
pub fn list_runs(state: State<'_, AppState>) -> Vec<RunDto> {
    state.runner().runs().iter().map(run_dto).collect()
}

/// Everything a run has printed so far (the panel then follows `run-log`).
#[tauri::command]
pub fn run_log(run_id: u64, state: State<'_, AppState>) -> Result<Vec<LogLineDto>, AppError> {
    state
        .runner()
        .log(run_id)
        .map(|lines| lines.iter().map(line_dto).collect())
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::NotApplicable,
                format!("Çalışma bulunamadı: {run_id}"),
            )
        })
}

fn start(state: &AppState, spec: Spec) -> Result<RunDto, AppError> {
    state
        .runner()
        .start(spec)
        .map(|info| run_dto(&info))
        .map_err(|e| AppError::new(ErrorKind::Busy, e.to_string()))
}

pub fn run_dto(r: &RunInfo) -> RunDto {
    RunDto {
        id: r.id,
        tool_id: r.tool.clone(),
        kind: match r.kind {
            RunKind::Tool => RunKindDto::Tool,
            RunKind::Build => RunKindDto::Build,
        },
        command: r.command.clone(),
        cwd: r.cwd.display().to_string(),
        pid: r.pid,
        started_at: r.started_at,
        ended_at: r.ended_at,
        state: match &r.state {
            RunState::Running => RunStateDto::Running,
            RunState::Exited { code, stopped } => RunStateDto::Exited {
                code: *code,
                stopped: *stopped,
            },
            RunState::Failed { message } => RunStateDto::Failed {
                message: message.clone(),
            },
        },
    }
}

fn line_dto(l: &LogLine) -> LogLineDto {
    LogLineDto {
        stream: match l.stream {
            Stream::Stdout => StreamDto::Stdout,
            Stream::Stderr => StreamDto::Stderr,
        },
        text: l.text.clone(),
    }
}
