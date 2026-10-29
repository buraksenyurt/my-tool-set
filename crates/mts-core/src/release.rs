//! `release.json`: what a tool's GitHub Release contains.
//!
//! Produced by `scripts/release/compose-manifest.sh` in each tool repository.
//! The shell never guesses asset names; it reads this file.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Platform;
use crate::manifest::is_safe_relative_path;
use crate::version::Version;

/// The only release.json schema this build understands.
pub const RELEASE_SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub schema: u32,
    pub tool: String,
    pub repo: String,
    pub version: String,
    pub tag: String,
    pub commit: String,
    pub created_at: String,
    pub readme: String,
    /// Keyed by platform key, e.g. `windows-x64`.
    pub platforms: BTreeMap<String, PlatformRelease>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformRelease {
    pub portable: PortableAsset,
    #[serde(default)]
    pub installers: Vec<Asset>,
}

/// The file the shell downloads, verifies and unpacks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortableAsset {
    pub asset: String,
    pub sha256: String,
    pub size: u64,
    pub format: ArchiveFormat,
    /// What to run once unpacked, relative to the install folder.
    pub entry: String,
}

/// An installer, listed for information only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    pub asset: String,
    pub sha256: String,
    pub size: u64,
    pub format: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ArchiveFormat {
    #[serde(rename = "zip")]
    Zip,
    #[serde(rename = "tar.gz")]
    TarGz,
    /// A single file that is the program itself (e.g. an AppImage).
    #[serde(rename = "file")]
    File,
}

impl ReleaseManifest {
    pub fn for_platform(&self, platform: Platform) -> Option<&PlatformRelease> {
        self.platforms.get(&platform.key())
    }

    /// Checks the file against the tool it is supposed to describe.
    pub fn validate(&self, tool_id: &str, repo: &str) -> Vec<String> {
        let mut problems = Vec::new();
        if self.schema != RELEASE_SCHEMA {
            problems.push(format!(
                "release.json schema = {} desteklenmiyor (beklenen {RELEASE_SCHEMA})",
                self.schema
            ));
        }
        if self.tool != tool_id {
            problems.push(format!(
                "release.json \"{}\" aracına ait, beklenen \"{tool_id}\"",
                self.tool
            ));
        }
        if !self.repo.eq_ignore_ascii_case(repo) {
            problems.push(format!(
                "release.json {} reposuna ait, beklenen {repo}",
                self.repo
            ));
        }
        if Version::parse(&self.version).is_none() {
            problems.push(format!("sürüm \"{}\" semver değil", self.version));
        }
        if self.tag.strip_prefix('v') != Some(self.version.as_str()) {
            problems.push(format!(
                "tag \"{}\" sürümle (\"{}\") eşleşmiyor",
                self.tag, self.version
            ));
        }
        for (key, release) in &self.platforms {
            if Platform::from_key(key).is_none() {
                problems.push(format!("bilinmeyen platform anahtarı \"{key}\""));
            }
            let p = &release.portable;
            if !is_plain_file_name(&p.asset) {
                problems.push(format!("{key}: dosya adı \"{}\" geçersiz", p.asset));
            }
            if !is_sha256(&p.sha256) {
                problems.push(format!("{key}: sha256 64 haneli onaltılık olmalı"));
            }
            if !is_safe_relative_path(&p.entry) {
                problems.push(format!("{key}: entry \"{}\" göreli olmalı", p.entry));
            }
            if p.format == ArchiveFormat::File && p.entry != p.asset {
                problems.push(format!(
                    "{key}: format \"file\" için entry dosyanın kendisi olmalı"
                ));
            }
        }
        problems
    }
}

/// A release asset name: no separators, no `..`.
fn is_plain_file_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\', ':'])
}

fn is_sha256(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Arch, Os};

    // Shape copied from a real release.json produced by the tool repositories.
    const SAMPLE: &str = r#"{
      "schema": 1,
      "tool": "easy-git",
      "repo": "buraksenyurt/easy-git",
      "version": "0.1.0",
      "tag": "v0.1.0",
      "commit": "5d9f639bfc2c13caef9ca236afd13b335ec5e176",
      "created_at": "2026-10-29T14:01:44Z",
      "readme": "README.md",
      "platforms": {
        "linux-x64": {
          "portable": {
            "asset": "easy-git-0.1.0-linux-x64.AppImage",
            "sha256": "d90f557f5259b1ead23ce4fa47048b2c1f59e27af06d3e9d103f792ef6eb9d47",
            "size": 82815480,
            "format": "file",
            "entry": "easy-git-0.1.0-linux-x64.AppImage"
          },
          "installers": [
            { "asset": "easy-git-0.1.0-linux-x64.deb",
              "sha256": "732860595f0c4435b686fecd5dff29ded949ee2fcf56681cd9050fa8809a4438",
              "size": 4317198, "format": "deb" }
          ]
        },
        "macos-arm64": {
          "portable": {
            "asset": "easy-git-0.1.0-macos-arm64.app.tar.gz",
            "sha256": "cd47d6ce8c25e5c92422bfc6e63ae848644460075916584426db7355a2d128a1",
            "size": 4, "format": "tar.gz",
            "entry": "easy-git.app/Contents/MacOS/easy-git"
          }
        }
      }
    }"#;

    fn sample() -> ReleaseManifest {
        serde_json::from_str(SAMPLE).unwrap()
    }

    #[test]
    fn parses_real_release_json() {
        let r = sample();
        assert_eq!(r.version, "0.1.0");
        let linux = r.for_platform(Platform::new(Os::Linux, Arch::X64)).unwrap();
        assert_eq!(linux.portable.format, ArchiveFormat::File);
        assert_eq!(linux.installers.len(), 1);
        let mac = r
            .for_platform(Platform::new(Os::Macos, Arch::Arm64))
            .unwrap();
        assert_eq!(mac.portable.format, ArchiveFormat::TarGz);
        assert!(mac.installers.is_empty());
        assert!(
            r.for_platform(Platform::new(Os::Windows, Arch::X64))
                .is_none()
        );
    }

    #[test]
    fn sample_is_valid_for_its_tool() {
        assert_eq!(
            sample().validate("easy-git", "buraksenyurt/easy-git"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn detects_a_release_of_another_tool() {
        assert_eq!(
            sample()
                .validate("java-atlas", "buraksenyurt/java-atlas")
                .len(),
            2
        );
    }

    #[test]
    fn detects_unsafe_or_broken_assets() {
        let mut r = sample();
        let p = &mut r.platforms.get_mut("linux-x64").unwrap().portable;
        p.asset = "../evil".into();
        p.sha256 = "abc".into();
        r.tag = "v9.9.9".into();
        // bad asset name, bad sha, tag mismatch, entry != asset for "file"
        assert_eq!(r.validate("easy-git", "buraksenyurt/easy-git").len(), 4);
    }
}
