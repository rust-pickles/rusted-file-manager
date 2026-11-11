//! Tauri 2 shell for RUSTED.
//!
//! Responsibilities, in order of importance:
//! 1. Own the window and the IPC surface.
//! 2. Translate between backend types and the frontend contract (`dto`).
//! 3. Keep the asset protocol scope narrow and enforced in Rust.
//!
//! It intentionally implements almost no behaviour. Filesystem commands land in
//! `commands` one at a time, each delegating straight to a backend crate.

pub mod commands;
pub mod dto;

use tauri::Manager;

/// Builds and runs the desktop application.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::app_info])
        .setup(|app| {
            // The window is created hidden (`visible: false`) so the webview never
            // paints an empty white frame before React mounts; it is shown once the
            // frontend signals readiness, with a timeout as a failsafe.
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                std::thread::spawn(move || {
                    // Failsafe: show the window even if the frontend never calls in,
                    // so a JavaScript error cannot leave an invisible process.
                    std::thread::sleep(std::time::Duration::from_millis(2500));
                    let _ = window_clone.show();
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to start the RUSTED desktop shell");
}
