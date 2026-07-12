use crate::pipeline::{MetadataRecord, RawFile, WriteJob};
use crate::thumbnail;
use crossbeam_channel::{Receiver, Sender};
use std::path::Path;

pub fn run(
    rx: Receiver<RawFile>,
    write_tx: Sender<WriteJob>,
    db_tx: Sender<MetadataRecord>,
    source_dir: &Path,
    dest_path: &Path,
) {
    for file in rx {
        let ext = file
            .src_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
    

        let thumbnail = match thumbnail::extract_thumbnail_from_bytes(&file.bytes, ext) {
            Ok(bytes) => Some(bytes),
            Err(e) => {
                eprintln!(
                    "thumbnail extraction failed for {}: {e}",
                    file.src_path.display()
                );
                None
            }
        };

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

        let relative = file
            .src_path
            .strip_prefix(source_dir)
            .unwrap_or(&file.src_path);
        let dest = dest_dir.join(relative);

        let write_job = WriteJob {
            dest_path: dest_path.clone(),
            bytes: file.bytes,
        };
        let record = MetadataRecord {
            file.src_path,
            dest_path,
            exif,
            thumbnail,
        };

        if write_tx.send(write_job).is_err() {
            continue;
        }
        let _ = db_tx.send(record);
    };
}
