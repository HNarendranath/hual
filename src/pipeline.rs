mod db_writer;
mod scanner;
mod ssd_writer;
mod worker;

pub use db_writer::{PhotoFilters, PhotoRow, RangeFilter, list_photos, open_db};

// use crate::pipeline::{MetadataRecord, RawFile, WriteJob};
use crate::hidden_dir;
use crate::thumbnail::ExifData;
use crossbeam_channel::bounded;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
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

pub enum ImportMode {
    CopyAndImport(PathBuf),
    ImportOnly,
}

const CHANNEL_CAPACITY: usize = 32;

pub fn run_import(
    source_dir: &Path,
    mode: ImportMode,
    raw_only: bool,
    on_progress: impl Fn(usize) + Sync,
) {
    let hual_root: PathBuf = match &mode {
        ImportMode::CopyAndImport(dest_dir) => {
            // ensure dest_dir before making hidden folder
            if let Err(e) = std::fs::create_dir_all(dest_dir) {
                eprintln!(
                    "Error creating destination directory {}: {e}",
                    dest_dir.display()
                );
                return;
            }
            dest_dir.clone()
        }
        ImportMode::ImportOnly => {
            // For import-only mode, we'll use the source directory as the hual root
            source_dir.to_path_buf()
        }
    };

    // .hual folder
    let hidden = hual_root.join(".hual");
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

    let dest_dir_ref: Option<&Path> = match &mode {
        ImportMode::CopyAndImport(dest_dir) => Some(dest_dir.as_path()),
        ImportMode::ImportOnly => None,
    };

    let (raw_tx, raw_rx) = bounded::<RawFile>(CHANNEL_CAPACITY);
    let (write_tx, write_rx) = bounded::<WriteJob>(CHANNEL_CAPACITY);
    let (db_tx, db_rx) = bounded::<MetadataRecord>(CHANNEL_CAPACITY);

    let worker_count = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let counter = AtomicUsize::new(0);
    let on_progress: &(dyn Fn(usize) + Sync) = &on_progress;

    thread::scope(|s| {
        s.spawn(|| scanner::run(source_dir, raw_tx, raw_only));

        let l2_cache_ref = &l2_cache;
        let counter_ref = &counter;

        for _ in 0..worker_count {
            let raw_rx = raw_rx.clone();
            let write_tx = write_tx.clone();
            let db_tx = db_tx.clone();
            s.spawn(move || {
                worker::run(
                    raw_rx,
                    write_tx,
                    db_tx,
                    source_dir,
                    dest_dir_ref,
                    l2_cache_ref,
                    counter_ref,
                    on_progress,
                );
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
