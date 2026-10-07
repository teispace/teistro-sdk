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
//! The second is **graha-śānti**: what BPHS ch. 84 and *Yājñavalkya*
//! I.295–307 prescribe for each graha (the image, the ṛk, the japa, the
//! samidh, the food and the fee), with the gem *Jataka Parijata* II.21
//! gives it and the direction and substance of *Brihat Jataka* II.5 and
//! II.12, each value the verse's own word ([`shanti`]). With it come
//! the 81 antardaśā śāntis of BPHS chs. 37 to 45, each condition the
//! predicate the verse states ([`dasha_shanti`]).
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

mod dasha;
mod devata;
mod functional;
mod shanti;
mod subjects;

#[cfg(test)]
mod dasha_tests;
#[cfg(test)]
mod devata_tests;
#[cfg(test)]
mod shanti_tests;
#[cfg(test)]
mod subjects_tests;
#[cfg(test)]
mod tests;

pub use dasha::{Condition, DashaShanti, Remedy, dasha_shanti, dasha_shantis};
pub use devata::{
    Deity, DevataRules, Devotion, IshtaDevata, SunWithKetu, baseline_ishta_devata, ishta_devata,
};
pub use functional::{
    Badhaka, Clause, ClauseKind, Functional, FunctionalRow, FunctionalRules, Nature, Scheme,
    functional,
};
pub use shanti::{
    Dakshina, Food, Gem, ImageMaterial, MandalaPlace, OFFERINGS, RikSource, Samidh, Shanti,
    ShantiRules, Substance, shanti,
};
pub use subjects::{
    AntardashaShanti, NINE, Reason, RemedySky, Running, Subject, Subjects, holds, subjects,
};
