mod support;

use super::*;
use crate::thumbnail::ExifData;
use crossbeam_channel::unbounded;
use std::path::PathBuf;
use support::TempDir;

// `run` consumes its `Connection` by value and doesn't hand it back, so a
// `:memory:` connection can't be inspected after `run` returns -- each test
// uses a real file-backed db in a TempDir instead, and opens a fresh
// `Connection` afterward to verify what got persisted.

#[test]
fn run_persists_records_with_metadata() {
    let dir = TempDir::new("db_writer_metadata");
    let db_path = dir.path.join("test.db");
    let conn = open_db(&db_path).unwrap();

    let (tx, rx) = unbounded();
    tx.send(MetadataRecord {
        src_path: PathBuf::from("/source/photo0.arw"),
        dest_path: PathBuf::from("/dest/photo0.arw"),
        exif: Some(ExifData {
            exposure_time: Some((1, 800)),
            f_stop: Some((28, 10)),
            iso: Some(100),
        }),
        thumbnail: Some(vec![0xFF, 0xD8, 0xFF, 0xD9]),
    })
    .unwrap();
    drop(tx);

    run(rx, conn);

    let verify = Connection::open(&db_path).unwrap();
    let (dest, exposure, f_stop, iso): (String, f64, f64, i64) = verify
        .query_row(
            "SELECT dest_path, exposure_time, f_stop, iso FROM photos WHERE src_path = ?1",
            params!["/source/photo0.arw"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();

    assert_eq!(dest, "/dest/photo0.arw");
    assert!((exposure - 1.0 / 800.0).abs() < 1e-9);
    assert!((f_stop - 2.8).abs() < 1e-9);
    assert_eq!(iso, 100);
}

#[test]
fn run_persists_records_with_no_metadata_as_null() {
    let dir = TempDir::new("db_writer_no_metadata");
    let db_path = dir.path.join("test.db");
    let conn = open_db(&db_path).unwrap();

    let (tx, rx) = unbounded();
    tx.send(MetadataRecord {
        src_path: PathBuf::from("/source/unsupported.txt"),
        dest_path: PathBuf::from("/dest/unsupported.txt"),
        exif: None,
        thumbnail: None,
    })
    .unwrap();
    drop(tx);

    run(rx, conn);

    let verify = Connection::open(&db_path).unwrap();
    let (exposure, f_stop, iso): (Option<f64>, Option<f64>, Option<i64>) = verify
        .query_row(
            "SELECT exposure_time, f_stop, iso FROM photos WHERE src_path = ?1",
            params!["/source/unsupported.txt"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();

    assert_eq!(exposure, None);
    assert_eq!(f_stop, None);
    assert_eq!(iso, None);
}

#[test]
fn run_replaces_existing_row_on_reimport_of_same_src_path() {
    let dir = TempDir::new("db_writer_reimport");
    let db_path = dir.path.join("test.db");

    let conn1 = open_db(&db_path).unwrap();
    let (tx1, rx1) = unbounded();
    tx1.send(MetadataRecord {
        src_path: PathBuf::from("/source/photo0.arw"),
        dest_path: PathBuf::from("/dest/photo0.arw"),
        exif: Some(ExifData {
            exposure_time: Some((1, 800)),
            f_stop: Some((28, 10)),
            iso: Some(100),
        }),
        thumbnail: None,
    })
    .unwrap();
    drop(tx1);
    run(rx1, conn1);

    let conn2 = open_db(&db_path).unwrap();
    let (tx2, rx2) = unbounded();
    tx2.send(MetadataRecord {
        src_path: PathBuf::from("/source/photo0.arw"),
        dest_path: PathBuf::from("/dest/photo0_renamed.arw"),
        exif: Some(ExifData {
            exposure_time: Some((1, 200)),
            f_stop: Some((40, 10)),
            iso: Some(400),
        }),
        thumbnail: None,
    })
    .unwrap();
    drop(tx2);
    run(rx2, conn2);

    let verify = Connection::open(&db_path).unwrap();
    let count: i64 = verify
        .query_row("SELECT COUNT(*) FROM photos", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);

    let dest: String = verify
        .query_row(
            "SELECT dest_path FROM photos WHERE src_path = ?1",
            params!["/source/photo0.arw"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(dest, "/dest/photo0_renamed.arw");
}
