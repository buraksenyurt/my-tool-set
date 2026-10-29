//! `settings.toml`: per-machine settings. Today only the roots that local
//! (non-release) tools are found under.
//!
//! ```toml
//! [roots]
//! coding-vibes = "C:/Users/burak/Development/coding-vibes"
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use mts_core::manifest::SourceSpec;
use mts_core::{Os, ToolManifest};
use serde::Deserialize;

use crate::CatalogError;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    #[serde(default)]
    pub roots: BTreeMap<String, PathBuf>,
}

impl Settings {
    /// A missing file means "no settings yet", not an error.
    pub fn load(path: &Path) -> Result<Self, CatalogError> {
        match fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|e| {
                CatalogError::Toml(path.display().to_string(), e.to_string().trim().into())
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(CatalogError::io(path, e)),
        }
    }
}

/// Where a local tool stands on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalStatus {
    /// The executable for this OS exists.
    Ready { folder: PathBuf, exe: PathBuf },
    /// The folder exists but the executable has not been built.
    NotBuilt { folder: PathBuf },
    /// The folder exists; the tool runs from source (`run.dev`).
    FromSource { folder: PathBuf },
    /// `source.root` is not in settings.toml.
    UnknownRoot(String),
    /// The root is set but the tool folder is not there.
    MissingFolder(PathBuf),
    /// No `run.exe` for this OS and no `run.dev`.
    NoExeForOs,
}

pub fn local_status(manifest: &ToolManifest, settings: &Settings, os: Os) -> LocalStatus {
    let SourceSpec {
        root: Some(root),
        path: Some(path),
        ..
    } = &manifest.source
    else {
        // validate() guarantees both are set for local tools.
        return LocalStatus::UnknownRoot(String::new());
    };
    let Some(base) = settings.roots.get(root) else {
        return LocalStatus::UnknownRoot(root.clone());
    };
    let folder = base.join(path);
    if !folder.is_dir() {
        return LocalStatus::MissingFolder(folder);
    }
    match manifest.run.exe.get(&os) {
        Some(exe) => {
            let exe = folder.join(exe);
            if exe.is_file() {
                LocalStatus::Ready { folder, exe }
            } else if manifest.run.dev.is_some() {
                LocalStatus::FromSource { folder }
            } else {
                LocalStatus::NotBuilt { folder }
            }
        }
        None if manifest.run.dev.is_some() => LocalStatus::FromSource { folder },
        None => LocalStatus::NoExeForOs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::parse_manifest;

    const GIT_DASH: &str = r#"
schema = 0
id = "git-dash"
name = "GitDash"
summary = "Aktivite"
category = "git"
kind = "cli"

[source]
root = "coding-vibes"
path = "GitDash"

[run]
host = "terminal"
exe.windows = "zig-out/bin/GitDash.exe"
exe.linux = "zig-out/bin/GitDash"
"#;

    #[test]
    fn missing_settings_file_is_default() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            Settings::load(&dir.path().join("settings.toml")).unwrap(),
            Settings::default()
        );
    }

    #[test]
    fn parses_roots_and_rejects_typos() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        fs::write(&path, "[roots]\ncoding-vibes = \"C:/dev/coding-vibes\"\n").unwrap();
        let s = Settings::load(&path).unwrap();
        assert_eq!(
            s.roots["coding-vibes"],
            PathBuf::from("C:/dev/coding-vibes")
        );

        fs::write(&path, "[root]\nx = \"y\"\n").unwrap();
        assert!(matches!(Settings::load(&path), Err(CatalogError::Toml(..))));
    }

    #[test]
    fn local_status_walks_root_folder_and_exe() {
        let m = parse_manifest(GIT_DASH).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let mut settings = Settings::default();

        assert_eq!(
            local_status(&m, &settings, Os::Linux),
            LocalStatus::UnknownRoot("coding-vibes".into())
        );

        settings
            .roots
            .insert("coding-vibes".into(), dir.path().into());
        assert!(matches!(
            local_status(&m, &settings, Os::Linux),
            LocalStatus::MissingFolder(_)
        ));

        let folder = dir.path().join("GitDash");
        fs::create_dir_all(folder.join("zig-out/bin")).unwrap();
        assert!(matches!(
            local_status(&m, &settings, Os::Linux),
            LocalStatus::NotBuilt { .. }
        ));

        fs::write(folder.join("zig-out/bin/GitDash"), b"").unwrap();
        assert!(matches!(
            local_status(&m, &settings, Os::Linux),
            LocalStatus::Ready { .. }
        ));
        assert_eq!(
            local_status(&m, &settings, Os::Macos),
            LocalStatus::NoExeForOs
        );
    }
}
