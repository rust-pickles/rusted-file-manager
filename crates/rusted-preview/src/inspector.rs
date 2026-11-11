use rusted_core::FileEntry;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorInfo {
    pub name: String,
    pub kind: String,
    pub location: String,
    pub size: String,
    pub created: String,
    pub modified: String,
    pub readonly: bool,
    pub hidden: bool,
    pub extra: Vec<(String, String)>,
}

/// Builds the inspector rows shown in the right-hand panel. Metadata comes from
/// the filesystem, never from stale cached entries.
pub fn describe(entry: &FileEntry) -> InspectorInfo {
    let location = entry.path.parent().map(Path::display).map(|value| value.to_string()).unwrap_or_default();
    let mut extra = Vec::new();
    if let Some(extension) = &entry.extension {
        extra.push(("Extension".to_string(), extension.clone()));
    }
    if entry.is_link() {
        if let Ok(target) = std::fs::read_link(&entry.path) {
            extra.push(("Target".to_string(), target.display().to_string()));
        }
    }
    InspectorInfo {
        name: entry.name.clone(),
        kind: entry.type_name().to_string(),
        location,
        size: if entry.is_dir() { "—".into() } else { rusted_core::format_bytes(entry.size) },
        created: entry.created.map(|value| value.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_else(|| "Unknown".into()),
        modified: entry.modified.map(|value| value.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_else(|| "Unknown".into()),
        readonly: entry.readonly,
        hidden: entry.hidden,
        extra,
    }
}
