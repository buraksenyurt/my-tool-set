//! `tool.toml`: what a tool is and how the shell gets and runs it.
//!
//! The types mirror the file one to one (serde), and `validate` adds the rules
//! a TOML schema cannot express. Parsing the TOML text lives in mts-catalog.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Os;

/// The only manifest schema this build understands.
pub const MANIFEST_SCHEMA: u32 = 0;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolManifest {
    pub schema: u32,
    pub id: String,
    pub name: String,
    pub summary: String,
    pub category: String,
    pub kind: ToolKind,
    #[serde(default)]
    pub stack: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Set for tools distributed through GitHub Releases.
    pub release: Option<ReleaseRef>,
    #[serde(default)]
    pub source: SourceSpec,
    #[serde(default)]
    pub platforms: PlatformSupport,
    pub build: Option<BuildSpec>,
    pub run: RunSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ToolKind {
    Gui,
    Cli,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseRef {
    /// `owner/name` on GitHub.
    pub repo: String,
}

/// For release tools only `readme` and `screenshots` are used (relative to the
/// repository at the release tag). Local tools also set `root` and `path`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpec {
    /// A name from `[roots]` in settings.toml.
    pub root: Option<String>,
    /// Folder relative to the root.
    pub path: Option<String>,
    pub readme: Option<String>,
    #[serde(default)]
    pub screenshots: Vec<String>,
}

/// How well the author says the tool works on each OS. Independent of whether
/// a release actually contains a build for it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformSupport {
    #[serde(default)]
    pub windows: Support,
    #[serde(default)]
    pub linux: Support,
    #[serde(default)]
    pub macos: Support,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Support {
    Yes,
    #[default]
    Untested,
    No,
}

impl PlatformSupport {
    pub fn on(&self, os: Os) -> Support {
        match os {
            Os::Windows => self.windows,
            Os::Linux => self.linux,
            Os::Macos => self.macos,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildSpec {
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunSpec {
    pub host: HostMode,
    /// Local tools: executable per OS, relative to the tool folder.
    #[serde(default)]
    pub exe: BTreeMap<Os, String>,
    /// Local tools: a command that runs the tool from source.
    pub dev: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub params: Vec<RunParam>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum HostMode {
    /// A separate process with its own window.
    External,
    /// Inside the shell's embedded terminal.
    Terminal,
    /// Compiled into the shell as a plugin. Not supported in the MVP.
    Module,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct RunParam {
    pub name: String,
    pub kind: ParamKind,
    pub label: String,
    /// Passed as `<flag> <value>` (e.g. `--days 60`); positional when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub flag: Option<String>,
    /// Omitted (not `null`) when unset, to match the optional field in TypeScript.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub default: Option<ParamValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ParamKind {
    Folder,
    File,
    Number,
    Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum ParamValue {
    Number(i32),
    Text(String),
}

/// Where a tool's files come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    Release,
    Local,
}

impl ToolManifest {
    pub fn delivery(&self) -> Delivery {
        if self.release.is_some() {
            Delivery::Release
        } else {
            Delivery::Local
        }
    }

    /// Rules that serde cannot check. An empty list means the manifest is usable.
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let mut problem = |m: String| problems.push(m);

        if self.schema != MANIFEST_SCHEMA {
            problem(format!(
                "schema = {} desteklenmiyor (beklenen {MANIFEST_SCHEMA})",
                self.schema
            ));
        }
        if !is_kebab_case(&self.id) {
            problem(format!(
                "id \"{}\" kebab-case olmalı (küçük harf, rakam ve tire)",
                self.id
            ));
        }
        for (field, value) in [
            ("name", &self.name),
            ("summary", &self.summary),
            ("category", &self.category),
        ] {
            if value.trim().is_empty() {
                problem(format!("{field} boş olamaz"));
            }
        }

        match &self.release {
            Some(release) => {
                if !is_github_repo(&release.repo) {
                    problem(format!(
                        "release.repo \"{}\" owner/name biçiminde olmalı",
                        release.repo
                    ));
                }
                if self.source.root.is_some() || self.source.path.is_some() {
                    problem("[release] araçları source.root / source.path kullanmaz".into());
                }
                if !self.run.exe.is_empty() || self.run.dev.is_some() {
                    problem(
                        "[release] araçlarında çalıştırılacak dosya release.json'dan gelir; \
                         run.exe / run.dev kullanılmaz"
                            .into(),
                    );
                }
                if self.build.is_some() {
                    problem("[release] araçları [build] kullanmaz".into());
                }
            }
            None => {
                if self.source.root.is_none() || self.source.path.is_none() {
                    problem("[release] yoksa source.root ve source.path ikisi de gerekli".into());
                }
                if self.run.exe.is_empty() && self.run.dev.is_none() {
                    problem("yerel araçlarda run.exe ya da run.dev gerekli".into());
                }
            }
        }

        if self.run.host == HostMode::Module {
            problem("run.host = \"module\" MVP'de desteklenmiyor".into());
        }

        for path in self
            .source
            .readme
            .iter()
            .chain(self.source.screenshots.iter())
            .chain(self.source.path.iter())
            .chain(self.run.exe.values())
        {
            if !is_safe_relative_path(path) {
                problem(format!("\"{path}\" göreli olmalı ve \"..\" içermemeli"));
            }
        }

        let mut seen = std::collections::BTreeSet::new();
        for param in &self.run.params {
            if let Some(flag) = &param.flag
                && (!flag.starts_with('-') || flag.contains(char::is_whitespace))
            {
                problem(format!(
                    "run.params.{}: flag \"{flag}\" - ile başlamalı ve boşluk içermemeli",
                    param.name
                ));
            }
            if !seen.insert(param.name.as_str()) {
                problem(format!(
                    "run.params içinde \"{}\" iki kez geçiyor",
                    param.name
                ));
            }
            match (&param.kind, &param.default) {
                (ParamKind::Number, Some(ParamValue::Text(_))) => problem(format!(
                    "run.params.{}: default bir sayı olmalı",
                    param.name
                )),
                (
                    ParamKind::Folder | ParamKind::File | ParamKind::Text,
                    Some(ParamValue::Number(_)),
                ) => problem(format!(
                    "run.params.{}: default bir metin olmalı",
                    param.name
                )),
                _ => {}
            }
        }

        problems
    }
}

pub(crate) fn is_kebab_case(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn is_github_repo(s: &str) -> bool {
    let ok = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
            && part != "."
            && part != ".."
    };
    matches!(s.split_once('/'), Some((owner, name)) if ok(owner) && ok(name))
}

/// Relative, no `..`, no drive letters or roots. Both `/` and `\` count as
/// separators so a Windows-style path is checked the same way.
pub fn is_safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with(['/', '\\'])
        && !path.contains(':')
        && path
            .split(['/', '\\'])
            .all(|part| part != ".." && !part.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release_tool() -> ToolManifest {
        ToolManifest {
            schema: 0,
            id: "easy-git".into(),
            name: "easy-git".into(),
            summary: "Git dalları".into(),
            category: "git".into(),
            kind: ToolKind::Gui,
            stack: vec![],
            tags: vec![],
            release: Some(ReleaseRef {
                repo: "buraksenyurt/easy-git".into(),
            }),
            source: SourceSpec {
                readme: Some("README.md".into()),
                ..Default::default()
            },
            platforms: PlatformSupport::default(),
            build: None,
            run: RunSpec {
                host: HostMode::External,
                exe: BTreeMap::new(),
                dev: None,
                args: vec![],
                params: vec![],
            },
        }
    }

    fn local_tool() -> ToolManifest {
        let mut m = release_tool();
        m.id = "git-dash".into();
        m.kind = ToolKind::Cli;
        m.release = None;
        m.source.root = Some("coding-vibes".into());
        m.source.path = Some("GitDash".into());
        m.run.host = HostMode::Terminal;
        m.run
            .exe
            .insert(Os::Windows, "zig-out/bin/GitDash.exe".into());
        m
    }

    #[test]
    fn valid_manifests_have_no_problems() {
        assert_eq!(release_tool().validate(), Vec::<String>::new());
        assert_eq!(local_tool().validate(), Vec::<String>::new());
    }

    #[test]
    fn delivery_follows_release_section() {
        assert_eq!(release_tool().delivery(), Delivery::Release);
        assert_eq!(local_tool().delivery(), Delivery::Local);
    }

    #[test]
    fn release_tools_cannot_point_at_local_files() {
        let mut m = release_tool();
        m.source.root = Some("coding-vibes".into());
        m.run.dev = Some("npm run tauri dev".into());
        let problems = m.validate();
        assert_eq!(problems.len(), 2, "{problems:?}");
    }

    #[test]
    fn local_tools_need_root_path_and_a_way_to_run() {
        let mut m = local_tool();
        m.source.path = None;
        m.run.exe.clear();
        assert_eq!(m.validate().len(), 2);
    }

    #[test]
    fn rejects_bad_ids_and_repos() {
        let mut m = release_tool();
        m.id = "Easy_Git".into();
        m.release = Some(ReleaseRef {
            repo: "easy-git".into(),
        });
        assert_eq!(m.validate().len(), 2);
    }

    #[test]
    fn rejects_unsupported_schema_and_module_host() {
        let mut m = release_tool();
        m.schema = 1;
        m.run.host = HostMode::Module;
        assert_eq!(m.validate().len(), 2);
    }

    #[test]
    fn rejects_escaping_paths() {
        let mut m = local_tool();
        m.source.path = Some("../outside".into());
        m.source.screenshots = vec!["C:\\shot.png".into(), "/abs.png".into()];
        assert_eq!(m.validate().len(), 3);
    }

    #[test]
    fn checks_param_defaults_against_kind() {
        let mut m = local_tool();
        m.run.params = vec![
            RunParam {
                name: "days".into(),
                kind: ParamKind::Number,
                label: "Gün".into(),
                flag: None,
                default: Some(ParamValue::Text("180".into())),
            },
            RunParam {
                name: "days".into(),
                kind: ParamKind::Number,
                label: "Gün".into(),
                flag: Some("days".into()),
                default: Some(ParamValue::Number(180)),
            },
        ];
        // wrong default type + bad flag + duplicate name
        assert_eq!(m.validate().len(), 3);
    }

    #[test]
    fn unset_param_default_is_omitted_in_json() {
        let param = RunParam {
            name: "repo".into(),
            kind: ParamKind::Folder,
            label: "Repo".into(),
            flag: None,
            default: None,
        };
        let json = serde_json::to_string(&param).unwrap();
        assert_eq!(json, r#"{"name":"repo","kind":"folder","label":"Repo"}"#);
    }

    #[test]
    fn kebab_case_rules() {
        for ok in ["a", "easy-git", "tool-2"] {
            assert!(is_kebab_case(ok), "{ok}");
        }
        for bad in ["", "-a", "a-", "a--b", "A", "a_b", "a b"] {
            assert!(!is_kebab_case(bad), "{bad}");
        }
    }

    #[test]
    fn safe_relative_paths() {
        for ok in ["README.md", "docs/images/a.png", "src\\x.exe"] {
            assert!(is_safe_relative_path(ok), "{ok}");
        }
        for bad in ["", "/a", "\\a", "a/../b", "..", "C:\\a", "a//b"] {
            assert!(!is_safe_relative_path(bad), "{bad}");
        }
    }
}
