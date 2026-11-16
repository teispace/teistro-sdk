//! The rows a divisional chart is computed by, as a document and a settings
//! patch write them, and the check every row passes before a chart is
//! computed by it (`03-design/varga-schemes.md`).
//!
//! `teistro-vargas` evaluates a chart over one rule: sort the sign into a
//! group, take the part of the sign the longitude falls in, and either step
//! through the signs or read the answer off a list. Its twenty-one shipped
//! rows and its named variants are `'static` tables; the types here are the
//! same rule **owned**, which is what a consumer writes when the texts it
//! follows are not among the named schemes. They live in the core because
//! the settings carry them and refuse a bad row when they are resolved,
//! naming the field, before any chart is asked of them.
//!
//! ```
//! use teistro_core::varga_rows::{Classifier, Rows};
//!
//! // Kashinatha's hora as a consumer would write it: a list per sign.
//! let rows: Rows = serde_json::from_str(
//!     r#"{"classifier": "PARITY", "groups": [
//!         {"spans": {"spans": "EQUAL"}, "map": {"map": "LISTED", "signs": [4, 3]}},
//!         {"spans": {"spans": "EQUAL"}, "map": {"map": "LISTED", "signs": [3, 4]}}
//!     ]}"#,
//! ).unwrap();
//! assert_eq!(rows.classifier, Classifier::Parity);
//! assert!(rows.check(2, "vargas.schemes.D2").is_ok());
//! // Three signs for a sign cut in two is refused, by the field.
//! let error = rows.check(3, "vargas.schemes.D3").unwrap_err();
//! assert_eq!(error.field(), Some("vargas.schemes.D3.groups[0].map.signs"));
//! ```

use serde::{Deserialize, Serialize};

use crate::catalogue::{Rashi, Varga};
use crate::error::{Error, Status};

/// The most parts a chart may divide a sign into: past every attested chart
/// and past the point where a part is smaller than an ephemeris's own
/// uncertainty.
///
/// ```
/// use teistro_core::varga_rows::MOST_DIVISIONS;
///
/// assert_eq!(MOST_DIVISIONS, 300);
/// ```
pub const MOST_DIVISIONS: u16 = 300;

/// The signs of the zodiac.
const SIGNS: u8 = 12;

/// How a sign is sorted into groups, each with its own rule.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_core::varga_rows::Classifier;
///
/// // Taurus is even, fixed and earthy, and the second sign.
/// assert_eq!(Classifier::Parity.group_of(Rashi::Taurus), 1);
/// assert_eq!(Classifier::Modality.group_of(Rashi::Taurus), 1);
/// assert_eq!(Classifier::Element.group_of(Rashi::Taurus), 1);
/// assert_eq!(Classifier::Sign.group_of(Rashi::Pisces), 11);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Classifier {
    /// One group: every sign treated alike.
    All,
    /// Odd and even signs, counting Aries as odd.
    Parity,
    /// Movable, fixed and dual.
    Modality,
    /// Fire, earth, air and water.
    Element,
    /// Each sign its own group: a whole table, one list per sign, for a
    /// rule no arithmetic over the sign's number gives (Kashinatha's hora,
    /// Somanatha's drekkana).
    Sign,
}

impl Classifier {
    /// How many groups it sorts the twelve signs into.
    ///
    /// ```
    /// use teistro_core::varga_rows::Classifier;
    ///
    /// assert_eq!(Classifier::Element.groups(), 4);
    /// assert_eq!(Classifier::Sign.groups(), 12);
    /// ```
    #[must_use]
    pub const fn groups(self) -> u8 {
        match self {
            Classifier::All => 1,
            Classifier::Parity => 2,
            Classifier::Modality => 3,
            Classifier::Element => 4,
            Classifier::Sign => SIGNS,
        }
    }

    /// Which group a sign belongs to.
    ///
    /// Every one of the five is the sign's index modulo the group count,
    /// because the zodiac alternates parity, cycles the modalities in threes
    /// and the elements in fours from Aries, and has twelve signs. That is
    /// arithmetic rather than a coincidence, and it is why one classifier
    /// covers all five.
    ///
    /// ```
    /// use teistro_core::catalogue::Rashi;
    /// use teistro_core::varga_rows::Classifier;
    ///
    /// // Sagittarius is dual: the third modality.
    /// assert_eq!(Classifier::Modality.group_of(Rashi::Sagittarius), 2);
    /// assert_eq!(Classifier::All.group_of(Rashi::Sagittarius), 0);
    /// ```
    #[must_use]
    pub const fn group_of(self, sign: Rashi) -> usize {
        (sign as u8 % self.groups()) as usize
    }
}

/// The multiplier a stepping rule puts on the sign.
///
/// ```
/// use teistro_core::varga_rows::SignBase;
///
/// assert_eq!(SignBase::Same.multiplier(9), 1);
/// // The navamsha counts on round the zodiac: nine parts a sign.
/// assert_eq!(SignBase::Cyclic.multiplier(9), 9);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignBase {
    /// Count from a fixed sign, which the offset names: `a = 0`.
    Fixed,
    /// Count from the sign the body already stands in: `a = 1`.
    Same,
    /// Count continuously round the zodiac, the parivritti form:
    /// `a = divisions`.
    Cyclic,
}

impl SignBase {
    /// The multiplier for a chart of so many divisions.
    ///
    /// ```
    /// use teistro_core::varga_rows::SignBase;
    ///
    /// assert_eq!(SignBase::Fixed.multiplier(9), 0);
    /// ```
    #[must_use]
    pub const fn multiplier(self, divisions: u16) -> u32 {
        match self {
            SignBase::Fixed => 0,
            SignBase::Same => 1,
            SignBase::Cyclic => divisions as u32,
        }
    }
}

/// Where the parts of a sign begin, owned.
///
/// ```
/// use teistro_core::varga_rows::RowSpans;
///
/// let d30: RowSpans = serde_json::from_str(r#"{"spans": "DEGREES", "widths": [5, 5, 8, 7, 5]}"#).unwrap();
/// assert_eq!(d30, RowSpans::Degrees { widths: vec![5, 5, 8, 7, 5] });
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "spans", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum RowSpans {
    /// The chart's divisions, each of `30/N` degrees.
    Equal,
    /// Whole-degree widths, summing to thirty.
    Degrees {
        /// The parts' widths in degrees, in order.
        widths: Vec<u8>,
    },
}

/// How a part of a sign becomes a sign, owned.
///
/// ```
/// use teistro_core::varga_rows::{RowMap, SignBase};
///
/// let d9: RowMap = serde_json::from_str(r#"{"map": "STEP", "base": "CYCLIC", "step": 1, "offset": 0}"#).unwrap();
/// assert_eq!(d9, RowMap::Step { base: SignBase::Cyclic, step: 1, offset: 0 });
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "map", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum RowMap {
    /// Step through the signs: `(a·rashi + step·part + offset) mod 12`.
    Step {
        /// What the sign is multiplied by.
        base: SignBase,
        /// How many signs on each part is from the last, 0 to 11; eleven
        /// steps backwards.
        step: u8,
        /// Where the first part starts, counted from Aries, 0 to 11.
        offset: u8,
    },
    /// One sign per part, listed in order, 0 for Aries.
    Listed {
        /// The signs, a part each.
        signs: Vec<u8>,
    },
}

/// One classifier group's rule, owned.
///
/// ```
/// use teistro_core::varga_rows::{RowGroup, RowMap, RowSpans, SignBase};
///
/// let group = RowGroup { spans: RowSpans::Equal, map: RowMap::Step { base: SignBase::Same, step: 4, offset: 0 } };
/// assert_eq!(group.parts(3), 3);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct RowGroup {
    /// Where its parts begin.
    pub spans: RowSpans,
    /// Where each part sends a body.
    pub map: RowMap,
}

impl RowGroup {
    /// How many parts it cuts a sign into: the chart's divisions, or one a
    /// width.
    ///
    /// ```
    /// use teistro_core::varga_rows::{RowGroup, RowMap, RowSpans};
    ///
    /// let five = RowGroup { spans: RowSpans::Degrees { widths: vec![5, 5, 8, 7, 5] }, map: RowMap::Listed { signs: vec![0, 10, 8, 2, 6] } };
    /// assert_eq!(five.parts(30), 5);
    /// ```
    #[must_use]
    pub fn parts(&self, divisions: u16) -> usize {
        match &self.spans {
            RowSpans::Equal => usize::from(divisions),
            RowSpans::Degrees { widths } => widths.len(),
        }
    }
}

/// A divisional chart's rule, owned: how its signs are sorted and one rule
/// a group. What a consumer writes as `CUSTOM` in `vargas.schemes`, and the
/// rule a chart document writes beside the chart computed by it.
///
/// ```
/// use teistro_core::varga_rows::{Classifier, Rows};
///
/// let rows = Rows { classifier: Classifier::All, groups: vec![] };
/// // One group a classifier group, or the field is named.
/// assert_eq!(rows.check(9, "rows").unwrap_err().field(), Some("rows.groups"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Rows {
    /// How its signs are sorted into groups.
    pub classifier: Classifier,
    /// One rule per group, in group order.
    pub groups: Vec<RowGroup>,
}

impl Rows {
    /// Whether the rows describe a chart of `divisions` a computer can
    /// evaluate: one group a classifier group, spans that fill a sign in
    /// whole degrees, one listed sign a part, and every sign, step and
    /// offset in 0 to 11. The first fault is refused, naming the field
    /// under `field`.
    ///
    /// ```
    /// use teistro_core::varga_rows::{Classifier, RowGroup, RowMap, RowSpans, Rows, SignBase};
    ///
    /// let mut rows = Rows {
    ///     classifier: Classifier::All,
    ///     groups: vec![RowGroup { spans: RowSpans::Equal, map: RowMap::Step { base: SignBase::Cyclic, step: 1, offset: 0 } }],
    /// };
    /// assert!(rows.check(9, "vargas.schemes.D9").is_ok());
    /// rows.groups[0].map = RowMap::Step { base: SignBase::Cyclic, step: 12, offset: 0 };
    /// let error = rows.check(9, "vargas.schemes.D9").unwrap_err();
    /// assert_eq!(error.field(), Some("vargas.schemes.D9.groups[0].map.step"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a rule that cannot be evaluated, or `OUT_OF_RANGE`
    /// for divisions outside 1 to [`MOST_DIVISIONS`], each with the field.
    pub fn check(&self, divisions: u16, field: &str) -> Result<(), Error> {
        if divisions == 0 || divisions > MOST_DIVISIONS {
            return Err(Error::new(
                Status::OutOfRange,
                format!("a divisional chart divides a sign 1 to {MOST_DIVISIONS} times, not {divisions}"),
            )
            .with_field(format!("{field}.divisions")));
        }
        let wanted = usize::from(self.classifier.groups());
        if self.groups.len() != wanted {
            return Err(Error::invalid_arg(format!(
                "{:?} sorts the signs into {wanted} groups, and the rows give {}",
                self.classifier,
                self.groups.len()
            ))
            .with_field(format!("{field}.groups")));
        }
        for (index, group) in self.groups.iter().enumerate() {
            check_group(group, divisions, &format!("{field}.groups[{index}]"))?;
        }
        Ok(())
    }
}

/// One group's check, under `field`.
fn check_group(group: &RowGroup, divisions: u16, field: &str) -> Result<(), Error> {
    if let RowSpans::Degrees { widths } = &group.spans {
        let total: u32 = widths.iter().map(|width| u32::from(*width)).sum();
        if widths.is_empty() || widths.contains(&0) || total != 30 {
            return Err(Error::invalid_arg(format!(
                "spans are whole degrees, none nought, filling a sign's thirty; these sum to {total}"
            ))
            .with_field(format!("{field}.spans.widths")));
        }
    }
    let parts = group.parts(divisions);
    match &group.map {
        RowMap::Step { step, offset, .. } => {
            for (name, value) in [("step", step), ("offset", offset)] {
                if *value >= SIGNS {
                    return Err(Error::invalid_arg(format!(
                        "a {name} counts signs 0 to 11, not {value}"
                    ))
                    .with_field(format!("{field}.map.{name}")));
                }
            }
        }
        RowMap::Listed { signs } => {
            if signs.len() != parts {
                return Err(Error::invalid_arg(format!(
                    "a listed group names one sign a part: {parts} parts, {} signs",
                    signs.len()
                ))
                .with_field(format!("{field}.map.signs")));
            }
            if let Some((at, sign)) = signs.iter().enumerate().find(|(_, sign)| **sign >= SIGNS) {
                return Err(Error::invalid_arg(format!(
                    "a sign is 0 (Aries) to 11 (Pisces), not {sign}"
                ))
                .with_field(format!("{field}.map.signs[{at}]")));
            }
        }
    }
    Ok(())
}

/// Which way a divisional chart is computed, as the settings name it for one
/// catalogued chart (`vargas.schemes`): a named scheme of the texts, or a
/// consumer's own rows.
///
/// ```
/// use teistro_core::settings::VargaVariant;
/// use teistro_core::varga_rows::VargaChoice;
///
/// let named: VargaChoice = serde_json::from_str(r#"{"kind": "NAMED", "which": "KASHINATHA"}"#).unwrap();
/// assert_eq!(named, VargaChoice::Named { which: VargaVariant::Kashinatha });
/// let own: VargaChoice = serde_json::from_str(
///     r#"{"kind": "CUSTOM", "classifier": "ALL", "groups": [{"spans": {"spans": "EQUAL"}, "map": {"map": "STEP", "base": "SAME", "step": 4, "offset": 0}}]}"#,
/// ).unwrap();
/// assert!(matches!(own, VargaChoice::Custom { .. }));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum VargaChoice {
    /// A scheme the SDK ships, each cited (`VargaVariant`).
    Named {
        /// The scheme.
        which: crate::settings::VargaVariant,
    },
    /// A consumer's own rows, checked as the shipped ones are.
    Custom {
        /// How its signs are sorted into groups.
        classifier: Classifier,
        /// One rule per group, in group order.
        groups: Vec<RowGroup>,
    },
}

impl VargaChoice {
    /// The check a choice for `varga` passes when the settings resolve: a
    /// named scheme must be one of that chart's, and a consumer's rows must
    /// pass [`Rows::check`]. The field is `vargas.schemes.<chart>`.
    ///
    /// ```
    /// use teistro_core::catalogue::Varga;
    /// use teistro_core::settings::VargaVariant;
    /// use teistro_core::varga_rows::VargaChoice;
    ///
    /// assert!(VargaChoice::Named { which: VargaVariant::Kashinatha }.check(Varga::D2).is_ok());
    /// let error = VargaChoice::Named { which: VargaVariant::Kashinatha }.check(Varga::D9).unwrap_err();
    /// assert_eq!(error.field(), Some("vargas.schemes.D9.which"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the field, as above.
    pub fn check(&self, varga: Varga) -> Result<(), Error> {
        let field = format!("vargas.schemes.{}", varga.key());
        if varga == Varga::D1 && *self != (VargaChoice::Named { which: crate::settings::VargaVariant::Parashara }) {
            return Err(Error::invalid_arg(
                "D1 is the rashi chart itself, the sign a body stands in, and takes no scheme",
            )
            .with_field(field));
        }
        match self {
            VargaChoice::Named { which } => {
                if which.serves(varga) {
                    Ok(())
                } else {
                    Err(Error::invalid_arg(format!(
                        "{} is not a scheme of {}; it has {}",
                        which.key(),
                        varga.key(),
                        crate::settings::VargaVariant::ALL
                            .iter()
                            .filter(|variant| variant.serves(varga))
                            .map(|variant| variant.key())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .with_field(format!("{field}.which")))
                }
            }
            VargaChoice::Custom { classifier, groups } => Rows {
                classifier: *classifier,
                groups: groups.clone(),
            }
            .check(varga.attributes().divisions, &field),
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::{Classifier, RowGroup, RowMap, RowSpans, Rows, SignBase};

    fn rows(groups: Vec<RowGroup>) -> Rows {
        Rows {
            classifier: Classifier::Parity,
            groups,
        }
    }

    fn equal(map: RowMap) -> RowGroup {
        RowGroup {
            spans: RowSpans::Equal,
            map,
        }
    }

    #[test]
    fn every_fault_names_its_field() {
        let good = equal(RowMap::Listed { signs: vec![4, 3] });
        let cases = [
            (rows(vec![good.clone()]), 2, "r.groups"),
            (
                rows(vec![good.clone(), equal(RowMap::Listed { signs: vec![4, 12] })]),
                2,
                "r.groups[1].map.signs[1]",
            ),
            (
                rows(vec![
                    good.clone(),
                    equal(RowMap::Step {
                        base: SignBase::Same,
                        step: 1,
                        offset: 12,
                    }),
                ]),
                2,
                "r.groups[1].map.offset",
            ),
            (
                rows(vec![
                    RowGroup {
                        spans: RowSpans::Degrees {
                            widths: vec![10, 10],
                        },
                        map: RowMap::Listed { signs: vec![1, 2] },
                    },
                    good.clone(),
                ]),
                2,
                "r.groups[0].spans.widths",
            ),
            (rows(vec![good.clone(), good.clone()]), 0, "r.divisions"),
            (rows(vec![good.clone(), good]), 301, "r.divisions"),
        ];
        for (rows, divisions, field) in cases {
            let error = rows.check(divisions, "r").unwrap_err();
            assert_eq!(error.field(), Some(field), "{error}");
        }
    }

    #[test]
    fn the_settings_refuse_a_choice_by_its_field_and_round_trip_a_good_one() {
        use crate::catalogue::Varga;
        use crate::settings::{ProfileId, SettingsPatch, VargaChoice, VargaVariant, root};

        let patch: SettingsPatch = serde_json::from_str(
            r#"{"vargas": {"schemes": {
                "D2": {"kind": "NAMED", "which": "KASHINATHA"},
                "D3": {"kind": "CUSTOM", "classifier": "ALL", "groups": [
                    {"spans": {"spans": "EQUAL"}, "map": {"map": "STEP", "base": "SAME", "step": 4, "offset": 6}}
                ]}
            }}}"#,
        )
        .unwrap();
        let settings = root().patched(&patch);
        let resolved = settings.clone().validated(ProfileId::new("mine")).unwrap();
        // The document the settings write reads back as the same settings.
        let written = serde_json::to_string(&resolved.settings).unwrap();
        assert_eq!(serde_json::from_str::<crate::settings::Settings>(&written).unwrap(), resolved.settings);
        assert!(written.contains(r#""D2":{"kind":"NAMED","which":"KASHINATHA"}"#), "{written}");

        for (varga, choice, field) in [
            (Varga::D9, VargaChoice::Named { which: VargaVariant::Somanatha }, "vargas.schemes.D9.which"),
            (Varga::D1, VargaChoice::Named { which: VargaVariant::Kashinatha }, "vargas.schemes.D1"),
            (Varga::D2, VargaChoice::Named { which: VargaVariant::Custom }, "vargas.schemes.D2.which"),
            (
                Varga::D9,
                VargaChoice::Custom { classifier: Classifier::Parity, groups: vec![] },
                "vargas.schemes.D9.groups",
            ),
        ] {
            let mut refused = settings.clone();
            refused.vargas.schemes.insert(varga, choice);
            let diagnostics = refused.validated(ProfileId::new("mine")).unwrap_err();
            let errors: Vec<_> = diagnostics.errors().collect();
            assert_eq!(errors.len(), 1, "{diagnostics}");
            assert_eq!(errors[0].rule, "varga-scheme");
            assert!(errors[0].message.contains(field), "{}", errors[0].message);
        }
    }

    #[test]
    fn a_written_rule_reads_back_as_itself() {
        let rows = rows(vec![
            RowGroup {
                spans: RowSpans::Degrees {
                    widths: vec![5, 5, 8, 7, 5],
                },
                map: RowMap::Listed {
                    signs: vec![0, 10, 8, 2, 6],
                },
            },
            equal(RowMap::Step {
                base: SignBase::Cyclic,
                step: 11,
                offset: 8,
            }),
        ]);
        let written = serde_json::to_string(&rows).unwrap();
        assert_eq!(serde_json::from_str::<Rows>(&written).unwrap(), rows);
    }
}
