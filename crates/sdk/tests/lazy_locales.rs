//! The embedded locales are decoded the first time a context reads them,
//! and only those (`03-design/intl-engine-and-packs.md` §8, "Embedded
//! locales"): a context that renders in Nepali decodes Nepali and its
//! English fallback, and nothing else.
//!
//! Decoding is process-wide, so this file is one test in its own process:
//! another test building contexts beside it would decode what it reads.

#![allow(
    clippy::unwrap_used,
    clippy::print_stdout,
    reason = "tests fail by panicking, and report what they measured"
)]

use teistro::{Context, decoded_locales};
use teistro_intl::source::{Entry, Tree};
use teistro_intl::{Params, pack, sdk_root};
use teistro_test_allocator::{Counting, measure};

#[global_allocator]
static ALLOCATOR: Counting = Counting::system();

/// The first context in a process decoded all five bundles before this
/// budget: 59,010 allocations and 6.3 MB. It now verifies them and decodes
/// none: 434 allocations and 52 KB (both measured 2026-11-11, macOS).
const FIRST_CONTEXT_ALLOCATIONS: u64 = 1_000;

fn said(sdk: &Context) -> String {
    sdk.intl().render("sdk.reason.appName", &Params::new()).text
}

#[test]
fn a_context_decodes_the_locales_it_reads_and_no_others() {
    let (sdk, first) = measure(|| Context::builder().locale("ne").build().unwrap());
    println!(
        "the first context: {} allocations, {} bytes, peak {}",
        first.allocations, first.bytes, first.peak
    );
    assert_eq!(
        decoded_locales(),
        Vec::<String>::new(),
        "choosing a locale reads tags"
    );
    assert!(
        first.allocations <= FIRST_CONTEXT_ALLOCATIONS,
        "the first context allocated {} times against a budget of {FIRST_CONTEXT_ALLOCATIONS}: \
         is a bundle decoded before it is read?",
        first.allocations
    );

    let (nepali, render) = measure(|| said(&sdk));
    println!(
        "its first Nepali render: {} allocations, {} bytes",
        render.allocations, render.bytes
    );
    assert_eq!(decoded_locales(), ["en-Latn", "ne-Deva-NP"]);

    // A fork, a shared context and a second context read what is decoded.
    let fork = sdk.fork();
    let shared = sdk.share().context();
    let second = Context::builder().locale("ne-Deva-NP").build().unwrap();
    for other in [&fork, &shared, &second] {
        assert_eq!(said(other), nepali);
    }
    assert_eq!(decoded_locales(), ["en-Latn", "ne-Deva-NP"]);

    // Another locale is decoded when a context first reads it.
    fork.intl().set_locale("hi").unwrap();
    assert_eq!(decoded_locales(), ["en-Latn", "ne-Deva-NP"]);
    said(&fork);
    assert_eq!(decoded_locales(), ["en-Latn", "hi-Deva-IN", "ne-Deva-NP"]);

    // A runtime pack still lays itself over an embedded locale, for the
    // context that loaded it alone.
    let tree = Tree::load(&sdk_root()).unwrap();
    let mut patched = tree.locales.get("ne-Deva-NP").unwrap().clone();
    patched.namespaces.retain(|name, _| name == "sdk.reason");
    patched.namespaces.get_mut("sdk.reason").unwrap().insert(
        String::from("appName"),
        Entry::Message(String::from("मेरो पात्र")),
    );
    sdk.intl()
        .load_pack(&pack::build(&patched, "sdk.reason").unwrap())
        .unwrap();
    assert_eq!(said(&sdk), "मेरो पात्र");
    assert_eq!(said(&second), nepali, "another context keeps the build's");
    assert_eq!(
        sdk.intl().entity("graha.SUN").unwrap().name(),
        "सूर्य",
        "the rest of the locale stands under the pack"
    );

    // Every embedded bundle decodes: the gate on the bytes the build
    // embedded, which the first context only verified.
    let tags: Vec<String> = ["en-Latn", "hi-Deva-IN", "ne-Deva-NP", "sa-Deva", "sa-Latn"]
        .map(String::from)
        .into();
    for tag in &tags {
        second.intl().set_locale(tag).unwrap();
        said(&second);
    }
    assert_eq!(decoded_locales(), tags);
}
