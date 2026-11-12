//! Tauri 2 shell for RUSTED.
//!
//! Responsibilities, in order of importance:
//! 1. Own the window and the IPC surface.
//! 2. Translate between backend types and the frontend contract (`dto`).
//! 3. Translate between the webview's virtual paths and real ones (`paths`).
//! 4. Keep the asset protocol scope narrow and enforced in Rust.
//!
//! It implements no filesystem logic of its own: every command resolves a path
//! and delegates to a backend crate, so the same code stays usable without a
//! webview.

#[cfg(windows)]
pub mod clipboard;
pub mod commands;
pub mod dto;
pub mod paths;

use commands::AppState;
use tauri::Manager;

/// Builds and runs the desktop application.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::frontend_ready,
            commands::home_location,
            commands::list_directory,
            commands::get_item,
            commands::create_folder,
            commands::create_file,
            commands::rename_item,
            commands::delete_item,
            commands::copy_items,
            commands::move_items,
            commands::duplicate_item,
            commands::toggle_starred,
            commands::search,
            commands::get_storage_overview,
            commands::get_recent_files,
            commands::get_starred_items,
            commands::note_opened,
            commands::resolve_dropped_paths,
            commands::open_path,
            commands::reveal_path,
            commands::copy_paths_to_clipboard,
            commands::read_clipboard_paths,
            commands::quick_access_entries,
        ])
        .setup(|app| {
            // Persistence lives next to the OS's per-user app data. If the
            // database cannot be opened the app still starts: a broken star list
            // must not prevent browsing files, so an in-memory database is used
            // and the failure is reported rather than swallowed silently.
            let db = match rusted_db::Database::open_default() {
                Ok(db) => db,
                Err(error) => {
                    eprintln!("rusted: falling back to an in-memory database: {error}");
                    rusted_db::Database::in_memory().expect("in-memory database")
                }
            };
            // rusqlite::Connection is Send but not Sync, so the handle is
            // mutex-wrapped inside AppState; see commands::AppState.
            app.manage(AppState::new(db));

            // The window is created hidden so the webview never paints an empty
            // white frame before React mounts. The frontend calls
            // `frontend_ready`; this timeout is the failsafe so a JavaScript
            // error cannot leave an invisible process running.
            if let Some(window) = app.get_webview_window("main") {
                let window = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(2500));
                    let _ = window.show();
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to start the RUSTED desktop shell");
}
