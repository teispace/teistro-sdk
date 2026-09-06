//! The intl generator and its gate: `i18n/` in, the SDK's typed messages
//! (`crates/intl/src/messages.rs`) out. `gen intl` validates the sources
//! and writes the accessors; `check-intl` validates and regenerates in
//! memory, failing on any validation error or any difference, so the
//! checked-in surface can never drift from the sources.

use std::path::Path;

use teistro_intl::generate::{Model, RustPaths, rust};
use teistro_intl::source::Tree;
use teistro_intl::validate;

use crate::generated::{Output, check, write};

const MESSAGES: &str = "crates/intl/src/messages.rs";

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
    vec![Output {
        path: MESSAGES,
        text: rust(&model, RustPaths::SDK),
    }]
}

pub(crate) fn generate(root: &Path) -> i32 {
    write(root, &outputs(root))
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    let failures = check(root, &outputs(root), "cargo xtask gen intl");
    i32::from(failures != 0)
}
