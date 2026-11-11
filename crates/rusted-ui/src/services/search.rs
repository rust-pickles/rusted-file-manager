use rusted_core::SearchFilters;
use rusted_search::{SearchError, SearchHit, SearchIndex, SearchQuery};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Thin owner of the FTS5 index. Owns the cancel flag so the UI can stop a long
/// indexing run without tearing down the index itself.
pub struct SearchService {
    index: Arc<SearchIndex>,
    cancel: Arc<AtomicBool>,
    indexing: Arc<AtomicBool>,
}

impl SearchService {
    pub fn in_memory() -> Result<Self, SearchError> {
        Ok(Self::wrap(Arc::new(SearchIndex::in_memory()?)))
    }

    pub fn open(path: &Path) -> Result<Self, SearchError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| SearchError::Io { path: parent.into(), source })?;
        }
        Ok(Self::wrap(Arc::new(SearchIndex::open(path)?)))
    }

    fn wrap(index: Arc<SearchIndex>) -> Self {
        Self { index, cancel: Arc::new(AtomicBool::new(false)), indexing: Arc::new(AtomicBool::new(false)) }
    }

    pub fn is_indexing(&self) -> bool {
        self.indexing.load(Ordering::Acquire)
    }

    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        self.cancel.clone()
    }

    pub fn cancel_indexing(&self) {
        self.cancel.store(true, Ordering::Release);
    }

    /// Indexes one root. Blocking; call from a worker thread. Clears the cancel
    /// flag first so a previous cancellation does not poison this run.
    pub fn index_root(&self, root: &Path, exclusions: &[PathBuf]) -> Result<u64, SearchError> {
        self.cancel.store(false, Ordering::Release);
        self.indexing.store(true, Ordering::Release);
        let result = self.index.index_root(root, exclusions, &self.cancel, |_stats| {});
        self.indexing.store(false, Ordering::Release);
        result.map(|stats| stats.indexed as u64)
    }

    pub fn search(&self, text: &str, filters: &SearchFilters, limit: usize) -> Result<Vec<SearchHit>, SearchError> {
        let query = SearchQuery {
            text: text.to_string(),
            roots: Vec::new(),
            path_contains: None,
            extension: (!filters.extension.trim().is_empty()).then(|| filters.extension.trim().to_lowercase()),
            kind: None,
            min_size: filters.min_size_kib.map(|kib| kib * 1024),
            max_size: filters.max_size_kib.map(|kib| kib * 1024),
            modified_after: filters.modified_within_days.map(|days| {
                chrono::Utc::now() - chrono::Duration::days(days as i64)
            }),
            modified_before: None,
            limit: limit.clamp(1, 5_000),
        };
        self.index.search(&query)
    }

    pub fn remove_path(&self, path: &Path) -> Result<(), SearchError> {
        self.index.remove_path(path)
    }

    pub fn upsert(&self, path: &Path, root: &Path) -> Result<(), SearchError> {
        self.index.upsert_path(path, root)
    }

    pub fn indexed_count(&self) -> Result<u64, SearchError> {
        self.index.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexing_and_searching_a_real_tree() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("quarterly-report.txt"), "x").unwrap();
        std::fs::create_dir(dir.path().join("nested")).unwrap();
        std::fs::write(dir.path().join("nested").join("invoice.txt"), "y").unwrap();
        let service = SearchService::in_memory().unwrap();
        let indexed = service.index_root(dir.path(), &[]).unwrap();
        assert!(indexed >= 2);
        assert!(!service.is_indexing());
        let hits = service.search("quarterly", &SearchFilters::default(), 20).unwrap();
        assert!(hits.iter().any(|hit| hit.name == "quarterly-report.txt"));
        service.remove_path(&dir.path().join("quarterly-report.txt")).unwrap();
        let hits = service.search("quarterly", &SearchFilters::default(), 20).unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn extension_filter_is_applied() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("note.md"), "x").unwrap();
        std::fs::write(dir.path().join("note.txt"), "y").unwrap();
        let service = SearchService::in_memory().unwrap();
        service.index_root(dir.path(), &[]).unwrap();
        let filters = SearchFilters { extension: "md".into(), ..SearchFilters::default() };
        let hits = service.search("note", &filters, 20).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name, "note.md");
    }
}
