use crate::pipeline::RawFile;
use crossbeam_channel::Sender;
use std::fs;
use std::path::Path;

fn scan_dir(dir: &Path, tx: &Sender<RawFile>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Error reading directory {}: {e}", dir.display());
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Error reading directory entry in {}: {e}", dir.display());
                continue;
            }
        };

        let ftype = match entry.file_type() {
            Ok(ftype) => ftype,
            Err(e) => {
                eprintln!(
                    "Error reading file type for {}: {e}",
                    entry.path().display()
                );
                continue;
            }
        };

        if ftype.is_symlink() {
            continue;
        }

        // recurse into dirs
        let path = entry.path();
        if ftype.is_dir() {
            scan_dir(&path, tx);
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Error reading file {}: {e}", path.display());
                continue;
            }
        };

        if tx
            .send(RawFile {
                src_path: path,
                bytes,
            })
            .is_err()
        {
            return;
        }
    }
}

pub fn run(source_dir: &Path, tx: Sender<RawFile>) {
    scan_dir(source_dir, &tx);
}
