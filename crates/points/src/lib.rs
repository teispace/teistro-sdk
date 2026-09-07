//! Points that are not bodies but behave like them.
//!
//! A derived point has a longitude, falls in a sign, is held by a house
//! and is reached by a drishti, so everything above the chart reads it
//! the way it reads a graha. Three families ship:
//!
//! - the **upagrahas the Sun casts** ([`solar`]) — Dhuma, Vyatipata,
//!   Parivesha, Indrachapa, Upaketu — a chain of five, each defined
//!   from the one before it;
//! - the **upagrahas the day divides** ([`eighth`]) — Gulika where
//!   Saturn's eighth of the arc begins and Mandi where it ends;
//! - the **special lagnas** ([`lagna`]) — the hora, ghati and pranapada
//!   lagnas that the clock drives, the Sree lagna that the Moon's
//!   nakshatra drives, and the Yogi and Avayogi points.
//!
//! The corpus decides all of it: every input is recorded beside every
//! answer, so a formula reproduces a recorded value or it does not
//! (`03-design/points-measured.md`, over `03-design/derived-points.md`).
//! Six of the eight rules are **exact**, to the last bit of a double.
//!
//! Four things worth knowing:
//!
//! - **The five the Sun casts are a chain**, not five offsets. Two of
//!   its steps are reflections, so two of the five run backwards as the
//!   Sun advances.
//! - **The time-driven lagnas start at the Sun *at birth***, not at
//!   sunrise, which most statements of the rule say. Using sunrise is
//!   out by half a degree.
//! - **Gulika begins Saturn's eighth and Mandi ends it** — start
//!   against end, not start against middle, derived from the corpus
//!   rather than proposed.
//! - **The Varnada does not ship.** No reading of the received rule
//!   reproduces the engine's, which is crux C22 met in the data.
//!
//! ```
//! use teistro_points::lagna::{avayogi, yogi};
//! use teistro_points::solar::chain;
//! use teistro_core::catalogue::Point;
//!
//! // The chain begins 133°20′ ahead of the Sun.
//! let cast = chain(0.0).expect("a finite longitude");
//! assert_eq!(cast[0].point, Point::Dhuma);
//! // The Yogi is the two luminaries together and seven nakshatras on;
//! // the Avayogi fourteen past that.
//! let point = yogi(10.0, 20.0).expect("finite");
//! let other = avayogi(10.0, 20.0).expect("finite");
//! let apart = (other.longitude_deg - point.longitude_deg).rem_euclid(360.0);
//! assert!((apart - (186.0 + 40.0 / 60.0)).abs() < 1e-9);
//! ```

pub mod chart;
pub mod derived;
pub mod eighth;
pub mod lagna;
pub mod solar;

pub use chart::Points;
pub use derived::Derived;
pub use eighth::{Ascendant, Portion};
