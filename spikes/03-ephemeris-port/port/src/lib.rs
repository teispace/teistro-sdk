//! Spike 3: the ephemeris port.
//!
//! The one thing only an ephemeris can supply is the positions of bodies;
//! the port requires that and nothing else (ADR-0002). A provider declares
//! its identity, coverage, native frame and any native overrides
//! (obliquity, Delta T, ayanamsha here; more in the SDK), and the SDK's
//! `astro` layer completes the frame the caller asked for from the frame
//! the provider returns, stamping which implementation did each step.
//!
//! What this crate holds, each in its own module:
//!
//! - [`model`]: the request, the columnar response, frames, capabilities
//!   and errors, all serialisable so a report can quote them;
//! - [`provider`]: the trait itself;
//! - [`vtable`]: the same contract as a C vtable, so a native engine or a
//!   binding's host object is one shape to the SDK;
//! - [`astro`]: the IAU routines the completion needs, ported from ERFA
//!   with a provenance table (ADR-0021), and the coordinate rotation;
//! - [`completion`]: the frame completion with the override policy;
//! - [`kit`]: the provider conformance kit and its report;
//! - [`bench`]: one timing helper shared by every measurement in Rust;
//! - [`runner`]: the shared body of the kit binaries;
//! - [`sefile`]: the `.se1` file family both licensed engines read, for
//!   coverage and provenance hashes;
//! - [`test_provider`]: the spike-2 analytic provider behind this port.
//!
//! The adapters for Teimeris and Swiss Ephemeris live outside the
//! workspace (`../adapters/`), as ADR-0019 requires.

pub mod astro;
pub mod bench;
pub mod completion;
pub mod kit;
pub mod model;
pub mod provider;
pub mod runner;
pub mod sefile;
pub mod test_provider;
pub mod vtable;

pub use model::{
    AyanamshaId, Body, Capabilities, Cell, CellStatus, Center, Coordinates, Corrections, DataHash,
    EphemerisKind, Equinox, Frame, Identity, Obliquity, Observer, OverridePolicy, Overrides,
    PositionColumns, PositionRequest, ProviderError, Source, Tier, TimeScale, Zodiac,
};
pub use provider::EphemerisProvider;
