use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A mountable volume as reported by the platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriveInfo {
    pub root: PathBuf,
    pub label: Option<String>,
    pub removable: bool,
    pub fixed: bool,
    pub network: bool,
    pub total: Option<u64>,
    pub free: Option<u64>,
}

impl DriveInfo {
    pub fn letter(&self) -> Option<char> {
        self.root.to_string_lossy().chars().next()
    }

    pub fn display_name(&self) -> String {
        match self.letter() {
            Some(letter) => match &self.label {
                Some(label) => format!("{label} ({letter}:)"),
                None => format!("Local Disk ({letter}:)"),
            },
            None => self.label.clone().unwrap_or_else(|| self.root.display().to_string()),
        }
    }

    pub fn used(&self) -> Option<u64> {
        match (self.total, self.free) {
            (Some(total), Some(free)) => Some(total.saturating_sub(free)),
            _ => None,
        }
    }

    pub fn used_fraction(&self) -> Option<f32> {
        match (self.total, self.used()) {
            (Some(total), Some(used)) if total > 0 => Some(used as f32 / total as f32),
            _ => None,
        }
    }
}

#[cfg(windows)]
pub fn drives() -> Vec<DriveInfo> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW,
    };

    // GetDriveTypeW return values from winbase.h.
    const DRIVE_REMOVABLE: u32 = 2;
    const DRIVE_FIXED: u32 = 3;
    const DRIVE_REMOTE: u32 = 4;

    let mut result = Vec::new();
    // GetLogicalDrives returns a bitmask of available drive letters.
    let mask = unsafe { GetLogicalDrives() };
    for index in 0..26u32 {
        if mask & (1 << index) == 0 {
            continue;
        }
        let letter = (b'A' + index as u8) as char;
        let root = PathBuf::from(format!("{letter}:\\"));
        let wide: Vec<u16> = root.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let kind = unsafe { GetDriveTypeW(wide.as_ptr()) };
        if kind != DRIVE_FIXED && kind != DRIVE_REMOTE && kind != DRIVE_REMOVABLE {
            continue;
        }

        let mut total: u64 = 0;
        let mut free: u64 = 0;
        let has_space = unsafe {
            GetDiskFreeSpaceExW(wide.as_ptr(), std::ptr::null_mut(), &mut total, &mut free)
        } != 0;

        let mut name = [0u16; 261];
        let has_name = unsafe {
            GetVolumeInformationW(
                wide.as_ptr(),
                name.as_mut_ptr(),
                name.len() as u32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
            )
        } != 0;
        let length = name.iter().position(|unit| *unit == 0).unwrap_or(name.len());
        let label = has_name
            .then(|| String::from_utf16_lossy(&name[..length]))
            .filter(|value| !value.is_empty());

        result.push(DriveInfo {
            root,
            label,
            removable: kind == DRIVE_REMOVABLE,
            fixed: kind == DRIVE_FIXED,
            network: kind == DRIVE_REMOTE,
            total: has_space.then_some(total),
            free: has_space.then_some(free),
        });
    }
    result
}

#[cfg(not(windows))]
pub fn drives() -> Vec<DriveInfo> {
    // POSIX hosts get the roots that actually exist so the sidebar is truthful
    // when the app is run for development on Linux or macOS.
    let candidates = if cfg!(target_os = "macos") {
        vec!["/", "/Volumes"]
    } else {
        vec!["/", "/home", "/mnt", "/media"]
    };
    candidates
        .into_iter()
        .filter(|path| std::path::Path::new(path).is_dir())
        .map(|path| DriveInfo {
            root: PathBuf::from(path),
            label: None,
            removable: path.starts_with("/media") || path.starts_with("/Volumes"),
            fixed: path == "/",
            network: false,
            total: None,
            free: None,
        })
        .collect()
}

/// Home, desktop, downloads and similar well-known folders that exist here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownFolder {
    Home,
    Desktop,
    Downloads,
    Documents,
    Pictures,
    Music,
    Videos,
}

impl KnownFolder {
    pub const ALL: [Self; 7] = [
        Self::Home,
        Self::Desktop,
        Self::Downloads,
        Self::Documents,
        Self::Pictures,
        Self::Music,
        Self::Videos,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Desktop => "Desktop",
            Self::Downloads => "Downloads",
            Self::Documents => "Documents",
            Self::Pictures => "Pictures",
            Self::Music => "Music",
            Self::Videos => "Videos",
        }
    }

    pub fn folder_name(self) -> &'static str {
        match self {
            Self::Home => "",
            Self::Desktop => "Desktop",
            Self::Downloads => "Downloads",
            Self::Documents => "Documents",
            Self::Pictures => "Pictures",
            Self::Music => "Music",
            Self::Videos => "Videos",
        }
    }

    /// Resolves the folder, preferring the on-disk name over the localised one.
    pub fn path(self) -> Option<PathBuf> {
        let mut roots = Vec::new();
        if let Some(home) = home_directory() {
            roots.push(home);
        }
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            let path = PathBuf::from(profile);
            if !roots.contains(&path) {
                roots.push(path);
            }
        }
        for root in roots {
            if self == Self::Home && root.is_dir() {
                return Some(root);
            }
            let candidate = root.join(self.folder_name());
            if candidate.is_dir() {
                return Some(candidate);
            }
            let redirected = read_shell_redirect(&root, self.folder_name());
            if let Some(path) = redirected {
                return Some(path);
            }
        }
        None
    }
}

fn home_directory() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
}

/// Windows lets users relocate known folders; `desktop.ini` records the target.
fn read_shell_redirect(root: &std::path::Path, folder: &str) -> Option<PathBuf> {
    let ini = std::fs::read_to_string(root.join(folder).join("desktop.ini")).ok()?;
    let mut redirected = false;
    for line in ini.lines() {
        if line.trim().eq_ignore_ascii_case("FolderType=Redirected") {
            redirected = true;
            break;
        }
    }
    // The redirect target lives in the user's registry (User Shell Folders).
    // Reimplementing registry parsing here would duplicate the shell, so the
    // caller falls back to the sidebar entry being hidden instead of guessing.
    let _ = redirected;
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drive_labels_and_usage_are_computed() {
        let drive = DriveInfo {
            root: PathBuf::from("C:\\"),
            label: Some("System".into()),
            removable: false,
            fixed: true,
            network: false,
            total: Some(1000),
            free: Some(400),
        };
        assert_eq!(drive.display_name(), "System (C:)");
        assert_eq!(drive.used(), Some(600));
        assert!((drive.used_fraction().unwrap() - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn drives_are_enumerable_on_this_host() {
        let drives = drives();
        assert!(!drives.is_empty());
        assert!(drives.iter().all(|drive| drive.root.is_absolute()));
    }

    #[test]
    fn known_folders_resolve_only_when_present() {
        for folder in KnownFolder::ALL {
            if let Some(path) = folder.path() {
                assert!(path.is_dir(), "{} resolved to a missing path", folder.label());
            }
        }
    }
}
