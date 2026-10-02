//! Hellenistic and traditional Western doctrine: the terms of the signs and
//! a planet's essential dignities (`03-design/essential-dignities.md`).
//!
//! Everything here is a reading of tables, so there is no ephemeris, no
//! chart and no instant: a planet, a longitude in the chart's own zodiac and
//! the chart's sect are the whole input. Every table is transcribed from a
//! cited source and held to it in `03-design/terms-measured.md`.
//!
//! ```
//! use teistro_hellenistic::{DignityRules, Scores, Sect, Terms, essential_dignity};
//! use teistro_core::catalogue::Graha;
//!
//! // Venus at 5° Taurus by day: her house, her triplicity, her term.
//! let venus = essential_dignity(Graha::Venus, 35.0, Sect::Day, &DignityRules::LILLY)?;
//! assert!(venus.house && venus.triplicity && venus.term);
//! assert_eq!(venus.score(&Scores::LILLY), 10);
//!
//! // The same under the Chaldean terms, where Taurus opens with Venus too.
//! let chaldean = DignityRules::LILLY.with_terms(Terms::Chaldean);
//! assert!(essential_dignity(Graha::Venus, 35.0, Sect::Day, &chaldean)?.term);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

#![doc(html_no_source)]

/// The module name the accidental fortitudes' house division is overridden
/// under, `houses.module_overrides.hellenistic`; unset, Lilly's
/// Regiomontanus.
pub const MODULE: &str = "hellenistic";

mod accidental;
mod almuten;
mod dignity;
mod fortitude;
mod reading;
mod reception;
mod terms;

pub use accidental::{
    Accident, AccidentalRules, AccidentalScores, AccidentalSky, Partile, PlanetAccidents, Siege,
    accidental_dignities, house_of,
};
pub use almuten::{
    Almuten, AlmutenRules, Almutens, FortuneRule, PlaceReading, almuten_of, almuten_of_places,
    part_of_fortune,
};
pub use dignity::{
    CHALDEAN_ORDER, DignityRules, EssentialDignity, Scores, Sect, SectRule, Triplicities,
    essential_dignity, exaltation_degree, face_lord,
};
pub use fortitude::{FortitudeRequest, Fortitudes};
pub use reading::{ChartSky, Dignities, DignityRequest, PlanetDignity};
pub use reception::{DignityKind, Reception};
pub use terms::{TERM_LORDS, TERMS_PER_SIGN, Term, Terms, TermsTable};
