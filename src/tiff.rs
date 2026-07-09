use byteorder::{ByteOrder, LittleEndian, BigEndian};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub enum TiffError {
    Io(std::io::Error),
    BadMagic { found: [u8; 2] },
    BadMagicNumber(u16),
    Truncated { offset: usize, needed: usize, len: usize },
    ThumbnailNotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Endian { Little, Big }
impl Endian {
    fn read_u16(self, data: &[u8], offset: usize) -> Result<u16, TiffError> {
        todo!()
    }
    fn read_u32(self, data: &[u8], offset: usize) -> Result<u32, TiffError> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
struct IfdEntry { tag: u16, field_type: u16, count: u32, value_or_offset: u32 }
impl IfdEntry { 
    fn data_size(&self) -> u32 {
        type_size(self.field_type) * self.count
    }
}
fn type_size(field_type: u16) -> u32 {
    todo!()
}               

struct Ifd { entries: Vec<IfdEntry>, next_offset: u32 } // IFD = Image File Directory
struct TiffHeader { endian: Endian, ifd0_offset: u32 }
#[derive(Debug, Clone, Copy)]
struct ThumbnailLocation { offset: u32, length: u32 }

fn parse_header(data: &[u8]) -> Result<TiffHeader, TiffError> {
    todo!()
}
fn read_ifd(data: &[u8], endian: Endian, offset: u32) -> Result<Ifd, TiffError> {
    todo!()
}
fn thumbnail_tags(ifd: &Ifd) -> Option<ThumbnailLocation> {  // find 0x0201 + 0x0202
    todo!()
}
fn subifd_offsets(data: &[u8], endian: Endian, ifd: &Ifd) -> Result<Vec<u32>, TiffError> { // tag 0x014A
    todo!()
}

const MAX_IFD_CHAIN: u32 = 16; // guard against corrupt/circular next-offset chains

fn find_thumbnail(data: &[u8], endian: Endian, ifd0_offset: u32) -> Result<ThumbnailLocation, TiffError> {
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
