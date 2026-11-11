//! On-demand storage analysis. Nothing is scanned until the user asks for it,
//! and every walk is cancellable.

use rusted_core::format_bytes;
use rusted_fs::DriveInfo;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("scan cancelled")]
    Cancelled,
    #[error("could not read {path}: {source}")]
    Io { path: PathBuf, #[source] source: std::io::Error },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FolderUsage {
    pub path: PathBuf,
    pub name: String,
    pub bytes: u64,
    /// Bytes of files directly inside this folder (excluding subfolders).
    pub own_bytes: u64,
    pub files: usize,
    pub folders: usize,
    pub skipped_links: usize,
    pub children: Vec<FolderUsage>,
    pub error: Option<String>,
}

impl FolderUsage {
    pub fn size_label(&self) -> String {
        format_bytes(self.bytes)
    }

    /// Flattens the tree into the largest folders first, for the list pane.
    pub fn flattened(&self, out: &mut Vec<(PathBuf, u64)>) {
        out.push((self.path.clone(), self.bytes));
        for child in &self.children {
            child.flattened(out);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryUsage {
    pub category: &'static str,
    pub bytes: u64,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LargestFile {
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StorageReport {
    pub root: PathBuf,
    pub total_bytes: u64,
    pub total_files: usize,
    pub total_folders: usize,
    pub tree: FolderUsage,
    pub categories: Vec<CategoryUsage>,
    pub largest_files: Vec<LargestFile>,
    pub skipped: usize,
    pub cancelled: bool,
    pub elapsed_ms: u128,
}

impl StorageReport {
    pub fn total_label(&self) -> String {
        format_bytes(self.total_bytes)
    }

    /// Top-level folders sorted by size, ready for a treemap.
    pub fn treemap(&self) -> Vec<(String, u64)> {
        let mut rows: Vec<(String, u64)> = self.tree.children.iter().map(|child| (child.name.clone(), child.bytes)).collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1));
        rows
    }

    pub fn largest_folder_list(&self, limit: usize) -> Vec<(PathBuf, u64)> {
        let mut all = Vec::new();
        self.tree.flattened(&mut all);
        all.sort_by(|a, b| b.1.cmp(&a.1));
        all.truncate(limit);
        all
    }
}

pub fn category_for(extension: Option<&str>) -> &'static str {
    match extension.unwrap_or_default() {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" | "tif" | "tiff" | "svg" | "heic" => "Images",
        "mp4" | "mkv" | "mov" | "webm" | "avi" | "m4v" | "wmv" | "mp3" | "wav" | "flac" | "m4a" | "ogg" | "opus" => "Media",
        "zip" | "7z" | "tar" | "gz" | "tgz" | "rar" | "bz2" | "xz" | "iso" => "Archives",
        "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "pdf" | "txt" | "md" | "csv" | "rtf" | "odt" => "Documents",
        "exe" | "msi" | "dll" | "sys" | "cab" | "appx" => "Applications",
        "rs" | "js" | "ts" | "py" | "c" | "h" | "cpp" | "hpp" | "cs" | "java" | "go" | "rb" | "php" | "html" | "css" | "json" | "toml" | "yaml" | "yml" | "sql" => "Code",
        _ => "Other",
    }
}

pub const CATEGORIES: [&str; 7] = ["Images", "Media", "Archives", "Documents", "Applications", "Code", "Other"];

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub max_depth: usize,
    pub largest_files: usize,
    pub follow_links: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self { max_depth: 6, largest_files: 25, follow_links: false }
    }
}

/// Walks a subtree and returns real measured sizes. Directories that cannot be
/// read are recorded with an error instead of being silently reported as empty.
pub fn scan(root: &Path, options: &ScanOptions, cancel: &AtomicBool) -> Result<StorageReport, StorageError> {
    let started = std::time::Instant::now();
    let mut report = StorageReport { root: root.to_path_buf(), ..StorageReport::default() };
    let mut categories: std::collections::BTreeMap<&'static str, (u64, usize)> = Default::default();
    let mut largest: Vec<LargestFile> = Vec::new();
    let mut tree = FolderUsage { path: root.to_path_buf(), name: display_name(root), ..FolderUsage::default() };
    walk(root, options, cancel, 0, &mut tree, &mut categories, &mut largest, &mut report)?;
    tree.bytes = tree.children.iter().map(|child| child.bytes).sum::<u64>() + tree.own_bytes;
    report.total_bytes = tree.bytes;
    report.total_files = tree.files;
    report.total_folders = tree.folders;
    report.categories = CATEGORIES
        .iter()
        .map(|name| {
            let (bytes, count) = categories.get(name).copied().unwrap_or((0, 0));
            CategoryUsage { category: name, bytes, count }
        })
        .collect();
    largest.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    largest.truncate(options.largest_files);
    report.largest_files = largest;
    report.tree = tree;
    if cancel.load(Ordering::Relaxed) {
        report.cancelled = true;
    }
    report.elapsed_ms = started.elapsed().as_millis();
    Ok(report)
}

fn walk(
    path: &Path,
    options: &ScanOptions,
    cancel: &AtomicBool,
    depth: usize,
    folder: &mut FolderUsage,
    categories: &mut std::collections::BTreeMap<&'static str, (u64, usize)>,
    largest: &mut Vec<LargestFile>,
    report: &mut StorageReport,
) -> Result<(), StorageError> {
    if cancel.load(Ordering::Relaxed) {
        return Err(StorageError::Cancelled);
    }
    let reader = match std::fs::read_dir(rusted_fs::long_path(path)) {
        Ok(reader) => reader,
        Err(source) => {
            folder.error = Some(source.to_string());
            report.skipped += 1;
            return Ok(());
        }
    };
    for item in reader {
        if cancel.load(Ordering::Relaxed) {
            return Err(StorageError::Cancelled);
        }
        let item = match item {
            Ok(item) => item,
            Err(_) => {
                report.skipped += 1;
                continue;
            }
        };
        let metadata = match std::fs::symlink_metadata(item.path()) {
            Ok(metadata) => metadata,
            Err(_) => {
                report.skipped += 1;
                continue;
            }
        };
        let file_type = metadata.file_type();
        if file_type.is_symlink() && !options.follow_links {
            folder.skipped_links += 1;
            continue;
        }
        if file_type.is_dir() {
            folder.folders += 1;
            report.total_folders += 1;
            let mut child = FolderUsage { path: item.path(), name: display_name(&item.path()), ..FolderUsage::default() };
            if depth < options.max_depth {
                walk(&item.path(), options, cancel, depth + 1, &mut child, categories, largest, report)?;
                child.bytes = child.children.iter().map(|nested| nested.bytes).sum::<u64>() + child.own_bytes;
            }
            folder.bytes += child.bytes;
            folder.folders += child.folders;
            folder.files += child.files;
            folder.skipped_links += child.skipped_links;
            folder.children.push(child);
        } else if file_type.is_file() {
            let bytes = metadata.len();
            folder.files += 1;
            folder.own_bytes += bytes;
            report.total_files += 1;
            let extension = item.path().extension().map(|value| value.to_string_lossy().to_lowercase());
            let entry = categories.entry(category_for(extension.as_deref())).or_insert((0, 0));
            entry.0 += bytes;
            entry.1 += 1;
            largest.push(LargestFile { path: item.path(), bytes });
            if largest.len() > options.largest_files * 8 {
                largest.sort_by(|a, b| b.bytes.cmp(&a.bytes));
                largest.truncate(options.largest_files * 4);
            }
        }
    }
    let mut children = std::mem::take(&mut folder.children);
    children.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    folder.children = children;
    Ok(())
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// Capacity rows for the drive list. Free/total come from the platform API.
pub fn drive_usage(drive: &DriveInfo) -> String {
    match (drive.total, drive.free) {
        (Some(total), Some(free)) => format!(
            "{} free of {} ({} used)",
            format_bytes(free),
            format_bytes(total),
            format_bytes(total.saturating_sub(free))
        ),
        _ => "Capacity unavailable".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_real_tree_and_categories() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("pics")).unwrap();
        std::fs::create_dir_all(dir.path().join("code/src")).unwrap();
        std::fs::write(dir.path().join("pics/a.png"), vec![0u8; 1000]).unwrap();
        std::fs::write(dir.path().join("code/src/main.rs"), vec![0u8; 500]).unwrap();
        std::fs::write(dir.path().join("notes.txt"), vec![0u8; 100]).unwrap();
        let report = scan(dir.path(), &ScanOptions::default(), &AtomicBool::new(false)).unwrap();
        assert_eq!(report.total_bytes, 1600);
        assert_eq!(report.total_files, 3);
        assert_eq!(report.total_folders, 4);
        let images = report.categories.iter().find(|c| c.category == "Images").unwrap();
        assert_eq!(images.bytes, 1000);
        let code = report.categories.iter().find(|c| c.category == "Code").unwrap();
        assert_eq!(code.bytes, 500);
        assert_eq!(report.largest_files.first().unwrap().bytes, 1000);
        assert!(!report.treemap().is_empty());
        assert!(report.largest_folder_list(3).len() <= 3);
    }

    #[test]
    fn cancellation_is_reported_without_partial_claims() {
        let dir = tempfile::tempdir().unwrap();
        for index in 0..50 {
            std::fs::write(dir.path().join(format!("f{index}.bin")), [0u8; 16]).unwrap();
        }
        let cancel = AtomicBool::new(false);
        cancel.store(true, Ordering::Relaxed);
        let result = scan(dir.path(), &ScanOptions::default(), &cancel);
        assert!(matches!(result, Err(StorageError::Cancelled)));
    }

    #[test]
    fn unreadable_subfolders_are_reported_not_hidden() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("gone");
        let report = scan(&missing, &ScanOptions::default(), &AtomicBool::new(false)).unwrap();
        assert_eq!(report.skipped, 1);
        assert!(report.tree.error.is_some());
    }

    #[test]
    fn category_mapping_is_stable() {
        assert_eq!(category_for(Some("png")), "Images");
        assert_eq!(category_for(Some("rs")), "Code");
        assert_eq!(category_for(None), "Other");
        assert_eq!(category_for(Some("xyz")), "Other");
    }
}
