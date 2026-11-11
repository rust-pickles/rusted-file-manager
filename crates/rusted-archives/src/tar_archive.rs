use crate::{normalise, Archive, ArchiveEntry, ArchiveError};
use flate2::read::GzDecoder;
use std::io::Read;
use std::path::{Path, PathBuf};

enum Source {
    Plain(std::fs::File),
    Gzip(GzDecoder<std::fs::File>),
}

impl Read for Source {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Source::Plain(file) => file.read(buffer),
            Source::Gzip(file) => file.read(buffer),
        }
    }
}

/// TAR/TAR.GZ reader. The table of contents is captured on open; entry data is
/// re-read on demand because TAR is not randomly seekable when gzipped.
pub struct TarArchive {
    path: PathBuf,
    gzip: bool,
    entries: Vec<ArchiveEntry>,
}

impl TarArchive {
    pub fn open(path: &Path) -> Result<Self, ArchiveError> {
        Self::load(path, false)
    }

    pub fn open_gzip(path: &Path) -> Result<Self, ArchiveError> {
        Self::load(path, true)
    }

    fn load(path: &Path, gzip: bool) -> Result<Self, ArchiveError> {
        let mut entries = Vec::new();
        Self::scan(path, gzip, |entry| entries.push(entry))?;
        Ok(Self { path: path.to_path_buf(), gzip, entries })
    }

    fn scan(path: &Path, gzip: bool, mut on_entry: impl FnMut(ArchiveEntry)) -> Result<(), ArchiveError> {
        let file = std::fs::File::open(rusted_fs::long_path(path))
            .map_err(|source| ArchiveError::Io { path: path.into(), source })?;
        let source: Source = if gzip { Source::Gzip(GzDecoder::new(file)) } else { Source::Plain(file) };
        let mut archive = tar::Archive::new(source);
        let entries = archive.entries().map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
        for entry in entries {
            let entry = entry.map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
            let header = entry.header();
            let is_dir = header.entry_type().is_dir();
            let size = header.size().unwrap_or(0);
            let raw = entry.path().map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
            let path = normalise(&raw.to_string_lossy());
            if path.is_empty() {
                continue;
            }
            on_entry(ArchiveEntry { path, is_dir, size, compressed_size: None });
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Archive for TarArchive {
    fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    fn read_entry(&mut self, path: &str) -> Result<Vec<u8>, ArchiveError> {
        let wanted = normalise(path);
        let file = std::fs::File::open(rusted_fs::long_path(&self.path))
            .map_err(|source| ArchiveError::Io { path: self.path.clone(), source })?;
        let source: Source = if self.gzip { Source::Gzip(GzDecoder::new(file)) } else { Source::Plain(file) };
        let mut archive = tar::Archive::new(source);
        let entries = archive.entries().map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
        for entry in entries {
            let mut entry = entry.map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
            let raw = entry.path().map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
            if normalise(&raw.to_string_lossy()) == wanted {
                if entry.header().entry_type().is_dir() {
                    return Err(ArchiveError::Missing(wanted));
                }
                let mut buffer = Vec::with_capacity(entry.size() as usize);
                entry.read_to_end(&mut buffer).map_err(|source| ArchiveError::Io { path: self.path.clone(), source })?;
                return Ok(buffer);
            }
        }
        Err(ArchiveError::Missing(wanted))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::AtomicBool;

    fn fixture(dir: &Path) -> PathBuf {
        let source = dir.join("docs");
        std::fs::create_dir_all(source.join("sub")).unwrap();
        std::fs::write(source.join("readme.md"), "# hello").unwrap();
        std::fs::write(source.join("sub/data.bin"), [1u8, 2, 3]).unwrap();
        let archive_path = dir.join("docs.tar.gz");
        let file = std::fs::File::create(&archive_path).unwrap();
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);
        builder.append_dir_all("docs", &source).unwrap();
        builder.into_inner().unwrap().finish().unwrap();
        archive_path
    }

    #[test]
    fn lists_and_reads_tar_gz_members() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = fixture(dir.path());
        let mut archive = TarArchive::open_gzip(&archive_path).unwrap();
        assert!(archive.entries().iter().any(|entry| entry.path == "docs/readme.md"));
        assert_eq!(archive.read_entry("docs/readme.md").unwrap(), b"# hello");
        assert!(matches!(archive.read_entry("docs/nope"), Err(ArchiveError::Missing(_))));
    }

    #[test]
    fn extracts_entries_with_progress() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = fixture(dir.path());
        let out = dir.path().join("out");
        let mut archive = TarArchive::open_gzip(&archive_path).unwrap();
        let mut seen = Vec::new();
        crate::extract_all(&mut archive, &out, &AtomicBool::new(false), |done, total| seen.push((done, total))).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("docs/readme.md")).unwrap(), "# hello");
        assert_eq!(seen.last().copied(), Some((archive.entries().len(), archive.entries().len())));
    }

    #[test]
    fn plain_tar_is_supported() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("plain");
        std::fs::create_dir(&source).unwrap();
        std::fs::File::create(source.join("x.txt")).unwrap().write_all(b"x").unwrap();
        let archive_path = dir.path().join("plain.tar");
        let mut builder = tar::Builder::new(std::fs::File::create(&archive_path).unwrap());
        builder.append_dir_all("plain", &source).unwrap();
        builder.finish().unwrap();
        let archive = TarArchive::open(&archive_path).unwrap();
        assert!(archive.entries().iter().any(|entry| entry.path == "plain/x.txt"));
    }
}
