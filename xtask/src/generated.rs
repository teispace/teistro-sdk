//! What every generator shares: writing its outputs, and the gate that
//! regenerates them in memory and fails on any difference, so a
//! checked-in artefact can never drift from its sources.

use std::borrow::Cow;
use std::path::Path;

/// One generated file: its repository-relative path and its content. The
/// path is borrowed when a generator knows it at compile time and owned
/// when it computes one (a derived locale's files).
pub(crate) struct Output {
    pub(crate) path: Cow<'static, str>,
    pub(crate) text: String,
}

impl Output {
    /// One output, whatever kind of path it was given.
    pub(crate) fn new(path: impl Into<Cow<'static, str>>, text: String) -> Output {
        Output {
            path: path.into(),
            text,
        }
    }
}

/// Writes every output.
pub(crate) fn write(root: &Path, outputs: &[Output]) -> i32 {
    for output in outputs {
        let path = root.join(output.path.as_ref());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|err| panic!("cannot create {}: {err}", parent.display()));
        }
        std::fs::write(&path, &output.text)
            .unwrap_or_else(|err| panic!("cannot write {}: {err}", output.path));
        println!("wrote {} ({} bytes)", output.path, output.text.len());
    }
    0
}

/// Compares every output with the checked-in file.
pub(crate) fn check(root: &Path, outputs: &[Output], generator: &str) -> i32 {
    let mut failures = 0;
    for output in outputs {
        let actual = std::fs::read_to_string(root.join(output.path.as_ref())).unwrap_or_default();
        if actual == output.text {
            println!("ok    {}", output.path);
        } else {
            println!(
                "FAIL  {} differs from what `{generator}` produces",
                output.path
            );
            failures += 1;
        }
    }
    failures
}
