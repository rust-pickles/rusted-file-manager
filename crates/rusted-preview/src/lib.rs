//! Asynchronous-friendly, bounded preview generation.
//!
//! Everything here is pure: give it a path and limits, get a `Preview`. The UI
//! calls it from a background task and renders the result. Formats that need a
//! heavy codec (PDF rendering, proprietary video) are reported as unsupported
//! instead of being faked.

mod audio;
mod image_preview;
mod inspector;
mod text;

pub use audio::{audio_metadata, video_metadata};
pub use image_preview::{decode_image, ImagePreview};
pub use inspector::{describe, InspectorInfo};
pub use text::{read_text_preview, TextPreview};

use rusted_core::{FileEntry, Settings};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PreviewError {
    #[error("could not read {path}: {source}")]
    Io { path: PathBuf, #[source] source: std::io::Error },
    #[error("unsupported format: {0}")]
    Unsupported(String),
    #[error("image could not be decoded: {0}")]
    Decode(String),
    #[error("file is too large to preview ({0} bytes)")]
    TooLarge(u64),
}

impl PreviewError {
    pub fn io(path: &Path, source: std::io::Error) -> Self {
        Self::Io { path: path.to_path_buf(), source }
    }
}

/// Hard limits that keep previews cheap and the UI responsive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviewLimits {
    pub text_bytes: u64,
    pub image_bytes: u64,
    pub image_dimension: u32,
    pub folder_entries: usize,
}

impl PreviewLimits {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            text_bytes: settings.preview_text_limit_kib as u64 * 1024,
            image_bytes: 32 * 1024 * 1024,
            image_dimension: 8_192,
            folder_entries: 60,
        }
    }
}

impl Default for PreviewLimits {
    fn default() -> Self {
        Self::from_settings(&Settings::default())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Preview {
    Text(TextPreview),
    Image(ImagePreview),
    /// Metadata-only preview: audio/video tags, PDF info, or a folder summary.
    Details { kind: String, fields: Vec<(String, String)>, note: Option<String> },
    Unsupported { kind: String, reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewKind {
    Text,
    Image,
    Audio,
    Video,
    Pdf,
    Folder,
    Archive,
    Unsupported,
}

/// Classifies by extension only; no content sniffing is required to pick a view.
pub fn classify(entry: &FileEntry) -> PreviewKind {
    if entry.is_dir() {
        return PreviewKind::Folder;
    }
    let extension = entry.extension.as_deref().unwrap_or_default();
    match extension {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" | "tif" | "tiff" => PreviewKind::Image,
        "txt" | "md" | "markdown" | "rs" | "toml" | "json" | "yaml" | "yml" | "toml" | "ini" | "cfg"
        | "conf" | "log" | "csv" | "tsv" | "xml" | "html" | "htm" | "css" | "js" | "ts" | "tsx" | "jsx"
        | "c" | "h" | "cpp" | "hpp" | "cs" | "java" | "kt" | "py" | "rb" | "go" | "lua" | "php" | "sh"
        | "ps1" | "bat" | "cmd" | "sql" | "gitignore" | "editorconfig" | "lock" => PreviewKind::Text,
        "mp3" | "wav" | "flac" | "m4a" | "aac" | "ogg" | "opus" | "wma" => PreviewKind::Audio,
        "mp4" | "mkv" | "mov" | "webm" | "avi" | "m4v" | "wmv" => PreviewKind::Video,
        "pdf" => PreviewKind::Pdf,
        "zip" | "tar" | "gz" | "tgz" | "7z" | "bz2" | "xz" | "rar" => PreviewKind::Archive,
        _ => PreviewKind::Unsupported,
    }
}

/// Builds a preview synchronously. Callers must run this off the UI thread.
pub fn preview(entry: &FileEntry, limits: PreviewLimits) -> Result<Preview, PreviewError> {
    match classify(entry) {
        PreviewKind::Image => {
            if entry.size > limits.image_bytes {
                return Err(PreviewError::TooLarge(entry.size));
            }
            Ok(Preview::Image(decode_image(&entry.path, limits)?))
        }
        PreviewKind::Text => Ok(Preview::Text(read_text_preview(&entry.path, limits.text_bytes)?)),
        PreviewKind::Audio => Ok(Preview::Details {
            kind: "Audio".into(),
            fields: audio_metadata(&entry.path, entry.size),
            note: Some("Playback is handled by your default audio application.".into()),
        }),
        PreviewKind::Video => Ok(Preview::Details {
            kind: "Video".into(),
            fields: video_metadata(&entry.path, entry.size),
            note: Some("Frames are not decoded; open with the default player for playback.".into()),
        }),
        PreviewKind::Pdf => Ok(Preview::Details {
            kind: "PDF document".into(),
            fields: pdf_metadata(&entry.path, entry.size),
            note: Some("PDF rendering needs an external engine; use Open to view the document.".into()),
        }),
        PreviewKind::Folder => Ok(Preview::Details {
            kind: "Folder".into(),
            fields: folder_summary(&entry.path, limits.folder_entries),
            note: None,
        }),
        PreviewKind::Archive => Ok(Preview::Unsupported {
            kind: "Archive".into(),
            reason: "Archive contents are browsed through the archive view.".into(),
        }),
        PreviewKind::Unsupported => Ok(Preview::Unsupported {
            kind: entry.type_name().to_string(),
            reason: "No preview is available for this file type.".into(),
        }),
    }
}

fn folder_summary(path: &Path, limit: usize) -> Vec<(String, String)> {
    let mut folders = 0usize;
    let mut files = 0usize;
    let mut bytes = 0u64;
    let mut samples = Vec::new();
    if let Ok(reader) = std::fs::read_dir(rusted_fs::long_path(path)) {
        for item in reader.flatten() {
            match item.file_type() {
                Ok(kind) if kind.is_dir() => folders += 1,
                _ => {
                    files += 1;
                    if let Ok(metadata) = item.metadata() {
                        bytes += metadata.len();
                    }
                }
            }
            if samples.len() < limit {
                samples.push(item.file_name().to_string_lossy().into_owned());
            }
        }
    }
    let mut fields = vec![
        ("Items".into(), format!("{} folders, {} files", folders, files)),
        ("Size (top level)".into(), rusted_core::format_bytes(bytes)),
    ];
    if !samples.is_empty() {
        samples.sort();
        fields.push(("Contains".into(), samples.join(", ")));
    }
    fields
}

/// PDF metadata without a renderer: page count is read from the document
/// catalog when it can be located; otherwise only size is reported.
fn pdf_metadata(path: &Path, size: u64) -> Vec<(String, String)> {
    let mut fields = vec![("Size".into(), rusted_core::format_bytes(size))];
    if let Ok(bytes) = std::fs::read(rusted_fs::long_path(path)) {
        let text = String::from_utf8_lossy(&bytes[..bytes.len().min(2 * 1024 * 1024)]).into_owned();
        if let Some(version) = text.split_once("%PDF-").map(|(_, rest)| rest.chars().take(4).collect::<String>()) {
            fields.push(("PDF version".into(), version));
        }
        let pages = text.matches("/Type /Page").count() + text.matches("/Type/Page").count();
        if pages > 0 {
            fields.push(("Pages (estimated)".into(), pages.to_string()));
        }
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusted_core::{EntryKind, FileEntry};
    use std::io::Write;

    fn entry(path: &Path) -> FileEntry {
        let metadata = std::fs::metadata(path).unwrap();
        FileEntry {
            path: path.to_path_buf(),
            name: path.file_name().unwrap().to_string_lossy().into_owned(),
            kind: if metadata.is_dir() { EntryKind::Directory } else { EntryKind::File },
            size: metadata.len(),
            modified: None,
            created: None,
            readonly: false,
            hidden: false,
            extension: path.extension().map(|value| value.to_string_lossy().to_lowercase()),
        }
    }

    #[test]
    fn text_preview_reads_real_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.txt");
        std::fs::write(&path, "hello rusted").unwrap();
        match preview(&entry(&path), PreviewLimits::default()).unwrap() {
            Preview::Text(text) => {
                assert_eq!(text.content, "hello rusted");
                assert!(!text.truncated);
            }
            other => panic!("unexpected preview {other:?}"),
        }
    }

    #[test]
    fn oversized_text_is_truncated_not_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.log");
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(&vec![b'a'; 4096]).unwrap();
        let limits = PreviewLimits { text_bytes: 512, ..PreviewLimits::default() };
        match preview(&entry(&path), limits).unwrap() {
            Preview::Text(text) => {
                assert!(text.truncated);
                assert_eq!(text.content.len(), 512);
            }
            other => panic!("unexpected preview {other:?}"),
        }
    }

    #[test]
    fn folder_preview_counts_real_entries() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "x").unwrap();
        std::fs::create_dir(dir.path().join("nested")).unwrap();
        match preview(&entry(dir.path()), PreviewLimits::default()).unwrap() {
            Preview::Details { fields, .. } => {
                assert!(fields.iter().any(|(key, value)| key == "Items" && value == "1 folders, 1 files"));
            }
            other => panic!("unexpected preview {other:?}"),
        }
    }

    #[test]
    fn unknown_extensions_are_reported_as_unsupported() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("thing.weird");
        std::fs::write(&path, "x").unwrap();
        assert!(matches!(preview(&entry(&path), PreviewLimits::default()).unwrap(), Preview::Unsupported { .. }));
    }
}
