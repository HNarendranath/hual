use std::fs;
use std::io;

use crate::pipeline::MetadataRecord;
use crossbeam_channel::Receiver;

fn write_record(job: &MetadataRecord) -> io::Result<()> {
    todo!()
}

pub fn run(rx: Receiver<MetadataRecord>) {
    // for record in rx {
    //     if let Err(e) = write_record(&record) {
    //         eprintln!("Error writing {}: {e}", record.dest_path.display());
    //     }
    // }

    for record in rx {
        println!(
            "{} -> {} | exif: {:?} | thumbnail {} bytes",
            record.src_path.display(),
            record.dest_path.display(),
            record.exif,
            record.thumbnail.as_ref().map_or(0, |t| t.len()),
        );
    }
}

#[cfg(test)]
#[path = "../../tests/unit/db_writer_tests.rs"]
mod db_writer_tests;
