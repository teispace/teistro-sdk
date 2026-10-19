//! Writes the shipped rule packs and tables into `OUT_DIR` without the
//! whitespace between their tokens, for `src/shipped.rs` and
//! `src/table.rs` to embed. The files stay laid out for reading in the
//! repository, and every library, addon and wasm module carries the half
//! of their bytes that was indentation (2.7 MB of 5.8 MB when this was
//! written). Strings and numbers pass through as written, so a pack reads
//! the same.

// A build script stops the build by panicking and speaks to cargo through
// stdout, so the library lints against both are allowed here.
#![allow(
    clippy::print_stdout,
    clippy::expect_used,
    clippy::panic,
    reason = "a build script"
)]

use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    for dir in ["rules", "tables"] {
        // A directory is rescanned whole: a pack added, changed or removed
        // reruns the script.
        println!("cargo:rerun-if-changed={dir}");
        let target = out.join(dir);
        fs::create_dir_all(&target).expect("OUT_DIR is writable");
        for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("{dir}: {error}")) {
            let path = entry.expect("a directory entry reads").path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                write(&path, &target);
            }
        }
    }
}

fn write(path: &Path, target: &Path) {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let name = path.file_name().expect("a file has a name");
    fs::write(target.join(name), compact(&text)).expect("OUT_DIR is writable");
}

/// `text` without the whitespace outside its strings: a JSON document
/// keeps its meaning, every string and number byte for byte.
fn compact(text: &str) -> String {
    let mut out = String::with_capacity(text.len() / 2);
    let (mut quoted, mut escaped) = (false, false);
    for c in text.chars() {
        if quoted {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                quoted = false;
            }
        } else if c == '"' {
            quoted = true;
            out.push(c);
        } else if !c.is_ascii_whitespace() {
            out.push(c);
        }
    }
    out
}
