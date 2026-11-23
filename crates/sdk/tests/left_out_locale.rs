//! A build that leaves one of the SDK's locales out (SR-150): it embeds
//! `en-Latn` and the locales whose `locale-` features it names, refuses
//! the rest by name, saying which feature embeds one and which release
//! bundle loads it, and answers in one once that bundle is loaded.
//!
//! Compiled only without `locale-ne-deva-np`; fast-check runs it in the
//! build that names no locale (`--no-default-features --features
//! builtin-compact`).

#![cfg(not(feature = "locale-ne-deva-np"))]
#![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

use teistro::Context;
use teistro_intl::Params;
use teistro_intl::pack::build_bundle;
use teistro_intl::source::Tree;

const NEPALI: &str = "ne-Deva-NP";

#[test]
fn a_locale_left_out_is_refused_by_name_and_answers_once_its_bundle_is_loaded() {
    let refused = Context::builder().locale(NEPALI).build().unwrap_err();
    let hint = refused.hint().unwrap_or_default();
    assert!(hint.contains("`locale-ne-deva-np`"), "{hint}");
    assert!(
        hint.contains(&format!(
            "teistro-intl-{}-{NEPALI}.tbundle",
            env!("CARGO_PKG_VERSION")
        )),
        "{hint}"
    );

    let sdk = Context::builder().build().unwrap();
    let english = sdk.intl().render("sdk.reason.appName", &Params::new()).text;
    // The release's bundle for the locale: the same bytes `teistro-intl`
    // builds from its own sources.
    let tree = Tree::sdk().unwrap();
    let bundle = build_bundle(&tree.locales[NEPALI]).unwrap();
    sdk.intl().load_pack(&bundle).unwrap();
    assert_eq!(sdk.intl().set_locale(NEPALI).unwrap(), NEPALI);
    let nepali = sdk.intl().render("sdk.reason.appName", &Params::new());
    assert_eq!(nepali.resolved_from.as_deref(), Some(NEPALI), "{nepali:?}");
    assert_ne!(nepali.text, english);
}
