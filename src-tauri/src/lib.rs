//! The Tauri shell: wires commands, state and plugins together.
//! The domain model lives in mts-core.

mod commands;
mod dto;
mod error;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::app_info])
        .run(tauri::generate_context!())
        .expect("error while running my-tool-set");
}
