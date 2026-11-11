//! Filesystem access for RUSTED: streamed enumeration, metadata, drive
//! discovery, path validation and change notifications.
//!
//! Nothing in this crate touches UI state and nothing is executed on a UI
//! thread by design; callers run these functions on worker threads.

pub mod directory;
pub mod drives;
pub mod metadata;
pub mod paths;
pub mod watcher;

#[cfg(windows)]
pub mod windows;

pub use directory::*;
pub use drives::{drives, DriveInfo, KnownFolder};
pub use metadata::*;
pub use paths::*;
pub use watcher::*;
