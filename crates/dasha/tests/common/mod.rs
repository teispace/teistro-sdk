//! What the corpus tests share: where the corpus is, how a recorded file is
//! read, and the tree walked to a depth.

#![allow(
    dead_code,
    clippy::panic,
    clippy::unwrap_used,
    reason = "each test binary uses what it needs of these, and fails by panicking"
)]

use std::path::{Path as FsPath, PathBuf};

use serde_json::Value;
use teistro_dasha::{Period, Timeline};

/// A boundary's bound, in days: a tenth of a millisecond above the worst
/// the measurements found, and ten thousand times inside the corpus's
/// tolerance.
pub(crate) const BOUND_DAYS: f64 = 1e-8;

/// The corpus's baseline directory.
pub(crate) fn baseline() -> PathBuf {
    FsPath::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline")
}

/// One baseline file, by its path under the baseline directory.
pub(crate) fn read(relative: &str) -> Value {
    let path = baseline().join(relative);
    serde_json::from_str(
        &std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{relative}: {err}")),
    )
    .unwrap()
}

/// Every JSON file of a baseline subdirectory, sorted, with its name.
pub(crate) fn files(dir: &str) -> Vec<(String, Value)> {
    let root = baseline().join(dir);
    let mut paths: Vec<_> = std::fs::read_dir(&root)
        .unwrap_or_else(|err| panic!("{dir}: {err}; is the fixtures submodule checked out?"))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let json = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            (
                path.file_name().unwrap().to_string_lossy().into_owned(),
                json,
            )
        })
        .collect()
}

/// A recorded number.
pub(crate) fn jd(value: &Value) -> f64 {
    value
        .as_f64()
        .unwrap_or_else(|| panic!("{value} is not a number"))
}

/// Every period of the birth cycle to `depth`, depth first.
pub(crate) fn tree(dasha: &impl Timeline, depth: usize) -> Vec<Period> {
    fn walk(dasha: &impl Timeline, period: Period, depth: usize, out: &mut Vec<Period>) {
        out.push(period);
        if period.path.depth() < depth {
            for child in dasha.children(&period) {
                walk(dasha, child, depth, out);
            }
        }
    }
    let mut out = Vec::new();
    for maha in dasha.mahadashas() {
        walk(dasha, maha, depth, &mut out);
    }
    out
}
