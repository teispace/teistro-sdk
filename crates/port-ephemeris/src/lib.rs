//! The ephemeris port of the Teistro SDK
//! (`docs/03-design/ephemeris-port-and-adapters.md`).
//!
//! The one thing only an ephemeris can supply is the positions of bodies;
//! the port requires that and nothing else (ADR-0002). A provider declares
//! its identity, coverage, native frame and any native overrides (the
//! obliquity and nutation, Delta T, the ayanamsha, rise and set), and the
//! SDK's `astro` layer completes the frame the caller asked for from the
//! frame the provider returns, stamping which implementation did each step
//! under the profile's override policy (ADR-0013).
//!
//! - [`body`]: the bodies and the time scales;
//! - [`frame`]: the frame (centre, equinox, coordinates, zodiac,
//!   corrections) and its 32-bit packing;
//! - [`columns`]: the columnar response with a status and a source per
//!   cell;
//! - [`capabilities`]: identity with content hashes, coverage, overrides
//!   as a bit set, the obliquity record;
//! - [`error`]: a whole-batch failure with its C code and its form as the
//!   SDK's error;
//! - [`horizon`]: the horizon convention and the rise and set request;
//! - [`provider`]: the trait, the request, and the validation every
//!   request passes first;
//! - [`vtable`]: the same contract as a `#[repr(C)]` vtable, both ways;
//! - [`sefile`]: the `.se1` file family both licensed engines read;
//! - [`test_provider`]: the analytic provider the SDK is built and tested
//!   against with no engine present.
//!
//! ```
//! use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, PositionRequest, TestProvider, TimeScale};
//!
//! let provider = TestProvider::new();
//! let jds = [2_451_545.0];
//! let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
//! let columns = provider.positions(&request).expect("the canonical frame");
//! let sun = columns.at(0, 0).expect("one cell");
//! assert!(sun.is_ok());
//! assert_eq!(columns.frame, Frame::CANONICAL);
//! ```

pub mod body;
pub mod capabilities;
pub mod columns;
pub mod error;
pub mod frame;
pub mod horizon;
pub mod provider;
pub mod sefile;
pub mod test_provider;
pub mod vtable;

pub use body::{Body, TimeScale};
pub use capabilities::{Capabilities, DataHash, Identity, Obliquity, Overrides};
pub use columns::{Cell, CellStatus, EphemerisKind, PositionColumns, Source};
pub use error::ProviderError;
pub use frame::{Centre, Coordinates, Corrections, Equinox, Frame, Zodiac};
pub use horizon::{DiscPoint, Horizon, HorizonEventKind, HorizonRequest, Refraction};
pub use provider::{EphemerisProvider, PositionRequest, validate};
pub use test_provider::TestProvider;
pub use vtable::{Exported, ProviderVtable, VTABLE_ABI_VERSION, VtableProvider};
