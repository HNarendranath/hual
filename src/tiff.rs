use byteorder::{BigEndian, ByteOrder, LittleEndian};
use core::num;
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub enum TiffError {
    Io(std::io::Error),
    BadMagic {
        found: [u8; 2],
    },
    BadMagicNumber(u16),
    Truncated {
        offset: usize,
        needed: usize,
        len: usize,
    },
    ThumbnailNotFound,
}

// NB this made me WAY too happy lol
impl From<std::io::Error> for TiffError {
    fn from(e: std::io::Error) -> Self {
        TiffError::Io(e)
    }
}

impl std::fmt::Display for TiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TiffError::Io(e) => write!(f, "I/O error: {e}"),
            TiffError::BadMagic { found } => write!(
                f,
                "unrecognized TIFF byte order marker {found:?} (expected \"II\" or \"MM\")"
            ),
            TiffError::BadMagicNumber(n) => {
                write!(f, "invalid TIFF magic number {n} (expected 42)")
            }
            TiffError::Truncated {
                offset,
                needed,
                len,
            } => write!(
                f,
                "file truncated: needed {needed} bytes at offset {offset}, but file is only {len} bytes long"
            ),
            TiffError::ThumbnailNotFound => write!(f, "no embedded thumbnail found in this file"),
        }
    }
}

impl std::error::Error for TiffError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TiffError::Io(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Endian {
    Little,
    Big,
}
impl Endian {
    fn read_u16(self, data: &[u8], offset: usize) -> Result<u16, TiffError> {
        let bytes = data.get(offset..offset + 2).ok_or(TiffError::Truncated {
            offset,
            needed: 2,
            len: data.len(),
        })?;
        Ok(match self {
            Endian::Little => LittleEndian::read_u16(bytes),
            Endian::Big => BigEndian::read_u16(bytes),
        })
    }
    fn read_u32(self, data: &[u8], offset: usize) -> Result<u32, TiffError> {
        let bytes = data.get(offset..offset + 4).ok_or(TiffError::Truncated {
            offset,
            needed: 4,
            len: data.len(),
        })?;
        Ok(match self {
            Endian::Little => LittleEndian::read_u32(bytes),
            Endian::Big => BigEndian::read_u32(bytes),
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct IfdEntry {
    tag: u16,
    field_type: u16,
    count: u32,
    value_or_offset: u32,
}
impl IfdEntry {
    fn data_size(&self) -> u32 {
        type_size(self.field_type) * self.count
    }
}
fn type_size(field_type: u16) -> u32 {
    match field_type {
        1 | 2 | 6 | 7 => 1,   // BYTE, ASCII, SBYTE, UNDEFINED
        3 | 8 => 2,           // SHORT, SSHORT
        4 | 9 | 11 | 13 => 4, // LONG, SLONG, FLOAT, IFD
        5 | 10 | 12 => 8,     // RATIONAL, SRATIONAL, DOUBLE
        _ => 0,               // return 0 for unkown types, so we can skip them
    }
}

struct Ifd {
    entries: Vec<IfdEntry>,
    next_offset: u32,
} // IFD = Image File Directory
struct TiffHeader {
    endian: Endian,
    ifd0_offset: u32,
}
#[derive(Debug, Clone, Copy)]
struct ThumbnailLocation {
    offset: u32,
    length: u32,
}

fn parse_header(data: &[u8]) -> Result<TiffHeader, TiffError> {
    let magic = data.get(0..2).ok_or(TiffError::Truncated {
        offset: 0,
        needed: 2,
        len: data.len(),
    })?;
    let endian = match magic {
        b"II" => Endian::Little,
        b"MM" => Endian::Big,
        _ => {
            return Err(TiffError::BadMagic {
                found: [magic[0], magic[1]],
            });
        }
    };

    let magic_number = endian.read_u16(data, 2)?;
    if magic_number != 42 {
        return Err(TiffError::BadMagicNumber(magic_number));
    }

    let ifd0_offset = endian.read_u32(data, 4)?;

    Ok(TiffHeader {
        endian,
        ifd0_offset,
    })
}
fn read_ifd(data: &[u8], endian: Endian, offset: u32) -> Result<Ifd, TiffError> {
    let mut current_offset = offset as usize;
    let num_entries = endian.read_u16(data, current_offset)? as usize;
    current_offset += 2;

    let mut entries = Vec::new();
    for _ in 0..num_entries {
        let tag = endian.read_u16(data, current_offset)?;
        let field_type = endian.read_u16(data, current_offset + 2)?;
        let count = endian.read_u32(data, current_offset + 4)?;
        let value_or_offset = endian.read_u32(data, current_offset + 8)?;

        entries.push(IfdEntry {
            tag,
            field_type,
            count,
            value_or_offset,
        });

        current_offset += 12;
    }
    Ok(Ifd {
        entries,
        next_offset: endian.read_u32(data, current_offset)?,
    })
}
fn thumbnail_tags(ifd: &Ifd) -> Option<ThumbnailLocation> {
    // find 0x0201 + 0x0202
    let offset_entry = ifd.entries.iter().find(|e| e.tag == 0x0201)?;
    let length_entry = ifd.entries.iter().find(|e| e.tag == 0x0202)?;

    Some(ThumbnailLocation {
        offset: offset_entry.value_or_offset,
        length: length_entry.value_or_offset,
    })
}
fn subifd_offsets(data: &[u8], endian: Endian, ifd: &Ifd) -> Result<Vec<u32>, TiffError> {
    // tag 0x014A
    let Some(entry) = ifd.entries.iter().find(|e| e.tag == 0x014A) else {
        return Ok(Vec::new());
    };

    if entry.count == 1 {
        return Ok(vec![entry.value_or_offset]);
    }

    let mut offsets = Vec::with_capacity(entry.count as usize);
    for i in 0..entry.count {
        let offset = entry.value_or_offset as usize + (i as usize) * 4;
        offsets.push(endian.read_u32(data, offset)?);
    }
    Ok(offsets)
}

const MAX_IFD_CHAIN: u32 = 16; // guard against corrupt/circular next-offset chains

fn find_thumbnail(
    data: &[u8],
    endian: Endian,
    ifd0_offset: u32,
) -> Result<ThumbnailLocation, TiffError> {
    let ifd0 = read_ifd(data, endian, ifd0_offset)?;

    // 1. fall back to SubIFDs referenced from IFD0 (tag 0x014A)
    for sub_offset in subifd_offsets(data, endian, &ifd0)? {
        let sub_ifd = read_ifd(data, endian, sub_offset)?;
        if let Some(loc) = thumbnail_tags(&sub_ifd) {
            return Ok(loc);
        }
    }

    // 2. fall back to IFD0's own 0x0201/0x0202 tags
    if let Some(loc) = thumbnail_tags(&ifd0) {
        return Ok(loc);
    }

    // 3. walk IFD0's sibling chain (IFD1 = spec's thumbnail IFD) — PRIMARY path
    let mut next = ifd0.next_offset;
    let mut depth = 0;

    while next != 0 && depth < MAX_IFD_CHAIN {
        let ifd = read_ifd(data, endian, next)?;
        if let Some(loc) = thumbnail_tags(&ifd) {
            return Ok(loc);
        }
        next = ifd.next_offset;
        depth += 1;
    }

    Err(TiffError::ThumbnailNotFound)
}

pub fn extract_thumbnail(path: &Path) -> Result<Vec<u8>, TiffError> {
    // File::open, unsafe { Mmap::map }, parse_header, find_thumbnail,
    // bounds-checked slice, .to_vec()
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let data: &[u8] = &mmap;

    let header = parse_header(data)?;
    let location = find_thumbnail(data, header.endian, header.ifd0_offset)?;

    let start = location.offset as usize;
    let end = start
        .checked_add(location.length as usize)
        .filter(|&e| e <= data.len())
        .ok_or(TiffError::Truncated {
            offset: start,
            needed: location.length as usize,
            len: data.len(),
        })?;
    Ok(data[start..end].to_vec())
}

#[cfg(test)]
#[path = "../tests/unit/tiff_tests.rs"]
mod tiff_tests;
