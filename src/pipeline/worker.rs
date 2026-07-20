use crate::cache::L2Cache;
use crate::pipeline::{MetadataRecord, RawFile, WriteJob};
use crate::thumbnail;
use crossbeam_channel::{Receiver, Sender};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn run(
    rx: Receiver<RawFile>,
    write_tx: Sender<WriteJob>,
    db_tx: Sender<MetadataRecord>,
    source_dir: &Path,
    dest_dir: Option<&Path>,
    l2_cache: &L2Cache,
    counter: &AtomicUsize,
    on_progress: &(dyn Fn(usize) + Sync),
) {
    for file in rx {
        let ext = file
            .src_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let exif = match thumbnail::extract_exif_from_bytes(&file.bytes, ext) {
            Ok(exif) => Some(exif),
            Err(e) => {
                eprintln!(
                    "exif extraction failed for {}: {e}",
                    file.src_path.display()
                );
                None
            }
        };

        let thumb = match thumbnail::extract_thumbnail_from_bytes(&file.bytes, ext) {
            Ok(exif) => Some(exif),
            Err(e) => {
                eprint!(
                    "thumbnail extraction failed for {}: {e}",
                    file.src_path.display()
                );
                None
            }
        };

        if let Some(bytes) = &thumb {
            let key = file.src_path.to_string_lossy();
            if let Err(e) = l2_cache.put(&key, bytes) {
                eprint!(
                    "thumbnail cache write failed for {}: {e}",
                    file.src_path.display()
                );
            }
        }

        let dest = match dest_dir {
            Some(dir) => {
                let relative = file
                    .src_path
                    .strip_prefix(source_dir)
                    .unwrap_or(&file.src_path);
                dir.join(relative)
            }
            None => file.src_path.clone(),
        };

        if dest_dir.is_some() {
            let write_job = WriteJob {
                dest_path: dest.clone(),
                bytes: file.bytes,
            };
            if write_tx.send(write_job).is_err() {
                continue;
            }
        }

        let record = MetadataRecord {
            src_path: file.src_path,
            dest_path: dest,
            exif,
        };
        let _ = db_tx.send(record);

        let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
        on_progress(count);
    }
}

#[cfg(test)]
#[path = "../../tests/unit/worker_tests.rs"]
mod worker_tests;
