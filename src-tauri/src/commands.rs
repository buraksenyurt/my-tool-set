//! Commands the front end can `invoke`. Each one is a thin adapter: it
//! gathers what it needs from Tauri and returns a DTO.

use mts_core::Platform;
use tauri::{AppHandle, Manager};

use crate::dto::AppInfoDto;
use crate::error::{AppError, ErrorKind};

/// Version, platform and data folder, for the status bar and the about tab.
#[tauri::command]
pub fn app_info(app: AppHandle) -> Result<AppInfoDto, AppError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::new(ErrorKind::Internal, format!("Veri klasörü bulunamadı: {e}")))?;

    Ok(AppInfoDto {
        name: app.package_info().name.clone(),
        version: app.package_info().version.to_string(),
        platform: Platform::current(),
        data_dir: data_dir.display().to_string(),
    })
}
