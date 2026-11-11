//! Metadata-only media previews. Tags are read from the container header with
//! a small, dependency-free parser; nothing is decoded, so this stays cheap and
//! cannot spin up a codec the user did not ask for.

use std::path::Path;

/// Reads ID3v2 / MP4 / FLAC / OGG style tags conservatively.
pub fn audio_metadata(path: &Path, size: u64) -> Vec<(String, String)> {
    let mut fields = vec![("Size".into(), rusted_core::format_bytes(size))];
    let header = match std::fs::read(rusted_fs::long_path(path)) {
        Ok(bytes) => bytes,
        Err(error) => {
            fields.push(("Error".into(), error.to_string()));
            return fields;
        }
    };
    let head = &header[..header.len().min(256 * 1024)];
    if head.starts_with(b"ID3") {
        fields.push(("Tag".into(), "ID3v2".into()));
        if let Some(field) = id3_text(head, "TIT2", "Title") {
            fields.push(field);
        }
        if let Some(field) = id3_text(head, "TPE1", "Artist") {
            fields.push(field);
        }
        if let Some(field) = id3_text(head, "TALB", "Album") {
            fields.push(field);
        }
        if let Some(field) = id3_text(head, "TYER", "Year") {
            fields.push(field);
        }
    } else if head.starts_with(b"fLaC") {
        fields.push(("Tag".into(), "FLAC".into()));
        if let Some(info) = flac_stream_info(head) {
            fields.push(info);
        }
    } else if head.starts_with(b"OggS") {
        fields.push(("Tag".into(), "Ogg".into()));
    } else if head.len() > 12 && &head[4..8] == b"ftyp" {
        fields.push(("Tag".into(), "MPEG-4".into()));
    } else if head.starts_with(b"RIFF") && head.len() > 12 && &head[8..12] == b"WAVE" {
        fields.push(("Tag".into(), "RIFF/WAVE".into()));
        if let Some(field) = wave_format(head) {
            fields.push(field);
        }
    } else {
        fields.push(("Tag".into(), "No embedded tags found".into()));
    }
    fields
}

/// Video files reuse the MP4 container probe: duration and dimensions when the
/// boxes needed are found in the header, otherwise just honest size reporting.
pub fn video_metadata(path: &Path, size: u64) -> Vec<(String, String)> {
    let mut fields = audio_metadata(path, size);
    if let Ok(bytes) = std::fs::read(rusted_fs::long_path(path)) {
        if let Some((width, height)) = mp4_dimensions(&bytes) {
            fields.push(("Dimensions".into(), format!("{width} x {height}")));
        }
    }
    fields
}

fn id3_text(data: &[u8], frame: &str, label: &str) -> Option<(String, String)> {
    let needle = frame.as_bytes();
    let limit = data.len().min(128 * 1024);
    let start = (0..limit.saturating_sub(10)).find(|index| &data[*index..*index + 4] == needle)?;
    let size_bytes = &data[start + 4..start + 8];
    let size = ((size_bytes[0] as usize & 0x7f) << 21)
        | ((size_bytes[1] as usize & 0x7f) << 14)
        | ((size_bytes[2] as usize & 0x7f) << 7)
        | (size_bytes[3] as usize & 0x7f);
    let body = data.get(start + 10..start + 10 + size.min(4096))?;
    let text = body.get(1..).unwrap_or_default();
    let value = if body.first() == Some(&1) {
        let units: Vec<u16> = text.chunks_exact(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect();
        String::from_utf16_lossy(&units)
    } else {
        String::from_utf8_lossy(text).into_owned()
    };
    let value = value.trim_matches('\0').trim().to_string();
    (!value.is_empty()).then(|| (label.to_string(), value))
}

fn flac_stream_info(data: &[u8]) -> Option<(String, String)> {
    // Streaminfo block: sample rate (20 bits) and channels (3 bits) live in a packed word.
    let block = data.get(8..42)?;
    if block.len() < 16 {
        return None;
    }
    let packed = u32::from_be_bytes([block[10], block[11], block[12], block[13]]);
    let sample_rate = packed >> 12;
    let channels = ((packed >> 9) & 0x7) + 1;
    Some(("Stream".into(), format!("{} Hz, {channels} channels", sample_rate)))
}

fn wave_format(data: &[u8]) -> Option<(String, String)> {
    let channels = u16::from_le_bytes([*data.get(22)?, *data.get(23)?]);
    let rate = u32::from_le_bytes([*data.get(24)?, *data.get(25)?, *data.get(26)?, *data.get(27)?]);
    Some(("Stream".into(), format!("{rate} Hz, {channels} channels")))
}

fn mp4_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    let limit = data.len().min(512 * 1024);
    let window = &data[..limit];
    let index = window.windows(4).position(|bytes| bytes == b"tkhd")?;
    // tkhd: full box (4) + version/flags (4) + times/duration, then reserved and
    // two 16.16 fixed-point dimensions near the end of the box.
    let box_start = index + 4;
    let body = window.get(box_start + 4..box_start + 88 + 4)?;
    let width = u32::from_be_bytes([body[80], body[81], body[82], body[83]]) >> 16;
    let height = u32::from_be_bytes([body[84], body[85], body[86], body[87]]) >> 16;
    (width > 0 && height > 0 && width < 20_000 && height < 20_000).then_some((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_reports_an_error_field_instead_of_panicking() {
        let fields = audio_metadata(Path::new("definitely-not-here.mp3"), 0);
        assert!(fields.iter().any(|(key, _)| key == "Error"));
    }

    #[test]
    fn id3_title_is_extracted() {
        let mut data = b"ID3\x03\x00\x00".to_vec();
        data.extend_from_slice(b"TIT2");
        data.extend_from_slice(&[0, 0, 0, 6, 0, 0, 0]);
        data.extend_from_slice(b"Hello");
        let field = id3_text(&data, "TIT2", "Title").unwrap();
        assert_eq!(field, ("Title".to_string(), "Hello".to_string()));
    }
}
