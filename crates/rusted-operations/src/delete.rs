use crate::OperationError;
use std::{fs,path::Path};
pub fn recycle(paths:&[impl AsRef<Path>])->Result<(),OperationError>{
    if paths.is_empty(){return Ok(());}trash::delete_all(paths.iter().map(|p|p.as_ref())).map_err(|e|OperationError::Recycle(e.to_string()))
}
pub fn delete_permanent(path:&Path,confirmed:bool)->Result<(),OperationError>{
    if !confirmed{return Err(OperationError::Invalid("permanent deletion requires explicit confirmation".into()));}
    let metadata=fs::symlink_metadata(path).map_err(|e|OperationError::io(path,e))?;
    let result=if metadata.is_dir()&&!metadata.file_type().is_symlink(){fs::remove_dir_all(path)}else if metadata.permissions().readonly(){let mut permissions=metadata.permissions();permissions.set_readonly(false);fs::set_permissions(path,permissions).map_err(|e|OperationError::io(path,e))?;fs::remove_file(path)}else{fs::remove_file(path)};
    result.map_err(|e|OperationError::io(path,e))
}
#[cfg(windows)]
pub fn open_default(path:&Path)->Result<(),OperationError>{open::that(path).map_err(|e|OperationError::Invalid(e.to_string()))}
#[cfg(not(windows))]
pub fn open_default(path:&Path)->Result<(),OperationError>{open::that(path).map_err(|e|OperationError::Invalid(e.to_string()))}
