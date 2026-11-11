//! File operation engine: verified copy, safe move, batch rename, recycle and
//! permanent delete, an undo journal for reversible actions, and a bounded
//! worker queue that streams honest progress.

pub mod copy;
pub mod delete;
pub mod model;
pub mod queue;
pub mod rename;
pub mod undo;

pub use copy::*;
pub use delete::*;
pub use model::*;
pub use queue::*;
pub use rename::*;
pub use undo::*;
