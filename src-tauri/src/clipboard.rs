//! Windows clipboard file lists (`CF_HDROP`).
//!
//! Plain text on the clipboard cannot copy files: Explorer and every other file
//! manager look for a `DROPFILES` structure holding wide path strings. Without
//! it, a copy inside RUSTED can never be pasted into Explorer, and a copy made
//! in Explorer can never be read back.
//!
//! Everything here is best-effort: a clipboard already held by another
//! application returns a plain error instead of hanging or panicking.

#![cfg(windows)]

use std::path::PathBuf;

use windows_sys::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
    SetClipboardData,
};
// GlobalFree lives in Foundation rather than Memory in windows-sys.
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::System::Ole::CF_HDROP;
use windows_sys::Win32::UI::Shell::DragQueryFileW;

/// `DROPFILES` from the Windows SDK, declared explicitly because the layout
/// matters: `f_wide` must be non-zero or the shell reads the paths as ANSI.
#[repr(C)]
struct DropFiles {
    p_files: u32,
    pt_x: i32,
    pt_y: i32,
    f_nc: i32,
    f_wide: i32,
}

/// Puts `paths` on the clipboard as a file list.
pub fn set_files(paths: &[PathBuf]) -> Result<(), String> {
    if paths.is_empty() {
        return Err("no files to copy".into());
    }

    // One UTF-16 buffer: every path NUL-terminated, then a final NUL so the end
    // of the list is unambiguous.
    let mut wide: Vec<u16> = Vec::new();
    for path in paths {
        use std::os::windows::ffi::OsStrExt;
        wide.extend(path.as_os_str().encode_wide());
        wide.push(0);
    }
    wide.push(0);

    let header = std::mem::size_of::<DropFiles>();
    let payload = header + wide.len() * std::mem::size_of::<u16>();
    let drop_files = DropFiles { p_files: header as u32, pt_x: 0, pt_y: 0, f_nc: 0, f_wide: 1 };

    // SAFETY: the block is allocated at exactly `payload` bytes and every write
    // stays inside it. On success the clipboard owns the handle and frees it; on
    // every failure path the handle is freed here.
    unsafe {
        let handle: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, payload);
        if handle.is_null() {
            return Err("could not allocate the clipboard buffer".into());
        }
        let block = GlobalLock(handle) as *mut u8;
        if block.is_null() {
            GlobalFree(handle);
            return Err("could not lock the clipboard buffer".into());
        }
        (block as *mut DropFiles).write_unaligned(drop_files);
        std::ptr::copy_nonoverlapping(wide.as_ptr(), block.add(header).cast::<u16>(), wide.len());
        GlobalUnlock(handle);

        // Another application can hold the clipboard briefly; retry instead of
        // failing on a transient lock.
        let mut opened = false;
        for _ in 0..10 {
            if OpenClipboard(std::ptr::null_mut()) != 0 {
                opened = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        if !opened {
            GlobalFree(handle);
            return Err("the clipboard is in use by another application".into());
        }

        if EmptyClipboard() == 0 {
            CloseClipboard();
            GlobalFree(handle);
            return Err("could not clear the clipboard".into());
        }
        if SetClipboardData(CF_HDROP as u32, handle as HANDLE) == std::ptr::null_mut() {
            CloseClipboard();
            GlobalFree(handle);
            return Err("could not put the files on the clipboard".into());
        }
        CloseClipboard();
    }

    Ok(())
}

/// Reads the file list currently on the clipboard; empty when there is none.
pub fn get_files() -> Result<Vec<PathBuf>, String> {
    // SAFETY: every pointer is checked before use, and the clipboard is closed
    // on every path after it has been opened.
    unsafe {
        if IsClipboardFormatAvailable(CF_HDROP as u32) == 0 {
            return Ok(Vec::new());
        }
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err("the clipboard is in use by another application".into());
        }

        let data = GetClipboardData(CF_HDROP as u32);
        if data == std::ptr::null_mut() {
            CloseClipboard();
            return Ok(Vec::new());
        }
        if GlobalLock(data as HGLOBAL).is_null() {
            CloseClipboard();
            return Ok(Vec::new());
        }

        // u32::MAX asks for the count rather than a path.
        let count = DragQueryFileW(data as _, u32::MAX, std::ptr::null_mut(), 0);
        let mut files = Vec::with_capacity(count as usize);
        for index in 0..count {
            let len = DragQueryFileW(data as _, index, std::ptr::null_mut(), 0);
            if len == 0 {
                continue;
            }
            let mut buffer = vec![0u16; len as usize + 1];
            let written = DragQueryFileW(data as _, index, buffer.as_mut_ptr(), buffer.len() as u32);
            if written == 0 {
                continue;
            }
            buffer.truncate(written as usize);
            files.push(PathBuf::from(String::from_utf16_lossy(&buffer)));
        }

        GlobalUnlock(data as HGLOBAL);
        CloseClipboard();
        Ok(files)
    }
}
