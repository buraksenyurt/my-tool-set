//! The Tauri shell: wires commands, state and plugins together.
//! The domain model lives in mts-core, disk and network access in mts-catalog.

mod commands;
mod dto;
mod error;
mod state;

use tauri::Manager;

use state::{AppState, Paths};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let path = app.path();
            let paths = Paths::new(&path.app_local_data_dir()?, &path.app_config_dir()?);
            let user_agent = format!(
                "my-tool-set/{} (+https://github.com/buraksenyurt/my-tool-set)",
                app.package_info().version
            );
            // Debug builds can point at a local server with GitHub's URL layout,
            // to try installs without publishing a release.
            let base = std::env::var("MTS_RELEASE_BASE")
                .ok()
                .filter(|_| cfg!(debug_assertions));
            app.manage(AppState::new(paths, &user_agent, base.as_deref())?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_tools,
            commands::reload_catalog,
            commands::check_updates,
            commands::install_tool,
            commands::uninstall_tool,
        ])
        .run(tauri::generate_context!())
        .expect("error while running my-tool-set");
}
