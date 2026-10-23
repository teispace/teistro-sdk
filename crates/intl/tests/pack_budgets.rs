//! Each locale's pack within its budget (`05-testing/01-quality-bar.md`,
//! "size"): the bundle `crates/sdk/build.rs` compiles from `i18n/` into
//! every artefact, against `05-testing/pack-budgets.json`. A bundle is
//! built from the sources alone, so its size is the same on every
//! machine and this test measures what ships.

#![allow(
    clippy::unwrap_used,
    clippy::print_stdout,
    reason = "tests fail by panicking, and report what they measured"
)]

use std::collections::BTreeMap;

use teistro_intl::pack::build_bundle;
use teistro_intl::source::Tree;

#[test]
fn every_locale_pack_is_within_its_budget() {
    let root = teistro_intl::sdk_root();
    let tree = Tree::load(&root).unwrap();
    let page = root
        .parent()
        .unwrap()
        .join("docs/05-testing/pack-budgets.json");
    let record: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&page).unwrap()).unwrap();
    let budgets: BTreeMap<String, u64> =
        serde_json::from_value(record.get("budgets").cloned().unwrap()).unwrap();

    let mut faults = Vec::new();
    for (tag, locale) in &tree.locales {
        let size = build_bundle(locale).unwrap().len() as u64;
        println!("{tag}: {size} bytes");
        match budgets.get(tag.as_str()) {
            None => faults.push(format!("{tag} ({size} bytes) has no budget")),
            Some(&budget) if size > budget => {
                faults.push(format!(
                    "{tag} is {size} bytes, over its budget of {budget}"
                ));
            }
            Some(&budget) if budget > size + size / 4 => faults.push(format!(
                "{tag}'s budget of {budget} is more than a quarter over its {size} bytes"
            )),
            Some(_) => {}
        }
    }
    for tag in budgets.keys() {
        if !tree.locales.keys().any(|shipped| shipped.as_str() == tag) {
            faults.push(format!("{tag} has a budget and no locale"));
        }
    }
    assert!(
        faults.is_empty(),
        "{} of `docs/05-testing/pack-budgets.json`:\n{}",
        faults.len(),
        faults.join("\n")
    );
}
