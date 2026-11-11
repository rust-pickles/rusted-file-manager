//! Tauri commands. Each one is a thin, typed adapter over a backend crate.
//!
//! Step 1 (this commit) exposes exactly one command: `app_info`, the handshake
//! the frontend uses to prove the IPC bridge is alive. Filesystem commands are
//! added deliberately in a later step so the shell can be verified building and
//! loading the real UI before any data path exists.

use crate::dto::{AppInfo, AssetRoot, ThemeId};

/// Returns build metadata and the theme list the frontend needs to render.
///
/// The asset roots reported here are the same directories the asset protocol
/// scope allows, so the UI can tell the user which folders may be previewed
/// without guessing.
#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "RUSTED".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        platform: std::env::consts::OS.into(),
        themes: ThemeId::ALL.to_vec(),
        asset_roots: default_asset_roots(),
    }
}

/// Well-known user folders that the asset protocol may serve from.
///
/// These are resolved at runtime rather than hard-coded so a profile on a
/// different volume keeps working. Entries whose folder does not exist are
/// omitted instead of being advertised and then failing to load.
fn default_asset_roots() -> Vec<AssetRoot> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(std::path::PathBuf::from);
    let Some(home) = home else {
        return Vec::new();
    };
    // `Documents`, `Pictures` and friends can be redirected onto another drive on
    // Windows, so only the home directory itself is assumed to exist.
    [(home.clone(), "Home"), (home.join("Pictures"), "Pictures"), (home.join("Downloads"), "Downloads")]
        .into_iter()
        .filter(|(path, _)| path.is_dir())
        .map(|(path, label)| AssetRoot { path: path.to_string_lossy().into_owned(), label: label.into() })
        .collect()
}
