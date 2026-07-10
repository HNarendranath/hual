use byteorder::{BigEndian, ByteOrder};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub enum Cr3Error {
    Io(std::io::Error),
    NotAnIsobmff {
        found: [u8; 4],
    },
    BadFtype {
        major_brand: [u8; 4],
    },
    Truncated {
        offset: usize,
        needed: usize,
        len: usize,
    },
    BoxTooShort {
        offset: usize,
        size: usize,
        header_len: usize,
    },
    BoxSizeOverflow {
        offset: usize,
    },
    MissingBox {
        fourcc: [u8; 4],
        context: &'static str,
    },
    ThumbnailNotFound,
}

impl From<std::io::Error> for Cr3Error {
    fn from(e: std::io::Error) -> Self {
        Cr3Error::Io(e)
    }
}

impl std::fmt::Display for Cr3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cr3Error::Io(e) => write!(f, "I/O error: {e}"),
            Cr3Error::NotAnIsobmff { found } => {
                write!(f, "unrecognized file format: {found:?} (expected \"ftyp\")")
            }
            Cr3Error::BadFtype { major_brand } => {
                write!(f, "invalid file type: {major_brand:?} (expected \"crx \")")
            }
            Cr3Error::Truncated {
                offset,
                needed,
                len,
            } => write!(
                f,
                "file truncated: needed {needed} bytes at offset {offset}, but file is only {len} bytes long"
            ),
            Cr3Error::BoxTooShort {
                offset,
                size,
                header_len,
            } => write!(
                f,
                "box too short: needed {size} bytes at offset {offset}, but file is only {header_len} bytes long"
            ),
            Cr3Error::BoxSizeOverflow { offset } => write!(f, "box size overflow"),
            Cr3Error::MissingBox { fourcc, context } => write!(
                f,
                "missing box {fourcc:?} in {context} (file may be truncated or malformed)"
            ),
            Cr3Error::ThumbnailNotFound => write!(f, "no embedded thumbnail found in this file"),
        }
    }
}

impl std::error::Error for Cr3Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Cr3Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

fn read_u32_be(data: &[u8], offset: usize) -> Result<u32, Cr3Error> {
    let bytes = data.get(offset..offset + 4).ok_or(Cr3Error::Truncated {
        offset,
        needed: 4,
        len: data.len(),
    })?;
    Ok(BigEndian::read_u32(bytes))
}

fn read_u64_be(data: &[u8], offset: usize) -> Result<u64, Cr3Error> {
    let bytes = data.get(offset..offset + 8).ok_or(Cr3Error::Truncated {
        offset,
        needed: 8,
        len: data.len(),
    })?;
    Ok(BigEndian::read_u64(bytes))
}

#[derive(Debug, Clone, Copy)]
struct BoxHeader {
    box_type: [u8; 4],
    header_len: usize,
    payload_offset: usize,
    payload_len: usize,
    usertype: Option<[u8; 16]>,
}

fn read_box_header(data: &[u8], offset: usize, end: usize) -> Result<BoxHeader, Cr3Error> {
    let size = read_u32_be(data, offset)?;
    let mut current_offset = offset + 4;
    let type_slice = data
        .get(current_offset..current_offset + 4)
        .ok_or(Cr3Error::Truncated {
            offset: current_offset,
            needed: 4,
            len: data.len(),
        })?;

    let mut box_type = [0u8; 4];
    box_type.copy_from_slice(type_slice);
    current_offset += 4;

    // if size == 1 -> 64-bit size
    let total_size = if size == 1 {
        let size_u64 = read_u64_be(data, current_offset)?;
        current_offset += 8;
        usize::try_from(size_u64).map_err(|_| Cr3Error::BoxSizeOverflow { offset })?
    } else if size == 0 {
        end - offset
    } else {
        size as usize
    };

    let mut usertype = None;
    if box_type == *b"uuid" {
        let uuid_slice =
            data.get(current_offset..current_offset + 16)
                .ok_or(Cr3Error::Truncated {
                    offset: current_offset,
                    needed: 16,
                    len: data.len(),
                })?;
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes.copy_from_slice(uuid_slice);
        usertype = Some(uuid_bytes);
        current_offset += 16;
    }

    let header_len = current_offset - offset;

    if total_size < header_len {
        return Err(Cr3Error::BoxTooShort {
            offset,
            size: total_size,
            header_len,
        });
    }

    let payload_len = total_size - header_len;
    let payload_offset = current_offset;

    let box_end = payload_offset
        .checked_add(payload_len)
        .ok_or(Cr3Error::BoxSizeOverflow { offset })?;
    if box_end > end {
        return Err(Cr3Error::Truncated {
            offset,
            needed: total_size,
            len: end - offset,
        });
    }

    Ok(BoxHeader {
        box_type,
        header_len: header_len,
        payload_offset: payload_offset,
        payload_len: payload_len,
        usertype,
    })
}

fn read_boxes(data: &[u8], start: usize, end: usize) -> Result<Vec<BoxHeader>, Cr3Error> {
    let mut boxes = Vec::new();
    let mut current_offset = start;

    while current_offset < end {
        let box_header = read_box_header(data, current_offset, end)?;
        boxes.push(box_header);
        current_offset = box_header
            .payload_offset
            .checked_add(box_header.payload_len)
            .ok_or(Cr3Error::BoxSizeOverflow {
                offset: box_header.payload_offset,
            })?;
    }
    Ok(boxes)
}

fn find_jpeg_soi(payload: &[u8]) -> Option<usize> {
    payload.windows(2).position(|window| window == [0xFF, 0xD8])
}

// usertype of the TOP-LEVEL "preview data" uuid box — a sibling of moov/mdat.
const CANON_PREVIEW_UUID: [u8; 16] = [
    0xea, 0xf4, 0x2b, 0x5e, 0x1c, 0x98, 0x4b, 0x88, 0xb9, 0xfb, 0xb7, 0xdc, 0x40, 0x6e, 0x4d, 0x16,
];
const PRVW_UUID_PAYLOAD_SKIP: usize = 8; // undocumented gap before PRVW's siblings start
const PRVW_HEADER_LEN: usize = 16; // unknown:u32, unknown:u16, width:u16, height:u16, unknown:u16, jpeg_size:u32
const PRVW_JPEG_SIZE_FIELD_OFFSET: usize = 12;

struct ThumbnailLocation {
    offset: usize,
    length: usize,
}

fn find_thumbnail(data: &[u8]) -> Result<ThumbnailLocation, Cr3Error> {
    let file_len = data.len();
    let boxes = read_boxes(data, 0, file_len)?;

    let ftyp = boxes
        .iter()
        .find(|b| b.box_type == *b"ftyp")
        .ok_or(Cr3Error::NotAnIsobmff { found: [0; 4] })?;

    let mut major_brand = [0u8; 4];
    major_brand.copy_from_slice(
        data.get(ftyp.payload_offset..ftyp.payload_offset + 4)
            .ok_or(Cr3Error::Truncated {
                offset: ftyp.payload_offset,
                needed: 4,
                len: data.len(),
            })?,
    );
    if major_brand != *b"crx " {
        return Err(Cr3Error::BadFtype { major_brand });
    }

    let preview_uuid = boxes
        .iter()
        .find(|b| b.box_type == *b"uuid" && b.usertype == Some(CANON_PREVIEW_UUID))
        .ok_or(Cr3Error::MissingBox {
            fourcc: *b"uuid",
            context: "top-level preview container",
        })?;

    let children_start = preview_uuid
        .payload_offset
        .checked_add(PRVW_UUID_PAYLOAD_SKIP)
        .ok_or(Cr3Error::BoxSizeOverflow {
            offset: preview_uuid.payload_offset,
        })?;
    let children_end = preview_uuid
        .payload_offset
        .checked_add(preview_uuid.payload_len)
        .ok_or(Cr3Error::BoxSizeOverflow {
            offset: preview_uuid.payload_offset,
        })?;
    let children = read_boxes(data, children_start, children_end)?;

    let prvw = children
        .iter()
        .find(|b| b.box_type == *b"PRVW")
        .ok_or(Cr3Error::MissingBox {
            fourcc: *b"PRVW",
            context: "inside top-level preview uuid",
        })?;

    let jpeg_size_offset = prvw
        .payload_offset
        .checked_add(PRVW_JPEG_SIZE_FIELD_OFFSET)
        .ok_or(Cr3Error::BoxSizeOverflow {
            offset: prvw.payload_offset,
        })?;
    let jpeg_size = usize::try_from(read_u32_be(data, jpeg_size_offset)?).map_err(|_| {
        Cr3Error::BoxSizeOverflow {
            offset: jpeg_size_offset,
        }
    })?;

    let jpeg_offset =
        prvw.payload_offset
            .checked_add(PRVW_HEADER_LEN)
            .ok_or(Cr3Error::BoxSizeOverflow {
                offset: prvw.payload_offset,
            })?;
    let jpeg_end = jpeg_offset
        .checked_add(jpeg_size)
        .ok_or(Cr3Error::BoxSizeOverflow {
            offset: jpeg_offset,
        })?;
    let prvw_payload_end =
        prvw.payload_offset
            .checked_add(prvw.payload_len)
            .ok_or(Cr3Error::BoxSizeOverflow {
                offset: prvw.payload_offset,
            })?;
    // check for corruption
    if jpeg_end > prvw_payload_end {
        return Err(Cr3Error::BoxTooShort {
            offset: prvw.payload_offset,
            size: jpeg_size,
            header_len: PRVW_HEADER_LEN,
        });
    }

    Ok(ThumbnailLocation {
        offset: jpeg_offset,
        length: jpeg_size,
    })
}

pub fn extract_thumbnail(path: &Path) -> Result<Vec<u8>, Cr3Error> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let data: &[u8] = &mmap;

    let location = find_thumbnail(data)?;

    let start = location.offset;
    let end = start
        .checked_add(location.length)
        .filter(|&e| e <= data.len())
        .ok_or(Cr3Error::Truncated {
            offset: start,
            needed: location.length,
            len: data.len(),
        })?;
    let thumbnail_bytes = data[start..end].to_vec();
    Ok(thumbnail_bytes)
}

#[cfg(test)]
#[path = "../tests/unit/cr3_tests.rs"]
mod cr3_tests;
