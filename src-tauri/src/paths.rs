//! Path translation between the webview and the filesystem.
//!
//! The React frontend is written against a *virtual* path space: its sidebar
//! hard-codes `/Home/Desktop`, `/Home/Documents`, `/` and so on, and it splits
//! those strings on `/` to build breadcrumbs. Real Windows paths look nothing
//! like that (`C:\Users\dev\Desktop`).
//!
//! Rather than rewrite the frontend, the shell translates. Rules:
//!
//! * Virtual roots: `/` (all drives), `/Home` (the profile folder), plus one
//!   `/Home/<Name>` per known folder.
//! * Children of a real directory are addressed as `<virtual parent>/<name>`,
//!   so navigation is uniform down the whole tree.
//! * A real path is accepted anywhere a virtual one is, which lets the address
//!   bar take `C:\Users\dev` verbatim.
//!
//! Every function here is pure string work except `to_real`, which touches the
//! disk only to resolve known folders. That keeps it unit-testable on any host.

use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// A navigable location: the virtual path the UI shows and the real path on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    /// What the address bar and breadcrumbs display, always `/`-separated.
    pub virtual_path: String,
    /// Absolute path on disk, or None for a purely virtual node such as `/`.
    pub real_path: Option<String>,
}

/// The virtual root. Listing it yields the drives.
pub const VIRTUAL_ROOT: &str = "/";
/// The profile folder (`C:\Users\<name>`), shown as "Home" in the sidebar.
pub const VIRTUAL_HOME: &str = "/Home";

/// Sub-folders under `/Home`, with the backend's `KnownFolder` used to resolve
/// each one. Order matters: it is the order the sidebar renders.
const HOME_FOLDERS: [(&str, rusted_fs::KnownFolder); 7] = [
    ("Desktop", rusted_fs::KnownFolder::Desktop),
    ("Downloads", rusted_fs::KnownFolder::Downloads),
    ("Documents", rusted_fs::KnownFolder::Documents),
    ("Pictures", rusted_fs::KnownFolder::Pictures),
    ("Videos", rusted_fs::KnownFolder::Videos),
    ("Music", rusted_fs::KnownFolder::Music),
    ("Home", rusted_fs::KnownFolder::Home),
];

/// True when the string looks like a native absolute path (`C:\...`, `C:/...`,
/// `\\server\share`, or a POSIX `/...` that is not one of our virtual roots).
pub fn is_native_path(text: &str) -> bool {
    let bytes = text.as_bytes();
    if text.starts_with("\\\\") {
        return true;
    }
    if bytes.len() >= 3 && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/') && bytes[0].is_ascii_alphabetic() {
        return true;
    }
    false
}

/// The profile folder the virtual `/Home` points at.
pub fn home_directory() -> Option<PathBuf> {
    rusted_fs::KnownFolder::Home.path()
}

/// Children of `/`: the drives, as virtual locations.
pub fn drive_locations() -> Vec<Location> {
    rusted_fs::drives()
        .into_iter()
        .map(|drive| {
            let real = drive.root.to_string_lossy().into_owned();
            Location { virtual_path: real.clone(), real_path: Some(real) }
        })
        .collect()
}

/// Resolves a virtual path (or a native one) to a real path on disk.
///
/// Returns `None` for nodes that have no single on-disk counterpart (`/`,
/// `/Home` when no profile directory can be found) and for known-folder names
/// that are not present on this machine.
pub fn to_real(virtual_path: &str) -> Option<PathBuf> {
    let trimmed = virtual_path.trim();
    if trimmed.is_empty() || trimmed == VIRTUAL_ROOT {
        return None;
    }
    if is_native_path(trimmed) {
        return Some(PathBuf::from(trimmed));
    }
    let normalised = trimmed.replace('\\', "/");
    let stripped = normalised.trim_start_matches('/');
    let mut parts = stripped.split('/').filter(|part| !part.is_empty());
    let head = parts.next()?;

    if head.eq_ignore_ascii_case("Home") {
        let rest: Vec<&str> = parts.collect();
        if rest.is_empty() {
            return home_directory();
        }
        let first = rest[0];
        // "/Home/Documents" maps through the known folder, because Windows may
        // have redirected Documents onto another drive.
        if let Some((_, folder)) = HOME_FOLDERS
            .iter()
            .find(|(name, folder)| name.eq_ignore_ascii_case(first) && *folder != rusted_fs::KnownFolder::Home)
        {
            let base = folder.path()?;
            let mut target = base;
            for part in &rest[1..] {
                target.push(part);
            }
            return Some(target);
        }
        // Anything else under /Home is a real child of the profile directory.
        let mut target = home_directory()?;
        for part in rest {
            target.push(part);
        }
        return Some(target);
    }

    // A bare drive letter such as "C:" or "C:/".
    if head.len() == 2 && head.as_bytes()[1] == b':' {
        let mut target = PathBuf::from(format!("{}:\\", &head[..1]));
        for part in parts {
            target.push(part);
        }
        return Some(target);
    }
    None
}

/// Converts a real path into the virtual path the UI should display.
///
/// Paths inside the profile folder become `/Home/...` so breadcrumbs stay
/// short; everything else keeps its native spelling.
pub fn to_virtual(real: &Path) -> String {
    // Known folders win over the plain path: OneDrive relocates Pictures to
    // `OneDrive\Pictures`, and showing that as "/Home/OneDrive/Pictures" would
    // disagree with the sidebar entry that produced it. Resolving through the
    // known folder keeps the two spellings identical, so navigation round-trips.
    for (name, folder) in HOME_FOLDERS.iter() {
        if *folder == rusted_fs::KnownFolder::Home {
            continue;
        }
        if let Some(base) = folder.path() {
            if let Ok(rest) = real.strip_prefix(&base) {
                let mut text = format!("{VIRTUAL_HOME}/{name}");
                for part in rest.iter() {
                    text.push('/');
                    text.push_str(&part.to_string_lossy());
                }
                return text;
            }
        }
    }

    if let Some(home) = home_directory() {
        if let Ok(rest) = real.strip_prefix(&home) {
            let mut text = String::from(VIRTUAL_HOME);
            for part in rest.iter() {
                text.push('/');
                text.push_str(&part.to_string_lossy());
            }
            return text;
        }
    }
    // Drive roots render as "C:/" so they join cleanly with children.
    let text = real.to_string_lossy().replace('\\', "/");
    let trimmed = text.trim_end_matches('/');
    if trimmed.is_empty() {
        return text;
    }
    // "C:" -> "C:/" keeps the root navigable as a virtual path.
    if trimmed.len() == 2 && trimmed.as_bytes()[1] == b':' {
        return format!("{trimmed}/");
    }
    trimmed.to_string()
}

/// Joins a child name onto a virtual parent path.
pub fn child_of(parent: &str, name: &str) -> String {
    let base = parent.trim_end_matches('/');
    if base.is_empty() {
        format!("/{name}")
    } else {
        format!("{base}/{name}")
    }
}

/// Rejects names that could escape their parent when turned into a path.
///
/// The frontend never sends these, but IPC is a trust boundary: a compromised
/// or buggy webview must not be able to write outside the folder it named.
pub fn validate_new_name(name: &str) -> Result<&str, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("a name is required".into());
    }
    if trimmed == "." || trimmed == ".." {
        return Err("'.' and '..' are not valid names".into());
    }
    if trimmed.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
        return Err(format!("'{trimmed}' contains a character Windows forbids in file names"));
    }
    if trimmed.chars().any(|ch| ch.is_control()) {
        return Err("control characters are not allowed in names".into());
    }
    // Windows silently strips trailing dots and spaces, which would make the
    // created name differ from the one the user asked for.
    if trimmed.ends_with('.') || trimmed.ends_with(' ') {
        return Err("names cannot end with a dot or a space on Windows".into());
    }
    const RESERVED: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let stem = trimmed.split('.').next().unwrap_or(trimmed);
    if RESERVED.iter().any(|reserved| reserved.eq_ignore_ascii_case(stem)) {
        return Err(format!("'{trimmed}' is a reserved Windows device name"));
    }
    Ok(trimmed)
}

/// Normalises an absolute path without touching the filesystem.
pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// One sidebar entry: the virtual path the UI navigates to, and its label.
#[derive(Debug, Clone, serde::Serialize)]
pub struct QuickAccess {
    pub name: String,
    pub virtual_path: String,
    pub real_path: String,
}

/// The known folders that exist here, in sidebar order.
///
/// Resolution is delegated to `KnownFolder::path`, which asks the shell, so a
/// redirected folder is reported with its real location rather than a guess.
pub fn quick_access() -> Vec<QuickAccess> {
    let mut entries = Vec::new();
    for (name, folder) in HOME_FOLDERS.iter() {
        if *folder == rusted_fs::KnownFolder::Home {
            continue;
        }
        if let Some(real) = folder.path() {
            entries.push(QuickAccess {
                name: (*name).to_string(),
                virtual_path: format!("{VIRTUAL_HOME}/{name}"),
                real_path: real.to_string_lossy().into_owned(),
            });
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_paths_are_recognised() {
        assert!(is_native_path("C:\\Users\\dev"));
        assert!(is_native_path("C:/Users/dev"));
        assert!(is_native_path("\\\\server\\share"));
        assert!(!is_native_path("/Home/Documents"));
        assert!(!is_native_path("/"));
        assert!(!is_native_path("Desktop"));
    }

    #[test]
    fn home_children_are_joined() {
        assert_eq!(child_of("/Home", "Desktop"), "/Home/Desktop");
        assert_eq!(child_of("C:/Users/dev/", "x.txt"), "C:/Users/dev/x.txt");
        assert_eq!(child_of("/", "Home"), "/Home");
        // Root children must not produce a double slash.
        assert_eq!(child_of("/Home/", "Documents"), "/Home/Documents");
    }

    #[test]
    fn names_that_could_escape_are_rejected() {
        assert!(validate_new_name("report.txt").is_ok());
        assert!(validate_new_name(" spaced name .txt ").is_ok());
        for bad in ["", "   ", ".", "..", "a/b", "a\\b", "C:evil", "a<b", "a|b", "a?b", "a*b", "con", "NUL.txt", "trailing."] {
            assert!(validate_new_name(bad).is_err(), "should reject {bad:?}");
        }
        assert!(validate_new_name("line\nbreak").is_err());

        // Surrounding whitespace is trimmed rather than rejected, matching what
        // Windows does to names anyway. The caller must use the returned value,
        // which is the trimmed one, so the name on disk matches what is reported.
        assert_eq!(validate_new_name("  report.txt  ").unwrap(), "report.txt");
        assert_eq!(validate_new_name("trailing ").unwrap(), "trailing");
    }

    #[test]
    fn virtual_root_has_no_real_path() {
        assert!(to_real("/").is_none());
        assert!(to_real("").is_none());
    }

    #[test]
    fn native_input_is_passed_through() {
        assert_eq!(to_real("C:\\Users\\dev\\x").unwrap(), PathBuf::from("C:\\Users\\dev\\x"));
    }

    #[test]
    fn bare_drives_resolve_to_their_root() {
        let resolved = to_real("C:/").unwrap();
        assert_eq!(resolved, PathBuf::from("C:\\"));
        let nested = to_real("C:/Users").unwrap();
        assert!(nested.ends_with("Users"));
    }

    #[test]
    fn unknown_virtual_roots_do_not_resolve() {
        // A bogus top-level segment must not be invented into a path.
        assert!(to_real("/DefinitelyNotAThing/x").is_none());
    }

    #[test]
    fn known_folders_round_trip_through_their_own_name() {
        // A redirected known folder must still spell as "/Home/<Name>", or the
        // sidebar entry and the breadcrumb disagree after one navigation.
        for (name, folder) in HOME_FOLDERS.iter() {
            if *folder == rusted_fs::KnownFolder::Home {
                continue;
            }
            let Some(real) = folder.path() else { continue };
            let virtual_path = to_virtual(&real);
            assert_eq!(virtual_path, format!("/Home/{name}"), "{real:?}");
            assert_eq!(to_real(&virtual_path).as_deref(), Some(real.as_path()), "{name} did not round-trip");
        }
    }

    #[test]
    fn virtual_spelling_prefers_home_relative() {
        // Only meaningful when a profile directory exists for this user.
        if let Some(home) = home_directory() {
            let child = home.join("Documents").join("notes.txt");
            let virtual_path = to_virtual(&child);
            assert!(virtual_path.starts_with("/Home/"), "{virtual_path}");
            assert!(virtual_path.ends_with("Documents/notes.txt"), "{virtual_path}");
            // And it round-trips back to the same file.
            assert_eq!(to_real(&virtual_path).unwrap(), child);
        }
    }

    #[test]
    fn drive_roots_keep_a_trailing_slash() {
        assert_eq!(to_virtual(Path::new("C:\\")), "C:/");
    }
}
