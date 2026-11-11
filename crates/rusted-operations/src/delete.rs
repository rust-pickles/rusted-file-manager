use crate::OperationError;
use std::fs;
use std::path::Path;

/// Sends items to the OS Recycle Bin (Windows) or trash (other hosts).
/// This is the default delete path; it is reversible by the user.
pub fn recycle(paths: &[impl AsRef<Path>]) -> Result<usize, OperationError> {
    let mut done = 0;
    for path in paths {
        let path = path.as_ref();
        if fs::symlink_metadata(path).is_err() {
            return Err(OperationError::MissingSource(path.into()));
        }
        trash::delete(path).map_err(|error| OperationError::Recycle(error.to_string()))?;
        done += 1;
    }
    Ok(done)
}

/// Irreversible deletion. Requires an explicit confirmation flag from the UI so
/// no code path can trigger it by accident.
pub fn delete_permanent(path: &Path, confirmed: bool) -> Result<(), OperationError> {
    if !confirmed {
        return Err(OperationError::ConfirmationRequired);
    }
    let metadata = fs::symlink_metadata(rusted_fs::long_path(path)).map_err(|error| OperationError::io(path, error))?;
    let result = if metadata.is_dir() && !metadata.file_type().is_symlink() {
        remove_tree(path)
    } else if metadata.permissions().readonly() {
        let mut permissions = metadata.permissions();
        permissions.set_readonly(false);
        fs::set_permissions(rusted_fs::long_path(path), permissions).map_err(|error| OperationError::io(path, error))?;
        fs::remove_file(rusted_fs::long_path(path)).map_err(|error| OperationError::io(path, error))
    } else {
        fs::remove_file(rusted_fs::long_path(path)).map_err(|error| OperationError::io(path, error))
    };
    result
}

fn remove_tree(path: &Path) -> Result<(), OperationError> {
    // Read-only children would otherwise abort remove_dir_all on Windows.
    if let Ok(reader) = fs::read_dir(rusted_fs::long_path(path)) {
        for child in reader.flatten() {
            let child_path = child.path();
            if let Ok(metadata) = fs::symlink_metadata(&child_path) {
                if metadata.is_dir() && !metadata.file_type().is_symlink() {
                    remove_tree(&child_path)?;
                    continue;
                }
                if metadata.permissions().readonly() {
                    let mut permissions = metadata.permissions();
                    permissions.set_readonly(false);
                    let _ = fs::set_permissions(&child_path, permissions);
                }
            }
            if fs::remove_file(&child_path).is_err() {
                // Directories that appeared between listing and removal.
                let _ = fs::remove_dir(&child_path);
            }
        }
    }
    match fs::remove_dir(rusted_fs::long_path(path)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(OperationError::io(path, error)),
    }
}

/// Opens the item with the user's default application.
pub fn open_default(path: &Path) -> Result<(), OperationError> {
    open::that(path).map_err(|error| OperationError::Invalid(error.to_string()))
}

/// Reveals the item in Windows File Explorer (or the host file manager).
pub fn reveal(path: &Path) -> Result<(), OperationError> {
    #[cfg(windows)]
    {
        // /select, requires the path to already exist.
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()
            .map_err(|error| OperationError::io(path, error))?;
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg("-R").arg(path).spawn().map_err(|error| OperationError::io(path, error))?;
        return Ok(());
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let parent = path.parent().unwrap_or(path);
        std::process::Command::new("xdg-open").arg(parent).spawn().map_err(|error| OperationError::io(path, error))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn permanent_delete_requires_confirmation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("victim.txt");
        std::fs::write(&path, b"x").unwrap();
        assert!(matches!(delete_permanent(&path, false), Err(OperationError::ConfirmationRequired)));
        assert!(path.exists());
    }

    #[test]
    fn permanent_delete_removes_files_and_non_empty_folders() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("tree");
        std::fs::create_dir_all(folder.join("nested")).unwrap();
        std::fs::write(folder.join("nested/one.txt"), b"1").unwrap();
        let file = dir.path().join("single.txt");
        std::fs::write(&file, b"x").unwrap();
        delete_permanent(&file, true).unwrap();
        assert!(!file.exists());
        delete_permanent(&folder, true).unwrap();
        assert!(!folder.exists());
    }

    #[test]
    fn read_only_files_are_still_deletable_after_confirmation() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("locked.txt");
        std::fs::write(&file, b"x").unwrap();
        let mut permissions = std::fs::metadata(&file).unwrap().permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&file, permissions).unwrap();
        delete_permanent(&file, true).unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn missing_sources_are_reported() {
        let missing = PathBuf::from("/definitely/missing/file.txt");
        assert!(matches!(recycle(&[missing]), Err(OperationError::MissingSource(_))));
    }
}
