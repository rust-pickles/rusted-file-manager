use chrono::{DateTime, Utc};
use rusted_core::{EntryKind, FileEntry};
use std::{fs::Metadata, path::Path, time::SystemTime};

fn utc(value: std::io::Result<SystemTime>) -> Option<DateTime<Utc>> { value.ok().map(DateTime::<Utc>::from) }

pub fn entry_from_metadata(path: &Path, metadata: &Metadata, file_type: std::fs::FileType) -> FileEntry {
    let kind = if file_type.is_symlink() { EntryKind::Symlink }
        else if is_junction(path, metadata) { EntryKind::Junction }
        else if file_type.is_dir() { EntryKind::Directory }
        else if file_type.is_file() { EntryKind::File } else { EntryKind::Unknown };
    FileEntry {
        path: path.to_path_buf(),
        name: path.file_name().map(|n|n.to_string_lossy().into_owned()).unwrap_or_else(||path.display().to_string()),
        kind,
        size: if metadata.is_file(){metadata.len()}else{0},
        modified: utc(metadata.modified()), created: utc(metadata.created()),
        readonly: metadata.permissions().readonly(), hidden: is_hidden(path, metadata),
        extension: path.extension().map(|e|e.to_string_lossy().to_lowercase()),
    }
}
#[cfg(unix)]
fn is_hidden(path:&Path,_:&Metadata)->bool { path.file_name().is_some_and(|n|n.to_string_lossy().starts_with('.')) }
#[cfg(windows)]
fn is_hidden(_path:&Path,metadata:&Metadata)->bool { use std::os::windows::fs::MetadataExt; metadata.file_attributes() & 0x2 != 0 }
#[cfg(not(any(unix,windows)))]
fn is_hidden(_path:&Path,_:&Metadata)->bool { false }
#[cfg(windows)]
fn is_junction(_path:&Path,metadata:&Metadata)->bool { use std::os::windows::fs::MetadataExt; const IO_REPARSE_TAG_MOUNT_POINT:u32=0xA0000003; metadata.file_attributes()&0x400!=0 && metadata.reparse_tag()==IO_REPARSE_TAG_MOUNT_POINT }
#[cfg(not(windows))]
fn is_junction(_path:&Path,_:&Metadata)->bool { false }
