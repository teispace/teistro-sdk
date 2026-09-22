//! A consumer's own dasha system, of either kernel
//! (`03-design/dasha-kernels.md`, "A consumer's own system").
//!
//! The two kernels take different rows — [`UduDefinition`] is lords, years
//! and a nakshatra; [`RashiDefinition`] is where a system starts, the order
//! it visits the signs in and how long a sign runs — so a registry over one
//! of them can only take half the family. This is the other half's other
//! arm, and it is one type rather than two registries because a consumer
//! registers *a dasha system* and should not have to know which kernel the
//! SDK files it under.
//!
//! The kernel is **stated** in the JSON, under `kernel`, rather than
//! inferred from which fields are present. Two rows that differ by a typo
//! would otherwise be read as the other kernel and refused by a field the
//! caller never wrote, which is the error message a consumer cannot act on.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_core::quantity::Depth;

use crate::rashi::RashiDefinition;
use crate::row::UduDefinition;

/// A consumer's system, named by the kernel that runs it.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_dasha::{DashaDefinition, RashiDefinition, UduDefinition};
///
/// let udu: DashaDefinition = UduDefinition::of("ACME_TRAYA", Nakshatra::Ashwini).into();
/// let rashi: DashaDefinition = RashiDefinition::of("ACME_STHIRA").into();
/// assert_eq!(udu.key(), "ACME_TRAYA");
/// assert_eq!(rashi.key(), "ACME_STHIRA");
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kernel", rename_all = "snake_case")]
pub enum DashaDefinition {
    /// The nakshatra-seeded kernel: lords for years, seeded by the Moon's
    /// nakshatra.
    Udu(UduDefinition),
    /// The sign-based kernel: the twelve signs in an order, each for a
    /// number of years.
    Rashi(RashiDefinition),
}

impl DashaDefinition {
    /// The key it is registered under.
    #[must_use]
    pub fn key(&self) -> &str {
        match self {
            DashaDefinition::Udu(definition) => &definition.key,
            DashaDefinition::Rashi(definition) => &definition.key,
        }
    }

    /// How many levels of periods a reading of it carries.
    #[must_use]
    pub const fn depth(&self) -> Depth {
        match self {
            DashaDefinition::Udu(definition) => definition.depth,
            DashaDefinition::Rashi(definition) => definition.depth,
        }
    }

    /// The length of its year, which is the one setting a row states for
    /// itself rather than taking from the context.
    #[must_use]
    pub const fn year_length(&self) -> teistro_core::settings::YearLength {
        match self {
            DashaDefinition::Udu(definition) => definition.year_length,
            DashaDefinition::Rashi(definition) => definition.year_length,
        }
    }

    /// The nakshatra-seeded row, when that is the kernel it names.
    #[must_use]
    pub const fn udu(&self) -> Option<&UduDefinition> {
        match self {
            DashaDefinition::Udu(definition) => Some(definition),
            DashaDefinition::Rashi(_) => None,
        }
    }

    /// The sign-based row, when that is the kernel it names.
    #[must_use]
    pub const fn rashi(&self) -> Option<&RashiDefinition> {
        match self {
            DashaDefinition::Rashi(definition) => Some(definition),
            DashaDefinition::Udu(_) => None,
        }
    }
}

impl From<UduDefinition> for DashaDefinition {
    fn from(definition: UduDefinition) -> DashaDefinition {
        DashaDefinition::Udu(definition)
    }
}

impl From<RashiDefinition> for DashaDefinition {
    fn from(definition: RashiDefinition) -> DashaDefinition {
        DashaDefinition::Rashi(definition)
    }
}

impl teistro_core::registry::Definition for DashaDefinition {
    fn key(&self) -> &str {
        DashaDefinition::key(self)
    }

    fn validate(&self) -> Result<(), Error> {
        match self {
            DashaDefinition::Udu(definition) => definition.row().validate(),
            DashaDefinition::Rashi(definition) => definition.row().validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "tests fail by panicking"
    )]

    use teistro_core::catalogue::Nakshatra;
    use teistro_core::registry::Definition as _;

    use super::*;
    use crate::rashi::Length;

    #[test]
    fn a_kernel_is_stated_and_not_guessed() {
        let rashi: DashaDefinition = RashiDefinition::of("ACME_STHIRA").into();
        let json = serde_json::to_string(&rashi).unwrap();
        assert!(json.contains(r#""kernel":"rashi""#), "{json}");
        assert_eq!(
            serde_json::from_str::<DashaDefinition>(&json).unwrap(),
            rashi
        );
    }

    #[test]
    fn a_definition_without_a_kernel_is_refused_by_that_field() {
        let why = serde_json::from_str::<DashaDefinition>(r#"{"key":"ACME_STHIRA"}"#)
            .expect_err("a kernel is stated");
        assert!(why.to_string().contains("kernel"), "{why}");
    }

    #[test]
    fn each_arm_answers_for_its_own_kernel_and_not_the_other() {
        let udu: DashaDefinition = UduDefinition::of("ACME_TRAYA", Nakshatra::Ashwini).into();
        let rashi: DashaDefinition = RashiDefinition::of("ACME_STHIRA").into();
        assert!(udu.udu().is_some() && udu.rashi().is_none());
        assert!(rashi.rashi().is_some() && rashi.udu().is_none());
    }

    #[test]
    fn a_row_is_refused_by_the_field_that_is_wrong() {
        let no_years = DashaDefinition::from(RashiDefinition {
            length: Length::Fixed(0),
            ..RashiDefinition::of("ACME_NOTHING")
        });
        assert_eq!(no_years.validate().unwrap_err().field(), Some("length"));

        let thirteenth = DashaDefinition::from(RashiDefinition {
            stronger_of: vec![1, 13],
            ..RashiDefinition::of("ACME_THIRTEEN")
        });
        assert_eq!(
            thirteenth.validate().unwrap_err().field(),
            Some("stronger_of[1]")
        );

        let alone = DashaDefinition::from(RashiDefinition {
            stronger_of: vec![1],
            ..RashiDefinition::of("ACME_ALONE")
        });
        assert_eq!(alone.validate().unwrap_err().field(), Some("stronger_of"));

        let twice = DashaDefinition::from(RashiDefinition {
            stronger_of: vec![1, 1],
            ..RashiDefinition::of("ACME_TWICE")
        });
        assert_eq!(twice.validate().unwrap_err().field(), Some("stronger_of"));
    }

    #[test]
    fn every_shipped_row_passes_the_checks_a_registered_one_must() {
        for row in crate::rashi::RASHI_ROWS {
            row.validate()
                .unwrap_or_else(|why| panic!("{:?}: {why}", row.system));
        }
    }
}
