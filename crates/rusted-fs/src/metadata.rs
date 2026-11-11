use chrono::{DateTime, Utc};
use rusted_core::{EntryKind, FileEntry};
use std::fs::Metadata;
use std::path::Path;
use std::time::SystemTime;

const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

fn utc(value: std::io::Result<SystemTime>) -> Option<DateTime<Utc>> {
    value.ok().map(DateTime::<Utc>::from)
}

/// Builds a file entry from metadata that was already read by the enumerator.
/// No extra syscalls happen here beyond the platform reparse-tag probe.
pub fn entry_from_metadata(path: &Path, metadata: &Metadata, file_type: std::fs::FileType) -> FileEntry {
    let link = file_type.is_symlink();
    let kind = if link {
        if is_junction(path) {
            EntryKind::Junction
        } else {
            EntryKind::Symlink
        }
    } else if file_type.is_dir() {
        EntryKind::Directory
    } else if file_type.is_file() {
        EntryKind::File
    } else {
        EntryKind::Unknown
    };
    FileEntry {
        path: path.to_path_buf(),
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string()),
        kind,
        size: if metadata.is_file() { metadata.len() } else { 0 },
        modified: utc(metadata.modified()),
        created: utc(metadata.created()),
        readonly: metadata.permissions().readonly(),
        hidden: is_hidden(path, metadata),
        extension: path.extension().map(|extension| extension.to_string_lossy().to_lowercase()),
    }
}

#[cfg(unix)]
fn is_hidden(path: &Path, _: &Metadata) -> bool {
    path.file_name().is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

#[cfg(windows)]
fn is_hidden(_: &Path, metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0
}

#[cfg(not(any(unix, windows)))]
fn is_hidden(_: &Path, _: &Metadata) -> bool {
    false
}

/// Directory junctions are reparse points whose tag is IO_REPARSE_TAG_MOUNT_POINT.
/// They must never be walked recursively by default (they can form loops).
#[cfg(windows)]
pub fn is_junction(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT == 0 {
        return false;
    }
    super::windows::reparse_tag(path) == Some(super::windows::IO_REPARSE_TAG_MOUNT_POINT)
}

#[cfg(not(windows))]
pub fn is_junction(_: &Path) -> bool {
    false
}

/// True when the entry is a link of any kind; callers skip recursion into these.
pub fn is_link_like(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink() || is_junction(path))
        .unwrap_or(false)
}
