//! The emitters over the boundary's description with one thing changed
//! that the checked-in description does not have: a deprecated value, a
//! brand with no unit and no range. Each binding says what changed where
//! a reader of it looks.

#![allow(
    clippy::expect_used,
    reason = "a test fails on the first thing it cannot read"
)]

use std::path::Path;

use teistro_idl::emit::{dart, python, ts};
use teistro_idl::model::Api;

fn api() -> Api {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../idl/api.json");
    let text = std::fs::read_to_string(path).expect("idl/api.json");
    serde_json::from_str(&text).expect("idl/api.json describes the boundary")
}

#[test]
fn a_deprecated_value_says_so_in_each_catalogue() {
    let mut api = api();
    let before = (python::catalogue(&api), dart::catalogue(&api));
    let value = api
        .enums
        .iter_mut()
        .find(|e| e.name == "Graha")
        .and_then(|e| e.values.first_mut())
        .expect("a graha");
    value.deprecated = true;
    let python = python::catalogue(&api);
    let dart = dart::catalogue(&api);
    assert_ne!(python, before.0);
    assert_ne!(dart, before.1);
    assert!(
        python.contains("\n\n    Deprecated.") && !before.0.contains("Deprecated."),
        "the docstring under the member"
    );
    assert!(
        dart.contains("Deprecated.") && !before.1.contains("Deprecated."),
        "the comment above the member"
    );
}

#[test]
fn a_brand_without_a_unit_or_a_range_is_named_alone() {
    let mut api = api();
    for field in api.structs.iter_mut().flat_map(|s| &mut s.fields) {
        if field.meta.brand.as_deref() == Some("latitude") {
            field.meta.unit = None;
            field.meta.range = None;
        }
    }
    let types = ts::catalogue_declarations(&api);
    assert!(types.contains(" * A latitude. Its own type"), "{types}");
    assert!(types.contains("/** A latitude, checked. */"), "{types}");
    let declarations = python::declarations(&api);
    assert!(
        declarations.contains("class Latitude(float):\n    \"\"\"A latitude.\n"),
        "no unit, no range and no check"
    );
    let dart = dart::declarations(&api);
    assert!(
        dart.contains("/// A latitude. Its own type, so it cannot be passed where"),
        "{dart}"
    );
}
