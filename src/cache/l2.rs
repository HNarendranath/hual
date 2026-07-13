use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

const QUALITY: f32 = 80.0;
const MAX_DIMENSTION: u32 = 256;

#[derive(Debug)]
pub enum L2CacheError {
    Io(io::Error),
    Decode(image::ImageError),
    Encode(String),
}

impl fmt::Display for L2CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            L2CacheError::Io(e) => write!(f, "io error: {e}"),
            L2CacheError::Decode(e) => write!(f, "decode error: {e}"),
            L2CacheError::Encode(e) => write!(f, "encode error: {e}"),
        }
    }
}

impl std::error::Error for L2CacheError {}
impl From<io::Error> for L2CacheError {
    fn from(e: io::Error) -> Self {
        L2CacheError::Io(e)
    }
}
impl From<image::ImageError> for L2CacheError {
    fn from(e: image::ImageError) -> Self {
        L2CacheError::Decode(e)
    }
}

pub struct L2Cache {
    root: PathBuf,
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    bytes
        .iter()
        .fold(OFFSET, |hash, &b| (hash ^ b as u64).wrapping_mul(PRIME))
}

impl L2Cache {
    pub fn new(root: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn webp_path(&self, key: &str) -> PathBuf {
        self.root
            .join(format!("{:16x}.webp", fnv1a_64(key.as_bytes())))
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        fs::read(self.webp_path(key)).ok()
    }

    pub fn put(&self, key: &str, jpeg_bytes: &[u8]) -> Result<(), L2CacheError> {
        let img = image::load_from_memory_with_format(jpeg_bytes, image::ImageFormat::Jpeg)?;
        let thumb = if img.width() > MAX_DIMENSTION || img.height() > MAX_DIMENSTION {
            img.thumbnail(MAX_DIMENSTION, MAX_DIMENSTION)
        } else {
            img
        };
        let encoder =
            webp::Encoder::from_image(&thumb).map_err(|e| L2CacheError::Encode(e.to_string()))?;
        let webp_bytes = encoder.encode(QUALITY);
        fs::write(self.webp_path(key), &*webp_bytes)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/l2_tests.rs"]
mod l2_tests;
