//! Krishnamurti Paddhati: the lords of a longitude, and the horary
//! numbers (`03-design/kp.md`).
//!
//! The KP Readers divide each nakshatra among the nine Vimshottari lords
//! in proportion to their years, starting from the nakshatra's own lord,
//! and each of those **subs** again the same way, starting from the sub's
//! own lord. A longitude's lords are the sign's lord, the nakshatra's
//! (the **star** lord), the sub's, the sub-sub's, and so on down.
//!
//! Every boundary is a rational multiple of a nanoarcsecond, so the lords
//! are found in integers and never by comparing floating-point degrees: a
//! sub spans its lord's years × 6′40″ exactly, and each deeper level is
//! compared after scaling by 120 per level. A position on a boundary
//! belongs to the level it opens.
//!
//! ```
//! use teistro_core::angle::Nas;
//! use teistro_core::catalogue::Graha;
//! use teistro_kp::{KpNumber, lords};
//!
//! // Gemini 9°: Mercury's sign, Ardra (Rahu's star), Jupiter's sub, the
//! // Reader's horary number 48, which opens at Gemini 8°40′.
//! let at = Nas::new(69 * Nas::PER_DEGREE);
//! let lords = lords(at);
//! assert_eq!(
//!     (lords.sign, lords.star.lord, lords.sub.lord),
//!     (Graha::Mercury, Graha::Rahu, Graha::Jupiter),
//! );
//! // Twenty arcminutes into a sub of 1°46′40″.
//! assert_eq!(lords.sub.span.margin(at), 20 * Nas::PER_ARCMINUTE);
//! assert_eq!(KpNumber::of(at).get(), 48);
//! assert_eq!(KpNumber::new(48)?.start(), Nas::new(68 * Nas::PER_DEGREE + 40 * Nas::PER_ARCMINUTE));
//! # Ok::<(), teistro_core::Error>(())
//! ```

#![doc(html_no_source)]

/// The module name KP's house system is overridden under,
/// `houses.module_overrides.kp`: `PLACIDUS` in the `kp-default` profile.
pub const MODULE: &str = "kp";

mod chain;
mod chart;
mod number;
mod significators;

pub use chain::{Chain, Level, Lords, MAX_LEVELS, Span, chain, lords};
pub use chart::{Cusp, KpChart, Planet, Position, house_of};
pub use number::KpNumber;
pub use significators::{HouseSignificators, NodeAgency, Significators, Signified};

#[cfg(test)]
mod tests;
