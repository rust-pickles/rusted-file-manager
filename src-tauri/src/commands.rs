//! Tauri commands. Each one is a thin, typed adapter over a backend crate.
//!
//! Contract with the frontend: every function here has a camelCase twin in
//! `ui/services/FileSystemService.ts`, and every payload is a type from `dto`.
//! No command implements filesystem logic itself; it resolves a path, calls the
//! backend crate, and maps the result.

use crate::dto::{
    AppInfo, AssetRoot, DirectoryListing, DriveDto, DriveHealthDto, FileItem, FileType,
    FolderUsageRow, ThemeId,
};
use crate::paths::{self, Location};
use tauri::State;

/// Shared application state.
///
/// The database handle is behind a `Mutex` because `rusqlite::Connection` is
/// `Send` but not `Sync`: Tauri requires managed state to be both, so without
/// this the commands do not satisfy `CommandArg` at all. A poisoned lock is
/// recovered rather than panicking, because losing a star update must not take
/// the whole window down.
pub struct AppState {
    db: std::sync::Mutex<rusted_db::Database>,
}

impl AppState {
    pub fn new(db: rusted_db::Database) -> Self {
        Self { db: std::sync::Mutex::new(db) }
    }

    /// Locks the database, recovering from a poisoned mutex.
    fn db(&self) -> std::sync::MutexGuard<'_, rusted_db::Database> {
        self.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// An error returned to the webview. Kept as a plain string because the UI shows
/// `error.message` verbatim in its toasts.
#[derive(Debug, serde::Serialize)]
pub struct CommandError {
    pub message: String,
}

impl<E: std::fmt::Display> From<E> for CommandError {
    fn from(error: E) -> Self {
        Self { message: error.to_string() }
    }
}

type Result<T> = std::result::Result<T, CommandError>;

/// Opens a file with its default application, the way Explorer does.
///
/// `Command::new(path)` is deliberately avoided: it would try to *execute* the
/// file, so a `.pdf` or `.jpg` would fail and a `.exe` would run without the
/// shell's normal handling. `cmd /C start` hands the path to the shell, which
/// honours file associations, `.lnk` targets and elevation prompts.
#[tauri::command]
pub fn open_path(path: String) -> Result<()> {
    let real = paths::to_real(&path)
        .ok_or_else(|| CommandError { message: format!("'{path}' is not a location") })?;
    if !real.exists() {
        return Err(CommandError { message: format!("'{}' no longer exists", real.display()) });
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        // The empty title argument is required: `start` treats the first quoted
        // argument as the window title, so a path with spaces would be eaten.
        let target = real.to_string_lossy().into_owned();
        std::process::Command::new("cmd")
            .args(["/C", "start", "", target.as_str()])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|error| CommandError { message: format!("could not open '{}': {error}", real.display()) })?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
        std::process::Command::new(opener)
            .arg(&real)
            .spawn()
            .map_err(|error| CommandError { message: format!("could not open '{}': {error}", real.display()) })?;
        Ok(())
    }
}

/// Opens the folder and selects the item in the system file browser.
#[tauri::command]
pub fn reveal_path(path: String) -> Result<()> {
    let real = paths::to_real(&path)
        .ok_or_else(|| CommandError { message: format!("'{path}' is not a location") })?;
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        // /select, needs the comma glued to it and the path passed separately.
        // explorer.exe parses its own command line, so the path is quoted by
        // hand: Command's escaping would mangle a path ending in a separator.
        let selection = format!("/select,\"{}\"", real.display());
        std::process::Command::new("explorer")
            .raw_arg(selection)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|error| CommandError { message: format!("could not reveal '{}': {error}", real.display()) })?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let folder = if real.is_dir() { real.clone() } else { real.parent().map(|p| p.to_path_buf()).unwrap_or(real.clone()) };
        let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
        std::process::Command::new(opener)
            .arg(folder)
            .spawn()
            .map_err(|error| CommandError { message: error.to_string() })?;
        Ok(())
    }
}

/// Puts native file paths on the system clipboard as "files".
///
/// Text-only clipboard support cannot express this: pasting into Explorer needs
/// the CF_HDROP format, which holds a list of paths. That is why a copied file
/// pasted here could never be pasted into Explorer before.
#[tauri::command]
pub fn copy_paths_to_clipboard(paths_to_copy: Vec<String>) -> Result<()> {
    #[cfg(windows)]
    {
        let mut files = Vec::new();
        for path in &paths_to_copy {
            if let Some(real) = paths::to_real(path) {
                if real.exists() {
                    files.push(real);
                }
            }
        }
        if files.is_empty() {
            return Err(CommandError { message: "nothing to copy".into() });
        }
        crate::clipboard::set_files(&files)
            .map_err(|error| CommandError { message: error })?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = paths_to_copy;
        Err(CommandError { message: "copying files to the clipboard is Windows-only".into() })
    }
}

/// Reads back the file list on the clipboard, so the app can tell the user what
/// a paste would bring in even when the copy happened in Explorer.
#[tauri::command]
pub fn read_clipboard_paths() -> Vec<String> {
    #[cfg(windows)]
    {
        return crate::clipboard::get_files()
            .unwrap_or_default()
            .into_iter()
            .map(|path| paths::to_virtual(&path))
            .collect();
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

/// Which `/Home` entries actually exist for this user, as virtual paths.
///
/// The sidebar used to render all seven unconditionally, so on a machine where
/// OneDrive owns Pictures or Videos the entry pointed at a stub and opened the
/// wrong folder. Reporting the resolved set lets the UI show only real entries,
/// and because resolution goes through the shell the path is the redirected one.
#[tauri::command]
pub fn quick_access_entries() -> Vec<paths::QuickAccess> {
    paths::quick_access()
}

/// Drives with capacity only, for the first paint.
///
/// Split from `get_storage_overview` on purpose: this reads cached volume
/// metadata and returns in microseconds, while the overview scans folders. The
/// sidebar and the Storage view both call this first so space appears at once,
/// instead of after a scan.
#[tauri::command]
pub fn list_drives() -> Vec<DriveDto> {
    rusted_fs::drives()
        .into_iter()
        .map(|mut drive| {
            // Capacity is already part of DriveInfo; nothing else is needed here,
            // but the field is re-read through the fast probe so a stale cached
            // value in DriveInfo cannot disagree with the badge.
            if let Some((total, free)) = rusted_fs::drive_capacity(&drive.root) {
                drive.total = Some(total);
                drive.free = Some(free);
            }
            DriveDto::from(drive)
        })
        .collect()
}

/// Per-drive filesystem health, including a real write probe.
///
/// The Storage view calls this to show warnings. A drive that reports free space
/// but rejects writes (read-only mount, failing disk, full allocation) is the
/// failure this catches, and it is invisible to a capacity check alone.
#[tauri::command]
pub async fn drive_health_report() -> Result<Vec<DriveHealthDto>> {
    tauri::async_runtime::spawn_blocking(|| {
        rusted_fs::drives()
            .into_iter()
            .map(|drive| {
                let health = rusted_fs::drive_health(&drive.root);
                DriveHealthDto {
                    name: drive.display_name(),
                    root: paths::to_virtual(&drive.root),
                    total: health.total,
                    free: health.free,
                    writable: health.writable,
                    problem: health.problem,
                }
            })
            .collect::<Vec<_>>()
    })
    .await
    .map_err(|error| CommandError { message: format!("drive health check failed: {error}") })
}

/// Bytes and file counts per top-level folder under `root`.
///
/// Deliberately one level deep plus a bounded walk inside each child, which is
/// what makes it fast: a full recursive scan of a 500 GB drive takes minutes,
/// and the Storage view only needs to answer "which folders are big". Anything
/// past `MAX_FILES_PER_FOLDER` stops counting, and the row is marked so the UI
/// can say the number is a lower bound instead of pretending it is exact.
#[tauri::command]
pub async fn folder_usage(root: String) -> Result<Vec<FolderUsageRow>> {
    tauri::async_runtime::spawn_blocking(move || {
        let real = paths::to_real(&root)
            .ok_or_else(|| CommandError { message: format!("'{root}' is not a location") })?;
        if !real.is_dir() {
            return Err(CommandError { message: format!("'{}' is not a folder", real.display()) });
        }

        const MAX_FILES_PER_FOLDER: usize = 200_000;
        let mut rows: Vec<FolderUsageRow> = Vec::new();

        let entries = std::fs::read_dir(&real)
            .map_err(|error| CommandError { message: format!("cannot read '{}': {error}", real.display()) })?;

        for entry in entries.flatten() {
            let path = entry.path();
            let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
            if !is_dir {
                // Loose files at the root are grouped so the totals still add up.
                if let Ok(meta) = entry.metadata() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    rows.push(FolderUsageRow {
                        name,
                        path: paths::to_virtual(&path),
                        bytes: meta.len(),
                        file_count: 1,
                        truncated: false,
                    });
                }
                continue;
            }

            let mut bytes: u64 = 0;
            let mut file_count: u64 = 0;
            let mut truncated = false;
            // A bounded, iterative walk: no recursion depth surprises, and it can
            // stop early. Links are not followed, so a junction cannot loop.
            let mut stack = vec![path.clone()];
            while let Some(current) = stack.pop() {
                let Ok(listing) = std::fs::read_dir(&current) else { continue };
                for child in listing.flatten() {
                    let Ok(kind) = child.file_type() else { continue };
                    if kind.is_dir() {
                        stack.push(child.path());
                        continue;
                    }
                    if let Ok(meta) = child.metadata() {
                        bytes += meta.len();
                        file_count += 1;
                        if file_count as usize >= MAX_FILES_PER_FOLDER {
                            truncated = true;
                            break;
                        }
                    }
                }
                if truncated {
                    break;
                }
            }

            let name = entry.file_name().to_string_lossy().into_owned();
            rows.push(FolderUsageRow {
                path: paths::to_virtual(&path),
                name,
                bytes,
                file_count,
                truncated,
            });
        }

        // Largest first: the question this answers is "what is eating the drive".
        rows.sort_by(|a, b| b.bytes.cmp(&a.bytes));
        rows.truncate(40);
        Ok(rows)
    })
    .await
    .map_err(|error| CommandError { message: format!("folder scan failed: {error}") })?
}

/// Returns build metadata and the theme list the frontend needs to render.
#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "RUSTED".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        platform: std::env::consts::OS.into(),
        themes: ThemeId::ALL.to_vec(),
        asset_roots: home_asset_roots(),
    }
}

/// Shows the window once the frontend has mounted.
///
/// The window starts hidden so React's first paint is what the user sees, and
/// the main module shows it after a timeout in case this is never called.
#[tauri::command]
pub fn frontend_ready(window: tauri::Window) {
    let _ = window.show();
}

/// Where the current user's Home points, so the UI can label it truthfully.
#[tauri::command]
pub fn home_location() -> Result<Location> {
    let real = paths::home_directory().ok_or_else(|| CommandError { message: "no profile folder".into() })?;
    Ok(Location {
        virtual_path: paths::VIRTUAL_HOME.into(),
        real_path: Some(real.to_string_lossy().into_owned()),
    })
}

/// Lists a folder. `path` may be a virtual path (`/Home/Desktop`) or a native
/// one (`C:\Users\dev`); the reply always carries the virtual spelling.
#[tauri::command]
pub fn list_directory(state: State<'_, AppState>, path: String, show_hidden: bool) -> Result<DirectoryListing> {
    let trimmed = path.trim();

    // "/" is not a folder: it lists the mounted drives, which is what the
    // sidebar's "This PC" entry expects.
    if trimmed.is_empty() || trimmed == paths::VIRTUAL_ROOT {
        let drives = paths::drive_locations();
        let entries = drives
            .into_iter()
            .map(|location| virtual_root_item(&location))
            .collect();
        return Ok(DirectoryListing {
            path: paths::VIRTUAL_ROOT.into(),
            real_path: None,
            entries,
            is_virtual_root: true,
        });
    }

    let real = paths::to_real(trimmed).ok_or_else(|| CommandError {
        message: format!("'{trimmed}' is not a location this app can open"),
    })?;
    if !real.is_dir() {
        return Err(CommandError { message: format!("'{}' is not a folder", real.display()) });
    }

    let virtual_path = paths::to_virtual(&real);
    let _ = state.db().visit(&real);
    let starred = state.db().starred_paths().unwrap_or_default();

    let entries = rusted_fs::read_directory(&real, show_hidden, Default::default())
        .map_err(|error| CommandError { message: error.to_string() })?
        .into_iter()
        .map(|entry| {
            let is_starred = starred.iter().any(|star| star == &entry.path);
            let name = entry.name.clone();
            let item: FileItem = entry.into();
            let child_virtual = paths::child_of(&virtual_path, &name);
            item.with_state(is_starred, child_virtual, virtual_path.clone())
        })
        .collect();

    Ok(DirectoryListing {
        path: virtual_path,
        real_path: Some(real.to_string_lossy().into_owned()),
        entries,
        is_virtual_root: false,
    })
}

/// A drive rendered as a folder row for the virtual root.
fn virtual_root_item(location: &Location) -> FileItem {
    let real = location.real_path.clone().unwrap_or_default();
    FileItem {
        id: location.virtual_path.clone(),
        name: real.clone(),
        path: location.virtual_path.clone(),
        parent_path: paths::VIRTUAL_ROOT.into(),
        file_type: FileType::Folder,
        extension: None,
        size: 0,
        created_at: String::new(),
        modified_at: String::new(),
        mime_type: None,
        is_starred: Some(false),
        is_hidden: Some(false),
        is_system: None,
        preview_kind: None,
        preview_url: None,
        code_language: None,
        text_content: None,
        dimensions: None,
        duration: None,
        item_count: None,
        children_ids: None,
    }
}

/// Full details for one path, including a preview URL for images and text.
#[tauri::command]
pub fn get_item(state: State<'_, AppState>, path: String) -> Result<Option<FileItem>> {
    let real = paths::to_real(&path).ok_or_else(|| CommandError { message: format!("'{path}' is not a location") })?;
    if !real.exists() {
        return Ok(None);
    }
    let metadata = std::fs::symlink_metadata(rusted_fs::long_path(&real))
        .map_err(|error| CommandError { message: error.to_string() })?;
    let file_type = metadata.file_type();
    let entry = rusted_fs::entry_from_metadata(&real, &metadata, file_type);
    let virtual_path = paths::to_virtual(&real);
    let parent = paths::to_virtual(real.parent().unwrap_or(&real));
    let starred = state.db().is_starred(&real).unwrap_or(false);
    let item: FileItem = entry.into();
    Ok(Some(item.with_state(starred, virtual_path, parent)))
}

#[tauri::command]
pub fn create_folder(state: State<'_, AppState>, parent_path: String, name: String) -> Result<FileItem> {
    let parent = existing_folder(&parent_path)?;
    let name = paths::validate_new_name(&name).map_err(|message| CommandError { message })?;
    let target = parent.join(name);
    if target.exists() {
        return Err(CommandError { message: format!("'{name}' already exists here") });
    }
    std::fs::create_dir(rusted_fs::long_path(&target))
        .map_err(|error| CommandError { message: format!("could not create the folder: {error}") })?;
    item_for(&state, &target)
}

#[tauri::command]
pub fn create_file(
    state: State<'_, AppState>,
    parent_path: String,
    name: String,
    content: Option<String>,
) -> Result<FileItem> {
    let parent = existing_folder(&parent_path)?;
    let name = paths::validate_new_name(&name).map_err(|message| CommandError { message })?;
    let target = parent.join(name);
    if target.exists() {
        return Err(CommandError { message: format!("'{name}' already exists here") });
    }
    // create_new refuses to clobber a file that appeared between the check and
    // the write, which a plain write would silently truncate.
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(rusted_fs::long_path(&target))
        .map_err(|error| CommandError { message: format!("could not create the file: {error}") })?;
    if let Some(content) = content {
        use std::io::Write;
        file.write_all(content.as_bytes())
            .map_err(|error| CommandError { message: error.to_string() })?;
    }
    item_for(&state, &target)
}

#[tauri::command]
pub fn rename_item(state: State<'_, AppState>, path: String, new_name: String) -> Result<FileItem> {
    let source = paths::to_real(&path).ok_or_else(|| CommandError { message: format!("'{path}' is not a location") })?;
    if !source.exists() {
        return Err(CommandError { message: "that item no longer exists".into() });
    }
    let new_name = paths::validate_new_name(&new_name).map_err(|message| CommandError { message })?;
    let parent = source
        .parent()
        .ok_or_else(|| CommandError { message: "cannot rename a drive root".into() })?
        .to_path_buf();
    let target = parent.join(new_name);
    if target != source && target.exists() {
        return Err(CommandError { message: format!("'{new_name}' already exists here") });
    }
    std::fs::rename(rusted_fs::long_path(&source), rusted_fs::long_path(&target))
        .map_err(|error| CommandError { message: format!("could not rename: {error}") })?;
    // The star follows the file, not its old name. The flag is read into a local
    // first: `std::sync::Mutex` is not reentrant, so locking twice with a guard
    // still alive would deadlock the command thread.
    let was_starred = state.db().is_starred(&source).unwrap_or(false);
    if was_starred {
        let _ = state.db().set_star_target(&source, &target);
    }
    item_for(&state, &target)
}

#[tauri::command]
pub fn delete_item(state: State<'_, AppState>, path: String, permanent: Option<bool>) -> Result<()> {
    let target = paths::to_real(&path).ok_or_else(|| CommandError { message: format!("'{path}' is not a location") })?;
    if !target.exists() {
        return Err(CommandError { message: "that item no longer exists".into() });
    }
    if permanent.unwrap_or(false) {
        // Deleting for good is destructive, so it is confirmed explicitly here
        // rather than relying on a flag the UI could forget to send.
        rusted_operations::delete_permanent(&target, true)
            .map_err(|error| CommandError { message: error.to_string() })?;
    } else {
        rusted_operations::recycle(&[&target])
            .map_err(|error| CommandError { message: format!("could not move to the Recycle Bin: {error}") })?;
    }
    // A deleted item must not keep a star or a recent entry pointing at nothing.
    let _ = state.db().forget_star(&target);
    let _ = state.db().forget_recent(&target);
    Ok(())
}

#[tauri::command]
pub fn copy_items(state: State<'_, AppState>, sources: Vec<String>, dest_folder: String) -> Result<Vec<FileItem>> {
    transfer(&state, &sources, &dest_folder, false)
}

#[tauri::command]
pub fn move_items(state: State<'_, AppState>, sources: Vec<String>, dest_folder: String) -> Result<Vec<FileItem>> {
    transfer(&state, &sources, &dest_folder, true)
}

#[tauri::command]
pub fn duplicate_item(state: State<'_, AppState>, path: String) -> Result<FileItem> {
    let source = paths::to_real(&path).ok_or_else(|| CommandError { message: format!("'{path}' is not a location") })?;
    if !source.exists() {
        return Err(CommandError { message: "that item no longer exists".into() });
    }
    if source.parent().is_none() {
        return Err(CommandError { message: "cannot duplicate a drive root".into() });
    }
    // Duplicating onto itself is the AutoRename case: pick "name (1).ext".
    let destination = rusted_operations::unique_destination(&source);
    let control = rusted_operations::OperationControl::default();
    rusted_operations::copy_path(&source, &destination, rusted_operations::ConflictPolicy::AutoRename, &control, &mut |_| {})
        .map_err(|error| CommandError { message: error.to_string() })?;
    item_for(&state, &destination)
}

#[tauri::command]
pub fn toggle_starred(state: State<'_, AppState>, path: String) -> Result<FileItem> {
    let real = paths::to_real(&path).ok_or_else(|| CommandError { message: format!("'{path}' is not a location") })?;
    state
        .db()
        .toggle_star(&real)
        .map_err(|error| CommandError { message: error.to_string() })?;
    item_for(&state, &real)
}

/// Searches below a folder by file name. Bounded so a search on a huge tree
/// cannot freeze the window.
#[tauri::command]
pub fn search(
    state: State<'_, AppState>,
    query: String,
    root: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<FileItem>> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Ok(Vec::new());
    }
    let start = match root.as_deref().map(paths::to_real) {
        Some(Some(real)) if real.is_dir() => real,
        // No usable root: fall back to the profile folder rather than scanning
        // an entire drive by accident.
        _ => paths::home_directory().ok_or_else(|| CommandError { message: "no folder to search".into() })?,
    };
    let budget = limit.unwrap_or(200).clamp(1, 2000);
    let starred = state.db().starred_paths().unwrap_or_default();
    let mut hits = Vec::new();
    let mut stack = vec![start.clone()];
    let mut visited = 0usize;

    while let Some(folder) = stack.pop() {
        // A hard ceiling on directories visited keeps a search responsive even
        // when the tree is far larger than the result budget.
        visited += 1;
        if visited > 5_000 || hits.len() >= budget {
            break;
        }
        let Ok(reader) = std::fs::read_dir(rusted_fs::long_path(&folder)) else {
            continue;
        };
        for item in reader.flatten() {
            let path = item.path();
            let Ok(metadata) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            // Junctions can form loops; never follow them during a search.
            if metadata.file_type().is_symlink() {
                continue;
            }
            let name = item.file_name().to_string_lossy().into_owned();
            let is_dir = metadata.is_dir();
            if name.to_lowercase().contains(&needle) {
                if let Ok(file_type) = item.file_type() {
                    let entry = rusted_fs::entry_from_metadata(&path, &metadata, file_type);
                    let virtual_path = paths::to_virtual(&path);
                    let parent = paths::to_virtual(path.parent().unwrap_or(&path));
                    let is_starred = starred.iter().any(|star| star == &path);
                    let hit: FileItem = entry.into();
                    hits.push(hit.with_state(is_starred, virtual_path, parent));
                    if hits.len() >= budget {
                        break;
                    }
                }
            }
            if is_dir {
                stack.push(path);
            }
        }
    }
    Ok(hits)
}

/// Drives plus a category breakdown for the Home and Storage views.
///
/// Async on purpose: measuring a profile folder walks thousands of files and
/// would freeze the window for seconds if it ran on the IPC thread. The blocking
/// work is moved to the thread pool and the UI keeps painting.
#[tauri::command]
pub async fn get_storage_overview() -> Result<StorageOverview> {
    tauri::async_runtime::spawn_blocking(measure_storage)
        .await
        .map_err(|error| CommandError { message: format!("storage scan failed: {error}") })?
}

fn measure_storage() -> Result<StorageOverview> {
    let drives: Vec<DriveDto> = rusted_fs::drives().into_iter().map(DriveDto::from).collect();
    let mut categories: Vec<CategoryRow> = vec![
        CategoryRow { category: "Media & Videos".into(), bytes: 0, color: "#38BDF8".into(), file_count: 0 },
        CategoryRow { category: "Archives & Binaries".into(), bytes: 0, color: "#818CF8".into(), file_count: 0 },
        CategoryRow { category: "Documents & PDFs".into(), bytes: 0, color: "#34D399".into(), file_count: 0 },
        CategoryRow { category: "Code & Repositories".into(), bytes: 0, color: "#F59E0B".into(), file_count: 0 },
        CategoryRow { category: "Other Assets".into(), bytes: 0, color: "#94A3B8".into(), file_count: 0 },
    ];

    // Measure the profile folder only. Scanning every drive would take minutes
    // on a real machine, and the Storage view is about the user's own data.
    if let Some(home) = paths::home_directory() {
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let options = rusted_storage::ScanOptions { max_depth: 4, largest_files: 25, follow_links: false };
        if let Ok(report) = rusted_storage::scan(&home, &options, &cancel) {
            for usage in &report.categories {
                let slot = match usage.category {
                    "Images" | "Media" => Some(0usize),
                    "Archives" | "Applications" => Some(1),
                    "Documents" => Some(2),
                    "Code" => Some(3),
                    "Other" => Some(4),
                    _ => None,
                };
                if let Some(index) = slot {
                    categories[index].bytes += usage.bytes;
                    categories[index].file_count += usage.count;
                }
            }
        }
    }
    Ok(StorageOverview { drives, categories })
}

/// Files the user opened most recently, newest first.
#[tauri::command]
pub fn get_recent_files(state: State<'_, AppState>, limit: Option<usize>) -> Result<Vec<FileItem>> {
    let paths = state.db().recent_files(limit.unwrap_or(24)).unwrap_or_default();
    Ok(items_for_paths(&state, &paths))
}

/// Everything the user has starred, newest first.
#[tauri::command]
pub fn get_starred_items(state: State<'_, AppState>) -> Result<Vec<FileItem>> {
    // Drop stars for files that vanished outside the app before reporting them.
    let _ = state.db().prune_missing_stars();
    let paths = state.db().starred_paths().unwrap_or_default();
    Ok(items_for_paths(&state, &paths))
}

/// Records that the user opened a file, so it appears under Recent.
#[tauri::command]
pub fn note_opened(state: State<'_, AppState>, path: String) -> Result<()> {
    if let Some(real) = paths::to_real(&path) {
        let _ = state.db().opened_file(&real);
    }
    Ok(())
}

/// Handles a drop from Windows Explorer: the payload is a list of native paths.
#[tauri::command]
pub fn resolve_dropped_paths(paths_in: Vec<String>) -> Vec<Location> {
    paths_in
        .into_iter()
        .filter(|candidate| std::path::Path::new(candidate).exists())
        .map(|candidate| {
            let real = std::path::PathBuf::from(&candidate);
            Location { virtual_path: paths::to_virtual(&real), real_path: Some(candidate) }
        })
        .collect()
}

// ---- helpers ------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRow {
    pub category: String,
    pub bytes: u64,
    pub color: String,
    pub file_count: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageOverview {
    pub drives: Vec<DriveDto>,
    pub categories: Vec<CategoryRow>,
}

fn existing_folder(path: &str) -> Result<std::path::PathBuf> {
    let real = paths::to_real(path).ok_or_else(|| CommandError { message: format!("'{path}' is not a location") })?;
    if !real.is_dir() {
        return Err(CommandError { message: "the destination is not a folder".into() });
    }
    Ok(real)
}

fn item_for(state: &AppState, real: &std::path::Path) -> Result<FileItem> {
    let metadata = std::fs::symlink_metadata(rusted_fs::long_path(real))
        .map_err(|error| CommandError { message: error.to_string() })?;
    let file_type = metadata.file_type();
    let entry = rusted_fs::entry_from_metadata(real, &metadata, file_type);
    let virtual_path = paths::to_virtual(real);
    let parent = paths::to_virtual(real.parent().unwrap_or(real));
    let starred = state.db().is_starred(real).unwrap_or(false);
    let item: FileItem = entry.into();
    Ok(item.with_state(starred, virtual_path, parent))
}

fn items_for_paths(state: &AppState, candidates: &[std::path::PathBuf]) -> Vec<FileItem> {
    candidates
        .iter()
        .filter(|path| path.exists())
        .filter_map(|path| item_for(state, path).ok())
        .collect()
}

fn transfer(state: &AppState, sources: &[String], dest_folder: &str, move_it: bool) -> Result<Vec<FileItem>> {
    let destination = existing_folder(dest_folder)?;
    let control = rusted_operations::OperationControl::default();
    let mut created = Vec::new();
    for source in sources {
        let real = paths::to_real(source).ok_or_else(|| CommandError { message: format!("'{source}' is not a location") })?;
        if !real.exists() {
            return Err(CommandError { message: format!("'{}' no longer exists", real.display()) });
        }
        let name = real
            .file_name()
            .ok_or_else(|| CommandError { message: "cannot copy a drive root".into() })?;
        let target = destination.join(name);
        let policy = rusted_operations::ConflictPolicy::AutoRename;
        let result = if move_it {
            rusted_operations::move_path(&real, &target, policy, &control, &mut |_| {})
        } else {
            rusted_operations::copy_path(&real, &target, policy, &control, &mut |_| {})
        };
        let placed = result.map_err(|error| CommandError { message: error.to_string() })?;
        if let Some(placed) = placed {
            let was_starred = move_it && state.db().is_starred(&real).unwrap_or(false);
            if was_starred {
                let _ = state.db().set_star_target(&real, &placed);
            }
            if let Ok(item) = item_for(state, &placed) {
                created.push(item);
            }
        }
    }
    Ok(created)
}

fn home_asset_roots() -> Vec<AssetRoot> {
    let Some(home) = paths::home_directory() else {
        return Vec::new();
    };
    [("Home", home.clone()), ("Pictures", home.join("Pictures")), ("Downloads", home.join("Downloads"))]
        .into_iter()
        .filter(|(_, path)| path.is_dir())
        .map(|(label, path)| AssetRoot { path: path.to_string_lossy().into_owned(), label: label.into() })
        .collect()
}

