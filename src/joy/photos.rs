//! One photo per kept date: decoded, turned the right way up (EXIF), shrunk to
//! fit 1600px and stored as <dir>/<history id>.jpg. Never served from /static.

use std::{io::Cursor, path::PathBuf, sync::LazyLock};

use image::{codecs::jpeg::JpegEncoder, DynamicImage, ImageDecoder, ImageReader, Limits};
use tokio::sync::Semaphore;

pub const MAX_UPLOAD_BYTES: usize = 15 * 1024 * 1024;
const MAX_SIDE: u32 = 1600;
/// Declared size cap, checked from the header before any pixels are decoded. `max_alloc` does not
/// bound JPEG decoding in image 0.25, so this is what keeps a 12000x12000 file from costing ~430 MB.
const MAX_PIXELS: u64 = 40_000_000;
/// At most this many photos decode at once, so a burst of uploads can't exhaust a 512 MB VM.
static DECODE_SLOTS: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(2));

pub fn path(dir: &std::path::Path, history_id: i64) -> PathBuf {
    dir.join(format!("{history_id}.jpg"))
}

/// A night at a regular: <dir>/v<visit id>.jpg, so it can never collide with a kept date's file.
pub fn visit_path(dir: &std::path::Path, visit_id: i64) -> PathBuf {
    dir.join(format!("v{visit_id}.jpg"))
}

/// A message safe to show the person when a photo can't be used.
#[derive(Debug)]
pub struct PhotoError(pub &'static str);

/// Decode, orient and shrink; returns JPEG bytes. CPU-heavy: call from spawn_blocking.
pub fn process(bytes: &[u8]) -> Result<Vec<u8>, PhotoError> {
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err(PhotoError("That photo is over 15 MB. Try a smaller one."));
    }
    if looks_like_heic(bytes) {
        return Err(PhotoError("That's an HEIC photo, which can't be read here. Choose “Most Compatible” in camera settings, or send a screenshot."));
    }
    let header = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| PhotoError("That file doesn't look like a photo."))?;
    let (w, h) = header.into_dimensions().map_err(|_| PhotoError("That file doesn't look like a photo."))?;
    check_pixels(w, h)?;
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| PhotoError("That file doesn't look like a photo."))?;
    // Keep decoding inside what a small VM can afford.
    let mut limits = Limits::default();
    limits.max_image_width = Some(12_000);
    limits.max_image_height = Some(12_000);
    limits.max_alloc = Some(160 * 1024 * 1024);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(|_| PhotoError("That file doesn't look like a photo."))?;
    let orientation = decoder.orientation().ok();
    let mut img = DynamicImage::from_decoder(decoder).map_err(|_| PhotoError("That photo couldn't be read. Try another one."))?;
    if let Some(o) = orientation {
        img.apply_orientation(o);
    }
    let img = if img.width() > MAX_SIDE || img.height() > MAX_SIDE { img.thumbnail(MAX_SIDE, MAX_SIDE) } else { img };
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, 82)
        .encode_image(&img.to_rgb8())
        .map_err(|_| PhotoError("That photo couldn't be saved. Try another one."))?;
    Ok(out)
}

/// Refuses a declared size above [`MAX_PIXELS`]. Takes the header's numbers, so it never decodes.
fn check_pixels(w: u32, h: u32) -> Result<(), PhotoError> {
    if u64::from(w) * u64::from(h) > MAX_PIXELS {
        Err(PhotoError("That photo is too big to read here. Try a smaller one."))
    } else {
        Ok(())
    }
}

/// [`process`] on a blocking thread, one of at most two at a time. The outer `Err` is a join failure.
pub async fn process_limited(bytes: Vec<u8>) -> Result<Result<Vec<u8>, PhotoError>, tokio::task::JoinError> {
    let _slot = DECODE_SLOTS.acquire().await.expect("decode semaphore is never closed");
    tokio::task::spawn_blocking(move || process(&bytes)).await
}

/// HEIC/HEIF files carry an "ftyp" box with a heic-family brand near the start.
fn looks_like_heic(b: &[u8]) -> bool {
    b.len() > 12 && &b[4..8] == b"ftyp" && matches!(&b[8..12], b"heic" | b"heix" | b"hevc" | b"hevx" | b"mif1" | b"msf1" | b"heim" | b"heis")
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, RgbImage};

    #[test]
    fn shrinks_big_photos_to_a_jpeg() {
        let mut png = Vec::new();
        DynamicImage::ImageRgb8(RgbImage::new(3200, 2000)).write_to(&mut Cursor::new(&mut png), ImageFormat::Png).unwrap();
        let out = process(&png).unwrap();
        let back = image::load_from_memory(&out).unwrap();
        assert_eq!((back.width(), back.height()), (1600, 1000));
        assert_eq!(image::guess_format(&out).unwrap(), ImageFormat::Jpeg);
    }

    #[test]
    fn refuses_huge_declared_sizes_before_decoding() {
        assert!(check_pixels(12_000, 12_000).is_err());
        assert!(check_pixels(u32::MAX, u32::MAX).is_err());
        assert!(check_pixels(8_000, 6_000).is_err());
        assert!(check_pixels(7_000, 5_000).is_ok());
        assert!(check_pixels(3_200, 2_000).is_ok());
    }

    #[test]
    fn reads_dimensions_from_the_header() {
        let mut png = Vec::new();
        DynamicImage::ImageRgb8(RgbImage::new(3200, 2000)).write_to(&mut Cursor::new(&mut png), ImageFormat::Png).unwrap();
        let (w, h) = ImageReader::new(Cursor::new(&png)).with_guessed_format().unwrap().into_dimensions().unwrap();
        assert_eq!((w, h), (3200, 2000));
    }

    #[test]
    fn says_why_it_cant_use_a_file() {
        let heic = [&[0, 0, 0, 24][..], b"ftypheic", &[0; 16]].concat();
        assert!(process(&heic).unwrap_err().0.contains("HEIC"));
        assert!(process(b"definitely not a photo").is_err());
    }
}
