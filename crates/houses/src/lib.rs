//! Which house, under which reading, and whether the chart can be
//! trusted.
//!
//! The geometry is elsewhere and stays there: `astro::houses` computes
//! twenty-two systems with their polar outcomes, `chart::bhava` makes
//! bhavas of any system's cusps, and both are measured against the
//! corpus. **This crate is not a second copy of either.** It is the
//! layer that answers what a caller asks and nothing else answered:
//!
//! - *Which system should this module use here?* ([`system`]) — a
//!   settings question, and `houses.module_overrides` is a knob nothing
//!   read until now.
//! - *Which house is this body in?* ([`chart`]) — under **both**
//!   readings, with the bodies that differ between them named.
//! - *Can I trust this chart?* — the degeneracy outcome, which the
//!   falsification pass found the recording engine's own boolean
//!   disagrees with in **both directions**.
//! - *What kind of house is it?* ([`classify`]) — kendra, trikona,
//!   dusthana, upachaya and the lord, which `strength` and `rules` both
//!   need and neither has.
//!
//! Three things worth knowing:
//!
//! - **A boolean cannot say what happened.** The engine flags charts at
//!   64° where the SDK computes Placidus, and leaves one clear at 69.6°
//!   where it cannot. The outcome says which of three things occurred,
//!   and that is what decides whether a chart is trustworthy
//!   (`03-design/houses-measured.md` §3, registry entry 26).
//! - **A house takes the sign its *middle* falls in.** Under an unequal
//!   division a house can begin in one sign and be centred in another.
//! - **A degenerate chart is not an error.** It is reported, and the
//!   caller decides — otherwise the polar policy would mean nothing.
//!
//! ```
//! use teistro_houses::classify::{Quadrant, is_dusthana, quadrant};
//!
//! // The first house is angular and a trine at once, which is why the
//! // tradition calls it the strongest there is.
//! assert_eq!(quadrant(1), Some(Quadrant::Kendra));
//! assert!(teistro_houses::classify::is_trikona(1));
//! // And the sixth is a house of difficulty that grows better with time.
//! assert!(is_dusthana(6) && teistro_houses::classify::is_upachaya(6));
//! ```

pub mod chart;
pub mod classify;
pub mod system;

pub use chart::{Bhava, Houses, Placed};
pub use classify::Quadrant;
pub use system::{Purpose, system_for};
