//! Shared, UI-independent state for RUSTED: tabs, navigation, split layouts,
//! theme tokens and user settings. Nothing here touches the filesystem.

pub mod commands;
pub mod history;
pub mod models;
pub mod split;
pub mod tabs;
pub mod theme;
pub mod workspace;

pub use commands::{Action, Keymap, Shortcut};
pub use history::NavigationHistory;
pub use models::*;
pub use split::*;
pub use tabs::Tab;
pub use theme::*;
pub use workspace::Workspace;

/// Re-exported so consumers do not have to depend on uuid directly.
pub use uuid::Uuid;
