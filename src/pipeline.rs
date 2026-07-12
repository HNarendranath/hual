mod db_writer;
mod scanner;
mod ssd_writer;
mod worker;

use crate::thumbnail::ExifData;
use std::path::PathBuf;

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
    pub thumbnail: Option<Vec<u8>>,
}
