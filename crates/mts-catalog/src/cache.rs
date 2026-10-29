//! The last `release.json` seen per tool, so the shell knows the latest
//! version on startup and when offline.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use mts_core::ReleaseManifest;
use serde::{Deserialize, Serialize};

use crate::CatalogError;
use crate::error::IoContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cached {
    /// Seconds since the Unix epoch.
    pub fetched_at: u64,
    pub release: ReleaseManifest,
}

#[derive(Debug, Clone)]
pub struct ReleaseCache {
    dir: PathBuf,
}

impl ReleaseCache {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn path(&self, tool: &str) -> PathBuf {
        self.dir.join(format!("{tool}.json"))
    }

    /// A missing or unreadable entry is simply "nothing cached".
    pub fn get(&self, tool: &str) -> Option<Cached> {
        let text = fs::read(self.path(tool)).ok()?;
        serde_json::from_slice(&text).ok()
    }

    pub fn put(&self, tool: &str, release: &ReleaseManifest) -> Result<(), CatalogError> {
        fs::create_dir_all(&self.dir).at(&self.dir)?;
        let cached = Cached {
            fetched_at: now(),
            release: release.clone(),
        };
        let path = self.path(tool);
        let tmp = path.with_extension("json.tmp");
        let json = serde_json::to_vec_pretty(&cached).expect("serializable");
        fs::write(&tmp, json).at(&tmp)?;
        fs::rename(&tmp, &path).at(&path)
    }
}

pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn round_trips_and_tolerates_garbage() {
        let dir = tempfile::tempdir().unwrap();
        let cache = ReleaseCache::new(dir.path().join("releases"));
        assert!(cache.get("t").is_none());

        let release = ReleaseManifest {
            schema: 1,
            tool: "t".into(),
            repo: "o/t".into(),
            version: "1.2.3".into(),
            tag: "v1.2.3".into(),
            commit: "c".into(),
            created_at: "x".into(),
            readme: "README.md".into(),
            platforms: BTreeMap::new(),
        };
        cache.put("t", &release).unwrap();
        let got = cache.get("t").unwrap();
        assert_eq!(got.release, release);
        assert!(got.fetched_at > 0);

        fs::write(dir.path().join("releases/t.json"), b"{not json").unwrap();
        assert!(cache.get("t").is_none());
    }
}
