//! Background services the UI depends on: directory streaming with cancellation,
//! thumbnails, previews and the search index. All work happens off the UI thread.

pub mod directory;
pub mod previews;
pub mod search;
pub mod thumbnails;

pub use directory::{DirectoryLoad, DirectoryLoader};
pub use previews::{PreviewCache, PreviewOutcome};
pub use search::SearchService;
pub use thumbnails::ThumbnailCache;
