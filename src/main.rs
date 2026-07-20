mod cache;
mod hidden_dir;
mod pipeline;
mod thumbnail;

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);

    let Some(cmd) = args.next() else {
        eprintln!("Usage: hual <thumb|info> <input> [output]");
        return ExitCode::FAILURE;
    };

    match cmd.to_str() {
        Some("thumb") => thumbnail(args),
        Some("info") => exif(args),
        Some("import") => import(args),
        _ => {
            eprintln!("Usage: hual <thumb|info> <input> [output]");
            return ExitCode::FAILURE;
        }
    };

    ExitCode::SUCCESS
}

fn thumbnail(mut args: impl Iterator<Item = OsString>) -> ExitCode {
    let Some(input) = args.next() else {
        eprintln!("Usage: hual thumb <input> [output]");
        return ExitCode::FAILURE;
    };

    let input = PathBuf::from(input);
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| input.with_extension("jpg"));

    let result = thumbnail::extract_thumbnail(&input);

    let bytes = match result {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error extracting thumbnail from {}: {e}", input.display());
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = std::fs::write(&output, &bytes) {
        eprintln!("Error writing {}: {e}", output.display());
        return ExitCode::FAILURE;
    }

    println!("Wrote {} bytes to {}", bytes.len(), output.display());
    ExitCode::SUCCESS
}

fn exif(mut args: impl Iterator<Item = OsString>) -> ExitCode {
    let Some(input) = args.next() else {
        eprintln!("Usage: hual info <input>");
        return ExitCode::FAILURE;
    };

    let input = PathBuf::from(input);
    let ext = input.extension().and_then(|e| e.to_str()).unwrap_or("");

    let bytes = match std::fs::read(&input) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error reading {}: {e}", input.display());
            return ExitCode::FAILURE;
        }
    };

    match thumbnail::extract_exif_from_bytes(&bytes, ext) {
        Ok(exif) => {
            println!("{exif:#?}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Err extracting EXIF from {}: {e}", input.display());
            ExitCode::FAILURE
        }
    }
}

fn import(mut args: impl Iterator<Item = OsString>) -> ExitCode {
    let Some(source) = args.next() else {
        eprintln!("Usage: hual info <input>");
        return ExitCode::FAILURE;
    };
    let Some(dest) = args.next() else {
        eprintln!("Usage: hual import <source_dir> <dest_dir>");
        return ExitCode::FAILURE;
    };

    pipeline::run_import(
        &PathBuf::from(source),
        pipeline::ImportMode::CopyAndImport(PathBuf::from(dest)),
        |_| {},
    );
    ExitCode::SUCCESS
}
