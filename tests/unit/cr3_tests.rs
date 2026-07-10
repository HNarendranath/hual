mod support;

use super::*;
use support::{write_box, write_ftyp, write_uuid_box, TempFile};

// ---- read_u32_be / read_u64_be ----

#[test]
fn read_u32_be_reads_big_endian() {
    let data = [0x00, 0x00, 0x01, 0x02];
    assert_eq!(read_u32_be(&data, 0).unwrap(), 0x0102);
}

#[test]
fn read_u32_be_out_of_bounds_is_truncated() {
    let data = [0x00, 0x00];
    assert!(matches!(read_u32_be(&data, 0), Err(Cr3Error::Truncated { .. })));
}

#[test]
fn read_u64_be_reads_big_endian() {
    let data = [0, 0, 0, 0, 0, 0, 0x01, 0x02];
    assert_eq!(read_u64_be(&data, 0).unwrap(), 0x0102);
}

#[test]
fn read_u64_be_out_of_bounds_is_truncated() {
    let data = [0u8; 4];
    assert!(matches!(read_u64_be(&data, 0), Err(Cr3Error::Truncated { .. })));
}

// ---- read_box_header ----

#[test]
fn read_box_header_parses_normal_32_bit_box() {
    let mut data = Vec::new();
    write_box(&mut data, b"free", b"hello");

    let header = read_box_header(&data, 0, data.len()).unwrap();
    assert_eq!(header.box_type, *b"free");
    assert_eq!(header.header_len, 8);
    assert_eq!(header.payload_offset, 8);
    assert_eq!(header.payload_len, 5);
    assert_eq!(header.usertype, None);
}

#[test]
fn read_box_header_parses_largesize_64_bit_box() {
    let payload = vec![0xABu8; 20];
    let total_size: u64 = 16 + payload.len() as u64;

    let mut data = Vec::new();
    data.extend_from_slice(&1u32.to_be_bytes()); // size == 1 -> largesize follows
    data.extend_from_slice(b"free");
    data.extend_from_slice(&total_size.to_be_bytes());
    data.extend_from_slice(&payload);

    let header = read_box_header(&data, 0, data.len()).unwrap();
    assert_eq!(header.box_type, *b"free");
    assert_eq!(header.header_len, 16);
    assert_eq!(header.payload_offset, 16);
    assert_eq!(header.payload_len, 20);
}

#[test]
fn read_box_header_size_zero_extends_to_container_end_not_data_end() {
    let mut data = Vec::new();
    data.extend_from_slice(&0u32.to_be_bytes()); // size == 0 -> extends to container end
    data.extend_from_slice(b"mdat");
    data.extend_from_slice(&[0xCD; 40]); // far more bytes physically present than the container allows

    let container_end = 8 + 12;
    let header = read_box_header(&data, 0, container_end).unwrap();
    assert_eq!(header.payload_offset, 8);
    assert_eq!(header.payload_len, 12);
}

#[test]
fn read_box_header_parses_uuid_box_usertype() {
    let usertype = [0x11u8; 16];
    let mut data = Vec::new();
    write_uuid_box(&mut data, &usertype, b"payload-bytes");

    let header = read_box_header(&data, 0, data.len()).unwrap();
    assert_eq!(header.box_type, *b"uuid");
    assert_eq!(header.header_len, 24); // 8-byte base header + 16-byte usertype
    assert_eq!(header.usertype, Some(usertype));
    assert_eq!(header.payload_offset, 24);
    assert_eq!(header.payload_len, "payload-bytes".len());
}

#[test]
fn read_box_header_rejects_size_smaller_than_own_header() {
    let mut data = Vec::new();
    data.extend_from_slice(&4u32.to_be_bytes()); // declares size 4, smaller than the 8-byte header itself
    data.extend_from_slice(b"free");

    assert!(matches!(
        read_box_header(&data, 0, data.len()),
        Err(Cr3Error::BoxTooShort { size: 4, header_len: 8, .. })
    ));
}

#[test]
fn read_box_header_truncated_size_field() {
    let data = [0x00, 0x00]; // fewer than the 4 bytes needed for the size field
    assert!(matches!(read_box_header(&data, 0, data.len()), Err(Cr3Error::Truncated { .. })));
}

#[test]
fn read_box_header_truncated_type_field() {
    let mut data = Vec::new();
    data.extend_from_slice(&100u32.to_be_bytes());
    data.extend_from_slice(b"fr"); // only 2 of the 4 fourCC bytes present

    assert!(matches!(read_box_header(&data, 0, data.len()), Err(Cr3Error::Truncated { .. })));
}

#[test]
fn read_box_header_truncated_largesize_field() {
    let mut data = Vec::new();
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(b"free");
    data.extend_from_slice(&[0u8; 4]); // only 4 of the 8 largesize bytes present

    assert!(matches!(read_box_header(&data, 0, data.len()), Err(Cr3Error::Truncated { .. })));
}

#[test]
fn read_box_header_truncated_uuid_usertype() {
    let mut data = Vec::new();
    data.extend_from_slice(&100u32.to_be_bytes());
    data.extend_from_slice(b"uuid");
    data.extend_from_slice(&[0u8; 10]); // only 10 of the 16 usertype bytes present

    assert!(matches!(read_box_header(&data, 0, data.len()), Err(Cr3Error::Truncated { .. })));
}

#[test]
fn read_box_header_rejects_box_extending_past_container_end() {
    let mut data = Vec::new();
    write_box(&mut data, b"free", &[0u8; 20]); // a real, well-formed, complete box

    // but the surrounding container is claimed to end before this box's payload actually finishes
    assert!(matches!(
        read_box_header(&data, 0, 10),
        Err(Cr3Error::Truncated { .. })
    ));
}

#[test]
fn read_box_header_offset_plus_size_overflow_is_box_size_overflow() {
    let mut data = vec![0u8; 8]; // padding so the box doesn't start at offset 0
    data.extend_from_slice(&1u32.to_be_bytes()); // size == 1 -> largesize follows
    data.extend_from_slice(b"free");
    data.extend_from_slice(&u64::MAX.to_be_bytes()); // largest possible declared size

    // offset(8) + total_size(u64::MAX) overflows usize
    assert!(matches!(
        read_box_header(&data, 8, data.len()),
        Err(Cr3Error::BoxSizeOverflow { offset: 8 })
    ));
}

// ---- read_boxes ----

#[test]
fn read_boxes_parses_multiple_siblings_in_order() {
    let mut data = Vec::new();
    write_box(&mut data, b"free", b"aaa");
    write_box(&mut data, b"skip", b"bb");
    write_box(&mut data, b"wide", b"c");

    let boxes = read_boxes(&data, 0, data.len()).unwrap();
    let types: Vec<[u8; 4]> = boxes.iter().map(|b| b.box_type).collect();
    assert_eq!(types, vec![*b"free", *b"skip", *b"wide"]);
}

#[test]
fn read_boxes_empty_range_returns_empty_vec() {
    let data = Vec::new();
    let boxes = read_boxes(&data, 0, 0).unwrap();
    assert!(boxes.is_empty());
}

#[test]
fn read_boxes_propagates_error_from_truncated_final_box() {
    let mut data = Vec::new();
    write_box(&mut data, b"free", b"aaa"); // one well-formed box

    data.extend_from_slice(&20u32.to_be_bytes()); // a second box header claiming 20 bytes total
    data.extend_from_slice(b"bad!"); // but no payload bytes actually follow

    assert!(matches!(read_boxes(&data, 0, data.len()), Err(Cr3Error::Truncated { .. })));
}

// ---- find_jpeg_soi ----

#[test]
fn find_jpeg_soi_at_start() {
    let payload = [0xFF, 0xD8, 0x01, 0x02];
    assert_eq!(find_jpeg_soi(&payload), Some(0));
}

#[test]
fn find_jpeg_soi_mid_buffer() {
    let payload = [0x00, 0x11, 0xFF, 0xD8, 0x22];
    assert_eq!(find_jpeg_soi(&payload), Some(2));
}

#[test]
fn find_jpeg_soi_absent_returns_none() {
    let payload = [0x00, 0xFF, 0x00, 0xD8]; // FF and D8 both present but never adjacent
    assert_eq!(find_jpeg_soi(&payload), None);
}

#[test]
fn find_jpeg_soi_empty_payload_returns_none() {
    assert_eq!(find_jpeg_soi(&[]), None);
}

// ---- find_thumbnail / extract_thumbnail ----

fn build_valid_cr3(jpeg: &[u8]) -> Vec<u8> {
    let mut prvw_payload = vec![0u8; PRVW_JPEG_SIZE_FIELD_OFFSET]; // unknown:u32, unknown:u16, width:u16, height:u16, unknown:u16
    prvw_payload.extend_from_slice(&(jpeg.len() as u32).to_be_bytes()); // jpeg_size
    prvw_payload.extend_from_slice(jpeg);

    let mut prvw_box = Vec::new();
    write_box(&mut prvw_box, b"PRVW", &prvw_payload);

    let mut uuid_payload = vec![0u8; PRVW_UUID_PAYLOAD_SKIP]; // undocumented gap before PRVW's siblings start
    uuid_payload.extend_from_slice(&prvw_box);

    let mut data = Vec::new();
    write_ftyp(&mut data, b"crx ");
    write_uuid_box(&mut data, &CANON_PREVIEW_UUID, &uuid_payload); // top-level, sibling of ftyp
    data
}

#[test]
fn find_thumbnail_locates_jpeg_inside_prvw_box() {
    let jpeg = b"\xFF\xD8fake-jpeg-bytes\xFF\xD9";
    let data = build_valid_cr3(jpeg);

    let location = find_thumbnail(&data).unwrap();
    assert_eq!(&data[location.offset..location.offset + location.length], jpeg.as_slice());
}

#[test]
fn find_thumbnail_rejects_non_isobmff_file() {
    let mut data = Vec::new();
    write_box(&mut data, b"free", b"not ftyp"); // well-formed box, but not "ftyp"

    assert!(matches!(find_thumbnail(&data), Err(Cr3Error::NotAnIsobmff { .. })));
}

#[test]
fn find_thumbnail_rejects_wrong_major_brand() {
    let mut data = Vec::new();
    write_ftyp(&mut data, b"isom"); // valid ftyp box, wrong brand for CR3

    assert!(matches!(
        find_thumbnail(&data),
        Err(Cr3Error::BadFtype { major_brand }) if major_brand == *b"isom"
    ));
}

#[test]
fn find_thumbnail_missing_preview_uuid_box() {
    let mut data = Vec::new();
    write_ftyp(&mut data, b"crx "); // no top-level "uuid" box at all follows

    assert!(matches!(
        find_thumbnail(&data),
        Err(Cr3Error::MissingBox { fourcc, .. }) if fourcc == *b"uuid"
    ));
}

#[test]
fn find_thumbnail_unrelated_top_level_boxes_dont_satisfy_uuid_lookup() {
    let mut data = Vec::new();
    write_ftyp(&mut data, b"crx ");
    write_box(&mut data, b"moov", b"unrelated-moov-data"); // present, but irrelevant to the new design

    assert!(matches!(
        find_thumbnail(&data),
        Err(Cr3Error::MissingBox { fourcc, .. }) if fourcc == *b"uuid"
    ));
}

#[test]
fn find_thumbnail_top_level_uuid_with_wrong_usertype_is_missing() {
    let wrong_usertype = [0xEE; 16]; // a top-level "uuid" box exists, but it isn't Canon's specific one
    let mut data = Vec::new();
    write_ftyp(&mut data, b"crx ");
    write_uuid_box(&mut data, &wrong_usertype, b"irrelevant");

    assert!(matches!(
        find_thumbnail(&data),
        Err(Cr3Error::MissingBox { fourcc, .. }) if fourcc == *b"uuid"
    ));
}

#[test]
fn find_thumbnail_preview_uuid_present_without_prvw_box() {
    let uuid_payload = vec![0u8; PRVW_UUID_PAYLOAD_SKIP]; // skip bytes present, but no PRVW child after them

    let mut data = Vec::new();
    write_ftyp(&mut data, b"crx ");
    write_uuid_box(&mut data, &CANON_PREVIEW_UUID, &uuid_payload);

    assert!(matches!(
        find_thumbnail(&data),
        Err(Cr3Error::MissingBox { fourcc, .. }) if fourcc == *b"PRVW"
    ));
}

#[test]
fn find_thumbnail_prvw_jpeg_size_larger_than_payload_is_box_too_short() {
    let mut prvw_payload = vec![0u8; PRVW_JPEG_SIZE_FIELD_OFFSET];
    prvw_payload.extend_from_slice(&999u32.to_be_bytes()); // jpeg_size claims far more than actually follows
    prvw_payload.extend_from_slice(b"only-a-few-bytes");

    let mut prvw_box = Vec::new();
    write_box(&mut prvw_box, b"PRVW", &prvw_payload);

    let mut uuid_payload = vec![0u8; PRVW_UUID_PAYLOAD_SKIP];
    uuid_payload.extend_from_slice(&prvw_box);

    let mut data = Vec::new();
    write_ftyp(&mut data, b"crx ");
    write_uuid_box(&mut data, &CANON_PREVIEW_UUID, &uuid_payload);

    assert!(matches!(find_thumbnail(&data), Err(Cr3Error::BoxTooShort { .. })));
}

#[test]
fn extract_thumbnail_end_to_end() {
    let jpeg = b"\xFF\xD8real-fake-jpeg-payload\xFF\xD9".to_vec();
    let data = build_valid_cr3(&jpeg);
    let file = TempFile::new("cr3_extract_thumbnail_end_to_end", &data);

    let result = extract_thumbnail(&file.path).unwrap();
    assert_eq!(result, jpeg);
}

#[test]
fn extract_thumbnail_missing_file_is_io_error() {
    let path = std::env::temp_dir().join("hual_test_cr3_does_not_exist_98765.cr3");
    assert!(matches!(extract_thumbnail(&path), Err(Cr3Error::Io(_))));
}
