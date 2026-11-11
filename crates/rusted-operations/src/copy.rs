use crate::{ConflictPolicy, OperationControl, OperationError};
use rusted_fs::long_path;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const COPY_BUFFER: usize = 256 * 1024;
/// Bytes between progress callbacks; keeps the UI smooth without flooding it.
const PROGRESS_STEP: u64 = 256 * 1024;

/// First free "name (n).ext" beside `destination`.
pub fn unique_destination(destination: &Path) -> PathBuf {
    if !destination.exists() {
        return destination.into();
    }
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let stem = destination.file_stem().unwrap_or_default().to_string_lossy().into_owned();
    let extension = destination
        .extension()
        .map(|value| format!(".{}", value.to_string_lossy()))
        .unwrap_or_default();
    for index in 1..10_000 {
        let candidate = parent.join(format!("{stem} ({index}){extension}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    parent.join(format!("{stem}-{}{extension}", std::process::id()))
}

/// Applies the conflict policy. `Ok(None)` means "skip this item".
pub fn resolve_destination(destination: &Path, policy: ConflictPolicy) -> Result<Option<PathBuf>, OperationError> {
    if !destination.exists() {
        return Ok(Some(destination.to_path_buf()));
    }
    match policy {
        ConflictPolicy::Skip => Ok(None),
        ConflictPolicy::AutoRename => Ok(Some(unique_destination(destination))),
        ConflictPolicy::Replace => Ok(Some(destination.to_path_buf())),
        ConflictPolicy::Ask => Err(OperationError::Conflict(destination.to_path_buf())),
    }
}

fn temporary_path(destination: &Path) -> PathBuf {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let name = destination.file_name().unwrap_or_default().to_string_lossy().into_owned();
    destination.with_file_name(format!(".{name}.rusted-part-{}-{stamp}", std::process::id()))
}

/// Copies one file through a temporary sibling, verifies the byte count, then
/// renames into place. The destination is never truncated before the copy is
/// complete, so an interrupted run cannot destroy existing data.
pub fn copy_file(
    source: &Path,
    destination: &Path,
    policy: ConflictPolicy,
    control: &OperationControl,
    progress: &mut impl FnMut(u64),
) -> Result<Option<PathBuf>, OperationError> {
    if !source.is_file() {
        return Err(OperationError::MissingSource(source.into()));
    }
    if source == destination {
        return Err(OperationError::Conflict(destination.to_path_buf()));
    }
    let Some(destination) = resolve_destination(destination, policy)? else {
        return Ok(None);
    };
    if destination == source {
        return Err(OperationError::Conflict(destination));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(long_path(parent)).map_err(|error| OperationError::io(parent, error))?;
    }
    let temporary = temporary_path(&destination);
    let result = (|| {
        let mut input = fs::File::open(long_path(source)).map_err(|error| OperationError::io(source, error))?;
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(long_path(&temporary))
            .map_err(|error| OperationError::io(&temporary, error))?;
        let mut buffer = vec![0u8; COPY_BUFFER];
        let mut total = 0u64;
        let mut reported = 0u64;
        loop {
            control.checkpoint()?;
            let read = input.read(&mut buffer).map_err(|error| OperationError::io(source, error))?;
            if read == 0 {
                break;
            }
            output
                .write_all(&buffer[..read])
                .map_err(|error| OperationError::io(&temporary, error))?;
            total += read as u64;
            control.add_bytes(read as u64);
            if total - reported >= PROGRESS_STEP {
                reported = total;
                progress(total);
            }
        }
        output.flush().map_err(|error| OperationError::io(&temporary, error))?;
        output.sync_all().map_err(|error| OperationError::io(&temporary, error))?;
        let expected = fs::metadata(long_path(source)).map_err(|error| OperationError::io(source, error))?.len();
        if total != expected {
            return Err(OperationError::VerificationFailed(source.into()));
        }
        drop(output);
        if destination.exists() {
            if destination.is_dir() {
                return Err(OperationError::Conflict(destination.clone()));
            }
            fs::remove_file(long_path(&destination)).map_err(|error| OperationError::io(&destination, error))?;
        }
        fs::rename(long_path(&temporary), long_path(&destination))
            .map_err(|error| OperationError::io(&destination, error))?;
        let permissions = fs::metadata(long_path(source)).map_err(|error| OperationError::io(source, error))?.permissions();
        fs::set_permissions(long_path(&destination), permissions)
            .map_err(|error| OperationError::io(&destination, error))?;
        progress(total);
        Ok(Some(destination))
    })();
    if result.is_err() {
        // Best-effort cleanup of a partial temporary file; failure here is not
        // allowed to mask the original error.
        let _ = fs::remove_file(long_path(&temporary));
    }
    result
}

/// Recursive copy that refuses to nest a folder inside itself and never
/// follows symlinks (they are copied as links, not as their targets).
pub fn copy_path(
    source: &Path,
    destination: &Path,
    policy: ConflictPolicy,
    control: &OperationControl,
    progress: &mut impl FnMut(u64),
) -> Result<Option<PathBuf>, OperationError> {
    let metadata = fs::symlink_metadata(long_path(source)).map_err(|error| OperationError::io(source, error))?;
    if metadata.file_type().is_symlink() {
        return copy_symlink(source, destination, policy);
    }
    if metadata.is_dir() {
        if destination.starts_with(source) {
            return Err(OperationError::NestedDestination(destination.into()));
        }
        let Some(destination) = resolve_destination(destination, policy)? else {
            return Ok(None);
        };
        if destination.exists() && !destination.is_dir() {
            return Err(OperationError::Conflict(destination));
        }
        fs::create_dir_all(long_path(&destination)).map_err(|error| OperationError::io(&destination, error))?;
        let reader = fs::read_dir(long_path(source)).map_err(|error| OperationError::io(source, error))?;
        for child in reader {
            control.checkpoint()?;
            let child = child.map_err(|error| OperationError::io(source, error))?;
            copy_path(&child.path(), &destination.join(child.file_name()), policy, control, progress)?;
        }
        Ok(Some(destination))
    } else {
        copy_file(source, destination, policy, control, progress)
    }
}

#[cfg(unix)]
fn copy_symlink(source: &Path, destination: &Path, policy: ConflictPolicy) -> Result<Option<PathBuf>, OperationError> {
    use std::os::unix::fs::symlink;
    let Some(destination) = resolve_destination(destination, policy)? else {
        return Ok(None);
    };
    if destination.exists() {
        fs::remove_file(&destination).map_err(|error| OperationError::io(&destination, error))?;
    }
    let target = fs::read_link(source).map_err(|error| OperationError::io(source, error))?;
    symlink(target, &destination).map_err(|error| OperationError::io(&destination, error))?;
    Ok(Some(destination))
}

#[cfg(windows)]
fn copy_symlink(source: &Path, destination: &Path, policy: ConflictPolicy) -> Result<Option<PathBuf>, OperationError> {
    use std::os::windows::fs::{symlink_dir, symlink_file};
    let Some(destination) = resolve_destination(destination, policy)? else {
        return Ok(None);
    };
    if destination.exists() {
        if destination.is_dir() { fs::remove_dir(&destination) } else { fs::remove_file(&destination) }
            .map_err(|error| OperationError::io(&destination, error))?;
    }
    let target = fs::read_link(source).map_err(|error| OperationError::io(source, error))?;
    let is_dir = fs::metadata(source).map(|metadata| metadata.is_dir()).unwrap_or(false);
    if is_dir {
        symlink_dir(target, &destination)
    } else {
        symlink_file(target, &destination)
    }
    .map_err(|error| OperationError::io(&destination, error))?;
    Ok(Some(destination))
}

/// Same-volume moves use a single atomic rename; cross-volume moves fall back to
/// a verified copy and only then remove the original.
pub fn move_path(
    source: &Path,
    destination: &Path,
    policy: ConflictPolicy,
    control: &OperationControl,
    progress: &mut impl FnMut(u64),
) -> Result<Option<PathBuf>, OperationError> {
    if fs::symlink_metadata(long_path(source)).is_err() {
        return Err(OperationError::MissingSource(source.into()));
    }
    let target = match resolve_destination(destination, policy) {
        Ok(Some(target)) => target,
        Ok(None) => return Ok(None),
        Err(OperationError::Conflict(_)) if source == destination => destination.to_path_buf(),
        Err(error) => return Err(error),
    };
    match fs::rename(long_path(source), long_path(&target)) {
        Ok(()) => Ok(Some(target)),
        Err(rename_error) => {
            let Some(copied) = copy_path(source, &target, ConflictPolicy::Replace, control, progress)? else {
                return Ok(None);
            };
            let metadata = fs::symlink_metadata(long_path(source)).map_err(|error| OperationError::io(source, error))?;
            let removal = if metadata.is_dir() && !metadata.file_type().is_symlink() {
                fs::remove_dir_all(long_path(source))
            } else {
                fs::remove_file(long_path(source))
            };
            removal.map_err(|error| {
                OperationError::Invalid(format!(
                    "copied to {} but could not remove the original {}: {error} (rename failed with: {rename_error})",
                    copied.display(),
                    source.display()
                ))
            })?;
            Ok(Some(copied))
        }
    }
}

/// Copies a file or folder beside itself as "name - copy". Used by Duplicate.
pub fn duplicate(source: &Path, control: &OperationControl, progress: &mut impl FnMut(u64)) -> Result<PathBuf, OperationError> {
    let name = source.file_name().ok_or_else(|| OperationError::Invalid("source has no file name".into()))?;
    let stem = source.file_stem().map(|value| value.to_string_lossy().into_owned()).unwrap_or_else(|| name.to_string_lossy().into_owned());
    let extension = source.extension().map(|value| format!(".{}", value.to_string_lossy())).unwrap_or_default();
    let parent = source.parent().unwrap_or_else(|| Path::new("."));
    let base = parent.join(format!("{stem} - copy{extension}"));
    let destination = unique_destination(&base);
    copy_path(source, &destination, ConflictPolicy::AutoRename, control, progress)?
        .ok_or_else(|| OperationError::Invalid("duplicate was skipped".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OperationControl, OperationError};

    fn control() -> OperationControl {
        OperationControl::default()
    }

    #[test]
    fn copies_file_contents_and_reports_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("a.bin");
        std::fs::write(&source, vec![7u8; 600_000]).unwrap();
        let destination = dir.path().join("b.bin");
        let mut reported = 0;
        let written = copy_file(&source, &destination, ConflictPolicy::Ask, &control(), &mut |bytes| reported = bytes).unwrap().unwrap();
        assert_eq!(written, destination);
        assert_eq!(std::fs::read(&destination).unwrap().len(), 600_000);
        assert!(reported > 0);
        // No temporary artefacts left behind.
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
    }

    #[test]
    fn conflict_policies_behave_differently() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("a.txt");
        std::fs::write(&source, "new").unwrap();
        let destination = dir.path().join("a.txt");
        assert!(matches!(
            copy_file(&source, &destination, ConflictPolicy::Ask, &control(), &mut |_| {}),
            Err(OperationError::Conflict(_))
        ));
        assert!(copy_file(&source, &destination, ConflictPolicy::Skip, &control(), &mut |_| {}).unwrap().is_none());
        let renamed = copy_file(&source, &destination, ConflictPolicy::AutoRename, &control(), &mut |_| {}).unwrap().unwrap();
        assert_eq!(renamed.file_name().unwrap(), "a (1).txt");
    }

    #[test]
    fn recursive_copy_preserves_structure() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("tree");
        std::fs::create_dir_all(source.join("nested")).unwrap();
        std::fs::write(source.join("one.txt"), "1").unwrap();
        std::fs::write(source.join("nested/two.txt"), "2").unwrap();
        let destination = dir.path().join("copy");
        copy_path(&source, &destination, ConflictPolicy::Ask, &control(), &mut |_| {}).unwrap();
        assert_eq!(std::fs::read_to_string(destination.join("nested/two.txt")).unwrap(), "2");
    }

    #[test]
    fn folder_cannot_be_copied_into_itself() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("tree");
        std::fs::create_dir_all(source.join("inner")).unwrap();
        let result = copy_path(&source, &source.join("inner"), ConflictPolicy::Ask, &control(), &mut |_| {});
        assert!(matches!(result, Err(OperationError::NestedDestination(_))));
    }

    #[test]
    fn cancelled_copy_leaves_no_partial_output() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("big.bin");
        std::fs::write(&source, vec![1u8; 4 * 1024 * 1024]).unwrap();
        let destination = dir.path().join("out.bin");
        let control = OperationControl::default();
        control.cancel();
        let result = copy_file(&source, &destination, ConflictPolicy::Ask, &control, &mut |_| {});
        assert!(result.unwrap_err().is_cancelled());
        assert!(!destination.exists());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn move_renames_within_the_same_volume() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("move-me.txt");
        std::fs::write(&source, "payload").unwrap();
        let destination = dir.path().join("moved.txt");
        move_path(&source, &destination, ConflictPolicy::Ask, &control(), &mut |_| {}).unwrap();
        assert!(!source.exists());
        assert_eq!(std::fs::read_to_string(&destination).unwrap(), "payload");
    }

    #[test]
    fn duplicate_creates_a_copy_beside_the_original() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("doc.md");
        std::fs::write(&source, "text").unwrap();
        let copy = duplicate(&source, &control(), &mut |_| {}).unwrap();
        assert_eq!(copy.file_name().unwrap(), "doc - copy.md");
        assert_eq!(std::fs::read_to_string(&copy).unwrap(), "text");
    }
}
