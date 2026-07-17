mod support;

use super::*;
use crate::thumbnail::ExifData;
use crossbeam_channel::unbounded;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use support::{IFD0_OFFSET, TempDir, entry, ifd_len, tiny_jpeg, write_header, write_ifd};

fn tiff_with_embedded_thumbnail(thumb_bytes: Vec<u8>) -> Vec<u8> {
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

fn valid_tiff_with_thumbnail() -> Vec<u8> {
    tiff_with_embedded_thumbnail(tiny_jpeg(4, 4, [200, 50, 50]))
}

fn run_worker_on(
    files: Vec<RawFile>,
    source_dir: &Path,
    dest_dir: &Path,
    l2_cache: &L2Cache,
) -> (Vec<WriteJob>, Vec<MetadataRecord>) {
    let (raw_tx, raw_rx) = unbounded();
    let (write_tx, write_rx) = unbounded();
    let (db_tx, db_rx) = unbounded();

    for f in files {
        raw_tx.send(f).unwrap();
    }
    drop(raw_tx);

    let counter = AtomicUsize::new(0);

    run(
        raw_rx,
        write_tx,
        db_tx,
        source_dir,
        dest_dir,
        l2_cache,
        &counter,
        &|_| {},
    );

    (write_rx.iter().collect(), db_rx.iter().collect())
}

#[test]
fn successful_extraction_caches_thumbnail_and_populates_exif() {
    let source_dir = PathBuf::from("/source");
    let dest_dir = PathBuf::from("/dest");
    let cache_dir = TempDir::new("worker_success");
    let l2_cache = L2Cache::new(cache_dir.path.clone()).unwrap();

    let files = vec![RawFile {
        src_path: source_dir.join("photo.arw"),
        bytes: valid_tiff_with_thumbnail(),
    }];

    let (jobs, records) = run_worker_on(files, &source_dir, &dest_dir, &l2_cache);

    assert_eq!(jobs.len(), 1);
    assert_eq!(records.len(), 1);
    // no EXIF_IFD_POINTER in this synthetic IFD0, so extraction succeeds
    // (Ok, not Err) but every field is None -- exif is Some(_), not None.
    assert_eq!(
        records[0].exif,
        Some(ExifData {
            exposure_time: None,
            f_stop: None,
            iso: None
        })
    );
    let key = source_dir.join("photo.arw").to_string_lossy().into_owned();
    assert!(l2_cache.get(&key).is_some());
}

#[test]
fn failed_extraction_still_produces_job_and_record_with_none_fields() {
    let source_dir = PathBuf::from("/source");
    let dest_dir = PathBuf::from("/dest");
    let cache_dir = TempDir::new("worker_failed_extraction");
    let l2_cache = L2Cache::new(cache_dir.path.clone()).unwrap();

    let files = vec![RawFile {
        src_path: source_dir.join("garbage.arw"),
        bytes: b"not a tiff file at all".to_vec(),
    }];

    let (jobs, records) = run_worker_on(files, &source_dir, &dest_dir, &l2_cache);

    assert_eq!(jobs.len(), 1);
    assert_eq!(records.len(), 1);
    assert!(records[0].exif.is_none());
    // bytes still get forwarded for copying even though extraction failed
    assert_eq!(jobs[0].bytes, b"not a tiff file at all");
    let key = source_dir
        .join("garbage.arw")
        .to_string_lossy()
        .into_owned();
    assert!(l2_cache.get(&key).is_none());
}

#[test]
fn unsupported_extension_still_copies_with_no_metadata() {
    let source_dir = PathBuf::from("/source");
    let dest_dir = PathBuf::from("/dest");
    let cache_dir = TempDir::new("worker_unsupported_extension");
    let l2_cache = L2Cache::new(cache_dir.path.clone()).unwrap();

    let files = vec![RawFile {
        src_path: source_dir.join("readme.txt"),
        bytes: b"hello".to_vec(),
    }];

    let (jobs, records) = run_worker_on(files, &source_dir, &dest_dir, &l2_cache);

    assert_eq!(jobs.len(), 1);
    assert!(records[0].exif.is_none());
    let key = source_dir.join("readme.txt").to_string_lossy().into_owned();
    assert!(l2_cache.get(&key).is_none());
}

#[test]
fn thumbnail_extraction_succeeds_but_undecodable_bytes_dont_abort_record() {
    let source_dir = PathBuf::from("/source");
    let dest_dir = PathBuf::from("/dest");
    let cache_dir = TempDir::new("worker_cache_put_failure");
    let l2_cache = L2Cache::new(cache_dir.path.clone()).unwrap();

    // Bytes sit at the right TIFF offset/length, so extraction itself
    // succeeds, but they aren't a real JPEG -- L2Cache::put's decode step
    // will fail. The job/record should still be produced normally; only the
    // cache write is skipped.
    let data = tiff_with_embedded_thumbnail(b"\xFF\xD8fake-thumbnail-data\xFF\xD9".to_vec());

    let files = vec![RawFile {
        src_path: source_dir.join("photo.arw"),
        bytes: data,
    }];

    let (jobs, records) = run_worker_on(files, &source_dir, &dest_dir, &l2_cache);

    assert_eq!(jobs.len(), 1);
    assert_eq!(records.len(), 1);
    let key = source_dir.join("photo.arw").to_string_lossy().into_owned();
    assert!(
        l2_cache.get(&key).is_none(),
        "undecodable bytes should not produce a cache entry"
    );
}

#[test]
fn dest_path_preserves_relative_structure() {
    let source_dir = PathBuf::from("/source");
    let dest_dir = PathBuf::from("/dest");
    let cache_dir = TempDir::new("worker_relative_structure");
    let l2_cache = L2Cache::new(cache_dir.path.clone()).unwrap();

    let files = vec![RawFile {
        src_path: source_dir.join("sub").join("dir").join("photo.arw"),
        bytes: b"whatever".to_vec(),
    }];

    let (jobs, records) = run_worker_on(files, &source_dir, &dest_dir, &l2_cache);

    let expected = dest_dir.join("sub").join("dir").join("photo.arw");
    assert_eq!(jobs[0].dest_path, expected);
    assert_eq!(records[0].dest_path, expected);
}

#[test]
fn bytes_ownership_moves_into_write_job_unchanged() {
    let source_dir = PathBuf::from("/source");
    let dest_dir = PathBuf::from("/dest");
    let cache_dir = TempDir::new("worker_bytes_ownership");
    let l2_cache = L2Cache::new(cache_dir.path.clone()).unwrap();
    let original = b"exact bytes preserved".to_vec();

    let files = vec![RawFile {
        src_path: source_dir.join("photo.arw"),
        bytes: original.clone(),
    }];

    let (jobs, _records) = run_worker_on(files, &source_dir, &dest_dir, &l2_cache);

    assert_eq!(jobs[0].bytes, original);
}

#[test]
fn multiple_files_all_get_processed() {
    let source_dir = PathBuf::from("/source");
    let dest_dir = PathBuf::from("/dest");
    let cache_dir = TempDir::new("worker_multiple_files");
    let l2_cache = L2Cache::new(cache_dir.path.clone()).unwrap();

    let files = vec![
        RawFile {
            src_path: source_dir.join("a.arw"),
            bytes: valid_tiff_with_thumbnail(),
        },
        RawFile {
            src_path: source_dir.join("b.arw"),
            bytes: b"garbage".to_vec(),
        },
        RawFile {
            src_path: source_dir.join("c.txt"),
            bytes: b"plain text".to_vec(),
        },
    ];

    let (jobs, records) = run_worker_on(files, &source_dir, &dest_dir, &l2_cache);

    assert_eq!(jobs.len(), 3);
    assert_eq!(records.len(), 3);
}
