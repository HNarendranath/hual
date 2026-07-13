use crate::pipeline::MetadataRecord;
use crossbeam_channel::Receiver;
use rusqlite::Connection;
use std::io;
use std::path::Path;

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

pub fn open_db(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS photos (
            id INTEGER PRIMARY KEY,
            src_path TEXT NOT NULL UNIQUE,
            dest_path TEXT NOT NULL,
            exposure_time REAL,
            f_stop REAL,
            iso INTEGER    
        );
        CREATE INDEX IF NOT EXISTS idx_photos_exif ON photos (iso, f_stop, exposure_time);",
    )?;
    Ok(conn)
}

#[cfg(test)]
#[path = "../../tests/unit/db_writer_tests.rs"]
mod db_writer_tests;
