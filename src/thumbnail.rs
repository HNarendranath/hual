use crate::cr3;
use crate::tiff;

use std::path::Path;

pub fn extract_thumbnail(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    const TIFF_EXTENSIONS: &[&str] = &[
        "tiff", "tif", "arw", "cr2", "nef", "raf", "orf", "dng", "nrw", "rw2", "pef", "iiq", "3fr",
        "fff", "sr2", "srf",
    ];

    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let ext_lowercase = extension.to_ascii_lowercase();
    let result: Result<(Vec<u8>), Box<dyn std::error::Error>> = if ext_lowercase == "cr3" {
        cr3::extract_thumbnail(&path).map_err(|e| e.into())
    } else if TIFF_EXTENSIONS.contains(&ext_lowercase.as_str()) {
        tiff::extract_thumbnail(&path).map_err(|e| e.into())
    } else if ext_lowercase.is_empty() {
        Err(format!("File '{}' has no extension", path.display()).into())
    } else {
        Err(format!("Unsupported file format: '.{}'", extension).into())
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
