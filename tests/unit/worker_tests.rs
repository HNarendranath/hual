mod support;

use super::*;
use crate::thumbnail::ExifData;
use crossbeam_channel::unbounded;
use std::path::PathBuf;
use support::{entry, ifd_len, write_header, write_ifd, IFD0_OFFSET};

fn valid_tiff_with_thumbnail() -> Vec<u8> {
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
    data
}

fn run_worker_on(
    files: Vec<RawFile>,
    source_dir: &Path,
    dest_dir: &Path,
) -> (Vec<WriteJob>, Vec<MetadataRecord>) {
    let (raw_tx, raw_rx) = unbounded();
    let (write_tx, write_rx) = unbounded();
    let (db_tx, db_rx) = unbounded();

    for f in files {
        raw_tx.send(f).unwrap();
    }
    drop(raw_tx);

    run(raw_rx, write_tx, db_tx, source_dir, dest_dir);

    (write_rx.iter().collect(), db_rx.iter().collect())
}

#[test]
fn successful_extraction_populates_thumbnail_and_exif() {
    let source_dir = PathBuf::from("/source");
    let dest_dir = PathBuf::from("/dest");

    let files = vec![RawFile {
        src_path: source_dir.join("photo.arw"),
        bytes: valid_tiff_with_thumbnail(),
    }];

    let (jobs, records) = run_worker_on(files, &source_dir, &dest_dir);

    assert_eq!(jobs.len(), 1);
    assert_eq!(records.len(), 1);
    assert!(records[0].thumbnail.is_some());
    // no EXIF_IFD_POINTER in this synthetic IFD0, so extraction succeeds
    // (Ok, not Err) but every field is None -- exif is Some(_), not None.
    assert_eq!(
        records[0].exif,
        Some(ExifData { exposure_time: None, f_stop: None, iso: None })
    );
}

#[test]
fn failed_extraction_still_produces_job_and_record_with_none_fields() {
    let source_dir = PathBuf::from("/source");
    let dest_dir = PathBuf::from("/dest");

    let files = vec![RawFile {
        src_path: source_dir.join("garbage.arw"),
        bytes: b"not a tiff file at all".to_vec(),
    }];

    let (jobs, records) = run_worker_on(files, &source_dir, &dest_dir);

    assert_eq!(jobs.len(), 1);
    assert_eq!(records.len(), 1);
    assert!(records[0].thumbnail.is_none());
    assert!(records[0].exif.is_none());
    // bytes still get forwarded for copying even though extraction failed
    assert_eq!(jobs[0].bytes, b"not a tiff file at all");
}

#[test]
fn unsupported_extension_still_copies_with_no_metadata() {
    let source_dir = PathBuf::from("/source");
    let dest_dir = PathBuf::from("/dest");

    let files = vec![RawFile {
        src_path: source_dir.join("readme.txt"),
        bytes: b"hello".to_vec(),
    }];

    let (jobs, records) = run_worker_on(files, &source_dir, &dest_dir);

    assert_eq!(jobs.len(), 1);
    assert!(records[0].thumbnail.is_none());
    assert!(records[0].exif.is_none());
}

#[test]
fn dest_path_preserves_relative_structure() {
    let source_dir = PathBuf::from("/source");
    let dest_dir = PathBuf::from("/dest");

    let files = vec![RawFile {
        src_path: source_dir.join("sub").join("dir").join("photo.arw"),
        bytes: b"whatever".to_vec(),
    }];

    let (jobs, records) = run_worker_on(files, &source_dir, &dest_dir);

    let expected = dest_dir.join("sub").join("dir").join("photo.arw");
    assert_eq!(jobs[0].dest_path, expected);
    assert_eq!(records[0].dest_path, expected);
}

#[test]
fn bytes_ownership_moves_into_write_job_unchanged() {
    let source_dir = PathBuf::from("/source");
    let dest_dir = PathBuf::from("/dest");
    let original = b"exact bytes preserved".to_vec();

    let files = vec![RawFile {
        src_path: source_dir.join("photo.arw"),
        bytes: original.clone(),
    }];

    let (jobs, _records) = run_worker_on(files, &source_dir, &dest_dir);

    assert_eq!(jobs[0].bytes, original);
}

#[test]
fn multiple_files_all_get_processed() {
    let source_dir = PathBuf::from("/source");
    let dest_dir = PathBuf::from("/dest");

    let files = vec![
        RawFile { src_path: source_dir.join("a.arw"), bytes: valid_tiff_with_thumbnail() },
        RawFile { src_path: source_dir.join("b.arw"), bytes: b"garbage".to_vec() },
        RawFile { src_path: source_dir.join("c.txt"), bytes: b"plain text".to_vec() },
    ];

    let (jobs, records) = run_worker_on(files, &source_dir, &dest_dir);

    assert_eq!(jobs.len(), 3);
    assert_eq!(records.len(), 3);
}
