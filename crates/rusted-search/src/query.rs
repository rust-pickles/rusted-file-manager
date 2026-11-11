use chrono::{DateTime,Utc};
use serde::{Deserialize,Serialize};
use std::path::PathBuf;
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct SearchQuery{pub text:String,pub roots:Vec<PathBuf>,pub path_contains:Option<String>,pub extension:Option<String>,pub kind:Option<String>,pub min_size:Option<u64>,pub max_size:Option<u64>,pub modified_after:Option<DateTime<Utc>>,pub modified_before:Option<DateTime<Utc>>,pub limit:usize}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct SearchHit{pub path:PathBuf,pub name:String,pub size:u64,pub modified:Option<i64>,pub kind:String,pub score:i32}
