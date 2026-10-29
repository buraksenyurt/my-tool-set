use serde::Serialize;
use ts_rs::TS;

/// What the front end receives when a command fails. Tauri serialises the
/// `Err` side of a command's `Result`, so this type must be `Serialize`.
#[derive(Debug, Clone, Serialize, TS, thiserror::Error)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[error("{message}")]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ErrorKind {
    /// No tool with that id in the catalog.
    UnknownTool,
    /// The tool is local, or the action does not apply to it.
    NotApplicable,
    /// An install of this tool is already running.
    Busy,
    /// GitHub could not be reached or answered with an error.
    Network,
    /// Size or SHA256 did not match release.json.
    Integrity,
    /// No build for this OS/CPU in the release.
    NoBuildForPlatform,
    /// Disk, archive or anything else.
    Internal,
}

impl AppError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl From<mts_catalog::CatalogError> for AppError {
    fn from(err: mts_catalog::CatalogError) -> Self {
        use mts_catalog::CatalogError as E;
        let kind = match &err {
            E::Http { .. } | E::BadReleaseJson(..) => ErrorKind::Network,
            E::SizeMismatch { .. } | E::ShaMismatch { .. } | E::InvalidRelease(_) => {
                ErrorKind::Integrity
            }
            E::NoBuildForPlatform { .. } => ErrorKind::NoBuildForPlatform,
            _ => ErrorKind::Internal,
        };
        Self::new(kind, err.to_string())
    }
}
