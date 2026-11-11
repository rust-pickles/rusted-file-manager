use serde::{Deserialize,Serialize};
use std::path::PathBuf;
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct DriveInfo{pub root:PathBuf,pub label:Option<String>,pub removable:bool,pub total:Option<u64>,pub free:Option<u64>}

#[cfg(windows)]
pub fn drives()->Vec<DriveInfo>{
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW,GetDriveTypeW,GetLogicalDrives,GetVolumeInformationW,DRIVE_FIXED,DRIVE_REMOTE,DRIVE_REMOVABLE};
    let bits=unsafe{GetLogicalDrives()};let mut result=Vec::new();
    for i in 0..26{if bits&(1<<i)==0{continue;}let letter=(b'A'+i)as char;let root=PathBuf::from(format!("{letter}:\\"));let wide:Vec<u16>=root.as_os_str().encode_wide().chain(Some(0)).collect();let kind=unsafe{GetDriveTypeW(wide.as_ptr())};if kind!=DRIVE_FIXED&&kind!=DRIVE_REMOTE&&kind!=DRIVE_REMOVABLE{continue;}let mut total=0;let mut free=0;let has_space=unsafe{GetDiskFreeSpaceExW(wide.as_ptr(),std::ptr::null_mut(),&mut total,&mut free)}!=0;let mut name=[0u16;261];let has_name=unsafe{GetVolumeInformationW(wide.as_ptr(),name.as_mut_ptr(),name.len()as u32,std::ptr::null_mut(),std::ptr::null_mut(),std::ptr::null_mut(),std::ptr::null_mut(),0)}!=0;let len=name.iter().position(|c|*c==0).unwrap_or(name.len());result.push(DriveInfo{root,label:has_name.then(||String::from_utf16_lossy(&name[..len])).filter(|s|!s.is_empty()),removable:kind==DRIVE_REMOVABLE,total:has_space.then_some(total),free:has_space.then_some(free)});}
    result
}
#[cfg(not(windows))]
pub fn drives()->Vec<DriveInfo>{vec![DriveInfo{root:PathBuf::from("/"),label:Some("Filesystem".into()),removable:false,total:None,free:None}]}
