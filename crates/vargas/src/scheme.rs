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
use teistro_core::settings::{Settings, UnattestedDn, VargaChoice, VargaVariant};
use teistro_core::varga_rows::{RowGroup, RowMap, RowSpans, Rows};

/// The most parts a chart may divide the zodiac into, the core's
/// [`teistro_core::varga_rows::MOST_DIVISIONS`]: three hundred, past every
/// attested chart and past the point where a part is smaller than the
/// ephemeris's own uncertainty. A caller asking for more has a bug rather
/// than a requirement, and the refusal names the limit.
///
/// ```
/// use teistro_vargas::{MOST_DIVISIONS, Scheme};
///
/// assert!(Scheme::cyclic(MOST_DIVISIONS).is_ok());
/// assert!(Scheme::cyclic(MOST_DIVISIONS + 1).is_err());
/// ```
pub use teistro_core::varga_rows::MOST_DIVISIONS;

/// How a sign is sorted into groups, and the multiplier a stepping rule
/// puts on the sign: the core's, since a consumer's own rows in the
/// settings name them too.
pub use teistro_core::varga_rows::{Classifier, SignBase};

/// The signs of the zodiac.
const SIGNS: u16 = 12;

/// How a part of a sign becomes a sign.
///
/// ```
/// use teistro_core::catalogue::{Rashi, Varga};
/// use teistro_vargas::{Map, Scheme};
///
/// // The hora lists its two signs: an odd sign's first half is Leo's.
/// let hora = Scheme::of(Varga::D2);
/// assert_eq!(hora.group(Rashi::Aries).map, Map::Listed { signs: &[4, 3] });
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "VargaMap"))]
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
    ///
    /// A struct variant, not a newtype: the enum is tagged by `map`, and
    /// serde cannot write a tagged newtype holding a list — which it did
    /// not, and every document carrying a listed scheme failed to
    /// serialise and was sealed with the hash of nothing.
    Listed {
        /// The signs, 0 for Aries, a part each.
        signs: &'static [u8],
    },
}

/// Where the parts of a sign begin.
///
/// ```
/// use teistro_core::catalogue::{Rashi, Varga};
/// use teistro_vargas::{Scheme, Spans};
///
/// let d30 = Scheme::of(Varga::D30);
/// assert_eq!(d30.group(Rashi::Aries).spans, Spans::Degrees { widths: &[5, 5, 8, 7, 5] });
/// assert_eq!(d30.group(Rashi::Taurus).spans, Spans::Degrees { widths: &[5, 7, 8, 5, 5] });
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "spans", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Spans {
    /// The chart's divisions, each of `30/N` degrees.
    Equal,
    /// Whole-degree widths, summing to thirty. Only the trimshamsha
    /// needs them, and every attested variant of it is whole degrees
    /// too.
    ///
    /// A struct variant, as [`Map::Listed`] is and for its reason: serde
    /// cannot write a tagged newtype holding a list.
    Degrees {
        /// The parts' widths in degrees, in order.
        widths: &'static [u8],
    },
}

/// One classifier group's rule: how wide its parts are, and where each
/// one sends a body.
///
/// ```
/// use teistro_core::catalogue::{Rashi, Varga};
/// use teistro_vargas::{Scheme, Spans};
///
/// let d9 = Scheme::of(Varga::D9);
/// assert_eq!(d9.group(Rashi::Leo).spans, Spans::Equal);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "VargaGroup"))]
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
    ///
    /// ```
    /// use teistro_core::catalogue::{Rashi, Varga};
    /// use teistro_vargas::Scheme;
    ///
    /// let d30 = Scheme::of(Varga::D30);
    /// assert_eq!(d30.group(Rashi::Aries).parts(d30.divisions), 5);
    /// ```
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a span table fills a sign in whole degrees, so it is at most thirty entries"
    )]
    pub const fn parts(&self, divisions: u16) -> u16 {
        match self.spans {
            Spans::Equal => divisions,
            Spans::Degrees { widths } => widths.len() as u16,
        }
    }
}

impl Group {
    /// The group as owned rows.
    ///
    /// ```
    /// use teistro_core::catalogue::{Rashi, Varga};
    /// use teistro_core::varga_rows::{RowMap, RowSpans};
    /// use teistro_vargas::Scheme;
    ///
    /// let hora = Scheme::of(Varga::D2);
    /// let rows = hora.group(Rashi::Aries).rows();
    /// assert_eq!((rows.spans, rows.map), (RowSpans::Equal, RowMap::Listed { signs: vec![4, 3] }));
    /// ```
    #[must_use]
    pub fn rows(&self) -> RowGroup {
        RowGroup {
            spans: match self.spans {
                Spans::Equal => RowSpans::Equal,
                Spans::Degrees { widths } => RowSpans::Degrees {
                    widths: widths.to_vec(),
                },
            },
            map: match self.map {
                Map::Step { base, step, offset } => RowMap::Step { base, step, offset },
                Map::Listed { signs } => RowMap::Listed {
                    signs: signs.to_vec(),
                },
            },
        }
    }
}

/// The most distinct consumer rules a process keeps (see
/// [`Scheme::from_rows`]): far past any consumer's set of charts, and a
/// bound on what a caller generating rules in a loop can hold.
///
/// ```
/// assert_eq!(teistro_vargas::scheme::MOST_RULES, 1024);
/// ```
pub const MOST_RULES: usize = 1024;

/// The consumer rules kept so far, each with the `'static` groups it was
/// made into.
type Kept = Vec<(Rows, &'static [Group])>;

/// The `'static` groups of checked rows: the ones made before for equal
/// rows, or new ones kept for the life of the process.
fn intern(rows: &Rows) -> Result<&'static [Group], Error> {
    static KEPT: std::sync::OnceLock<std::sync::Mutex<Kept>> = std::sync::OnceLock::new();
    let mut kept = KEPT
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
        // A panic elsewhere while holding the lock leaves the list whole:
        // it is only ever pushed to.
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((_, groups)) = kept.iter().find(|(seen, _)| seen == rows) {
        return Ok(groups);
    }
    if kept.len() >= MOST_RULES {
        return Err(Error::new(
            Status::Limit,
            format!("a process keeps at most {MOST_RULES} distinct divisional rules"),
        )
        .with_field("vargas.schemes"));
    }
    let groups: Vec<Group> = rows
        .groups
        .iter()
        .map(|group| Group {
            spans: match &group.spans {
                RowSpans::Equal => Spans::Equal,
                RowSpans::Degrees { widths } => Spans::Degrees {
                    widths: Box::leak(widths.clone().into_boxed_slice()),
                },
            },
            map: match &group.map {
                RowMap::Step { base, step, offset } => Map::Step {
                    base: *base,
                    step: *step,
                    offset: *offset,
                },
                RowMap::Listed { signs } => Map::Listed {
                    signs: Box::leak(signs.clone().into_boxed_slice()),
                },
            },
        })
        .collect();
    let groups: &'static [Group] = Box::leak(groups.into_boxed_slice());
    kept.push((rows.clone(), groups));
    Ok(groups)
}

/// A divisional chart's rule.
///
/// ```
/// use teistro_core::catalogue::Varga;
/// use teistro_core::settings::VargaVariant;
/// use teistro_vargas::{Classifier, Scheme};
///
/// let d7 = Scheme::of(Varga::D7);
/// assert_eq!((d7.divisions, d7.classifier, d7.groups.len()), (7, Classifier::Parity, 2));
/// assert_eq!(d7.variant, VargaVariant::Parashara);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Scheme {
    /// The catalogued chart, or `None` for an arbitrary D-N.
    pub varga: Option<Varga>,
    /// Which of the chart's schemes this is: BPHS ch. 6's, a named
    /// variant, or a consumer's own rows (`CUSTOM`). Left out of a written
    /// scheme when it is `PARASHARA`, so a chart computed by the shipped
    /// rows writes, and hashes, as it always has.
    #[serde(default, skip_serializing_if = "VargaVariant::is_parashara")]
    pub variant: VargaVariant,
    /// What the chart is called: D9's nine, D150's hundred and fifty.
    pub divisions: u16,
    /// How its signs are sorted into groups.
    pub classifier: Classifier,
    /// One rule per group, in group order.
    pub groups: &'static [Group],
}

impl<'de> Deserialize<'de> for Scheme {
    /// Reads a scheme back from a document by its **identity**, and takes
    /// a shipped rule itself from this build.
    ///
    /// The groups are written into a document so a reader can see the
    /// rule that was applied, and so the content hash moves when the rule
    /// does. A shipped rule's (`PARASHARA` or a named variant) are not
    /// read back: the hash is what catches a document written under a
    /// different table, the groups being in the bytes it is taken over. A
    /// consumer's own rule (`CUSTOM`) has no row in this build, so its
    /// groups are read, checked as the settings check them, and become the
    /// rule: the chart reads back as the rows that computed it.
    ///
    /// What is read is checked. A catalogued chart's divisions and
    /// classifier must be the ones this build's row gives it, so a
    /// document that names D9 and describes something else is refused by
    /// name rather than quietly read as D9.
    ///
    /// # Errors
    ///
    /// A `varga` this build does not catalogue, a variant that is not one of
    /// the chart's, a `divisions` or `classifier` that disagrees with its
    /// row, rows that fail their check, or an unattested D-N whose divisions
    /// are outside [`MOST_DIVISIONS`].
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Scheme, D::Error> {
        /// The fields a document carries, as it carries them.
        #[derive(Deserialize)]
        struct Written {
            varga: Option<Varga>,
            #[serde(default)]
            variant: VargaVariant,
            divisions: u16,
            classifier: Classifier,
            /// Read whole: a consumer's rows are the rule, and a shipped
            /// rule's are dropped (see the note above).
            #[serde(default)]
            groups: Vec<RowGroup>,
        }
        let written = Written::deserialize(deserializer)?;
        let rebuilt = match (written.varga, written.variant) {
            (Some(varga), VargaVariant::Custom) => Scheme::from_rows(
                varga,
                &Rows {
                    classifier: written.classifier,
                    groups: written.groups,
                },
            ),
            (Some(varga), variant) => Scheme::named(varga, variant),
            // The only convention this crate implements. A document does
            // not carry the one it was computed under — that is in the
            // envelope's `applied_conventions` — so a second convention
            // means reading it from there rather than assuming here.
            (None, _) => Scheme::cyclic(written.divisions),
        }
        .map_err(serde::de::Error::custom)?;
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
        map: Map::Listed { signs },
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
        spans: Spans::Degrees {
            widths: &[5, 5, 8, 7, 5],
        },
        map: Map::Listed { signs: &FIVE_ODD },
    },
    Group {
        spans: Spans::Degrees {
            widths: &[5, 7, 8, 5, 5],
        },
        map: Map::Listed { signs: &FIVE_EVEN },
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
        variant: VargaVariant::Parashara,
        divisions,
        classifier,
        groups,
    }
}

// The named variants (`03-design/varga-schemes.md`, cruxes C2063 to C2066).
// Each is rows for the one evaluator, and each is cited where the knob's
// member is (`VargaVariant`).

/// Kashinatha's hora (P.V.R. Narasimha Rao, *Personal Finances from Hora
/// Chart*, 2003, after Kashinath Rath): a sign's Sun half — an odd sign's
/// first, an even sign's second, as Parashara gives them — goes to the
/// day-strong sign of the sign's lord, its Moon half to the night-strong
/// one. The day-strong signs are Leo, Virgo, Libra, Scorpio, Aquarius and
/// Pisces; Leo and Cancer are taken as a pair. One list a sign, the first
/// half's sign first.
const KASHINATHA_HORA: [Group; 12] = [
    listed(&[7, 0]),  // Aries, Mars's: Scorpio by day, Aries by night.
    listed(&[1, 6]),  // Taurus, Venus's: Moon half first.
    listed(&[5, 2]),  // Gemini, Mercury's.
    listed(&[3, 4]),  // Cancer, the Moon's: Cancer, then Leo.
    listed(&[4, 3]),  // Leo, the Sun's.
    listed(&[2, 5]),  // Virgo, Mercury's.
    listed(&[6, 1]),  // Libra, Venus's.
    listed(&[0, 7]),  // Scorpio, Mars's.
    listed(&[11, 8]), // Sagittarius, Jupiter's.
    listed(&[9, 10]), // Capricorn, Saturn's.
    listed(&[10, 9]), // Aquarius, Saturn's.
    listed(&[8, 11]), // Pisces, Jupiter's.
];

/// The Uma-Shambhu hora (Narasimha Rao, *Parasara's Hora Chart Decoded*,
/// 2012): the twenty-four halves of the zodiac counted twice round from
/// Aries, an even sign's two halves in reverse. Odd signs step on from
/// `2·rashi`; even signs from `2·rashi + 1` back by one.
const UMA_SHAMBHU_HORA: [Group; 2] = [step(SignBase::Cyclic, 1, 0), step(SignBase::Cyclic, 11, 1)];

/// Somanatha's drekkana (Somanatha's *Kalpalata* on Krishna Mishra's
/// *Phalaratnamala*, as Iranganti Rangacharya gives it): the odd signs'
/// drekkanas counted on from Aries, three a sign, and the even signs' back
/// from Pisces. Half the sign's number is no whole multiplier, so it is a
/// rule a sign.
const SOMANATHA_DREKKANA: [Group; 12] = [
    step(SignBase::Fixed, 1, 0),   // Aries: Aries, Taurus, Gemini.
    step(SignBase::Fixed, 11, 11), // Taurus: Pisces, Aquarius, Capricorn.
    step(SignBase::Fixed, 1, 3),   // Gemini: Cancer on.
    step(SignBase::Fixed, 11, 8),  // Cancer: Sagittarius back.
    step(SignBase::Fixed, 1, 6),   // Leo: Libra on.
    step(SignBase::Fixed, 11, 5),  // Virgo: Virgo back.
    step(SignBase::Fixed, 1, 9),   // Libra: Capricorn on.
    step(SignBase::Fixed, 11, 2),  // Scorpio: Gemini back.
    step(SignBase::Fixed, 1, 0),   // Sagittarius: round again from Aries.
    step(SignBase::Fixed, 11, 11), // Capricorn: from Pisces again.
    step(SignBase::Fixed, 1, 3),   // Aquarius.
    step(SignBase::Fixed, 11, 8),  // Pisces.
];

/// Krishna Mishra's navamsha (Narasimha Rao 2012): an odd sign's nine as
/// Parashara's, an even sign's in reverse, from the ninth back to the
/// first.
const KRISHNA_MISHRA_NAVAMSHA: [Group; 2] =
    [step(SignBase::Cyclic, 1, 0), step(SignBase::Cyclic, 11, 8)];

/// A named variant's row.
const fn variant(
    varga: Varga,
    divisions: u16,
    variant: VargaVariant,
    classifier: Classifier,
    groups: &'static [Group],
) -> Scheme {
    Scheme {
        varga: Some(varga),
        variant,
        divisions,
        classifier,
        groups,
    }
}

/// The named variants the SDK ships beside [`SCHEMES`], each a row for the
/// one evaluator; `VargaVariant::serves` names exactly these charts, which
/// the crate's own test holds.
///
/// ```
/// use teistro_core::settings::VargaVariant;
/// use teistro_vargas::scheme::VARIANTS;
///
/// assert!(VARIANTS.iter().all(|row| row.variant != VargaVariant::Parashara));
/// ```
pub const VARIANTS: [Scheme; 8] = [
    variant(
        Varga::D2,
        2,
        VargaVariant::Parivritti,
        Classifier::All,
        &CONTINUOUS,
    ),
    variant(
        Varga::D2,
        2,
        VargaVariant::Kashinatha,
        Classifier::Sign,
        &KASHINATHA_HORA,
    ),
    variant(
        Varga::D2,
        2,
        VargaVariant::UmaShambhu,
        Classifier::Parity,
        &UMA_SHAMBHU_HORA,
    ),
    variant(
        Varga::D3,
        3,
        VargaVariant::Parivritti,
        Classifier::All,
        &CONTINUOUS,
    ),
    variant(
        Varga::D3,
        3,
        VargaVariant::Somanatha,
        Classifier::Sign,
        &SOMANATHA_DREKKANA,
    ),
    variant(
        Varga::D4,
        4,
        VargaVariant::Parivritti,
        Classifier::All,
        &CONTINUOUS,
    ),
    variant(
        Varga::D5,
        5,
        VargaVariant::Parivritti,
        Classifier::All,
        &CONTINUOUS,
    ),
    variant(
        Varga::D9,
        9,
        VargaVariant::KrishnaMishra,
        Classifier::Parity,
        &KRISHNA_MISHRA_NAVAMSHA,
    ),
];

/// The twenty-one divisional charts the SDK ships, in catalogue order.
///
/// ```
/// use teistro_core::catalogue::Varga;
/// use teistro_vargas::SCHEMES;
///
/// assert_eq!(SCHEMES.len(), Varga::ALL.len());
/// assert_eq!(SCHEMES[8].key(), "D9");
/// ```
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
    ///
    /// ```
    /// use teistro_core::catalogue::Varga;
    /// use teistro_vargas::Scheme;
    ///
    /// assert_eq!(Scheme::of(Varga::D60).divisions, 60);
    /// ```
    #[must_use]
    pub fn of(varga: Varga) -> Scheme {
        SCHEMES
            .iter()
            .copied()
            .find(|scheme| scheme.varga == Some(varga))
            .unwrap_or(SCHEMES[0])
    }

    /// A catalogued chart under a named scheme: its BPHS ch. 6 row for
    /// `PARASHARA`, or the named variant's row.
    ///
    /// ```
    /// use teistro_core::catalogue::{Rashi, Varga};
    /// use teistro_core::error::Status;
    /// use teistro_core::settings::VargaVariant;
    /// use teistro_vargas::{Scheme, sign};
    ///
    /// // Narasimha Rao's worked example: Jupiter at 25°16′ Taurus is in
    /// // Venus's day-strong sign, Libra, in Kashinatha's hora.
    /// let hora = Scheme::named(Varga::D2, VargaVariant::Kashinatha)?;
    /// let jupiter = teistro_core::angle::Nas::try_from_degrees(30.0 + 25.0 + 16.0 / 60.0)?;
    /// assert_eq!(sign(&hora, jupiter), Rashi::Libra);
    /// // A scheme that is not the chart's is refused by the field.
    /// let refused = Scheme::named(Varga::D9, VargaVariant::Kashinatha).unwrap_err();
    /// assert_eq!((refused.status, refused.field()), (Status::InvalidArg, Some("vargas.schemes.D9.which")));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a scheme that is not one of the chart's, naming
    /// `vargas.schemes.<chart>.which` and the chart's schemes.
    pub fn named(varga: Varga, which: VargaVariant) -> Result<Scheme, Error> {
        if which == VargaVariant::Parashara {
            return Ok(Scheme::of(varga));
        }
        VargaChoice::Named { which }.check(varga)?;
        VARIANTS
            .iter()
            .copied()
            .find(|row| row.varga == Some(varga) && row.variant == which)
            .ok_or_else(|| {
                Error::internal(format!(
                    "{} serves {} and has no row",
                    which.key(),
                    varga.key()
                ))
            })
    }

    /// A catalogued chart computed by a consumer's own rows, checked as the
    /// settings check them ([`Rows::check`]): one group a classifier group,
    /// spans filling a sign, one listed sign a part, every sign in 0 to 11.
    ///
    /// A scheme holds its rule as `'static` rows, as the shipped ones are,
    /// so a consumer's rows are kept for the life of the process, **once per
    /// distinct rule**: a context opened a thousand times over the same rows
    /// keeps one copy. Past [`MOST_RULES`] distinct rules the call is
    /// refused rather than growing without bound.
    ///
    /// ```
    /// use teistro_core::catalogue::{Rashi, Varga};
    /// use teistro_core::varga_rows::{Classifier, RowGroup, RowMap, RowSpans, Rows, SignBase};
    /// use teistro_vargas::{Scheme, sign};
    ///
    /// // A drekkana counted from the sign's opposite: a rule no text gives.
    /// let rows = Rows {
    ///     classifier: Classifier::All,
    ///     groups: vec![RowGroup { spans: RowSpans::Equal, map: RowMap::Step { base: SignBase::Same, step: 4, offset: 6 } }],
    /// };
    /// let own = Scheme::from_rows(Varga::D3, &rows)?;
    /// assert_eq!(sign(&own, teistro_core::angle::Nas::try_from_degrees(1.0)?), Rashi::Libra);
    /// // The same rows are the same rule, kept once.
    /// assert!(core::ptr::eq(own.groups, Scheme::from_rows(Varga::D3, &rows)?.groups));
    /// assert_eq!(own.rows(), rows);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The check's refusal, naming the field under `vargas.schemes.<chart>`;
    /// `LIMIT` past [`MOST_RULES`] distinct rules.
    pub fn from_rows(varga: Varga, rows: &Rows) -> Result<Scheme, Error> {
        let divisions = varga.attributes().divisions;
        rows.check(divisions, &format!("vargas.schemes.{}", varga.key()))?;
        Ok(Scheme {
            varga: Some(varga),
            variant: VargaVariant::Custom,
            divisions,
            classifier: rows.classifier,
            groups: intern(rows)?,
        })
    }

    /// The scheme a chart is computed by under `settings`: the one
    /// `vargas.schemes` names for it, or BPHS ch. 6's where it names none.
    /// Every reader of a divisional chart under a context asks here, so a
    /// choice reaches the chart document, its drawing, its sign changes,
    /// the rules over it and the strengths alike.
    ///
    /// ```
    /// use teistro_core::catalogue::Varga;
    /// use teistro_core::settings::{VargaChoice, VargaVariant};
    /// use teistro_vargas::Scheme;
    ///
    /// let mut settings = teistro_chart::sample::resolved().settings;
    /// assert_eq!(Scheme::chosen(Varga::D3, &settings)?, Scheme::of(Varga::D3));
    /// settings.vargas.schemes.insert(Varga::D3, VargaChoice::Named { which: VargaVariant::Somanatha });
    /// assert_eq!(Scheme::chosen(Varga::D3, &settings)?.variant, VargaVariant::Somanatha);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Scheme::named`] and [`Scheme::from_rows`]; resolved settings
    /// have passed the same checks, so only settings built by hand fail.
    pub fn chosen(varga: Varga, settings: &Settings) -> Result<Scheme, Error> {
        match settings.vargas.schemes.get(&varga) {
            None => Ok(Scheme::of(varga)),
            Some(VargaChoice::Named { which }) => Scheme::named(varga, *which),
            Some(VargaChoice::Custom { classifier, groups }) => Scheme::from_rows(
                varga,
                &Rows {
                    classifier: *classifier,
                    groups: groups.clone(),
                },
            ),
        }
    }

    /// The rule as owned rows: what a consumer would write to compute this
    /// chart as a `CUSTOM` scheme, and what [`Scheme::from_rows`] reads.
    ///
    /// ```
    /// use teistro_core::catalogue::Varga;
    /// use teistro_vargas::Scheme;
    ///
    /// // Every shipped row is rows a consumer could have written.
    /// let d30 = Scheme::of(Varga::D30);
    /// assert_eq!(Scheme::from_rows(Varga::D30, &d30.rows())?.groups, d30.groups);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn rows(&self) -> Rows {
        Rows {
            classifier: self.classifier,
            groups: self.groups.iter().map(Group::rows).collect(),
        }
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
    /// ```
    /// use teistro_core::settings::UnattestedDn;
    /// use teistro_vargas::Scheme;
    ///
    /// assert_eq!(Scheme::unattested(37, UnattestedDn::Cyclic)?, Scheme::cyclic(37)?);
    /// assert!(Scheme::unattested(0, UnattestedDn::Cyclic).is_err());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
    /// ```
    /// use teistro_vargas::Scheme;
    ///
    /// let settings = teistro_chart::sample::resolved().settings;
    /// assert_eq!(Scheme::for_settings(37, &settings)?.key(), "D37");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
    /// ```
    /// use teistro_core::error::Status;
    /// use teistro_vargas::Scheme;
    ///
    /// let d37 = Scheme::cyclic(37)?;
    /// assert_eq!(d37.varga, None);
    /// assert_eq!(Scheme::cyclic(400).map_err(|error| error.status), Err(Status::OutOfRange));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
            variant: VargaVariant::Parashara,
            divisions,
            classifier: Classifier::All,
            groups: &CONTINUOUS,
        })
    }

    /// The group a sign belongs to.
    ///
    /// ```
    /// use teistro_core::catalogue::{Rashi, Varga};
    /// use teistro_vargas::Scheme;
    ///
    /// // The saptamsha's even signs count from the seventh.
    /// let d7 = Scheme::of(Varga::D7);
    /// assert_eq!(d7.group(Rashi::Taurus), &d7.groups[1]);
    /// ```
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
    ///
    /// ```
    /// use teistro_core::catalogue::{Rashi, Varga};
    /// use teistro_vargas::Scheme;
    ///
    /// assert_eq!(Scheme::of(Varga::D9).parts(Rashi::Aries), 9);
    /// // Called thirty, cut into five.
    /// assert_eq!(Scheme::of(Varga::D30).parts(Rashi::Aries), 5);
    /// ```
    #[must_use]
    pub fn parts(&self, sign: Rashi) -> u16 {
        self.group(sign).parts(self.divisions)
    }

    /// What the chart is called: `D9`, or `D37` for one the catalogue
    /// does not name.
    ///
    /// ```
    /// use teistro_core::catalogue::Varga;
    /// use teistro_vargas::Scheme;
    ///
    /// assert_eq!(Scheme::of(Varga::D9).key(), "D9");
    /// assert_eq!(Scheme::cyclic(37)?.key(), "D37");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
///
/// ```
/// use teistro_core::catalogue::{Rashi, Varga};
/// use teistro_vargas::Scheme;
/// use teistro_vargas::scheme::target;
///
/// let d9 = Scheme::of(Varga::D9);
/// // Aries's fourth navamsha is Cancer; Taurus's first is Capricorn.
/// assert_eq!(target(d9.group(Rashi::Aries), Rashi::Aries, 3, 9), Some(Rashi::Cancer));
/// assert_eq!(target(d9.group(Rashi::Taurus), Rashi::Taurus, 0, 9), Some(Rashi::Capricorn));
/// ```
#[must_use]
pub fn target(group: &Group, sign: Rashi, part: u16, divisions: u16) -> Option<Rashi> {
    let index = match group.map {
        Map::Step { base, step, offset } => {
            let from = base.multiplier(divisions) * u32::from(sign as u8);
            let along = u32::from(step) * u32::from(part);
            (from + along + u32::from(offset)) % u32::from(SIGNS)
        }
        Map::Listed { signs } => u32::from(*signs.get(usize::from(part))?),
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
///
/// ```
/// use teistro_core::angle::Nas;
/// use teistro_core::catalogue::{Rashi, Varga};
/// use teistro_vargas::Scheme;
/// use teistro_vargas::scheme::part_of;
///
/// let d30 = Scheme::of(Varga::D30);
/// // Twelve degrees of Aries is past the 5 and 5 into the 8.
/// let longitude = Nas::try_from_degrees(12.0)?;
/// assert_eq!(part_of(d30.group(Rashi::Aries), longitude, 30), 2);
/// // Exactly on a navamsha's edge, 10° of Aries, is the next one.
/// let d9 = Scheme::of(Varga::D9);
/// assert_eq!(part_of(d9.group(Rashi::Aries), Nas::try_from_degrees(10.0)?, 9), 3);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn part_of(group: &Group, longitude: Nas, divisions: u16) -> u16 {
    match group.spans {
        Spans::Equal => {
            let part = longitude.part(u32::from(divisions.max(1)));
            u16::try_from(part)
                .unwrap_or(0)
                .min(divisions.saturating_sub(1))
        }
        Spans::Degrees { widths } => {
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use teistro_core::settings::{UnattestedDn, VargaVariant};

    use super::{
        Classifier, Group, MOST_DIVISIONS, Map, SCHEMES, Scheme, SignBase, Spans, VARIANTS, listed,
        part_of, row, step, target, variant,
    };

    use teistro_core::angle::Nas;
    use teistro_core::catalogue::{Rashi, Varga};
    use teistro_core::quantity::Degrees;

    fn at(degrees: f64) -> Nas {
        Nas::from_degrees(Degrees::try_new(degrees).expect("a finite angle"))
    }

    /// The rows' builders run at compile time, where coverage cannot see
    /// them: each row and group built again at run time is the one compiled.
    #[test]
    fn every_row_built_at_run_time_is_the_row_compiled() {
        let rebuilt = |group: &Group| match (group.spans, group.map) {
            (
                Spans::Equal,
                Map::Step {
                    base,
                    step: by,
                    offset,
                },
            ) => step(base, by, offset),
            (Spans::Equal, Map::Listed { signs }) => listed(signs),
            _ => *group,
        };
        for scheme in SCHEMES.iter().chain(&VARIANTS) {
            let groups: Vec<Group> = scheme.groups.iter().map(rebuilt).collect();
            assert_eq!(groups, scheme.groups, "{:?}", scheme.varga);
            let chart = scheme.varga.unwrap();
            let built = match scheme.variant {
                VargaVariant::Parashara => {
                    row(chart, scheme.divisions, scheme.classifier, scheme.groups)
                }
                which => variant(
                    chart,
                    scheme.divisions,
                    which,
                    scheme.classifier,
                    scheme.groups,
                ),
            };
            assert_eq!(built, *scheme);
        }
    }

    /// A chart the catalogue does not name is written without one and read
    /// back as the cyclic rule.
    #[test]
    fn a_written_arbitrary_chart_reads_back_as_the_cyclic_rule() {
        let d7 = Scheme::cyclic(7).unwrap();
        let written = serde_json::to_value(d7).unwrap();
        let read: Scheme = serde_json::from_value(written).unwrap();
        assert_eq!((read, read.varga), (d7, None));
    }

    /// A written scheme reads back as this build's row, and one whose
    /// divisions or classifier disagree with the row it names is refused
    /// rather than silently replaced.
    #[test]
    fn a_written_scheme_must_agree_with_this_builds_row() {
        let written = serde_json::to_value(Scheme::of(Varga::D9)).unwrap();
        let read: Scheme = serde_json::from_value(written.clone()).unwrap();
        assert_eq!(read, Scheme::of(Varga::D9));

        let mut divisions = written.clone();
        divisions["divisions"] = serde_json::json!(10);
        let refused = serde_json::from_value::<Scheme>(divisions).unwrap_err();
        assert!(refused.to_string().contains("divides a sign"), "{refused}");

        let other = Varga::ALL
            .into_iter()
            .map(|varga| serde_json::to_value(Scheme::of(varga)).unwrap())
            .find(|value| value["classifier"] != written["classifier"])
            .expect("two charts sort their signs differently");
        let mut classifier = written;
        classifier["classifier"] = other["classifier"].clone();
        let refused = serde_json::from_value::<Scheme>(classifier).unwrap_err();
        assert!(refused.to_string().contains("sorts"), "{refused}");
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
                if let Map::Listed { signs } = group.map {
                    assert_eq!(
                        u16::try_from(signs.len()).unwrap(),
                        group.parts(scheme.divisions),
                        "{:?}",
                        scheme.varga
                    );
                    assert!(signs.iter().all(|sign| *sign < 12), "{:?}", scheme.varga);
                }
                if let Spans::Degrees { widths } = group.spans {
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
