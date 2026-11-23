//! The embedded terminal: CLI tools run in a pseudo terminal, their output
//! goes to xterm.js as raw bytes over a channel, keystrokes come back.

use std::collections::BTreeMap;
use std::path::PathBuf;

use mts_catalog::settings::{LocalStatus, local_status};
use mts_core::args::build_args;
use mts_core::{Delivery, HostMode, Platform};
use mts_runner::pty::{DIR_PLACEHOLDER, PtyProgram, PtySpec, command_line, expand_dir};
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::{AppHandle, Emitter, State};

use crate::dto::{TerminalDto, TerminalExitEvent};
use crate::error::{AppError, ErrorKind};
use crate::state::AppState;

pub const TERMINAL_EXIT: &str = "terminal-exit";

/// Starts a terminal tool with the run form's values. Output arrives on
/// `on_data` as raw bytes; the end is announced with `terminal-exit`.
/// `cwd` is the working folder chosen in the form; empty means the tool's own
/// folder.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn open_terminal(
    id: String,
    values: BTreeMap<String, serde_json::Value>,
    extra: String,
    cwd: Option<String>,
    cols: u16,
    rows: u16,
    on_data: Channel<InvokeResponseBody>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TerminalDto, AppError> {
    // Form fields arrive as text, but a number input may send a number:
    // accept any JSON scalar and treat it as text.
    let values: BTreeMap<String, String> = values
        .into_iter()
        .map(|(k, v)| {
            let text = match v {
                serde_json::Value::String(s) => s,
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            };
            (k, text)
        })
        .collect();
    let spec = terminal_spec(&state, &id, &values, &extra, cwd.as_deref(), cols, rows)?;
    let line = command_line(&spec.program);
    let cwd = spec.cwd.display().to_string();

    let session = state
        .terminals
        .open(
            &spec,
            move |bytes| {
                let _ = on_data.send(InvokeResponseBody::Raw(bytes.to_vec()));
            },
            move |session_id, code| {
                let _ = app.emit(TERMINAL_EXIT, TerminalExitEvent { session_id, code });
            },
        )
        .map_err(|e| AppError::new(ErrorKind::Internal, e.to_string()))?;

    Ok(TerminalDto {
        session_id: session,
        tool_id: id,
        command_line: line,
        cwd,
    })
}

#[tauri::command]
pub fn write_terminal(
    session_id: u64,
    data: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state
        .terminals
        .write(session_id, data.as_bytes())
        .map_err(|e| AppError::new(ErrorKind::NotApplicable, e.to_string()))
}

#[tauri::command]
pub fn resize_terminal(
    session_id: u64,
    cols: u16,
    rows: u16,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state
        .terminals
        .resize(session_id, cols, rows)
        .map_err(|e| AppError::new(ErrorKind::NotApplicable, e.to_string()))
}

/// Ends the program in a terminal (closing its tab).
#[tauri::command]
pub fn close_terminal(session_id: u64, state: State<'_, AppState>) -> Result<(), AppError> {
    // Already gone is fine: the tab is closing either way.
    let _ = state.terminals.close(session_id);
    Ok(())
}

fn terminal_spec(
    state: &AppState,
    id: &str,
    values: &BTreeMap<String, String>,
    extra: &str,
    workdir: Option<&str>,
    cols: u16,
    rows: u16,
) -> Result<PtySpec, AppError> {
    let catalog = state.catalog.read().unwrap();
    let entry = catalog
        .get(id)
        .ok_or_else(|| AppError::new(ErrorKind::UnknownTool, format!("Katalogda \"{id}\" yok")))?;
    let m = &entry.manifest;
    if m.run.host != HostMode::Terminal {
        return Err(AppError::new(
            ErrorKind::NotApplicable,
            format!("{} terminal aracı değil", m.name),
        ));
    }
    let args = build_args(&m.run, values, extra)
        .map_err(|e| AppError::new(ErrorKind::NotApplicable, e))?;

    // `relocatable`: the program still works when started outside its folder.
    let (program, folder, relocatable) = match m.delivery() {
        Delivery::Release => {
            let installed = state.installer.installed(id);
            let current = installed.first().ok_or_else(|| {
                AppError::new(
                    ErrorKind::NotApplicable,
                    format!("{} kurulu değil; önce kur", m.name),
                )
            })?;
            (
                PtyProgram::Exe {
                    path: current.entry.clone(),
                    args,
                },
                current.folder.clone(),
                true,
            )
        }
        Delivery::Local => {
            let settings = state.settings.read().unwrap();
            let os = Platform::current().map(|p| p.os).ok_or_else(|| {
                AppError::new(ErrorKind::NoBuildForPlatform, "Platform bilinmiyor")
            })?;
            match local_status(m, &settings, os) {
                LocalStatus::Ready { folder, exe } => {
                    (PtyProgram::Exe { path: exe, args }, folder, true)
                }
                LocalStatus::FromSource { folder } => {
                    let dev = m.run.dev.as_deref().unwrap_or_default();
                    (
                        PtyProgram::Shell {
                            command: expand_dir(dev, &folder),
                            args,
                        },
                        folder,
                        dev.contains(DIR_PLACEHOLDER),
                    )
                }
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
    };
    let cwd = match workdir.map(str::trim).filter(|d| !d.is_empty()) {
        None => folder,
        Some(dir) => {
            let dir = PathBuf::from(dir);
            if !dir.is_absolute() || !dir.is_dir() {
                return Err(AppError::new(
                    ErrorKind::NotApplicable,
                    format!("Çalışma klasörü bulunamadı: {}", dir.display()),
                ));
            }
            if !relocatable {
                return Err(AppError::new(
                    ErrorKind::NotApplicable,
                    format!(
                        "{} kendi klasörü dışında çalıştırılamıyor: run.dev komutu {DIR_PLACEHOLDER} kullanmıyor",
                        m.name
                    ),
                ));
            }
            dir
        }
    };
    Ok(PtySpec {
        tool: id.to_string(),
        program,
        cwd,
        cols,
        rows,
    })
}
