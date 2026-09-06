//! What a render allocates: bounded, and less the second time
//! (`05-testing/01-quality-bar.md`, "allocation counts").
//!
//! A render must allocate — it builds text — so the budget here is not
//! zero. What it pins is the shape: the message is parsed once and cached,
//! so rendering the same key again does not parse it again, and the count
//! does not grow with the number of renders.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    reason = "tests fail by panicking, and report what they measured"
)]

use teistro_intl::source::Tree;
use teistro_intl::{Intl, Value, params, sdk_root};
use teistro_test_allocator::{Counting, measure};

#[global_allocator]
static ALLOCATOR: Counting = Counting::system();

fn engine(locale: &str) -> Intl {
    let tree = Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"));
    let mut intl = Intl::from_tree(&tree).unwrap_or_else(|e| panic!("{e}"));
    intl.set_locale(locale).unwrap_or_else(|e| panic!("{e}"));
    intl
}

#[test]
fn a_message_is_parsed_once_and_rendered_many_times() {
    let intl = engine("ne-Deva-NP");
    let key = "sdk.reason.grahaInBhava";
    let arguments = || {
        params([
            ("graha", Value::entity("graha.JUPITER")),
            ("bhava", Value::Int(7)),
        ])
    };

    let (first, parsed) = measure(|| intl.render(key, &arguments()));
    let (second, cached) = measure(|| intl.render(key, &arguments()));
    assert_eq!(first.text, second.text, "the same message twice");
    assert!(first.warnings.is_empty(), "{:?}", first.warnings);
    println!(
        "parsed: {} allocations, {} bytes; cached: {} allocations, {} bytes",
        parsed.allocations, parsed.bytes, cached.allocations, cached.bytes
    );
    assert!(
        cached.allocations < parsed.allocations,
        "the second render allocated {} times against the first's {}: the parse is cached",
        cached.allocations,
        parsed.allocations
    );
    assert!(
        cached.allocations <= 64,
        "a cached render allocates {} times; the message's own parts and its text",
        cached.allocations
    );

    // Ten more renders cost ten times a cached one, not ten times a parse.
    let ((), ten) = measure(|| {
        for _ in 0..10 {
            let _ = intl.render(key, &arguments());
        }
    });
    assert!(
        ten.allocations <= cached.allocations * 12,
        "ten renders allocated {} times against a single cached render's {}",
        ten.allocations,
        cached.allocations
    );
}
