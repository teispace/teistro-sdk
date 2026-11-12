//! The packs the tests read, and the provider over them.
#![allow(
    clippy::print_stderr,
    clippy::panic,
    dead_code,
    reason = "a test without packs says so loudly where the runner shows it, a test fails by panicking, and each test file uses part of this module"
)]

use std::path::PathBuf;

use teistro_geo_teigeodata::TeigeodataGeo;

/// Names the pack files the tests open, separated as the platform
/// separates a `PATH`. Unset, every test that needs a pack skips, saying
/// so; the adapter's gate job sets it (`../README.md`).
pub(crate) const PACKS_VARIABLE: &str = "TEISTRO_TEIGEODATA_PACKS";

/// The pack files [`PACKS_VARIABLE`] names, or `None`, said on stderr,
/// when it is unset.
pub(crate) fn packs(test: &str) -> Option<Vec<PathBuf>> {
    if let Some(list) = std::env::var_os(PACKS_VARIABLE) {
        return Some(std::env::split_paths(&list).collect());
    }
    eprintln!(
        "SKIP  {test}: {PACKS_VARIABLE} names no teigeodata pack (Nepal's country pack, \
         `np-places-lite.tgd` from `forge make np --release 2026.11.0 --lite`)"
    );
    None
}

/// The provider over [`packs`], `None` when the variable is unset, or a
/// failure when a named pack is missing or does not open.
pub(crate) fn provider(test: &str) -> Option<TeigeodataGeo> {
    let packs = packs(test)?;
    for pack in &packs {
        assert!(
            pack.is_file(),
            "{} is named by {PACKS_VARIABLE} and is not there",
            pack.display()
        );
    }
    match TeigeodataGeo::open(&packs) {
        Ok(provider) => Some(provider),
        Err(why) => panic!("the packs did not open: {why:?}"),
    }
}
