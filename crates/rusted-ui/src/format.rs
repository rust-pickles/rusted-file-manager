//! Human-readable labels shared by the panels. Everything here is pure so it can
//! be unit tested without a window.

use rusted_core::{format_bytes, EntryKind, FileEntry};

pub fn size_label(entry: &FileEntry) -> String {
    if entry.is_dir() {
        String::new()
    } else {
        format_bytes(entry.size)
    }
}

pub fn date_label(entry: &FileEntry) -> String {
    entry
        .modified
        .map(|value| value.with_timezone(&chrono::Local).format("%d/%m/%Y %H:%M").to_string())
        .unwrap_or_default()
}

pub fn type_label(entry: &FileEntry) -> String {
    match entry.kind {
        EntryKind::Directory => "Folder".into(),
        EntryKind::Junction => "Junction".into(),
        EntryKind::Symlink => "Shortcut".into(),
        EntryKind::File => entry
            .extension
            .as_deref()
            .map(|extension| format!("{} file", extension.to_uppercase()))
            .unwrap_or_else(|| "File".into()),
        EntryKind::Unknown => "Unknown".into(),
    }
}

/// Truncates on a character boundary, never mid-glyph.
pub fn ellipsize(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Shortens a deep path for the status bar: keeps the root and the leaf.
pub fn compact_path(path: &std::path::Path, max_chars: usize) -> String {
    let text = path.display().to_string();
    if text.chars().count() <= max_chars {
        return text;
    }
    let leaf = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let root = path
        .components()
        .next()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .unwrap_or_default();
    let budget = max_chars.saturating_sub(root.chars().count() + leaf.chars().count() + 2);
    if budget == 0 {
        return ellipsize(&text, max_chars);
    }
    format!("{root}{}…{}{leaf}", std::path::MAIN_SEPARATOR, std::path::MAIN_SEPARATOR)
}

/// Selection summary for the status bar, e.g. `3 items · 1.4 MB`.
pub fn selection_summary(entries: &[FileEntry], selected: usize, bytes: u64) -> String {
    if selected == 0 {
        return format!("{} items", entries.len());
    }
    format!("{selected} of {} selected · {}", entries.len(), format_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn entry(name: &str, kind: EntryKind) -> FileEntry {
        FileEntry {
            path: std::path::PathBuf::from(name),
            name: name.into(),
            kind,
            size: 2048,
            modified: None,
            created: None,
            readonly: false,
            hidden: false,
            extension: name.rsplit_once('.').map(|(_, extension)| extension.to_string()),
        }
    }

    #[test]
    fn ellipsize_respects_character_boundaries() {
        assert_eq!(ellipsize("short", 10), "short");
        assert_eq!(ellipsize("averylongname", 5), "aver…");
        assert_eq!(ellipsize("日本語のファイル名", 4), "日本語…");
    }

    #[test]
    fn compact_paths_keep_root_and_leaf() {
        let compact = compact_path(Path::new("/home/user/projects/rusted/crates/rusted-ui/src/shell.rs"), 24);
        assert!(compact.contains("shell.rs"));
        assert!(compact.chars().count() <= 26);
    }

    #[test]
    fn labels_describe_real_metadata() {
        assert_eq!(size_label(&entry("a.bin", EntryKind::File)), "2.0 KB");
        assert_eq!(size_label(&entry("dir", EntryKind::Directory)), "");
        assert_eq!(type_label(&entry("notes.md", EntryKind::File)), "MD file");
        assert_eq!(type_label(&entry("dir", EntryKind::Directory)), "Folder");
        assert_eq!(date_label(&entry("a.bin", EntryKind::File)), "");
    }

    #[test]
    fn selection_summary_reports_counts_and_bytes() {
        let entries = vec![entry("a", EntryKind::File), entry("b", EntryKind::File)];
        assert_eq!(selection_summary(&entries, 0, 0), "2 items");
        assert_eq!(selection_summary(&entries, 2, 4096), "2 of 2 selected · 4.0 KB");
    }
}
