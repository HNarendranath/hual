mod support;

use super::*;
use support::{entry, ifd_len, write_header, write_ifd, TempFile, IFD0_OFFSET};

fn ifd_entry(tag: u16, field_type: u16, count: u32, value: u32) -> IfdEntry {
    IfdEntry { tag, field_type, count, value_or_offset: value }
}

#[test]
fn type_size_covers_all_tiff_types() {
    let expected = [
        (1, 1), (2, 1), (6, 1), (7, 1),
        (3, 2), (8, 2),
        (4, 4), (9, 4), (11, 4), (13, 4),
        (5, 8), (10, 8), (12, 8),
    ];
    for (field_type, size) in expected {
        assert_eq!(type_size(field_type), size, "field_type {field_type}");
    }
}

#[test]
fn type_size_unknown_type_is_zero() {
    assert_eq!(type_size(0), 0);
    assert_eq!(type_size(14), 0);
}

#[test]
fn endian_reads_little_and_big() {
    let data = [0x01, 0x02, 0x03, 0x04];
    assert_eq!(Endian::Little.read_u16(&data, 0).unwrap(), 0x0201);
    assert_eq!(Endian::Big.read_u16(&data, 0).unwrap(), 0x0102);
    assert_eq!(Endian::Little.read_u32(&data, 0).unwrap(), 0x0403_0201);
    assert_eq!(Endian::Big.read_u32(&data, 0).unwrap(), 0x0102_0304);
}

#[test]
fn endian_read_out_of_bounds_is_truncated() {
    let data = [0x01];
    assert!(matches!(Endian::Little.read_u16(&data, 0), Err(TiffError::Truncated { .. })));
}

#[test]
fn parse_header_little_endian() {
    let mut data = Vec::new();
    write_header(&mut data, true);
    let header = parse_header(&data).unwrap();
    assert_eq!(header.endian, Endian::Little);
    assert_eq!(header.ifd0_offset, IFD0_OFFSET);
}

#[test]
fn parse_header_big_endian() {
    let mut data = Vec::new();
    write_header(&mut data, false);
    let header = parse_header(&data).unwrap();
    assert_eq!(header.endian, Endian::Big);
    assert_eq!(header.ifd0_offset, IFD0_OFFSET);
}

#[test]
fn parse_header_rejects_bad_magic() {
    let data = *b"XX\x2a\x00\x08\x00\x00\x00";
    assert!(matches!(parse_header(&data), Err(TiffError::BadMagic { .. })));
}

#[test]
fn parse_header_rejects_bad_magic_number() {
    let mut data = Vec::new();
    data.extend_from_slice(b"II");
    data.extend_from_slice(&99u16.to_le_bytes());
    data.extend_from_slice(&IFD0_OFFSET.to_le_bytes());
    assert!(matches!(parse_header(&data), Err(TiffError::BadMagicNumber(99))));
}

#[test]
fn parse_header_rejects_truncated_buffer() {
    let data = b"II";
    assert!(matches!(parse_header(data), Err(TiffError::Truncated { .. })));
}

#[test]
fn read_ifd_parses_entries_and_next_offset() {
    let mut data = vec![0u8; IFD0_OFFSET as usize];
    let entries = vec![entry(0x0100, 4, 1, 1920), entry(0x0101, 4, 1, 1080)];
    write_ifd(&mut data, &entries, 0, true);

    let ifd = read_ifd(&data, Endian::Little, IFD0_OFFSET).unwrap();
    assert_eq!(ifd.entries.len(), 2);
    assert_eq!(ifd.entries[0].tag, 0x0100);
    assert_eq!(ifd.entries[0].value_or_offset, 1920);
    assert_eq!(ifd.entries[1].value_or_offset, 1080);
    assert_eq!(ifd.next_offset, 0);
}

#[test]
fn read_ifd_rejects_truncated_entry_table() {
    let mut data = vec![0u8; IFD0_OFFSET as usize];
    data.extend_from_slice(&2u16.to_le_bytes());
    assert!(matches!(
        read_ifd(&data, Endian::Little, IFD0_OFFSET),
        Err(TiffError::Truncated { .. })
    ));
}

#[test]
fn thumbnail_tags_found() {
    let ifd = Ifd {
        entries: vec![ifd_entry(0x0201, 4, 1, 100), ifd_entry(0x0202, 4, 1, 200)],
        next_offset: 0,
    };
    let loc = thumbnail_tags(&ifd).unwrap();
    assert_eq!(loc.offset, 100);
    assert_eq!(loc.length, 200);
}

#[test]
fn thumbnail_tags_missing_is_none() {
    let ifd = Ifd { entries: vec![ifd_entry(0x0201, 4, 1, 100)], next_offset: 0 };
    assert!(thumbnail_tags(&ifd).is_none());

    let ifd = Ifd { entries: vec![], next_offset: 0 };
    assert!(thumbnail_tags(&ifd).is_none());
}

#[test]
fn subifd_offsets_inline_single() {
    let ifd = Ifd { entries: vec![ifd_entry(0x014A, 4, 1, 500)], next_offset: 0 };
    let offsets = subifd_offsets(&[], Endian::Little, &ifd).unwrap();
    assert_eq!(offsets, vec![500]);
}

#[test]
fn subifd_offsets_indirect_array() {
    let mut data = vec![0u8; 16];
    data[4..8].copy_from_slice(&111u32.to_le_bytes());
    data[8..12].copy_from_slice(&222u32.to_le_bytes());

    let ifd = Ifd { entries: vec![ifd_entry(0x014A, 4, 2, 4)], next_offset: 0 };
    let offsets = subifd_offsets(&data, Endian::Little, &ifd).unwrap();
    assert_eq!(offsets, vec![111, 222]);
}

#[test]
fn subifd_offsets_absent_tag_is_empty() {
    let ifd = Ifd { entries: vec![], next_offset: 0 };
    assert_eq!(subifd_offsets(&[], Endian::Little, &ifd).unwrap(), Vec::<u32>::new());
}

#[test]
fn find_thumbnail_prefers_ifd0_own_tags_over_ifd1() {
    let ifd0_entries = vec![entry(0x0201, 4, 1, 9000), entry(0x0202, 4, 1, 500)];
    let ifd1_entries = vec![entry(0x0201, 4, 1, 1000), entry(0x0202, 4, 1, 50)];
    let ifd1_offset = IFD0_OFFSET + ifd_len(ifd0_entries.len());

    let mut data = vec![0u8; IFD0_OFFSET as usize];
    write_ifd(&mut data, &ifd0_entries, ifd1_offset, true);
    write_ifd(&mut data, &ifd1_entries, 0, true);

    // IFD0's own tags point at the higher-quality PreviewImage and take priority
    // over IFD1's (smaller, spec-mandated EXIF thumbnail) — see commit 1187727.
    let loc = find_thumbnail(&data, Endian::Little, IFD0_OFFSET).unwrap();
    assert_eq!(loc.offset, 9000);
    assert_eq!(loc.length, 500);
}

#[test]
fn find_thumbnail_falls_back_to_ifd0_tags() {
    let ifd0_entries = vec![entry(0x0201, 4, 1, 1000), entry(0x0202, 4, 1, 50)];
    let mut data = vec![0u8; IFD0_OFFSET as usize];
    write_ifd(&mut data, &ifd0_entries, 0, true);

    let loc = find_thumbnail(&data, Endian::Little, IFD0_OFFSET).unwrap();
    assert_eq!(loc.offset, 1000);
    assert_eq!(loc.length, 50);
}

#[test]
fn find_thumbnail_falls_back_to_subifd() {
    let sub_entries = vec![entry(0x0201, 4, 1, 1000), entry(0x0202, 4, 1, 50)];
    let sub_offset = IFD0_OFFSET + ifd_len(1);
    let ifd0_entries = vec![entry(0x014A, 4, 1, sub_offset)];

    let mut data = vec![0u8; IFD0_OFFSET as usize];
    write_ifd(&mut data, &ifd0_entries, 0, true);
    write_ifd(&mut data, &sub_entries, 0, true);

    let loc = find_thumbnail(&data, Endian::Little, IFD0_OFFSET).unwrap();
    assert_eq!(loc.offset, 1000);
    assert_eq!(loc.length, 50);
}

#[test]
fn find_thumbnail_not_found() {
    let mut data = vec![0u8; IFD0_OFFSET as usize];
    write_ifd(&mut data, &[], 0, true);
    assert!(matches!(
        find_thumbnail(&data, Endian::Little, IFD0_OFFSET),
        Err(TiffError::ThumbnailNotFound)
    ));
}

#[test]
fn extract_thumbnail_end_to_end() {
    let thumb_bytes = b"\xFF\xD8fake-thumbnail-data\xFF\xD9".to_vec();

    let ifd1_offset = IFD0_OFFSET + ifd_len(0);
    let thumb_offset = ifd1_offset + ifd_len(2);
    let ifd1_entries = vec![
        entry(0x0201, 4, 1, thumb_offset),
        entry(0x0202, 4, 1, thumb_bytes.len() as u32),
    ];

    let mut data = Vec::new();
    write_header(&mut data, true);
    write_ifd(&mut data, &[], ifd1_offset, true);
    write_ifd(&mut data, &ifd1_entries, 0, true);
    data.extend_from_slice(&thumb_bytes);

    let file = TempFile::new("extract_thumbnail_end_to_end", &data);
    let result = extract_thumbnail(&file.path).unwrap();
    assert_eq!(result, thumb_bytes);
}

#[test]
fn extract_thumbnail_missing_file_is_io_error() {
    let path = std::env::temp_dir().join("hual_test_does_not_exist_12345.tiff");
    assert!(matches!(extract_thumbnail(&path), Err(TiffError::Io(_))));
}

// === EXIF tests ===

#[test]
fn read_rational_reads_num_and_denom() {
    let mut data = vec![0u8; 8];
    data[0..4].copy_from_slice(&1u32.to_le_bytes());
    data[4..8].copy_from_slice(&200u32.to_le_bytes());
    assert_eq!(read_rational(&data, Endian::Little, 0).unwrap(), (1, 200));

    let mut data_be = vec![0u8; 8];
    data_be[0..4].copy_from_slice(&1u32.to_be_bytes());
    data_be[4..8].copy_from_slice(&200u32.to_be_bytes());
    assert_eq!(read_rational(&data_be, Endian::Big, 0).unwrap(), (1, 200));
}

#[test]
fn read_rational_rejects_truncated() {
    let data = [0u8; 4];
    assert!(matches!(
        read_rational(&data, Endian::Little, 0),
        Err(TiffError::Truncated { .. })
    ));
}

#[test]
fn inline_u16_little_endian_uses_low_bytes() {
    let e = ifd_entry(TAG_ISO, 3, 1, 800);
    assert_eq!(inline_u16(&e, Endian::Little), 800);
}

#[test]
fn inline_u16_big_endian_uses_high_bytes() {
    let e = ifd_entry(TAG_ISO, 3, 1, 800u32 << 16);
    assert_eq!(inline_u16(&e, Endian::Big), 800);
}

#[test]
fn exif_ifd_offset_found() {
    let ifd = Ifd { entries: vec![ifd_entry(EXIF_IFD_POINTER, 4, 1, 500)], next_offset: 0 };
    assert_eq!(exif_ifd_offset(&ifd), Some(500));
}

#[test]
fn exif_ifd_offset_missing_is_none() {
    let ifd = Ifd { entries: vec![], next_offset: 0 };
    assert_eq!(exif_ifd_offset(&ifd), None);
}

#[test]
fn exif_tags_reads_all_present() {
    let mut data = vec![0u8; 24];
    data[0..4].copy_from_slice(&1u32.to_le_bytes());
    data[4..8].copy_from_slice(&250u32.to_le_bytes());
    data[8..12].copy_from_slice(&28u32.to_le_bytes());
    data[12..16].copy_from_slice(&10u32.to_le_bytes());
    data[16..20].copy_from_slice(&50u32.to_le_bytes());
    data[20..24].copy_from_slice(&1u32.to_le_bytes());

    let ifd = Ifd {
        entries: vec![
            ifd_entry(TAG_EXPOSURE_TIME, 5, 1, 0),
            ifd_entry(TAG_F_STOP, 5, 1, 8),
            ifd_entry(TAG_ISO, 3, 1, 400),
            ifd_entry(TAG_FOCAL_LENGTH, 5, 1, 16),
        ],
        next_offset: 0,
    };

    let exif = exif_tags(&data, Endian::Little, &ifd).unwrap();
    assert_eq!(exif.exposure_time, Some((1, 250)));
    assert_eq!(exif.f_stop, Some((28, 10)));
    assert_eq!(exif.iso, Some(400));
    assert_eq!(exif.focal_length, Some((50, 1)));
}

#[test]
fn exif_tags_missing_tags_are_all_none() {
    let ifd = Ifd { entries: vec![], next_offset: 0 };
    let exif = exif_tags(&[], Endian::Little, &ifd).unwrap();
    assert_eq!(exif, ExifData { exposure_time: None, f_stop: None, iso: None, focal_length: None });
}

#[test]
fn exif_tags_partial_only_iso_present() {
    let ifd = Ifd { entries: vec![ifd_entry(TAG_ISO, 3, 1, 100)], next_offset: 0 };
    let exif = exif_tags(&[], Endian::Little, &ifd).unwrap();
    assert_eq!(exif.exposure_time, None);
    assert_eq!(exif.f_stop, None);
    assert_eq!(exif.iso, Some(100));
    assert_eq!(exif.focal_length, None);
}

#[test]
fn extract_exif_from_bytes_reads_full_exif_ifd() {
    let exif_ifd_offset = IFD0_OFFSET + ifd_len(1);
    let rational1_offset = exif_ifd_offset + ifd_len(4);
    let rational2_offset = rational1_offset + 8;
    let rational3_offset = rational2_offset + 8;

    let ifd0_entries = vec![entry(EXIF_IFD_POINTER, 4, 1, exif_ifd_offset)];
    let exif_entries = vec![
        entry(TAG_EXPOSURE_TIME, 5, 1, rational1_offset),
        entry(TAG_F_STOP, 5, 1, rational2_offset),
        entry(TAG_ISO, 3, 1, 400),
        entry(TAG_FOCAL_LENGTH, 5, 1, rational3_offset),
    ];

    let mut data = Vec::new();
    write_header(&mut data, true);
    write_ifd(&mut data, &ifd0_entries, 0, true);
    write_ifd(&mut data, &exif_entries, 0, true);
    data.extend_from_slice(&1u32.to_le_bytes());
    data.extend_from_slice(&250u32.to_le_bytes());
    data.extend_from_slice(&28u32.to_le_bytes());
    data.extend_from_slice(&10u32.to_le_bytes());
    data.extend_from_slice(&50u32.to_le_bytes());
    data.extend_from_slice(&1u32.to_le_bytes());

    let exif = extract_exif_from_bytes(&data).unwrap();
    assert_eq!(exif.exposure_time, Some((1, 250)));
    assert_eq!(exif.f_stop, Some((28, 10)));
    assert_eq!(exif.iso, Some(400));
    assert_eq!(exif.focal_length, Some((50, 1)));
}

#[test]
fn extract_exif_from_bytes_reads_full_exif_ifd_big_endian() {
    let exif_ifd_offset = IFD0_OFFSET + ifd_len(1);
    let rational1_offset = exif_ifd_offset + ifd_len(2);

    let ifd0_entries = vec![entry(EXIF_IFD_POINTER, 4, 1, exif_ifd_offset)];
    let exif_entries = vec![
        entry(TAG_ISO, 3, 1, 400u32 << 16),
        entry(TAG_EXPOSURE_TIME, 5, 1, rational1_offset),
    ];

    let mut data = Vec::new();
    write_header(&mut data, false);
    write_ifd(&mut data, &ifd0_entries, 0, false);
    write_ifd(&mut data, &exif_entries, 0, false);
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&320u32.to_be_bytes());

    let exif = extract_exif_from_bytes(&data).unwrap();
    assert_eq!(exif.iso, Some(400));
    assert_eq!(exif.exposure_time, Some((1, 320)));
    assert_eq!(exif.f_stop, None);
    assert_eq!(exif.focal_length, None);
}

#[test]
fn extract_exif_from_bytes_missing_exif_pointer_returns_all_none() {
    let mut data = Vec::new();
    write_header(&mut data, true);
    write_ifd(&mut data, &[], 0, true);

    let exif = extract_exif_from_bytes(&data).unwrap();
    assert_eq!(exif, ExifData { exposure_time: None, f_stop: None, iso: None, focal_length: None });
}

#[test]
fn extract_exif_end_to_end() {
    let exif_ifd_offset = IFD0_OFFSET + ifd_len(1);
    let ifd0_entries = vec![entry(EXIF_IFD_POINTER, 4, 1, exif_ifd_offset)];
    let exif_entries = vec![entry(TAG_ISO, 3, 1, 200)];

    let mut data = Vec::new();
    write_header(&mut data, true);
    write_ifd(&mut data, &ifd0_entries, 0, true);
    write_ifd(&mut data, &exif_entries, 0, true);

    let file = TempFile::new("extract_exif_end_to_end", &data);
    let exif = extract_exif(&file.path).unwrap();
    assert_eq!(exif.iso, Some(200));
    assert_eq!(exif.exposure_time, None);
    assert_eq!(exif.f_stop, None);
    assert_eq!(exif.focal_length, None);
}

#[test]
fn extract_exif_missing_file_is_io_error() {
    let path = std::env::temp_dir().join("hual_test_does_not_exist_exif_12345.tiff");
    assert!(matches!(extract_exif(&path), Err(TiffError::Io(_))));
}
