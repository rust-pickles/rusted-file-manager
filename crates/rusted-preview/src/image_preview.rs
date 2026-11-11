use crate::PreviewError;
use image::ImageReader;
use std::io::Cursor;
use std::path::Path;

/// Decoded image kept at its natural size; the UI scales it for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImagePreview {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub source_bytes: u64,
}

impl ImagePreview {
    pub fn byte_len(&self) -> usize {
        self.rgba.len()
    }

    /// Cheap nearest-neighbour downscale for thumbnails. Enough for grid tiles
    /// where the final draw is a few hundred pixels wide.
    pub fn thumbnail(&self, max_edge: u32) -> ImagePreview {
        if self.width <= max_edge && self.height <= max_edge {
            return self.clone();
        }
        let scale = max_edge as f32 / self.width.max(self.height) as f32;
        let width = (self.width as f32 * scale).round().max(1.0) as u32;
        let height = (self.height as f32 * scale).round().max(1.0) as u32;
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            let source_y = (y as f32 / scale).min((self.height - 1) as f32) as u32;
            for x in 0..width {
                let source_x = (x as f32 / scale).min((self.width - 1) as f32) as u32;
                let index = ((source_y * self.width + source_x) * 4) as usize;
                rgba.extend_from_slice(&self.rgba[index..index + 4]);
            }
        }
        ImagePreview { width, height, rgba, source_bytes: self.source_bytes }
    }
}

pub fn decode_image(path: &Path, limits: crate::PreviewLimits) -> Result<ImagePreview, PreviewError> {
    let bytes = std::fs::read(rusted_fs::long_path(path)).map_err(|error| PreviewError::io(path, error))?;
    if bytes.len() as u64 > limits.image_bytes {
        return Err(PreviewError::TooLarge(bytes.len() as u64));
    }
    decode_bytes(&bytes, limits.image_dimension)
}

pub fn decode_bytes(bytes: &[u8], max_dimension: u32) -> Result<ImagePreview, PreviewError> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|error| PreviewError::Decode(error.to_string()))?;
    if reader.format().is_none() {
        return Err(PreviewError::Decode("unknown image format".into()));
    }
    let mut image = reader.decode().map_err(|error| PreviewError::Decode(error.to_string()))?;
    if image.width() > max_dimension || image.height() > max_dimension {
        image = image.thumbnail(max_dimension, max_dimension);
    }
    let rgba = image.to_rgba8();
    Ok(ImagePreview {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
        source_bytes: bytes.len() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let buffer = image::RgbaImage::from_fn(width, height, |x, y| {
            image::Rgba([(x * 3) as u8, (y * 5) as u8, 128, 255])
        });
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(buffer)
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .unwrap();
        bytes
    }

    #[test]
    fn decodes_real_png_and_preserves_pixels() {
        let bytes = png_bytes(4, 3);
        let preview = decode_bytes(&bytes, 4096).unwrap();
        assert_eq!((preview.width, preview.height), (4, 3));
        assert_eq!(preview.rgba.len(), 4 * 3 * 4);
    }

    #[test]
    fn oversized_images_are_downscaled() {
        let bytes = png_bytes(40, 20);
        let preview = decode_bytes(&bytes, 10).unwrap();
        assert!(preview.width <= 10 && preview.height <= 10);
    }

    #[test]
    fn thumbnails_shrink_and_keep_aspect() {
        let bytes = png_bytes(20, 10);
        let preview = decode_bytes(&bytes, 4096).unwrap();
        let thumb = preview.thumbnail(10);
        assert_eq!(thumb.width, 10);
        assert_eq!(thumb.height, 5);
        assert_eq!(thumb.byte_len(), 10 * 5 * 4);
    }

    #[test]
    fn garbage_is_rejected() {
        assert!(decode_bytes(b"not an image", 4096).is_err());
    }
}
