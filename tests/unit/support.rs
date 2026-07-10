use std::path::PathBuf;

#[derive(Clone, Copy)]
pub struct Entry {
    pub tag: u16,
    pub field_type: u16,
    pub count: u32,
    pub value: u32,
}

pub fn entry(tag: u16, field_type: u16, count: u32, value: u32) -> Entry {
    Entry { tag, field_type, count, value }
}

pub const IFD0_OFFSET: u32 = 8;

pub fn ifd_len(entry_count: usize) -> u32 {
    (2 + entry_count * 12 + 4) as u32
}

fn write_u16(buf: &mut Vec<u8>, val: u16, little_endian: bool) {
    buf.extend_from_slice(&if little_endian { val.to_le_bytes() } else { val.to_be_bytes() });
}

fn write_u32(buf: &mut Vec<u8>, val: u32, little_endian: bool) {
    buf.extend_from_slice(&if little_endian { val.to_le_bytes() } else { val.to_be_bytes() });
}

pub fn write_header(buf: &mut Vec<u8>, little_endian: bool) {
    buf.extend_from_slice(if little_endian { b"II" } else { b"MM" });
    write_u16(buf, 42, little_endian);
    write_u32(buf, IFD0_OFFSET, little_endian);
}

pub fn write_ifd(buf: &mut Vec<u8>, entries: &[Entry], next_offset: u32, little_endian: bool) {
    write_u16(buf, entries.len() as u16, little_endian);
    for e in entries {
        write_u16(buf, e.tag, little_endian);
        write_u16(buf, e.field_type, little_endian);
        write_u32(buf, e.count, little_endian);
        write_u32(buf, e.value, little_endian);
    }
    write_u32(buf, next_offset, little_endian);
}

pub fn write_box(buf: &mut Vec<u8>, box_type: &[u8; 4], payload: &[u8]) {
    let size = 8 + payload.len();
    buf.extend_from_slice(&(size as u32).to_be_bytes());
    buf.extend_from_slice(box_type);
    buf.extend_from_slice(payload);
}

pub fn write_uuid_box(buf: &mut Vec<u8>, usertype: &[u8; 16], payload: &[u8]) {
    let size = 8 + 16 + payload.len();
    buf.extend_from_slice(&(size as u32).to_be_bytes());
    buf.extend_from_slice(b"uuid");
    buf.extend_from_slice(usertype);
    buf.extend_from_slice(payload);
}

pub fn write_ftyp(buf: &mut Vec<u8>, major_brand: &[u8; 4]) {
    let mut payload = Vec::new();
    payload.extend_from_slice(major_brand);
    payload.extend_from_slice(&0u32.to_be_bytes());
    write_box(buf, b"ftyp", &payload);
}

pub struct TempFile {
    pub path: PathBuf,
}

impl TempFile {
    pub fn new(name: &str, data: &[u8]) -> Self {
        let path = std::env::temp_dir().join(format!("hual_test_{name}_{}.tiff", std::process::id()));
        std::fs::write(&path, data).expect("write temp fixture file");
        Self { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
