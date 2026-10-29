//! The Tauri shell: wires commands, state and plugins together.
//! The domain model lives in mts-core, disk and network access in mts-catalog.

mod commands;
mod docs;
mod dto;
mod error;
mod runs;
mod state;

use tauri::Manager;

use state::{AppState, Paths};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .register_asynchronous_uri_scheme_protocol(docs::SCHEME, |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                responder.respond(docs::respond(app, request).await);
            });
        })
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
            let state = AppState::new(paths, &user_agent, base.as_deref())?;
            state.set_runner(mts_runner::Runner::new(runs::TauriSink(
                app.handle().clone(),
            )));
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_tools,
            commands::reload_catalog,
            commands::check_updates,
            commands::install_tool,
            commands::uninstall_tool,
            docs::get_readme,
            runs::start_tool,
            runs::build_tool,
            runs::stop_run,
            runs::list_runs,
            runs::run_log,
        ])
        .build(tauri::generate_context!())
        .expect("error while building my-tool-set")
        .run(|app, event| {
            // Builds are the shell's business and end with it; tools the user
            // opened keep running.
            if let tauri::RunEvent::Exit = event {
                app.state::<AppState>().runner().stop_builds();
            }
        });
}
