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

/// Joins a single user-supplied *name* onto `parent`.
///
/// The whole point is to reject anything that could escape the parent, so a name
/// containing a separator is refused outright: `Path::new("a/b").components()`
/// yields only `Normal` components, which is why component inspection alone is
/// not enough to catch traversal.
pub fn safe_child(parent: &Path, name: &str) -> Result<PathBuf, PathError> {
    let rejected = name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
        || name.chars().any(|c| c.is_control());
    if rejected {
        return Err(PathError::InvalidTraversal);
    }
    let candidate = Path::new(name);
    if candidate.components().any(|component| !matches!(component, Component::Normal(_))) {
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
    // `output` already holds the resolved `%VAR%` prefix, so the dollar pass has to
    // run over the accumulated text plus the tail, not just the tail.
    let combined = format!("{output}{rest}");
    let mut output = String::with_capacity(combined.len());
    rest = &combined;
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

    /// An absolute root that exists on every host, so assertions about parent
    /// relationships do not depend on which platform the tests run on.
    fn root() -> PathBuf {
        tempfile::tempdir().unwrap().path().to_path_buf()
    }

    #[test]
    fn children_cannot_escape_the_parent() {
        let parent = root();
        assert!(safe_child(&parent, "ok").is_ok());
        assert_eq!(safe_child(&parent, "ok").unwrap(), parent.join("ok"));
        // `"a/b"` is the interesting case: its components are all `Normal`, so only
        // an explicit separator check rejects it.
        for bad in ["../secret", "/absolute", "a/b", "a\\b", "", "..", ".", "C:evil", "nul\0byte"] {
            assert!(safe_child(&parent, bad).is_err(), "{bad:?} should be rejected");
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
        let start = root().join("user").join("docs");
        let chain = ancestor_chain(&start, 10);
        assert_eq!(chain[0], start);
        // Walking parents must terminate at the filesystem root for this host.
        let last = chain.last().unwrap();
        assert_eq!(last.parent(), None, "chain should end at a root: {last:?}");
        assert!(chain.len() >= 2);
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
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("user")).unwrap();
        assert_eq!(location_label(&dir.path().join("user")), "user");
        // A root has no final component, so it must fall back to its own text.
        let root_path = ancestor_chain(dir.path(), 64).pop().unwrap();
        assert!(!location_label(&root_path).is_empty());
    }

    #[test]
    fn mixed_percent_and_dollar_expansion_resolves_both() {
        std::env::set_var("RUSTED_TEST_MIX", "mix");
        std::env::set_var("RUSTED_TEST_TAIL", "tail");
        // A `%VAR%` prefix used to swallow the `$VAR` suffix.
        assert_eq!(expand_environment("%RUSTED_TEST_MIX%/$RUSTED_TEST_TAIL"), "mix/tail");
        assert_eq!(expand_environment("$RUSTED_TEST_TAIL/%RUSTED_TEST_MIX%"), "tail/mix");
    }
}
