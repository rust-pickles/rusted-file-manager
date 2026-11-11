use crate::{ConflictPolicy,OperationControl,OperationError};
use rusted_fs::long_path;
use std::{fs,io::{Read,Write},path::{Path,PathBuf},time::{SystemTime,UNIX_EPOCH}};

pub fn unique_destination(destination:&Path)->PathBuf{
    if !destination.exists(){return destination.into();}
    let parent=destination.parent().unwrap_or_else(||Path::new(""));let stem=destination.file_stem().unwrap_or_default().to_string_lossy();let ext=destination.extension().map(|e|format!(".{}",e.to_string_lossy())).unwrap_or_default();
    for n in 1..10000{let candidate=parent.join(format!("{stem} ({n}){ext}"));if !candidate.exists(){return candidate;}}
    parent.join(format!("{stem}-{}{}",std::process::id(),ext))
}
fn resolve_destination(source:&Path,destination:&Path,policy:ConflictPolicy)->Result<Option<PathBuf>,OperationError>{
    if !destination.exists(){return Ok(Some(destination.into()));}
    match policy{ConflictPolicy::Skip=>Ok(None),ConflictPolicy::AutoRename=>Ok(Some(unique_destination(destination))),ConflictPolicy::Replace=>Ok(Some(destination.into())),ConflictPolicy::Ask=>Err(OperationError::Conflict(destination.into()))}
}
fn temporary_path(destination:&Path)->PathBuf{let stamp=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();let name=destination.file_name().unwrap_or_default().to_string_lossy();destination.with_file_name(format!(".{name}.rusted-part-{}-{stamp}",std::process::id()))}

pub fn copy_file(source:&Path,destination:&Path,policy:ConflictPolicy,control:&OperationControl,mut progress:impl FnMut(u64))->Result<Option<PathBuf>,OperationError>{
    if !source.is_file(){return Err(OperationError::MissingSource(source.into()));}
    let Some(destination)=resolve_destination(source,destination,policy)?else{return Ok(None)};
    if let Some(parent)=destination.parent(){fs::create_dir_all(long_path(parent)).map_err(|e|OperationError::io(parent,e))?;}
    let temp=temporary_path(&destination);
    let result=(||{
        let mut input=fs::File::open(long_path(source)).map_err(|e|OperationError::io(source,e))?;
        let mut output=fs::OpenOptions::new().write(true).create_new(true).open(long_path(&temp)).map_err(|e|OperationError::io(&temp,e))?;
        let mut buffer=[0u8;256*1024];let mut total=0;
        loop{control.checkpoint()?;let read=input.read(&mut buffer).map_err(|e|OperationError::io(source,e))?;if read==0{break;}output.write_all(&buffer[..read]).map_err(|e|OperationError::io(&temp,e))?;total+=read as u64;progress(total);}
        output.sync_all().map_err(|e|OperationError::io(&temp,e))?;
        let expected=fs::metadata(source).map_err(|e|OperationError::io(source,e))?.len();
        if total!=expected{return Err(OperationError::Invalid(format!("incomplete copy of {}",source.display())));}
        drop(output);
        if destination.exists(){if destination.is_dir(){return Err(OperationError::Conflict(destination));}fs::remove_file(long_path(&destination)).map_err(|e|OperationError::io(&destination,e))?;}
        fs::rename(long_path(&temp),long_path(&destination)).map_err(|e|OperationError::io(&destination,e))?;
        let permissions=fs::metadata(source).map_err(|e|OperationError::io(source,e))?.permissions();fs::set_permissions(long_path(&destination),permissions).map_err(|e|OperationError::io(&destination,e))?;
        Ok(Some(destination))
    })();
    if result.is_err(){let _=fs::remove_file(long_path(&temp));}
    result
}
pub fn copy_path(source:&Path,destination:&Path,policy:ConflictPolicy,control:&OperationControl,progress:impl FnMut(u64))->Result<Option<PathBuf>,OperationError>{
    let meta=fs::symlink_metadata(source).map_err(|e|OperationError::io(source,e))?;
    if meta.file_type().is_symlink(){return copy_symlink(source,destination,policy);}
    if meta.is_dir(){
        if destination.starts_with(source){return Err(OperationError::NestedDestination(destination.into()));}
        let Some(destination)=resolve_destination(source,destination,policy)?else{return Ok(None)};
        if destination.exists()&&!destination.is_dir(){return Err(OperationError::Conflict(destination));}
        fs::create_dir_all(long_path(&destination)).map_err(|e|OperationError::io(&destination,e))?;
        for child in fs::read_dir(source).map_err(|e|OperationError::io(source,e))?{control.checkpoint()?;let child=child.map_err(|e|OperationError::io(source,e))?;copy_path(&child.path(),&destination.join(child.file_name()),policy,control,&progress)?;}
        Ok(Some(destination))
    }else{copy_file(source,destination,policy,control,progress)}
}
#[cfg(unix)]
fn copy_symlink(source:&Path,destination:&Path,policy:ConflictPolicy)->Result<Option<PathBuf>,OperationError>{use std::os::unix::fs::symlink;let Some(dest)=resolve_destination(source,destination,policy)?else{return Ok(None)};if dest.exists(){fs::remove_file(&dest).map_err(|e|OperationError::io(&dest,e))?;}let target=fs::read_link(source).map_err(|e|OperationError::io(source,e))?;symlink(target,&dest).map_err(|e|OperationError::io(&dest,e))?;Ok(Some(dest))}
#[cfg(windows)]
fn copy_symlink(source:&Path,destination:&Path,policy:ConflictPolicy)->Result<Option<PathBuf>,OperationError>{use std::os::windows::fs::{symlink_dir,symlink_file};let Some(dest)=resolve_destination(source,destination,policy)?else{return Ok(None)};if dest.exists(){if dest.is_dir(){fs::remove_dir(&dest)}else{fs::remove_file(&dest)}.map_err(|e|OperationError::io(&dest,e))?;}let target=fs::read_link(source).map_err(|e|OperationError::io(source,e))?;if source.is_dir(){symlink_dir(target,&dest)}else{symlink_file(target,&dest)}.map_err(|e|OperationError::io(&dest,e))?;Ok(Some(dest))}

pub fn move_path(source:&Path,destination:&Path,policy:ConflictPolicy,control:&OperationControl,progress:impl FnMut(u64))->Result<Option<PathBuf>,OperationError>{
    if !source.exists()&&!fs::symlink_metadata(source).is_ok(){return Err(OperationError::MissingSource(source.into()));}
    match fs::rename(long_path(source),long_path(&destination)){
        Ok(())=>Ok(Some(destination.into())),
        Err(rename_error)=>{
            // Cross-device moves fall back to verified copy. Other failures may also be recoverable by copy;
            // the original is removed only after the complete copy succeeds.
            let Some(actual)=copy_path(source,&destination,policy,control,progress)?else{return Ok(None)};
            let metadata=fs::symlink_metadata(source).map_err(|e|OperationError::io(source,e))?;
            let removal=if metadata.is_dir()&&!metadata.file_type().is_symlink(){fs::remove_dir_all(source)}else{fs::remove_file(source)};
            removal.map_err(|e|OperationError::Invalid(format!("copied to {}, but could not remove original {}: {e}; rename error was {rename_error}",actual.display(),source.display())))?;
            Ok(Some(actual))
        }
    }
}
