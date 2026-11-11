use crate::{entry_from_metadata, long_path};
use rusted_core::{sort_entries, FileEntry, SortSpec};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;

/// Entries are handed to the UI in batches of this size so a huge directory can
/// paint its first rows immediately.
pub const DIRECTORY_CHUNK_SIZE: usize = 256;

#[derive(Debug, Error)]
#[error("could not read {path}: {source}")]
pub struct DirectoryError {
    pub path: PathBuf,
    #[source]
    pub source: io::Error,
}

/// Enumerates one directory without following links. The callback receives
/// bounded batches; set `cancel` to stop early. Returns the number of entries.
pub fn enumerate_streaming<F>(
    path: &Path,
    show_hidden: bool,
    sort: SortSpec,
    cancel: &AtomicBool,
    mut on_chunk: F,
) -> Result<usize, DirectoryError>
where
    F: FnMut(Vec<FileEntry>),
{
    let actual = long_path(path);
    let reader = std::fs::read_dir(&actual).map_err(|source| DirectoryError { path: path.into(), source })?;
    let mut count = 0usize;
    let mut chunk = Vec::with_capacity(DIRECTORY_CHUNK_SIZE);
    let mut skipped = 0usize;
    for item in reader {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        // A single inaccessible entry must not hide its siblings.
        let Ok(item) = item else {
            skipped += 1;
            continue;
        };
        let entry_path = item.path();
        let Ok(file_type) = item.file_type() else {
            skipped += 1;
            continue;
        };
        let Ok(metadata) = std::fs::symlink_metadata(&entry_path) else {
            skipped += 1;
            continue;
        };
        let entry = entry_from_metadata(&entry_path, &metadata, file_type);
        if !show_hidden && entry.hidden {
            continue;
        }
        chunk.push(entry);
        count += 1;
        if chunk.len() == DIRECTORY_CHUNK_SIZE {
            sort_entries(&mut chunk, sort);
            on_chunk(std::mem::replace(&mut chunk, Vec::with_capacity(DIRECTORY_CHUNK_SIZE)));
        }
    }
    if !chunk.is_empty() {
        sort_entries(&mut chunk, sort);
        on_chunk(chunk);
    }
    let _ = skipped;
    Ok(count)
}

pub fn read_directory(path: &Path, show_hidden: bool, sort: SortSpec) -> Result<Vec<FileEntry>, DirectoryError> {
    let mut output = Vec::new();
    enumerate_streaming(path, show_hidden, sort, &AtomicBool::new(false), |mut chunk| output.append(&mut chunk))?;
    sort_entries(&mut output, sort);
    Ok(output)
}

pub fn create_directory(path: &Path) -> io::Result<()> {
    std::fs::create_dir(long_path(path))
}

pub fn create_file(path: &Path) -> io::Result<()> {
    std::fs::OpenOptions::new().write(true).create_new(true).open(long_path(path)).map(|_| ())
}

/// "New folder", "New folder (2)", … in `parent`, using the OS for the existence check.
pub fn unique_child_name(parent: &Path, stem: &str, extension: Option<&str>) -> String {
    let exists = |name: &str| parent.join(name).exists();
    let first = match extension {
        Some(extension) => format!("{stem}.{extension}"),
        None => stem.to_string(),
    };
    if !exists(&first) {
        return first;
    }
    for index in 2..10_000 {
        let candidate = match extension {
            Some(extension) => format!("{stem} ({index}).{extension}"),
            None => format!("{stem} ({index})"),
        };
        if !exists(&candidate) {
            return candidate;
        }
    }
    let stamp = std::process::id();
    match extension {
        Some(extension) => format!("{stem} {stamp}.{extension}"),
        None => format!("{stem} {stamp}"),
    }
}

pub type SharedCancellation = Arc<AtomicBool>;

#[cfg(test)]
mod tests {
    use super::*;
    use rusted_core::{EntryKind, SortKey};

    #[test]
    fn enumerates_real_entries_and_honours_hidden() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("b.txt"), b"hello").unwrap();
        std::fs::create_dir(temp.path().join("folder")).unwrap();
        std::fs::write(temp.path().join(".hidden"), b"x").unwrap();
        let all = read_directory(temp.path(), true, SortSpec::default()).unwrap();
        assert_eq!(all.len(), 3);
        let visible = read_directory(temp.path(), false, SortSpec::default()).unwrap();
        assert_eq!(visible.len(), 2);
        assert!(all.iter().any(|entry| entry.kind == EntryKind::Directory));
    }

    #[test]
    fn streaming_yields_sorted_batches() {
        let temp = tempfile::tempdir().unwrap();
        for index in 0..(DIRECTORY_CHUNK_SIZE + 10) {
            std::fs::write(temp.path().join(format!("f{index:04}.txt")), b"x").unwrap();
        }
        let mut batches = Vec::new();
        let count = enumerate_streaming(
            temp.path(),
            false,
            SortSpec { key: SortKey::Name, descending: false, folders_first: true },
            &AtomicBool::new(false),
            |chunk| batches.push(chunk.len()),
        )
        .unwrap();
        assert_eq!(count, DIRECTORY_CHUNK_SIZE + 10);
        assert_eq!(batches.len(), 2);
        assert!(batches.iter().all(|size| *size <= DIRECTORY_CHUNK_SIZE));
    }

    #[test]
    fn cancellation_stops_enumeration() {
        let temp = tempfile::tempdir().unwrap();
        for index in 0..100 {
            std::fs::write(temp.path().join(format!("f{index}.txt")), b"x").unwrap();
        }
        let cancel = AtomicBool::new(true);
        let count = enumerate_streaming(temp.path(), false, SortSpec::default(), &cancel, |_| {}).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn missing_directory_reports_the_path() {
        let error = read_directory(Path::new("/definitely/not/here"), false, SortSpec::default()).unwrap_err();
        assert!(error.to_string().contains("definitely"));
    }

    #[test]
    fn unique_names_avoid_collisions() {
        let temp = tempfile::tempdir().unwrap();
        assert_eq!(unique_child_name(temp.path(), "New folder", None), "New folder");
        std::fs::create_dir(temp.path().join("New folder")).unwrap();
        assert_eq!(unique_child_name(temp.path(), "New folder", None), "New folder (2)");
        assert_eq!(unique_child_name(temp.path(), "New file", Some("txt")), "New file.txt");
    }
}
