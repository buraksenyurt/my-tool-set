//! README files and the pictures they show, for release and local tools.
//!
//! Release tools: files come from the repository at a revision (a tag, or
//! `HEAD` when nothing is released yet) and are cached under
//! `<cache>/<tool>/<revision>/<path>`. Tags never change, so a cached file is
//! used as is; `HEAD` is always fetched again and the cache is only a fallback
//! when offline.
//!
//! Local tools: files are read from the tool folder, and a path can never
//! leave that folder.

use std::fs;
use std::path::{Path, PathBuf};

use mts_core::manifest::is_safe_relative_path;

use crate::CatalogError;
use crate::error::IoContext;
use crate::releases::ReleaseClient;

/// README text larger than this is not rendered.
pub const MAX_README: u64 = 2 * 1024 * 1024;
/// Pictures larger than this are not served.
pub const MAX_IMAGE: u64 = 20 * 1024 * 1024;

/// The revision used for repositories without a known release.
pub const HEAD: &str = "HEAD";

#[derive(Debug, Clone)]
pub struct DocStore {
    cache_dir: PathBuf,
}

/// A file and whether it came from the cache because the network failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    pub bytes: Vec<u8>,
    pub stale: bool,
}

impl DocStore {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
        }
    }

    fn cache_path(&self, tool: &str, revision: &str, path: &str) -> Option<PathBuf> {
        if !is_safe_relative_path(path) || !is_safe_segment(tool) || !is_safe_segment(revision) {
            return None;
        }
        Some(self.cache_dir.join(tool).join(revision).join(path))
    }

    /// A file from `repo` at `revision`, through the cache.
    pub async fn release_file(
        &self,
        client: &ReleaseClient,
        tool: &str,
        repo: &str,
        revision: &str,
        path: &str,
        max: u64,
    ) -> Result<Fetched, CatalogError> {
        let cached = self
            .cache_path(tool, revision, path)
            .ok_or_else(|| unsafe_path(path))?;
        let immutable = revision != HEAD;
        if immutable && let Ok(bytes) = fs::read(&cached) {
            return Ok(Fetched {
                bytes,
                stale: false,
            });
        }
        match client.raw_file(repo, revision, path, max).await {
            Ok(bytes) => {
                if let Some(dir) = cached.parent() {
                    fs::create_dir_all(dir).at(dir)?;
                }
                let tmp = cached.with_extension("part");
                fs::write(&tmp, &bytes).at(&tmp)?;
                fs::rename(&tmp, &cached).at(&cached)?;
                Ok(Fetched {
                    bytes,
                    stale: false,
                })
            }
            Err(e) => match fs::read(&cached) {
                Ok(bytes) => Ok(Fetched { bytes, stale: true }),
                Err(_) => Err(e),
            },
        }
    }
}

/// A file inside a local tool's folder. Symlinks and `..` that would leave the
/// folder are refused.
pub fn local_file(folder: &Path, path: &str, max: u64) -> Result<Vec<u8>, CatalogError> {
    if !is_safe_relative_path(path) {
        return Err(unsafe_path(path));
    }
    let root = fs::canonicalize(folder).at(folder)?;
    let target = root.join(path);
    let real = fs::canonicalize(&target).at(&target)?;
    if !real.starts_with(&root) {
        return Err(unsafe_path(path));
    }
    let size = fs::metadata(&real).at(&real)?.len();
    if size > max {
        return Err(CatalogError::io(
            &real,
            std::io::Error::other(format!("dosya {max} bayttan büyük")),
        ));
    }
    fs::read(&real).at(&real)
}

fn is_safe_segment(s: &str) -> bool {
    !s.is_empty() && s != "." && s != ".." && !s.contains(['/', '\\', ':'])
}

fn unsafe_path(path: &str) -> CatalogError {
    CatalogError::io(
        path,
        std::io::Error::new(std::io::ErrorKind::PermissionDenied, "güvensiz yol"),
    )
}

/// Content type for the pictures a README may show. Anything else is refused.
pub fn image_content_type(path: &str) -> Option<&'static str> {
    let ext = path.rsplit_once('.')?.1.to_ascii_lowercase();
    Some(match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::releases::test_server::serve;
    use std::collections::HashMap;

    #[tokio::test]
    async fn tags_are_cached_and_served_offline() {
        let server = serve(HashMap::from([(
            "/raw/o/t/v1.0.0/docs/a%20b.png".into(),
            b"PNG".to_vec(),
        )]));
        let client = ReleaseClient::with_base(&server.base, "test").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = DocStore::new(dir.path());

        let first = store
            .release_file(&client, "t", "o/t", "v1.0.0", "docs/a b.png", MAX_IMAGE)
            .await
            .unwrap();
        assert_eq!(first.bytes, b"PNG");
        assert!(dir.path().join("t/v1.0.0/docs/a b.png").is_file());

        // A client that cannot reach anything still gets the cached tag.
        let offline = ReleaseClient::with_base("http://127.0.0.1:1", "test").unwrap();
        let again = store
            .release_file(&offline, "t", "o/t", "v1.0.0", "docs/a b.png", MAX_IMAGE)
            .await
            .unwrap();
        assert_eq!(
            again,
            Fetched {
                bytes: b"PNG".to_vec(),
                stale: false
            }
        );
    }

    #[tokio::test]
    async fn head_is_refetched_and_cache_is_only_a_fallback() {
        let server = serve(HashMap::from([(
            "/raw/o/t/HEAD/README.md".into(),
            b"new".to_vec(),
        )]));
        let client = ReleaseClient::with_base(&server.base, "test").unwrap();
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("t/HEAD")).unwrap();
        fs::write(dir.path().join("t/HEAD/README.md"), b"old").unwrap();
        let store = DocStore::new(dir.path());

        let fresh = store
            .release_file(&client, "t", "o/t", HEAD, "README.md", MAX_README)
            .await
            .unwrap();
        assert_eq!(fresh.bytes, b"new");

        let offline = ReleaseClient::with_base("http://127.0.0.1:1", "test").unwrap();
        let fallback = store
            .release_file(&offline, "t", "o/t", HEAD, "README.md", MAX_README)
            .await
            .unwrap();
        assert_eq!(
            fallback,
            Fetched {
                bytes: b"new".to_vec(),
                stale: true
            }
        );
    }

    #[tokio::test]
    async fn refuses_paths_that_leave_the_cache() {
        let client = ReleaseClient::with_base("http://127.0.0.1:1", "test").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = DocStore::new(dir.path());
        for (tool, rev, path) in [
            ("t", "v1", "../x"),
            ("..", "v1", "a.png"),
            ("t", "a/b", "a.png"),
        ] {
            assert!(
                store
                    .release_file(&client, tool, "o/t", rev, path, 10)
                    .await
                    .is_err(),
                "{tool} {rev} {path}"
            );
        }
    }

    #[tokio::test]
    async fn too_large_files_are_refused() {
        let server = serve(HashMap::from([(
            "/raw/o/t/v1/big.png".into(),
            vec![0u8; 2048],
        )]));
        let client = ReleaseClient::with_base(&server.base, "test").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let err = DocStore::new(dir.path())
            .release_file(&client, "t", "o/t", "v1", "big.png", 1024)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("büyük"), "{err}");
    }

    #[test]
    fn local_files_stay_inside_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        let tool = dir.path().join("GitDash");
        fs::create_dir_all(tool.join("docs")).unwrap();
        fs::write(tool.join("docs/a.png"), b"PNG").unwrap();
        fs::write(dir.path().join("secret.txt"), b"no").unwrap();

        assert_eq!(local_file(&tool, "docs/a.png", MAX_IMAGE).unwrap(), b"PNG");
        assert!(local_file(&tool, "../secret.txt", MAX_IMAGE).is_err());
        assert!(local_file(&tool, "docs/a.png", 1).is_err());

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(dir.path().join("secret.txt"), tool.join("link.png"))
                .unwrap();
            assert!(local_file(&tool, "link.png", MAX_IMAGE).is_err());
        }
    }

    #[test]
    fn only_picture_types_are_served() {
        assert_eq!(image_content_type("a/B.PNG"), Some("image/png"));
        assert_eq!(image_content_type("x.svg"), Some("image/svg+xml"));
        assert_eq!(image_content_type("README.md"), None);
        assert_eq!(image_content_type("noext"), None);
    }
}
