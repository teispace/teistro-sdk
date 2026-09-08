//! The rule a divisional chart is computed by, and the twenty-one rows
//! the SDK ships.
//!
//! There is one evaluator. Sort the sign into a group, take the part of
//! the sign the longitude falls in, and either step through the signs
//! from somewhere or read the answer off a list:
//!
//! ```text
//! sign = (a·rashi + step·part + offset) mod 12
//! ```
//!
//! with `a` nought (count from a fixed sign), one (from the sign itself)
//! or the chart's own number (count continuously round the zodiac). Of
//! the twenty-one charts the SDK ships, eighteen step and three list, and
//! the corpus decides it outright: a divisional chart's answer is a
//! function of one sidereal longitude, and 19 530 recorded placements
//! over two zodiacs reproduce these rows with nothing left over
//! (`03-design/varga-tables-measured.md`).
//!
//! Two things the measurement corrected in the design page:
//!
//! - **The spans belong to the group, not to the chart.** D30's odd signs
//!   are cut 5, 5, 8, 7, 5 degrees and its even signs 5, 7, 8, 5, 5 — the
//!   same widths reversed — so one chart has two span rules.
//! - **`divisions` names the chart and is not always its part count.**
//!   D30 is called thirty and cuts a sign into five. Only equal spans
//!   make the two the same, and [`Group::parts`] is what a caller counts
//!   with.

use serde::{Deserialize, Deserializer, Serialize};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Rashi, Varga};
use teistro_core::error::{Error, Status};
use teistro_core::settings::{Settings, UnattestedDn};

/// The most parts a chart may divide the zodiac into.
///
/// Three hundred, which is past every attested chart and past the point
/// where a part is smaller than the ephemeris's own uncertainty. A caller
/// asking for more has a bug rather than a requirement, and the refusal
/// names the limit.
pub const MOST_DIVISIONS: u16 = 300;

/// The signs of the zodiac.
const SIGNS: u16 = 12;

/// How a sign is sorted into groups, each with its own rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
}

impl Classifier {
    /// How many groups it sorts the twelve signs into.
    #[must_use]
    pub const fn groups(self) -> u8 {
        match self {
            Classifier::All => 1,
            Classifier::Parity => 2,
            Classifier::Modality => 3,
            Classifier::Element => 4,
        }
    }

    /// Which group a sign belongs to.
    ///
    /// Every one of the four is the sign's index modulo the group count,
    /// because the zodiac alternates parity, cycles the modalities in
    /// threes and the elements in fours from Aries. That is arithmetic
    /// rather than a coincidence, and it is why one classifier covers
    /// all four.
    #[must_use]
    pub const fn group_of(self, sign: Rashi) -> usize {
        (sign as u8 % self.groups()) as usize
    }
}

/// The multiplier a stepping rule puts on the sign.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    #[must_use]
    pub const fn multiplier(self, divisions: u16) -> u32 {
        match self {
            SignBase::Fixed => 0,
            SignBase::Same => 1,
            SignBase::Cyclic => divisions as u32,
        }
    }
}

/// How a part of a sign becomes a sign.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "map", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Map {
    /// Step through the signs: `(a·rashi + step·part + offset) mod 12`.
    Step {
        /// What the sign is multiplied by.
        base: SignBase,
        /// How many signs on each part is from the last.
        step: u8,
        /// Where the first part starts, counted from Aries.
        offset: u8,
    },
    /// One sign per part, listed in order.
    Listed(&'static [u8]),
}

/// Where the parts of a sign begin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "spans", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Spans {
    /// The chart's divisions, each of `30/N` degrees.
    Equal,
    /// Whole-degree widths, summing to thirty. Only the trimshamsha
    /// needs them, and every attested variant of it is whole degrees
    /// too.
    Degrees(&'static [u8]),
}

/// One classifier group's rule: how wide its parts are, and where each
/// one sends a body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Group {
    /// Where its parts begin.
    pub spans: Spans,
    /// Where each part sends a body.
    pub map: Map,
}

impl Group {
    /// How many parts it cuts a sign into.
    ///
    /// Not always the chart's divisions: D30 is called thirty and cuts a
    /// sign into five.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a span table fills a sign in whole degrees, so it is at most thirty entries"
    )]
    pub const fn parts(&self, divisions: u16) -> u16 {
        match self.spans {
            Spans::Equal => divisions,
            Spans::Degrees(widths) => widths.len() as u16,
        }
    }
}

/// A divisional chart's rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Scheme {
    /// The catalogued chart, or `None` for an arbitrary D-N.
    pub varga: Option<Varga>,
    /// What the chart is called: D9's nine, D150's hundred and fifty.
    pub divisions: u16,
    /// How its signs are sorted into groups.
    pub classifier: Classifier,
    /// One rule per group, in group order.
    pub groups: &'static [Group],
}

impl<'de> Deserialize<'de> for Scheme {
    /// Reads a scheme back from a document by its **identity**, and takes
    /// the rule itself from this build.
    ///
    /// The groups are written into a document so a reader can see the
    /// rule that was applied, and so the content hash moves when the rule
    /// does. They are not read back: a group's map may name a
    /// `&'static [u8]` of signs, and no document can produce one. That
    /// costs nothing, because the hash is what catches a document written
    /// under a different table — the groups are in the bytes it is taken
    /// over.
    ///
    /// What is read is checked. A catalogued chart's divisions and
    /// classifier must be the ones this build's catalogue gives it, so a
    /// document that names D9 and describes something else is refused by
    /// name rather than quietly read as D9.
    ///
    /// # Errors
    ///
    /// A `varga` this build does not catalogue, a `divisions` or
    /// `classifier` that disagrees with it, or an unattested D-N whose
    /// divisions are outside [`MOST_DIVISIONS`].
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Scheme, D::Error> {
        /// The fields a document carries, as it carries them.
        #[derive(Deserialize)]
        struct Written {
            varga: Option<Varga>,
            divisions: u16,
            classifier: Classifier,
            /// Read and dropped; see the note above.
            #[serde(default)]
            groups: serde::de::IgnoredAny,
        }
        let written = Written::deserialize(deserializer)?;
        let _ = written.groups;
        let rebuilt = match written.varga {
            Some(varga) => Scheme::of(varga),
            // The only convention this crate implements. A document does
            // not carry the one it was computed under — that is in the
            // envelope's `applied_conventions` — so a second convention
            // means reading it from there rather than assuming here.
            None => Scheme::cyclic(written.divisions).map_err(serde::de::Error::custom)?,
        };
        if rebuilt.divisions != written.divisions {
            return Err(serde::de::Error::custom(format!(
                "the document says {:?} divides a sign {} times and this build says {}",
                written.varga, written.divisions, rebuilt.divisions
            )));
        }
        if rebuilt.classifier != written.classifier {
            return Err(serde::de::Error::custom(format!(
                "the document sorts {:?}'s signs by {:?} and this build sorts them by {:?}",
                written.varga, written.classifier, rebuilt.classifier
            )));
        }
        Ok(rebuilt)
    }
}

/// A group whose parts are equal and whose rule steps.
const fn step(base: SignBase, step: u8, offset: u8) -> Group {
    Group {
        spans: Spans::Equal,
        map: Map::Step { base, step, offset },
    }
}

/// A group whose parts are equal and whose signs are listed.
const fn listed(signs: &'static [u8]) -> Group {
    Group {
        spans: Spans::Equal,
        map: Map::Listed(signs),
    }
}

// The rows. Each is named for what it does rather than for the chart it
// first appeared in, because several charts share one.

/// The rashi chart: the sign itself, unchanged.
const IDENTITY: [Group; 1] = [step(SignBase::Same, 0, 0)];
/// The drekkana: trines from the sign.
const TRINES: [Group; 1] = [step(SignBase::Same, 4, 0)];
/// The chaturthamsha: quadrants from the sign.
const QUADRANTS: [Group; 1] = [step(SignBase::Same, 3, 0)];
/// One sign per part, from the sign itself: the dwadashamsha and kin.
const FROM_SIGN: [Group; 1] = [step(SignBase::Same, 1, 0)];
/// The navamsha: counted continuously round the zodiac.
const CONTINUOUS: [Group; 1] = [step(SignBase::Cyclic, 1, 0)];

/// The saptamsha: odd signs from themselves, even from the seventh on.
const FROM_SIGN_OR_SEVENTH: [Group; 2] = [step(SignBase::Same, 1, 0), step(SignBase::Same, 1, 6)];
/// The dashamsha: even signs from the ninth on.
const FROM_SIGN_OR_NINTH: [Group; 2] = [step(SignBase::Same, 1, 0), step(SignBase::Same, 1, 8)];
/// The rudramsha: even signs from the eleventh on.
const FROM_SIGN_OR_ELEVENTH: [Group; 2] = [step(SignBase::Same, 1, 0), step(SignBase::Same, 1, 10)];
/// The shashtamsha and khavedamsha: odd from Aries, even from Libra.
const ARIES_OR_LIBRA: [Group; 2] = [step(SignBase::Fixed, 1, 0), step(SignBase::Fixed, 1, 6)];
/// The chaturvimshamsha: odd from Leo, even from Cancer.
const LEO_OR_CANCER: [Group; 2] = [step(SignBase::Fixed, 1, 4), step(SignBase::Fixed, 1, 3)];

/// The ashtamsha, shodashamsha and akshavedamsha: movable from Aries,
/// fixed from Leo, dual from Sagittarius.
const BY_MODALITY: [Group; 3] = [
    step(SignBase::Fixed, 1, 0),
    step(SignBase::Fixed, 1, 4),
    step(SignBase::Fixed, 1, 8),
];
/// The vimshamsha: movable from Aries, fixed from Sagittarius, dual from
/// Leo — the same three signs in another order.
const BY_MODALITY_REVERSED: [Group; 3] = [
    step(SignBase::Fixed, 1, 0),
    step(SignBase::Fixed, 1, 8),
    step(SignBase::Fixed, 1, 4),
];
/// The nakshatramsha: from the first sign of the body's element.
const BY_ELEMENT: [Group; 4] = [
    step(SignBase::Fixed, 1, 0),
    step(SignBase::Fixed, 1, 3),
    step(SignBase::Fixed, 1, 6),
    step(SignBase::Fixed, 1, 9),
];

/// The hora: Leo and Cancer only, which way round depending on the
/// sign's parity.
const HORA: [Group; 2] = [listed(&[4, 3]), listed(&[3, 4])];

/// The five signs a panchamsha and a trimshamsha alike send a body to:
/// the fiery, airy and other lords of the odd signs, and their mirror.
const FIVE_ODD: [u8; 5] = [0, 10, 8, 2, 6];
const FIVE_EVEN: [u8; 5] = [1, 5, 11, 9, 7];

/// The panchamsha: the five signs over five equal parts.
const PANCHAMSHA: [Group; 2] = [listed(&FIVE_ODD), listed(&FIVE_EVEN)];

/// The trimshamsha: the same five signs over five unequal ones, mirrored
/// between the parities.
const TRIMSHAMSHA: [Group; 2] = [
    Group {
        spans: Spans::Degrees(&[5, 5, 8, 7, 5]),
        map: Map::Listed(&FIVE_ODD),
    },
    Group {
        spans: Spans::Degrees(&[5, 7, 8, 5, 5]),
        map: Map::Listed(&FIVE_EVEN),
    },
];

/// One row of the twenty-one, as it is written down.
const fn row(
    varga: Varga,
    divisions: u16,
    classifier: Classifier,
    groups: &'static [Group],
) -> Scheme {
    Scheme {
        varga: Some(varga),
        divisions,
        classifier,
        groups,
    }
}

/// The twenty-one divisional charts the SDK ships, in catalogue order.
pub const SCHEMES: [Scheme; 21] = [
    row(Varga::D1, 1, Classifier::All, &IDENTITY),
    row(Varga::D2, 2, Classifier::Parity, &HORA),
    row(Varga::D3, 3, Classifier::All, &TRINES),
    row(Varga::D4, 4, Classifier::All, &QUADRANTS),
    row(Varga::D5, 5, Classifier::Parity, &PANCHAMSHA),
    row(Varga::D6, 6, Classifier::Parity, &ARIES_OR_LIBRA),
    row(Varga::D7, 7, Classifier::Parity, &FROM_SIGN_OR_SEVENTH),
    row(Varga::D8, 8, Classifier::Modality, &BY_MODALITY),
    row(Varga::D9, 9, Classifier::All, &CONTINUOUS),
    row(Varga::D10, 10, Classifier::Parity, &FROM_SIGN_OR_NINTH),
    row(Varga::D11, 11, Classifier::Parity, &FROM_SIGN_OR_ELEVENTH),
    row(Varga::D12, 12, Classifier::All, &FROM_SIGN),
    row(Varga::D16, 16, Classifier::Modality, &BY_MODALITY),
    row(Varga::D20, 20, Classifier::Modality, &BY_MODALITY_REVERSED),
    row(Varga::D24, 24, Classifier::Parity, &LEO_OR_CANCER),
    row(Varga::D27, 27, Classifier::Element, &BY_ELEMENT),
    row(Varga::D30, 30, Classifier::Parity, &TRIMSHAMSHA),
    row(Varga::D40, 40, Classifier::Parity, &ARIES_OR_LIBRA),
    row(Varga::D45, 45, Classifier::Modality, &BY_MODALITY),
    row(Varga::D60, 60, Classifier::All, &FROM_SIGN),
    row(Varga::D150, 150, Classifier::All, &FROM_SIGN),
];

impl Scheme {
    /// The rule of a catalogued chart.
    ///
    /// Every member of the catalogue has one, so this cannot fail; a
    /// chart the catalogue gains without a row here is caught by the
    /// crate's own test rather than at run time.
    #[must_use]
    pub fn of(varga: Varga) -> Scheme {
        SCHEMES
            .iter()
            .copied()
            .find(|scheme| scheme.varga == Some(varga))
            .unwrap_or(SCHEMES[0])
    }

    /// The rule of an arbitrary D-N under the convention the settings
    /// name.
    ///
    /// **There is no classical rule for D37.** Producing one takes a
    /// chosen convention, and `vargas.unattested_dn` is where the choice
    /// is made; the result carries it in its provenance, which is what
    /// makes the chart reproducible rather than merely plausible.
    ///
    /// The match on the convention is **exhaustive**, so a reading added
    /// to the catalogue forces a decision here rather than being
    /// silently ignored — which is what the knob having no reader had
    /// meant until now (`check-lints`, `knob-has-a-reader`).
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for nought divisions, and `OUT_OF_RANGE` past
    /// [`MOST_DIVISIONS`], both naming the limit; `UNSUPPORTED` for a
    /// convention this crate does not implement, naming it.
    pub fn unattested(divisions: u16, convention: UnattestedDn) -> Result<Scheme, Error> {
        match convention {
            UnattestedDn::Cyclic => Scheme::cyclic(divisions),
            // `UnattestedDn` is non-exhaustive: a reading the catalogue
            // gains and this crate has not been taught is refused by
            // name rather than quietly read as the cyclic one.
            other => Err(Error::unsupported(format!(
                "no rule for an arbitrary divisional chart under `{}`; \
                 the SDK ships `{}`",
                other.key(),
                UnattestedDn::Cyclic.key()
            ))
            .with_field("vargas.unattested_dn")),
        }
    }

    /// The rule of an arbitrary D-N under the convention a chart's own
    /// settings name.
    ///
    /// This is the reader of `vargas.unattested_dn`: a caller with
    /// settings in hand asks here rather than choosing a convention and
    /// hoping it is the one the chart was computed under.
    ///
    /// # Errors
    ///
    /// As [`Scheme::unattested`].
    pub fn for_settings(divisions: u16, settings: &Settings) -> Result<Scheme, Error> {
        Scheme::unattested(divisions, settings.vargas.unattested_dn)
    }

    /// The rule of an arbitrary D-N under the parivritti (cyclic)
    /// convention, which is what [`Scheme::unattested`] resolves
    /// `vargas.unattested_dn` to.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for nought divisions, and `OUT_OF_RANGE` past
    /// [`MOST_DIVISIONS`], both naming the limit.
    pub fn cyclic(divisions: u16) -> Result<Scheme, Error> {
        if divisions == 0 {
            return Err(
                Error::invalid_arg("a divisional chart divides a sign at least once")
                    .with_field("divisions"),
            );
        }
        if divisions > MOST_DIVISIONS {
            return Err(Error::new(
                Status::OutOfRange,
                format!("a divisional chart divides into at most {MOST_DIVISIONS} parts, not {divisions}"),
            )
            .with_field("divisions"));
        }
        Ok(Scheme {
            varga: None,
            divisions,
            classifier: Classifier::All,
            groups: &CONTINUOUS,
        })
    }

    /// The group a sign belongs to.
    #[must_use]
    pub fn group(&self, sign: Rashi) -> &Group {
        self.groups
            .get(self.classifier.group_of(sign))
            // Every row has one group per classifier group, which the
            // crate's own test asserts over all of them; the first
            // stands in rather than panicking at a caller.
            .unwrap_or(&IDENTITY[0])
    }

    /// How many parts a sign is cut into, which is the same for every
    /// group of every shipped chart.
    #[must_use]
    pub fn parts(&self, sign: Rashi) -> u16 {
        self.group(sign).parts(self.divisions)
    }

    /// What the chart is called: `D9`, or `D37` for one the catalogue
    /// does not name.
    #[must_use]
    pub fn key(&self) -> String {
        self.varga.map_or_else(
            || format!("D{}", self.divisions),
            |varga| varga.key().to_string(),
        )
    }
}

/// The sign a part of a group's sign is sent to.
///
/// Returns `None` only for a listed group asked for a part it does not
/// list, which the crate's own invariants rule out.
#[must_use]
pub fn target(group: &Group, sign: Rashi, part: u16, divisions: u16) -> Option<Rashi> {
    let index = match group.map {
        Map::Step { base, step, offset } => {
            let from = base.multiplier(divisions) * u32::from(sign as u8);
            let along = u32::from(step) * u32::from(part);
            (from + along + u32::from(offset)) % u32::from(SIGNS)
        }
        Map::Listed(signs) => u32::from(*signs.get(usize::from(part))?),
    };
    Rashi::from_id(u16::try_from(index).ok()?)
}

/// The part of a sign a longitude falls in, exactly.
///
/// Integer arithmetic on the canonical angle (ADR-0016), never a
/// division of degrees: the recording engine computes it as
/// `floor(deg / (30/N))` in floating point, and none of 30/7, 30/11,
/// 30/27 or 0.2 is representable, so a body exactly on a part boundary
/// can land either side of it depending on the platform.
#[must_use]
pub fn part_of(group: &Group, longitude: Nas, divisions: u16) -> u16 {
    match group.spans {
        Spans::Equal => {
            let part = longitude.part(u32::from(divisions.max(1)));
            u16::try_from(part)
                .unwrap_or(0)
                .min(divisions.saturating_sub(1))
        }
        Spans::Degrees(widths) => {
            let inside = longitude.in_sign().get();
            let mut edge = 0_i64;
            for (index, width) in widths.iter().enumerate() {
                edge += i64::from(*width) * Nas::PER_DEGREE;
                if inside < edge {
                    return u16::try_from(index).unwrap_or(0);
                }
            }
            u16::try_from(widths.len().saturating_sub(1)).unwrap_or(0)
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use teistro_core::settings::UnattestedDn;

    use super::{
        Classifier, MOST_DIVISIONS, Map, SCHEMES, Scheme, SignBase, Spans, part_of, target,
    };
    use teistro_core::angle::Nas;
    use teistro_core::catalogue::{Rashi, Varga};
    use teistro_core::quantity::Degrees;

    fn at(degrees: f64) -> Nas {
        Nas::from_degrees(Degrees::try_new(degrees).expect("a finite angle"))
    }

    #[test]
    fn every_catalogued_chart_has_exactly_one_row() {
        assert_eq!(SCHEMES.len(), Varga::ALL.len());
        for varga in Varga::ALL {
            let rows = SCHEMES.iter().filter(|s| s.varga == Some(varga)).count();
            assert_eq!(rows, 1, "{varga:?}");
            let scheme = Scheme::of(varga);
            assert_eq!(scheme.varga, Some(varga));
            assert_eq!(
                scheme.divisions,
                varga.attributes().divisions,
                "the row's divisions are the catalogue's"
            );
            assert_eq!(scheme.key(), varga.key());
        }
    }

    #[test]
    fn a_row_has_one_group_per_classifier_group() {
        for scheme in SCHEMES {
            assert_eq!(
                scheme.groups.len(),
                usize::from(scheme.classifier.groups()),
                "{:?}",
                scheme.varga
            );
            // And every sign reaches one of them.
            for sign in Rashi::ALL {
                let group = scheme.group(sign);
                assert!(scheme.groups.contains(group), "{:?} {sign:?}", scheme.varga);
            }
        }
    }

    #[test]
    fn a_listed_group_lists_one_sign_per_part() {
        for scheme in SCHEMES {
            for group in scheme.groups {
                if let Map::Listed(signs) = group.map {
                    assert_eq!(
                        u16::try_from(signs.len()).unwrap(),
                        group.parts(scheme.divisions),
                        "{:?}",
                        scheme.varga
                    );
                    assert!(signs.iter().all(|sign| *sign < 12), "{:?}", scheme.varga);
                }
                if let Spans::Degrees(widths) = group.spans {
                    let total: u16 = widths.iter().map(|width| u16::from(*width)).sum();
                    assert_eq!(total, 30, "{:?}: the spans fill a sign", scheme.varga);
                }
            }
        }
    }

    #[test]
    fn the_chart_is_named_by_its_divisions_and_cut_by_its_spans() {
        for scheme in SCHEMES {
            for sign in Rashi::ALL {
                let parts = scheme.parts(sign);
                if scheme.varga == Some(Varga::D30) {
                    assert_eq!(parts, 5, "a sign holds five trimshamshas");
                } else {
                    assert_eq!(parts, scheme.divisions, "{:?}", scheme.varga);
                }
            }
        }
    }

    #[test]
    fn the_rashi_chart_is_the_identity() {
        let rashi = Scheme::of(Varga::D1);
        for sign in Rashi::ALL {
            for degree in [0.0, 15.0, 29.999] {
                let longitude = at(f64::from(sign as u8) * 30.0 + degree);
                let group = rashi.group(sign);
                assert_eq!(part_of(group, longitude, 1), 0);
                assert_eq!(target(group, sign, 0, 1), Some(sign));
            }
        }
    }

    #[test]
    fn the_navamsha_counts_continuously_round_the_zodiac() {
        let navamsha = Scheme::of(Varga::D9);
        assert_eq!(navamsha.classifier, Classifier::All);
        let group = navamsha.group(Rashi::Aries);
        // Aries' nine navamshas run Aries to Sagittarius.
        for part in 0..9_u16 {
            assert_eq!(
                target(group, Rashi::Aries, part, 9),
                Rashi::from_id(part % 12)
            );
        }
        // And Taurus's pick up where they left off, which is what
        // "continuously" means.
        assert_eq!(target(group, Rashi::Taurus, 0, 9), Some(Rashi::Capricorn));
    }

    #[test]
    fn a_part_index_is_exact_at_a_boundary() {
        // Seven parts of thirty degrees: 30/7 is not representable, and
        // the integer path lands on the part that owns the boundary.
        let seven = Scheme::of(Varga::D7);
        let group = seven.group(Rashi::Aries);
        assert_eq!(part_of(group, at(0.0), 7), 0);
        let boundary = 30.0 / 7.0;
        assert_eq!(part_of(group, at(boundary * 0.999_999), 7), 0);
        assert_eq!(part_of(group, at(boundary * 1.000_001), 7), 1);
        assert_eq!(part_of(group, at(29.999_999), 7), 6);
        // And it never leaves the table, whatever it is handed.
        assert_eq!(part_of(group, at(30.0), 7), 0, "thirty is the next sign");
    }

    #[test]
    fn the_trimshamshas_parts_are_the_measured_widths() {
        let scheme = Scheme::of(Varga::D30);
        let odd = scheme.group(Rashi::Aries);
        for (degree, part) in [
            (0.0, 0),
            (4.999, 0),
            (5.0, 1),
            (9.999, 1),
            (10.0, 2),
            (17.999, 2),
            (18.0, 3),
            (24.999, 3),
            (25.0, 4),
            (29.999, 4),
        ] {
            assert_eq!(part_of(odd, at(degree), 30), part, "odd at {degree}");
        }
        let even = scheme.group(Rashi::Taurus);
        for (degree, part) in [
            (4.999, 0),
            (5.0, 1),
            (11.999, 1),
            (12.0, 2),
            (19.999, 2),
            (20.0, 3),
            (25.0, 4),
        ] {
            assert_eq!(
                part_of(even, at(30.0 + degree), 30),
                part,
                "even at {degree}"
            );
        }
        // The five signs are the panchamsha's, in the same order.
        let panchamsha = Scheme::of(Varga::D5);
        let five = panchamsha.group(Rashi::Aries);
        for part in 0..5_u16 {
            assert_eq!(
                target(odd, Rashi::Aries, part, 30),
                target(five, Rashi::Aries, part, 5)
            );
        }
    }

    #[test]
    fn an_arbitrary_chart_is_cyclic_and_bounded() {
        let d37 = Scheme::cyclic(37).expect("inside the limit");
        assert_eq!(d37.varga, None);
        assert_eq!(d37.divisions, 37);
        assert_eq!(d37.key(), "D37");
        assert!(matches!(
            d37.group(Rashi::Aries).map,
            Map::Step {
                base: SignBase::Cyclic,
                ..
            }
        ));
        // Its cells cover the twelve signs evenly, which is what
        // parivritti means.
        let mut seen = [0_u16; 12];
        for sign in Rashi::ALL {
            for part in 0..37 {
                let target = target(d37.group(sign), sign, part, 37).expect("a sign");
                seen[target as usize] += 1;
            }
        }
        assert_eq!(seen.iter().sum::<u16>(), 12 * 37);
        assert!(seen.iter().all(|count| *count == 37), "{seen:?}");

        assert!(Scheme::cyclic(0).is_err(), "nought is not a division");
        assert!(Scheme::cyclic(MOST_DIVISIONS).is_ok());
        assert!(Scheme::cyclic(MOST_DIVISIONS + 1).is_err());
    }

    #[test]
    fn an_arbitrary_chart_takes_the_convention_the_settings_name() {
        // The knob had no reader until this: `Scheme::cyclic` named a
        // convention it never asked for, and a reading added to the
        // catalogue would have been ignored (`check-lints`,
        // `knob-has-a-reader`).
        let asked = Scheme::unattested(37, UnattestedDn::Cyclic).expect("a cyclic D37");
        let direct = Scheme::cyclic(37).expect("the same");
        assert_eq!(asked.divisions, direct.divisions);
        assert_eq!(asked.classifier, direct.classifier);
        assert_eq!(asked.varga, None, "no catalogue row names it");
        // And the bounds are the same either way in.
        assert!(Scheme::unattested(0, UnattestedDn::Cyclic).is_err());
        assert!(Scheme::unattested(MOST_DIVISIONS + 1, UnattestedDn::Cyclic).is_err());
        assert!(Scheme::unattested(MOST_DIVISIONS, UnattestedDn::Cyclic).is_ok());

        // And a caller with settings in hand asks them rather than
        // choosing, which is what gives the knob its reader.
        let settings =
            teistro_core::settings::Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
                .expect("the default profile")
                .resolve(&teistro_core::settings::SettingsPatch::default())
                .expect("it resolves")
                .settings;
        let from_settings = Scheme::for_settings(37, &settings).expect("a D37");
        assert_eq!(from_settings.divisions, asked.divisions);
        assert_eq!(from_settings.classifier, asked.classifier);
    }
}
