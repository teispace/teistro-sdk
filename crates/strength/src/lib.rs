//! Strength measures (Phase 5; `docs/03-design/strength-schemes.md`).
//!
//! - [`ashtakavarga`]: each graha's bindus by sign, the sarvashtakavarga,
//!   and their reductions and pindas under BPHS's reading or the conformance
//!   corpus's engine's (`docs/03-design/ashtakavarga-measured.md`).
//!
//! The bala schemes (shadbala, vimshopaka, bhava bala) are the next rows of
//! this crate.
//!
//! ```
//! use teistro_core::catalogue::Rashi;
//! use teistro_core::settings::{Ekadhipatya, Shodhana};
//! use teistro_strength::ashtakavarga::{AshtakavargaChart, AshtakavargaReading, AshtakavargaRules};
//!
//! let chart = AshtakavargaChart {
//!     lagna: Rashi::Pisces,
//!     signs: [Rashi::Aries, Rashi::Scorpio, Rashi::Aquarius, Rashi::Aries, Rashi::Gemini, Rashi::Aquarius, Rashi::Capricorn],
//! };
//! let rules = AshtakavargaRules { shodhana: Shodhana::EachGraha, ekadhipatya: Ekadhipatya::Bphs };
//! let reading = AshtakavargaReading::of(&chart, rules);
//! // Every chart holds the classical 337 bindus.
//! assert_eq!(reading.sarva.iter().map(|b| u32::from(*b)).sum::<u32>(), 337);
//! ```

pub mod ashtakavarga;

pub use ashtakavarga::{
    AshtakavargaChart, AshtakavargaReading, AshtakavargaRules, GrahaAshtakavarga,
};
