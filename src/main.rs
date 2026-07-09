mod tiff;
mod cr3;
mod thumbnail;

use memmap2::Mmap;
use std::fs::File;

fn main() -> std::io::Result<()> {
    let file = File::open("test.arw")?;
    let mmap = unsafe { Mmap::map(&file)? };

    println!("File is {} bytes", mmap.len());
    println!("First 8 bytes: {:?}", &mmap[0..8]);

    Ok(())
}