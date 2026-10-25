//! The Teistro SDK's built-in analytic ephemeris.
//!
//! The SDK must compute a correct chart with nothing but the SDK
//! installed (ADR-0008). That means an ephemeris that lives inside it,
//! needs no files, no network and no licence beyond the SDK's own, and
//! **publishes its accuracy as a measurement rather than a claim**.
//!
//! What is here so far is the series arithmetic every tier shares: a
//! periodic term, and the sum that turns a body's terms into a
//! coordinate. The tables themselves are generated, and the thresholds
//! that produce them are not yet chosen — the curve they will be chosen
//! from is measured in
//! `docs/03-design/builtin-ephemeris-measured.md`.
//!
//! # Reading the published files
//!
//! Parsing a VSOP87 source file is the **generator's** job and is behind
//! the `ingest` feature, off by default. A consumer receives truncated
//! tables; it has no use for a parser of a 5.7 MB text file, and paying
//! for one in a wasm binary would be the opposite of the point.

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod chebyshev;
pub mod elp;
pub mod fitted;
#[cfg(any(feature = "compact", feature = "standard", feature = "full"))]
#[rustfmt::skip]
pub mod pluto;
#[cfg(any(feature = "compact", feature = "standard", feature = "full"))]
pub mod provider;
pub mod series;
#[cfg(any(feature = "compact", feature = "standard", feature = "full"))]
#[rustfmt::skip]
pub mod tables;

// Said by name rather than left to a missing item: a dependent that turns
// on the built-in and names no tier gets this sentence alone. The modules
// that read a tier's data are left out above, so no error about `PLUTO` or
// `TIER` follows it and buries it.
#[cfg(not(any(feature = "compact", feature = "standard", feature = "full")))]
compile_error!(
    "teistro-ephemeris-builtin needs a tier: enable `compact`, `standard` or `full` (on the façade and the boundary, `builtin-compact`, `builtin-standard` or `builtin-full`)"
);

#[cfg(feature = "ingest")]
pub mod ingest;
