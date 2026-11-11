use crate::ThemeName;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    Directory,
    File,
    Symlink,
    Junction,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub kind: EntryKind,
    pub size: u64,
    pub modified: Option<DateTime<Utc>>,
    pub created: Option<DateTime<Utc>>,
    pub readonly: bool,
    pub hidden: bool,
    pub extension: Option<String>,
}

impl FileEntry {
    pub fn is_dir(&self) -> bool {
        matches!(self.kind, EntryKind::Directory | EntryKind::Junction)
    }

    pub fn is_link(&self) -> bool {
        matches!(self.kind, EntryKind::Symlink | EntryKind::Junction)
    }

    pub fn type_name(&self) -> &str {
        match self.kind {
            EntryKind::Directory => "Folder",
            EntryKind::Junction => "Junction",
            EntryKind::Symlink => "Shortcut",
            EntryKind::File => self.extension.as_deref().unwrap_or("File"),
            EntryKind::Unknown => "Unknown",
        }
    }

    pub fn human_size(&self) -> String {
        if self.is_dir() {
            String::new()
        } else {
            format_bytes(self.size)
        }
    }

    pub fn modified_label(&self) -> String {
        self.modified.map(|value| value.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string()).unwrap_or_default()
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    Grid,
    List,
    #[default]
    Details,
    Miller,
    Tree,
}

impl ViewMode {
    pub const ALL: [Self; 5] = [Self::Grid, Self::List, Self::Details, Self::Miller, Self::Tree];

    pub fn label(self) -> &'static str {
        match self {
            Self::Grid => "Grid",
            Self::List => "List",
            Self::Details => "Details",
            Self::Miller => "Miller",
            Self::Tree => "Tree",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortKey {
    #[default]
    Name,
    Size,
    Modified,
    Kind,
}

impl SortKey {
    pub const ALL: [Self; 4] = [Self::Name, Self::Size, Self::Modified, Self::Kind];

    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Size => "Size",
            Self::Modified => "Date modified",
            Self::Kind => "Type",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortSpec {
    pub key: SortKey,
    pub descending: bool,
    pub folders_first: bool,
}

impl Default for SortSpec {
    fn default() -> Self {
        Self { key: SortKey::Name, descending: false, folders_first: true }
    }
}

impl SortSpec {
    pub fn toggled(self, key: SortKey) -> Self {
        if self.key == key {
            Self { descending: !self.descending, ..self }
        } else {
            Self { key, descending: false, ..self }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Selection {
    paths: BTreeSet<PathBuf>,
    pub anchor: Option<PathBuf>,
}

impl Selection {
    pub fn paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.paths.iter()
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn contains(&self, path: &PathBuf) -> bool {
        self.paths.contains(path)
    }

    pub fn clear(&mut self) {
        self.paths.clear();
        self.anchor = None;
    }

    pub fn select_only(&mut self, path: PathBuf) {
        self.clear();
        self.paths.insert(path.clone());
        self.anchor = Some(path);
    }

    pub fn toggle(&mut self, path: PathBuf) {
        if !self.paths.remove(&path) {
            self.paths.insert(path.clone());
        }
        self.anchor = Some(path);
    }

    pub fn select_range(&mut self, ordered: &[FileEntry], from: &PathBuf, to: &PathBuf) {
        let start = ordered.iter().position(|entry| &entry.path == from);
        let end = ordered.iter().position(|entry| &entry.path == to);
        if let (Some(start), Some(end)) = (start, end) {
            let (low, high) = if start <= end { (start, end) } else { (end, start) };
            self.paths = ordered[low..=high].iter().map(|entry| entry.path.clone()).collect();
        } else {
            self.select_only(to.clone());
        }
    }

    pub fn select_all(&mut self, entries: &[FileEntry]) {
        self.paths = entries.iter().map(|entry| entry.path.clone()).collect();
        self.anchor = entries.first().map(|entry| entry.path.clone());
    }

    pub fn retain_existing(&mut self, entries: &[FileEntry]) {
        let valid: BTreeSet<_> = entries.iter().map(|entry| entry.path.clone()).collect();
        self.paths.retain(|path| valid.contains(path));
        if self.anchor.as_ref().is_some_and(|path| !valid.contains(path)) {
            self.anchor = None;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Density {
    Compact,
    #[default]
    Comfortable,
}

impl Density {
    pub fn row_height(self) -> f32 {
        match self {
            Self::Compact => 22.0,
            Self::Comfortable => 28.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Comfortable => "Comfortable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConflictPolicy {
    Replace,
    Skip,
    AutoRename,
    #[default]
    Ask,
}

impl ConflictPolicy {
    pub const ALL: [Self; 4] = [Self::Ask, Self::Replace, Self::Skip, Self::AutoRename];

    pub fn label(self) -> &'static str {
        match self {
            Self::Ask => "Ask each time",
            Self::Replace => "Replace",
            Self::Skip => "Skip",
            Self::AutoRename => "Keep both (rename)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub theme: ThemeName,
    pub density: Density,
    pub animations: bool,
    pub animation_intensity: u8,
    pub default_view: ViewMode,
    pub show_hidden: bool,
    pub thumbnail_size: u16,
    pub sidebar_width: u16,
    pub sidebar_collapsed: bool,
    pub sidebar_shortcuts: Vec<PathBuf>,
    pub confirm_permanent_delete: bool,
    pub default_conflict_policy: ConflictPolicy,
    pub indexed_roots: Vec<PathBuf>,
    pub index_exclusions: Vec<PathBuf>,
    pub preview_text_limit_kib: u32,
    pub preview_enabled: bool,
    pub restore_session: bool,
    pub root_font_size: u16,
    pub tab_close_button: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemeName::ObsidianBlack,
            density: Density::Comfortable,
            animations: true,
            animation_intensity: 70,
            default_view: ViewMode::Details,
            show_hidden: false,
            thumbnail_size: 112,
            sidebar_width: 216,
            sidebar_collapsed: false,
            sidebar_shortcuts: Vec::new(),
            confirm_permanent_delete: true,
            default_conflict_policy: ConflictPolicy::Ask,
            indexed_roots: Vec::new(),
            index_exclusions: Vec::new(),
            preview_text_limit_kib: 512,
            preview_enabled: true,
            restore_session: true,
            root_font_size: 13,
            tab_close_button: true,
        }
    }
}

impl Settings {
    /// Clamps user-editable numeric ranges so persisted junk cannot break layout.
    pub fn sanitized(mut self) -> Self {
        self.animation_intensity = self.animation_intensity.min(100);
        self.thumbnail_size = self.thumbnail_size.clamp(64, 320);
        self.sidebar_width = self.sidebar_width.clamp(160, 480);
        self.preview_text_limit_kib = self.preview_text_limit_kib.clamp(16, 8192);
        self.root_font_size = self.root_font_size.clamp(11, 20);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SearchState {
    pub query: String,
    pub recursive: bool,
    pub indexed: bool,
}

impl SearchState {
    pub fn is_active(&self) -> bool {
        !self.query.trim().is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SearchFilters {
    pub extension: String,
    pub min_size_kib: Option<u64>,
    pub max_size_kib: Option<u64>,
    pub modified_within_days: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub message: String,
    pub is_error: bool,
}

impl Notice {
    pub fn info(message: impl Into<String>) -> Self {
        Self { message: message.into(), is_error: false }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self { message: message.into(), is_error: true }
    }
}

/// Folders first (optional), then the chosen key, then a stable name tiebreak.
pub fn sort_entries(entries: &mut [FileEntry], spec: SortSpec) {
    entries.sort_by(|a, b| {
        let folder_order =
            if spec.folders_first { b.is_dir().cmp(&a.is_dir()) } else { std::cmp::Ordering::Equal };
        folder_order.then_with(|| {
            let order = match spec.key {
                SortKey::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()).then_with(|| a.name.cmp(&b.name)),
                SortKey::Size => a.size.cmp(&b.size).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                SortKey::Modified => a.modified.cmp(&b.modified).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                SortKey::Kind => a.type_name().cmp(b.type_name()).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
            };
            if spec.descending { order.reverse() } else { order }
        })
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, kind: EntryKind, size: u64) -> FileEntry {
        FileEntry {
            path: PathBuf::from(name),
            name: name.into(),
            kind,
            size,
            modified: None,
            created: None,
            readonly: false,
            hidden: false,
            extension: None,
        }
    }

    #[test]
    fn sorting_keeps_folders_first_and_is_stable() {
        let mut entries = vec![
            entry("b.txt", EntryKind::File, 10),
            entry("a.txt", EntryKind::File, 10),
            entry("Zeta", EntryKind::Directory, 0),
        ];
        sort_entries(&mut entries, SortSpec::default());
        assert_eq!(entries[0].name, "Zeta");
        assert_eq!(entries[1].name, "a.txt");
        assert_eq!(entries[2].name, "b.txt");
    }

    #[test]
    fn toggling_the_same_key_reverses_order() {
        let spec = SortSpec::default().toggled(SortKey::Name);
        assert!(spec.descending && spec.key == SortKey::Name);
        let spec = spec.toggled(SortKey::Size);
        assert!(!spec.descending && spec.key == SortKey::Size);
    }

    #[test]
    fn selection_ranges_and_toggle() {
        let entries = vec![entry("a", EntryKind::File, 0), entry("b", EntryKind::File, 0), entry("c", EntryKind::File, 0)];
        let mut selection = Selection::default();
        selection.select_range(&entries, &entries[0].path, &entries[2].path);
        assert_eq!(selection.len(), 3);
        selection.toggle(entries[1].path.clone());
        assert_eq!(selection.len(), 2);
        selection.retain_existing(&entries[..1]);
        assert_eq!(selection.len(), 1);
        selection.clear();
        assert!(selection.is_empty() && selection.anchor.is_none());
    }

    #[test]
    fn settings_sanitize_clamps_out_of_range_values() {
        let mut settings = Settings::default();
        settings.thumbnail_size = 5000;
        settings.root_font_size = 1;
        let settings = settings.sanitized();
        assert_eq!(settings.thumbnail_size, 320);
        assert_eq!(settings.root_font_size, 11);
    }

    #[test]
    fn byte_formatting_is_readable() {
        assert_eq!(format_bytes(999), "999 B");
        assert_eq!(format_bytes(1536), "1.5 KB");
    }
}
