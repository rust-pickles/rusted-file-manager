use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::{Path,PathBuf},sync::mpsc::{self,Receiver}};
use thiserror::Error;

#[derive(Debug,Error)]
pub enum WatchError{#[error("filesystem watcher failed: {0}")]Notify(#[from]notify::Error),#[error("watcher channel closed")]Closed}
pub struct DirectoryWatcher{_watcher:RecommendedWatcher,events:Receiver<Result<Event,notify::Error>>}
impl DirectoryWatcher{
    pub fn watch(path:&Path)->Result<Self,WatchError>{
        let(tx,rx)=mpsc::channel();let mut watcher=notify::recommended_watcher(move |event|{let _=tx.send(event);})?;
        // Non-recursive by default: indexing roots opt into recursion independently.
        watcher.watch(path,RecursiveMode::NonRecursive)?;Ok(Self{_watcher:watcher,events:rx})
    }
    pub fn try_changed_paths(&self)->Result<Vec<PathBuf>,WatchError>{
        let mut paths=Vec::new();
        while let Ok(event)=self.events.try_recv(){for path in event.map_err(WatchError::Notify)?.paths{if !paths.contains(&path){paths.push(path);}}}
        Ok(paths)
    }
}
