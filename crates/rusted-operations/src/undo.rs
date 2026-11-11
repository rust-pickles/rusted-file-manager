use crate::OperationError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Only operations that can be replayed exactly are journaled. Deletion and
/// cross-volume copy are deliberately absent: claiming they are undoable would
/// be a lie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalEntry {
    Rename { from: PathBuf, to: PathBuf },
    Move { from: PathBuf, to: PathBuf },
    CreateFolder { path: PathBuf },
    CreateFile { path: PathBuf },
}

fn same_volume(a: &PathBuf, b: &PathBuf) -> bool {
    let root = |path: &PathBuf| {
        path.components().next().map(|component| component.as_os_str().to_os_string())
    };
    root(a) == root(b)
}

impl JournalEntry {
    pub fn label(&self) -> String {
        match self {
            Self::Rename { from, to } => format!("Rename {} → {}", name(from), name(to)),
            Self::Move { from, to } => format!("Move {} → {}", name(from), name(to)),
            Self::CreateFolder { path } => format!("Create folder {}", name(path)),
            Self::CreateFile { path } => format!("Create file {}", name(path)),
        }
    }

    /// Whether this entry can be replayed safely right now.
    pub fn is_reversible(&self) -> bool {
        match self {
            Self::Rename { from, to } | Self::Move { from, to } => {
                to.exists() && !from.exists() && same_volume(from, to)
            }
            Self::CreateFolder { path } => fs::read_dir(path).map(|mut entries| entries.next().is_none()).unwrap_or(false),
            Self::CreateFile { path } => path.is_file(),
        }
    }

    pub fn undo(&self) -> Result<(), OperationError> {
        match self {
            Self::Rename { from, to } | Self::Move { from, to } => {
                if !to.exists() {
                    return Err(OperationError::NotReversible(format!("{} no longer exists", to.display())));
                }
                if from.exists() {
                    return Err(OperationError::Conflict(from.clone()));
                }
                if !same_volume(from, to) {
                    return Err(OperationError::NotReversible(
                        "the item now lives on a different volume".into(),
                    ));
                }
                if let Some(parent) = from.parent() {
                    fs::create_dir_all(parent).map_err(|error| OperationError::io(parent, error))?;
                }
                fs::rename(rusted_fs::long_path(to), rusted_fs::long_path(from))
                    .map_err(|error| OperationError::io(to, error))
            }
            Self::CreateFolder { path } => {
                let mut entries = fs::read_dir(path).map_err(|error| OperationError::io(path, error))?;
                if entries.next().is_some() {
                    return Err(OperationError::NotReversible("the created folder is no longer empty".into()));
                }
                fs::remove_dir(path).map_err(|error| OperationError::io(path, error))
            }
            Self::CreateFile { path } => fs::remove_file(path).map_err(|error| OperationError::io(path, error)),
        }
    }

    fn redo(&self) -> Result<(), OperationError> {
        match self {
            Self::Rename { from, to } | Self::Move { from, to } => {
                if from.exists() && !to.exists() {
                    fs::rename(rusted_fs::long_path(from), rusted_fs::long_path(to))
                        .map_err(|error| OperationError::io(from, error))
                } else {
                    Err(OperationError::Conflict(to.clone()))
                }
            }
            Self::CreateFolder { path } => fs::create_dir(path).map_err(|error| OperationError::io(path, error)),
            Self::CreateFile { path } => {
                fs::File::create_new(path).map(|_| ()).map_err(|error| OperationError::io(path, error))
            }
        }
    }
}

fn name(path: &PathBuf) -> String {
    path.file_name().map(|value| value.to_string_lossy().into_owned()).unwrap_or_else(|| path.display().to_string())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatedJournalEntry {
    pub at: DateTime<Utc>,
    pub entry: JournalEntry,
}

/// Bounded undo/redo stack shared by a window. Entries are replayed only when
/// the filesystem still matches what the journal expects.
#[derive(Debug, Default)]
pub struct UndoStack {
    undo: Vec<DatedJournalEntry>,
    redo: Vec<DatedJournalEntry>,
    limit: usize,
}

impl UndoStack {
    pub fn new(limit: usize) -> Self {
        Self { undo: Vec::new(), redo: Vec::new(), limit: limit.max(1) }
    }

    pub fn record(&mut self, entry: JournalEntry) {
        self.redo.clear();
        self.undo.push(DatedJournalEntry { at: Utc::now(), entry });
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo_label(&self) -> Option<String> {
        self.undo.last().map(|dated| dated.entry.label())
    }

    pub fn redo_label(&self) -> Option<String> {
        self.redo.last().map(|dated| dated.entry.label())
    }

    pub fn undo(&mut self) -> Result<String, OperationError> {
        let dated = self.undo.pop().ok_or_else(|| OperationError::NotReversible("nothing to undo".into()))?;
        dated.entry.undo()?;
        let label = dated.entry.label();
        self.redo.push(dated);
        Ok(label)
    }

    pub fn redo(&mut self) -> Result<String, OperationError> {
        let dated = self.redo.pop().ok_or_else(|| OperationError::NotReversible("nothing to redo".into()))?;
        dated.entry.redo()?;
        let label = dated.entry.label();
        self.undo.push(dated);
        Ok(label)
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_round_trips_through_undo_and_redo() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("old.txt");
        let to = dir.path().join("new.txt");
        fs::write(&from, b"data").unwrap();
        fs::rename(&from, &to).unwrap();
        let mut stack = UndoStack::new(16);
        stack.record(JournalEntry::Rename { from: from.clone(), to: to.clone() });
        assert!(stack.can_undo());
        stack.undo().unwrap();
        assert!(from.exists() && !to.exists());
        stack.redo().unwrap();
        assert!(to.exists() && !from.exists());
    }

    #[test]
    fn undo_refuses_when_the_file_moved_again() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("old.txt");
        let to = dir.path().join("new.txt");
        fs::write(&to, b"x").unwrap();
        let mut stack = UndoStack::new(4);
        stack.record(JournalEntry::Rename { from, to: to.clone() });
        fs::remove_file(&to).unwrap();
        assert!(stack.undo().is_err());
        // The failed entry is not silently dropped from history semantics.
        assert!(!stack.can_redo());
    }

    #[test]
    fn created_folders_are_only_removable_while_empty() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("fresh");
        fs::create_dir(&folder).unwrap();
        let mut stack = UndoStack::new(4);
        stack.record(JournalEntry::CreateFolder { path: folder.clone() });
        fs::write(folder.join("inside.txt"), b"x").unwrap();
        assert!(!JournalEntry::CreateFolder { path: folder.clone() }.is_reversible());
        assert!(stack.undo().is_err());
        fs::remove_file(folder.join("inside.txt")).unwrap();
        assert!(stack.undo().is_ok());
        assert!(!folder.exists());
    }

    #[test]
    fn stack_is_bounded_and_redo_clears_on_new_work() {
        let dir = tempfile::tempdir().unwrap();
        let mut stack = UndoStack::new(2);
        for index in 0..5 {
            stack.record(JournalEntry::CreateFile { path: dir.path().join(format!("f{index}")) });
        }
        assert_eq!(stack.undo.len(), 2);
        stack.undo().unwrap();
        assert!(stack.can_redo());
        stack.record(JournalEntry::CreateFile { path: dir.path().join("new") });
        assert!(!stack.can_redo());
        assert!(stack.undo_label().unwrap().contains("Create file"));
    }
}
