use crate::PreviewError;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPreview {
    pub content: String,
    pub truncated: bool,
    pub bytes_read: usize,
    pub line_count: usize,
    pub binary: bool,
}

/// Reads at most `limit` bytes and always yields valid UTF-8. Invalid byte
/// sequences are replaced rather than failing, so a mislabelled file still
/// previews; a NUL byte in the head marks it binary and suppresses the body.
pub fn read_text_preview(path: &Path, limit: u64) -> Result<TextPreview, PreviewError> {
    let mut file = std::fs::File::open(rusted_fs::long_path(path)).map_err(|error| PreviewError::io(path, error))?;
    let mut buffer = Vec::new();
    let read = file.by_ref().take(limit).read_to_end(&mut buffer).map_err(|error| PreviewError::io(path, error))?;
    let binary = buffer.iter().take(8192).any(|byte| *byte == 0);
    let mut content = String::from_utf8_lossy(&buffer).into_owned();
    if binary {
        content.clear();
    }
    let line_count = content.lines().count();
    Ok(TextPreview { content, truncated: read as u64 == limit, bytes_read: read, line_count, binary })
}
