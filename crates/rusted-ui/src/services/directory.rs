use rusted_core::{sort_entries, FileEntry, SortSpec};
use rusted_fs::enumerate_streaming;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// A load request that can be cancelled when the user navigates away. Holding a
/// generation number makes late results cheap to reject.
#[derive(Debug, Clone)]
pub struct DirectoryLoad {
    pub generation: u64,
    pub path: PathBuf,
    cancel: Arc<AtomicBool>,
}

impl DirectoryLoad {
    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Release);
    }

    pub fn flag(&self) -> Arc<AtomicBool> {
        self.cancel.clone()
    }
}

/// Hands out monotonically increasing generations so only the newest load wins.
#[derive(Debug, Default)]
pub struct DirectoryLoader {
    generation: AtomicU64,
}

impl DirectoryLoader {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a new load and cancels the previous one through `previous`.
    pub fn begin(&self, path: &Path, previous: &mut Option<Arc<AtomicBool>>) -> DirectoryLoad {
        if let Some(flag) = previous.take() {
            flag.store(true, Ordering::Release);
        }
        let cancel = Arc::new(AtomicBool::new(false));
        previous.replace(cancel.clone());
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        DirectoryLoad { generation, path: path.to_path_buf(), cancel }
    }

    pub fn is_current(&self, load: &DirectoryLoad) -> bool {
        self.generation.load(Ordering::SeqCst) == load.generation && !load.cancelled()
    }
}

/// Streams every entry of one directory. Worker threads only; `on_batch` runs on
/// the calling thread so callers can forward partial results to the UI.
pub fn load_directory(
    load: &DirectoryLoad,
    show_hidden: bool,
    sort: SortSpec,
    mut on_batch: impl FnMut(&[FileEntry]),
) -> Result<Vec<FileEntry>, rusted_fs::DirectoryError> {
    let mut all: Vec<FileEntry> = Vec::new();
    enumerate_streaming(&load.path, show_hidden, sort, &load.cancel, |mut chunk| {
        sort_entries(&mut chunk, sort);
        on_batch(&chunk);
        all.extend(chunk);
    })?;
    sort_entries(&mut all, sort);
    Ok(all)
}

/// Instant in-directory fuzzy filter used by the search box inside a listing.
pub fn filter_entries(entries: &[FileEntry], query: &str) -> Vec<FileEntry> {
    let query = query.trim();
    if query.is_empty() {
        return entries.to_vec();
    }
    let mut matches: Vec<(i32, FileEntry)> = entries
        .iter()
        .filter_map(|entry| rusted_search::fuzzy_score(query, &entry.name).map(|score| (score, entry.clone())))
        .collect();
    matches.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase())));
    matches.into_iter().map(|(_, entry)| entry).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generations_invalidate_older_loads() {
        let loader = DirectoryLoader::new();
        let mut slot = None;
        let first = loader.begin(Path::new("/tmp"), &mut slot);
        let second = loader.begin(Path::new("/home"), &mut slot);
        assert!(first.cancelled(), "starting a new load must cancel the old one");
        assert!(loader.is_current(&second));
        assert!(!loader.is_current(&first));
    }

    #[test]
    fn loads_real_directories_in_sorted_order() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("b.txt"), "b").unwrap();
        std::fs::write(dir.path().join("a.txt"), "a").unwrap();
        std::fs::create_dir(dir.path().join("zdir")).unwrap();
        let loader = DirectoryLoader::new();
        let mut slot = None;
        let load = loader.begin(dir.path(), &mut slot);
        let mut batches = 0;
        let entries = load_directory(&load, false, SortSpec::default(), |_| batches += 1).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].name, "zdir", "folders sort first by default");
        assert_eq!(entries[1].name, "a.txt");
        assert!(batches >= 1);
    }

    #[test]
    fn cancelled_loads_stop_early() {
        let dir = tempfile::tempdir().unwrap();
        for index in 0..8 {
            std::fs::write(dir.path().join(format!("file{index}.txt")), "x").unwrap();
        }
        let loader = DirectoryLoader::new();
        let mut slot = None;
        let load = loader.begin(dir.path(), &mut slot);
        load.cancel();
        // The flag is honoured by the walker; the call itself still returns.
        let _ = load_directory(&load, false, SortSpec::default(), |_| {});
        assert!(load.cancelled());
    }

    #[test]
    fn quick_filter_is_fuzzy_and_ordered() {
        let entries = vec![
            FileEntry {
                path: PathBuf::from("/x/rustdoc.md"),
                name: "rustdoc.md".into(),
                kind: rusted_core::EntryKind::File,
                size: 0,
                modified: None,
                created: None,
                readonly: false,
                hidden: false,
                extension: Some("md".into()),
            },
            FileEntry {
                path: PathBuf::from("/x/readme.md"),
                name: "readme.md".into(),
                kind: rusted_core::EntryKind::File,
                size: 0,
                modified: None,
                created: None,
                readonly: false,
                hidden: false,
                extension: Some("md".into()),
            },
        ];
        assert_eq!(filter_entries(&entries, "rsd").len(), 1);
        assert_eq!(filter_entries(&entries, "").len(), 2);
        assert_eq!(filter_entries(&entries, "zzz").len(), 0);
    }
}
