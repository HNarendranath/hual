mod cr3;
mod thumbnail;
mod tiff;

use std::backtrace::BacktraceStatus::Unsupported;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);

    let Some(input) = args.next() else {
        eprintln!("Usage: hual <input.tiff>");
        return ExitCode::FAILURE;
    };

    let input = PathBuf::from(input);
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| input.with_extension("jpg"));

    const TIFF_EXTENSIONS: &[&str] = &[
        "tiff", "tif", "arw", "cr2", "nef", "raf", "orf", "dng", "nrw", "rw2", "pef", "iiq", "3fr",
        "fff", "sr2", "srf",
    ];

    let extension = input.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let ext_lowercase = extension.to_ascii_lowercase();
    let result: Result<Vec<u8>, Box<dyn std::error::Error>> = if ext_lowercase == "cr3" {
        cr3::extract_thumbnail(&input).map_err(|e| e.into())
    } else if TIFF_EXTENSIONS.contains(&ext_lowercase.as_str()) {
        tiff::extract_thumbnail(&input).map_err(|e| e.into())
    } else if ext_lowercase.is_empty() {
        Err(format!("File '{}' has no extension", input.display()).into())
    } else {
        Err(format!("Unsupported file format: '.{}'", extension).into())
    };

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
