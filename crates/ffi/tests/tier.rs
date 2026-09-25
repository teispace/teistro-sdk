//! The built-in the boundary carries is the tier its features asked for.
//!
//! Tiers are additive and the richest enabled wins, so the tier a build
//! computes with is the richest of the three features — and until
//! 2026-09-25 it was never `compact`, because the base feature every tier
//! turns on brought `standard` with it. The tier job in `verify.yml` runs
//! this at each tier; without it, that job proved the tests pass and not
//! that the tier was the one asked for.

#![cfg(feature = "builtin-ephemeris")]
#![allow(clippy::expect_used, reason = "a test fails by panicking")]

use teistro::{Context, Ephemeris};
use teistro_core::settings::Tier;

#[test]
fn the_built_in_is_the_richest_tier_asked_for() {
    let asked = if cfg!(feature = "builtin-full") {
        Tier::Full
    } else if cfg!(feature = "builtin-standard") {
        Tier::Standard
    } else {
        Tier::Compact
    };
    let context = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a context on the built-in");
    let tier = context
        .ephemeris()
        .expect("the built-in is the ephemeris")
        .capabilities()
        .identity
        .tier;
    assert_eq!(tier, Some(asked));
}
