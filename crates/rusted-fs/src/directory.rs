use crate::{entry_from_metadata, long_path};
use rusted_core::{sort_entries, FileEntry, SortSpec};
use std::{fs, io, path::{Path, PathBuf}, sync::{Arc,atomic::{AtomicBool,Ordering}}};
use thiserror::Error;

pub const DIRECTORY_CHUNK_SIZE:usize=256;
#[derive(Debug,Error)]
#[error("could not read {path}: {source}")]
pub struct DirectoryError{pub path:PathBuf,#[source]pub source:io::Error}

/// Enumerates one directory without following links. The callback receives bounded batches.
pub fn enumerate_streaming<F>(path:&Path,show_hidden:bool,sort:SortSpec,cancel:&AtomicBool,mut on_chunk:F)->Result<usize,DirectoryError>
where F:FnMut(Vec<FileEntry>){
    let actual=long_path(path);
    let reader=fs::read_dir(&actual).map_err(|source|DirectoryError{path:path.into(),source})?;
    let mut count=0;let mut chunk=Vec::with_capacity(DIRECTORY_CHUNK_SIZE);
    for item in reader {
        if cancel.load(Ordering::Relaxed){break;}
        let item=match item{Ok(v)=>v,Err(_)=>continue}; // an individual inaccessible entry does not hide its siblings
        let entry_path=item.path();
        let file_type=match item.file_type(){Ok(v)=>v,Err(_)=>continue};
        let metadata=match fs::symlink_metadata(&entry_path){Ok(v)=>v,Err(_)=>continue};
        let entry=entry_from_metadata(&entry_path,&metadata,file_type);
        if !show_hidden&&entry.hidden{continue;}
        chunk.push(entry);count+=1;
        if chunk.len()==DIRECTORY_CHUNK_SIZE{sort_entries(&mut chunk,sort);on_chunk(std::mem::replace(&mut chunk,Vec::with_capacity(DIRECTORY_CHUNK_SIZE)));}
    }
    if !chunk.is_empty(){sort_entries(&mut chunk,sort);on_chunk(chunk);}
    Ok(count)
}
pub fn read_directory(path:&Path,show_hidden:bool)->Result<Vec<FileEntry>,DirectoryError>{
    let mut output=Vec::new();enumerate_streaming(path,show_hidden,SortSpec::default(),&AtomicBool::new(false),|mut c|output.append(&mut c))?;sort_entries(&mut output,SortSpec::default());Ok(output)
}
pub fn create_directory(path:&Path)->io::Result<()>{fs::create_dir(long_path(path))}
pub fn create_file(path:&Path)->io::Result<()>{fs::OpenOptions::new().write(true).create_new(true).open(long_path(path)).map(|_|())}

pub type SharedCancellation=Arc<AtomicBool>;
#[cfg(test)]
mod tests{
    use super::*;use std::io::Write;
    #[test]fn enumerates_real_entries_without_following_symlinks(){let temp=tempfile::tempdir().unwrap();fs::write(temp.path().join("b.txt"),b"hello").unwrap();fs::create_dir(temp.path().join("folder")).unwrap();let mut f=fs::File::create(temp.path().join(".hidden")).unwrap();f.write_all(b"x").unwrap();let all=read_directory(temp.path(),true).unwrap();assert_eq!(all.len(),3);assert_eq!(read_directory(temp.path(),false).unwrap().len(),2);}
}
