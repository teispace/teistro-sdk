//! The falsification pass over **the engine passthrough**: which of an
//! engine's own functions the SDK can offer at `sdk.engine.*`, which it
//! must not, and which it cannot yet.
//!
//! `cargo xtask engine` writes the page; `check-engine` regenerates it in
//! memory and fails on any difference, so a figure on it is a figure this
//! build produces from the engine's own description.
//!
//! # Why this is a pass and not a paragraph
//!
//! ADR-0030 puts the engine's own functions at `sdk.engine.*` for the
//! flexibility of reaching what the SDK has not ported. The port's own
//! contract says how: "the marshalling belongs to the adapter, generated
//! from the engine's manifest". Generated from a description means the
//! coverage is a **measurement** — it changes when the engine does — and
//! a measurement that is not gated drifts.
//!
//! The classification matters more than the count. A function this
//! adapter refuses to expose is not the same as one it has not learned to
//! marshal, and neither is a defect: the first is a boundary and the
//! second is a queue.

mod classify;
mod dispatch;
mod idl;
mod page;

use std::path::Path;

pub(crate) use classify::{Described, describe};
pub(crate) use dispatch::reached;
pub(crate) use idl::{Function, Shape, Vocabulary};

use crate::generated::{Output, check, write};
use classify::{Standing, standing};
use idl::Idl;

/// The engine's own description, vendored beside the adapter.
///
/// Vendored rather than read from a sibling checkout: the pass has to run
/// in CI, where no engine is installed, and a coverage figure that could
/// only be produced on one machine is not a gate. Refreshing it is a copy
/// and a regeneration, which is a deliberate act with a diff.
pub(crate) const IDL: &str = "adapters/ephemeris-teimeris/rust/data/teimeris.idl";

pub(crate) const PAGE: &str = "docs/03-design/engine-passthrough-measured.md";

/// The generated marshalling, beside the adapter that uses it.
pub(crate) const DISPATCH: &str = "adapters/ephemeris-teimeris/rust/src/dispatch.rs";

/// The page and the marshalling, from one reading of the description, so
/// a figure on the page and the code that answers it cannot disagree.
pub(crate) fn outputs(root: &Path) -> Result<Vec<Output>, String> {
    let text = std::fs::read_to_string(root.join(IDL))
        .map_err(|error| format!("{IDL} is not readable: {error}"))?;
    let idl: Idl =
        serde_json::from_str(&text).map_err(|error| format!("{IDL} does not parse: {error}"))?;
    let vocabulary = Vocabulary::new(&idl);
    let classified: Vec<(&Function, Standing, &'static str)> = idl
        .functions
        .iter()
        .map(|function| {
            let (standing, why) = standing(function, &vocabulary);
            (function, standing, why)
        })
        .collect();
    let callable: Vec<&Function> = classified
        .iter()
        .filter(|(_, standing, _)| *standing == Standing::Callable)
        .map(|(function, _, _)| *function)
        .collect();
    let mut outputs = vec![
        Output::new(PAGE, page::page(&idl, &classified, &vocabulary)),
        Output::new(DISPATCH, dispatch::dispatch(&idl, &classified, &vocabulary)),
    ];
    // The typed façades, from the same reading: a façade that typed an
    // argument the dispatch would refuse by name is the one thing
    // generating both from one description makes impossible (ADR-0030).
    outputs.extend(crate::facade::outputs(&idl.version, &callable, &vocabulary));
    Ok(outputs)
}

/// Runs the classification and writes both.
pub(crate) fn generate(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => write(root, &outputs),
        Err(message) => {
            eprintln!("{message}");
            1
        }
    }
}

/// Regenerates both in memory and fails on any difference.
pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => check(root, &outputs, "cargo xtask engine"),
        Err(message) => {
            eprintln!("{message}");
            1
        }
    }
}
