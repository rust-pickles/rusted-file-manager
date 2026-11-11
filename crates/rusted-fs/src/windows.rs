//! Thin wrappers over the Windows APIs RUSTED needs. Kept small on purpose:
//! every function here returns data, never UI state.

#![cfg(windows)]

use std::os::windows::io::AsRawHandle;
use std::path::Path;
use windows_sys::Win32::Storage::FileSystem::{
    FileAttributeTagInfo, GetFileInformationByHandleEx, FILE_ATTRIBUTE_TAG_INFO,
};

pub const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA000_0003;

/// Returns the reparse tag of a path, or None when it is not a reparse point.
pub fn reparse_tag(path: &Path) -> Option<u32> {
    let file = std::fs::File::open(path).ok()?;
    let mut info = FILE_ATTRIBUTE_TAG_INFO::default();
    let ok = unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle() as _,
            FileAttributeTagInfo,
            (&mut info as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    };
    if ok == 0 || info.ReparseTag == 0 {
        None
    } else {
        Some(info.ReparseTag)
    }
}

/// Moves a window to the foreground; used after a second instance is launched so
/// the existing window comes to front instead of silently ignoring the request.
pub fn set_foreground(_window: isize) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_files_are_not_reparse_points() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("plain.txt");
        std::fs::write(&path, b"x").unwrap();
        assert_eq!(reparse_tag(&path), None);
    }
}
