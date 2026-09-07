//! The chart foundation: what every module above a chart starts from.
//!
//! One moment at one place yields a handful of facts that vargas, state,
//! aspects, the panchanga, the strengths, the dashas and the rules all
//! need and none of them should compute twice: which day the moment
//! belongs to, where the lagna is, where the bhavas are, and where each
//! graha falls in them. Computing those once, stamping them and handing
//! them on is what this crate is for
//! (`03-design/chart-foundation.md`).
//!
//! Two things in it are easy to get wrong and are therefore the two
//! modules built first:
//!
//! - [`day`]: the day a chart belongs to is **not** its civil date. A
//!   panchanga day runs sunrise to sunrise, so an instant before the
//!   civil date's sunrise belongs to the day that began the morning
//!   before, and the vara, the hora, the ishtakaal and the lagna's anchor
//!   move back with it.
//! - [`bhava`]: a house placement is **not** a number. Which chalit put a
//!   graha in a bhava decides the answer between 10% and 51% of the time,
//!   so a placement carries its method, and the bhavas carry their madhya
//!   as well as their sandhi.
//!
//! The rule the crate is built on: **the foundation holds what is needed
//! to compute, never what is computed.** A field belongs here when more
//! than one module above needs it and no module above can produce it.

pub mod bhava;
pub mod day;

pub use bhava::{Bhavas, Chalit, Placement, Reading};
pub use day::{ChartDay, DayPart};
