//! Talks to GitHub Releases without the GitHub API.
//!
//! `https://github.com/<repo>/releases/latest/download/release.json` redirects to
//! the newest published release's file, and assets live at
//! `.../releases/download/<tag>/<asset>`. Neither is subject to the API's
//! 60-requests-per-hour limit, and no token is needed.

use std::path::Path;
use std::sync::Once;
use std::time::Duration;

use futures_util::StreamExt;
use mts_core::ReleaseManifest;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use crate::CatalogError;
use crate::error::IoContext;

#[derive(Debug, Clone)]
pub struct ReleaseClient {
    http: reqwest::Client,
    base: String,
    /// Where repository files are read from at a revision.
    raw_base: String,
}

/// Bytes received so far and the expected total.
pub type ProgressFn<'a> = &'a (dyn Fn(u64, u64) + Send + Sync);

impl ReleaseClient {
    /// A client for github.com.
    pub fn new(user_agent: &str) -> Result<Self, CatalogError> {
        let mut client = Self::with_base("https://github.com", user_agent)?;
        client.raw_base = "https://raw.githubusercontent.com".into();
        Ok(client)
    }

    /// A client for another host with the same URL layout (tests use a local
    /// server). Repository files are read from `<base>/raw/...`.
    pub fn with_base(base: &str, user_agent: &str) -> Result<Self, CatalogError> {
        install_crypto_provider();
        let http = reqwest::Client::builder()
            .user_agent(user_agent)
            .connect_timeout(Duration::from_secs(15))
            .read_timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| CatalogError::Http {
                url: base.into(),
                message: e.to_string(),
            })?;
        let base = base.trim_end_matches('/').to_string();
        Ok(Self {
            http,
            raw_base: format!("{base}/raw"),
            base,
        })
    }

    pub fn latest_manifest_url(&self, repo: &str) -> String {
        format!("{}/{repo}/releases/latest/download/release.json", self.base)
    }

    pub fn asset_url(&self, repo: &str, tag: &str, asset: &str) -> String {
        format!("{}/{repo}/releases/download/{tag}/{asset}", self.base)
    }

    /// `<raw>/<repo>/<revision>/<path>`, each segment percent-encoded.
    pub fn raw_url(&self, repo: &str, revision: &str, path: &str) -> String {
        format!(
            "{}/{repo}/{revision}/{}",
            self.raw_base,
            crate::markdown::encode_path(path)
        )
    }

    /// A file from the repository at `revision`, at most `max` bytes.
    pub async fn raw_file(
        &self,
        repo: &str,
        revision: &str,
        path: &str,
        max: u64,
    ) -> Result<Vec<u8>, CatalogError> {
        let url = self.raw_url(repo, revision, path);
        let response = self.get(&url).await?;
        if response.content_length().is_some_and(|len| len > max) {
            return Err(CatalogError::Http {
                url,
                message: format!("dosya {max} bayttan büyük"),
            });
        }
        let mut stream = response.bytes_stream();
        let mut out = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| http_error(&url, e))?;
            if (out.len() + chunk.len()) as u64 > max {
                return Err(CatalogError::Http {
                    url,
                    message: format!("dosya {max} bayttan büyük"),
                });
            }
            out.extend_from_slice(&chunk);
        }
        Ok(out)
    }

    /// The newest published release of `repo`.
    pub async fn latest(&self, repo: &str) -> Result<ReleaseManifest, CatalogError> {
        let url = self.latest_manifest_url(repo);
        let response = self.get(&url).await?;
        let bytes = response.bytes().await.map_err(|e| http_error(&url, e))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| CatalogError::BadReleaseJson(url.clone(), e.to_string()))
    }

    /// Streams `url` into `dest` and returns the file's SHA256 (hex) and size.
    pub async fn download(
        &self,
        url: &str,
        dest: &Path,
        expected_size: u64,
        progress: ProgressFn<'_>,
    ) -> Result<(String, u64), CatalogError> {
        let response = self.get(url).await?;
        let mut file = tokio::fs::File::create(dest).await.at(dest)?;
        let mut hasher = Sha256::new();
        let mut received = 0u64;
        let mut stream = response.bytes_stream();
        progress(0, expected_size);
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| http_error(url, e))?;
            received += chunk.len() as u64;
            // Never write more than promised: a wrong or hostile server cannot
            // fill the disk.
            if received > expected_size {
                return Err(CatalogError::SizeMismatch {
                    asset: file_name(url),
                    expected: expected_size,
                    actual: received,
                });
            }
            hasher.update(&chunk);
            file.write_all(&chunk).await.at(dest)?;
            progress(received, expected_size);
        }
        file.flush().await.at(dest)?;
        Ok((hex::encode(hasher.finalize()), received))
    }

    async fn get(&self, url: &str) -> Result<reqwest::Response, CatalogError> {
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| http_error(url, e))?;
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(CatalogError::Http {
                url: url.into(),
                message: "bulunamadı (404) — henüz yayınlanmış bir sürüm olmayabilir".into(),
            });
        }
        if !status.is_success() {
            return Err(CatalogError::Http {
                url: url.into(),
                message: format!("HTTP {status}"),
            });
        }
        Ok(response)
    }
}

fn http_error(url: &str, e: reqwest::Error) -> CatalogError {
    let message = if e.is_timeout() {
        "zaman aşımı".to_string()
    } else if e.is_connect() {
        "bağlanılamadı (ağ bağlantısını kontrol et)".to_string()
    } else {
        e.to_string()
    };
    CatalogError::Http {
        url: url.into(),
        message,
    }
}

fn file_name(url: &str) -> String {
    url.rsplit('/').next().unwrap_or(url).to_string()
}

/// rustls needs a process-wide crypto provider; we use ring (see Cargo.toml).
fn install_crypto_provider() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        // Err only means another part of the process installed one first.
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

#[cfg(test)]
pub(crate) mod test_server {
    //! A tiny HTTP/1.1 server for tests: serves fixed bodies by path.

    use std::collections::HashMap;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::sync::Arc;

    pub struct Server {
        pub base: String,
    }

    pub fn serve(routes: HashMap<String, Vec<u8>>) -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let routes = Arc::new(routes);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let routes = routes.clone();
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    let path = line.split_whitespace().nth(1).unwrap_or("/").to_string();
                    loop {
                        let mut header = String::new();
                        if reader.read_line(&mut header).unwrap() <= 2 {
                            break;
                        }
                    }
                    let (status, body) = match routes.get(&path) {
                        Some(b) => ("200 OK", b.clone()),
                        None => ("404 Not Found", b"nope".to_vec()),
                    };
                    let head = format!(
                        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(head.as_bytes());
                    let _ = stream.write_all(&body);
                });
            }
        });
        Server { base }
    }
}

#[cfg(test)]
mod tests {
    use super::test_server::serve;
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};

    const RELEASE_JSON: &str = r#"{"schema":1,"tool":"t","repo":"o/t","version":"1.0.0","tag":"v1.0.0",
        "commit":"c","created_at":"x","readme":"README.md","platforms":{}}"#;

    #[tokio::test]
    async fn fetches_latest_release_json() {
        let server = serve(HashMap::from([(
            "/o/t/releases/latest/download/release.json".into(),
            RELEASE_JSON.as_bytes().to_vec(),
        )]));
        let client = ReleaseClient::with_base(&server.base, "test").unwrap();
        let r = client.latest("o/t").await.unwrap();
        assert_eq!(r.version, "1.0.0");
    }

    #[tokio::test]
    async fn missing_release_is_a_clear_404() {
        let server = serve(HashMap::new());
        let client = ReleaseClient::with_base(&server.base, "test").unwrap();
        let err = client.latest("o/none").await.unwrap_err().to_string();
        assert!(err.contains("404"), "{err}");
    }

    #[tokio::test]
    async fn download_hashes_and_reports_progress() {
        let body = vec![7u8; 100_000];
        let server = serve(HashMap::from([("/f.bin".into(), body.clone())]));
        let client = ReleaseClient::with_base(&server.base, "test").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("f.bin");
        let last = AtomicU64::new(0);
        let (sha, size) = client
            .download(
                &format!("{}/f.bin", server.base),
                &dest,
                100_000,
                &|r, _| last.store(r, Ordering::Relaxed),
            )
            .await
            .unwrap();
        assert_eq!(size, 100_000);
        assert_eq!(last.load(Ordering::Relaxed), 100_000);
        assert_eq!(sha, hex::encode(Sha256::digest(&body)));
        assert_eq!(std::fs::read(&dest).unwrap(), body);
    }

    #[tokio::test]
    async fn download_stops_when_larger_than_promised() {
        let server = serve(HashMap::from([("/big".into(), vec![0u8; 5000])]));
        let client = ReleaseClient::with_base(&server.base, "test").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let err = client
            .download(
                &format!("{}/big", server.base),
                &dir.path().join("big"),
                1000,
                &|_, _| {},
            )
            .await
            .unwrap_err();
        assert!(matches!(err, CatalogError::SizeMismatch { .. }), "{err}");
    }
}
