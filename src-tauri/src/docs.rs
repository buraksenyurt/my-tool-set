//! README and screenshots for the tool tab, and the `mtsimg` protocol that
//! serves their pictures to the webview.
//!
//! The webview never fetches from the network: README HTML is rendered here,
//! and every picture URL in it points at `mtsimg`, which only serves files
//! that belong to a tool in the catalog (its repository at the shown
//! revision, or its local folder).

use std::borrow::Cow;
use std::path::PathBuf;

use mts_catalog::docs::{self, HEAD, image_content_type};
use mts_catalog::markdown::{self, DocContext};
use mts_catalog::settings::{LocalStatus, local_status};
use mts_core::{Delivery, Platform, ToolManifest};
use percent_encoding::percent_decode_str;
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{AppHandle, Manager, State};

use crate::dto::ReadmeDto;
use crate::error::{AppError, ErrorKind};
use crate::state::AppState;

pub const SCHEME: &str = "mtsimg";

/// How custom protocols are addressed in the webview: Windows (WebView2) and
/// Android use `http://<scheme>.localhost`, the others `<scheme>://localhost`.
pub fn asset_base() -> &'static str {
    if cfg!(any(windows, target_os = "android")) {
        "http://mtsimg.localhost"
    } else {
        "mtsimg://localhost"
    }
}

pub const LOCAL_REVISION: &str = "local";

/// Where a tool's documentation comes from right now.
#[derive(Debug, Clone)]
pub enum DocSource {
    Release {
        repo: String,
        revision: String,
    },
    Local {
        folder: PathBuf,
    },
    /// A local tool whose folder cannot be found (no root, missing folder).
    Unavailable(String),
}

impl DocSource {
    pub fn revision(&self) -> Option<&str> {
        match self {
            DocSource::Release { revision, .. } => Some(revision),
            DocSource::Local { .. } => Some(LOCAL_REVISION),
            DocSource::Unavailable(_) => None,
        }
    }
}

/// Release tools: the installed version's tag (so the README matches what is
/// on disk), else the latest known tag, else `HEAD`.
pub fn doc_source(state: &AppState, m: &ToolManifest) -> DocSource {
    match m.delivery() {
        Delivery::Release => {
            let repo = m
                .release
                .as_ref()
                .map(|r| r.repo.clone())
                .unwrap_or_default();
            let revision = state
                .installer
                .installed(&m.id)
                .first()
                .map(|i| i.record.tag.clone())
                .or_else(|| state.cache.get(&m.id).map(|c| c.release.tag))
                .unwrap_or_else(|| HEAD.to_string());
            DocSource::Release { repo, revision }
        }
        Delivery::Local => {
            let settings = state.settings.read().unwrap();
            let Some(platform) = Platform::current() else {
                return DocSource::Unavailable("Platform bilinmiyor".into());
            };
            match local_status(m, &settings, platform.os) {
                LocalStatus::Ready { folder, .. }
                | LocalStatus::NotBuilt { folder }
                | LocalStatus::FromSource { folder } => DocSource::Local { folder },
                LocalStatus::UnknownRoot(root) => DocSource::Unavailable(format!(
                    "settings.toml içinde \"{root}\" kökü tanımlı değil"
                )),
                LocalStatus::MissingFolder(folder) => {
                    DocSource::Unavailable(format!("Klasör bulunamadı: {}", folder.display()))
                }
                LocalStatus::NoExeForOs => match (&m.source.root, &m.source.path) {
                    (Some(root), Some(path)) => match settings.roots.get(root) {
                        Some(base) if base.join(path).is_dir() => DocSource::Local {
                            folder: base.join(path),
                        },
                        _ => DocSource::Unavailable("Araç klasörü bulunamadı".into()),
                    },
                    _ => DocSource::Unavailable("Araç klasörü bulunamadı".into()),
                },
            }
        }
    }
}

/// Picture URLs for the manifest's `source.screenshots`.
pub fn screenshot_urls(m: &ToolManifest, source: &DocSource) -> Vec<String> {
    let Some(revision) = source.revision() else {
        return Vec::new();
    };
    m.source
        .screenshots
        .iter()
        .map(|p| markdown::asset_url(asset_base(), &m.id, revision, p))
        .collect()
}

#[tauri::command]
pub async fn get_readme(id: String, state: State<'_, AppState>) -> Result<ReadmeDto, AppError> {
    let (manifest, source) = {
        let catalog = state.catalog.read().unwrap();
        let entry = catalog.get(&id).ok_or_else(|| {
            AppError::new(ErrorKind::UnknownTool, format!("Katalogda \"{id}\" yok"))
        })?;
        let m = entry.manifest.clone();
        let source = doc_source(&state, &m);
        (m, source)
    };
    let readme_path = manifest
        .source
        .readme
        .clone()
        .unwrap_or_else(|| "README.md".into());

    let (bytes, stale, repo, revision, web_url) = match &source {
        DocSource::Unavailable(reason) => {
            return Ok(ReadmeDto {
                html: None,
                revision: None,
                web_url: None,
                stale: false,
                error: Some(reason.clone()),
            });
        }
        DocSource::Release { repo, revision } => {
            let web_url = format!("https://github.com/{repo}/blob/{revision}/{readme_path}");
            match state
                .docs
                .release_file(
                    &state.client,
                    &id,
                    repo,
                    revision,
                    &readme_path,
                    docs::MAX_README,
                )
                .await
            {
                Ok(f) => (
                    f.bytes,
                    f.stale,
                    Some(repo.clone()),
                    revision.clone(),
                    Some(web_url),
                ),
                Err(e) => {
                    return Ok(ReadmeDto {
                        html: None,
                        revision: Some(revision.clone()),
                        web_url: Some(web_url),
                        stale: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        DocSource::Local { folder } => {
            let folder = folder.clone();
            let path = readme_path.clone();
            let read = tokio::task::spawn_blocking(move || {
                docs::local_file(&folder, &path, docs::MAX_README)
            })
            .await
            .map_err(|e| AppError::new(ErrorKind::Internal, e.to_string()))?;
            match read {
                Ok(bytes) => (bytes, false, None, LOCAL_REVISION.to_string(), None),
                Err(e) => {
                    return Ok(ReadmeDto {
                        html: None,
                        revision: Some(LOCAL_REVISION.into()),
                        web_url: None,
                        stale: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
    };

    let text = String::from_utf8_lossy(&bytes).into_owned();
    let ctx = DocContext {
        tool: id,
        revision: revision.clone(),
        repo,
        asset_base: asset_base().into(),
        readme_path,
    };
    let html = tokio::task::spawn_blocking(move || markdown::render(&text, &ctx))
        .await
        .map_err(|e| AppError::new(ErrorKind::Internal, e.to_string()))?;
    Ok(ReadmeDto {
        html: Some(html),
        revision: Some(revision),
        web_url,
        stale,
        error: None,
    })
}

/// `mtsimg://localhost/<tool>/<revision>/<path>` → the picture's bytes.
/// Runs on the async runtime: a slow download must never block the UI thread.
pub async fn respond(app: AppHandle, request: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    let path = request.uri().path().trim_start_matches('/').to_string();
    match serve(&app, &path).await {
        Ok((bytes, content_type)) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CACHE_CONTROL, "max-age=3600")
            .body(Cow::Owned(bytes))
            .unwrap(),
        Err((status, message)) => Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Cow::Owned(message.into_bytes()))
            .unwrap(),
    }
}

async fn serve(
    app: &AppHandle,
    raw_path: &str,
) -> Result<(Vec<u8>, &'static str), (StatusCode, String)> {
    let not_found = |m: &str| (StatusCode::NOT_FOUND, m.to_string());
    let mut parts = raw_path.splitn(3, '/');
    let (Some(tool), Some(revision), Some(path)) = (parts.next(), parts.next(), parts.next())
    else {
        return Err(not_found("geçersiz adres"));
    };
    let decode = |s: &str| percent_decode_str(s).decode_utf8().map(|c| c.into_owned());
    let (tool, revision, path) = match (decode(tool), decode(revision), decode(path)) {
        (Ok(t), Ok(r), Ok(p)) => (t, r, p),
        _ => return Err(not_found("geçersiz adres")),
    };
    let content_type = image_content_type(&path).ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            "yalnızca görseller sunulur".to_string(),
        )
    })?;

    let state = app.state::<AppState>();
    let manifest = state
        .catalog
        .read()
        .unwrap()
        .get(&tool)
        .map(|e| e.manifest.clone())
        .ok_or_else(|| not_found("araç katalogda yok"))?;

    // The revision in the URL must be the one the shell would show: a page
    // cannot ask for files of arbitrary commits or other folders.
    let bytes = match doc_source(&state, &manifest) {
        DocSource::Release {
            repo,
            revision: current,
        } if current == revision => state
            .docs
            .release_file(
                &state.client,
                &tool,
                &repo,
                &revision,
                &path,
                docs::MAX_IMAGE,
            )
            .await
            .map(|f| f.bytes)
            .map_err(|e| not_found(&e.to_string()))?,
        DocSource::Local { folder } if revision == LOCAL_REVISION => {
            tokio::task::spawn_blocking(move || docs::local_file(&folder, &path, docs::MAX_IMAGE))
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .map_err(|e| not_found(&e.to_string()))?
        }
        _ => return Err(not_found("bu revizyon gösterilmiyor")),
    };
    Ok((bytes, content_type))
}
