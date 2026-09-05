//! What every generator shares: writing its outputs, and the gate that
//! regenerates them in memory and fails on any difference, so a
//! checked-in artefact can never drift from its sources.

use std::path::Path;

/// One generated file: its repository-relative path and its content.
pub(crate) struct Output {
    pub(crate) path: &'static str,
    pub(crate) text: String,
}

/// Writes every output.
pub(crate) fn write(root: &Path, outputs: &[Output]) -> i32 {
    for output in outputs {
        std::fs::write(root.join(output.path), &output.text)
            .unwrap_or_else(|err| panic!("cannot write {}: {err}", output.path));
        println!("wrote {} ({} bytes)", output.path, output.text.len());
    }
    0
}

/// Compares every output with the checked-in file.
pub(crate) fn check(root: &Path, outputs: &[Output], generator: &str) -> i32 {
    let mut failures = 0;
    for output in outputs {
        let actual = std::fs::read_to_string(root.join(output.path)).unwrap_or_default();
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
