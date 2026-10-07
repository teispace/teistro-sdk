//! Remedies, from the texts that prescribe them (`03-design/remedies.md`).
//!
//! The first layer is **functional nature**: which graha the lordships of
//! a lagna make good or bad for it, read from *Laghu Parashari*
//! (*Uḍudāya-pradīpa*) vv. 6 to 25 and BPHS ch. 13 (the 1923 print). It
//! reads no sky, only the lagna's sign. A graha's nature is reported as the
//! clauses that hold for it, each with its verse, and the summary nature is
//! derived from them by the text's precedence, so a reader sees why as well
//! as what.
//!
//! ```
//! use teistro_core::catalogue::{Graha, Rashi};
//! use teistro_remedies::{FunctionalRules, Nature, functional};
//!
//! // Taurus: Saturn owns the 9th and the 10th, a trikona and a kendra, and
//! // is the yogakaraka (v. 20); Jupiter owns the 8th and the 11th (v. 6).
//! let taurus = functional(Rashi::Taurus, FunctionalRules::default());
//! assert_eq!(taurus.yogakarakas, [Graha::Saturn]);
//! assert_eq!(taurus.row(Graha::Jupiter).map(|row| row.nature), Some(Nature::Malefic));
//! ```

#![doc(html_no_source)]

mod functional;

#[cfg(test)]
mod tests;

pub use functional::{
    Badhaka, Clause, ClauseKind, Functional, FunctionalRow, FunctionalRules, Nature, Scheme,
    functional,
};
