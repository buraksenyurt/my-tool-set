//! Long-lived state shared by the commands.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock, RwLock};

use mts_catalog::cache::ReleaseCache;
use mts_catalog::catalog::{self, Catalog, Origin};
use mts_catalog::docs::DocStore;
use mts_catalog::install::Installer;
use mts_catalog::releases::ReleaseClient;
use mts_catalog::settings::Settings;
use mts_runner::Runner;
use mts_runner::pty::PtyManager;

include!(concat!(env!("OUT_DIR"), "/embedded_catalog.rs"));

/// Template written on first start, so there is a file to edit.
const SETTINGS_TEMPLATE: &str = "\
# my-tool-set ayarları
#
# Release'i olmayan (yerel) araçların bulunduğu kök klasörler. Adlar,
# tool.toml dosyalarındaki source.root değerleriyle eşleşir.
[roots]
# coding-vibes = \"C:/Users/burak/Development/coding-vibes\"
";

#[derive(Debug, Clone)]
pub struct Paths {
    /// Installed tools (large; local, not roaming).
    pub tools: PathBuf,
    /// Cached release.json files.
    pub release_cache: PathBuf,
    /// Cached README files and pictures.
    pub doc_cache: PathBuf,
    /// settings.toml
    pub settings: PathBuf,
    /// Extra or overriding tool.toml files.
    pub user_catalog: PathBuf,
}

impl Paths {
    pub fn new(local_data: &Path, config: &Path) -> Self {
        Self {
            tools: local_data.join("tools"),
            release_cache: local_data.join("cache").join("releases"),
            doc_cache: local_data.join("cache").join("docs"),
            settings: config.join("settings.toml"),
            user_catalog: config.join("catalog"),
        }
    }
}

pub struct AppState {
    pub paths: Paths,
    pub catalog: RwLock<Catalog>,
    pub settings: RwLock<Settings>,
    pub client: ReleaseClient,
    pub cache: ReleaseCache,
    pub installer: Installer,
    pub docs: DocStore,
    /// Tools with an install in progress, so a double click starts one.
    pub installing: Mutex<HashSet<String>>,
    /// Set once the app handle exists (it forwards run events to the webview).
    runner: OnceLock<Runner>,
    /// Embedded terminal sessions.
    pub terminals: PtyManager,
}

impl AppState {
    pub fn new(
        paths: Paths,
        user_agent: &str,
        release_base: Option<&str>,
    ) -> Result<Self, mts_catalog::CatalogError> {
        let client = match release_base {
            Some(base) => ReleaseClient::with_base(base, user_agent)?,
            None => ReleaseClient::new(user_agent)?,
        };
        let state = Self {
            cache: ReleaseCache::new(&paths.release_cache),
            installer: Installer::new(&paths.tools),
            docs: DocStore::new(&paths.doc_cache),
            catalog: RwLock::new(Catalog::default()),
            settings: RwLock::new(Settings::default()),
            installing: Mutex::new(HashSet::new()),
            runner: OnceLock::new(),
            terminals: PtyManager::new(),
            client,
            paths,
        };
        state.ensure_settings_file();
        state.reload();
        Ok(state)
    }

    pub fn set_runner(&self, runner: Runner) {
        let _ = self.runner.set(runner);
    }

    pub fn runner(&self) -> &Runner {
        self.runner.get().expect("runner is set during setup")
    }

    fn ensure_settings_file(&self) {
        if !self.paths.settings.exists() {
            if let Some(dir) = self.paths.settings.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(&self.paths.settings, SETTINGS_TEMPLATE);
        }
    }

    /// Re-reads settings.toml and every catalog source.
    pub fn reload(&self) {
        let mut catalog = Catalog::default();
        catalog.add_all(
            Origin::Embedded,
            EMBEDDED_CATALOG
                .iter()
                .map(|(location, text)| (location.to_string(), (*text).into())),
        );

        // In development, read the repository's catalog/ directly so editing a
        // manifest does not need a rebuild.
        if cfg!(debug_assertions) {
            let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog");
            let (files, issues) = catalog::read_dir(&repo);
            catalog.add_all(Origin::Repository, files);
            for issue in issues {
                catalog.add_issue(issue.location, issue.message);
            }
        }

        let (files, issues) = catalog::read_dir(&self.paths.user_catalog);
        catalog.add_all(Origin::User, files);
        for issue in issues {
            catalog.add_issue(issue.location, issue.message);
        }

        let settings = match Settings::load(&self.paths.settings) {
            Ok(s) => s,
            Err(e) => {
                catalog.add_issue(self.paths.settings.display().to_string(), e.to_string());
                Settings::default()
            }
        };

        catalog.sort();
        *self.catalog.write().unwrap() = catalog;
        *self.settings.write().unwrap() = settings;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_catalog_is_complete_and_valid() {
        let mut catalog = Catalog::default();
        catalog.add_all(
            Origin::Embedded,
            EMBEDDED_CATALOG
                .iter()
                .map(|(l, t)| (l.to_string(), (*t).into())),
        );
        assert_eq!(catalog.issues(), &[]);
        assert_eq!(catalog.entries().len(), 5);
    }

    #[test]
    fn first_start_writes_a_settings_template_that_parses() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(&dir.path().join("local"), &dir.path().join("config"));
        let state = AppState::new(paths.clone(), "test", None).unwrap();
        assert!(paths.settings.is_file());
        assert!(state.catalog.read().unwrap().issues().is_empty());
        assert!(state.settings.read().unwrap().roots.is_empty());
    }
}
