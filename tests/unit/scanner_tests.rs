mod support;

use super::*;
use crossbeam_channel::unbounded;
use support::TempDir;

#[test]
fn finds_files_in_nested_subdirectories() {
    let dir = TempDir::new("scanner_nested");
    dir.write_file("a.txt", b"hello");
    dir.write_file("sub/b.txt", b"world");
    dir.write_file("sub/deeper/c.txt", b"!");

    let (tx, rx) = unbounded();
    run(&dir.path, tx);

    let mut found: Vec<_> = rx.iter().collect();
    found.sort_by(|a, b| a.src_path.cmp(&b.src_path));

    assert_eq!(found.len(), 3);
    assert!(found.iter().any(|f| f.src_path.ends_with("a.txt") && f.bytes == b"hello"));
    assert!(found.iter().any(|f| f.src_path.ends_with("b.txt") && f.bytes == b"world"));
    assert!(found.iter().any(|f| f.src_path.ends_with("c.txt") && f.bytes == b"!"));
}

#[test]
fn includes_files_regardless_of_extension() {
    let dir = TempDir::new("scanner_ext");
    dir.write_file("no_extension", b"data1");
    dir.write_file("photo.xyz", b"data2");

    let (tx, rx) = unbounded();
    run(&dir.path, tx);

    assert_eq!(rx.iter().count(), 2);
}

#[test]
fn nonexistent_source_dir_yields_no_files_without_panicking() {
    let missing = std::env::temp_dir().join("hual_test_scanner_does_not_exist_12345");

    let (tx, rx) = unbounded();
    run(&missing, tx);

    assert_eq!(rx.iter().count(), 0);
}

#[test]
fn skips_symlinks() {
    let dir = TempDir::new("scanner_symlink");
    let real_path = dir.write_file("real.txt", b"hello");
    let link_path = dir.path.join("link.txt");

    #[cfg(unix)]
    let created = std::os::unix::fs::symlink(&real_path, &link_path).is_ok();
    #[cfg(windows)]
    let created = std::os::windows::fs::symlink_file(&real_path, &link_path).is_ok();

    if !created {
        eprintln!("skipping skips_symlinks: could not create a symlink on this machine (needs Developer Mode or elevated privileges on Windows)");
        return;
    }

    let (tx, rx) = unbounded();
    run(&dir.path, tx);

    let found: Vec<_> = rx.iter().collect();
    assert_eq!(found.len(), 1);
    assert!(found[0].src_path.ends_with("real.txt"));
}
