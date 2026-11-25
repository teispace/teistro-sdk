//! A build that leaves a text out (core#241): its bundles carry none of
//! the text's words, and a request for it is refused as `UNSUPPORTED`
//! naming the feature that keeps it. Loading a bundle that carries the
//! words makes them render again.
//!
//! Compiled only without `festival-about`; fast-check runs it in the
//! build that names no text (`--no-default-features --features
//! builtin-compact`).

#![cfg(not(feature = "festival-about"))]
#![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

use teistro::Status;
use teistro::{Context, PackRequest};
use teistro_intl::Params;
use teistro_intl::pack::build_bundle;
use teistro_intl::source::Tree;

#[test]
fn a_text_left_out_is_refused_by_name_and_carries_no_words() {
    let refused = PackRequest::from_json(r#"{"pack": "NEPAL", "about": true}"#).unwrap_err();
    assert_eq!(refused.status, Status::Unsupported);
    assert_eq!(refused.field(), Some("pack.about"));
    assert!(
        refused.message.contains("`festival-about`"),
        "{}",
        refused.message
    );
    // Without `about` the pack still lists.
    assert!(PackRequest::from_json(r#"{"pack": "NEPAL"}"#).is_ok());

    let sdk = Context::builder().build().unwrap();
    let observance = "sdk.entity.observance.KALASH_SET_UP";
    let silent = sdk.intl().render(observance, &Params::new());
    assert_eq!(silent.resolved_from, None, "{silent:?}");
    // The base locale's whole bundle, as the release ships it, words
    // the text again.
    let tree = Tree::sdk().unwrap();
    let english = build_bundle(tree.base().unwrap()).unwrap();
    sdk.intl().load_pack(&english).unwrap();
    let said = sdk.intl().render(observance, &Params::new());
    assert_eq!(said.resolved_from.as_deref(), Some("en-Latn"), "{said:?}");
}
