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
    todo!()
}
fn subifd_offsets(data: &[u8], endian: Endian, ifd: &Ifd) -> Result<Vec<u32>, TiffError> {
    // tag 0x014A
    todo!()
}

const MAX_IFD_CHAIN: u32 = 16; // guard against corrupt/circular next-offset chains

fn find_thumbnail(
    data: &[u8],
    endian: Endian,
    ifd0_offset: u32,
) -> Result<ThumbnailLocation, TiffError> {
    // 1. walk IFD0's sibling chain (IFD1 = spec's thumbnail IFD) — PRIMARY path
    // 2. fall back to IFD0's own 0x0201/0x0202 tags
    // 3. fall back to SubIFDs referenced from IFD0 (tag 0x014A)
    todo!()
}

pub fn extract_thumbnail(path: &Path) -> Result<Vec<u8>, TiffError> {
    // File::open, unsafe { Mmap::map }, parse_header, find_thumbnail,
    // bounds-checked slice, .to_vec()
    todo!()
}
