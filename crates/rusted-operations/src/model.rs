use serde::{Deserialize,Serialize};
use std::{path::PathBuf,sync::{Arc,atomic::{AtomicBool,Ordering}}};
use uuid::Uuid;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum ConflictPolicy{Replace,Skip,AutoRename,Ask}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum OperationKind{Copy,Move,Rename,Delete,Extract,CreateArchive}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum OperationStatus{Queued,Running,Paused,Cancelling,Cancelled,Completed,Failed}
#[derive(Debug,Clone)]
pub struct OperationRequest{pub id:Uuid,pub kind:OperationKind,pub sources:Vec<PathBuf>,pub destination:PathBuf,pub conflict:ConflictPolicy}
impl OperationRequest{pub fn new(kind:OperationKind,sources:Vec<PathBuf>,destination:PathBuf)->Self{Self{id:Uuid::new_v4(),kind,sources,destination,conflict:ConflictPolicy::Ask}}}
#[derive(Debug,Clone)]
pub struct OperationProgress{pub id:Uuid,pub kind:OperationKind,pub status:OperationStatus,pub current:Option<PathBuf>,pub completed_files:usize,pub total_files:Option<usize>,pub bytes_done:u64,pub total_bytes:Option<u64>,pub bytes_per_second:Option<f64>,pub error:Option<String>}
impl OperationProgress{pub fn fraction(&self)->Option<f64>{self.total_bytes.filter(|b|*b>0).map(|b|(self.bytes_done as f64/b as f64).clamp(0.0,1.0))}}

#[derive(Debug,Default)]
pub struct OperationControl{paused:AtomicBool,cancelled:AtomicBool}
impl OperationControl{
    pub fn pause(&self){self.paused.store(true,Ordering::Release)}pub fn resume(&self){self.paused.store(false,Ordering::Release)}pub fn cancel(&self){self.cancelled.store(true,Ordering::Release);self.resume()}
    pub fn is_cancelled(&self)->bool{self.cancelled.load(Ordering::Acquire)}
    pub fn checkpoint(&self)->Result<(),OperationError>{while self.paused.load(Ordering::Acquire){if self.is_cancelled(){return Err(OperationError::Cancelled)}std::thread::park_timeout(std::time::Duration::from_millis(50));}if self.is_cancelled(){Err(OperationError::Cancelled)}else{Ok(())}}
}
pub type SharedControl=Arc<OperationControl>;
#[derive(Debug,thiserror::Error)]
pub enum OperationError{#[error("operation cancelled")]Cancelled,#[error("destination exists: {0}")]Conflict(PathBuf),#[error("source does not exist: {0}")]MissingSource(PathBuf),#[error("cannot copy a directory into itself: {0}")]NestedDestination(PathBuf),#[error("I/O failed for {path}: {source}")]Io{path:PathBuf,#[source]source:std::io::Error},#[error("recycle operation failed: {0}")]Recycle(String),#[error("operation is not safely reversible: {0}")]NotReversible(String),#[error("invalid request: {0}")]Invalid(String)}
impl OperationError{pub fn io(path:impl Into<PathBuf>,source:std::io::Error)->Self{Self::Io{path:path.into(),source}}}
