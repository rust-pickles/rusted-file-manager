use crate::{normalise, Archive, ArchiveEntry, ArchiveError};
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive as ZipReader;

pub struct ZipArchive {
    reader: ZipReader<std::fs::File>,
    entries: Vec<ArchiveEntry>,
}

impl ZipArchive {
    pub fn open(path: &Path) -> Result<Self, ArchiveError> {
        let file = std::fs::File::open(rusted_fs::long_path(path))
            .map_err(|source| ArchiveError::Io { path: path.into(), source })?;
        let mut reader = ZipReader::new(file).map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
        let mut entries = Vec::with_capacity(reader.len());
        for index in 0..reader.len() {
            let member = reader.by_index(index).map_err(|error| ArchiveError::Corrupt(error.to_string()))?;
            entries.push(ArchiveEntry {
                path: normalise(member.name()),
                is_dir: member.is_dir(),
                size: member.size(),
                compressed_size: Some(member.compressed_size()),
            });
        }
        Ok(Self { reader, entries })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Archive for ZipArchive {
    fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    fn read_entry(&mut self, path: &str) -> Result<Vec<u8>, ArchiveError> {
        let wanted = normalise(path);
        let mut member = self.reader.by_name(&wanted).map_err(|_| ArchiveError::Missing(wanted.clone()))?;
        if member.is_dir() {
            return Err(ArchiveError::Missing(wanted));
        }
        let mut buffer = Vec::with_capacity(member.size() as usize);
        member.read_to_end(&mut buffer).map_err(|source| ArchiveError::Io { path: PathBuf::from(path), source })?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_zip, safe_join};
    use std::io::Write;
    use std::sync::atomic::AtomicBool;

    fn build_fixture(dir: &Path) -> PathBuf {
        std::fs::create_dir_all(dir.join("in/nested")).unwrap();
        std::fs::write(dir.join("in/a.txt"), "alpha").unwrap();
        std::fs::write(dir.join("in/nested/b.txt"), "bravo").unwrap();
        let archive = dir.join("fixture.zip");
        create_zip(&archive, &[dir.join("in")], &AtomicBool::new(false)).unwrap();
        archive
    }

    #[test]
    fn browses_and_reads_entries_without_extracting() {
        let dir = tempfile::tempdir().unwrap();
        let archive = build_fixture(dir.path());
        let mut reader = ZipArchive::open(&archive).unwrap();
        let paths: Vec<_> = reader.entries().iter().map(|entry| entry.path.clone()).collect();
        assert!(paths.contains(&"in/a.txt".to_string()));
        assert!(paths.contains(&"in/nested/b.txt".to_string()));
        assert_eq!(reader.read_entry("in/nested/b.txt").unwrap(), b"bravo");
        assert!(matches!(reader.read_entry("missing.txt"), Err(ArchiveError::Missing(_))));
    }

    #[test]
    fn extract_all_creates_files_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let archive = build_fixture(dir.path());
        let out = dir.path().join("out");
        let mut reader = ZipArchive::open(&archive).unwrap();
        let count = crate::extract_all(&mut reader, &out, &AtomicBool::new(false), |_, _| {}).unwrap();
        assert!(count >= 4);
        assert_eq!(std::fs::read_to_string(out.join("in/a.txt")).unwrap(), "alpha");
    }

    #[test]
    fn extraction_can_be_cancelled() {
        let dir = tempfile::tempdir().unwrap();
        let archive = build_fixture(dir.path());
        let out = dir.path().join("out");
        let mut reader = ZipArchive::open(&archive).unwrap();
        let cancel = AtomicBool::new(true);
        assert!(matches!(crate::extract_all(&mut reader, &out, &cancel, |_, _| {}), Err(ArchiveError::Cancelled)));
    }

    #[test]
    fn traversal_entries_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        assert!(safe_join(dir.path(), "../../evil.txt").is_err());
        assert!(safe_join(dir.path(), "C:/Windows/system32").is_err());
        assert!(safe_join(dir.path(), "safe/file.txt").is_ok());
    }

    #[test]
    fn adding_files_rebuilds_the_archive_without_losing_entries() {
        let dir = tempfile::tempdir().unwrap();
        let archive = build_fixture(dir.path());
        let extra = dir.path().join("extra.txt");
        std::fs::File::create(&extra).unwrap().write_all(b"charlie").unwrap();
        let added = crate::add_files_to_zip(&archive, &[extra], &AtomicBool::new(false)).unwrap();
        assert_eq!(added, 1);
        let mut reader = ZipArchive::open(&archive).unwrap();
        let paths: Vec<_> = reader.entries().iter().map(|entry| entry.path.clone()).collect();
        assert!(paths.contains(&"in/a.txt".to_string()));
        assert!(paths.contains(&"extra.txt".to_string()));
        assert_eq!(reader.read_entry("extra.txt").unwrap(), b"charlie");
    }

    #[test]
    fn seven_zip_is_honestly_unsupported() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("thing.7z");
        std::fs::write(&path, b"7z\xbc\xaf\x27\x1c").unwrap();
        assert!(matches!(crate::open(&path), Err(ArchiveError::Unsupported(_))));
    }
}
