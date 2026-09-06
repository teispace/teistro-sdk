//! The intl generator and its gate: `i18n/` in, the SDK's typed messages
//! (`crates/intl/src/messages.rs`) out. `gen intl` validates the sources
//! and writes the accessors; `check-intl` validates and regenerates in
//! memory, failing on any validation error or any difference, so the
//! checked-in surface can never drift from the sources.

use std::path::Path;

use teistro_intl::derive::{derive, overrides_of};
use teistro_intl::generate::{Model, RustPaths, dart, javascript, rust, typescript_declarations};
use teistro_intl::source::Tree;
use teistro_intl::validate;

use crate::generated::{Output, check, write};

const MESSAGES: &str = "crates/intl/src/messages.rs";
/// The typed accessors each binding ships: the same model, in the shape
/// its language reads (`03-design/intl-engine-and-packs.md`).
const NODE_MESSAGES: &str = "bindings/node/lib/messages.js";
const NODE_MESSAGE_TYPES: &str = "bindings/node/lib/messages.d.ts";
const DART_MESSAGES: &str = "bindings/dart/lib/src/messages.dart";
/// The locale derived from another by transliteration, and the one it is
/// derived from (`03-design/intl-engine-and-packs.md`, §3).
const DERIVED: (&str, &str) = ("sa-Deva", "sa-Latn");

fn outputs(root: &Path) -> Vec<Output> {
    let tree = Tree::load(&root.join("i18n")).expect("the i18n/ sources load");
    let report = validate::validate(&tree);
    if report.passed() {
        eprintln!("{}", report.markdown().lines().next().unwrap_or_default());
    } else {
        eprintln!("{}", report.markdown());
        panic!("the i18n/ sources do not validate");
    }
    let base = tree.base().expect("the base locale");
    let model = Model::of(base).expect("every base message parses");
    let overrides = overrides_of(&root.join("i18n"), DERIVED.1)
        .unwrap_or_else(|e| panic!("the derived locale's overrides load: {e}"));
    let derived = derive(&tree, DERIVED.0, DERIVED.1, &overrides)
        .unwrap_or_else(|e| panic!("{} derives from {}: {e}", DERIVED.1, DERIVED.0));
    if !derived.stale.is_empty() {
        println!(
            "FAIL  i18n/{}/_overrides.json corrects entities the sources no longer have: {}",
            DERIVED.1,
            derived.stale.join(", ")
        );
    }
    let mut outputs: Vec<Output> = derived
        .files
        .iter()
        .map(|(path, text)| Output::new(format!("i18n/{}", path.display()), text.clone()))
        .collect();
    outputs.extend([
        Output::new(MESSAGES, rust(&model, RustPaths::SDK)),
        Output::new(NODE_MESSAGES, javascript(&model)),
        Output::new(NODE_MESSAGE_TYPES, typescript_declarations(&model)),
        Output::new(DART_MESSAGES, dart(&model)),
    ]);
    outputs
}

pub(crate) fn generate(root: &Path) -> i32 {
    write(root, &outputs(root))
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    let failures = check(root, &outputs(root), "cargo xtask gen intl");
    i32::from(failures != 0)
}
