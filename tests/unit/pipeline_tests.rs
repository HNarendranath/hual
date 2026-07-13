mod support;

use super::*;
use support::{write_header, write_ifd, TempDir};

fn synthetic_tiff() -> Vec<u8> {
    let mut data = Vec::new();
    write_header(&mut data, true);
    write_ifd(&mut data, &[], 0, true);
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
fn run_import_on_empty_source_dir_does_nothing_and_does_not_hang() {
    let source = TempDir::new("pipeline_import_empty_source");
    let dest = TempDir::new("pipeline_import_empty_dest");

    run_import(&source.path, &dest.path);
    // nothing to assert beyond "this returned" -- proves the whole
    // scanner -> worker -> writer -> db_writer shutdown chain completes
    // cleanly even when zero files ever flow through it.
}
