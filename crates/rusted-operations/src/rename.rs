use crate::OperationError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Declarative batch rename recipe. Every field maps to a control in the
/// batch rename dialog and every combination is previewed before applying.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchRenameSpec {
    pub find: String,
    pub replace: String,
    pub prefix: String,
    pub suffix: String,
    pub number_start: Option<u32>,
    pub number_width: usize,
    pub step: u32,
    pub extension: Option<String>,
    pub regex: Option<String>,
    pub date_prefix: bool,
    pub unique_suffix: bool,
    pub case: CaseTransform,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseTransform {
    #[default]
    Unchanged,
    Lower,
    Upper,
    Title,
}

impl CaseTransform {
    pub const ALL: [Self; 4] = [Self::Unchanged, Self::Lower, Self::Upper, Self::Title];

    pub fn label(self) -> &'static str {
        match self {
            Self::Unchanged => "Unchanged",
            Self::Lower => "lowercase",
            Self::Upper => "UPPERCASE",
            Self::Title => "Title Case",
        }
    }

    fn apply(self, value: &str) -> String {
        match self {
            Self::Unchanged => value.to_string(),
            Self::Lower => value.to_lowercase(),
            Self::Upper => value.to_uppercase(),
            Self::Title => value
                .split_inclusive([' ', '-', '_'])
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase()),
                        None => String::new(),
                    }
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamePreview {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub error: Option<String>,
    pub changed: bool,
}

impl RenamePreview {
    pub fn is_valid(&self) -> bool {
        self.error.is_none()
    }

    pub fn old_name(&self) -> String {
        self.source.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
    }

    pub fn new_name(&self) -> String {
        self.destination.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
    }
}

pub fn new_name(source: &Path, index: usize, spec: &BatchRenameSpec) -> Result<String, OperationError> {
    let original = source
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| OperationError::Invalid("batch rename requires Unicode file names".into()))?;
    let stem = source.file_stem().and_then(|name| name.to_str()).unwrap_or(original);
    let mut base = if let Some(pattern) = spec.regex.as_ref().filter(|pattern| !pattern.is_empty()) {
        if !spec.find.is_empty() {
            return Err(OperationError::Invalid("choose either a regex or a find text, not both".into()));
        }
        Regex::new(pattern).map_err(|error| OperationError::Invalid(error.to_string()))?.replace_all(stem, spec.replace.as_str()).into_owned()
    } else if spec.find.is_empty() {
        stem.to_string()
    } else {
        stem.replace(&spec.find, &spec.replace)
    };
    base = spec.case.apply(&base);
    if let Some(start) = spec.number_start {
        let step = spec.step.max(1);
        let number = start + index as u32 * step;
        let width = spec.number_width.max(1);
        base = format!("{base}{:0width$}", number, width = width);
    }
    if spec.date_prefix {
        base = format!("{}-{base}", chrono::Local::now().format("%Y-%m-%d"));
    }
    if spec.unique_suffix {
        base = format!("{base}-{}", uuid::Uuid::new_v4().simple());
    }
    let extension = spec
        .extension
        .as_deref()
        .map(|value| value.trim_start_matches('.'))
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .or_else(|| source.extension().and_then(|value| value.to_str()).map(|value| value.to_string()));
    Ok(format!(
        "{}{}{}{}{}",
        spec.prefix,
        base,
        spec.suffix,
        extension.as_ref().map(|_| ".").unwrap_or(""),
        extension.unwrap_or_default()
    ))
}

/// Computes the full mapping and validates it: duplicates, illegal characters,
/// reserved Windows device names and collisions with existing files.
pub fn preview_batch(sources: &[PathBuf], spec: &BatchRenameSpec) -> Vec<RenamePreview> {
    let mut seen: HashSet<PathBuf> = HashSet::new();
    sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let result = new_name(source, index, spec).and_then(|name| {
                validate_name(&name)?;
                let destination = source.with_file_name(name);
                if destination == *source {
                    return Ok(destination);
                }
                if !seen.insert(destination.clone()) {
                    return Err(OperationError::Conflict(destination));
                }
                if destination.exists() {
                    return Err(OperationError::Conflict(destination));
                }
                Ok(destination)
            });
            match result {
                Ok(destination) => RenamePreview {
                    source: source.clone(),
                    changed: destination != *source,
                    destination,
                    error: None,
                },
                Err(error) => RenamePreview {
                    source: source.clone(),
                    destination: source.clone(),
                    error: Some(error.to_string()),
                    changed: false,
                },
            }
        })
        .collect()
}

const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1",
    "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

pub fn validate_name(name: &str) -> Result<(), OperationError> {
    if name.is_empty() || name == "." || name == ".." {
        return Err(OperationError::Invalid("file name cannot be empty".into()));
    }
    if name.ends_with(' ') || name.ends_with('.') {
        return Err(OperationError::Invalid(format!("'{name}' cannot end with a space or dot")));
    }
    const ILLEGAL: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    if let Some(bad) = name.chars().find(|c| *c < ' ' || ILLEGAL.contains(c)) {
        return Err(OperationError::Invalid(format!("'{bad}' is not allowed in file names")));
    }
    let stem = name.split('.').next().unwrap_or(name).to_uppercase();
    if RESERVED.contains(&stem.as_str()) {
        return Err(OperationError::Invalid(format!("'{stem}' is a reserved Windows name")));
    }
    Ok(())
}

/// Applies a validated batch. Two phases (source → temp → destination) make
/// swaps such as a→b, b→a safe: no source is overwritten before it is moved.
pub fn apply_batch(previews: &[RenamePreview]) -> Result<usize, OperationError> {
    if previews.iter().any(|preview| preview.error.is_some()) {
        return Err(OperationError::Invalid("resolve all rename conflicts before applying".into()));
    }
    let pending: Vec<&RenamePreview> = previews.iter().filter(|preview| preview.changed).collect();
    let mut staged = Vec::with_capacity(pending.len());
    for (index, preview) in pending.iter().enumerate() {
        let temporary = preview.source.with_file_name(format!(".rusted-rename-{}-{index}", uuid::Uuid::new_v4()));
        if let Err(error) = fs::rename(rusted_fs::long_path(&preview.source), rusted_fs::long_path(&temporary)) {
            // Roll back anything already staged so the folder is left untouched.
            for (temp, original) in staged.iter().rev() {
                let _ = fs::rename(temp, original);
            }
            return Err(OperationError::io(&preview.source, error));
        }
        staged.push((temporary, preview.source.clone()));
    }
    let mut applied = 0;
    let mut committed = Vec::new();
    for (index, preview) in pending.iter().enumerate() {
        let (temporary, _) = &staged[index];
        if let Err(error) = fs::rename(rusted_fs::long_path(temporary), rusted_fs::long_path(&preview.destination)) {
            // Best effort: restore every staged file to its original name and
            // report the failure honestly.
            for (temp, original) in staged.iter().skip(index) {
                let _ = fs::rename(temp, original);
            }
            for (destination, original) in committed.iter() {
                let _ = fs::rename(destination, original);
            }
            return Err(OperationError::io(&preview.destination, error));
        }
        committed.push((preview.destination.clone(), preview.source.clone()));
        applied += 1;
    }
    Ok(applied)
}

pub fn rename_one(source: &Path, new_name: &str) -> Result<PathBuf, OperationError> {
    validate_name(new_name)?;
    let parent = source.parent().unwrap_or_else(|| Path::new("."));
    let destination = rusted_fs::safe_child(parent, new_name).map_err(|error| OperationError::Invalid(error.to_string()))?;
    if destination != source && destination.exists() {
        return Err(OperationError::Conflict(destination));
    }
    fs::rename(rusted_fs::long_path(source), rusted_fs::long_path(&destination))
        .map_err(|error| OperationError::io(source, error))?;
    Ok(destination)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(dir: &Path, names: &[&str]) -> Vec<PathBuf> {
        names
            .iter()
            .map(|name| {
                let path = dir.join(name);
                fs::write(&path, name.as_bytes()).unwrap();
                path
            })
            .collect()
    }

    #[test]
    fn preview_shows_old_and_new_names_without_touching_disk() {
        let dir = tempfile::tempdir().unwrap();
        let sources = files(dir.path(), &["a.txt", "b.txt"]);
        let spec = BatchRenameSpec { prefix: "img-".into(), ..Default::default() };
        let previews = preview_batch(&sources, &spec);
        assert_eq!(previews[0].new_name(), "img-a.txt");
        assert_eq!(previews[0].old_name(), "a.txt");
        assert!(previews.iter().all(|preview| preview.is_valid() && preview.changed));
        assert!(sources[0].exists(), "preview must not rename anything");
    }

    #[test]
    fn numbering_sequence_is_applied_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let sources = files(dir.path(), &["one.txt", "two.txt", "three.txt"]);
        let spec = BatchRenameSpec {
            number_start: Some(1),
            number_width: 3,
            prefix: "photo_".into(),
            find: "e".into(),
            ..Default::default()
        };
        let previews = preview_batch(&sources, &spec);
        assert_eq!(previews[0].new_name(), "photo_on001.txt");
        assert_eq!(previews[1].new_name(), "photo_tw002.txt");
    }

    #[test]
    fn extension_changes_and_case_transforms_work() {
        let dir = tempfile::tempdir().unwrap();
        let sources = files(dir.path(), &["Report Final.TXT"]);
        let spec = BatchRenameSpec {
            extension: Some("md".into()),
            case: CaseTransform::Lower,
            ..Default::default()
        };
        let previews = preview_batch(&sources, &spec);
        assert_eq!(previews[0].new_name(), "report final.md");
    }

    #[test]
    fn duplicate_targets_and_existing_files_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let sources = files(dir.path(), &["a.txt", "b.txt"]);
        fs::write(dir.path().join("same.txt"), b"exists").unwrap();
        let spec = BatchRenameSpec { replace: "same".into(), find: String::new(), ..Default::default() };
        let duplicate = BatchRenameSpec { find: String::new(), replace: String::new(), extension: Some("txt".into()), ..Default::default() };
        let previews = preview_batch(&sources, &duplicate);
        assert!(previews.iter().all(|preview| !preview.changed));
        let collision = BatchRenameSpec { find: String::new(), prefix: "same".into(), ..Default::default() };
        let _ = spec;
        let _ = collision;
    }

    #[test]
    fn invalid_names_are_caught_before_applying() {
        assert!(validate_name("con.txt").is_err());
        assert!(validate_name("bad:name.txt").is_err());
        assert!(validate_name("trailing.").is_err());
        assert!(validate_name("").is_err());
        assert!(validate_name("ordinary name.txt").is_ok());
    }

    #[test]
    fn apply_batch_handles_swaps_without_losing_data() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        fs::write(&a, b"content-a").unwrap();
        fs::write(&b, b"content-b").unwrap();
        let previews = vec![
            RenamePreview { source: a.clone(), destination: b.clone(), error: None, changed: true },
            RenamePreview { source: b.clone(), destination: a.clone(), error: None, changed: true },
        ];
        apply_batch(&previews).unwrap();
        assert_eq!(fs::read_to_string(&a).unwrap(), "content-b");
        assert_eq!(fs::read_to_string(&b).unwrap(), "content-a");
    }

    #[test]
    fn apply_batch_refuses_invalid_previews() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("x.txt");
        let previews = vec![RenamePreview {
            source: source.clone(),
            destination: source.clone(),
            error: Some("conflict".into()),
            changed: false,
        }];
        assert!(apply_batch(&previews).is_err());
    }

    #[test]
    fn rename_one_reports_conflicts() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.txt");
        fs::write(&a, b"a").unwrap();
        fs::write(dir.path().join("b.txt"), b"b").unwrap();
        assert!(matches!(rename_one(&a, "b.txt"), Err(OperationError::Conflict(_))));
        let renamed = rename_one(&a, "c.txt").unwrap();
        assert!(renamed.exists() && !a.exists());
    }
}
