mod support;

use super::*;
use crate::thumbnail::ExifData;
use crossbeam_channel::unbounded;
use std::path::PathBuf;

// db_writer.rs currently only stubs persistence (write_record is unused/`todo!()`,
// the active `run` just prints) -- this only covers the channel-draining shape,
// and will need real assertions once rusqlite inserts land behind `run`.
#[test]
fn run_drains_all_records_without_panicking() {
    let (tx, rx) = unbounded();

    for i in 0..3 {
        tx.send(MetadataRecord {
            src_path: PathBuf::from(format!("/source/photo{i}.arw")),
            dest_path: PathBuf::from(format!("/dest/photo{i}.arw")),
            exif: Some(ExifData {
                exposure_time: Some((1, 800)),
                f_stop: Some((28, 10)),
                iso: Some(100),
            }),
            thumbnail: Some(vec![0xFF, 0xD8, 0xFF, 0xD9]),
        })
        .unwrap();
    }
    drop(tx);

    run(rx);
}

#[test]
fn run_handles_records_with_no_metadata() {
    let (tx, rx) = unbounded();

    tx.send(MetadataRecord {
        src_path: PathBuf::from("/source/unsupported.txt"),
        dest_path: PathBuf::from("/dest/unsupported.txt"),
        exif: None,
        thumbnail: None,
    })
    .unwrap();
    drop(tx);

    run(rx);
}
