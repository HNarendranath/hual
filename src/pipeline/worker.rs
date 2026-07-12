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
    todo!()
}
