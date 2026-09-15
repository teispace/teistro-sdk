//! Strength measures (Phase 5; `docs/03-design/strength-schemes.md`).
//!
//! - [`ashtakavarga`]: each graha's bindus by sign, the sarvashtakavarga,
//!   and their reductions and pindas under BPHS's reading or the conformance
//!   corpus's engine's (`docs/03-design/ashtakavarga-measured.md`).
//! - [`vimshopaka`]: each graha's strength out of 20 across the divisional
//!   charts under the four schemes, by BPHS's points or the corpus's
//!   engine's virupas (`docs/03-design/vimshopaka-measured.md`).
//!
//! - [`shadbala`]: each graha's six strengths in virupas, under BPHS ch. 27's
//!   reading, Sripati's as B.V. Raman works it, or the corpus's engine's
//!   (`docs/03-design/shadbala-measured.md`).
//!
//! - [`bhava_bala`]: each house's strength from its lord, its direction and
//!   the drishtis it receives, under BPHS's, Sripati's or the engine's reading
//!   (`docs/03-design/bhava-bala-measured.md`).
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
pub mod bhava_bala;
pub mod shadbala;
pub mod vimshopaka;

pub use ashtakavarga::{
    AshtakavargaChart, AshtakavargaReading, AshtakavargaRules, GrahaAshtakavarga,
};
pub use bhava_bala::{BhavaBalaChart, BhavaBalaReading, BhavaBalaRules, BhavaGraha, BhavaStrength};
pub use shadbala::{
    GrahaShadbala, KaalaBala, ShadbalaChart, ShadbalaGraha, ShadbalaReading, ShadbalaRules,
    SthanaBala,
};
pub use vimshopaka::{GrahaVimshopaka, VimshopakaChart, VimshopakaReading};
