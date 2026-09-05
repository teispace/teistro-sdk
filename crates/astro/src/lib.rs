//! The astronomy layer of the Teistro SDK: everything above raw positions
//! (`docs/02-architecture/01-module-catalog.md`, `01-research/platform/13-astronomy-layer.md`).
//!
//! This seed holds [`solve`], the shared boundary solver: one root finder,
//! with a per-use tolerance and iteration caps, that every event search in
//! the SDK goes through (a sankranti, an ingress, a rise or set, a
//! station). The IAU routines, frame completion, the ayanamsha catalogue,
//! house systems and the rise and set solver arrive with the ephemeris
//! port's promotion.

pub mod solve;

pub use solve::{Caps, Crossing, SolveError, next_crossing};
