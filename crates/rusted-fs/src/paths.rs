use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PathError {
    #[error("path does not exist: {0}")]
    NotFound(PathBuf),
    #[error("path is not a directory: {0}")]
    NotDirectory(PathBuf),
    #[error("path contains an invalid parent traversal")]
    InvalidTraversal,
}

pub fn validate_directory(path: &Path) -> Result<PathBuf, PathError> {
    let actual = long_path(path);
    if !actual.exists() {
        return Err(PathError::NotFound(path.into()));
    }
    if !actual.is_dir() {
        return Err(PathError::NotDirectory(path.into()));
    }
    Ok(path.to_path_buf())
}

pub fn safe_child(parent: &Path, name: &str) -> Result<PathBuf, PathError> {
    let candidate = Path::new(name);
    if name.is_empty() || candidate.is_absolute() || candidate.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(PathError::InvalidTraversal);
    }
    Ok(parent.join(candidate))
}

/// Adds Windows' verbatim prefix so paths beyond MAX_PATH keep working.
#[cfg(windows)]
pub fn long_path(path: &Path) -> PathBuf {
    use std::path::Prefix;
    let text = path.as_os_str().to_string_lossy();
    if text.starts_with(r"\\?\") || text.len() < 240 {
        return path.into();
    }
    match path.components().next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(_) | Prefix::VerbatimDisk(_) => PathBuf::from(format!(r"\\?\{text}")),
            _ => path.into(),
        },
        _ => path.into(),
    }
}

#[cfg(not(windows))]
pub fn long_path(path: &Path) -> PathBuf {
    path.into()
}

pub fn parent(path: &Path) -> Option<PathBuf> {
    path.parent().filter(|candidate| *candidate != path).map(Path::to_path_buf)
}

/// Paths worth offering in Go-To: drives plus the current ancestors.
pub fn ancestor_chain(path: &Path, limit: usize) -> Vec<PathBuf> {
    let mut chain = Vec::new();
    let mut current = path.to_path_buf();
    while chain.len() < limit {
        chain.push(current.clone());
        match parent(&current) {
            Some(next) => current = next,
            None => break,
        }
    }
    chain
}

/// Accepts `~`, environment variables and unquoted drag/drop paths.
pub fn expand_input(input: &str) -> PathBuf {
    let trimmed = input.trim().trim_matches('"');
    let expanded = if let Some(rest) = trimmed.strip_prefix("~/").or_else(|| trimmed.strip_prefix("~\\")) {
        match home() {
            Some(home) => format!("{}{}{}", home.display(), std::path::MAIN_SEPARATOR, rest),
            None => trimmed.to_string(),
        }
    } else if trimmed == "~" {
        home().map(|home| home.display().to_string()).unwrap_or_else(|| trimmed.to_string())
    } else {
        expand_environment(trimmed)
    };
    PathBuf::from(expanded)
}

fn expand_environment(input: &str) -> String {
    if !input.contains('%') && !input.contains('$') {
        return input.to_string();
    }
    let mut output = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find('%') {
        let (before, after) = rest.split_at(start);
        output.push_str(before);
        let tail = &after[1..];
        match tail.find('%') {
            Some(end) => {
                let key = &tail[..end];
                match std::env::var(key) {
                    Ok(value) => output.push_str(&value),
                    Err(_) => {
                        output.push('%');
                        output.push_str(key);
                        output.push('%');
                    }
                }
                rest = &tail[end + 1..];
            }
            None => {
                output.push_str(after);
                rest = "";
            }
        }
    }
    if rest.contains('$') {
        let mut parts = rest.split('$');
        if let Some(first) = parts.next() {
            output.push_str(first);
        }
        for part in parts {
            let split = part.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(part.len());
            let (key, tail) = part.split_at(split);
            match std::env::var(key) {
                Ok(value) => output.push_str(&value),
                Err(_) => {
                    output.push('$');
                    output.push_str(key);
                }
            }
            output.push_str(tail);
        }
    } else {
        output.push_str(rest);
    }
    output
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// Human-friendly location label used by the toolbar and tab titles.
pub fn location_label(path: &Path) -> String {
    if let Some(name) = path.file_name() {
        return name.to_string_lossy().into_owned();
    }
    let text = path.display().to_string();
    if text.is_empty() { "This PC".into() } else { text }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn children_cannot_escape_the_parent() {
        assert!(safe_child(Path::new("/tmp"), "ok").is_ok());
        for bad in ["../secret", "/absolute", "a/b", "", ".."] {
            assert!(safe_child(Path::new("/tmp"), bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn validation_rejects_files_and_missing_paths() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("file.txt");
        std::fs::write(&file, b"x").unwrap();
        assert!(matches!(validate_directory(&file), Err(PathError::NotDirectory(_))));
        assert!(matches!(validate_directory(&temp.path().join("nope")), Err(PathError::NotFound(_))));
        assert!(validate_directory(temp.path()).is_ok());
    }

    #[test]
    fn ancestors_are_listed_root_last() {
        let chain = ancestor_chain(Path::new("/home/user/docs"), 10);
        assert_eq!(chain[0], PathBuf::from("/home/user/docs"));
        assert_eq!(chain.last().unwrap(), &PathBuf::from("/"));
    }

    #[test]
    fn environment_variables_expand() {
        std::env::set_var("RUSTED_TEST_VAR", "value");
        assert_eq!(expand_environment("%RUSTED_TEST_VAR%/x"), "value/x");
        assert_eq!(expand_environment("$RUSTED_TEST_VAR/x"), "value/x");
        assert_eq!(expand_environment("plain"), "plain");
        assert_eq!(expand_environment("%NOT_SET_HERE%/x"), "%NOT_SET_HERE%/x");
    }

    #[test]
    fn labels_use_the_last_component() {
        assert_eq!(location_label(Path::new("/home/user")), "user");
        assert_eq!(location_label(Path::new("/")), "/");
    }
}
