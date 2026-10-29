//! Builds the list of tools from `tool.toml` files.
//!
//! Sources are added in order (embedded → repository → user folder). A later
//! manifest with the same `id` replaces an earlier one, so a user can override
//! a built-in tool. A broken manifest never hides the others: it becomes an
//! [`Issue`] and the rest of the catalog loads.

use std::fs;
use std::path::{Path, PathBuf};

use mts_core::ToolManifest;

/// Where a manifest came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Compiled into the shell.
    Embedded,
    /// The repository's `catalog/` folder (debug builds only).
    Repository,
    /// `<config>/catalog/` on this machine.
    User,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub manifest: ToolManifest,
    pub origin: Origin,
    /// e.g. `easy-git/tool.toml` or a full path for user files.
    pub location: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub location: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct Catalog {
    entries: Vec<Entry>,
    issues: Vec<Issue>,
}

impl Catalog {
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    pub fn get(&self, id: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.manifest.id == id)
    }

    /// Parses, validates and adds one manifest.
    pub fn add(&mut self, origin: Origin, location: impl Into<String>, text: &str) {
        let location = location.into();
        let manifest = match parse_manifest(text) {
            Ok(m) => m,
            Err(message) => {
                self.issues.push(Issue { location, message });
                return;
            }
        };
        let problems = manifest.validate();
        if !problems.is_empty() {
            self.issues.push(Issue {
                location,
                message: problems.join("; "),
            });
            return;
        }
        let entry = Entry {
            manifest,
            origin,
            location,
        };
        match self
            .entries
            .iter_mut()
            .find(|e| e.manifest.id == entry.manifest.id)
        {
            Some(existing) => *existing = entry,
            None => self.entries.push(entry),
        }
    }

    pub fn add_all<'a>(
        &mut self,
        origin: Origin,
        files: impl IntoIterator<Item = (String, std::borrow::Cow<'a, str>)>,
    ) {
        for (location, text) in files {
            self.add(origin, location, &text);
        }
    }

    /// Records a problem that is not tied to a single manifest's content.
    pub fn add_issue(&mut self, location: impl Into<String>, message: impl Into<String>) {
        self.issues.push(Issue {
            location: location.into(),
            message: message.into(),
        });
    }

    /// Category, then name; stable for the sidebar.
    pub fn sort(&mut self) {
        self.entries.sort_by(|a, b| {
            (&a.manifest.category, a.manifest.name.to_lowercase())
                .cmp(&(&b.manifest.category, b.manifest.name.to_lowercase()))
        });
    }
}

pub fn parse_manifest(text: &str) -> Result<ToolManifest, String> {
    toml::from_str(text).map_err(|e| e.to_string().trim().to_string())
}

/// Reads `<dir>/<anything>/tool.toml`. A missing folder is not an error: it
/// just means the user has not added any tools.
pub fn read_dir(dir: &Path) -> (Vec<(String, std::borrow::Cow<'static, str>)>, Vec<Issue>) {
    let mut files = Vec::new();
    let mut issues = Vec::new();
    let Ok(read) = fs::read_dir(dir) else {
        return (files, issues);
    };
    let mut paths: Vec<PathBuf> = read
        .filter_map(|e| e.ok())
        .map(|e| e.path().join("tool.toml"))
        .filter(|p| p.is_file())
        .collect();
    paths.sort();
    for path in paths {
        let location = path.display().to_string();
        match fs::read_to_string(&path) {
            Ok(text) => files.push((location, text.into())),
            Err(e) => issues.push(Issue {
                location,
                message: e.to_string(),
            }),
        }
    }
    (files, issues)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EASY_GIT: &str = r#"
schema = 0
id       = "easy-git"
name     = "easy-git"
summary  = "Git dalları"
category = "git"
kind     = "gui"

[release]
repo = "buraksenyurt/easy-git"

[run]
host = "external"
"#;

    #[test]
    fn real_catalog_files_load_cleanly() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog");
        let (files, read_issues) = read_dir(&dir);
        assert!(read_issues.is_empty(), "{read_issues:?}");
        let mut catalog = Catalog::default();
        catalog.add_all(Origin::Repository, files);
        assert_eq!(catalog.issues(), &[], "catalog/ has invalid manifests");
        let mut ids: Vec<_> = catalog
            .entries()
            .iter()
            .map(|e| e.manifest.id.as_str())
            .collect();
        ids.sort();
        assert_eq!(
            ids,
            ["docker-city", "docky", "easy-git", "git-dash", "java-atlas"]
        );
    }

    #[test]
    fn later_source_overrides_same_id() {
        let mut catalog = Catalog::default();
        catalog.add(Origin::Embedded, "a", EASY_GIT);
        catalog.add(
            Origin::User,
            "b",
            &EASY_GIT.replace("Git dalları", "Benim sürümüm"),
        );
        assert_eq!(catalog.entries().len(), 1);
        let e = catalog.get("easy-git").unwrap();
        assert_eq!(e.origin, Origin::User);
        assert_eq!(e.manifest.summary, "Benim sürümüm");
    }

    #[test]
    fn broken_manifest_becomes_an_issue_not_a_failure() {
        let mut catalog = Catalog::default();
        catalog.add(Origin::User, "bad.toml", "schema = ");
        catalog.add(
            Origin::User,
            "unknown-field.toml",
            &format!("{EASY_GIT}\nextra = 1"),
        );
        catalog.add(
            Origin::User,
            "invalid.toml",
            &EASY_GIT.replace("easy-git\"\nname", "Easy Git\"\nname"),
        );
        catalog.add(Origin::Embedded, "ok", EASY_GIT);
        assert_eq!(catalog.entries().len(), 1);
        assert_eq!(catalog.issues().len(), 3, "{:?}", catalog.issues());
        assert!(catalog.issues()[1].message.contains("extra"));
        assert!(catalog.issues()[2].message.contains("kebab-case"));
    }

    #[test]
    fn missing_user_folder_is_empty_not_an_error() {
        let (files, issues) = read_dir(Path::new("/definitely/not/here"));
        assert!(files.is_empty() && issues.is_empty());
    }
}
