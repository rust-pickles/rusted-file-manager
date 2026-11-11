use lru::LruCache;
use rusted_core::{FileEntry, Settings};
use rusted_preview::{preview, Preview, PreviewLimits};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Result of one preview request. Failures are carried, never swallowed, so the
/// panel can say *why* something cannot be previewed.
#[derive(Debug, Clone)]
pub struct PreviewOutcome {
    pub entry: FileEntry,
    pub result: Result<Preview, String>,
}

pub struct PreviewCache {
    cache: LruCache<PathBuf, Arc<PreviewOutcome>>,
    limits: PreviewLimits,
}

impl PreviewCache {
    pub fn new(entries: usize, settings: &Settings) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(entries.max(1)).expect("non-zero capacity")),
            limits: PreviewLimits::from_settings(settings),
        }
    }

    pub fn limits(&self) -> PreviewLimits {
        self.limits
    }

    pub fn update_settings(&mut self, settings: &Settings) {
        self.limits = PreviewLimits::from_settings(settings);
        self.cache.clear();
    }

    pub fn get(&mut self, path: &Path) -> Option<Arc<PreviewOutcome>> {
        self.cache.get(path).cloned()
    }

    /// Synchronous; call from a worker thread. Caches failures too, so an
    /// unsupported file is not re-read on every frame.
    pub fn load(&mut self, entry: &FileEntry) -> Arc<PreviewOutcome> {
        if let Some(cached) = self.get(&entry.path) {
            return cached;
        }
        let result = preview(entry, self.limits).map_err(|error| error.to_string());
        let outcome = Arc::new(PreviewOutcome { entry: entry.clone(), result });
        self.cache.put(entry.path.clone(), outcome.clone());
        outcome
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &Path) -> FileEntry {
        let metadata = std::fs::metadata(path).unwrap();
        FileEntry {
            path: path.to_path_buf(),
            name: path.file_name().unwrap().to_string_lossy().into_owned(),
            kind: rusted_core::EntryKind::File,
            size: metadata.len(),
            modified: None,
            created: None,
            readonly: false,
            hidden: false,
            extension: path.extension().map(|value| value.to_string_lossy().to_lowercase()),
        }
    }

    #[test]
    fn previews_are_cached_and_settings_clear_them() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("readme.md");
        std::fs::write(&path, "# title").unwrap();
        let mut cache = PreviewCache::new(4, &Settings::default());
        let first = cache.load(&entry(&path));
        assert!(matches!(first.result, Ok(Preview::Text(_))));
        assert!(cache.get(&path).is_some());
        cache.update_settings(&Settings::default());
        assert!(cache.get(&path).is_none());
    }

    #[test]
    fn unsupported_files_report_a_reason() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("thing.bin");
        std::fs::write(&path, [0u8, 1, 2, 3]).unwrap();
        let mut cache = PreviewCache::new(4, &Settings::default());
        let outcome = cache.load(&entry(&path));
        match outcome.result {
            Ok(Preview::Unsupported { reason, .. }) => assert!(!reason.is_empty()),
            // A binary blob may also be classified as details-only; either way it
            // must not pretend to be text.
            Ok(Preview::Details { .. }) => {}
            other => panic!("expected an honest non-text preview, got {other:?}"),
        }
    }
}
