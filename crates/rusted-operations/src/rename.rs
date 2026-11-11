use crate::OperationError;
use regex::Regex;
use serde::{Deserialize,Serialize};
use std::{collections::HashSet,fs,path::{Path,PathBuf}};

#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct BatchRenameSpec{
    pub find:String,pub replace:String,pub prefix:String,pub suffix:String,
    pub number_start:Option<u32>,pub number_width:usize,pub extension:Option<String>,
    pub regex:Option<String>,pub date_prefix:bool,pub unique_suffix:bool,
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct RenamePreview{pub source:PathBuf,pub destination:PathBuf,pub error:Option<String>}
pub fn new_name(source:&Path,index:usize,spec:&BatchRenameSpec)->Result<String,OperationError>{
    let original=source.file_name().and_then(|n|n.to_str()).ok_or_else(||OperationError::Invalid("batch rename requires Unicode file names".into()))?;
    let stem=source.file_stem().and_then(|n|n.to_str()).unwrap_or(original);
    let mut base=if let Some(pattern)=&spec.regex{Regex::new(pattern).map_err(|e|OperationError::Invalid(e.to_string()))?.replace_all(stem,spec.replace.as_str()).into_owned()}else{stem.replace(&spec.find,&spec.replace)};
    if let Some(start)=spec.number_start{base=format!("{}{:0width$}",base,start+index as u32,width=spec.number_width);}
    if spec.date_prefix{base=format!("{}-{base}",chrono::Local::now().format("%Y-%m-%d"));}
    if spec.unique_suffix{base=format!("{base}-{}",uuid::Uuid::new_v4().simple());}
    let extension=spec.extension.as_deref().map(str::trim_start_matches).map(str::trim_start_matches('.')).or_else(||source.extension().and_then(|e|e.to_str()));
    Ok(format!("{}{}{}{}{}",spec.prefix,base,spec.suffix,extension.map(|_|".").unwrap_or(""),extension.unwrap_or("")))
}
pub fn preview_batch(sources:&[PathBuf],spec:&BatchRenameSpec)->Vec<RenamePreview>{
    let mut destinations=HashSet::new();
    sources.iter().enumerate().map(|(i,source)|{
        let result=new_name(source,i,spec).and_then(|name|{
            if name.is_empty()||name.contains(['/', '\\'])||name=="."||name==".."{return Err(OperationError::Invalid("invalid output filename".into()));}
            let destination=source.with_file_name(name);
            if !destinations.insert(destination.clone()){return Err(OperationError::Conflict(destination));}
            if destination.exists()&&destination!=*source{return Err(OperationError::Conflict(destination));}
            Ok(destination)
        });
        match result{Ok(destination)=>RenamePreview{source:source.clone(),error:None,destination},Err(e)=>RenamePreview{source:source.clone(),destination:source.clone(),error:Some(e.to_string())}}
    }).collect()
}
pub fn apply_batch(previews:&[RenamePreview])->Result<(),OperationError>{
    if previews.iter().any(|p|p.error.is_some()){return Err(OperationError::Invalid("resolve all rename conflicts before applying".into()));}
    // Two-phase rename prevents cycles (a→b, b→a) from overwriting either file.
    let mut temps=Vec::new();
    for (i,item) in previews.iter().enumerate(){if item.source==item.destination{continue;}let temp=item.source.with_file_name(format!(".rusted-rename-{}-{i}",uuid::Uuid::new_v4()));fs::rename(&item.source,&temp).map_err(|e|OperationError::io(&item.source,e))?;temps.push((temp,item.destination.clone()));}
    for(temp,destination)in temps{fs::rename(&temp,&destination).map_err(|e|OperationError::io(destination,e))?;}Ok(())
}
pub fn rename_one(source:&Path,new_name:&str)->Result<PathBuf,OperationError>{let dest=rusted_fs::safe_child(source.parent().unwrap_or_else(||Path::new("")),new_name).map_err(|e|OperationError::Invalid(e.to_string()))?;if dest.exists(){return Err(OperationError::Conflict(dest));}fs::rename(rusted_fs::long_path(source),rusted_fs::long_path(&dest)).map_err(|e|OperationError::io(source,e))?;Ok(dest)}
