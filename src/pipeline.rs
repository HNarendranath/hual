mod db_writer;
mod scanner;
mod ssd_writer;
mod worker;

// use crate::pipeline::{MetadataRecord, RawFile, WriteJob};
use crate::hidden_dir;
use crate::thumbnail::ExifData;
use crossbeam_channel::bounded;
use std::path::Path;
use std::path::PathBuf;
use std::thread;

// disk read -> worker
pub struct RawFile {
    pub src_path: PathBuf,
    pub bytes: Vec<u8>,
}

// worker -> ssd write
pub struct WriteJob {
    pub dest_path: PathBuf,
    pub bytes: Vec<u8>,
}

// worker -> db write
pub struct MetadataRecord {
    pub src_path: PathBuf,
    pub dest_path: PathBuf,
    pub exif: Option<ExifData>,
}

const CHANNEL_CAPACITY: usize = 32;

pub fn run_import(source_dir: &Path, dest_dir: &Path) {
    // ensure dest_dir before making hidden folder
    if let Err(e) = std::fs::create_dir_all(dest_dir) {
        eprintln!(
            "Error creating destination directory {}: {e}",
            dest_dir.display()
        );
        return;
    }

    // .hual folder
    let hidden = dest_dir.join(".hual");
    if let Err(e) = hidden_dir::ensure(&hidden) {
        eprintln!("Error creating {}: {e}", hidden.display());
        return;
    }

    let db_path = hidden.join("hual.db");
    let conn = match db_writer::open_db(&db_path) {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Error opening database {}: {e}", db_path.display());
            return;
        }
    };

    let l2_cache = match crate::cache::L2Cache::new(hidden.join("thumbcache")) {
        Ok(cache) => cache,
        Err(e) => {
            eprintln!("Error creating thumbnail cache directory: {e}");
            return;
        }
    };

    let (raw_tx, raw_rx) = bounded::<RawFile>(CHANNEL_CAPACITY);
    let (write_tx, write_rx) = bounded::<WriteJob>(CHANNEL_CAPACITY);
    let (db_tx, db_rx) = bounded::<MetadataRecord>(CHANNEL_CAPACITY);

    let worker_count = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    thread::scope(|s| {
        s.spawn(|| scanner::run(source_dir, raw_tx));

        let l2_cache_ref = &l2_cache;

        for _ in 0..worker_count {
            let raw_rx = raw_rx.clone();
            let write_tx = write_tx.clone();
            let db_tx = db_tx.clone();
            s.spawn(move || {
                worker::run(raw_rx, write_tx, db_tx, source_dir, dest_dir, l2_cache_ref)
            });
        }

        drop(write_tx);
        drop(db_tx);

        s.spawn(|| ssd_writer::run(write_rx));
        s.spawn(|| db_writer::run(db_rx, conn));
    });
}

#[cfg(test)]
#[path = "../tests/unit/pipeline_tests.rs"]
mod pipeline_tests;
