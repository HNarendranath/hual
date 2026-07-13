mod support;

use super::*;
use support::{entry, ifd_len, tiny_jpeg, write_header, write_ifd, TempDir, IFD0_OFFSET};

fn synthetic_tiff() -> Vec<u8> {
    let mut data = Vec::new();
    write_header(&mut data, true);
    write_ifd(&mut data, &[], 0, true);
    data
}

fn synthetic_tiff_with_thumbnail() -> Vec<u8> {
    let thumb_bytes = tiny_jpeg(4, 4, [10, 200, 10]);

    let ifd1_offset = IFD0_OFFSET + ifd_len(0);
    let thumb_offset = ifd1_offset + ifd_len(2);
    let ifd1_entries = vec![
        entry(0x0201, 4, 1, thumb_offset),
        entry(0x0202, 4, 1, thumb_bytes.len() as u32),
    ];

    let mut data = Vec::new();
    write_header(&mut data, true);
    write_ifd(&mut data, &[], ifd1_offset, true);
    write_ifd(&mut data, &ifd1_entries, 0, true);
    data.extend_from_slice(&thumb_bytes);
    data
}

#[test]
fn run_import_preserves_relative_structure_and_copies_bytes() {
    let source = TempDir::new("pipeline_import_source");
    let dest = TempDir::new("pipeline_import_dest");

    let a = synthetic_tiff();
    let b = synthetic_tiff();
    source.write_file("a.arw", &a);
    source.write_file("sub/b.arw", &b);

    run_import(&source.path, &dest.path);

    assert_eq!(std::fs::read(dest.path.join("a.arw")).unwrap(), a);
    assert_eq!(std::fs::read(dest.path.join("sub").join("b.arw")).unwrap(), b);
}

#[test]
fn run_import_copies_unsupported_files_too() {
    let source = TempDir::new("pipeline_import_unsupported_source");
    let dest = TempDir::new("pipeline_import_unsupported_dest");

    source.write_file("notes.txt", b"just some notes");

    run_import(&source.path, &dest.path);

    assert_eq!(std::fs::read(dest.path.join("notes.txt")).unwrap(), b"just some notes");
}

#[test]
fn run_import_populates_l2_thumbnail_cache() {
    let source = TempDir::new("pipeline_l2_cache_source");
    let dest = TempDir::new("pipeline_l2_cache_dest");

    source.write_file("photo.arw", &synthetic_tiff_with_thumbnail());

    run_import(&source.path, &dest.path);

    let thumbcache_dir = dest.path.join(".hual").join("thumbcache");
    let entries: Vec<_> = std::fs::read_dir(&thumbcache_dir)
        .expect("thumbcache dir should exist")
        .collect();
    assert_eq!(entries.len(), 1);
    let cached = entries[0].as_ref().unwrap();
    assert_eq!(cached.path().extension().unwrap(), "webp");
}

#[test]
fn run_import_on_empty_source_dir_does_nothing_and_does_not_hang() {
    let source = TempDir::new("pipeline_import_empty_source");
    let dest = TempDir::new("pipeline_import_empty_dest");

    run_import(&source.path, &dest.path);
    // nothing to assert beyond "this returned" -- proves the whole
    // scanner -> worker -> writer -> db_writer shutdown chain completes
    // cleanly even when zero files ever flow through it.
}
