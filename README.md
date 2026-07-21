# Hual

*Hari Up and Load.*

A local-only RAW photo ingestion and cataloguing engine, written in Rust, with a Tauri + React desktop UI on top.

No AI, no cloud services, no telemetry. Every .NEF/.ARW/.CR3 file already carries a pre-rendered JPEG preview in its metadata, the same one your camera's LCD uses, so hual never actually decodes RAW sensor data. It memory-maps the file, walks the container format's own binary structure, and pulls the embedded preview out directly. From there it's a hand-rolled LRU cache, a multithreaded ingestion pipeline, a SQLite metadata index, and a Tauri-wrapped frontend.

Performance was the main thing I cared about while building this, more than features or polish. That's why the file reads are memory-mapped instead of loaded whole, why the LRU cache is hand-rolled instead of a crate off the shelf, and why the worker pool is tuned by hand rather than left at defaults. The goal was always to make it hold up on a real photo library containing thousands of raw files, where loading each file as a whole is slow and impracticable.

It's also a from-scratch systems-engineering project in general: anything that could reasonably be hand-rolled instead of pulled in as a dependency has been, including the LRU cache, the TIFF/EXIF and ISO-BMFF parsers, the worker pool, and the scroll virtualisation.

## Contents

- [Getting started](#getting-started)
- [Tech stack](#tech-stack)
- [Project structure](#project-structure)
- [Architecture](#architecture)
  - [1. Thumbnail extraction](#1-thumbnail-extraction)
  - [2. Caching](#2-caching)
  - [3. Multithreaded ingestion pipeline](#3-multithreaded-ingestion-pipeline)
  - [4. Database indexing for metadata](#4-database-indexing-for-metadata)
  - [5. Tauri-wrapped frontend](#5-tauri-wrapped-frontend)
- [Contributing](#contributing)
- [Licence](#licence)

## Getting started

**Prerequisites**: Rust via [rustup](https://rustup.rs/) (edition 2024 needs
rustc 1.85+), Node.js + npm, and the Tauri CLI (`cargo install tauri-cli
--version "^2"`).

```sh
git clone <this-repo-url>
cd hual

# Run the full desktop app (Rust backend + Vite dev server, hot-reloading)
cargo tauri dev

# Or just the ingestion engine, headless, via the CLI:
cargo run -- thumb <input.arw> [output.jpg]   # extract embedded preview
cargo run -- info <input.arw>                  # dump parsed EXIF
cargo run -- import <source_dir> <dest_dir>    # full ingest, no UI

# Tests
cargo test           # Rust: ~115 tests across parsers, cache, pipeline, db
npm run build --prefix ui   # frontend type-check + production build

# Release bundle
cargo tauri build
```

## Tech stack

| Layer | Choice |
|---|---|
| Core engine | Rust |
| Database | [`rusqlite`](https://docs.rs/rusqlite) (bundled SQLite) |
| Desktop shell | [Tauri v2](https://tauri.app/) |
| Frontend | React + TypeScript + Vite |
| Icons | [`lucide-react`](https://lucide.dev/) |

## Project structure

```
hual/
├── src/                     Core engine — Tauri-agnostic, also used by tests/ and the CLI
│   ├── main.rs                CLI entry point (thumb / info / import)
│   ├── lib.rs                  Library root
│   ├── thumbnail.rs             Format dispatch — picks a parser by extension
│   │   ├── tiff.rs                TIFF/EXIF parser (NEF, ARW, DNG, ...) — mmap'd
│   │   ├── cr3.rs                 ISO-BMFF box parser (Canon CR3) — mmap'd
│   │   └── jpeg.rs                JPEG APP1/Exif segment parser + passthrough
│   ├── pipeline.rs              Ingestion orchestration — spawns the pipeline below
│   │   ├── scanner.rs             Recursive directory walk → RawFile
│   │   ├── worker.rs               Worker pool — EXIF + thumbnail extraction
│   │   ├── ssd_writer.rs           Disk writer (Copy & Import mode only)
│   │   └── db_writer.rs            Schema, inserts, filtered queries
│   ├── cache.rs                 L1/L2 re-exports
│   │   ├── l1.rs                   Hand-rolled O(1) in-memory LRU
│   │   └── l2.rs                   On-disk WebP thumbnail cache
│   └── hidden_dir.rs             `.hual/` metadata folder helper
├── src-tauri/               Tauri shell — thin IPC layer over the `hual` crate
│   └── src/
│       ├── lib.rs                Builder setup, managed state, command registration
│       └── commands.rs            #[tauri::command] handlers
├── ui/                      React + TypeScript frontend (Vite)
│   └── src/
│       ├── components/           Sidebar, ImportPanel, FilterPanel, PhotoGrid, Lightbox, ...
│       ├── hooks/                 usePhotos, useVirtualGrid, useImportProgress, ...
│       ├── lib/                   ipc.ts (typed invoke() wrappers), utils.ts
│       └── styles/
├── tests/unit/              Real unit tests against private functions via #[path] modules
└── Cargo.toml               Workspace root (`hual` + `src-tauri` members)
```

## Architecture

### 1. Thumbnail extraction

Zero-copy, no RAW decode. Every parser follows the same shape: `mmap` the
file once, then read everything else as slices into that mapping — no
intermediate buffers, no full-image decode, and no upfront read cost even
for a multi-hundred-megabyte file (the OS pages in only the bytes actually
touched).

```
 file on disk
      │
      ▼
 mmap (memmap2)   ──▶  &[u8], zero-copy view of the whole file
      │
      ▼
 parse_header()   ──▶  endianness + offset to IFD0 (TIFF) / ftyp box (CR3)
      │
      ▼
 walk container   ──▶  TIFF: IFD0 → SubIFDs → sibling IFD chain
                        CR3:  ISO-BMFF boxes → CMT./THMB/PRVW
      │
      ▼
 slice bytes[off..off+len]  ──▶  .to_vec()   (only allocation on the happy path)
```

Three formats share one dispatch (`thumbnail.rs`, matched by extension):
NEF/ARW/DNG/etc. via a hand-rolled **TIFF/EXIF IFD parser**, CR3 via a
hand-rolled **ISO-BMFF box parser**, and plain JPEGs via a **marker-segment
walker** that locates the `APP1`/`Exif` segment and hands the embedded TIFF
blob to the same TIFF parser — one parser, reused, rather than a fourth
implementation. Every multi-byte read goes through checked slicing (`.get()`,
never `[]`), because the input is an arbitrary, potentially-corrupt binary
file, not trusted memory.

### 2. Caching

Two layers: an O(1) in-memory LRU (L1) and an on-disk WebP cache (L2).

**L1** (`cache/l1.rs`) is a hand-rolled LRU with true O(1) `get`/`put`/evict —
no `Vec` shifting, no linked-list pointer-chasing allocations. The trick:
the doubly-linked list is built over a pre-allocated `Vec<Node>` **arena**,
where "pointers" are just `usize` indices into that vec, plus a `HashMap<K,
usize>` for O(1) key → index lookup:

```
map:  { key → index }              nodes:  [ Node{prev, next, key, value}, ... ]

HEAD ⇄ [3: most recent] ⇄ [0] ⇄ [7: least recent] ⇄ TAIL
```

Two dummy sentinel nodes (`HEAD`/`TAIL`) mean every real node always has a
valid `prev`/`next` — no `Option` branches on list boundaries. A `free: Vec
<usize>` list of unused arena slots means eviction reuses a slot instead of
deallocating, and insertion below capacity never touches the allocator at
all after the initial `Vec::with_capacity`. `get` does a hashmap lookup +
unlink/relink (pointer-index swaps, no traversal); `put` is the same plus,
on eviction, popping the tail node and removing its key from the map — both
strictly O(1) regardless of cache size. This backs the full-resolution
preview path (`get_preview`), the expensive one — a fresh RAW parse per miss.

**L2** (`cache/l2.rs`) is the on-disk grid-thumbnail cache: keyed by an
FNV-1a hash of the source path, it decodes the extracted preview, downscales
it (`image::thumbnail`) if larger than 256px, and re-encodes to WebP at
quality 80 — persistent across app restarts, unlike L1.

### 3. Multithreaded ingestion pipeline

A single `thread::scope` wires four stages together with bounded
`crossbeam-channel`s — structured concurrency, so every thread is guaranteed
joined before `run_import` returns, no detached threads, no `unsafe`:

```
                    ┌─────────────┐
   source dir  ───▶ │  scanner    │  (1 thread, recursive walk,
                    └──────┬──────┘   optional RAW-only filter,
                           │           skips .hual/ itself)
                     raw_tx│bounded(32)
                    ┌──────▼──────┐
                    │ worker pool │  (N = available_parallelism threads)
                    │ EXIF + thumb│  each: extract EXIF, extract/passthrough
                    │  extraction │  preview bytes, L2-cache the thumbnail
                    └──┬───────┬──┘
              write_tx │       │ db_tx
           bounded(32) │       │ bounded(32)
                 ┌─────▼──┐ ┌──▼──────┐
                 │ssd_writer│ │db_writer│  (1 thread each)
                 └──────────┘ └─────────┘
```

Progress is reported via a shared `AtomicUsize` counter plus a throttled
callback (`&dyn Fn(usize) + Sync`) threaded down into every worker — the
core engine has zero dependency on Tauri, so the callback is a plain
closure; the Tauri layer supplies one that throttles `emit()` calls to
~10/sec and the frontend listens for `import-progress` events. Two import
modes share this exact pipeline: **Copy & Import** sends a `WriteJob` to
`ssd_writer` and writes into a chosen destination; **Import Only** skips
that step entirely (`dest_dir: Option<&Path>` is `None`), indexing files
in place with `dest_path == src_path`.

### 4. Database indexing for metadata

SQLite via `rusqlite` (bundled — no system SQLite dependency), one table,
one composite index built for exactly the range queries the filter UI needs:

```sql
CREATE TABLE IF NOT EXISTS photos (
    id INTEGER PRIMARY KEY,
    src_path TEXT NOT NULL UNIQUE,
    dest_path TEXT NOT NULL,
    exposure_time REAL,
    f_stop REAL,
    focal_length REAL,
    iso INTEGER
);
CREATE INDEX IF NOT EXISTS idx_photos_exif
    ON photos (iso, f_stop, exposure_time, focal_length);
```

`list_photos` builds its `WHERE` clause dynamically — one `>=`/`<=` clause
per bound the user actually set, `AND`-joined — but only ever interpolates
fixed column/operator strings from a closed set; every user-supplied bound
is bound through a `?N` placeholder via `rusqlite::ToSql`, so there's no
injection surface despite the query shape being built at runtime.

### 5. Tauri-wrapped frontend

React + TypeScript, talking to the Rust backend through five typed
`#[tauri::command]`s (`list_photos`, `get_preview`, `get_webp`,
`import_photos`, `pick_dir`). The photo grid's scroll virtualisation —
windowing plus a `ResizeObserver`-driven column reflow — is hand-rolled
rather than a library, matching the backend's philosophy. A docked,
collapsible sidebar hosts import controls (Copy & Import / Import Only,
RAW-only filtering) and live, debounced range filters (ISO, f-stop,
exposure time, focal length); the Lightbox reuses the same cache-backed
preview fetch for prev/next navigation, so browsing between photos is just
changing *which* photo is requested, not a different code path.

## Contributing

- Match the existing philosophy: prefer hand-rolling a small, explainable
  piece of logic over pulling in a dependency for it, unless the dependency
  is doing something genuinely out of scope (image decode/encode, SQLite,
  channels).
- Untrusted input (anything read from a photo file) goes through checked
  slicing/parsing, never indexing or unwrapping that could panic on
  malformed data.
- Run `cargo test` and `npm run build --prefix ui` before opening a PR —
  both must pass cleanly.
- New parsing logic should ship with unit tests against synthetic byte
  fixtures (see `tests/unit/support.rs`), not just real sample files.
- Open an issue for anything beyond a small fix before investing time in a
  large change, so design direction can be agreed on first.

## Licence

MIT — see [LICENSE](LICENSE).
