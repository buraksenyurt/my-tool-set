//! Semantic versions, for "is there something newer than what is installed?".

use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version(semver::Version);

impl Version {
    /// Accepts `1.2.3` and `v1.2.3`.
    pub fn parse(s: &str) -> Option<Self> {
        semver::Version::parse(s.strip_prefix('v').unwrap_or(s))
            .ok()
            .map(Self)
    }

    pub fn is_prerelease(&self) -> bool {
        !self.0.pre.is_empty()
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Is `candidate` newer than `current`? Unparseable versions are never newer.
pub fn is_newer(candidate: &str, current: &str) -> bool {
    match (Version::parse(candidate), Version::parse(current)) {
        (Some(c), Some(i)) => c > i,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_numerically_not_as_text() {
        assert!(is_newer("0.10.0", "0.9.0"));
        assert!(is_newer("v1.0.0", "0.99.99"));
        assert!(!is_newer("0.1.0", "0.1.0"));
    }

    #[test]
    fn prerelease_is_older_than_release() {
        assert!(is_newer("1.0.0", "1.0.0-beta.1"));
        assert!(Version::parse("1.0.0-rc.1").unwrap().is_prerelease());
    }

    #[test]
    fn garbage_is_never_newer() {
        assert!(!is_newer("latest", "0.1.0"));
        assert!(!is_newer("0.2.0", "dev"));
    }
}
