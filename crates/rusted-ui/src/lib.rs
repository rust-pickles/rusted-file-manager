//! GPUI frontend for RUSTED.
//!
//! Layout of the crate:
//!
//! * [`theme`] bridges the palette tokens in `rusted-core` to GPUI colours and
//!   derives the density metrics every surface measures itself against.
//! * [`state`] is the UI model: workspace, per-pane entries, overlays, queue
//!   snapshots. It owns no GPUI entities, which keeps it testable.
//! * [`services`] holds the worker-thread helpers (directory streaming,
//!   thumbnails, previews, indexing).
//! * [`shell`] is the window: it renders [`state`] and dispatches every command.
//! * The remaining modules are pure render helpers used by the shell.

pub mod actions;
pub mod components;
pub mod dialogs;
pub mod format;
pub mod panels;
pub mod services;
pub mod state;
pub mod theme;
pub mod views;
pub mod workspace;
pub mod shell;
pub mod text_input;

pub use shell::RustedApp;
pub use state::{AppState, Dialog, Overlay, PanelTab};
pub use theme::Theme;
