mod cr3;
mod tiff;
pub use tiff::ExifData;

use std::path::Path;

const TIFF_EXTENSIONS: &[&str] = &[
    "tiff", "tif", "arw", "cr2", "nef", "raf", "orf", "dng", "nrw", "rw2", "pef", "iiq", "3fr",
    "fff", "sr2", "srf",
];

enum Format {
    Cr3,
    Tiff,
}

fn get_fmt(extension: &str) -> Option<Format> {
    let ext_lowercase = extension.to_ascii_lowercase();
    if ext_lowercase == "cr3" {
        Some(Format::Cr3)
    } else if TIFF_EXTENSIONS.contains(&ext_lowercase.as_str()) {
        Some(Format::Tiff)
    } else {
        None
    }
}

pub fn extract_thumbnail_from_bytes(
    data: &[u8],
    ext: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match get_fmt(ext) {
        Some(Format::Cr3) => cr3::extract_thumbnail_from_bytes(data).map_err(Into::into),
        Some(Format::Tiff) => tiff::extract_thumbnail_from_bytes(data).map_err(Into::into),
        None if ext.is_empty() => Err("no file extension given".into()),
        None => Err(format!("Unsupported file format: '.{}'", ext).into()),
    }
}

pub fn extract_thumbnail(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let result: Result<Vec<u8>, Box<dyn std::error::Error>> = match get_fmt(extension) {
        Some(Format::Cr3) => cr3::extract_thumbnail(path).map_err(Into::into),
        Some(Format::Tiff) => tiff::extract_thumbnail(path).map_err(Into::into),
        None if extension.is_empty() => {
            Err(format!("File '{}' has no extension", path.display()).into())
        }
        None => Err(format!("Unsupported file format: '.{}'", extension).into()),
    };

    let bytes = match result {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error extracting thumbnail from {}: {e}", path.display());
            return Err(e);
        }
    };
    Ok(bytes)
}

// EXIF

pub fn extract_exif_from_bytes(
    data: &[u8],
    ext: &str,
) -> Result<tiff::ExifData, Box<dyn std::error::Error>> {
    match get_fmt(ext) {
        Some(Format::Cr3) => cr3::extract_exif_from_bytes(data).map_err(Into::into),
        Some(Format::Tiff) => tiff::extract_exif_from_bytes(data).map_err(Into::into),
        None if ext.is_empty() => Err("no file extension given".into()),
        None => Err(format!("Unsupported file format: '.{}'", ext).into()),
    }
}
