use crate::OperationError;
use chrono::{DateTime,Utc};
use serde::{Deserialize,Serialize};
use std::{fs,path::PathBuf};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum JournalEntry{Rename{from:PathBuf,to:PathBuf},Move{from:PathBuf,to:PathBuf},CreateFolder{path:PathBuf},CreateFile{path:PathBuf}}
impl JournalEntry{
    pub fn undo(&self)->Result<(),OperationError>{match self{
        Self::Rename{from,to}|Self::Move{from,to}=>{if !to.exists(){return Err(OperationError::NotReversible(format!("{} no longer exists",to.display())));}if from.exists(){return Err(OperationError::Conflict(from.clone()));}if let Some(parent)=from.parent(){fs::create_dir_all(parent).map_err(|e|OperationError::io(parent,e))?;}fs::rename(to,from).map_err(|e|OperationError::io(to,e))},
        Self::CreateFolder{path}=>{if fs::read_dir(path).map_err(|e|OperationError::io(path,e))?.next().is_some(){return Err(OperationError::NotReversible("created folder is no longer empty".into()));}fs::remove_dir(path).map_err(|e|OperationError::io(path,e))},
        Self::CreateFile{path}=>fs::remove_file(path).map_err(|e|OperationError::io(path,e)),
    }}
}
#[derive(Debug,Clone,Serialize,Deserialize)]pub struct DatedJournalEntry{pub at:DateTime<Utc>,pub entry:JournalEntry}
#[derive(Default)]pub struct UndoStack{undo:Vec<JournalEntry>,redo:Vec<JournalEntry>}
impl UndoStack{
    pub fn record(&mut self,entry:JournalEntry){self.undo.push(entry);self.redo.clear();if self.undo.len()>128{self.undo.remove(0);}}
    pub fn undo(&mut self)->Result<(),OperationError>{let entry=self.undo.pop().ok_or_else(||OperationError::NotReversible("nothing to undo".into()))?;entry.undo()?;self.redo.push(entry);Ok(())}
    pub fn redo(&mut self)->Result<(),OperationError>{let entry=self.redo.pop().ok_or_else(||OperationError::NotReversible("nothing to redo".into()))?;match &entry{JournalEntry::Rename{from,to}|JournalEntry::Move{from,to}=>fs::rename(from,to).map_err(|e|OperationError::io(from,e))?,JournalEntry::CreateFolder{path}=>fs::create_dir(path).map_err(|e|OperationError::io(path,e))?,JournalEntry::CreateFile{path}=>{fs::File::create_new(path).map_err(|e|OperationError::io(path,e))?;}}self.undo.push(entry);Ok(())}
}
