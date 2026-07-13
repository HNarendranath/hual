mod support;

use super::*;
use crossbeam_channel::unbounded;
use std::path::PathBuf;
use support::TempDir;

#[test]
fn write_job_creates_parent_dirs_and_writes_bytes() {
    let dir = TempDir::new("ssd_writer_basic");
    let dest = dir.path.join("nested").join("photo.arw");

    let job = WriteJob {
        dest_path: dest.clone(),
        bytes: b"hello world".to_vec(),
    };
    write_job(&job).expect("write should succeed");

    assert_eq!(std::fs::read(&dest).unwrap(), b"hello world");
}

#[test]
fn write_job_fails_on_invalid_path() {
    let job = WriteJob {
        dest_path: PathBuf::from("\0invalid"),
        bytes: b"data".to_vec(),
    };
    assert!(write_job(&job).is_err());
}

#[test]
fn run_processes_all_jobs_and_tolerates_one_failure() {
    let dir = TempDir::new("ssd_writer_run");
    let good1 = dir.path.join("a.arw");
    let good2 = dir.path.join("b.arw");

    let (tx, rx) = unbounded();
    tx.send(WriteJob { dest_path: good1.clone(), bytes: b"1".to_vec() }).unwrap();
    tx.send(WriteJob { dest_path: PathBuf::from("\0invalid"), bytes: b"bad".to_vec() }).unwrap();
    tx.send(WriteJob { dest_path: good2.clone(), bytes: b"2".to_vec() }).unwrap();
    drop(tx);

    run(rx);

    assert_eq!(std::fs::read(&good1).unwrap(), b"1");
    assert_eq!(std::fs::read(&good2).unwrap(), b"2");
}
