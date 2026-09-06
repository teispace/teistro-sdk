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

/// Every file under a generated directory that no output claims.
///
/// A generator that writes a tree, rather than a fixed list of files,
/// leaves a stale file behind whenever an item is removed: the reference
/// page of an entry point that no longer exists still builds, still
/// appears in the sidebar, and still says the library has a call it does
/// not. Nothing compares a file against a generator that never mentions
/// it, so the directory is compared against the whole list instead.
pub(crate) fn strays(root: &Path, dir: &str, outputs: &[Output]) -> Vec<String> {
    let base = root.join(dir);
    let mut found = Vec::new();
    collect(&base, &mut found);
    found
        .iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
            let relative = relative.replace('\\', "/");
            (!outputs.iter().any(|o| o.path == relative)).then_some(relative)
        })
        .collect()
}

/// Every file under a directory, recursively.
fn collect(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<std::path::PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            collect(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// Removes what no output claims, so that writing a tree leaves only the
/// tree.
pub(crate) fn prune(root: &Path, dir: &str, outputs: &[Output]) {
    for stray in strays(root, dir, outputs) {
        let path = root.join(&stray);
        match std::fs::remove_file(&path) {
            Ok(()) => println!("removed {stray}"),
            Err(err) => eprintln!("cannot remove {stray}: {err}"),
        }
    }
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
