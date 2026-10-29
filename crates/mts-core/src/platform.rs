use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Operating systems a tool can be released for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Os {
    Windows,
    Linux,
    Macos,
}

/// CPU architectures a tool can be released for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Arch {
    X64,
    Arm64,
}

/// An OS + architecture pair. Its key (`windows-x64`, `macos-arm64`, ...) is the
/// same string the tools' `release.json` files use under `platforms`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

impl Platform {
    pub const fn new(os: Os, arch: Arch) -> Self {
        Self { os, arch }
    }

    /// The platform this binary was compiled for, or `None` on a platform the
    /// tools are never released for (32-bit, BSD, ...).
    pub const fn current() -> Option<Self> {
        let os = if cfg!(target_os = "windows") {
            Os::Windows
        } else if cfg!(target_os = "linux") {
            Os::Linux
        } else if cfg!(target_os = "macos") {
            Os::Macos
        } else {
            return None;
        };
        let arch = if cfg!(target_arch = "x86_64") {
            Arch::X64
        } else if cfg!(target_arch = "aarch64") {
            Arch::Arm64
        } else {
            return None;
        };
        Some(Self::new(os, arch))
    }

    /// The key used in `release.json`, e.g. `windows-x64`.
    pub fn key(&self) -> String {
        self.to_string()
    }

    /// Parses a `release.json` key such as `linux-x64`.
    pub fn from_key(key: &str) -> Option<Self> {
        let (os, arch) = key.split_once('-')?;
        let os = match os {
            "windows" => Os::Windows,
            "linux" => Os::Linux,
            "macos" => Os::Macos,
            _ => return None,
        };
        let arch = match arch {
            "x64" => Arch::X64,
            "arm64" => Arch::Arm64,
            _ => return None,
        };
        Some(Self::new(os, arch))
    }
}

impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Os::Windows => "windows",
            Os::Linux => "linux",
            Os::Macos => "macos",
        })
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
        })
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.os, self.arch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_round_trip() {
        for key in ["windows-x64", "windows-arm64", "linux-x64", "macos-arm64"] {
            let platform = Platform::from_key(key).expect(key);
            assert_eq!(platform.key(), key);
        }
    }

    #[test]
    fn rejects_unknown_keys() {
        for key in [
            "",
            "windows",
            "windows-x86",
            "freebsd-x64",
            "linux_x64",
            "x64-linux",
        ] {
            assert_eq!(Platform::from_key(key), None, "{key}");
        }
    }

    #[test]
    fn current_platform_is_known_on_ci_runners() {
        // Every runner we build on (Windows, Ubuntu, macOS) is x64 or arm64.
        let current = Platform::current().expect("unsupported build platform");
        assert_eq!(Platform::from_key(&current.key()), Some(current));
    }

    #[test]
    fn serializes_as_lowercase_parts() {
        let json = serde_json::to_string(&Platform::new(Os::Macos, Arch::Arm64)).unwrap();
        assert_eq!(json, r#"{"os":"macos","arch":"arm64"}"#);
    }
}
