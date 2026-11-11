use lru::LruCache;
use rusted_preview::ImagePreview;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Bounded cache of decoded thumbnails. Bounded because a folder of 20k photos
/// would otherwise pin gigabytes of RGBA.
pub struct ThumbnailCache {
    cache: LruCache<PathBuf, Arc<ImagePreview>>,
    max_edge: u32,
}

impl ThumbnailCache {
    pub fn new(entries: usize, max_edge: u32) -> Self {
        let capacity = NonZeroUsize::new(entries.max(1)).expect("non-zero capacity");
        Self { cache: LruCache::new(capacity), max_edge: max_edge.clamp(32, 512) }
    }

    pub fn max_edge(&self) -> u32 {
        self.max_edge
    }

    pub fn set_max_edge(&mut self, max_edge: u32) {
        let clamped = max_edge.clamp(32, 512);
        if clamped != self.max_edge {
            self.max_edge = clamped;
            self.cache.clear();
        }
    }

    pub fn get(&mut self, path: &Path) -> Option<Arc<ImagePreview>> {
        self.cache.get(path).cloned()
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.cache.contains(path)
    }

    /// Decodes on the calling thread; returns `None` for anything undecodable so
    /// the grid falls back to a glyph instead of showing a broken tile.
    pub fn load(&mut self, path: &Path) -> Option<Arc<ImagePreview>> {
        if let Some(cached) = self.get(path) {
            return Some(cached);
        }
        let decoded = rusted_preview::decode_image(path, rusted_preview::PreviewLimits::default()).ok()?;
        let thumbnail = Arc::new(decoded.thumbnail(self.max_edge));
        self.cache.put(path.to_path_buf(), thumbnail.clone());
        Some(thumbnail)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_is_bounded_and_ignores_undecodable_files() {
        let dir = tempfile::tempdir().unwrap();
        let broken = dir.path().join("broken.png");
        std::fs::write(&broken, b"not a png").unwrap();
        let mut cache = ThumbnailCache::new(2, 64);
        assert!(cache.load(&broken).is_none());
        assert!(cache.is_empty());
        let missing = dir.path().join("missing.png");
        assert!(cache.load(&missing).is_none());
        cache.set_max_edge(96);
        assert_eq!(cache.max_edge(), 96);
        assert_eq!(ThumbnailCache::new(1, 9999).max_edge(), 512);
    }
}
