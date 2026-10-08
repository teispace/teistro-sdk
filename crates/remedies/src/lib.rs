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
    AmatyaDevata, AmatyaDevatas, Deity, DevataRules, Devotion, IshtaDevata, IshtaDevatas,
    SunWithKetu, amatya_devata, baseline_ishta_devata, ishta_devata,
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

/// The readings a chart's remedies are given under, each group the texts'
/// own when left out.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RemedyRules {
    /// The functional natures (step 1).
    pub functional: FunctionalRules,
    /// The graha-śāntis (step 2).
    pub shanti: ShantiRules,
    /// The ishṭa-devatā (step 4).
    pub devata: DevataRules,
}

/// A chart's remedies, every step read off it: the lagna's functional
/// natures, whom a remedy is for and why, what the texts prescribe for
/// each of them, the running antardaśā's śānti and the ishṭa-devatā.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Remedies {
    /// The readings it was given under.
    pub rules: RemedyRules,
    /// The lagna's functional natures.
    pub functional: Functional,
    /// Whom a remedy is for, and the running antardaśā's śānti when a
    /// daśā was read.
    pub subjects: Subjects,
    /// The graha-śānti of each subject, in their order.
    pub shantis: Vec<Shanti>,
    /// The ishṭa-devatā, in both charts.
    pub ishta_devata: IshtaDevatas,
}
