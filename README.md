# Hual

A high-performance, local-only RAW photo ingestion and cataloging engine, written in Rust.

No AI, no cloud services, no telemetry. Just a mmap'd file, a hand-rolled binary parser, and
the goal of pulling a usable preview image out of a multi-megabyte RAW file without decoding
a single pixel of sensor data.

This is a from-scratch systems-engineering project (and a deliberate vehicle for learning
Rust properly) — every piece that could reasonably be hand-rolled instead of pulled in as a
dependency, is.

## Status

**Phase 1 (TIFF-based thumbnail extraction) is functional.** Everything else below is planned.

| Phase | Description | Status |
|---|---|---|
| 1 | NEF / ARW (TIFF/EXIF) embedded thumbnail extraction | ✅ Done |
| 2 | CR3 (ISO-BMFF) thumbnail extraction | ⬜ Not started |
| 3 | Directory scanner + producer/consumer worker pool | ⬜ Not started |
| 4 | SQLite metadata store + composite-index queries | ⬜ Not started |
| 5 | L1 (in-RAM LRU) + L2 (on-disk WebP) thumbnail cache | ⬜ Not started |
| 6 | Tauri UI shell | ⬜ Not started |

## Why this exists

Most photo catalog tools decode the full RAW sensor image just to show you a preview. That's
enormously wasteful: every `.NEF`/`.ARW`/`.CR3` already has a pre-rendered JPEG preview
embedded in its metadata — the same one your camera's LCD uses. Hual's entire premise is:
**never decode the RAW pixel data at all** for the purposes of browsing/cataloging. Instead:

1. `mmap` the file instead of reading it into a heap buffer — the OS pages in only the bytes
   actually touched, and there's no upfront read cost for a multi-hundred-megabyte file.
2. Parse the container's binary structure directly (TIFF IFD chains for NEF/ARW, ISO-BMFF
   boxes for CR3) to find exactly where the embedded preview lives.
3. Slice those bytes straight out of the mapped memory — zero-copy until the final write.

This is also why the design deliberately avoids some "obvious" shortcuts:
- **No `libraw` or similar full RAW-decode library** — that would defeat the entire point.
- **No `rayon`** for the eventual worker pool — it's hand-rolled with `crossbeam-channel` on
  purpose, so the concurrency design is a visible, explained part of the project rather than
  hidden behind a library call.
- **Bounds-checked, not panicking** — every multi-byte read out of the mmap'd bytes goes
  through checked slicing (`.get()`, not `[]`), because the input is an arbitrary,
  potentially-corrupt binary file, not trusted memory.

## Tech stack

| Concern | Choice | Why |
|---|---|---|
| Language | Rust, edition 2024 | Ownership/borrowing map well onto the cache + worker-pool design; also the point of the project is learning Rust |
| Memory-mapped I/O | [`memmap2`](https://docs.rs/memmap2) | Zero-copy file access |
| Endian-aware parsing | [`byteorder`](https://docs.rs/byteorder) | Safe, alignment-independent multi-byte reads from arbitrary byte offsets |
| Threading (planned) | `crossbeam-channel` + hand-rolled worker pool | Visible, explained concurrency design — not hidden behind `rayon` |
| Database (planned) | `rusqlite` (bundled SQLite) | Composite-index metadata queries (lens, aperture, ISO, ...) |
| Thumbnail re-encode (planned) | `webp` | L2 on-disk cache format |
| UI (planned) | [Tauri](https://tauri.app/) | Rust backend + web frontend, once the backend is proven |

## Project structure

```
hual/
├── src/
│   ├── main.rs           CLI entry point — arg parsing, dispatch, exit codes
│   ├── tiff.rs           TIFF/EXIF parser for NEF & ARW — IMPLEMENTED
│   ├── cr3.rs            ISO-BMFF parser for CR3 — planned (Phase 2), currently empty
│   └── thumbnail.rs      Shared extractor trait across formats — deferred until cr3.rs exists
├── tests/
│   └── unit/
│       ├── tiff_tests.rs Unit tests for tiff.rs's private parsing functions
│       └── support.rs    Synthetic TIFF byte-buffer builder + temp-file fixture helper
├── Cargo.toml
├── .gitignore            ignores raw source files (*.arw/*.nef/*.cr3) and *.jpg output
└── README.md
```

`tiff_tests.rs` lives under `tests/` but is wired into `tiff.rs` via a `#[path]`-attributed
`mod` declaration, rather than as a standalone `cargo`-discovered integration test — this
gives it access to `tiff.rs`'s private functions (real unit tests), while keeping the test
code itself out of `src/`.

### How thumbnail extraction actually works

```
 file on disk
      │
      ▼
 mmap (memmap2)  ──▶  &[u8], zero-copy view of the whole file
      │
      ▼
 parse_header()  ──▶  endianness ("II"/"MM") + offset to IFD0
      │
      ▼
 find_thumbnail() ──▶ walks IFD0 → SubIFDs → sibling IFD chain,
      │               looking for tags 0x0201/0x0202 (thumbnail offset + length)
      ▼
 slice data[offset..offset+length]  ──▶  .to_vec()  ──▶  written to disk as .jpg
```

No intermediate buffers, no full-image decode — the only allocation on the happy path is the
final owned `Vec<u8>` returned to the caller.

## Getting started

### Prerequisites

- Rust via [rustup](https://rustup.rs/) — edition 2024 requires **rustc 1.85 or newer**.
  Check with `rustc --version`; update with `rustup update`.

### Clone & build

```sh
git clone <this-repo-url>
cd hual
cargo build --release
```

No other setup is required — `memmap2` and `byteorder` are pure-Rust and pulled in
automatically by Cargo on first build.

### Run

```sh
cargo run --release -- <input.nef|input.arw> [output.jpg]
```

- `output.jpg` is optional — if omitted, it defaults to the input path with its extension
  swapped to `.jpg` (e.g. `photo.arw` → `photo.jpg`).
- Exit code is `0` on success, `1` on any failure (missing file, unrecognized/corrupt TIFF
  structure, no embedded thumbnail found, etc.), with a message printed to stderr.

Example:

```sh
cargo run --release -- samples/DSC00123.ARW
# Wrote 9189 bytes to samples/DSC00123.jpg
```

### Test

```sh
cargo test
```

Runs the full unit test suite (currently 22 tests covering header parsing, IFD walking,
endianness handling, the SubIFD/sibling-IFD thumbnail search fallback chain, and end-to-end
extraction against synthetic in-memory TIFF fixtures).

## Validating against real files

Phase 1's correctness target is matching a real EXIF tool's output on actual camera files.
With [`exiftool`](https://exiftool.org/) installed:

```sh
cargo run --release -- test.ARW out.jpg
exiftool -b -PreviewImage test.ARW > expected.jpg
# compare out.jpg and expected.jpg
```

(Real `.ARW`/`.NEF`/`.CR3` sample files are gitignored — they're large binary camera output,
not source — so you'll need your own to test against.)

## License

No license has been chosen yet — treat this as all-rights-reserved for now. This will be
revisited before the project is considered stable.
