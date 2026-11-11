use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

pub use rusted_core::ConflictPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Copy,
    Move,
    Rename,
    Delete,
    DeletePermanent,
    Extract,
    CreateArchive,
    Duplicate,
}

impl OperationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Copy => "Copy",
            Self::Move => "Move",
            Self::Rename => "Rename",
            Self::Delete => "Move to Recycle Bin",
            Self::DeletePermanent => "Delete permanently",
            Self::Extract => "Extract",
            Self::CreateArchive => "Create archive",
            Self::Duplicate => "Duplicate",
        }
    }

    pub fn is_destructive(self) -> bool {
        matches!(self, Self::Delete | Self::DeletePermanent | Self::Move)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    Queued,
    Running,
    Paused,
    Cancelled,
    Completed,
    Failed,
}

impl OperationStatus {
    pub fn is_finished(self) -> bool {
        matches!(self, Self::Cancelled | Self::Completed | Self::Failed)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Paused => "Paused",
            Self::Cancelled => "Cancelled",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct OperationRequest {
    pub id: Uuid,
    pub kind: OperationKind,
    pub sources: Vec<PathBuf>,
    /// Directory for copy/move/duplicate; ignored by delete.
    pub destination: PathBuf,
    pub conflict: ConflictPolicy,
    /// Explicit user consent for permanent deletion or other destructive steps.
    pub confirmed: bool,
}

impl OperationRequest {
    pub fn new(kind: OperationKind, sources: Vec<PathBuf>, destination: PathBuf) -> Self {
        Self { id: Uuid::new_v4(), kind, sources, destination, conflict: ConflictPolicy::Ask, confirmed: false }
    }

    pub fn with_conflict(mut self, conflict: ConflictPolicy) -> Self {
        self.conflict = conflict;
        self
    }

    pub fn confirmed(mut self) -> Self {
        self.confirmed = true;
        self
    }

    pub fn describe(&self) -> String {
        if self.sources.len() == 1 {
            let name = self.sources[0].file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            format!("{} {}", self.kind.label(), name)
        } else {
            format!("{} {} items", self.kind.label(), self.sources.len())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationProgress {
    pub id: Uuid,
    pub kind: OperationKind,
    pub status: OperationStatus,
    pub current: Option<PathBuf>,
    pub completed_items: usize,
    pub total_items: usize,
    pub bytes_done: u64,
    pub bytes_per_second: f64,
    pub error: Option<String>,
}

impl OperationProgress {
    pub fn new(id: Uuid, kind: OperationKind, status: OperationStatus, total_items: usize) -> Self {
        Self {
            id,
            kind,
            status,
            current: None,
            completed_items: 0,
            total_items,
            bytes_done: 0,
            bytes_per_second: 0.0,
            error: None,
        }
    }

    pub fn fraction(&self) -> f64 {
        if self.total_items == 0 {
            return 0.0;
        }
        (self.completed_items as f64 / self.total_items as f64).clamp(0.0, 1.0)
    }

    pub fn speed_label(&self) -> String {
        if self.bytes_per_second <= 1.0 {
            String::new()
        } else {
            format!("{}/s", rusted_core::format_bytes(self.bytes_per_second as u64))
        }
    }

    /// Total transfer size is only known once every source has been measured,
    /// so an ETA is deliberately not fabricated from partial data.
    pub fn remaining_items(&self) -> usize {
        self.total_items.saturating_sub(self.completed_items)
    }

    pub fn summary(&self) -> String {
        match self.status {
            OperationStatus::Completed => format!("{} finished: {} items", self.kind.label(), self.completed_items),
            OperationStatus::Failed => format!("{} failed: {}", self.kind.label(), self.error.clone().unwrap_or_default()),
            OperationStatus::Cancelled => format!("{} cancelled", self.kind.label()),
            _ => format!("{} {}/{}", self.kind.label(), self.completed_items, self.total_items),
        }
    }
}

/// Cooperative pause/cancel token shared between the queue and the workers.
#[derive(Debug, Default)]
pub struct OperationControl {
    paused: AtomicBool,
    cancelled: AtomicBool,
    /// Monotonic bytes counter so progress can be sampled without locks.
    bytes: AtomicU64,
}

impl OperationControl {
    pub fn pause(&self) {
        self.paused.store(true, Ordering::Release);
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::Release);
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        self.paused.store(false, Ordering::Release);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Acquire)
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub fn add_bytes(&self, bytes: u64) {
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn bytes(&self) -> u64 {
        self.bytes.load(Ordering::Relaxed)
    }

    /// Blocks while paused and fails fast when cancelled.
    pub fn checkpoint(&self) -> Result<(), OperationError> {
        loop {
            if self.is_cancelled() {
                return Err(OperationError::Cancelled);
            }
            if !self.is_paused() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(40));
        }
    }
}

pub type SharedControl = Arc<OperationControl>;

#[derive(Debug, Error)]
pub enum OperationError {
    #[error("operation cancelled")]
    Cancelled,
    #[error("destination already exists: {0}")]
    Conflict(PathBuf),
    #[error("source does not exist: {0}")]
    MissingSource(PathBuf),
    #[error("cannot copy a folder into itself: {0}")]
    NestedDestination(PathBuf),
    #[error("I/O failed for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Recycle Bin operation failed: {0}")]
    Recycle(String),
    #[error("operation is not safely reversible: {0}")]
    NotReversible(String),
    #[error("invalid request: {0}")]
    Invalid(String),
    #[error("copy verification failed for {0}")]
    VerificationFailed(PathBuf),
    #[error("permanent deletion requires explicit confirmation")]
    ConfirmationRequired,
}

impl OperationError {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io { path: path.into(), source }
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_pauses_and_cancels_cooperatively() {
        let control = Arc::new(OperationControl::default());
        assert!(control.checkpoint().is_ok());
        control.pause();
        let waiter = control.clone();
        let handle = std::thread::spawn(move || waiter.checkpoint());
        std::thread::sleep(Duration::from_millis(80));
        assert!(!handle.is_finished(), "paused worker must not pass its checkpoint");
        control.resume();
        assert!(handle.join().unwrap().is_ok());
        control.cancel();
        assert!(matches!(control.checkpoint(), Err(OperationError::Cancelled)));
    }

    #[test]
    fn progress_fraction_is_bounded_and_labelled() {
        let mut progress = OperationProgress::new(Uuid::new_v4(), OperationKind::Copy, OperationStatus::Running, 4);
        progress.completed_items = 2;
        assert!((progress.fraction() - 0.5).abs() < f64::EPSILON);
        progress.completed_items = 9;
        assert!((progress.fraction() - 1.0).abs() < f64::EPSILON);
        progress.bytes_per_second = 2048.0;
        assert_eq!(progress.speed_label(), "2.0 KB/s");
        progress.status = OperationStatus::Completed;
        assert!(progress.summary().contains("finished"));
    }

    #[test]
    fn request_describe_pluralises() {
        let single = OperationRequest::new(OperationKind::Copy, vec![PathBuf::from("/a.txt")], PathBuf::from("/d"));
        assert!(single.describe().contains("a.txt"));
        let many = OperationRequest::new(OperationKind::Delete, vec![PathBuf::from("/a"), PathBuf::from("/b")], PathBuf::new());
        assert!(many.describe().contains("2 items"));
    }
}
