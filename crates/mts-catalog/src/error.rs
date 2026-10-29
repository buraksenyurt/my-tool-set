use std::path::PathBuf;

/// Failures of the I/O side. Messages are in Turkish because they end up in
/// the UI as they are.
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{0}: TOML okunamadı: {1}")]
    Toml(String, String),

    #[error("{url}: {message}")]
    Http { url: String, message: String },

    #[error("{0}: release.json okunamadı: {1}")]
    BadReleaseJson(String, String),

    #[error("release.json geçersiz: {0}")]
    InvalidRelease(String),

    #[error("{tool} için bu platformda ({platform}) bir derleme yok")]
    NoBuildForPlatform { tool: String, platform: String },

    #[error("{asset}: boyut {actual} bayt, beklenen {expected}")]
    SizeMismatch {
        asset: String,
        expected: u64,
        actual: u64,
    },

    #[error("{asset}: SHA256 eşleşmiyor (beklenen {expected}, inen {actual})")]
    ShaMismatch {
        asset: String,
        expected: String,
        actual: String,
    },

    #[error("{asset}: arşiv açılamadı: {message}")]
    Archive { asset: String, message: String },

    #[error("Arşivde çalıştırılacak dosya yok: {0}")]
    EntryMissing(String),

    #[error("{tool} {version} kurulu değil")]
    NotInstalled { tool: String, version: String },
}

impl CatalogError {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

pub(crate) trait IoContext<T> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T, CatalogError>;
}

impl<T> IoContext<T> for std::io::Result<T> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T, CatalogError> {
        self.map_err(|e| CatalogError::io(path, e))
    }
}
