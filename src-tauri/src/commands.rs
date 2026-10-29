//! Commands the front end can `invoke`. Each one is a thin adapter between
//! Tauri and mts-catalog that returns DTOs.

use futures_util::future::join_all;
use mts_catalog::catalog::Origin;
use mts_catalog::install::{Installed, Step};
use mts_catalog::settings::{LocalStatus, local_status};
use mts_core::version::is_newer;
use mts_core::{Delivery, Platform};
use tauri::ipc::Channel;
use tauri::{AppHandle, State};

use crate::docs::{doc_source, screenshot_urls};
use crate::dto::*;
use crate::error::{AppError, ErrorKind};
use crate::state::AppState;

/// Version, platform and folders, for the status bar and the about tab.
#[tauri::command]
pub fn app_info(app: AppHandle, state: State<'_, AppState>) -> AppInfoDto {
    let config_dir = state
        .paths
        .settings
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let data_dir = state
        .paths
        .tools
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    AppInfoDto {
        name: app.package_info().name.clone(),
        version: app.package_info().version.to_string(),
        platform: Platform::current(),
        data_dir,
        config_dir,
    }
}

#[tauri::command]
pub fn list_tools(state: State<'_, AppState>) -> CatalogDto {
    catalog_dto(&state)
}

/// Re-reads settings.toml and the catalog folders.
#[tauri::command]
pub fn reload_catalog(state: State<'_, AppState>) -> CatalogDto {
    state.reload();
    catalog_dto(&state)
}

/// Fetches the latest release.json of every release tool, in parallel. One
/// tool failing (no release yet, offline) does not stop the others.
#[tauri::command]
pub async fn check_updates(state: State<'_, AppState>) -> Result<Vec<UpdateCheckDto>, AppError> {
    let targets: Vec<(String, String)> = state
        .catalog
        .read()
        .unwrap()
        .entries()
        .iter()
        .filter_map(|e| {
            let repo = e.manifest.release.as_ref()?.repo.clone();
            Some((e.manifest.id.clone(), repo))
        })
        .collect();

    let checks = targets.into_iter().map(|(id, repo)| {
        let state = &state;
        async move {
            let result = async {
                let release = state.client.latest(&repo).await?;
                let problems = release.validate(&id, &repo);
                if !problems.is_empty() {
                    return Err(mts_catalog::CatalogError::InvalidRelease(
                        problems.join("; "),
                    ));
                }
                state.cache.put(&id, &release)?;
                Ok(release.version)
            }
            .await;
            match result {
                Ok(version) => UpdateCheckDto {
                    id,
                    version: Some(version),
                    error: None,
                },
                Err(e) => UpdateCheckDto {
                    id,
                    version: None,
                    error: Some(e.to_string()),
                },
            }
        }
    });
    Ok(join_all(checks).await)
}

/// Installs the latest known release of `id`. Uses the cached release.json
/// when there is one, so the version installed is the one the UI showed.
#[tauri::command]
pub async fn install_tool(
    id: String,
    on_progress: Channel<InstallProgressDto>,
    state: State<'_, AppState>,
) -> Result<InstalledDto, AppError> {
    let platform = Platform::current().ok_or_else(|| {
        AppError::new(
            ErrorKind::NoBuildForPlatform,
            "Bu platform için araç yayınlanmıyor",
        )
    })?;
    let repo = release_repo(&state, &id)?;

    if !state.installing.lock().unwrap().insert(id.clone()) {
        return Err(AppError::new(
            ErrorKind::Busy,
            format!("{id} zaten kuruluyor"),
        ));
    }
    let result = async {
        let release = match state.cache.get(&id) {
            Some(cached) => cached.release,
            None => {
                let release = state.client.latest(&repo).await?;
                state.cache.put(&id, &release)?;
                release
            }
        };
        if release.tool != id || !release.repo.eq_ignore_ascii_case(&repo) {
            return Err(AppError::new(
                ErrorKind::Integrity,
                format!("Önbellekteki release.json {id} aracına ait değil"),
            ));
        }
        let installed = state
            .installer
            .install(&state.client, &release, platform, &|step| {
                let _ = on_progress.send(match step {
                    Step::Downloading { received, total } => {
                        InstallProgressDto::Downloading { received, total }
                    }
                    Step::Verifying => InstallProgressDto::Verifying,
                    Step::Unpacking => InstallProgressDto::Unpacking,
                });
            })
            .await?;
        Ok(installed_dto(&installed))
    }
    .await;
    state.installing.lock().unwrap().remove(&id);
    result
}

#[tauri::command]
pub async fn uninstall_tool(
    id: String,
    version: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    release_repo(&state, &id)?;
    if state.installing.lock().unwrap().contains(&id) {
        return Err(AppError::new(
            ErrorKind::Busy,
            format!("{id} şu an kuruluyor"),
        ));
    }
    let installer = state.installer.clone();
    tokio::task::spawn_blocking(move || installer.uninstall(&id, &version))
        .await
        .map_err(|e| AppError::new(ErrorKind::Internal, e.to_string()))?
        .map_err(AppError::from)
}

fn release_repo(state: &AppState, id: &str) -> Result<String, AppError> {
    let catalog = state.catalog.read().unwrap();
    let entry = catalog
        .get(id)
        .ok_or_else(|| AppError::new(ErrorKind::UnknownTool, format!("Katalogda \"{id}\" yok")))?;
    entry
        .manifest
        .release
        .as_ref()
        .map(|r| r.repo.clone())
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::NotApplicable,
                format!("{id} yerel bir araç; release'ten kurulmaz"),
            )
        })
}

fn catalog_dto(state: &AppState) -> CatalogDto {
    let catalog = state.catalog.read().unwrap();
    let settings = state.settings.read().unwrap();
    let platform = Platform::current();

    let tools = catalog
        .entries()
        .iter()
        .map(|entry| {
            let m = &entry.manifest;
            let delivery = match m.delivery() {
                Delivery::Release => {
                    let repo = m
                        .release
                        .as_ref()
                        .map(|r| r.repo.clone())
                        .unwrap_or_default();
                    let installed: Vec<_> = state
                        .installer
                        .installed(&m.id)
                        .iter()
                        .map(installed_dto)
                        .collect();
                    let latest = state.cache.get(&m.id).map(|c| {
                        let build = platform.and_then(|p| c.release.for_platform(p));
                        LatestDto {
                            version: c.release.version.clone(),
                            tag: c.release.tag.clone(),
                            fetched_at: c.fetched_at,
                            has_build: build.is_some(),
                            asset_size: build.map(|b| b.portable.size),
                        }
                    });
                    let update_available = match (&latest, installed.first()) {
                        (Some(l), Some(i)) => l.has_build && is_newer(&l.version, &i.version),
                        _ => false,
                    };
                    DeliveryDto::Release {
                        repo,
                        installed,
                        latest,
                        update_available,
                    }
                }
                Delivery::Local => {
                    let status = match platform {
                        Some(p) => local_status_dto(local_status(m, &settings, p.os)),
                        None => LocalStatusDto::NoExeForOs,
                    };
                    DeliveryDto::Local {
                        status,
                        build_command: m.build.as_ref().map(|b| b.command.clone()),
                        dev_command: m.run.dev.clone(),
                    }
                }
            };
            let docs = doc_source(state, m);
            ToolDto {
                screenshots: screenshot_urls(m, &docs),
                doc_revision: docs.revision().map(str::to_string),
                web_url: m
                    .release
                    .as_ref()
                    .map(|r| format!("https://github.com/{}", r.repo)),
                id: m.id.clone(),
                name: m.name.clone(),
                summary: m.summary.clone(),
                category: m.category.clone(),
                kind: m.kind,
                stack: m.stack.clone(),
                tags: m.tags.clone(),
                host: m.run.host,
                origin: match entry.origin {
                    Origin::Embedded => OriginDto::Embedded,
                    Origin::Repository => OriginDto::Repository,
                    Origin::User => OriginDto::User,
                },
                support: platform
                    .map(|p| m.platforms.on(p.os))
                    .unwrap_or(mts_core::manifest::Support::No),
                params: m.run.params.clone(),
                delivery,
            }
        })
        .collect();

    CatalogDto {
        tools,
        issues: catalog
            .issues()
            .iter()
            .map(|i| IssueDto {
                location: i.location.clone(),
                message: i.message.clone(),
            })
            .collect(),
        settings_file: state.paths.settings.display().to_string(),
        user_catalog_dir: state.paths.user_catalog.display().to_string(),
    }
}

fn installed_dto(i: &Installed) -> InstalledDto {
    InstalledDto {
        version: i.record.version.clone(),
        folder: i.folder.display().to_string(),
        entry: i.entry.display().to_string(),
        installed_at: i.record.installed_at,
    }
}

fn local_status_dto(s: LocalStatus) -> LocalStatusDto {
    let d = |p: std::path::PathBuf| p.display().to_string();
    match s {
        LocalStatus::Ready { folder, exe } => LocalStatusDto::Ready {
            folder: d(folder),
            exe: d(exe),
        },
        LocalStatus::NotBuilt { folder } => LocalStatusDto::NotBuilt { folder: d(folder) },
        LocalStatus::FromSource { folder } => LocalStatusDto::FromSource { folder: d(folder) },
        LocalStatus::UnknownRoot(root) => LocalStatusDto::UnknownRoot { root },
        LocalStatus::MissingFolder(folder) => LocalStatusDto::MissingFolder { folder: d(folder) },
        LocalStatus::NoExeForOs => LocalStatusDto::NoExeForOs,
    }
}
