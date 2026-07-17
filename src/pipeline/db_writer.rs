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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoRow {
    src_path: String,
    dest_path: String,
    exposure_time: Option<f64>,
    f_stop: Option<f64>,
    iso: Option<u16>,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct RangeFilter<T> {
    pub min: Option<T>,
    pub max: Option<T>,
}

impl<T> Default for RangeFilter<T> {
    fn default() -> Self {
        RangeFilter {
            min: None,
            max: None,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoFilters {
    #[serde(default)]
    pub iso: RangeFilter<u16>,
    #[serde(default)]
    pub f_stop: RangeFilter<f64>,
    #[serde(default)]
    pub exposure_time: RangeFilter<f64>,
}

pub fn list_photos(conn: &Connection, filters: &PhotoFilters) -> rusqlite::Result<Vec<PhotoRow>> {
    let mut clauses: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    let mut push = |col: &str, op: &str, val: Box<dyn rusqlite::ToSql>| {
        clauses.push(format!("{} {} ?{}", col, op, params.len() + 1));
        params.push(val);
    };

    if let Some(v) = filters.iso.min {
        push("iso", ">=", Box::new(v));
    }
    if let Some(v) = filters.iso.max {
        push("iso", "<=", Box::new(v));
    }
    if let Some(v) = filters.f_stop.min {
        push("f_stop", ">=", Box::new(v));
    }
    if let Some(v) = filters.f_stop.max {
        push("f_stop", "<=", Box::new(v));
    }
    if let Some(v) = filters.exposure_time.min {
        push("exposure_time", ">=", Box::new(v));
    }
    if let Some(v) = filters.exposure_time.max {
        push("exposure_time", "<=", Box::new(v));
    }

    let where_clause = if clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clauses.join(" AND "))
    };

    let sql = format!(
        "SELECT src_path, dest_path, exposure_time, f_stop, iso FROM photos {where_clause}"
    );

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(PhotoRow {
            src_path: row.get(0)?,
            dest_path: row.get(1)?,
            exposure_time: row.get(2)?,
            f_stop: row.get(3)?,
            iso: row.get(4)?,
        })
    })?;

    rows.collect()
}

#[cfg(test)]
#[path = "../../tests/unit/db_writer_tests.rs"]
mod db_writer_tests;
