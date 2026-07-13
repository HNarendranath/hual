use crate::pipeline::MetadataRecord;
use crossbeam_channel::Receiver;
use rusqlite::{Connection, params};
use std::path::Path;

fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
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
    )
}

pub fn open_db(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    init_schema(&conn)?;
    Ok(conn)
}

fn insert_record(conn: &Connection, record: &MetadataRecord) -> rusqlite::Result<()> {
    let exposure_time = record
        .exif
        .and_then(|e| e.exposure_time)
        .map(|(n, d)| n as f64 / d as f64);
    let f_stop = record
        .exif
        .and_then(|e| e.f_stop)
        .map(|(n, d)| n as f64 / d as f64);
    let iso = record.exif.and_then(|e| e.iso);

    conn.execute(
        "INSERT OR REPLACE INTO photos (src_path, dest_path, exposure_time, f_stop, iso)
        VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            record.src_path.to_string_lossy(),
            record.dest_path.to_string_lossy(),
            exposure_time,
            f_stop,
            iso,
        ],
    )?;
    Ok(())
}

pub fn run(rx: Receiver<MetadataRecord>, conn: Connection) {
    for record in rx {
        if let Err(e) = insert_record(&conn, &record) {
            eprintln!("Error writing {}: {e}", record.dest_path.display());
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/db_writer_tests.rs"]
mod db_writer_tests;
