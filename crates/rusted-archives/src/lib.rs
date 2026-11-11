//! Archive browsing without extracting to disk first.
//!
//! Supported today: ZIP (`zip`), TAR and TAR.GZ (`tar` + `flate2`). 7z requires a
//! separate decoder and is reported as unsupported rather than half-working.

mod tar_archive;
mod zip_archive;

pub use tar_archive::TarArchive;
pub use zip_archive::ZipArchive;

use rusted_core::format_bytes;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("unsupported archive format: {0}")]
    Unsupported(String),
    #[error("archive I/O for {path}: {source}")]
    Io { path: PathBuf, #[source] source: io::Error },
    #[error("damaged archive: {0}")]
    Corrupt(String),
    #[error("entry escapes the extraction folder: {0}")]
    UnsafeEntry(String),
    #[error("entry not found in archive: {0}")]
    Missing(String),
    #[error("operation cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    /// Normalised `/`-separated path inside the archive.
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub compressed_size: Option<u64>,
}

impl ArchiveEntry {
    pub fn name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }

    pub fn depth(&self) -> usize {
        self.path.matches('/').count()
    }

    pub fn size_label(&self) -> String {
        if self.is_dir { "—".into() } else { format_bytes(self.size) }
    }
}

/// A browsable archive. Implementors must be read-only during browsing.
pub trait Archive {
    fn entries(&self) -> &[ArchiveEntry];
    fn read_entry(&mut self, path: &str) -> Result<Vec<u8>, ArchiveError>;
    /// Extracts a single entry, creating parent folders as needed.
    fn extract_entry(&mut self, path: &str, destination: &Path) -> Result<PathBuf, ArchiveError> {
        let target = safe_join(destination, path)?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|source| ArchiveError::Io { path: parent.into(), source })?;
        }
        if path.ends_with('/') {
            std::fs::create_dir_all(&target).map_err(|source| ArchiveError::Io { path: target.clone(), source })?;
            return Ok(target);
        }
        let bytes = self.read_entry(path)?;
        std::fs::write(&target, bytes).map_err(|source| ArchiveError::Io { path: target.clone(), source })?;
        Ok(target)
    }
}

/// Opens an archive by extension. TAR.GZ is detected before plain TAR.
pub fn open(path: &Path) -> Result<Box<dyn Archive>, ArchiveError> {
    let name = path.file_name().map(|value| value.to_string_lossy().to_lowercase()).unwrap_or_default();
    if name.ends_with(".tgz") || name.ends_with(".tar.gz") {
        return Ok(Box::new(TarArchive::open_gzip(path)?));
    }
    if name.ends_with(".tar") {
        return Ok(Box::new(TarArchive::open(path)?));
    }
    if name.ends_with(".zip") {
        return Ok(Box::new(ZipArchive::open(path)?));
    }
    if name.ends_with(".7z") {
        return Err(ArchiveError::Unsupported("7z needs an extra decoder and is not enabled".into()));
    }
    if name.ends_with(".gz") {
        return Err(ArchiveError::Unsupported("a bare .gz stream has no file table".into()));
    }
    Err(ArchiveError::Unsupported(format!("no reader for {name}")))
}

/// Extracts every entry, honouring cancellation between items. Kept as a free
/// function so the `Archive` trait stays object safe.
pub fn extract_all(
    archive: &mut dyn Archive,
    destination: &Path,
    cancel: &AtomicBool,
    mut progress: impl FnMut(usize, usize),
) -> Result<usize, ArchiveError> {
    let paths: Vec<String> = archive.entries().iter().map(|entry| entry.path.clone()).collect();
    let total = paths.len();
    for (index, path) in paths.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err(ArchiveError::Cancelled);
        }
        archive.extract_entry(path, destination)?;
        progress(index + 1, total);
    }
    Ok(total)
}

/// Reads a single member out of an archive without keeping a handle around.
pub fn read_file(path: &Path, member: &str) -> Result<Vec<u8>, ArchiveError> {
    let mut archive = open(path)?;
    archive.read_entry(member)
}

/// Joins an archive entry onto an extraction root, rejecting traversal.
pub fn safe_join(root: &Path, member: &str) -> Result<PathBuf, ArchiveError> {
    let candidate = Path::new(member);
    if candidate.is_absolute() || member.contains(':') {
        return Err(ArchiveError::UnsafeEntry(member.into()));
    }
    let mut out = root.to_path_buf();
    for component in candidate.components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => return Err(ArchiveError::UnsafeEntry(member.into())),
        }
    }
    Ok(out)
}

pub(crate) fn normalise(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

pub fn create_zip(destination: &Path, sources: &[PathBuf], cancel: &AtomicBool) -> Result<usize, ArchiveError> {
    let file = std::fs::File::create(destination).map_err(|source| ArchiveError::Io { path: destination.into(), source })?;
    let mut writer = zip::ZipWriter::new(file);
    let options: zip::write::SimpleFileOptions =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut added = 0;
    for source in sources {
        if cancel.load(Ordering::Relaxed) {
            return Err(ArchiveError::Cancelled);
        }
        added += add_to_zip(&mut writer, source, source.file_name().unwrap_or_default().to_string_lossy().as_ref(), options, cancel)?;
    }
    writer.finish().map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
    Ok(added)
}

fn add_to_zip(
    writer: &mut zip::ZipWriter<std::fs::File>,
    path: &Path,
    name: &str,
    options: zip::write::SimpleFileOptions,
    cancel: &AtomicBool,
) -> Result<usize, ArchiveError> {
    if cancel.load(Ordering::Relaxed) {
        return Err(ArchiveError::Cancelled);
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|source| ArchiveError::Io { path: path.into(), source })?;
    if metadata.is_dir() {
        writer.add_directory(format!("{name}/"), options).map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
        let mut count = 1;
        for child in std::fs::read_dir(path).map_err(|source| ArchiveError::Io { path: path.into(), source })? {
            let child = child.map_err(|source| ArchiveError::Io { path: path.into(), source })?;
            let child_name = child.file_name().to_string_lossy().into_owned();
            count += add_to_zip(writer, &child.path(), &format!("{name}/{child_name}"), options, cancel)?;
        }
        Ok(count)
    } else if metadata.file_type().is_symlink() {
        // Symlinks are skipped deliberately: storing a target that resolves
        // differently on extraction is a safety hazard, not a feature.
        Ok(0)
    } else {
        writer.start_file(name, options).map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
        let mut file = std::fs::File::open(path).map_err(|source| ArchiveError::Io { path: path.into(), source })?;
        io::copy(&mut file, writer).map_err(|source| ArchiveError::Io { path: path.into(), source })?;
        Ok(1)
    }
}

/// Adds files to an existing archive by rebuilding it into a temporary file.
/// Zip writers cannot append to a finished archive in place, so the result is
/// built beside the original and swapped in with a single rename.
pub fn add_files_to_zip(destination: &Path, sources: &[PathBuf], cancel: &AtomicBool) -> Result<usize, ArchiveError> {
    let temporary = destination.with_extension("rusted-tmp.zip");
    let mut existing = zip::ZipArchive::new(
        std::fs::File::open(destination).map_err(|source| ArchiveError::Io { path: destination.into(), source })?,
    )
    .map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
    let out = std::fs::File::create(&temporary).map_err(|source| ArchiveError::Io { path: temporary.clone(), source })?;
    let mut writer = zip::ZipWriter::new(out);
    let options: zip::write::SimpleFileOptions =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for index in 0..existing.len() {
        if cancel.load(Ordering::Relaxed) {
            let _ = std::fs::remove_file(&temporary);
            return Err(ArchiveError::Cancelled);
        }
        let mut member = existing.by_index(index).map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
        if member.is_dir() {
            writer.add_directory(member.name().to_string(), options).map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
        } else {
            writer.start_file(member.name().to_string(), options).map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
            io::copy(&mut member, &mut writer).map_err(|source| ArchiveError::Io { path: destination.into(), source })?;
        }
    }
    let mut added = 0;
    for source in sources {
        added += add_to_zip(&mut writer, source, source.file_name().unwrap_or_default().to_string_lossy().as_ref(), options, cancel)?;
    }
    writer.finish().map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
    std::fs::rename(&temporary, destination).map_err(|source| ArchiveError::Io { path: destination.into(), source })?;
    Ok(added)
}


