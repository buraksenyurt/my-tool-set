//! Shapes sent to the front end. Generated as TypeScript into src/lib/bindings
//! by `cargo test` (ts-rs), so both sides always agree.

use mts_core::manifest::{RunParam, Support};
use mts_core::{HostMode, Platform, ToolKind};
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppInfoDto {
    pub name: String,
    pub version: String,
    /// `None` when the shell runs on a platform the tools are never released for.
    pub platform: Option<Platform>,
    /// Installed tools and caches.
    pub data_dir: String,
    /// settings.toml and the user catalog.
    pub config_dir: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CatalogDto {
    pub tools: Vec<ToolDto>,
    pub issues: Vec<IssueDto>,
    pub settings_file: String,
    pub user_catalog_dir: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IssueDto {
    pub location: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ToolDto {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub category: String,
    pub kind: ToolKind,
    pub stack: Vec<String>,
    pub tags: Vec<String>,
    pub host: HostMode,
    pub origin: OriginDto,
    /// What the manifest says about this OS.
    pub support: Support,
    pub params: Vec<RunParam>,
    pub delivery: DeliveryDto,
    /// `mtsimg` URLs of the manifest's screenshots.
    pub screenshots: Vec<String>,
    /// The repository on GitHub, for release tools.
    pub web_url: Option<String>,
    /// Revision the README and screenshots are shown at (tag, `HEAD` or
    /// `local`); `None` when a local tool's folder is not found.
    pub doc_revision: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum OriginDto {
    Embedded,
    Repository,
    User,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export)]
pub enum DeliveryDto {
    #[serde(rename_all = "camelCase")]
    Release {
        repo: String,
        /// Newest first.
        installed: Vec<InstalledDto>,
        /// From the last successful check (or the cache).
        latest: Option<LatestDto>,
        update_available: bool,
    },
    #[serde(rename_all = "camelCase")]
    Local {
        status: LocalStatusDto,
        build_command: Option<String>,
        dev_command: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InstalledDto {
    pub version: String,
    pub folder: String,
    pub entry: String,
    /// Seconds since the Unix epoch.
    #[ts(type = "number")]
    pub installed_at: u64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LatestDto {
    pub version: String,
    pub tag: String,
    /// Seconds since the Unix epoch.
    #[ts(type = "number")]
    pub fetched_at: u64,
    /// Does the release contain a build for this platform?
    pub has_build: bool,
    /// Download size for this platform, in bytes.
    #[ts(type = "number | null")]
    pub asset_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum LocalStatusDto {
    Ready { folder: String, exe: String },
    NotBuilt { folder: String },
    FromSource { folder: String },
    UnknownRoot { root: String },
    MissingFolder { folder: String },
    NoExeForOs,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateCheckDto {
    pub id: String,
    pub version: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "step", rename_all = "camelCase")]
#[ts(export)]
pub enum InstallProgressDto {
    #[serde(rename_all = "camelCase")]
    Downloading {
        #[ts(type = "number")]
        received: u64,
        #[ts(type = "number")]
        total: u64,
    },
    Verifying,
    Unpacking,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReadmeDto {
    /// Sanitized HTML; `None` when the README could not be read.
    pub html: Option<String>,
    pub revision: Option<String>,
    /// The README on GitHub at that revision.
    pub web_url: Option<String>,
    /// Served from the cache because GitHub could not be reached.
    pub stale: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunDto {
    #[ts(type = "number")]
    pub id: u64,
    pub tool_id: String,
    pub kind: RunKindDto,
    /// What was run, for display.
    pub command: String,
    pub cwd: String,
    pub pid: Option<u32>,
    /// Milliseconds since the Unix epoch.
    #[ts(type = "number")]
    pub started_at: u64,
    #[ts(type = "number | null")]
    pub ended_at: Option<u64>,
    pub state: RunStateDto,
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RunKindDto {
    Tool,
    Build,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "status", rename_all = "camelCase")]
#[ts(export)]
pub enum RunStateDto {
    Running,
    /// `code` is `null` when the process was ended by a signal.
    Exited {
        code: Option<i32>,
        stopped: bool,
    },
    Failed {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LogLineDto {
    pub stream: StreamDto,
    pub text: String,
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum StreamDto {
    Stdout,
    Stderr,
}

/// Payload of the `run-log` event.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunLogEvent {
    #[ts(type = "number")]
    pub run_id: u64,
    pub lines: Vec<LogLineDto>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TerminalDto {
    #[ts(type = "number")]
    pub session_id: u64,
    pub tool_id: String,
    /// What runs in the terminal, for the tab header.
    pub command_line: String,
    pub cwd: String,
}

/// Payload of the `terminal-exit` event.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TerminalExitEvent {
    #[ts(type = "number")]
    pub session_id: u64,
    pub code: Option<u32>,
}
