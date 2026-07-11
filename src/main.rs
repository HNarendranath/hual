mod cache;
mod thumbnail;

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
