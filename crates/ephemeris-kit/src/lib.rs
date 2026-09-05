//! The provider conformance kit of the Teistro SDK
//! (`docs/03-design/ephemeris-port-and-adapters.md`, §9): the checks every
//! adapter and every built-in tier must pass, with a machine-readable
//! report, under one published set of bounds; the timing rows; and the
//! runner the kit binaries share.
//!
//! ```
//! use teistro_ephemeris_kit::kit::{self, Bounds};
//! use teistro_port_ephemeris::TestProvider;
//!
//! let report = kit::run(&TestProvider::new(), &Bounds::DEFAULT);
//! assert!(report.passed, "{}", report.markdown());
//! ```

pub mod bench;
pub mod kit;
pub mod runner;

pub use kit::{Bounds, Check, Refusing, Report, Results, run};
