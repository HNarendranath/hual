mod support;

use super::*;
use support::{entry, ifd_len, tiny_jpeg, write_header, write_ifd, IFD0_OFFSET};

// Mirrors the real (private) tags in thumbnail::tiff -- can't reference
// those directly since `jpeg` and `tiff` are sibling modules, not
// ancestor/descendant, so tiff.rs's private consts aren't visible here.
const EXIF_IFD_POINTER: u16 = 0x8769;
const TAG_EXPOSURE_TIME: u16 = 0x829A;
const TAG_ISO: u16 = 0x8827;

fn soi() -> Vec<u8> {
    vec![0xFF, 0xD8]
}

fn sos_marker() -> Vec<u8> {
    vec![0xFF, 0xDA]
}

fn write_app_segment(buf: &mut Vec<u8>, marker: u8, payload: &[u8]) {
    let seg_len = payload.len() + 2; // length field includes itself
    buf.push(0xFF);
    buf.push(marker);
    buf.extend_from_slice(&(seg_len as u16).to_be_bytes());
    buf.extend_from_slice(payload);
}

#[test]
fn find_exif_payload_locates_app1_exif_segment() {
    let mut tiff_blob = Vec::new();
    write_header(&mut tiff_blob, true);
    write_ifd(&mut tiff_blob, &[], 0, true);

    let mut exif_payload = b"Exif\0\0".to_vec();
    exif_payload.extend_from_slice(&tiff_blob);

    let mut data = soi();
    write_app_segment(&mut data, 0xE1, &exif_payload);
    data.extend_from_slice(&sos_marker());

    let found = find_exif_payload(&data).unwrap();
    assert_eq!(found, tiff_blob.as_slice());
}

#[test]
fn find_exif_payload_skips_non_exif_app1_and_other_segments() {
    let mut tiff_blob = Vec::new();
    write_header(&mut tiff_blob, false);
    write_ifd(&mut tiff_blob, &[], 0, false);

    let mut exif_payload = b"Exif\0\0".to_vec();
    exif_payload.extend_from_slice(&tiff_blob);

    let mut data = soi();
    // APP0/JFIF segment first -- the walker must not stop here
    write_app_segment(&mut data, 0xE0, b"JFIF\0\x01\x01\x00\x00\x01\x00\x01\x00\x00");
    write_app_segment(&mut data, 0xE1, &exif_payload);
    data.extend_from_slice(&sos_marker());

    let found = find_exif_payload(&data).unwrap();
    assert_eq!(found, tiff_blob.as_slice());
}

#[test]
fn find_exif_payload_absent_returns_none() {
    let mut data = soi();
    write_app_segment(&mut data, 0xE0, b"JFIF\0\x01\x01\x00\x00\x01\x00\x01\x00\x00");
    data.extend_from_slice(&sos_marker());

    assert!(find_exif_payload(&data).is_none());
}

#[test]
fn extract_exif_from_bytes_returns_all_none_when_no_exif_segment() {
    let data = tiny_jpeg(4, 4, [10, 20, 30]);
    let exif = extract_exif_from_bytes(&data).unwrap();
    assert_eq!(
        exif,
        ExifData {
            exposure_time: None,
            f_stop: None,
            iso: None,
            focal_length: None,
        }
    );
}

#[test]
fn extract_exif_from_bytes_delegates_to_tiff_parser() {
    let exif_ifd_offset = IFD0_OFFSET + ifd_len(1);
    let rational_offset = exif_ifd_offset + ifd_len(2);

    let ifd0_entries = vec![entry(EXIF_IFD_POINTER, 4, 1, exif_ifd_offset)];
    let exif_entries = vec![
        entry(TAG_ISO, 3, 1, 400),
        entry(TAG_EXPOSURE_TIME, 5, 1, rational_offset),
    ];

    let mut tiff_blob = Vec::new();
    write_header(&mut tiff_blob, true);
    write_ifd(&mut tiff_blob, &ifd0_entries, 0, true);
    write_ifd(&mut tiff_blob, &exif_entries, 0, true);
    tiff_blob.extend_from_slice(&1u32.to_le_bytes());
    tiff_blob.extend_from_slice(&250u32.to_le_bytes());

    let mut exif_payload = b"Exif\0\0".to_vec();
    exif_payload.extend_from_slice(&tiff_blob);

    let mut data = soi();
    write_app_segment(&mut data, 0xE1, &exif_payload);
    data.extend_from_slice(&sos_marker());

    let exif = extract_exif_from_bytes(&data).unwrap();
    assert_eq!(exif.iso, Some(400));
    assert_eq!(exif.exposure_time, Some((1, 250)));
}

#[test]
fn extract_thumbnail_from_bytes_returns_input_unchanged() {
    let data = tiny_jpeg(4, 4, [1, 2, 3]);
    assert_eq!(extract_thumbnail_from_bytes(&data), data);
}
