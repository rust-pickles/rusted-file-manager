//! Rust half of the frontend contract.
//!
//! These types are the single source of truth for what crosses the IPC boundary
//! and mirror `ui/types/filesystem.ts` field for field. The webview's TypeScript
//! definitions are the other half; every field here appears there with the same
//! name and shape, because the frontend was written against its own mocks first
//! and renaming anything would be a UI rewrite.
//!
//! Rules that keep this layer honest:
//! * `camelCase` everywhere — serde renames, never manual string keys.
//! * Timestamps are ISO-8601 strings, not epoch integers: the React formatters
//!   parse them.
//! * Sizes are `u64`; TypeScript numbers hold file sizes exactly below 2^53.
//! * Nothing here owns logic. Conversion from the backend crates happens in
//!   `impl From<...>`, so a backend change breaks the conversion, not the UI.

use rusted_core::{EntryKind, FileEntry, ThemeName};
use rusted_fs::DriveInfo;
use serde::{Deserialize, Serialize};

/// `type FileType = 'file' | 'folder'`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    File,
    Folder,
}

/// `type PreviewKind = 'image' | ... | 'binary'`.
///
/// The backend does not decide previews yet; the classifier below only has to
/// agree with the frontend's string union so nothing has to be renamed later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreviewKind {
    Image,
    Video,
    Audio,
    Code,
    Markdown,
    Document,
    Archive,
    Pdf,
    Binary,
}

impl PreviewKind {
    /// Extension-based classification. Mirrors nothing in the backend on purpose:
    /// this is presentation metadata, and `rusted-preview` remains authoritative
    /// once real thumbnail commands exist.
    pub fn for_extension(extension: Option<&str>) -> Self {
        match extension.unwrap_or_default().to_ascii_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "ico" | "tif" | "tiff" | "avif" => Self::Image,
            "mp4" | "mkv" | "mov" | "avi" | "webm" | "wmv" | "m4v" => Self::Video,
            "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "wma" | "opus" => Self::Audio,
            "md" | "markdown" => Self::Markdown,
            "rs" | "ts" | "tsx" | "js" | "jsx" | "toml" | "json" | "yaml" | "yml" | "css" | "html"
            | "py" | "go" | "c" | "h" | "cpp" | "cs" | "java" | "sh" | "ps1" | "sql" => Self::Code,
            "doc" | "docx" | "odt" | "rtf" | "xls" | "xlsx" | "ods" | "ppt" | "pptx" | "odp" => Self::Document,
            "zip" | "tar" | "gz" | "tgz" | "7z" | "rar" | "bz2" | "xz" | "zst" => Self::Archive,
            "pdf" => Self::Pdf,
            _ => Self::Binary,
        }
    }
}

/// `interface FileItem`. Field order and optionality match the TypeScript type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub parent_path: String,
    #[serde(rename = "type")]
    pub file_type: FileType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<String>,
    pub size: u64,
    pub created_at: String,
    pub modified_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_starred: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_system: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_kind: Option<PreviewKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<Dimensions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl FileItem {
    /// Stable id for a path. The frontend uses `id` as a selection and React key,
    /// so it must not change between two listings of the same folder. The path
    /// itself is the only stable identity available without a database.
    pub fn id_for(path: &str) -> String {
        path.to_string()
    }
}

impl From<FileEntry> for FileItem {
    fn from(entry: FileEntry) -> Self {
        let path = entry.path.to_string_lossy().into_owned();
        let parent_path = entry
            .path
            .parent()
            .map(|parent| parent.to_string_lossy().into_owned())
            .unwrap_or_default();
        let file_type = match entry.kind {
            EntryKind::Directory | EntryKind::Junction => FileType::Folder,
            EntryKind::File | EntryKind::Symlink | EntryKind::Unknown => FileType::File,
        };
        // Junctions are navigable folders but are not files, so the preview
        // classifier is skipped for them.
        let preview_kind = match file_type {
            FileType::Folder => None,
            FileType::File => Some(PreviewKind::for_extension(entry.extension.as_deref())),
        };
        Self {
            id: Self::id_for(&path),
            name: entry.name,
            path,
            parent_path,
            file_type,
            extension: entry.extension,
            size: entry.size,
            created_at: iso(entry.created),
            modified_at: iso(entry.modified),
            mime_type: None,
            is_starred: None,
            is_hidden: Some(entry.hidden),
            // A read-only file is not a "system" file; that flag is reserved for
            // paths the OS protects and is filled in once policy exists.
            is_system: None,
            preview_kind,
            preview_url: None,
            code_language: None,
            text_content: None,
            dimensions: None,
            duration: None,
            item_count: None,
            children_ids: None,
        }
    }
}

/// `interface DriveInfo`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveDto {
    pub id: String,
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub icon_type: DriveIconType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DriveIconType {
    Internal,
    External,
    Network,
}

impl From<DriveInfo> for DriveDto {
    fn from(drive: DriveInfo) -> Self {
        let mount_point = drive.root.to_string_lossy().into_owned();
        let total = drive.total.unwrap_or(0);
        let free = drive.free.unwrap_or(0);
        Self {
            id: mount_point.clone(),
            // Falls back to the mount point ("C:\") when a volume has no label.
            name: drive
                .label
                .clone()
                .filter(|label| !label.trim().is_empty())
                .unwrap_or_else(|| mount_point.clone()),
            mount_point,
            // The filesystem name needs a platform probe that is not wired up yet;
            // reporting a value would be a guess, so it stays empty.
            file_system: String::new(),
            total_bytes: total,
            used_bytes: total.saturating_sub(free),
            free_bytes: free,
            icon_type: if drive.network {
                DriveIconType::Network
            } else if drive.removable {
                DriveIconType::External
            } else {
                DriveIconType::Internal
            },
        }
    }
}

/// `type ThemeId = 'obsidian' | 'midnight' | ...`
///
/// `rusted_core::ThemeName` already serialises to exactly these six strings
/// (snake_case), so the theme ids are shared with the CSS custom properties in
/// `ui/index.css` without a translation table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeId {
    Obsidian,
    Midnight,
    Graphite,
    PureBlack,
    Snow,
    WarmSand,
}

impl From<ThemeName> for ThemeId {
    fn from(name: ThemeName) -> Self {
        match name {
            ThemeName::ObsidianBlack => Self::Obsidian,
            ThemeName::Midnight => Self::Midnight,
            ThemeName::Graphite => Self::Graphite,
            ThemeName::PureBlack => Self::PureBlack,
            ThemeName::Snow => Self::Snow,
            ThemeName::WarmSand => Self::WarmSand,
        }
    }
}

impl ThemeId {
    /// Every theme the CSS knows about, in display order.
    pub const ALL: [Self; 6] =
        [Self::Obsidian, Self::Midnight, Self::Graphite, Self::PureBlack, Self::Snow, Self::WarmSand];
}

/// One row of the asset-protocol allow list as the UI sees it.
///
/// The webview never receives a scope rule it could exploit: it gets the roots
/// the backend has already canonicalised, and every asset URL it builds is
/// re-checked against the same list in Rust before the file is served.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRoot {
    pub path: String,
    pub label: String,
}

/// The single handshake command the minimal shell exposes.
///
/// It exists so the frontend can prove the IPC bridge is alive and so the DTO
/// layer is exercised on a real build without inventing filesystem commands yet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub platform: String,
    pub themes: Vec<ThemeId>,
    pub asset_roots: Vec<AssetRoot>,
}

fn iso(time: Option<chrono::DateTime<chrono::Utc>>) -> String {
    match time {
        Some(value) => value.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        // The TypeScript type declares these as required strings. An empty string
        // is the honest representation of "the platform did not report a time",
        // and the frontend's date formatter already tolerates it.
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::path::PathBuf;

    fn entry(kind: EntryKind, name: &str, extension: Option<&str>) -> FileEntry {
        FileEntry {
            path: PathBuf::from("C:/Users/dev").join(name),
            name: name.to_string(),
            kind,
            size: 2048,
            modified: Utc_timestamp(),
            created: None,
            readonly: false,
            hidden: false,
            extension: extension.map(|value| value.to_string()),
        }
    }

    fn Utc_timestamp() -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::Utc.with_ymd_and_hms(2024, 5, 6, 7, 8, 9).single()
    }

    /// The webview destructures these names literally, so the JSON keys are part
    /// of the contract and are asserted rather than assumed.
    #[test]
    fn file_item_serialises_to_the_typescript_shape() {
        let item: FileItem = entry(EntryKind::File, "report.pdf", Some("pdf")).into();
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["type"], "file");
        assert_eq!(json["parentPath"], "C:/Users/dev");
        assert_eq!(json["modifiedAt"], "2024-05-06T07:08:09.000Z");
        assert_eq!(json["previewKind"], "pdf");
        assert_eq!(json["isHidden"], false);
        assert_eq!(json["size"], 2048);
        // Optional fields the backend does not fill must be absent, not null: the
        // frontend uses `field === undefined` checks for several of them.
        assert!(json.get("textContent").is_none());
        assert!(json.get("mimeType").is_none());
        // `type` must not be emitted as `fileType`.
        assert!(json.get("fileType").is_none());
    }

    #[test]
    fn folders_are_folders_and_have_no_preview_kind() {
        let item: FileItem = entry(EntryKind::Directory, "docs", None).into();
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["type"], "folder");
        assert!(json.get("previewKind").is_none());
        // Junctions are navigable, so they are folders too.
        let junction: FileItem = entry(EntryKind::Junction, "legacy", None).into();
        assert!(matches!(junction.file_type, FileType::Folder));
    }

    #[test]
    fn missing_timestamps_are_empty_strings_not_null() {
        let item: FileItem = entry(EntryKind::File, "x.txt", Some("txt")).into();
        assert_eq!(item.created_at, "");
        assert_eq!(item.modified_at, "2024-05-06T07:08:09.000Z");
    }

    #[test]
    fn ids_are_stable_for_the_same_path() {
        let first: FileItem = entry(EntryKind::File, "a.txt", Some("txt")).into();
        let second: FileItem = entry(EntryKind::File, "a.txt", Some("txt")).into();
        assert_eq!(first.id, second.id);
        assert_eq!(first.id, first.path);
    }

    #[test]
    fn preview_kinds_match_the_frontend_union() {
        assert!(matches!(PreviewKind::for_extension(Some("PNG")), PreviewKind::Image));
        assert!(matches!(PreviewKind::for_extension(Some("md")), PreviewKind::Markdown));
        assert!(matches!(PreviewKind::for_extension(Some("zip")), PreviewKind::Archive));
        assert!(matches!(PreviewKind::for_extension(Some("mp4")), PreviewKind::Video));
        assert!(matches!(PreviewKind::for_extension(Some("wav")), PreviewKind::Audio));
        assert!(matches!(PreviewKind::for_extension(Some("rs")), PreviewKind::Code));
        assert!(matches!(PreviewKind::for_extension(Some("docx")), PreviewKind::Document));
        assert!(matches!(PreviewKind::for_extension(Some("bin")), PreviewKind::Binary));
        assert!(matches!(PreviewKind::for_extension(None), PreviewKind::Binary));
    }

    #[test]
    fn theme_ids_match_the_six_css_themes() {
        let ids: Vec<String> = ThemeId::ALL
            .iter()
            .map(|theme| serde_json::to_value(theme).unwrap().as_str().unwrap().to_string())
            .collect();
        assert_eq!(ids, ["obsidian", "midnight", "graphite", "pure_black", "snow", "warm_sand"]);
        // And the backend enum maps onto them one-to-one.
        assert_eq!(ThemeId::from(ThemeName::ObsidianBlack), ThemeId::Obsidian);
        assert_eq!(ThemeId::from(ThemeName::WarmSand), ThemeId::WarmSand);
    }

    #[test]
    fn drives_report_used_bytes_and_icon_types() {
        let drive = DriveInfo {
            root: PathBuf::from("D:/"),
            label: Some("Backup".into()),
            removable: true,
            fixed: false,
            network: false,
            total: Some(1000),
            free: Some(250),
        };
        let dto: DriveDto = drive.into();
        assert_eq!(dto.name, "Backup");
        assert_eq!(dto.used_bytes, 750);
        assert_eq!(dto.free_bytes, 250);
        assert!(matches!(dto.icon_type, DriveIconType::External));
        assert_eq!(serde_json::to_value(&dto).unwrap()["mountPoint"], "D:/");
    }
}
