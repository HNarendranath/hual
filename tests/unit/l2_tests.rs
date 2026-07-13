mod support;

use super::*;
use support::{tiny_jpeg, TempDir};

#[test]
fn new_creates_root_directory_if_missing() {
    let dir = TempDir::new("l2_new");
    let root = dir.path.join("thumbcache");
    assert!(!root.exists());
    L2Cache::new(root.clone()).unwrap();
    assert!(root.is_dir());
}

#[test]
fn get_on_missing_key_returns_none() {
    let dir = TempDir::new("l2_miss");
    let cache = L2Cache::new(dir.path.clone()).unwrap();
    assert!(cache.get("nonexistent").is_none());
}

#[test]
fn put_then_get_roundtrips_to_valid_webp() {
    let dir = TempDir::new("l2_roundtrip");
    let cache = L2Cache::new(dir.path.clone()).unwrap();
    let jpeg = tiny_jpeg(4, 4, [200, 50, 50]);

    cache.put("photo-a", &jpeg).unwrap();
    let webp_bytes = cache.get("photo-a").expect("just-written entry should hit");

    assert_eq!(&webp_bytes[0..4], b"RIFF");
    assert_eq!(&webp_bytes[8..12], b"WEBP");

    let decoded = webp::Decoder::new(&webp_bytes).decode().expect("valid webp");
    assert_eq!(decoded.width(), 4);
    assert_eq!(decoded.height(), 4);
}

#[test]
fn different_keys_produce_independent_entries() {
    let dir = TempDir::new("l2_distinct_keys");
    let cache = L2Cache::new(dir.path.clone()).unwrap();
    let jpeg = tiny_jpeg(4, 4, [10, 20, 30]);

    cache.put("photo-a", &jpeg).unwrap();
    cache.put("photo-b", &jpeg).unwrap();

    assert!(cache.get("photo-a").is_some());
    assert!(cache.get("photo-b").is_some());
    // two distinct files on disk, not one overwriting the other
    let entries = std::fs::read_dir(&dir.path).unwrap().count();
    assert_eq!(entries, 2);
}

#[test]
fn put_overwrites_existing_entry_for_same_key() {
    let dir = TempDir::new("l2_overwrite");
    let cache = L2Cache::new(dir.path.clone()).unwrap();

    cache.put("photo-a", &tiny_jpeg(4, 4, [0, 0, 0])).unwrap();
    let first = cache.get("photo-a").unwrap();

    cache.put("photo-a", &tiny_jpeg(8, 8, [255, 255, 255])).unwrap();
    let second = cache.get("photo-a").unwrap();

    assert_ne!(first, second);
    let entries = std::fs::read_dir(&dir.path).unwrap().count();
    assert_eq!(entries, 1, "overwrite should not leave a stale second file");

    let decoded = webp::Decoder::new(&second).decode().unwrap();
    assert_eq!((decoded.width(), decoded.height()), (8, 8));
}

#[test]
fn put_with_invalid_jpeg_bytes_returns_err() {
    let dir = TempDir::new("l2_invalid_input");
    let cache = L2Cache::new(dir.path.clone()).unwrap();
    let result = cache.put("bad", b"not a jpeg at all");
    assert!(result.is_err());
    assert!(cache.get("bad").is_none());
}

#[test]
fn put_downscales_images_larger_than_max_dimension() {
    let dir = TempDir::new("l2_downscale");
    let cache = L2Cache::new(dir.path.clone()).unwrap();
    // Larger than the 256px cap on both axes, 5:3 aspect ratio -- mirrors
    // CR3's real ~1620x1080 PRVW preview being much bigger than what L2
    // should ever actually store.
    let jpeg = tiny_jpeg(1000, 600, [128, 128, 128]);

    cache.put("big-photo", &jpeg).unwrap();
    let webp_bytes = cache.get("big-photo").unwrap();
    let decoded = webp::Decoder::new(&webp_bytes).decode().unwrap();

    assert!(decoded.width() <= 256 && decoded.height() <= 256);
    assert_eq!(decoded.width(), 256, "long edge should hit the cap exactly");
    // aspect ratio preserved: 1000/600 == 256/height, so height should land near 154
    assert!((150..=158).contains(&decoded.height()));
}

#[test]
fn put_does_not_upscale_images_smaller_than_max_dimension() {
    let dir = TempDir::new("l2_no_upscale");
    let cache = L2Cache::new(dir.path.clone()).unwrap();
    // Smaller than the 256px cap on both axes -- e.g. a TIFF-based format's
    // tiny ~160x120 IFD1 thumbnail. Should pass through unchanged, not get
    // upscaled/blurred to fill the box.
    let jpeg = tiny_jpeg(160, 120, [64, 64, 64]);

    cache.put("small-photo", &jpeg).unwrap();
    let webp_bytes = cache.get("small-photo").unwrap();
    let decoded = webp::Decoder::new(&webp_bytes).decode().unwrap();

    assert_eq!((decoded.width(), decoded.height()), (160, 120));
}
