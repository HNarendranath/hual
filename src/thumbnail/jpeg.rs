use super::tiff::{self, ExifData, TiffError};

fn find_exif_payload(data: &[u8]) -> Option<&[u8]> {
    if data.len() < 2 || data[0] != 0xff || data[1] != 0xd8 {
        return None;
    }

    let mut pos = 2;
    while pos + 4 < data.len() {
        if data[pos] != 0xff {
            return None;
        }
        let marker = data[pos + 1];

        if marker == 0xD8 || marker == 0xD9 {
            pos += 2;
            continue;
        }
        if marker == 0xDA {
            break;
        }
        let segment_length = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
        if segment_length < 2 || pos + 2 + segment_length > data.len() {
            return None;
        }
        let payload = &data[pos + 4..pos + 2 + segment_length];
        if marker == 0xE1 && payload.starts_with(b"Exif\0\0") {
            return Some(&payload[6..]);
        }
        pos += 2 + segment_length;
    }
    None
}

pub fn extract_exif_from_bytes(data: &[u8]) -> Result<ExifData, TiffError> {
    match find_exif_payload(data) {
        Some(exif_payload) => tiff::extract_exif_from_bytes(exif_payload),
        None => Ok(ExifData {
            exposure_time: None,
            f_stop: None,
            iso: None,
            focal_length: None,
        }),
    }
}

pub fn extract_thumbnail_from_bytes(data: &[u8]) -> Vec<u8> {
    data.to_vec() // the jpeg is already a thumbnail
}

pub fn extract_thumbnail(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    std::fs::read(path)
}

#[cfg(test)]
#[path = "../../tests/unit/jpeg_tests.rs"]
mod jpeg_tests;
