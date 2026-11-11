use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug,Error)]
pub enum PathError { #[error("path does not exist: {0}")] NotFound(PathBuf), #[error("path is not a directory: {0}")] NotDirectory(PathBuf), #[error("path contains an invalid parent traversal")] InvalidTraversal }

pub fn validate_directory(path:&Path)->Result<PathBuf,PathError>{
    if !path.exists(){return Err(PathError::NotFound(path.into()));}
    if !path.is_dir(){return Err(PathError::NotDirectory(path.into()));}
    Ok(path.to_path_buf())
}
pub fn safe_child(parent:&Path,name:&str)->Result<PathBuf,PathError>{
    let candidate=Path::new(name);
    if name.is_empty() || candidate.is_absolute() || candidate.components().any(|c|!matches!(c,Component::Normal(_))){return Err(PathError::InvalidTraversal);}
    Ok(parent.join(candidate))
}
/// Adds Windows' verbatim prefix to extended-length absolute paths.
#[cfg(windows)]
pub fn long_path(path:&Path)->PathBuf{
    use std::path::Prefix;
    let text=path.as_os_str().to_string_lossy();
    if text.starts_with(r"\\?\") || text.len()<240{return path.into();}
    match path.components().next(){Some(Component::Prefix(p))=>match p.kind(){Prefix::Disk(_)|Prefix::VerbatimDisk(_)=>PathBuf::from(format!(r"\\?\{}",text)),_=>path.into()},_=>path.into()}
}
#[cfg(not(windows))]
pub fn long_path(path:&Path)->PathBuf{path.into()}
pub fn parent(path:&Path)->Option<PathBuf>{path.parent().filter(|p|*p!=path).map(Path::to_path_buf)}

#[cfg(test)]
mod tests{use super::*;#[test]fn child_rejects_escapes(){assert!(safe_child(Path::new("/tmp"),"ok").is_ok());for bad in ["../secret","/absolute","a/b",""]{assert!(safe_child(Path::new("/tmp"),bad).is_err());}}}
