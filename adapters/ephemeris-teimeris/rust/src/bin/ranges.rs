//! Writes the manifest of the engine's published data blocks
//! (`data/ranges.json`): each `.se1` block file by name, with its size,
//! its SHA-256 and the span its own header records, read from the
//! engine's full data set. The span is the engine's reading of the file,
//! not its name's: the earliest and latest blocks hold less than their
//! names' 600 years. The adapter checks
//! every block it is handed against this manifest, so a truncated or
//! altered download is refused by name rather than read.
//!
//! ```text
//! cargo run --release --bin teistro-ephemeris-teimeris-ranges > data/ranges.json
//! ```
//!
//! The set read is `$TEIMERIS_DATA_DIR/ext`, the engine's whole published
//! span (13200 BC to AD 17191), else the directory given as the one
//! argument.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a tooling binary prints its manifest and stops on a missing file"
)]

use std::path::PathBuf;
use std::process::ExitCode;

use teimeris::{Body, Context, Flags};
use teistro_ephemeris_teimeris::data_dir_from_env;
use teistro_ephemeris_teimeris::ranges::{Manifest, SCHEMA};
use teistro_port_ephemeris::{DataHash, sefile};

/// Every file's own span, as the engine read it: each block opened by
/// asking for the body its stem serves at instants 50 years apart across
/// the block's nominal years, then the context's open files listed.
fn spans(dir: &std::path::Path, names: &[String]) -> Result<Vec<(String, f64, f64)>, String> {
    let context = Context::builder()
        .ephemeris_path(dir.to_string_lossy())
        .open()
        .map_err(|e| e.to_string())?;
    for name in names {
        let Some((first, _)) = sefile::block_years(name) else {
            continue;
        };
        let body = match name.get(..4) {
            Some("semo") => Body::MOON,
            Some("seas") => Body::CHIRON,
            _ => Body::SUN,
        };
        for offset in (0..600).step_by(50) {
            let jd = sefile::civil_julian_day(first + offset, 7, 1);
            // A miss is an instant the file does not hold; the listing
            // below says which files opened.
            let _ = context.position(jd, body, Flags::EPH_SWISS);
        }
    }
    Ok(context
        .loaded_files()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|file| (file.name, file.jd_start, file.jd_end))
        .collect())
}

/// The names of a set of hashed files.
fn names_of(files: &[DataHash]) -> Vec<String> {
    files.iter().map(|hash| hash.file.clone()).collect()
}

fn main() -> ExitCode {
    let dir = std::env::args_os()
        .nth(1)
        .map_or_else(|| data_dir_from_env().join("ext"), PathBuf::from);
    let names = match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| sefile::block_years(name).is_some())
            .collect::<Vec<String>>(),
        Err(error) => {
            eprintln!("{}: {error}", dir.display());
            return ExitCode::FAILURE;
        }
    };
    let mut files = Vec::with_capacity(names.len());
    for name in names {
        match DataHash::of_file(&dir.join(&name)) {
            Ok(hash) => files.push(hash),
            Err(error) => {
                eprintln!("{name}: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    let opened = match spans(&dir, &names_of(&files)) {
        Ok(opened) => opened,
        Err(error) => {
            eprintln!("the engine: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut recorded = Vec::with_capacity(files.len());
    for hash in files {
        let Some(&(_, first, last)) = opened.iter().find(|(name, ..)| *name == hash.file) else {
            eprintln!("{}: the engine opened no instant of it", hash.file);
            return ExitCode::FAILURE;
        };
        recorded.push((hash, (first, last)));
    }
    let manifest = Manifest::of(SCHEMA, recorded);
    match serde_json::to_string_pretty(&manifest) {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
