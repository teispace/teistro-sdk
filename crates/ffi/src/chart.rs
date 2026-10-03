//! The chart foundation at the C boundary: the enums a founded chart's
//! blob carries that no other entry point needed, and the conversions
//! from the Rust values they mirror.
//!
//! Designed in `03-design/chart-at-the-boundary.md`. Five enums are
//! declared here rather than taken from the catalogue, because they are
//! not catalogue kinds: two belong to the chart layer (`Reading`,
//! `DayPart`) and three are settings knobs a result has to carry, since
//! a ghati count means nothing without the reckoning that produced it
//! and a chalit means nothing without the reading it was taken under.
//!
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]
//!
//! Two of them mirror a Rust enum through an **exhaustive** match, which
//! is what stops the two drifting: a variant added stops this crate
//! compiling rather than silently mapping to whatever was first.
//! `TsResolution` in `calendar.rs` is the pattern.
//!
//! The three knobs cannot do that. A settings knob is `#[non_exhaustive]`
//! on purpose, so a match on one needs a `_` arm and a member added
//! later would fall into it silently. They convert **fallibly** instead
//! and refuse a member this build does not know, and the guard is a test
//! over the knob's own `ALL`: adding a member fails it by name rather
//! than shipping a wrong id.

use core::ffi::c_char;

use teistro::dasha::DashaName;
use teistro::render_svg::Theme;
use teistro::{ChartRequest, PlanInputs, PlanRequest, RuleRequest, RuleSet};
use teistro_aspect::drishti::Strength;
use teistro_chart::bhava::Reading;
use teistro_chart::day::DayPart;
use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::{ChartKind, Kind, Varga};
use teistro_core::envelope::{Envelope, Provenance};
use teistro_core::error::{Error, Status};
use teistro_core::key::KeyId;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_houses::classify::Quadrant;
use teistro_idl::blob::{ColumnData, FixedValue, Writer};
use teistro_serial::Document;
use teistro_state::burn::Burning;

use crate::blob::TsBlob;
use crate::context::TsContext;
use crate::schemas::SignedBy;
use crate::string::TsString;
use crate::support::{c_struct, optional_text, read_in, slice, with_context, write_plain};
use teistro_core::settings::{GhatiReckoning, HoraReckoning, PolarDayPolicy, Sunrise};
use teistro_time::local_day::{DayState, PolarKind};

/// Which bound of a bhava a placement was read against.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsReading {
    /// From one sandhi to the next: the bhava as a span between cusps.
    Sandhi = 0,
    /// From one madhya to the next: the bhava as a span between centres.
    Madhya = 1,
}

impl From<Reading> for TsReading {
    fn from(reading: Reading) -> TsReading {
        match reading {
            Reading::Sandhi => TsReading::Sandhi,
            Reading::Madhya => TsReading::Madhya,
        }
    }
}

/// How badly the Sun burns a body.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsBurning {
    /// Far enough from the Sun to be itself.
    None = 0,
    /// Combust.
    Combust = 1,
    /// Deeply combust; only a table that gives a deeper orb reaches it.
    Deep = 2,
}

impl From<Burning> for TsBurning {
    fn from(burning: Burning) -> TsBurning {
        match burning {
            Burning::None => TsBurning::None,
            Burning::Combust => TsBurning::Combust,
            Burning::Deep => TsBurning::Deep,
        }
    }
}

/// Which third of the wheel a bhava stands in.
///
/// The houses crate's own `Quadrant`, which is not a catalogue member —
/// it is a classification of a number rather than a thing with a key —
/// so it crosses as this boundary's own enum, as `TsStrength` does.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsQuadrant {
    /// Angular: the 1st, 4th, 7th and 10th.
    Kendra = 0,
    /// Succedent: the 2nd, 5th, 8th and 11th.
    Panapara = 1,
    /// Cadent: the 3rd, 6th, 9th and 12th.
    Apoklima = 2,
}

impl From<Quadrant> for TsQuadrant {
    fn from(quadrant: Quadrant) -> TsQuadrant {
        match quadrant {
            Quadrant::Kendra => TsQuadrant::Kendra,
            Quadrant::Panapara => TsQuadrant::Panapara,
            Quadrant::Apoklima => TsQuadrant::Apoklima,
        }
    }
}

/// How strongly one body looks at another.
///
/// The aspect crate's own `Strength`, which is not a catalogue member —
/// it is a property of a relation rather than a thing with a key — so it
/// crosses as this boundary's own enum, as `TsReading` and `TsDayPart`
/// do.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsStrength {
    /// No aspect at all.
    None = 0,
    /// A quarter aspect: the third and tenth.
    Quarter = 1,
    /// A half aspect: the fifth and ninth.
    Half = 2,
    /// A three-quarter aspect: the fourth and eighth.
    ThreeQuarters = 3,
    /// A full aspect: the seventh, and a special graha's own two houses.
    Full = 4,
}

impl From<Strength> for TsStrength {
    fn from(strength: Strength) -> TsStrength {
        match strength {
            Strength::None => TsStrength::None,
            Strength::Quarter => TsStrength::Quarter,
            Strength::Half => TsStrength::Half,
            Strength::ThreeQuarters => TsStrength::ThreeQuarters,
            Strength::Full => TsStrength::Full,
        }
    }
}

/// How a dasha's balance at birth was measured.
///
/// The settings' own `Balance`, which is a knob and not a catalogue member,
/// so it crosses as this boundary's own enum, as `TsStrength` does.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsBalance {
    /// By the elapsed part of the Moon's window of nakshatras.
    Spatial = 0,
    /// By the elapsed part of the Moon's stay in its nakshatra.
    Temporal = 1,
}

impl From<teistro_core::settings::Balance> for TsBalance {
    fn from(balance: teistro_core::settings::Balance) -> TsBalance {
        match balance {
            teistro_core::settings::Balance::Temporal => TsBalance::Temporal,
            _ => TsBalance::Spatial,
        }
    }
}

/// Where an Ashtakavarga's reductions and pindas were made: the settings'
/// own `Shodhana`, which is a knob and not a catalogue member.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsShodhana {
    /// In each graha's own Ashtakavarga (BPHS chs. 67 to 69).
    EachGraha = 0,
    /// On the sum of the seven, as the conformance corpus's engine makes them.
    Sarva = 1,
}

impl From<teistro_core::settings::Shodhana> for TsShodhana {
    fn from(shodhana: teistro_core::settings::Shodhana) -> TsShodhana {
        match shodhana {
            teistro_core::settings::Shodhana::Sarva => TsShodhana::Sarva,
            _ => TsShodhana::EachGraha,
        }
    }
}

/// How an Ashtakavarga's Ekadhipatya reduction treated a co-ruled sign beside
/// an occupied one: the settings' own `Ekadhipatya`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsEkadhipatya {
    /// BPHS ch. 68: an empty sign keeps a difference.
    Bphs = 0,
    /// The empty sign always goes to zero.
    EmptyToZero = 1,
}

impl From<teistro_core::settings::Ekadhipatya> for TsEkadhipatya {
    fn from(rule: teistro_core::settings::Ekadhipatya) -> TsEkadhipatya {
        match rule {
            teistro_core::settings::Ekadhipatya::EmptyToZero => TsEkadhipatya::EmptyToZero,
            _ => TsEkadhipatya::Bphs,
        }
    }
}

/// How a Vimshopaka scored a graha in a varga: the settings' own
/// `Vimshopaka`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsVimshopakaScoring {
    /// BPHS ch. 7: 20 in exaltation or the own sign, else by the compound
    /// relationship with the sign's lord.
    Bphs = 0,
    /// The conformance corpus's engine: the Saptavargaja virupas over 45 by
    /// natural friendship, rounded to hundredths.
    SaptavargajaVirupas = 1,
}

impl From<teistro_core::settings::Vimshopaka> for TsVimshopakaScoring {
    fn from(scoring: teistro_core::settings::Vimshopaka) -> TsVimshopakaScoring {
        match scoring {
            teistro_core::settings::Vimshopaka::SaptavargajaVirupas => {
                TsVimshopakaScoring::SaptavargajaVirupas
            }
            _ => TsVimshopakaScoring::Bphs,
        }
    }
}

/// Which rule a chart's Brahma graha was sought under: the settings' own
/// `jaimini.brahma` (`03-design/jaimini-significators.md`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsBrahmaRule {
    /// BPHS ch. 46 vv. 170 to 173 as the Sanskrit states them.
    Verses = 0,
    /// The translator's note after v. 173.
    TranslatorsNote = 1,
}

impl From<teistro_core::settings::BrahmaRule> for TsBrahmaRule {
    fn from(rule: teistro_core::settings::BrahmaRule) -> TsBrahmaRule {
        match rule {
            teistro_core::settings::BrahmaRule::TranslatorsNote => TsBrahmaRule::TranslatorsNote,
            _ => TsBrahmaRule::Verses,
        }
    }
}

/// Whether a chart's Brahma graha was found, and when not, why (C127,
/// C128). One code rather than a presence flag beside a reason, so the two
/// cannot disagree.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsBrahmaOutcome {
    /// Found: `brahma` names it.
    Found = 0,
    /// Under the verses, no lord of the 6th, 8th or 12th stands in an odd
    /// sign behind the sign counted from, and they give no fallback.
    NoLordQualifies = 1,
    /// Saturn or a node qualified, and no planet stands in the 6th sign from
    /// it to take its place.
    NoPlanetInTheSixth = 2,
    /// Under the translator's note, no planet stands in the 8th and none in
    /// an odd sign within the six signs behind.
    NoPlanetQualifies = 3,
}

impl From<Option<teistro::dasha::jaimini::NoBrahma>> for TsBrahmaOutcome {
    fn from(none: Option<teistro::dasha::jaimini::NoBrahma>) -> TsBrahmaOutcome {
        use teistro::dasha::jaimini::NoBrahma;
        match none {
            None => TsBrahmaOutcome::Found,
            Some(NoBrahma::NoLordQualifies) => TsBrahmaOutcome::NoLordQualifies,
            Some(NoBrahma::NoPlanetInTheSixth) => TsBrahmaOutcome::NoPlanetInTheSixth,
            Some(NoBrahma::NoPlanetQualifies) => TsBrahmaOutcome::NoPlanetQualifies,
        }
    }
}

/// What a gochar reading counted its houses from (crux C139).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsGocharFrom {
    /// The natal Moon's sign, Phaladeepika ch. 26 v. 1's.
    Moon = 0,
    /// The natal lagna's sign.
    Lagna = 1,
}

impl TsGocharFrom {
    /// The code a reference crosses as; `None` for one this boundary
    /// does not know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(from: teistro::GocharFrom) -> Option<TsGocharFrom> {
        match from {
            teistro::GocharFrom::Moon => Some(TsGocharFrom::Moon),
            teistro::GocharFrom::Lagna => Some(TsGocharFrom::Lagna),
            _ => None,
        }
    }
}

/// What Sade Sati's houses are reckoned in (C147, `03-design/sade-sati.md`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsReckoning {
    /// Whole signs from the reference's sign.
    Sign = 0,
    /// 30° houses with the reference's degree in the middle of the first.
    Degree = 1,
}

impl TsReckoning {
    /// The code a reckoning crosses as; `None` for one this boundary does
    /// not know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(reckoning: teistro::sade_sati::Reckoning) -> Option<TsReckoning> {
        match reckoning {
            teistro::sade_sati::Reckoning::Sign => Some(TsReckoning::Sign),
            teistro::sade_sati::Reckoning::Degree => Some(TsReckoning::Degree),
            _ => None,
        }
    }
}

/// Whether a chart is of the day or of the night
/// (`03-design/essential-dignities.md` §Sect).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsSect {
    /// A day chart.
    Day = 0,
    /// A night chart.
    Night = 1,
}

impl From<teistro::Sect> for TsSect {
    fn from(sect: teistro::Sect) -> TsSect {
        match sect {
            teistro::Sect::Day => TsSect::Day,
            teistro::Sect::Night => TsSect::Night,
        }
    }
}

/// How a chart's sect is read (C209, `03-design/essential-dignities.md`
/// §Sect).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsSectRule {
    /// The Sun's centre above the true horizon, Valens's hemisphere.
    Horizon = 0,
    /// The chart's own sunrise to sunset.
    Daylight = 1,
    /// Every chart read as a day chart.
    Day = 2,
    /// Every chart read as a night chart.
    Night = 3,
}

impl TsSectRule {
    /// The code a rule crosses as; `None` for one this boundary does not
    /// know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(rule: teistro::SectRule) -> Option<TsSectRule> {
        match rule {
            teistro::SectRule::Horizon => Some(TsSectRule::Horizon),
            teistro::SectRule::Daylight => Some(TsSectRule::Daylight),
            teistro::SectRule::Day => Some(TsSectRule::Day),
            teistro::SectRule::Night => Some(TsSectRule::Night),
            _ => None,
        }
    }
}

/// Which system of terms a reading used (C208,
/// `03-design/essential-dignities.md`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsTerms {
    /// The Egyptian terms, as Ptolemy transmits them.
    Egyptian = 0,
    /// Ptolemy's own terms as Lilly prints them.
    PtolemaicLilly = 1,
    /// Ptolemy's own terms as Ashmand translates them.
    PtolemaicAshmand = 2,
    /// The Chaldean terms, by the chart's sect.
    Chaldean = 3,
    /// The table the request's `dignities_json` gave.
    Table = 4,
}

impl TsTerms {
    /// The code a system crosses as; `None` for one this boundary does not
    /// know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(terms: &teistro::Terms) -> Option<TsTerms> {
        match terms {
            teistro::Terms::Egyptian => Some(TsTerms::Egyptian),
            teistro::Terms::PtolemaicLilly => Some(TsTerms::PtolemaicLilly),
            teistro::Terms::PtolemaicAshmand => Some(TsTerms::PtolemaicAshmand),
            teistro::Terms::Chaldean => Some(TsTerms::Chaldean),
            teistro::Terms::Table(_) => Some(TsTerms::Table),
            _ => None,
        }
    }
}

/// Who rules each triplicity (`03-design/essential-dignities.md`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsTriplicities {
    /// Ptolemy's, Mars ruling water with Venus and the Moon.
    Ptolemy = 0,
    /// Lilly's, Mars ruling water alone.
    Lilly = 1,
}

impl TsTriplicities {
    /// The code a scheme crosses as; `None` for one this boundary does
    /// not know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(triplicities: teistro::Triplicities) -> Option<TsTriplicities> {
        match triplicities {
            teistro::Triplicities::Ptolemy => Some(TsTriplicities::Ptolemy),
            teistro::Triplicities::Lilly => Some(TsTriplicities::Lilly),
            _ => None,
        }
    }
}

/// One of Lilly's accidental fortitudes or debilities (p. 115,
/// `03-design/essential-dignities.md` §Accidental fortitudes).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsAccident {
    /// Moving forward; void for the Sun and the Moon.
    Direct = 0,
    /// Moving backward.
    Retrograde = 1,
    /// Faster than its mean motion.
    Swift = 2,
    /// Slower than its mean motion.
    Slow = 3,
    /// Rising before the Sun. Not read for the Moon.
    Oriental = 4,
    /// Setting after the Sun.
    Occidental = 5,
    /// The Moon from her conjunction with the Sun to the opposition.
    Increasing = 6,
    /// The Moon from the opposition to the conjunction.
    Decreasing = 7,
    /// Clear of the Sun: neither combust, under his beams nor in cazimi.
    FreeFromCombustion = 8,
    /// In the heart of the Sun.
    Cazimi = 9,
    /// Within the combustion orb of the Sun.
    Combust = 10,
    /// Within the beams of the Sun, not combust.
    UnderBeams = 11,
    /// In partile conjunction with Jupiter or Venus.
    ConjunctBenefic = 12,
    /// In partile conjunction with the North Node.
    ConjunctNorthNode = 13,
    /// In partile trine to Jupiter or Venus.
    TrineBenefic = 14,
    /// In partile sextile to Jupiter or Venus.
    SextileBenefic = 15,
    /// In partile conjunction with Saturn or Mars.
    ConjunctMalefic = 16,
    /// In partile conjunction with the South Node.
    ConjunctSouthNode = 17,
    /// In partile opposition to Saturn or Mars.
    OpposedMalefic = 18,
    /// In partile square to Saturn or Mars.
    SquareMalefic = 19,
    /// Between the bodies of Saturn and Mars.
    Besieged = 20,
    /// With Cor Leonis (Regulus).
    Regulus = 21,
    /// With Spica.
    Spica = 22,
    /// With Caput Algol.
    Algol = 23,
}

impl TsAccident {
    /// The code an accident crosses as; `None` for one this boundary does
    /// not know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(accident: teistro::Accident) -> Option<TsAccident> {
        use teistro::Accident as A;
        Some(match accident {
            A::Direct => TsAccident::Direct,
            A::Retrograde => TsAccident::Retrograde,
            A::Swift => TsAccident::Swift,
            A::Slow => TsAccident::Slow,
            A::Oriental => TsAccident::Oriental,
            A::Occidental => TsAccident::Occidental,
            A::Increasing => TsAccident::Increasing,
            A::Decreasing => TsAccident::Decreasing,
            A::FreeFromCombustion => TsAccident::FreeFromCombustion,
            A::Cazimi => TsAccident::Cazimi,
            A::Combust => TsAccident::Combust,
            A::UnderBeams => TsAccident::UnderBeams,
            A::ConjunctBenefic => TsAccident::ConjunctBenefic,
            A::ConjunctNorthNode => TsAccident::ConjunctNorthNode,
            A::TrineBenefic => TsAccident::TrineBenefic,
            A::SextileBenefic => TsAccident::SextileBenefic,
            A::ConjunctMalefic => TsAccident::ConjunctMalefic,
            A::ConjunctSouthNode => TsAccident::ConjunctSouthNode,
            A::OpposedMalefic => TsAccident::OpposedMalefic,
            A::SquareMalefic => TsAccident::SquareMalefic,
            A::Besieged => TsAccident::Besieged,
            A::Regulus => TsAccident::Regulus,
            A::Spica => TsAccident::Spica,
            A::Algol => TsAccident::Algol,
            _ => return None,
        })
    }
}

/// When two planets are in partile aspect (C216,
/// `03-design/essential-dignities.md` §Accidental fortitudes).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsPartile {
    /// Lilly's: the same degree of signs the aspect apart.
    SameDegree = 0,
    /// Within an orb of the exact aspect.
    Within = 1,
}

impl TsPartile {
    /// The code a reading crosses as, with its orb (0 for
    /// [`TsPartile::SameDegree`]); `None` for one this boundary does not
    /// know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(partile: teistro::Partile) -> Option<(TsPartile, f64)> {
        match partile {
            teistro::Partile::SameDegree => Some((TsPartile::SameDegree, 0.0)),
            teistro::Partile::Within { orb_deg } => Some((TsPartile::Within, orb_deg)),
            _ => None,
        }
    }
}

/// When a planet is besieged by Saturn and Mars (C215,
/// `03-design/essential-dignities.md` §Accidental fortitudes).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsSiege {
    /// Lilly's example: all three in one sign, the planet between the two.
    SameSign = 0,
    /// On the shorter arc between the two, the arc no wider than a span.
    Within = 1,
}

impl TsSiege {
    /// The code a reading crosses as, with its span (0 for
    /// [`TsSiege::SameSign`]); `None` for one this boundary does not know
    /// yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(siege: teistro::Siege) -> Option<(TsSiege, f64)> {
        match siege {
            teistro::Siege::SameSign => Some((TsSiege::SameSign, 0.0)),
            teistro::Siege::Within { span_deg } => Some((TsSiege::Within, span_deg)),
            _ => None,
        }
    }
}

/// What of a place an almuten's dignities are counted from (C218,
/// `03-design/essential-dignities.md` §The almuten).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsPlaceReading {
    /// The degree: house, exaltation, triplicity, term and face.
    Degree = 0,
    /// The sign: house, exaltation and triplicity.
    Sign = 1,
}

impl TsPlaceReading {
    /// The code a reading crosses as; `None` for one this boundary does
    /// not know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(reading: teistro::PlaceReading) -> Option<TsPlaceReading> {
        match reading {
            teistro::PlaceReading::Degree => Some(TsPlaceReading::Degree),
            teistro::PlaceReading::Sign => Some(TsPlaceReading::Sign),
            _ => None,
        }
    }
}

/// How the Part of Fortune is taken by night (C220,
/// `03-design/essential-dignities.md` §The almuten).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsFortuneRule {
    /// Lilly's: the ascendant plus the Moon less the Sun, by day or night.
    DayAndNight = 0,
    /// By night, the ascendant plus the Sun less the Moon.
    ReversedByNight = 1,
    /// Valens's own: by night, reversed while the Moon is above the
    /// horizon, and counted from the Sun once it has set.
    ReversedWhileMoonUp = 2,
}

impl TsFortuneRule {
    /// The code a rule crosses as; `None` for one this boundary does not
    /// know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(rule: teistro::FortuneRule) -> Option<TsFortuneRule> {
        match rule {
            teistro::FortuneRule::DayAndNight => Some(TsFortuneRule::DayAndNight),
            teistro::FortuneRule::ReversedByNight => Some(TsFortuneRule::ReversedByNight),
            teistro::FortuneRule::ReversedWhileMoonUp => Some(TsFortuneRule::ReversedWhileMoonUp),
            _ => None,
        }
    }
}

/// One of Valens's lots (`03-design/hellenistic-lots.md`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsLot {
    /// Fortune: from the Sun to the Moon, from the ascendant.
    Fortune = 0,
    /// Daimon: Fortune reflected in the ascendant.
    Daimon = 1,
    /// Basis: the shorter arc between Fortune and Daimon.
    Basis = 2,
    /// Love: from Fortune to Daimon by day.
    Love = 3,
    /// Necessity: from Daimon to Fortune by day.
    Necessity = 4,
    /// Exaltation: from the Sun to its exaltation by day, the Moon to its
    /// by night.
    Exaltation = 5,
    /// Debt: from Mercury to Saturn.
    Debt = 6,
    /// Theft: from Mercury to Mars by day, counted from Saturn.
    Theft = 7,
    /// Deceit: from the Sun to Mars by day.
    Deceit = 8,
    /// Foreign lands: from Saturn to Mars.
    ForeignLands = 9,
    /// The father: from the Sun to Saturn by day, Venus to the Moon by
    /// night.
    Father = 10,
    /// Marriage: from Jupiter to Venus by day.
    Marriage = 11,
    /// Brothers: from Saturn to Jupiter by day.
    Brothers = 12,
    /// The crisis-producing place: from Saturn to Mars by day.
    Crisis = 13,
}

impl TsLot {
    /// The code a lot crosses as; `None` for one this boundary does not
    /// know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(lot: teistro::Lot) -> Option<TsLot> {
        use teistro::Lot;
        Some(match lot {
            Lot::Fortune => TsLot::Fortune,
            Lot::Daimon => TsLot::Daimon,
            Lot::Basis => TsLot::Basis,
            Lot::Love => TsLot::Love,
            Lot::Necessity => TsLot::Necessity,
            Lot::Exaltation => TsLot::Exaltation,
            Lot::Debt => TsLot::Debt,
            Lot::Theft => TsLot::Theft,
            Lot::Deceit => TsLot::Deceit,
            Lot::ForeignLands => TsLot::ForeignLands,
            Lot::Father => TsLot::Father,
            Lot::Marriage => TsLot::Marriage,
            Lot::Brothers => TsLot::Brothers,
            Lot::Crisis => TsLot::Crisis,
            _ => return None,
        })
    }
}

/// Why a horary figure is radical (Lilly p. 121,
/// `03-design/hellenistic-considerations.md`); a figure's grounds cross as
/// a bit set, bit `n` the member with code `n`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsRadicalGround {
    /// The lord of the hour and the lord of the Ascendant are one planet.
    OneLord = 0,
    /// The lord of the hour rules the rising sign's triplicity.
    Triplicity = 1,
    /// The two lords share a temperament.
    Nature = 2,
}

impl TsRadicalGround {
    /// The code a ground crosses as; `None` for one this boundary does not
    /// know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(ground: teistro::RadicalGround) -> Option<TsRadicalGround> {
        use teistro::RadicalGround;
        Some(match ground {
            RadicalGround::OneLord => TsRadicalGround::OneLord,
            RadicalGround::Triplicity => TsRadicalGround::Triplicity,
            RadicalGround::Nature => TsRadicalGround::Nature,
            _ => return None,
        })
    }
}

/// A set of up to eight members as a byte, bit `n` the member with id `n`;
/// `None` for a member past the eighth.
fn bit_set(ids: impl IntoIterator<Item = u16>) -> Option<u8> {
    ids.into_iter().try_fold(0_u8, |set, id| {
        let bit = 1_u8.checked_shl(u32::from(id))?;
        Some(set | bit)
    })
}

/// A Ptolemaic aspect the Moon perfects before judgement
/// (`03-design/hellenistic-considerations.md`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsPtolemaicAspect {
    /// 0°.
    Conjunction = 0,
    /// 60°.
    Sextile = 1,
    /// 90°.
    Square = 2,
    /// 120°.
    Trine = 3,
    /// 180°.
    Opposition = 4,
}

impl TsPtolemaicAspect {
    /// The code an aspect crosses as; `None` for one this boundary does
    /// not know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(aspect: teistro::PtolemaicAspect) -> Option<TsPtolemaicAspect> {
        use teistro::PtolemaicAspect;
        Some(match aspect {
            PtolemaicAspect::Conjunction => TsPtolemaicAspect::Conjunction,
            PtolemaicAspect::Sextile => TsPtolemaicAspect::Sextile,
            PtolemaicAspect::Square => TsPtolemaicAspect::Square,
            PtolemaicAspect::Trine => TsPtolemaicAspect::Trine,
            PtolemaicAspect::Opposition => TsPtolemaicAspect::Opposition,
            _ => return None,
        })
    }
}

/// Which of Lilly's three kinds an application is (p. 107,
/// `03-design/hellenistic-perfection.md`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsApplicationKind {
    /// A swifter planet to a slower, both direct.
    BothDirect = 0,
    /// Both retrograde, "an ill Application".
    BothRetrograde = 1,
    /// One direct and one retrograde, meeting.
    AgainstRetrograde = 2,
}

impl TsApplicationKind {
    /// The code a kind crosses as; `None` for one this boundary does not
    /// know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(kind: teistro::ApplicationKind) -> Option<TsApplicationKind> {
        use teistro::ApplicationKind;
        Some(match kind {
            ApplicationKind::BothDirect => TsApplicationKind::BothDirect,
            ApplicationKind::BothRetrograde => TsApplicationKind::BothRetrograde,
            ApplicationKind::AgainstRetrograde => TsApplicationKind::AgainstRetrograde,
            _ => return None,
        })
    }
}

/// What stops or hinders two significators' application (pp. 110–113,
/// `03-design/hellenistic-perfection.md`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsImpedimentKind {
    /// A third planet comes to a significator first.
    Prohibition = 0,
    /// A significator comes to a third planet first.
    Frustration = 1,
    /// A significator stations before the perfection its motion promises.
    Refranation = 2,
}

impl TsImpedimentKind {
    /// The code a kind crosses as; `None` for one this boundary does not
    /// know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(kind: teistro::ImpedimentKind) -> Option<TsImpedimentKind> {
        use teistro::ImpedimentKind;
        Some(match kind {
            ImpedimentKind::Prohibition => TsImpedimentKind::Prohibition,
            ImpedimentKind::Frustration => TsImpedimentKind::Frustration,
            ImpedimentKind::Refranation => TsImpedimentKind::Refranation,
            _ => return None,
        })
    }
}

/// One of Lilly's seven ways a matter is perfected (pp. 125–127,
/// `03-design/hellenistic-perfection.md`); the ways a figure holds cross
/// as a bit set, bit `n` the member with code `n`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsWay {
    /// The significators' conjunction, unhindered.
    Conjunction = 0,
    /// Their sextile or trine, unhindered.
    SextileOrTrine = 1,
    /// Their square, each in some dignity at its degree.
    Square = 2,
    /// Their opposition, with mutual reception by house and the Moon's
    /// relay.
    Opposition = 3,
    /// A translation of light, received by house, triplicity or term.
    Translation = 4,
    /// A collection of light, the collector in a dignity of each.
    Collection = 5,
    /// The quesited's significator in the Ascendant, the Moon translating.
    Dwelling = 6,
}

impl TsWay {
    /// The code a way crosses as; `None` for one this boundary does not
    /// know yet, which the encoder refuses rather than guessing.
    #[must_use]
    pub const fn of(way: teistro::Way) -> Option<TsWay> {
        use teistro::Way;
        Some(match way {
            Way::Conjunction => TsWay::Conjunction,
            Way::SextileOrTrine => TsWay::SextileOrTrine,
            Way::Square => TsWay::Square,
            Way::Opposition => TsWay::Opposition,
            Way::Translation => TsWay::Translation,
            Way::Collection => TsWay::Collection,
            Way::Dwelling => TsWay::Dwelling,
            _ => return None,
        })
    }
}

/// What a hit of the transit hit list was (`03-design/transit-hit-list.md`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsHitKind {
    /// The graha entered a sign.
    SignIngress = 0,
    /// The graha entered a nakshatra.
    NakshatraIngress = 1,
    /// The graha stood still in longitude.
    Station = 2,
    /// The graha aspected a natal point, or came within or left its orb.
    Aspect = 3,
}

impl From<&teistro::gochar::hits::HitEvent> for TsHitKind {
    fn from(event: &teistro::gochar::hits::HitEvent) -> TsHitKind {
        use teistro::gochar::hits::HitEvent;
        match event {
            HitEvent::SignIngress { .. } => TsHitKind::SignIngress,
            HitEvent::NakshatraIngress { .. } => TsHitKind::NakshatraIngress,
            HitEvent::Station { .. } => TsHitKind::Station,
            HitEvent::Aspect { .. } => TsHitKind::Aspect,
        }
    }
}

/// Which way a graha was moving, through a line or out of a station.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsMotion {
    /// Forward through the zodiac.
    Direct = 0,
    /// Backward.
    Retrograde = 1,
}

impl From<teistro::gochar::hits::Motion> for TsMotion {
    fn from(motion: teistro::gochar::hits::Motion) -> TsMotion {
        use teistro::gochar::hits::Motion;
        match motion {
            Motion::Direct => TsMotion::Direct,
            Motion::Retrograde => TsMotion::Retrograde,
        }
    }
}

/// A Western aspect, one of Leo's nine (`03-design/western-aspects.md`).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsWesternAspect {
    /// 0°.
    Conjunction = 0,
    /// 30°.
    SemiSextile = 1,
    /// 45°.
    SemiSquare = 2,
    /// 60°.
    Sextile = 3,
    /// 90°.
    Square = 4,
    /// 120°.
    Trine = 5,
    /// 135°.
    Sesquiquadrate = 6,
    /// 150°.
    Quincunx = 7,
    /// 180°.
    Opposition = 8,
}

impl TsWesternAspect {
    /// The code an aspect crosses as, or none for a member added to the
    /// non-exhaustive `WesternAspect` before it was given one here; the
    /// spelling gate (`tests/keys.rs`) fails on that member until it is.
    #[must_use]
    pub const fn of(aspect: teistro::WesternAspect) -> Option<TsWesternAspect> {
        use teistro::WesternAspect;
        Some(match aspect {
            WesternAspect::Conjunction => TsWesternAspect::Conjunction,
            WesternAspect::SemiSextile => TsWesternAspect::SemiSextile,
            WesternAspect::SemiSquare => TsWesternAspect::SemiSquare,
            WesternAspect::Sextile => TsWesternAspect::Sextile,
            WesternAspect::Square => TsWesternAspect::Square,
            WesternAspect::Trine => TsWesternAspect::Trine,
            WesternAspect::Sesquiquadrate => TsWesternAspect::Sesquiquadrate,
            WesternAspect::Quincunx => TsWesternAspect::Quincunx,
            WesternAspect::Opposition => TsWesternAspect::Opposition,
            _ => return None,
        })
    }
}

/// Where in an aspect's window a hit falls (C146).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsAspectPhase {
    /// The transit came within the orb.
    Entering = 0,
    /// The aspect is exact.
    Exact = 1,
    /// The transit passed out of the orb.
    Leaving = 2,
}

impl From<teistro::gochar::hits::AspectPhase> for TsAspectPhase {
    fn from(phase: teistro::gochar::hits::AspectPhase) -> TsAspectPhase {
        use teistro::gochar::hits::AspectPhase;
        match phase {
            AspectPhase::Entering => TsAspectPhase::Entering,
            AspectPhase::Exact => TsAspectPhase::Exact,
            AspectPhase::Leaving => TsAspectPhase::Leaving,
        }
    }
}

/// The nodes' vedha in transit, the settings' `gochar.node_vedha` (C136).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsNodeVedha {
    /// The Sun's vedha pairs.
    LikeTheSun = 0,
    /// None: nothing obstructs a node's transit.
    None = 1,
}

impl TsNodeVedha {
    /// The code a reading crosses as; `None` for one this boundary does not
    /// know yet.
    #[must_use]
    pub const fn of(vedha: teistro_core::settings::NodeVedha) -> Option<TsNodeVedha> {
        use teistro_core::settings::NodeVedha;
        match vedha {
            NodeVedha::LikeTheSun => Some(TsNodeVedha::LikeTheSun),
            NodeVedha::None => Some(TsNodeVedha::None),
            _ => None,
        }
    }
}

/// Whom the nodes obstruct in transit, the settings' `gochar.node_obstruction`
/// (C137, C140).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsNodeObstruction {
    /// The seven, and not each other.
    NotEachOther = 0,
    /// Every graha, each other too: the verses read literally.
    EachOtherToo = 1,
    /// Nobody: only the seven obstruct.
    None = 2,
}

impl TsNodeObstruction {
    /// The code a reading crosses as; `None` for one this boundary does not
    /// know yet.
    #[must_use]
    pub const fn of(
        obstruction: teistro_core::settings::NodeObstruction,
    ) -> Option<TsNodeObstruction> {
        use teistro_core::settings::NodeObstruction;
        match obstruction {
            NodeObstruction::NotEachOther => Some(TsNodeObstruction::NotEachOther),
            NodeObstruction::EachOtherToo => Some(TsNodeObstruction::EachOtherToo),
            NodeObstruction::None => Some(TsNodeObstruction::None),
            _ => None,
        }
    }
}

/// What a graha's transit comes to (Phaladeepika ch. 26 vv. 2 to 8).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsGocharVerdict {
    /// In a good house, and nothing stands in its vedha house.
    Good = 0,
    /// In a good house, and another graha stands in its vedha house.
    Obstructed = 1,
    /// Not in a house v. 2 names good.
    NotGood = 2,
}

impl From<teistro::gochar::Verdict> for TsGocharVerdict {
    fn from(verdict: teistro::gochar::Verdict) -> TsGocharVerdict {
        use teistro::gochar::Verdict;
        match verdict {
            Verdict::Good => TsGocharVerdict::Good,
            Verdict::Obstructed => TsGocharVerdict::Obstructed,
            Verdict::NotGood => TsGocharVerdict::NotGood,
        }
    }
}

/// Where in a sign a graha's transit bears fruit (v. 25).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsFruition {
    /// The first ten degrees: the Sun and Mars.
    First = 0,
    /// The middle ten: Jupiter and Venus.
    Middle = 1,
    /// The last ten: the Moon and Saturn.
    Last = 2,
    /// The whole sign: Mercury and the nodes.
    Throughout = 3,
}

impl From<teistro::gochar::Fruition> for TsFruition {
    fn from(fruition: teistro::gochar::Fruition) -> TsFruition {
        use teistro::gochar::Fruition;
        match fruition {
            Fruition::First => TsFruition::First,
            Fruition::Middle => TsFruition::Middle,
            Fruition::Last => TsFruition::Last,
            Fruition::Throughout => TsFruition::Throughout,
        }
    }
}

/// How many bindus make a transit good, the settings'
/// `gochar.ashtakavarga_good_from` (C141).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsAshtakavargaGoodFrom {
    /// Five: Phaladeepika ch. 23 v. 11 makes four a fear.
    Five = 0,
    /// Four.
    Four = 1,
}

impl TsAshtakavargaGoodFrom {
    /// The code a reading crosses as; `None` for one this boundary does not
    /// know yet.
    #[must_use]
    pub const fn of(
        good_from: teistro_core::settings::AshtakavargaGoodFrom,
    ) -> Option<TsAshtakavargaGoodFrom> {
        use teistro_core::settings::AshtakavargaGoodFrom;
        match good_from {
            AshtakavargaGoodFrom::Five => Some(TsAshtakavargaGoodFrom::Five),
            AshtakavargaGoodFrom::Four => Some(TsAshtakavargaGoodFrom::Four),
            _ => None,
        }
    }
}

/// Who lords an eighth of a sign (Phaladeepika ch. 23 vv. 18 and 19).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsKakshyaLord {
    /// The first eighth.
    Saturn = 0,
    /// The second.
    Jupiter = 1,
    /// The third.
    Mars = 2,
    /// The fourth.
    Sun = 3,
    /// The fifth.
    Venus = 4,
    /// The sixth.
    Mercury = 5,
    /// The seventh.
    Moon = 6,
    /// The last.
    Lagna = 7,
}

impl From<teistro::gochar::ashtakavarga::KakshyaLord> for TsKakshyaLord {
    fn from(lord: teistro::gochar::ashtakavarga::KakshyaLord) -> TsKakshyaLord {
        use teistro::gochar::ashtakavarga::KakshyaLord;
        match lord {
            KakshyaLord::Saturn => TsKakshyaLord::Saturn,
            KakshyaLord::Jupiter => TsKakshyaLord::Jupiter,
            KakshyaLord::Mars => TsKakshyaLord::Mars,
            KakshyaLord::Sun => TsKakshyaLord::Sun,
            KakshyaLord::Venus => TsKakshyaLord::Venus,
            KakshyaLord::Mercury => TsKakshyaLord::Mercury,
            KakshyaLord::Moon => TsKakshyaLord::Moon,
            KakshyaLord::Lagna => TsKakshyaLord::Lagna,
        }
    }
}

/// Where a sign's sarvashtakavarga stands against v. 20's 28.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsSarvaStanding {
    /// More than 28.
    Above = 0,
    /// Exactly 28, which the verse does not judge (C142).
    Even = 1,
    /// Fewer than 28.
    Below = 2,
}

impl From<teistro::gochar::ashtakavarga::SarvaStanding> for TsSarvaStanding {
    fn from(standing: teistro::gochar::ashtakavarga::SarvaStanding) -> TsSarvaStanding {
        use teistro::gochar::ashtakavarga::SarvaStanding;
        match standing {
            SarvaStanding::Above => TsSarvaStanding::Above,
            SarvaStanding::Even => TsSarvaStanding::Even,
            SarvaStanding::Below => TsSarvaStanding::Below,
        }
    }
}

/// Where in a dasha a graha's effects are felt (BPHS ch. 47 vv. 3 and 4).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsDashaPhase {
    /// At its commencement.
    Commencement = 0,
    /// In its middle.
    Middle = 1,
    /// At its end.
    End = 2,
}

impl From<teistro::strength::DashaPhase> for TsDashaPhase {
    fn from(phase: teistro::strength::DashaPhase) -> TsDashaPhase {
        match phase {
            teistro::strength::DashaPhase::Commencement => TsDashaPhase::Commencement,
            teistro::strength::DashaPhase::Middle => TsDashaPhase::Middle,
            teistro::strength::DashaPhase::End => TsDashaPhase::End,
        }
    }
}

/// Which arc of its day an instant falls in.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsDayPart {
    /// Between sunrise and sunset.
    Daylight = 0,
    /// Between sunset and the next sunrise.
    Night = 1,
}

impl From<DayPart> for TsDayPart {
    fn from(part: DayPart) -> TsDayPart {
        match part {
            DayPart::Daylight => TsDayPart::Daylight,
            DayPart::Night => TsDayPart::Night,
        }
    }
}

/// Which sunrise a day was reckoned from.
///
/// The named conventions only. A profile may ask for the centre of the
/// disc at a chosen altitude instead, which is a `Custom` convention;
/// a blob carries that as its altitude beside this, because a variant
/// with a payload cannot be an id (`03-design/chart-at-the-boundary.md`
/// §8).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsSunrise {
    /// The centre of the disc on the geometric horizon.
    CentreNoRefraction = 0,
    /// The upper limb with refraction.
    UpperLimbRefraction = 1,
    /// The lower limb with refraction.
    LowerLimbRefraction = 2,
    /// The centre of the disc with standard refraction.
    CentreRefraction = 3,
}

impl TsSunrise {
    /// The id this build gives a member, or `None` for one it does
    /// not know — a member added to the knob since this was written,
    /// which is refused by name rather than defaulted.
    #[must_use]
    pub fn of(sunrise: Sunrise) -> Option<TsSunrise> {
        match sunrise {
            Sunrise::CentreNoRefraction => Some(TsSunrise::CentreNoRefraction),
            Sunrise::UpperLimbRefraction => Some(TsSunrise::UpperLimbRefraction),
            Sunrise::LowerLimbRefraction => Some(TsSunrise::LowerLimbRefraction),
            Sunrise::CentreRefraction => Some(TsSunrise::CentreRefraction),
            _ => None,
        }
    }
}

/// How the sixty ghatis of a day are measured.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsGhatiReckoning {
    /// Twenty-four minutes each, from sunrise.
    Civil = 0,
    /// Thirty over the actual daylight and thirty over the actual night.
    Proportional = 1,
}

impl TsGhatiReckoning {
    /// The id this build gives a member, or `None` for one it does
    /// not know — a member added to the knob since this was written,
    /// which is refused by name rather than defaulted.
    #[must_use]
    pub fn of(reckoning: GhatiReckoning) -> Option<TsGhatiReckoning> {
        match reckoning {
            GhatiReckoning::Civil => Some(TsGhatiReckoning::Civil),
            GhatiReckoning::Proportional => Some(TsGhatiReckoning::Proportional),
            _ => None,
        }
    }
}

/// How the twenty-four horas of a day are measured.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsHoraReckoning {
    /// Twelve over the daylight and twelve over the night.
    Proportional = 0,
    /// Twenty-four of sixty minutes, from sunrise.
    Equal = 1,
}

impl TsHoraReckoning {
    /// The id this build gives a member, or `None` for one it does
    /// not know — a member added to the knob since this was written,
    /// which is refused by name rather than defaulted.
    #[must_use]
    pub fn of(reckoning: HoraReckoning) -> Option<TsHoraReckoning> {
        match reckoning {
            HoraReckoning::Proportional => Some(TsHoraReckoning::Proportional),
            HoraReckoning::Equal => Some(TsHoraReckoning::Equal),
            _ => None,
        }
    }
}

/// Whether a day had a sunrise, and what was done when it had not.
///
/// The kind half of a tagged enum: a polar day carries which polar
/// state it was and which policy was applied, in `state_polar_kind` and
/// `state_polar_policy` beside it, because a variant with a payload
/// cannot be an id (`03-design/chart-at-the-boundary.md` §8).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsDayState {
    /// Sunrise and sunset occurred; the two fields beside this are zero.
    Normal = 0,
    /// No horizon crossing, and the policy synthesised the bounds.
    Polar = 1,
}

/// Which polar state a day without a sunrise was in.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsPolarKind {
    /// The Sun stayed up.
    Day = 0,
    /// The Sun stayed down.
    Night = 1,
}

impl From<PolarKind> for TsPolarKind {
    fn from(kind: PolarKind) -> TsPolarKind {
        match kind {
            PolarKind::Day => TsPolarKind::Day,
            PolarKind::Night => TsPolarKind::Night,
        }
    }
}

/// What the settings say a day without a sunrise is.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsPolarDayPolicy {
    /// An undefined state: the day has no bounds.
    Undefined = 0,
    /// The nearest rise or set stands in for the missing one.
    NearestEvent = 1,
    /// Civil midnight stands in for it.
    CivilMidnight = 2,
}

impl TsPolarDayPolicy {
    /// The id this build gives a member, or `None` for one it does not
    /// know — a member added to the knob since this was written, which
    /// is refused by name rather than defaulted.
    #[must_use]
    pub fn of(policy: PolarDayPolicy) -> Option<TsPolarDayPolicy> {
        match policy {
            PolarDayPolicy::Undefined => Some(TsPolarDayPolicy::Undefined),
            PolarDayPolicy::NearestEvent => Some(TsPolarDayPolicy::NearestEvent),
            PolarDayPolicy::CivilMidnight => Some(TsPolarDayPolicy::CivilMidnight),
            _ => None,
        }
    }
}

impl TsDayState {
    /// A day's state split into the three scalars a blob carries: the
    /// kind, and the polar kind and policy that only a polar day has.
    #[must_use]
    pub fn split(state: DayState) -> (TsDayState, u8, u8) {
        match state {
            DayState::Normal => (TsDayState::Normal, 0, 0),
            DayState::Polar { kind, policy } => (
                TsDayState::Polar,
                TsPolarKind::from(kind) as u8,
                TsPolarDayPolicy::of(policy).map_or(0, |p| p as u8),
            ),
        }
    }
}

/// What a chart is founded on: when, where, what kind, and the clock its
/// day is reckoned in.
///
/// Everything else is the context's settings, which is what makes two
/// calls under one context comparable and what the settings hash is for.
/// The clock is here because nothing else knows it: a chart's day runs
/// from a local sunrise and its date is a civil date, and a longitude
/// gives local *mean* time rather than a civil offset
/// (`03-design/chart-at-the-boundary.md` §5).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TsChartRequest {
    /// `sizeof(ts_chart_request)` as the caller compiled it.
    pub struct_size: u32,
    /// What kind of chart to found.
    /// `api: enum=ChartKind example=0`
    pub kind: u16,
    /// Reserved; write zero.
    pub reserved: u16,
    /// The instants, as Julian days on the UTC scale: one chart each.
    ///
    /// A grid, not a scalar, because the founder shares the settings and
    /// the solar model across a batch and a rectification pass wants a
    /// hundred charts (`03-design/chart-at-the-boundary.md` §3a). A
    /// caller wanting one passes a grid of one, as `ts_positions` takes
    /// a grid of one instant.
    /// `api: len=instant_count unit=jd`
    pub instants: *const f64,
    /// How many instants `instants` points at.
    pub instant_count: usize,
    /// The place's latitude, degrees north.
    /// `api: unit=deg range=[-90,90] example=27.7172`
    pub latitude_deg: f64,
    /// The place's longitude, degrees east.
    /// `api: unit=deg range=[-180,180] example=85.324`
    pub longitude_deg: f64,
    /// The place's altitude, metres above the ellipsoid.
    /// `api: unit=m range=[-500,9000] example=1400`
    pub altitude_m: f64,
    /// The local clock's offset from UTC in seconds, east positive: the
    /// clock the day's date is read in.
    /// `api: unit=s range=[-64800,64800] example=20700`
    pub utc_offset_seconds: i32,
    /// Reserved; write zero.
    pub reserved_tail: i32,
    /// Which of the document's sections to compute beside the
    /// foundation, as a bit set: 1 the day's almanac, 2 the planetary
    /// states, 4 the aspects, 8 the derived points, 16 the houses
    /// service, 32 the Ashtakavarga, 64 the Vimshopaka, 128 the Shadbala, 256 the Bhava bala, 512 the Vaiseshikamsa, 1024 the dasha phala. Zero for the foundation alone, which is what every
    /// caller compiled against an earlier header passes by not passing
    /// it at all.
    ///
    /// A bit set here and a named option in every ergonomic layer, which
    /// is the split `ts_frame_pack` already has: nothing but a generated
    /// layer writes bits (`03-design/chart-reading.md` §5).
    /// `api: example=0`
    pub sections: u32,
    /// Reserved; write zero.
    pub reserved_sections: u32,
    /// Which divisional charts to compute, as catalogue ids, in the
    /// order they should be answered in; null with a count of zero for
    /// none, as `instants` takes a grid of none.
    ///
    /// **Not `nullable`**, and that is the description's word rather
    /// than a promise about the pointer: `nullable` makes the generated
    /// field an `Option` of the whole parameter, and an optional *array
    /// of enum members* is a shape no emitter has been shown — it mapped
    /// the option's contents where it meant to map the array's. An empty
    /// array says "none" without needing one, which is what `instants`
    /// already does.
    /// `api: len=varga_count enum=Varga`
    pub vargas: *const u16,
    /// How many divisional charts `vargas` points at.
    pub varga_count: usize,
    /// Which charts to draw, and in which layouts, in the order they should
    /// be answered in: each `layout_id << 16 | varga_id`, a `chart_layout`
    /// catalogue id and a `Varga` id, `D1` for the founded chart. Null with a
    /// count of zero for none.
    ///
    /// Packed, as `sections` is a bit set, so the request carries one array
    /// and one count rather than two arrays that must agree; every ergonomic
    /// layer takes named pairs and writes the bits (`03-design/chart-geometry.md`).
    /// `api: len=drawing_count`
    pub drawings: *const u32,
    /// How many drawings `drawings` points at.
    pub drawing_count: usize,
    /// Which dashas to compute, in the order they should be answered in: each
    /// a `DashaSystem` catalogue id, or the id `ts_key_parse` gives a system
    /// the context registered (`0x8000` and up). Each one's balance and its
    /// periods to its depth. Null with a count of zero for none.
    ///
    /// Ids and not an enum, as `drawings` carries layout ids: every ergonomic
    /// layer takes a catalogue member or a registered key and writes the id.
    /// `api: len=dasha_count`
    pub dashas: *const u16,
    /// How many dashas `dashas` points at.
    pub dasha_count: usize,
    /// A theme to write every drawing as SVG in, as JSON: an object of
    /// `style` and `content` naming only what it changes, over the light
    /// theme or the shipped one its `extends` names (`{"extends": "DARK"}`).
    /// The SVGs come back in the blob's `svgs` section, in the context's
    /// locale. Null for none, which costs nothing
    /// (`03-design/render-svg.md`).
    /// `api: nullable example={"extends":"DARK"}`
    pub theme_json: *const c_char,
    /// Rules to answer over every chart, as JSON: `shipped` names the
    /// kernel's sets, `rules` a consumer's own in the rule format, with
    /// `readings`, `houses` and `longevity` choosing what else comes back
    /// (`03-design/rules-at-the-boundary.md`). The answers come back in the
    /// blob's `rules` section, and the sections the rules read are computed
    /// whether or not `sections` asked for them. Null for none, which costs
    /// nothing.
    /// `api: nullable example={"shipped":["NABHASAS"]}`
    pub rules_json: *const c_char,
    /// Narrative plans to compose over every chart, as JSON: an object
    /// naming the composers to run — `placements`, `readings`, `strength`,
    /// `houses`, `positions`, `aspects`, `conditions` and `karakas` — each
    /// false by default. The plans come back in the blob's `plans`
    /// section, holding no words at all — an item's params are the JSON
    /// `ts_intl_render` takes, so a binding says one by handing it
    /// straight back, in any locale and in as many as it likes
    /// (`03-design/plans-at-the-boundary.md`). `readings` says what the
    /// rules answered, so it needs `rules_json` beside it; the sections
    /// the other composers read are computed for them, as the rules' are.
    /// Null for none, which costs nothing.
    /// `api: nullable example={"placements":true}`
    pub interpret_json: *const c_char,
    /// The annual charts to answer for every chart in the batch, as a JSON
    /// object: `reading` — `"SIDEREAL"` (the tradition's), `"TROPICAL"`
    /// (the Western solar return) or `"MEAN"` (a whole sidereal year each
    /// time) — and `through`, the last year of life wanted, 1 to 200. The
    /// instants come back in the `praveshas` section, ragged by
    /// `cast.pravesha_count`; an ephemeris that ends first answers fewer
    /// than asked for rather than refusing. Null for none
    /// (`03-design/annual-chart.md`). Refusals are named from the record
    /// every binding calls `varsha`, as `varsha.through`.
    /// `api: nullable`
    pub varsha_json: *const c_char,
    /// The transits to read against every chart in the batch, as a JSON
    /// object: `instants`, UTC Julian days, at least one; `from` —
    /// `"MOON"` (Phaladeepika ch. 26 v. 1's, the default) or `"LAGNA"`; and
    /// `ashtakavarga`, true to judge the seven by the natal bindus too, in
    /// the `gochar_ashtakavarga` section.
    /// Each chart's readings come back in the `gochar` section, a row an
    /// instant, and its grahas in `gochar_grahas`, under the settings'
    /// `gochar` group. Null for none (`03-design/gochar.md`). Refusals are
    /// named from the record every binding calls `gochar`, as
    /// `gochar.instants`.
    /// `api: nullable example={"instants":[2460676.5],"from":"MOON"}`
    pub gochar_json: *const c_char,
    /// The transit hit list to search against every chart in the batch,
    /// as a JSON object: the window `from` and `to`, UTC Julian days, and
    /// optionally `grahas` (keys, the nine by default), `kinds`
    /// (`"SIGN_INGRESS"`, `"NAKSHATRA_INGRESS"`, `"STATION"`, `"ASPECT"`;
    /// all by default), `points` (the natal points aspected: a graha's key
    /// or `"LAGNA"`, or an answer's `to`; the nine and the lagna by
    /// default), `aspects` (angles, whole degrees to 180; 0 and 180 by
    /// default, C145) and `orbDeg` (more than 0, under 15 and under half
    /// the step between the aspects' lines, for the windows' edges; exact
    /// only by default, C146).
    /// Each chart's hits come back in the `hits` section, `cast.hit_count`
    /// rows a chart, the sky searched once for the batch. Null for none
    /// (`03-design/transit-hit-list.md`). Refusals are named from the
    /// record every binding calls `hits`, as `hits.to`.
    /// `api: nullable example={"from":2460676.5,"to":2461041.5,"grahas":["SATURN"]}`
    pub hits_json: *const c_char,
    /// Sade Sati and the smaller spells of Saturn to find for every chart
    /// in the batch, as a JSON object: `from`, a UTC Julian day, and
    /// optionally `to` (the window's end, `from` by default), `countedFrom`
    /// (`"MOON"`, the default, or `"LAGNA"`; C139), `reckoning` (`"SIGN"`,
    /// the default, or `"DEGREE"`; C147) and `spells` (houses 3 to 11, the
    /// 4th and the 8th by default; C149). Every period reaching into the
    /// window comes back whole in the `sade_sati` and `sade_sati_visits`
    /// sections, the sky searched once for the batch. Null for none
    /// (`03-design/sade-sati.md`). Refusals are named from the record every
    /// binding calls `sadeSati`, as `sadeSati.to`.
    /// `api: nullable example={"from":2460676.5,"to":2464329.0,"reckoning":"SIGN"}`
    pub sade_sati_json: *const c_char,
    /// Every chart read as KP (Krishnamurti Paddhati), as a JSON object,
    /// every member optional: `number`, the querent's horary number 1 to
    /// 249, which casts the cusps from it (C156); `clock`, seconds east of
    /// UT that the civil day lord is the weekday on, this request's own
    /// when absent (C151); and `anyAyanamsha`, true to read a chart whose
    /// zodiac is not Krishnamurti's (C157). Each chart's reading — its
    /// cusps and planets to the sub-sub lord, its significators and the
    /// ruling planets of its moment — comes back in the `kp` section,
    /// under the settings' `kp` group. Null for none, which costs nothing
    /// (`03-design/kp.md`). Refusals are named from the record every
    /// binding calls `kp`, as `kp.number`.
    /// `api: nullable example={"number":74}`
    pub kp_json: *const c_char,
    /// Every chart's essential dignities, as a JSON object, every member
    /// optional: `sectRule` (`"HORIZON"`, the Sun's centre above the true
    /// horizon and the default; `"DAYLIGHT"`, the chart's own sunrise to
    /// sunset; or `"DAY"` or `"NIGHT"` outright; C209), `rules`
    /// (`{terms, triplicities}`: the terms `"PTOLEMAIC_LILLY"`, the
    /// default, `"EGYPTIAN"`, `"PTOLEMAIC_ASHMAND"`, `"CHALDEAN"` or
    /// `{"TABLE": [...]}`, twelve signs of five `{lord, end}` from Aries;
    /// the triplicities `"LILLY"`, the default, or `"PTOLEMY"`; C208) and
    /// `scores` (`house`, `exaltation`, `triplicity`, `term`, `face`,
    /// `detriment`, `fall`, `peregrine`, Lilly's by default). The sect and
    /// what was applied come back in the `dignities` section and the seven
    /// planets in `dignity_planets`. Null for none, which costs nothing
    /// (`03-design/essential-dignities.md`). Refusals are named from the
    /// record every binding calls `dignities`, as `dignities.sectRule`.
    /// `api: nullable example={"sectRule":"HORIZON","rules":{"terms":"EGYPTIAN"}}`
    pub dignities_json: *const c_char,
    /// Every chart's accidental fortitudes beside its essential dignities
    /// (Lilly, p. 115), as a JSON object, every member optional:
    /// `dignities` (the record `dignities_json` takes), `rules`
    /// (`combustionDeg` 8.5, `combustionInSign` true, `beamsDeg` 17,
    /// `cazimiDeg` 17′, `cuspOrbDeg` 5, `starOrbDeg` 5, `partile`
    /// `"SAME_DEGREE"` or `{"WITHIN": {"orbDeg": …}}` (C216), `siege`
    /// `"SAME_SIGN"` or `{"WITHIN": {"spanDeg": …}}` (C215) and
    /// `meanMotionDeg`, seven in the Chaldean order) and `scores` (the
    /// twelve `houses` and each line by name), Lilly's by default. The
    /// essential half comes back in the sections `dignities_json` fills, so
    /// asking for both is refused; the accidental half in `fortitudes`,
    /// `fortitude_houses`, `fortitude_planets` and `fortitude_accidents`.
    /// The houses are Regiomontanus's unless a profile names another
    /// division for the `hellenistic` module. Null for none, which costs
    /// nothing (`03-design/essential-dignities.md` §Accidental fortitudes).
    /// Refusals are named from the record every binding calls
    /// `fortitudes`, as `fortitudes.rules.beamsDeg`.
    /// `api: nullable example={"rules":{"partile":{"WITHIN":{"orbDeg":1}}},"scores":{"regulus":5}}`
    pub fortitudes_json: *const c_char,
    /// Every chart's lots, all fourteen Valens gives, as a JSON object,
    /// every member optional: `sectRule` (the record `dignities_json`
    /// names it in, Valens's `"HORIZON"` by default) and `fortune`, how
    /// the Part of Fortune is taken by night: `"REVERSED_BY_NIGHT"`
    /// (Valens II.22, the default), `"DAY_AND_NIGHT"` (Lilly) or
    /// `"REVERSED_WHILE_MOON_UP"` (Valens III.11, C221). What was applied
    /// comes back in the `lots` section and the fourteen in `lot_places`.
    /// Null for none, which costs nothing
    /// (`03-design/hellenistic-lots.md`). Refusals are named from the
    /// record every binding calls `lots`, as `lots.fortune`.
    /// `api: nullable example={"fortune":"REVERSED_WHILE_MOON_UP"}`
    pub lots_json: *const c_char,
    /// Every chart's considerations before judgement (Lilly, *Christian
    /// Astrology* I.XIX), as a JSON object, every member optional:
    /// `moonLateFromDeg` (27 by default, C229) and `orbsDeg`, the seven
    /// whole orbs in the Chaldean order whose halves make an application
    /// (Lilly's p. 107 by default, C230). The fortitudes they read are
    /// `fortitudes_json`'s, or Lilly's when it is null. The clauses come
    /// back in `considerations`, the Moon's two perfections in
    /// `consideration_perfections` and the orbs applied in
    /// `consideration_orbs`. Null for none, which costs nothing
    /// (`03-design/hellenistic-considerations.md`). Refusals are named from
    /// the record every binding calls `considerations`, as
    /// `considerations.moonLateFromDeg`.
    /// `api: nullable example={"moonLateFromDeg":25}`
    pub considerations_json: *const c_char,
    /// Whether a horary matter is brought to pass (Lilly, *Christian
    /// Astrology* pp. 107–113 and 125–127), as a JSON object:
    /// `querent` and `quesited`, the two significators by key, or
    /// `house`, the house of the matter, whose cusp's lord signifies the
    /// quesited, the querent's being the Ascendant's lord unless named;
    /// and `rules`, every member optional: `orbsDeg` (Lilly's p. 107),
    /// `horizonDays` (unset, until the swifter significator leaves its
    /// sign, C232) and `withinSign` (true: a third planet's contact counts
    /// only before the applier leaves its sign, C234). The houses and dignities it weighs are
    /// `fortitudes_json`'s, or Lilly's when it is null; the timeline is
    /// searched on the ephemeris. The relations come back in
    /// `perfection`, `perfection_impediments`, `perfection_translations`
    /// and `perfection_collections`, and the orbs applied in
    /// `perfection_orbs`. Null for none, which costs nothing
    /// (`03-design/hellenistic-perfection.md`). Refusals are named from
    /// the record every binding calls `perfection`, as
    /// `perfection.quesited`.
    /// `api: nullable example={"house":7}`
    pub perfection_json: *const c_char,
    /// The progressions to read every chart's birth through, as a JSON
    /// object, every field optional but one of `at` and `contacts`: `at`,
    /// the instant of life (a UTC Julian day) the progressed chart and the
    /// direction are read for; `rate` (`{"sky": "DAY", "life": "YEAR"}` by
    /// default; a span is `"DAY"`, `"SYNODIC_MONTH"`, `"SIDEREAL_MONTH"`,
    /// `"YEAR"` or `{"DAYS": n}`); `year` (`"TROPICAL"` by default,
    /// `"JULIAN"`, or Leo's `"NOON_SIDEREAL_TIME"`, C236); `angles` (how
    /// the progressed midheaven moves, `"NAIBOD_RIGHT_ASCENSION"` by
    /// default, C237); `direction` (`"SOLAR"` by default, `"NAIBOD"`,
    /// `"PTOLEMY"` or `{"PER_YEAR": degrees}`); and `contacts`, a window of
    /// life `{from, to}` with the progressed `grahas` (the seven by
    /// default), the radical `points` (the seven and the lagna) and the
    /// `aspects` (Leo's table, p. 48), spelled as `hits_json` spells them.
    /// The progressed chart is founded at the request's place. The answers
    /// come back in `progressions`, `progressed_grahas`, `directed_grahas`
    /// and `progressed_contacts`. Null for none, which costs nothing
    /// (`03-design/western-progressions.md`). Refusals are named from the
    /// record every binding calls `progressions`, as `progressions.year`.
    /// `api: nullable example={"at":2460676.5}`
    pub progressions_json: *const c_char,
    /// Every chart's Western aspect table, as a JSON object, every field
    /// optional: `aspects`, the keys looked for (`"CONJUNCTION"`,
    /// `"SEMI_SEXTILE"`, `"SEMI_SQUARE"`, `"SEXTILE"`, `"SQUARE"`,
    /// `"TRINE"`, `"SESQUIQUADRATE"`, `"QUINCUNX"`, `"OPPOSITION"`; Leo's
    /// nine when left out), and `orbs`, the model: `{"model": "LEO"}` by
    /// default (C240), `{"model": "MOIETIES", "orbs": [{"graha": "SUN",
    /// "orbDeg": 17}, …]}`, or `{"model": "BY_ASPECT", "orbs": [{"aspect":
    /// "TRINE", "orbDeg": 6}, …]}`. The pairs are the chart's planets: the
    /// seven, and the outer three when `TS_CHART_OUTER` placed them. The
    /// answers come back in `western_aspects` and `western_aspect_rows`.
    /// Null for none, which costs nothing (`03-design/western-aspects.md`).
    /// Refusals are named from the record every binding calls
    /// `westernAspects`, as `westernAspects.orbs.orbs`.
    /// `api: nullable example={"aspects":["TRINE","SQUARE"]}`
    pub western_aspects_json: *const c_char,
    /// Every chart's synastry with one partner, as a JSON object:
    /// `partner`, the second birth, `{"instant": jd, "place": {"latitude",
    /// "longitude", "altitude"}, "utcOffsetSeconds"}`, founded once under
    /// the context's settings with the outer planets when
    /// `TS_CHART_OUTER` placed them; and beside it, every field optional,
    /// `aspects` and `orbs` as `western_aspects_json` spells them, `lagna`
    /// (true: each side's lagna is read beside its planets, C242),
    /// `zodiac` (`"TROPICAL"`, the default, or `"CHARTS"`, C241) and
    /// `parallels` (`{"orbDeg": 1}` as `parallels_json` spells it: the
    /// parallels across the two, none when left out) and `antiscia`
    /// (`{"orbs": {"model": "LEO"}}` as `antiscia_json` spells it: the antiscia
    /// across the two, none when left out), `composite` (true: each chart's
    /// composite with the partner, C247) and `davison` (true: each chart's
    /// Davison birth with the partner, the chart's read on this request's
    /// clock, C248). Each chart is read against the
    /// partner, the chart's point first. The answers come back in
    /// `synastry`, `synastry_rows`, `synastry_parallel_rows`,
    /// `synastry_antiscion_rows`, `synastry_composites`,
    /// `synastry_composite_rows` and `synastry_davisons`. Null for none,
    /// which costs nothing
    /// (`03-design/western-synastry.md`). Refusals are
    /// named from the record every binding calls `synastry`, as
    /// `synastry.partner.place.latitude`.
    /// `api: nullable example={"partner":{"instant":2403113.4993,"place":{"latitude":51.5058,"longitude":-0.1878,"altitude":0}}}`
    pub synastry_json: *const c_char,
    /// Every chart's declinations and the parallels among its planets, as
    /// a JSON object, every field optional: `orbDeg`, how close two
    /// distances from the equator must stand, Leo's 1° by default and at
    /// most 10°. A pair on either side of the equator is a parallel
    /// (C243). The pairs are the chart's planets: the seven, and the outer
    /// three when `TS_CHART_OUTER` placed them. The answers come back in
    /// `declinations`, `declination_rows` and `parallel_rows`. Null for
    /// none, which costs nothing (`03-design/western-declinations.md`).
    /// Refusals are named from the record every binding calls
    /// `parallels`, as `parallels.orbDeg`.
    /// `api: nullable example={"orbDeg":1}`
    pub parallels_json: *const c_char,
    /// Every chart's antiscia, as a JSON object, every field optional:
    /// `orbs`, as `western_aspects_json` spells them, read at the
    /// conjunction, Lilly's moieties by default (C244). Each planet is
    /// reflected about the solstices and the equinoxes from its tropical
    /// longitude, and a pair whose longitudes sum to 180° or 0° within the
    /// orb stands in antiscion or contrantiscion. The planets are the
    /// seven, and the outer three when `TS_CHART_OUTER` placed them; one
    /// the orbs give none is reflected and stands in no pair. The answers
    /// come back in `antiscia`, `antiscion_points` and `antiscion_rows`.
    /// Null for none, which costs nothing
    /// (`03-design/western-antiscia.md`). Refusals are named from the
    /// record every binding calls `antiscia`, as `antiscia.orbs.orbs`.
    /// `api: nullable example={"orbs":{"model":"LEO"}}`
    pub antiscia_json: *const c_char,
    /// Every chart's equal distances, as a JSON object, every field
    /// optional: `orbDeg`, how far from the axis through two planets'
    /// midpoint a third may stand, 0.5° by default (C245) and at most 10°.
    /// A planet stands on the axis when it is equally distant from the
    /// two, on the shorter arc's midpoint or opposite it (C246). The
    /// planets are the seven, and the outer three when `TS_CHART_OUTER`
    /// placed them. The answers come back in `midpoints` and
    /// `midpoint_rows`. Null for none, which costs nothing
    /// (`03-design/western-midpoints.md`). Refusals are named from the
    /// record every binding calls `midpoints`, as `midpoints.orbDeg`.
    /// `api: nullable example={"orbDeg":1}`
    pub midpoints_json: *const c_char,
}

// **The handshake, which this struct carried and nothing read.**
// `struct_size` is documented as "`sizeof(ts_chart_request)` as the caller compiled
// it", and the entry point below dereferenced the pointer raw: a caller
// compiled against an older header passed a shorter struct and the
// library read past it, which is undefined behaviour rather than the
// `SCHEMA_VERSION` refusal the field exists to give. Eleven of the
// thirteen boundary structs with the field were registered here; these
// two were not, and they are the two biggest requests.
// `check-lints`' `handshake-is-checked` holds the class now.
c_struct!(TsChartRequest);

/// Which document sections a `sections` bit set asks for, as the
/// request's own vocabulary.
///
/// **The bits are the boundary's and the names are the façade's**, and
/// the translation is here rather than in `teistro` on purpose: these
/// five values are in `teistro.h` and are therefore an ABI, while the
/// façade's own set is an implementation detail that must stay free to
/// change. A `Reading` is built by naming what is wanted, which is what
/// makes an unknown bit a silent no rather than a wrong section — and
/// what makes the table below the only place the mapping is written
/// (`03-design/chart-reading.md` §5).
type SectionBit = (u32, fn(ChartRequest) -> ChartRequest);

// The bits a caller sets in a chart request's `sections`. Declared here
// beside the table and described as constants, so the header names them:
// the bits are the boundary's vocabulary, a consumer of the C ABI writes
// `TS_CHART_ASPECTS`, and every generated layer writes a named option
// instead (`03-design/chart-reading.md` §5).

/// A chart request's `sections` bit: the day's almanac.
///
/// `api: constant`
pub const TS_CHART_PANCHANGA: u32 = 1;
/// A chart request's `sections` bit: the planetary states.
///
/// `api: constant`
pub const TS_CHART_STATE: u32 = 2;
/// A chart request's `sections` bit: the drishti.
///
/// `api: constant`
pub const TS_CHART_ASPECTS: u32 = 4;
/// A chart request's `sections` bit: the derived points.
///
/// `api: constant`
pub const TS_CHART_POINTS: u32 = 8;
/// A chart request's `sections` bit: the houses service.
///
/// `api: constant`
pub const TS_CHART_HOUSES: u32 = 16;
/// A chart request's `sections` bit: the Ashtakavarga.
///
/// `api: constant`
pub const TS_CHART_ASHTAKAVARGA: u32 = 32;
/// A chart request's `sections` bit: the Vimshopaka.
///
/// `api: constant`
pub const TS_CHART_VIMSHOPAKA: u32 = 64;
/// A chart request's `sections` bit: the Shadbala.
///
/// `api: constant`
pub const TS_CHART_SHADBALA: u32 = 128;
/// A chart request's `sections` bit: the Bhava bala.
///
/// `api: constant`
pub const TS_CHART_BHAVA_BALA: u32 = 256;
/// A chart request's `sections` bit: the Vaiseshikamsa.
///
/// `api: constant`
pub const TS_CHART_VAISESHIKAMSA: u32 = 512;
/// A chart request's `sections` bit: the dasha phala.
///
/// `api: constant`
pub const TS_CHART_DASHA_PHALA: u32 = 1024;
/// A chart request's `sections` bit: Jaimini's significators, the
/// karakamsha and the Brahma graha.
///
/// `api: constant`
pub const TS_CHART_JAIMINI: u32 = 2048;
/// A chart request's `sections` bit: Uranus, Neptune and Pluto beside
/// the nine, in the `outer` section (`03-design/western-outer-planets.md`).
///
/// `api: constant`
pub const TS_CHART_OUTER: u32 = 4096;

const SECTION_BITS: [SectionBit; 13] = [
    (TS_CHART_PANCHANGA, ChartRequest::with_panchanga),
    (TS_CHART_STATE, ChartRequest::with_state),
    (TS_CHART_ASPECTS, ChartRequest::with_aspects),
    (TS_CHART_POINTS, ChartRequest::with_points),
    (TS_CHART_HOUSES, ChartRequest::with_houses),
    (TS_CHART_ASHTAKAVARGA, ChartRequest::with_ashtakavarga),
    (TS_CHART_VIMSHOPAKA, ChartRequest::with_vimshopaka),
    (TS_CHART_SHADBALA, ChartRequest::with_shadbala),
    (TS_CHART_BHAVA_BALA, ChartRequest::with_bhava_bala),
    (TS_CHART_VAISESHIKAMSA, ChartRequest::with_vaiseshikamsa),
    (TS_CHART_DASHA_PHALA, ChartRequest::with_dasha_phala),
    (TS_CHART_JAIMINI, ChartRequest::with_jaimini),
    (TS_CHART_OUTER, ChartRequest::with_outer_planets),
];

/// The reading a bit set asks for, added to a request.
fn sections_of(bits: u32, mut request: ChartRequest) -> ChartRequest {
    for (bit, add) in SECTION_BITS {
        if bits & bit == bit {
            request = add(request);
        }
    }
    request
}

/// The day's seventeen-and-five values, in the order `day_section`
/// declares them.
///
/// Declared once in `schemas::day_section` and filled once here, so the
/// panchanga blob writes the same day the same way rather than a second
/// copy of the same arithmetic
/// (`03-design/chart-at-the-boundary.md` §8).
#[must_use]
pub fn day_values(local: &teistro_time::local_day::LocalDay) -> Vec<FixedValue> {
    let (state, polar_kind, polar_policy) = TsDayState::split(local.state);
    let named = |which| TsSunrise::of(which).map_or(u64::from(u8::MAX), |s| s as u64);
    let (convention, convention_value) = match local.convention {
        // The convention an air was given to is named as the convention;
        // the air crosses beside it, as it was applied (`LocalDay::air`).
        teistro_core::settings::SunriseConvention::Named { which }
        | teistro_core::settings::SunriseConvention::Atmospheric { which, .. } => {
            (named(which), 0.0)
        }
        // No id names a custom convention, so the sentinel says "read the
        // altitude beside this" rather than naming a convention it is not.
        teistro_core::settings::SunriseConvention::Custom { altitude_deg } => {
            (u64::from(u8::MAX), altitude_deg)
        }
    };
    let air = local.air();
    let era = local.date.era;
    vec![
        local.sunrise.get().into(),
        local.sunset.get().into(),
        local.next_sunrise.get().into(),
        u64::from(local.vara.id()).into(),
        u64::from(local.date.calendar.id()).into(),
        era.map_or(u64::from(u16::MAX), |e| u64::from(e.era.id()))
            .into(),
        i64::from(local.date.year).into(),
        i64::from(era.map_or(0, |e| e.year)).into(),
        u64::from(local.date.month).into(),
        u64::from(local.date.day).into(),
        (resolution_id(&local.date.resolution)).into(),
        u64::from(computed(&local.date.resolution).0).into(),
        u64::from(computed(&local.date.resolution).1).into(),
        (state as u64).into(),
        u64::from(polar_kind).into(),
        u64::from(polar_policy).into(),
        convention.into(),
        convention_value.into(),
        air.map_or(0.0, |air| air.pressure_hpa).into(),
        air.map_or(0.0, |air| air.temperature_c).into(),
    ]
}

/// A resolution's id, as `TsResolution` numbers them.
fn resolution_id(resolution: &teistro_core::envelope::CalendarResolution) -> u64 {
    use teistro_core::envelope::CalendarResolution as R;
    match resolution {
        R::Defined => 0,
        R::Tabular { .. } => 1,
        R::Computed { .. } => 2,
        R::Divergent { .. } => 3,
    }
}

/// The engine's month and day where a divergent resolution reports them,
/// and nought otherwise.
fn computed(resolution: &teistro_core::envelope::CalendarResolution) -> (u8, u8) {
    match resolution {
        teistro_core::envelope::CalendarResolution::Divergent { computed, .. } => {
            (computed.month, computed.day)
        }
        _ => (0, 0),
    }
}

/// One vector per graha column: the writer takes slices, and a column
/// is the unit the format stores.
struct GrahaColumns {
    ids: Vec<u16>,
    longitudes: Vec<f64>,
    tropicals: Vec<f64>,
    latitudes: Vec<f64>,
    distances: Vec<f64>,
    speeds: Vec<f64>,
    house_bhava: Vec<u8>,
    house_method: Vec<u16>,
    house_through: Vec<f64>,
    house_from: Vec<f64>,
    placed_bhava: Vec<u8>,
    placed_method: Vec<u16>,
    placed_through: Vec<f64>,
    placed_from: Vec<f64>,
}

impl GrahaColumns {
    /// The columns of every chart's bodies in one list, charts outermost:
    /// row `chart * count + g` is body `g` of chart `chart`, the same
    /// order the positions blob puts its cells in.
    fn of(
        charts: &[&ChartFoundation],
        pick: fn(&ChartFoundation) -> &[teistro_chart::foundation::GrahaPosition],
    ) -> GrahaColumns {
        let rows: Vec<&teistro_chart::foundation::GrahaPosition> =
            charts.iter().flat_map(|c| pick(c)).collect();
        GrahaColumns {
            ids: rows.iter().map(|g| g.graha.id()).collect(),
            longitudes: rows.iter().map(|g| g.longitude_deg).collect(),
            tropicals: rows.iter().map(|g| g.tropical_deg).collect(),
            latitudes: rows.iter().map(|g| g.latitude_deg).collect(),
            distances: rows.iter().map(|g| g.distance_au).collect(),
            speeds: rows.iter().map(|g| g.speed_deg_per_day).collect(),
            house_bhava: rows.iter().map(|g| g.house.bhava).collect(),
            house_method: rows.iter().map(|g| g.house.method.id()).collect(),
            house_through: rows.iter().map(|g| g.house.through).collect(),
            house_from: rows.iter().map(|g| g.house.from_madhya_deg).collect(),
            placed_bhava: rows.iter().map(|g| g.placement.bhava).collect(),
            placed_method: rows.iter().map(|g| g.placement.method.id()).collect(),
            placed_through: rows.iter().map(|g| g.placement.through).collect(),
            placed_from: rows.iter().map(|g| g.placement.from_madhya_deg).collect(),
        }
    }
}

/// What the batch decided once, in the order `summary` declares it.
///
/// Every value here comes from the request rather than from a chart, so
/// a batch that founded nothing still says where and what it founded
/// nothing of — as the positions blob writes its grid's frame whether or
/// not the grid has a cell.
#[must_use]
fn summary_values(
    place: &Place,
    kind: ChartKind,
    chart_count: u32,
    graha_count: u32,
    varga_count: u32,
    dasha_count: u32,
) -> Vec<FixedValue> {
    vec![
        u64::from(kind.id()).into(),
        u64::from(chart_count).into(),
        u64::from(graha_count).into(),
        u64::from(varga_count).into(),
        u64::from(dasha_count).into(),
        place.latitude.get().into(),
        place.longitude.get().into(),
        place.altitude.get().into(),
    ]
}

/// How many grahas every chart in the batch holds, or `INTERNAL` for a
/// batch that mixes sizes.
///
/// The blob's layout is one count for the batch, and only this crate
/// could have built a value that disagrees with itself. The outer planets
/// are held to one count too: the request asks them of every chart, and
/// a reader divides the `outer` section by the batch.
fn one_size(charts: &[&ChartFoundation]) -> Result<usize, Error> {
    for (what, count) in [
        (
            "grahas",
            (|c: &ChartFoundation| c.grahas.len()) as fn(&ChartFoundation) -> usize,
        ),
        ("outer planets", |c: &ChartFoundation| c.outer.len()),
    ] {
        let first = charts.first().map_or(0, |c| count(c));
        if let Some(odd) = charts.iter().find(|c| count(c) != first) {
            return Err(Error::new(
                Status::Internal,
                format!(
                    "the batch mixes chart sizes: {first} {what} and {}, though every chart is the same kind",
                    count(odd)
                ),
            ));
        }
    }
    Ok(charts.first().map_or(0, |c| c.grahas.len()))
}

/// The drishti of a batch, as the section carries them.
///
/// **Charts outermost**, as every per-chart section here is, and a batch
/// whose charts hold different numbers of relations is `INTERNAL` for
/// the reason a batch of differing graha counts is: the layout is one
/// count for the batch. That is not a restriction in practice — the
/// relations are a function of the grahas' signs, and a batch founded
/// from one request over one place has the same nine bodies in every
/// chart — but it is a fact the layout depends on, so it is checked
/// rather than assumed.
struct AspectColumns {
    /// How many relations each chart holds, one entry per chart.
    ///
    /// **Per chart and not one for the batch**, and the check that would
    /// have enforced a batch-wide count is what found out: a chart's
    /// drishti are a function of where the bodies stand rather than of
    /// how many there are, and two charts of the same nine grahas at one
    /// place hold 47 relations and 40. So the rows are concatenated and
    /// a reader prefix-sums these, which is the panchanga blob's own
    /// rule for a ragged list.
    counts: Vec<u32>,
    /// The drishti table every one of them was read under, which is one
    /// to a batch because it is a setting.
    table: String,
    from: Vec<u16>,
    to: Vec<u16>,
    houses: Vec<u8>,
    strength: Vec<u8>,
    from_sign: Vec<f64>,
    from_nakshatra: Vec<f64>,
    from_pada: Vec<f64>,
    to_sign: Vec<f64>,
    to_nakshatra: Vec<f64>,
    to_pada: Vec<f64>,
}

impl AspectColumns {
    fn of(documents: &[Document]) -> AspectColumns {
        let rows: usize = documents
            .iter()
            .map(|d| d.aspects.as_ref().map_or(0, |a| a.all().len()))
            .sum();
        let mut columns = AspectColumns {
            counts: Vec::with_capacity(documents.len()),
            table: documents
                .first()
                .and_then(|d| d.aspects.as_ref())
                .map_or_else(String::new, |a| a.table().to_owned()),
            from: Vec::with_capacity(rows),
            to: Vec::with_capacity(rows),
            houses: Vec::with_capacity(rows),
            strength: Vec::with_capacity(rows),
            from_sign: Vec::with_capacity(rows),
            from_nakshatra: Vec::with_capacity(rows),
            from_pada: Vec::with_capacity(rows),
            to_sign: Vec::with_capacity(rows),
            to_nakshatra: Vec::with_capacity(rows),
            to_pada: Vec::with_capacity(rows),
        };
        for document in documents {
            let Some(aspects) = document.aspects.as_ref() else {
                columns.counts.push(0);
                continue;
            };
            columns
                .counts
                .push(u32::try_from(aspects.all().len()).unwrap_or(u32::MAX));
            for drishti in aspects.all() {
                columns.from.push(drishti.from.id());
                columns.to.push(drishti.to.id());
                columns.houses.push(drishti.houses);
                columns
                    .strength
                    .push(TsStrength::from(drishti.strength) as u8);
                columns.from_sign.push(drishti.from_edge.sign_deg);
                columns.from_nakshatra.push(drishti.from_edge.nakshatra_deg);
                columns.from_pada.push(drishti.from_edge.pada_deg);
                columns.to_sign.push(drishti.to_edge.sign_deg);
                columns.to_nakshatra.push(drishti.to_edge.nakshatra_deg);
                columns.to_pada.push(drishti.to_edge.pada_deg);
            }
        }
        columns
    }

    /// The section and the table it was read under.
    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "aspects",
            self.from.len(),
            &[
                ColumnData::U16(&self.from),
                ColumnData::U16(&self.to),
                ColumnData::U8(&self.houses),
                ColumnData::U8(&self.strength),
                ColumnData::F64(&self.from_sign),
                ColumnData::F64(&self.from_nakshatra),
                ColumnData::F64(&self.from_pada),
                ColumnData::F64(&self.to_sign),
                ColumnData::F64(&self.to_nakshatra),
                ColumnData::F64(&self.to_pada),
            ],
        )?;
        writer.bytes("drishti_table", self.table.as_bytes())
    }
}

/// What each graha is, as the section carries it.
///
/// One row per graha per chart and no count: a state is a reading of a
/// placement, so there is one per placement.
struct StateColumns {
    /// The combustion table every `burning` was judged against, one to a
    /// batch because it is a setting.
    table: String,
    rows: Vec<Vec<FixedValue>>,
}

/// A set of catalogue members as a bit set: bit `n` is the member with
/// id `n`.
///
/// Six members in the only enum this is used for, so a `u32` holds any
/// of the three lists with room to spare — and a set stays a set rather
/// than becoming three ragged sections with three prefix sums.
fn bits_of<M: teistro_core::catalogue::Catalogued>(members: &[M]) -> u64 {
    members
        .iter()
        .fold(0_u64, |set, member| set | (1_u64 << member.id()))
}

impl StateColumns {
    fn of(documents: &[Document]) -> StateColumns {
        let mut columns = StateColumns {
            table: String::new(),
            rows: Vec::new(),
        };
        for document in documents {
            let Some(states) = document.state.as_ref() else {
                continue;
            };
            for state in states {
                columns.rows.push(state_values(state));
            }
        }
        if documents.iter().any(|d| d.state.is_some()) {
            // The table is the settings', and every state in the batch
            // was judged against the same one.
            columns.table = String::from("settings.state.combustion_orbs");
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.rows("states", &self.rows)?;
        writer.bytes("combustion_orbs", self.table.as_bytes())
    }
}

/// One graha's state, in the order `states` declares its columns.
#[must_use]
fn state_values(state: &teistro_state::GrahaState) -> Vec<FixedValue> {
    let flag = |yes: bool| FixedValue::from(u64::from(yes));
    let friendship = &state.friendship;
    let war = state.war;
    vec![
        u64::from(state.graha.id()).into(),
        u64::from(state.sign.id()).into(),
        u64::from(state.house).into(),
        u64::from(state.dignity.id()).into(),
        u64::from(friendship.natural.id()).into(),
        u64::from(friendship.temporary.id()).into(),
        u64::from(friendship.compound.id()).into(),
        flag(friendship.dispositor.is_some()),
        u64::from(
            friendship
                .dispositor
                .map_or(0, teistro_core::catalogue::Graha::id),
        )
        .into(),
        (TsBurning::from(state.combustion.burning) as u64).into(),
        flag(state.combustion.from_sun_deg.is_some()),
        state.combustion.from_sun_deg.unwrap_or(0.0).into(),
        flag(state.combustion.orbs.is_some()),
        state.combustion.orbs.map_or(0.0, |o| o.orb_deg).into(),
        flag(state.combustion.orbs.is_some_and(|o| o.deep_deg.is_some())),
        state
            .combustion
            .orbs
            .and_then(|o| o.deep_deg)
            .unwrap_or(0.0)
            .into(),
        u64::from(state.age.id()).into(),
        u64::from(state.wakefulness.id()).into(),
        flag(state.deeptadi.is_some()),
        u64::from(
            state
                .deeptadi
                .map_or(0, teistro_core::catalogue::AvasthaDeeptadi::id),
        )
        .into(),
        bits_of(&state.lajjitadi.holding).into(),
        bits_of(&state.lajjitadi.ruled_out).into(),
        bits_of(&state.lajjitadi.undecided).into(),
        flag(war.is_some()),
        u64::from(war.map_or(0, |w| w.opponent.id())).into(),
        flag(war.is_some_and(|w| w.is_winner)),
        war.map_or(0.0, |w| w.apart_deg).into(),
        state.boundaries.sign_deg.into(),
        state.boundaries.nakshatra_deg.into(),
        state.boundaries.pada_deg.into(),
        flag(state.sayanadi.is_some()),
        u64::from(state.sayanadi.map_or(0, |s| s.avastha.id())).into(),
    ]
    .into_iter()
    .chain(
        teistro_state::Anka::ALL
            .map(|anka| u64::from(state.sayanadi.map_or(0, |s| s.cheshta(anka).id())).into()),
    )
    .collect()
}

/// The twelve bhavas of each chart, as the houses service reads them.
///
/// **Not ragged**, and this is the case that says why the rule is about
/// the values rather than about the section: a chart that has bhavas has
/// twelve, always, so the count is a constant and an empty section can
/// only mean "not asked for".
struct BhavaColumns {
    sign: Vec<u16>,
    lord: Vec<u16>,
    quadrant: Vec<u8>,
}

impl BhavaColumns {
    fn of(documents: &[Document]) -> BhavaColumns {
        let rows = documents.len() * 12;
        let mut columns = BhavaColumns {
            sign: Vec::with_capacity(rows),
            lord: Vec::with_capacity(rows),
            quadrant: Vec::with_capacity(rows),
        };
        for document in documents {
            let Some(houses) = document.houses.as_ref() else {
                continue;
            };
            for number in 1..=12_u8 {
                let Some(bhava) = houses.bhava(number) else {
                    continue;
                };
                columns.sign.push(bhava.sign.id());
                columns.lord.push(bhava.lord.id());
                columns
                    .quadrant
                    .push(TsQuadrant::from(bhava.quadrant) as u8);
            }
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "bhavas",
            self.sign.len(),
            &[
                ColumnData::U16(&self.sign),
                ColumnData::U16(&self.lord),
                ColumnData::U8(&self.quadrant),
            ],
        )
    }
}

/// The derived points of a batch, as the section carries them.
///
/// **Ragged**, as the drishti are: a chart's points depend on what its
/// day allows — Saturn's eighth needs an arc to divide — so the count is
/// a per-chart fact. The drishti taught that lesson by refusing a batch
/// (`03-design/chart-reading.md` §5); this one takes it as read.
/// Every chart's Ashtakavarga, each section empty when it was not asked for.
struct AshtakavargaColumns {
    graha: Vec<u16>,
    shodhana: Vec<u8>,
    ekadhipatya: Vec<u8>,
    rashi_pinda: Vec<u32>,
    graha_pinda: Vec<u32>,
    yoga_pinda: Vec<u32>,
    bindus: Vec<u8>,
    reduced: Vec<u8>,
    sarva: Vec<u16>,
    trikona: Vec<u16>,
    sarva_reduced: Vec<u16>,
}

impl AshtakavargaColumns {
    fn of(documents: &[Document]) -> AshtakavargaColumns {
        let charts = documents
            .iter()
            .filter(|d| d.ashtakavarga.is_some())
            .count();
        let mut columns = AshtakavargaColumns {
            graha: Vec::with_capacity(charts * 7),
            shodhana: Vec::with_capacity(charts * 7),
            ekadhipatya: Vec::with_capacity(charts * 7),
            rashi_pinda: Vec::with_capacity(charts * 7),
            graha_pinda: Vec::with_capacity(charts * 7),
            yoga_pinda: Vec::with_capacity(charts * 7),
            bindus: Vec::with_capacity(charts * 84),
            reduced: Vec::with_capacity(charts * 84),
            sarva: Vec::with_capacity(charts * 12),
            trikona: Vec::with_capacity(charts * 12),
            sarva_reduced: Vec::with_capacity(charts * 12),
        };
        for reading in documents.iter().filter_map(|d| d.ashtakavarga.as_ref()) {
            for graha in &reading.grahas {
                columns.graha.push(graha.graha.id());
                columns
                    .shodhana
                    .push(TsShodhana::from(reading.rules.shodhana) as u8);
                columns
                    .ekadhipatya
                    .push(TsEkadhipatya::from(reading.rules.ekadhipatya) as u8);
                columns.rashi_pinda.push(graha.rashi_pinda);
                columns.graha_pinda.push(graha.graha_pinda);
                columns.yoga_pinda.push(graha.yoga_pinda);
                columns.bindus.extend(graha.bindus);
                columns.reduced.extend(graha.reduced.unwrap_or([0; 12]));
            }
            columns.sarva.extend(reading.sarva);
            columns.trikona.extend(reading.trikona);
            columns.sarva_reduced.extend(reading.reduced);
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "ashtakavarga",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::U8(&self.shodhana),
                ColumnData::U8(&self.ekadhipatya),
                ColumnData::U32(&self.rashi_pinda),
                ColumnData::U32(&self.graha_pinda),
                ColumnData::U32(&self.yoga_pinda),
            ],
        )?;
        writer.columns(
            "ashtakavarga_bindus",
            self.bindus.len(),
            &[ColumnData::U8(&self.bindus), ColumnData::U8(&self.reduced)],
        )?;
        writer.columns(
            "sarvashtakavarga",
            self.sarva.len(),
            &[
                ColumnData::U16(&self.sarva),
                ColumnData::U16(&self.trikona),
                ColumnData::U16(&self.sarva_reduced),
            ],
        )
    }
}

/// A Shadbala value column: its name in the `shadbala` section, what it
/// holds, and where a graha's reading keeps it.
pub(crate) type ShadbalaColumn = (
    &'static str,
    &'static str,
    fn(&teistro::strength::GrahaShadbala) -> f64,
);

/// The `shadbala` section's value columns in order, which the section's
/// schema and its writer both read.
pub(crate) const SHADBALA_COLUMNS: [ShadbalaColumn; 25] = [
    (
        "uchcha",
        "Sthana: from the distance to the debilitation point, 0 to 60.",
        |g| g.sthana.uchcha,
    ),
    (
        "saptavargaja",
        "Sthana: from the dignity in the seven vargas.",
        |g| g.sthana.saptavargaja,
    ),
    (
        "ojayugma",
        "Sthana: from the rasi's and navamsha's parity, 0, 15 or 30.",
        |g| g.sthana.ojayugma,
    ),
    ("kendradi", "Sthana: from the house, 60, 30 or 15.", |g| {
        g.sthana.kendradi
    }),
    ("drekkana", "Sthana: from the decanate, 0 or 15.", |g| {
        g.sthana.drekkana
    }),
    (
        "dig",
        "Dig: from the distance to the powerless kendra, 0 to 60.",
        |g| g.dig,
    ),
    ("nathonnatha", "Kaala: from the hour, 0 to 60.", |g| {
        g.kaala.nathonnatha
    }),
    (
        "paksha",
        "Kaala: from the Moon's elongation, the Moon's doubled.",
        |g| g.kaala.paksha,
    ),
    (
        "tribhaga",
        "Kaala: 60 to the lord of the third of the day or night, and to Jupiter.",
        |g| g.kaala.tribhaga,
    ),
    ("abda", "Kaala: 15 to the year's lord.", |g| g.kaala.abda),
    ("masa", "Kaala: 30 to the month's lord.", |g| g.kaala.masa),
    ("vara", "Kaala: 45 to the weekday's lord.", |g| g.kaala.vara),
    ("hora", "Kaala: 60 to the hour's lord.", |g| g.kaala.hora),
    ("ayana", "Kaala: from the declination.", |g| g.kaala.ayana),
    (
        "yuddha",
        "Kaala: gained by the victor and lost by the vanquished of a planetary war.",
        |g| g.kaala.yuddha,
    ),
    ("cheshta", "Cheshta: motional strength.", |g| g.cheshta),
    ("naisargika", "Naisargika: natural strength.", |g| {
        g.naisargika
    }),
    (
        "drik",
        "Drik: aspectual strength, which may be negative.",
        |g| g.drik,
    ),
    ("virupas", "The six together, virupas.", |g| g.virupas),
    ("rupas", "The six together, rupas.", |g| g.rupas),
    (
        "required_rupas",
        "The rupas it must reach to be strong.",
        |g| g.required_rupas,
    ),
    (
        "ishta",
        "How far it tends to good, 0 to 60 (BPHS ch. 28).",
        |g| g.ishta,
    ),
    ("kashta", "How far it tends to harm, 0 to 60.", |g| g.kashta),
    (
        "subha_rashmi",
        "Its auspicious rays, 1 to 7: the mean of its Uchcha and Cheshta rays (BPHS ch. 28 v. 5).",
        |g| g.subha_rashmi,
    ),
    (
        "ashubha_rashmi",
        "Its inauspicious rays, 8 less the auspicious.",
        |g| g.ashubha_rashmi,
    ),
];

/// Every chart's Shadbala, a row a graha, empty when it was not asked for.
struct ShadbalaColumns {
    graha: Vec<u16>,
    values: Vec<Vec<f64>>,
    strong: Vec<u8>,
}

impl ShadbalaColumns {
    fn of(documents: &[Document]) -> ShadbalaColumns {
        let readings: Vec<_> = documents
            .iter()
            .filter_map(|d| d.shadbala.as_ref())
            .collect();
        let rows = readings.iter().map(|r| r.grahas.len()).sum();
        let mut columns = ShadbalaColumns {
            graha: Vec::with_capacity(rows),
            values: SHADBALA_COLUMNS
                .iter()
                .map(|_| Vec::with_capacity(rows))
                .collect(),
            strong: Vec::with_capacity(rows),
        };
        for graha in readings.iter().flat_map(|r| &r.grahas) {
            columns.graha.push(graha.graha.id());
            for (column, (_, _, read)) in columns.values.iter_mut().zip(SHADBALA_COLUMNS) {
                column.push(read(graha));
            }
            columns.strong.push(u8::from(graha.strong));
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        let mut data = Vec::with_capacity(self.values.len() + 2);
        data.push(ColumnData::U16(&self.graha));
        data.extend(self.values.iter().map(|column| ColumnData::F64(column)));
        data.push(ColumnData::U8(&self.strong));
        writer.columns("shadbala", self.graha.len(), &data)
    }
}

/// Where a graha's Vaiseshikamsa keeps its standing in one scheme.
type SchemeStanding = fn(&teistro::strength::GrahaVaiseshikamsa) -> teistro::strength::Standing;

/// The four schemes a Vaiseshikamsa reads, each column pair's name and where
/// a graha's reading keeps its standing, which the section's schema and its
/// writer both read.
pub(crate) const VAISESHIKAMSA_SCHEMES: [(&str, SchemeStanding); 4] = [
    ("shadvarga", |g| g.shadvarga),
    ("saptavarga", |g| g.saptavarga),
    ("dashavarga", |g| g.dashavarga),
    ("shodashavarga", |g| g.shodashavarga),
];

/// Every chart's dasha phala, a row a graha, Sun to Ketu, empty when it was
/// not asked for.
struct DashaPhalaColumns {
    graha: Vec<u16>,
    subhankas: [Vec<f64>; 7],
    subhanka: Vec<f64>,
    asubhanka: Vec<f64>,
    nature: Vec<u16>,
    phase: Vec<u8>,
    favourable: Vec<u8>,
    unfavourable: Vec<u8>,
}

impl DashaPhalaColumns {
    fn of(documents: &[Document]) -> DashaPhalaColumns {
        let grahas: Vec<_> = documents
            .iter()
            .filter_map(|d| d.dasha_phala.as_ref())
            .flat_map(|reading| &reading.grahas)
            .collect();
        let rows = grahas.len();
        let mut columns = DashaPhalaColumns {
            graha: Vec::with_capacity(rows),
            subhankas: std::array::from_fn(|_| Vec::with_capacity(rows)),
            subhanka: Vec::with_capacity(rows),
            asubhanka: Vec::with_capacity(rows),
            nature: Vec::with_capacity(rows),
            phase: Vec::with_capacity(rows),
            favourable: Vec::with_capacity(rows),
            unfavourable: Vec::with_capacity(rows),
        };
        for graha in grahas {
            columns.graha.push(graha.graha.id());
            for (column, value) in columns.subhankas.iter_mut().zip(graha.subhankas) {
                column.push(value);
            }
            columns.subhanka.push(graha.subhanka);
            columns.asubhanka.push(graha.asubhanka);
            columns.nature.push(graha.nature.id());
            columns.phase.push(TsDashaPhase::from(graha.phase) as u8);
            columns.favourable.push(u8::from(graha.favourable));
            columns.unfavourable.push(u8::from(graha.unfavourable));
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        let mut data = vec![ColumnData::U16(&self.graha)];
        data.extend(self.subhankas.iter().map(|column| ColumnData::F64(column)));
        data.extend([
            ColumnData::F64(&self.subhanka),
            ColumnData::F64(&self.asubhanka),
            ColumnData::U16(&self.nature),
            ColumnData::U8(&self.phase),
            ColumnData::U8(&self.favourable),
            ColumnData::U8(&self.unfavourable),
        ]);
        writer.columns("dasha_phala", self.graha.len(), &data)
    }
}

/// Every chart's Jaimini significators, a row a chart that asked for them,
/// and each chart's nine houses from the karakamsha, a row a graha.
struct JaiminiColumns {
    atmakaraka: Vec<u16>,
    karakamsha: Vec<u16>,
    brahma_rule: Vec<u8>,
    counted_from: Vec<u16>,
    qualified: Vec<u16>,
    brahma: Vec<u16>,
    brahma_outcome: Vec<u8>,
    passed_from: Vec<u16>,
    passed_from_present: Vec<u8>,
    /// The `jaimini_grahas` section.
    graha: Vec<u16>,
    in_rasi: Vec<u8>,
    in_navamsha: Vec<u8>,
    arudha: Vec<u16>,
    arudha_present: Vec<u8>,
}

impl JaiminiColumns {
    fn of(documents: &[Document]) -> JaiminiColumns {
        let readings: Vec<_> = documents
            .iter()
            .filter_map(|d| d.jaimini.as_ref())
            .collect();
        let (charts, rows) = (readings.len(), readings.len() * 9);
        let mut columns = JaiminiColumns {
            atmakaraka: Vec::with_capacity(charts),
            karakamsha: Vec::with_capacity(charts),
            brahma_rule: Vec::with_capacity(charts),
            counted_from: Vec::with_capacity(charts),
            qualified: Vec::with_capacity(charts),
            brahma: Vec::with_capacity(charts),
            brahma_outcome: Vec::with_capacity(charts),
            passed_from: Vec::with_capacity(charts),
            passed_from_present: Vec::with_capacity(charts),
            graha: Vec::with_capacity(rows),
            in_rasi: Vec::with_capacity(rows),
            in_navamsha: Vec::with_capacity(rows),
            arudha: Vec::with_capacity(rows),
            arudha_present: Vec::with_capacity(rows),
        };
        for reading in readings {
            let (karakamsha, brahma) = (&reading.karakamsha, &reading.brahma);
            debug_assert_eq!(brahma.graha.is_none(), brahma.none.is_some());
            columns.atmakaraka.push(karakamsha.atmakaraka.id());
            columns.karakamsha.push(karakamsha.sign.id());
            columns
                .brahma_rule
                .push(TsBrahmaRule::from(brahma.rule) as u8);
            columns.counted_from.push(brahma.counted_from.id());
            columns.qualified.push(graha_mask(&brahma.qualified));
            columns.brahma.push(graha_or_absent(brahma.graha).0);
            columns
                .brahma_outcome
                .push(TsBrahmaOutcome::from(brahma.none) as u8);
            let (passed_from, present) = graha_or_absent(brahma.passed_from);
            columns.passed_from.push(passed_from);
            columns.passed_from_present.push(present);
            for (((graha, rasi), navamsha), arudha) in teistro::catalogue::Graha::ALL
                .iter()
                .zip(karakamsha.in_rasi)
                .zip(karakamsha.in_navamsha)
                .zip(reading.graha_arudhas)
            {
                columns.graha.push(graha.id());
                columns.in_rasi.push(rasi);
                columns.in_navamsha.push(navamsha);
                let (sign, present) = arudha.map_or((0, 0), |sign| (sign.id(), 1));
                columns.arudha.push(sign);
                columns.arudha_present.push(present);
            }
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "jaimini",
            self.atmakaraka.len(),
            &[
                ColumnData::U16(&self.atmakaraka),
                ColumnData::U16(&self.karakamsha),
                ColumnData::U8(&self.brahma_rule),
                ColumnData::U16(&self.counted_from),
                ColumnData::U16(&self.qualified),
                ColumnData::U16(&self.brahma),
                ColumnData::U8(&self.brahma_outcome),
                ColumnData::U16(&self.passed_from),
                ColumnData::U8(&self.passed_from_present),
            ],
        )?;
        writer.columns(
            "jaimini_grahas",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::U8(&self.in_rasi),
                ColumnData::U8(&self.in_navamsha),
                ColumnData::U16(&self.arudha),
                ColumnData::U8(&self.arudha_present),
            ],
        )
    }
}

/// Every chart's transits, a row a chart an instant, and under each the
/// nine grahas, a row a graha; empty when none were asked for.
#[derive(Default)]
struct GocharColumns {
    instant: Vec<f64>,
    reference: Vec<u16>,
    counted_from: Vec<u8>,
    node_vedha: Vec<u8>,
    node_obstruction: Vec<u8>,
    ashtakavarga_good_from: Vec<u8>,
    /// The `gochar_grahas` section.
    graha: Vec<u16>,
    sign: Vec<u16>,
    degrees: Vec<f64>,
    house: Vec<u8>,
    good_house: Vec<u8>,
    vedha_house: Vec<u8>,
    obstructed_by: Vec<u16>,
    verdict: Vec<u8>,
    fruition: Vec<u8>,
    fruitful_now: Vec<u8>,
    /// The `gochar_ashtakavarga` section.
    av_graha: Vec<u16>,
    av_bindus: Vec<u8>,
    av_good: Vec<u8>,
    av_kakshya: Vec<u8>,
    av_kakshya_lord: Vec<u8>,
    av_kakshya_bindu: Vec<u8>,
    av_sarva: Vec<u16>,
    av_sarva_standing: Vec<u8>,
}

impl GocharColumns {
    /// Each chart's readings, in the batch's order and each chart's in the
    /// order its instants were asked in.
    fn of(
        readings: &[Vec<teistro::gochar::GocharReading>],
        instants: &[JulianDay<Utc>],
    ) -> Result<GocharColumns, Error> {
        let mut columns = GocharColumns::default();
        for chart in readings {
            if chart.len() != instants.len() {
                return Err(Error::internal(format!(
                    "a chart answered {} transits for {} instants",
                    chart.len(),
                    instants.len()
                )));
            }
            for (reading, at) in chart.iter().zip(instants) {
                let rules = reading.rules;
                columns.instant.push(at.get());
                columns.reference.push(reading.reference.sign.id());
                columns.counted_from.push(
                    TsGocharFrom::of(reading.reference.from)
                        .ok_or_else(|| no_code("the gochar reference"))? as u8,
                );
                columns.node_vedha.push(
                    TsNodeVedha::of(rules.node_vedha)
                        .ok_or_else(|| no_code("`gochar.node_vedha`"))? as u8,
                );
                columns.node_obstruction.push(
                    TsNodeObstruction::of(rules.node_obstruction)
                        .ok_or_else(|| no_code("`gochar.node_obstruction`"))?
                        as u8,
                );
                columns.ashtakavarga_good_from.push(
                    TsAshtakavargaGoodFrom::of(rules.ashtakavarga_good_from)
                        .ok_or_else(|| no_code("`gochar.ashtakavarga_good_from`"))?
                        as u8,
                );
                for read in reading.ashtakavarga.iter().flatten() {
                    columns.av_graha.push(read.graha.id());
                    columns.av_bindus.push(read.bindus);
                    columns.av_good.push(u8::from(read.good));
                    columns.av_kakshya.push(read.kakshya.index);
                    columns
                        .av_kakshya_lord
                        .push(TsKakshyaLord::from(read.kakshya.lord) as u8);
                    columns.av_kakshya_bindu.push(u8::from(read.kakshya_bindu));
                    columns.av_sarva.push(read.sarva);
                    columns
                        .av_sarva_standing
                        .push(TsSarvaStanding::from(read.sarva_standing) as u8);
                }
                for read in &reading.grahas {
                    columns.graha.push(read.graha.id());
                    columns.sign.push(read.transit.sign.id());
                    columns.degrees.push(read.transit.degrees);
                    columns.house.push(read.house);
                    columns.good_house.push(u8::from(read.good_house));
                    columns.vedha_house.push(read.vedha_house.unwrap_or(0));
                    columns.obstructed_by.push(graha_mask(&read.obstructed_by));
                    columns
                        .verdict
                        .push(TsGocharVerdict::from(read.verdict) as u8);
                    columns.fruition.push(TsFruition::from(read.fruition) as u8);
                    columns.fruitful_now.push(u8::from(read.fruitful_now));
                }
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "gochar",
            self.instant.len(),
            &[
                ColumnData::F64(&self.instant),
                ColumnData::U16(&self.reference),
                ColumnData::U8(&self.counted_from),
                ColumnData::U8(&self.node_vedha),
                ColumnData::U8(&self.node_obstruction),
                ColumnData::U8(&self.ashtakavarga_good_from),
            ],
        )?;
        writer.columns(
            "gochar_grahas",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::U16(&self.sign),
                ColumnData::F64(&self.degrees),
                ColumnData::U8(&self.house),
                ColumnData::U8(&self.good_house),
                ColumnData::U8(&self.vedha_house),
                ColumnData::U16(&self.obstructed_by),
                ColumnData::U8(&self.verdict),
                ColumnData::U8(&self.fruition),
                ColumnData::U8(&self.fruitful_now),
            ],
        )?;
        writer.columns(
            "gochar_ashtakavarga",
            self.av_graha.len(),
            &[
                ColumnData::U16(&self.av_graha),
                ColumnData::U8(&self.av_bindus),
                ColumnData::U8(&self.av_good),
                ColumnData::U8(&self.av_kakshya),
                ColumnData::U8(&self.av_kakshya_lord),
                ColumnData::U8(&self.av_kakshya_bindu),
                ColumnData::U16(&self.av_sarva),
                ColumnData::U8(&self.av_sarva_standing),
            ],
        )
    }
}

/// Every chart's transit hit list, a row a hit, charts outermost and
/// **ragged** by `cast.hit_count`; empty when none was asked for.
#[derive(Default)]
struct HitColumns {
    /// Each chart's rows, in the batch's order: zeroes when none asked.
    counts: Vec<u32>,
    instant: Vec<f64>,
    graha: Vec<u16>,
    kind: Vec<u8>,
    into: Vec<u16>,
    motion: Vec<u8>,
    to_lagna: Vec<u8>,
    to_graha: Vec<u16>,
    angle: Vec<u16>,
    phase: Vec<u8>,
}

impl HitColumns {
    /// Each chart's hits in the order the list gives them.
    fn of(lists: &[Vec<teistro::Hit>], charts: usize) -> Result<HitColumns, Error> {
        use teistro::gochar::hits::HitEvent;
        let mut columns = HitColumns {
            counts: vec![0; charts],
            ..HitColumns::default()
        };
        if lists.is_empty() {
            return Ok(columns);
        }
        if lists.len() != charts {
            return Err(Error::internal(format!(
                "{} hit lists for {charts} charts",
                lists.len()
            )));
        }
        for (count, list) in columns.counts.iter_mut().zip(lists) {
            *count = u32::try_from(list.len())
                .map_err(|_| Error::internal("a hit list longer than a section can count"))?;
        }
        for hit in lists.iter().flatten() {
            let (into, motion, to, angle, phase) = match hit.event {
                HitEvent::SignIngress { into, motion } => (into.id(), motion, None, 0, 0),
                HitEvent::NakshatraIngress { into, motion } => (into.id(), motion, None, 0, 0),
                HitEvent::Station { turns } => (0, turns, None, 0, 0),
                HitEvent::Aspect {
                    to,
                    angle,
                    phase,
                    motion,
                } => (0, motion, Some(to), angle, TsAspectPhase::from(phase) as u8),
            };
            columns.instant.push(hit.instant.get());
            columns.graha.push(hit.graha.id());
            columns.kind.push(TsHitKind::from(&hit.event) as u8);
            columns.into.push(into);
            columns.motion.push(TsMotion::from(motion) as u8);
            let (to_lagna, to_graha) = point_cells(to);
            columns.to_lagna.push(to_lagna);
            columns.to_graha.push(to_graha);
            columns.angle.push(angle);
            columns.phase.push(phase);
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "hits",
            self.instant.len(),
            &[
                ColumnData::F64(&self.instant),
                ColumnData::U16(&self.graha),
                ColumnData::U8(&self.kind),
                ColumnData::U16(&self.into),
                ColumnData::U8(&self.motion),
                ColumnData::U8(&self.to_lagna),
                ColumnData::U16(&self.to_graha),
                ColumnData::U16(&self.angle),
                ColumnData::U8(&self.phase),
            ],
        )
    }
}

/// What the searches over a window found for every chart: the transit hit
/// list and Sade Sati, the two sections a chart's own count makes ragged.
struct Searches {
    hits: HitColumns,
    sade_sati: SadeSatiColumns,
}

impl Searches {
    fn of(
        hits: &[Vec<teistro::Hit>],
        sade_sati: &[teistro::sade_sati::Report],
        charts: usize,
    ) -> Result<Searches, Error> {
        Ok(Searches {
            hits: HitColumns::of(hits, charts)?,
            sade_sati: SadeSatiColumns::of(sade_sati, charts)?,
        })
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        self.hits.write(writer)?;
        self.sade_sati.write(writer)
    }
}

/// Every chart's Sade Sati report: a row a chart in `sade_sati`, and a row
/// a visit in `sade_sati_visits`, **ragged** by `cast.sade_sati_visit_count`;
/// both empty when none was asked for.
#[derive(Default)]
struct SadeSatiColumns {
    /// Each chart's visit rows, in the batch's order: zeroes when none asked.
    counts: Vec<u32>,
    reference: Vec<u16>,
    counted_from: Vec<u8>,
    reckoning: Vec<u8>,
    /// The `sade_sati_visits` section.
    period: Vec<u16>,
    house: Vec<u8>,
    from: Vec<f64>,
    to: Vec<f64>,
}

impl SadeSatiColumns {
    /// Each chart's periods, its Sade Satis first and then its smaller
    /// spells, each in time order; a period's phases in the order they
    /// come, and each phase's visits in theirs.
    fn of(reports: &[teistro::sade_sati::Report], charts: usize) -> Result<SadeSatiColumns, Error> {
        let mut columns = SadeSatiColumns {
            counts: vec![0; charts],
            ..SadeSatiColumns::default()
        };
        if reports.is_empty() {
            return Ok(columns);
        }
        if reports.len() != charts {
            return Err(Error::internal(format!(
                "{} Sade Sati reports for {charts} charts",
                reports.len()
            )));
        }
        let bound = |at: Option<JulianDay<Utc>>| at.map_or(f64::NAN, JulianDay::get);
        for (count, report) in columns.counts.iter_mut().zip(reports) {
            columns.reference.push(report.reference.sign.id());
            columns.counted_from.push(
                TsGocharFrom::of(report.reference.from)
                    .ok_or_else(|| no_code("the Sade Sati reference"))? as u8,
            );
            columns.reckoning.push(
                TsReckoning::of(report.reckoning)
                    .ok_or_else(|| no_code("the Sade Sati reckoning"))? as u8,
            );
            let periods = report
                .sade_sati
                .iter()
                .map(|one| one.phases.as_slice())
                .chain(report.spells.iter().map(std::slice::from_ref));
            let before = columns.house.len();
            for (ordinal, spells) in periods.enumerate() {
                let ordinal = u16::try_from(ordinal)
                    .map_err(|_| Error::internal("more Sade Sati periods than a row can number"))?;
                for spell in spells {
                    for visit in &spell.visits {
                        columns.period.push(ordinal);
                        columns.house.push(spell.house);
                        columns.from.push(bound(visit.from));
                        columns.to.push(bound(visit.to));
                    }
                }
            }
            *count = u32::try_from(columns.house.len() - before)
                .map_err(|_| Error::internal("more Sade Sati visits than a section can count"))?;
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "sade_sati",
            self.reference.len(),
            &[
                ColumnData::U16(&self.reference),
                ColumnData::U8(&self.counted_from),
                ColumnData::U8(&self.reckoning),
            ],
        )?;
        writer.columns(
            "sade_sati_visits",
            self.house.len(),
            &[
                ColumnData::U16(&self.period),
                ColumnData::U8(&self.house),
                ColumnData::F64(&self.from),
                ColumnData::F64(&self.to),
            ],
        )
    }
}

/// A planet's dignities as the boundary writes them, one flag a column in
/// [`teistro::EssentialDignity`]'s order.
const fn dignity_flags(d: teistro::EssentialDignity) -> [bool; 7] {
    [
        d.house,
        d.exaltation,
        d.triplicity,
        d.term,
        d.face,
        d.detriment,
        d.fall,
    ]
}

/// Pushes one row of a dignity's flags onto its seven columns.
fn push_flags(columns: &mut [Vec<u8>; 7], d: teistro::EssentialDignity) {
    for (column, flag) in columns.iter_mut().zip(dignity_flags(d)) {
        column.push(u8::from(flag));
    }
}

/// Every chart's essential dignities: a row a chart in `dignities`, the
/// seven planets a chart in `dignity_planets`, in the Chaldean order, and
/// its receptions in `dignity_receptions`, ragged by `reception_count`;
/// all empty when none was asked for.
#[derive(Default)]
struct DignityColumns {
    sect: Vec<u8>,
    sect_rule: Vec<u8>,
    terms: Vec<u8>,
    triplicities: Vec<u8>,
    /// The scores, one column a dignity in [`teistro::Scores`]' order.
    scores: [Vec<i8>; 8],
    reception_count: Vec<u8>,
    /// The `dignity_planets` section.
    planet: Vec<u16>,
    longitude: Vec<f64>,
    flags: [Vec<u8>; 7],
    score: Vec<i16>,
    reception: Vec<i16>,
    /// The `dignity_receptions` section.
    first: Vec<u16>,
    second: Vec<u16>,
    first_in: [Vec<u8>; 7],
    second_in: [Vec<u8>; 7],
}

/// A reading for each chart or for none: the boundary's check that a
/// batch's readings line up with its charts.
fn one_a_chart(read: usize, charts: usize, what: &str) -> Result<(), Error> {
    if read == 0 || read == charts {
        Ok(())
    } else {
        Err(Error::internal(format!(
            "{read} readings of the {what} for {charts} charts"
        )))
    }
}

/// The refusal for a value the encoder has no code for yet.
fn no_code(what: &str) -> Error {
    Error::internal(format!("{what} has no code at the boundary yet"))
}

impl DignityColumns {
    fn of<'a>(
        read: impl ExactSizeIterator<Item = &'a teistro::Dignities>,
        charts: usize,
    ) -> Result<DignityColumns, Error> {
        one_a_chart(read.len(), charts, "dignities")?;
        let mut columns = DignityColumns::default();
        for one in read {
            columns.sect.push(TsSect::from(one.sect) as u8);
            columns
                .sect_rule
                .push(TsSectRule::of(one.sect_rule).ok_or_else(|| no_code("the sect rule"))? as u8);
            columns.terms.push(
                TsTerms::of(&one.rules.terms).ok_or_else(|| no_code("the system of terms"))? as u8,
            );
            columns.triplicities.push(
                TsTriplicities::of(one.rules.triplicities)
                    .ok_or_else(|| no_code("the triplicity scheme"))? as u8,
            );
            let s = one.scores;
            let scores = [
                s.house,
                s.exaltation,
                s.triplicity,
                s.term,
                s.face,
                s.detriment,
                s.fall,
                s.peregrine,
            ];
            for (column, value) in columns.scores.iter_mut().zip(scores) {
                column.push(value);
            }
            columns.reception_count.push(
                u8::try_from(one.receptions.len())
                    .map_err(|_| Error::internal("more receptions than pairs of the seven"))?,
            );
            for at in &one.planets {
                columns.planet.push(at.planet.id());
                columns.longitude.push(at.longitude_deg);
                push_flags(&mut columns.flags, at.dignity);
                columns.score.push(at.score);
                columns.reception.push(at.reception);
            }
            for pair in &one.receptions {
                let [first, second] = pair.planets;
                columns.first.push(first.id());
                columns.second.push(second.id());
                push_flags(&mut columns.first_in, pair.first_in);
                push_flags(&mut columns.second_in, pair.second_in);
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        let mut chart = vec![
            ColumnData::U8(&self.sect),
            ColumnData::U8(&self.sect_rule),
            ColumnData::U8(&self.terms),
            ColumnData::U8(&self.triplicities),
        ];
        chart.extend(self.scores.iter().map(|column| ColumnData::I8(column)));
        chart.push(ColumnData::U8(&self.reception_count));
        writer.columns("dignities", self.sect.len(), &chart)?;
        let mut planets = vec![
            ColumnData::U16(&self.planet),
            ColumnData::F64(&self.longitude),
        ];
        planets.extend(self.flags.iter().map(|column| ColumnData::U8(column)));
        planets.push(ColumnData::I16(&self.score));
        planets.push(ColumnData::I16(&self.reception));
        writer.columns("dignity_planets", self.planet.len(), &planets)?;
        let mut pairs = vec![ColumnData::U16(&self.first), ColumnData::U16(&self.second)];
        pairs.extend(
            self.first_in
                .iter()
                .chain(&self.second_in)
                .map(|column| ColumnData::U8(column)),
        );
        writer.columns("dignity_receptions", self.first.len(), &pairs)
    }
}

/// The `hellenistic` module's sections: both halves of Lilly's table (the
/// essential dignities, from `dignities_json` or from the fortitudes' own,
/// and the accidental fortitudes) and Valens's lots.
struct HellenisticColumns {
    dignities: DignityColumns,
    fortitudes: FortitudeColumns,
    lots: LotColumns,
    considerations: ConsiderationColumns,
    perfections: PerfectionColumns,
}

impl HellenisticColumns {
    /// The sections from what `composed` carries for `charts` charts.
    fn of(composed: &Composed<'_>, charts: usize) -> Result<HellenisticColumns, Error> {
        let Composed {
            dignities,
            fortitudes,
            lots,
            considerations,
            perfections,
            ..
        } = *composed;
        let essential = if fortitudes.is_empty() {
            DignityColumns::of(dignities.iter(), charts)?
        } else {
            DignityColumns::of(fortitudes.iter().map(|one| &one.dignities), charts)?
        };
        Ok(HellenisticColumns {
            dignities: essential,
            fortitudes: FortitudeColumns::of(fortitudes, charts)?,
            lots: LotColumns::of(lots, charts)?,
            considerations: ConsiderationColumns::of(considerations, charts)?,
            perfections: PerfectionColumns::of(perfections, charts)?,
        })
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        self.dignities.write(writer)?;
        self.fortitudes.write(writer)?;
        self.lots.write(writer)?;
        self.considerations.write(writer)?;
        self.perfections.write(writer)
    }
}

/// Every chart's progressions (`western-progressions.md`): a row a chart in
/// `progressions`, the progressed and directed planets a row a graha in
/// `progressed_grahas` and `directed_grahas` when the record named an
/// instant of life, and the contacts in `progressed_contacts`, ragged by
/// that row's count.
#[derive(Default)]
struct ProgressionColumns {
    life: Vec<f64>,
    sky: Vec<f64>,
    armc_deg: Vec<f64>,
    ascendant_deg: Vec<f64>,
    midheaven_deg: Vec<f64>,
    arc_deg: Vec<f64>,
    directed_ascendant_deg: Vec<f64>,
    directed_midheaven_deg: Vec<f64>,
    contact_count: Vec<u32>,
    contacts_asked: Vec<u8>,
    progressed_graha: Vec<u16>,
    progressed_longitude_deg: Vec<f64>,
    progressed_tropical_deg: Vec<f64>,
    progressed_speed: Vec<f64>,
    directed_graha: Vec<u16>,
    directed_longitude_deg: Vec<f64>,
    contact_life: Vec<f64>,
    contact_sky: Vec<f64>,
    contact_graha: Vec<u16>,
    contact_to_lagna: Vec<u8>,
    contact_to_graha: Vec<u16>,
    contact_angle: Vec<u16>,
    contact_motion: Vec<u8>,
}

/// The Western aspect tables a batch was asked for, written together since
/// their sections stand together: each chart's own, and each chart's with
/// the record's partner.
struct AspectTables {
    own: WesternAspectColumns,
    across: SynastryColumns,
    declined: DeclinationColumns,
    reflected: AntisciaColumns,
    between: MidpointColumns,
    davisons: DavisonColumns,
}

impl AspectTables {
    fn of(composed: &Composed<'_>, charts: usize) -> Result<AspectTables, Error> {
        Ok(AspectTables {
            own: WesternAspectColumns::of(composed.western_aspects, charts)?,
            across: SynastryColumns::of(composed.synastry, charts)?,
            declined: DeclinationColumns::of(composed.declinations, composed.parallels, charts)?,
            reflected: AntisciaColumns::of(composed.antiscia, charts)?,
            between: MidpointColumns::of(composed.midpoints, charts)?,
            davisons: DavisonColumns::of(composed.davisons, charts)?,
        })
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        self.own.write(writer)?;
        self.across.write(writer)?;
        self.declined.write(writer)?;
        self.across.write_parallels(writer)?;
        self.reflected.write(writer)?;
        self.across.write_antiscia(writer)?;
        self.between.write(writer)?;
        self.across.write_composites(writer)?;
        self.davisons.write(writer)
    }
}

/// `synastry_davisons`: each chart's Davison birth with the synastry's
/// partner, when asked (`western-composites.md`, C248).
#[derive(Default)]
struct DavisonColumns {
    instant: Vec<f64>,
    latitude_deg: Vec<f64>,
    longitude_deg: Vec<f64>,
    altitude_m: Vec<f64>,
    utc_offset_seconds: Vec<i32>,
}

impl DavisonColumns {
    fn of(read: &[teistro::Partner], charts: usize) -> Result<DavisonColumns, Error> {
        one_a_chart(read.len(), charts, "davisons")?;
        let mut columns = DavisonColumns::default();
        for birth in read {
            columns.instant.push(birth.instant.get());
            columns.latitude_deg.push(birth.place.latitude.get());
            columns.longitude_deg.push(birth.place.longitude.get());
            columns.altitude_m.push(birth.place.altitude.get());
            columns.utc_offset_seconds.push(birth.utc_offset.seconds());
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "synastry_davisons",
            self.instant.len(),
            &[
                ColumnData::F64(&self.instant),
                ColumnData::F64(&self.latitude_deg),
                ColumnData::F64(&self.longitude_deg),
                ColumnData::F64(&self.altitude_m),
                ColumnData::I32(&self.utc_offset_seconds),
            ],
        )
    }
}

/// `midpoints` and `midpoint_rows`: each chart's planets equally distant
/// from two others (`western-midpoints.md`).
#[derive(Default)]
struct MidpointColumns {
    count: Vec<u32>,
    first: Vec<u16>,
    second: Vec<u16>,
    middle: Vec<u16>,
    far: Vec<u8>,
    distance_deg: Vec<f64>,
    from_axis_deg: Vec<f64>,
    orb_deg: Vec<f64>,
}

impl MidpointColumns {
    fn of(read: &[Vec<teistro::MidpointRow>], charts: usize) -> Result<MidpointColumns, Error> {
        one_a_chart(read.len(), charts, "midpoints")?;
        let mut columns = MidpointColumns::default();
        for rows in read {
            columns.count.push(row_count(rows.len())?);
            for row in rows {
                columns.first.push(row.first.id());
                columns.second.push(row.second.id());
                columns.middle.push(row.middle.id());
                columns.far.push(u8::from(row.far));
                columns.distance_deg.push(row.distance_deg);
                columns.from_axis_deg.push(row.from_axis_deg);
                columns.orb_deg.push(row.orb_deg);
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "midpoints",
            self.count.len(),
            &[ColumnData::U32(&self.count)],
        )?;
        writer.columns(
            "midpoint_rows",
            self.first.len(),
            &[
                ColumnData::U16(&self.first),
                ColumnData::U16(&self.second),
                ColumnData::U16(&self.middle),
                ColumnData::U8(&self.far),
                ColumnData::F64(&self.distance_deg),
                ColumnData::F64(&self.from_axis_deg),
                ColumnData::F64(&self.orb_deg),
            ],
        )
    }
}

/// `antiscia`, `antiscion_points` and `antiscion_rows`: each chart's
/// planets reflected about the solstices and the equinoxes, and the pairs
/// standing in one (`western-antiscia.md`).
#[derive(Default)]
struct AntisciaColumns {
    point_count: Vec<u32>,
    pair_count: Vec<u32>,
    graha: Vec<u16>,
    antiscion_deg: Vec<f64>,
    contrantiscion_deg: Vec<f64>,
    paired: Vec<u8>,
    pairs: AntiscionRowColumns,
}

/// The pairs in antiscion, one chart's own or across two, as the row
/// sections cross them.
#[derive(Default)]
struct AntiscionRowColumns {
    first: Vec<u16>,
    second: Vec<u16>,
    contrary: Vec<u8>,
    apart_deg: Vec<f64>,
    orb_deg: Vec<f64>,
}

impl AntiscionRowColumns {
    fn push(&mut self, row: &teistro::AntiscionRow) {
        self.first.push(row.first.id());
        self.second.push(row.second.id());
        self.contrary.push(u8::from(row.contrary));
        self.apart_deg.push(row.apart_deg);
        self.orb_deg.push(row.orb_deg);
    }

    fn write(
        &self,
        writer: &mut Writer<'_>,
        section: &str,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            section,
            self.first.len(),
            &[
                ColumnData::U16(&self.first),
                ColumnData::U16(&self.second),
                ColumnData::U8(&self.contrary),
                ColumnData::F64(&self.apart_deg),
                ColumnData::F64(&self.orb_deg),
            ],
        )
    }
}

impl AntisciaColumns {
    fn of(read: &[teistro::Antiscia], charts: usize) -> Result<AntisciaColumns, Error> {
        one_a_chart(read.len(), charts, "antiscia")?;
        let mut columns = AntisciaColumns::default();
        for one in read {
            columns.point_count.push(row_count(one.points.len())?);
            columns.pair_count.push(row_count(one.pairs.len())?);
            for point in &one.points {
                columns.graha.push(point.graha.id());
                columns.antiscion_deg.push(point.antiscion_deg);
                columns.contrantiscion_deg.push(point.contrantiscion_deg);
                columns
                    .paired
                    .push(u8::from(!one.unpaired.contains(&point.graha)));
            }
            for row in &one.pairs {
                columns.pairs.push(row);
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "antiscia",
            self.point_count.len(),
            &[
                ColumnData::U32(&self.point_count),
                ColumnData::U32(&self.pair_count),
            ],
        )?;
        writer.columns(
            "antiscion_points",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::F64(&self.antiscion_deg),
                ColumnData::F64(&self.contrantiscion_deg),
                ColumnData::U8(&self.paired),
            ],
        )?;
        self.pairs.write(writer, "antiscion_rows")
    }
}

/// `declinations`, `declination_rows` and `parallel_rows`: each chart's
/// distances from the equator and the parallels among its planets
/// (`western-declinations.md`).
#[derive(Default)]
struct DeclinationColumns {
    obliquity_deg: Vec<f64>,
    lagna_deg: Vec<f64>,
    midheaven_deg: Vec<f64>,
    graha_count: Vec<u32>,
    parallel_count: Vec<u32>,
    graha: Vec<u16>,
    declination_deg: Vec<f64>,
    first: Vec<u16>,
    second: Vec<u16>,
    contrary: Vec<u8>,
    apart_deg: Vec<f64>,
    orb_deg: Vec<f64>,
}

impl DeclinationColumns {
    fn of(
        declinations: &[teistro::Declinations],
        parallels: &[Vec<teistro::ParallelRow>],
        charts: usize,
    ) -> Result<DeclinationColumns, Error> {
        one_a_chart(declinations.len(), charts, "declinations")?;
        one_a_chart(parallels.len(), declinations.len(), "parallels")?;
        let mut columns = DeclinationColumns::default();
        for (read, rows) in declinations.iter().zip(parallels) {
            columns.obliquity_deg.push(read.obliquity_deg);
            columns.lagna_deg.push(read.lagna_deg);
            columns.midheaven_deg.push(read.midheaven_deg);
            columns.graha_count.push(row_count(read.grahas.len())?);
            columns.parallel_count.push(row_count(rows.len())?);
            for one in &read.grahas {
                columns.graha.push(one.graha.id());
                columns.declination_deg.push(one.declination_deg);
            }
            for row in rows {
                columns.first.push(row.first.id());
                columns.second.push(row.second.id());
                columns.contrary.push(u8::from(row.contrary));
                columns.apart_deg.push(row.apart_deg);
                columns.orb_deg.push(row.orb_deg);
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "declinations",
            self.obliquity_deg.len(),
            &[
                ColumnData::F64(&self.obliquity_deg),
                ColumnData::F64(&self.lagna_deg),
                ColumnData::F64(&self.midheaven_deg),
                ColumnData::U32(&self.graha_count),
                ColumnData::U32(&self.parallel_count),
            ],
        )?;
        writer.columns(
            "declination_rows",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::F64(&self.declination_deg),
            ],
        )?;
        writer.columns(
            "parallel_rows",
            self.first.len(),
            &[
                ColumnData::U16(&self.first),
                ColumnData::U16(&self.second),
                ColumnData::U8(&self.contrary),
                ColumnData::F64(&self.apart_deg),
                ColumnData::F64(&self.orb_deg),
            ],
        )
    }
}

/// Every chart's Western aspects (`western-aspects.md`): a row a chart in
/// `western_aspects` with its count, and the rows ragged under it in
/// `western_aspect_rows`.
#[derive(Default)]
struct WesternAspectColumns {
    count: Vec<u32>,
    first: Vec<u16>,
    second: Vec<u16>,
    measures: AspectMeasures,
    applying: Vec<u8>,
}

impl WesternAspectColumns {
    fn of(
        read: &[Vec<teistro::WesternAspectRow>],
        charts: usize,
    ) -> Result<WesternAspectColumns, Error> {
        one_a_chart(read.len(), charts, "western aspects")?;
        let mut columns = WesternAspectColumns::default();
        for rows in read {
            columns.count.push(row_count(rows.len())?);
            for row in rows {
                columns.first.push(row.first.id());
                columns.second.push(row.second.id());
                columns.measures.push(
                    row.aspect,
                    row.apart_deg,
                    row.from_exact_deg,
                    row.orb_deg,
                )?;
                columns.applying.push(u8::from(row.applying));
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "western_aspects",
            self.count.len(),
            &[ColumnData::U32(&self.count)],
        )?;
        let [aspect, apart, from_exact, orb] = self.measures.columns();
        writer.columns(
            "western_aspect_rows",
            self.first.len(),
            &[
                ColumnData::U16(&self.first),
                ColumnData::U16(&self.second),
                aspect,
                apart,
                from_exact,
                orb,
                ColumnData::U8(&self.applying),
            ],
        )
    }
}

/// `synastry` and `synastry_rows`: each chart's contacts with the record's
/// partner (`western-synastry.md`); `synastry_parallels` and
/// `synastry_parallel_rows`: the parallels across the two, when asked
/// (`western-declinations.md`); `synastry_antiscia` and
/// `synastry_antiscion_rows`: the antiscia across the two, when asked
/// (`western-antiscia.md`).
#[derive(Default)]
struct SynastryColumns {
    count: Vec<u32>,
    parallel_count: Vec<u32>,
    first: PointCells,
    second: PointCells,
    measures: AspectMeasures,
    parallel_first: PointCells,
    parallel_second: PointCells,
    contrary: Vec<u8>,
    parallel_apart_deg: Vec<f64>,
    parallel_orb_deg: Vec<f64>,
    antiscion_count: Vec<u32>,
    antiscia: AntiscionRowColumns,
    composite_lagna_deg: Vec<f64>,
    composite_midheaven_deg: Vec<f64>,
    composite_lagna_turned: Vec<u8>,
    composite_count: Vec<u32>,
    composite_graha: Vec<u16>,
    composite_longitude_deg: Vec<f64>,
    composite_speed_deg_per_day: Vec<f64>,
}

impl SynastryColumns {
    fn of(read: &[teistro::PartnerReading], charts: usize) -> Result<SynastryColumns, Error> {
        one_a_chart(read.len(), charts, "synastries")?;
        let mut columns = SynastryColumns::default();
        for one in read {
            columns.count.push(row_count(one.aspects.len())?);
            if let Some(parallels) = &one.parallels {
                columns.parallel_count.push(row_count(parallels.len())?);
            }
            for row in one.parallels.iter().flatten() {
                columns.parallel_first.push(row.first);
                columns.parallel_second.push(row.second);
                columns.contrary.push(u8::from(row.contrary));
                columns.parallel_apart_deg.push(row.apart_deg);
                columns.parallel_orb_deg.push(row.orb_deg);
            }
            if let Some(antiscia) = &one.antiscia {
                columns.antiscion_count.push(row_count(antiscia.len())?);
            }
            for row in one.antiscia.iter().flatten() {
                columns.antiscia.push(row);
            }
            if let Some(composite) = &one.composite {
                columns.composite_lagna_deg.push(composite.lagna_deg);
                columns
                    .composite_midheaven_deg
                    .push(composite.midheaven_deg);
                columns
                    .composite_lagna_turned
                    .push(u8::from(composite.lagna_turned));
                columns
                    .composite_count
                    .push(row_count(composite.planets.len())?);
                for at in &composite.planets {
                    columns.composite_graha.push(at.graha.id());
                    columns.composite_longitude_deg.push(at.longitude_deg);
                    columns
                        .composite_speed_deg_per_day
                        .push(at.speed_deg_per_day);
                }
            }
            for row in &one.aspects {
                columns.first.push(row.first);
                columns.second.push(row.second);
                columns.measures.push(
                    row.aspect,
                    row.apart_deg,
                    row.from_exact_deg,
                    row.orb_deg,
                )?;
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "synastry",
            self.count.len(),
            &[ColumnData::U32(&self.count)],
        )?;
        let [first_lagna, first_graha] = self.first.columns();
        let [second_lagna, second_graha] = self.second.columns();
        let [aspect, apart, from_exact, orb] = self.measures.columns();
        writer.columns(
            "synastry_rows",
            self.measures.aspect.len(),
            &[
                first_lagna,
                first_graha,
                second_lagna,
                second_graha,
                aspect,
                apart,
                from_exact,
                orb,
            ],
        )
    }

    /// `synastry_parallels` and `synastry_parallel_rows`, after the
    /// declinations'.
    fn write_parallels(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "synastry_parallels",
            self.parallel_count.len(),
            &[ColumnData::U32(&self.parallel_count)],
        )?;
        let [first_lagna, first_graha] = self.parallel_first.columns();
        let [second_lagna, second_graha] = self.parallel_second.columns();
        writer.columns(
            "synastry_parallel_rows",
            self.contrary.len(),
            &[
                first_lagna,
                first_graha,
                second_lagna,
                second_graha,
                ColumnData::U8(&self.contrary),
                ColumnData::F64(&self.parallel_apart_deg),
                ColumnData::F64(&self.parallel_orb_deg),
            ],
        )
    }

    /// `synastry_antiscia` and `synastry_antiscion_rows`, after the
    /// antiscia's.
    fn write_antiscia(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "synastry_antiscia",
            self.antiscion_count.len(),
            &[ColumnData::U32(&self.antiscion_count)],
        )?;
        self.antiscia.write(writer, "synastry_antiscion_rows")
    }

    /// `synastry_composites` and `synastry_composite_rows`, after the
    /// equal distances.
    fn write_composites(
        &self,
        writer: &mut Writer<'_>,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "synastry_composites",
            self.composite_count.len(),
            &[
                ColumnData::F64(&self.composite_lagna_deg),
                ColumnData::F64(&self.composite_midheaven_deg),
                ColumnData::U8(&self.composite_lagna_turned),
                ColumnData::U32(&self.composite_count),
            ],
        )?;
        writer.columns(
            "synastry_composite_rows",
            self.composite_graha.len(),
            &[
                ColumnData::U16(&self.composite_graha),
                ColumnData::F64(&self.composite_longitude_deg),
                ColumnData::F64(&self.composite_speed_deg_per_day),
            ],
        )
    }
}

/// How many rows a chart holds in a ragged section, as its count column
/// says.
fn row_count(rows: usize) -> Result<u32, Error> {
    u32::try_from(rows).map_err(|_| Error::internal("more rows than a section can count"))
}

/// The four cells a Western aspect row measures, whichever two points
/// stand at it, in the order the schema's `western_aspect_measures` lists.
#[derive(Default)]
struct AspectMeasures {
    aspect: Vec<u8>,
    apart_deg: Vec<f64>,
    from_exact_deg: Vec<f64>,
    orb_deg: Vec<f64>,
}

impl AspectMeasures {
    fn push(
        &mut self,
        aspect: teistro::WesternAspect,
        apart_deg: f64,
        from_exact_deg: f64,
        orb_deg: f64,
    ) -> Result<(), Error> {
        let code = TsWesternAspect::of(aspect)
            .ok_or_else(|| no_code(&format!("the aspect {}", aspect.key())))?;
        self.aspect.push(code as u8);
        self.apart_deg.push(apart_deg);
        self.from_exact_deg.push(from_exact_deg);
        self.orb_deg.push(orb_deg);
        Ok(())
    }

    fn columns(&self) -> [ColumnData<'_>; 4] {
        [
            ColumnData::U8(&self.aspect),
            ColumnData::F64(&self.apart_deg),
            ColumnData::F64(&self.from_exact_deg),
            ColumnData::F64(&self.orb_deg),
        ]
    }
}

/// A natal point as two cells a row, as the schema's `point_columns`
/// names them.
#[derive(Default)]
struct PointCells {
    lagna: Vec<u8>,
    graha: Vec<u16>,
}

impl PointCells {
    fn push(&mut self, point: teistro::NatalPoint) {
        let (lagna, graha) = point_cells(Some(point));
        self.lagna.push(lagna);
        self.graha.push(graha);
    }

    fn columns(&self) -> [ColumnData<'_>; 2] {
        [ColumnData::U8(&self.lagna), ColumnData::U16(&self.graha)]
    }
}

/// A natal point as the boundary carries it: 1 and 0 for the lagna, 0 and
/// the graha's id for a graha, and 0 and 0 for none.
fn point_cells(point: Option<teistro::NatalPoint>) -> (u8, u16) {
    match point {
        Some(teistro::NatalPoint::Lagna) => (1, 0),
        Some(teistro::NatalPoint::Graha { graha }) => (0, graha.id()),
        None => (0, 0),
    }
}

impl ProgressionColumns {
    fn of(read: &[teistro::Progressions], charts: usize) -> Result<ProgressionColumns, Error> {
        one_a_chart(read.len(), charts, "progressions")?;
        let mut columns = ProgressionColumns::default();
        for one in read {
            let progressed = one.progressed.as_ref();
            let directed = one.directed.as_ref();
            columns
                .life
                .push(progressed.map_or(f64::NAN, |p| p.life.get()));
            columns
                .sky
                .push(progressed.map_or(f64::NAN, |p| p.sky.get()));
            columns
                .armc_deg
                .push(progressed.map_or(f64::NAN, |p| p.armc_deg));
            columns
                .ascendant_deg
                .push(progressed.map_or(f64::NAN, |p| p.angles.ascendant_deg));
            columns
                .midheaven_deg
                .push(progressed.map_or(f64::NAN, |p| p.angles.midheaven_deg));
            columns
                .arc_deg
                .push(directed.map_or(f64::NAN, |d| d.arc_deg));
            columns
                .directed_ascendant_deg
                .push(directed.map_or(f64::NAN, |d| d.ascendant_deg));
            columns
                .directed_midheaven_deg
                .push(directed.map_or(f64::NAN, |d| d.midheaven_deg));
            columns
                .contacts_asked
                .push(u8::from(one.contacts.is_some()));
            let contacts = one.contacts.as_deref().unwrap_or_default();
            columns.contact_count.push(
                u32::try_from(contacts.len())
                    .map_err(|_| Error::internal("more contacts than a section can count"))?,
            );
            let placed = progressed.map(|p| &p.chart.value.foundation);
            for at in placed
                .into_iter()
                .flat_map(|f| f.grahas.iter().chain(&f.outer))
            {
                columns.progressed_graha.push(at.graha.id());
                columns.progressed_longitude_deg.push(at.longitude_deg);
                columns.progressed_tropical_deg.push(at.tropical_deg);
                columns.progressed_speed.push(at.speed_deg_per_day);
            }
            for at in directed.map_or(&[][..], |d| d.planets.as_slice()) {
                columns.directed_graha.push(at.graha.id());
                columns.directed_longitude_deg.push(at.longitude_deg);
            }
            for contact in contacts {
                columns.contact_life.push(contact.life.get());
                columns.contact_sky.push(contact.sky.get());
                columns.contact_graha.push(contact.graha.id());
                let (to_lagna, to_graha) = point_cells(Some(contact.to));
                columns.contact_to_lagna.push(to_lagna);
                columns.contact_to_graha.push(to_graha);
                columns.contact_angle.push(contact.angle);
                columns
                    .contact_motion
                    .push(TsMotion::from(contact.motion) as u8);
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "progressions",
            self.life.len(),
            &[
                ColumnData::F64(&self.life),
                ColumnData::F64(&self.sky),
                ColumnData::F64(&self.armc_deg),
                ColumnData::F64(&self.ascendant_deg),
                ColumnData::F64(&self.midheaven_deg),
                ColumnData::F64(&self.arc_deg),
                ColumnData::F64(&self.directed_ascendant_deg),
                ColumnData::F64(&self.directed_midheaven_deg),
                ColumnData::U32(&self.contact_count),
                ColumnData::U8(&self.contacts_asked),
            ],
        )?;
        writer.columns(
            "progressed_grahas",
            self.progressed_graha.len(),
            &[
                ColumnData::U16(&self.progressed_graha),
                ColumnData::F64(&self.progressed_longitude_deg),
                ColumnData::F64(&self.progressed_tropical_deg),
                ColumnData::F64(&self.progressed_speed),
            ],
        )?;
        writer.columns(
            "directed_grahas",
            self.directed_graha.len(),
            &[
                ColumnData::U16(&self.directed_graha),
                ColumnData::F64(&self.directed_longitude_deg),
            ],
        )?;
        writer.columns(
            "progressed_contacts",
            self.contact_life.len(),
            &[
                ColumnData::F64(&self.contact_life),
                ColumnData::F64(&self.contact_sky),
                ColumnData::U16(&self.contact_graha),
                ColumnData::U8(&self.contact_to_lagna),
                ColumnData::U16(&self.contact_to_graha),
                ColumnData::U16(&self.contact_angle),
                ColumnData::U8(&self.contact_motion),
            ],
        )
    }
}

/// A planet's dignities as one byte, bit `n` the `n`th of
/// [`teistro::EssentialDignity`]'s flags in its order: house, exaltation,
/// triplicity, term, face, detriment, fall.
fn dignity_bits(d: teistro::EssentialDignity) -> u8 {
    dignity_flags(d)
        .into_iter()
        .enumerate()
        .fold(0, |bits, (n, flag)| bits | (u8::from(flag) << n))
}

/// A Ptolemaic aspect's code.
fn aspect_code(aspect: teistro::PtolemaicAspect) -> Result<u8, Error> {
    TsPtolemaicAspect::of(aspect)
        .map(|code| code as u8)
        .ok_or_else(|| no_code("the aspect"))
}

/// Every chart's perfection between its two significators: a row a chart
/// in `perfection`, its impediments, translations and collections ragged
/// by that row's counts, and the seven orbs a chart in `perfection_orbs`.
#[derive(Default)]
struct PerfectionColumns {
    querent: Vec<u16>,
    quesited: Vec<u16>,
    horizon_days: Vec<f64>,
    horizon_rule_days: Vec<f64>,
    within_sign_rule: Vec<u8>,
    application_present: Vec<u8>,
    application_aspect: Vec<u8>,
    application_days: Vec<f64>,
    applying: Vec<u16>,
    application_kind: Vec<u8>,
    gap_deg: Vec<f64>,
    within_moieties: Vec<u8>,
    separation_present: Vec<u8>,
    separation_aspect: Vec<u8>,
    separation_past_deg: Vec<f64>,
    querent_house: Vec<u8>,
    querent_dignity: Vec<u8>,
    quesited_house: Vec<u8>,
    quesited_dignity: Vec<u8>,
    mutual_by_house: Vec<u8>,
    infortunes_between: Vec<u8>,
    moon_relays: Vec<u8>,
    quesited_in_ascendant: Vec<u8>,
    ways_held: Vec<u8>,
    impediment_count: Vec<u32>,
    translation_count: Vec<u32>,
    collection_count: Vec<u32>,
    /// The `perfection_impediments` section.
    impediment_kind: Vec<u8>,
    impediment_significator: Vec<u16>,
    impediment_third_present: Vec<u8>,
    impediment_third: Vec<u16>,
    impediment_aspect: Vec<u8>,
    impediment_days: Vec<f64>,
    /// The `perfection_translations` section.
    translator: Vec<u16>,
    translated_from: Vec<u16>,
    translated_to: Vec<u16>,
    separating_aspect: Vec<u8>,
    separating_past_deg: Vec<f64>,
    translation_aspect: Vec<u8>,
    translation_days: Vec<f64>,
    received: Vec<u8>,
    /// The `perfection_collections` section.
    collector: Vec<u16>,
    from_querent_aspect: Vec<u8>,
    from_querent_days: Vec<f64>,
    from_quesited_aspect: Vec<u8>,
    from_quesited_days: Vec<f64>,
    collector_in_querent: Vec<u8>,
    collector_in_quesited: Vec<u8>,
    querent_in_collector: Vec<u8>,
    quesited_in_collector: Vec<u8>,
    /// The `perfection_orbs` section.
    orb: Vec<f64>,
}

impl PerfectionColumns {
    fn of(
        read: &[(teistro::Matter, teistro::PerfectionRules)],
        charts: usize,
    ) -> Result<PerfectionColumns, Error> {
        one_a_chart(read.len(), charts, "perfection")?;
        let mut columns = PerfectionColumns::default();
        let count =
            |n: usize| u32::try_from(n).map_err(|_| Error::internal("too many rows for one chart"));
        for (matter, rules) in read {
            columns.querent.push(matter.querent.id());
            columns.quesited.push(matter.quesited.id());
            columns.horizon_days.push(matter.horizon_days);
            columns
                .horizon_rule_days
                .push(rules.horizon_days.unwrap_or(f64::NAN));
            columns.within_sign_rule.push(u8::from(rules.within_sign));
            columns.push_application(matter.application)?;
            columns.push_separation(matter.separation)?;
            columns.push_ways(&matter.ways)?;
            columns
                .impediment_count
                .push(count(matter.impediments.len())?);
            columns
                .translation_count
                .push(count(matter.translations.len())?);
            columns
                .collection_count
                .push(count(matter.collections.len())?);
            for impediment in &matter.impediments {
                columns.push_impediment(impediment)?;
            }
            for translation in &matter.translations {
                columns.push_translation(translation)?;
            }
            for collection in &matter.collections {
                columns.push_collection(collection)?;
            }
            columns.orb.extend(rules.orbs_deg);
        }
        Ok(columns)
    }

    /// The significators' application, or a row saying there is none.
    fn push_application(&mut self, application: Option<teistro::Application>) -> Result<(), Error> {
        if let Some(found) = application {
            self.application_present.push(1);
            self.application_aspect.push(aspect_code(found.aspect)?);
            self.application_days.push(found.days);
            self.applying.push(found.applying.id());
            self.application_kind.push(
                TsApplicationKind::of(found.kind).ok_or_else(|| no_code("the application"))? as u8,
            );
            self.gap_deg.push(found.gap_deg);
            self.within_moieties.push(u8::from(found.within_moieties));
        } else {
            self.application_present.push(0);
            self.application_aspect.push(0);
            self.application_days.push(f64::NAN);
            self.applying.push(0);
            self.application_kind.push(0);
            self.gap_deg.push(f64::NAN);
            self.within_moieties.push(0);
        }
        Ok(())
    }

    /// The significators' separation, or a row saying there is none.
    fn push_separation(&mut self, separation: Option<teistro::Separation>) -> Result<(), Error> {
        if let Some(found) = separation {
            self.separation_present.push(1);
            self.separation_aspect.push(aspect_code(found.aspect)?);
            self.separation_past_deg.push(found.past_deg);
        } else {
            self.separation_present.push(0);
            self.separation_aspect.push(0);
            self.separation_past_deg.push(f64::NAN);
        }
        Ok(())
    }

    /// Where the significators stand and the ways the figure holds.
    fn push_ways(&mut self, ways: &teistro::Ways) -> Result<(), Error> {
        self.querent_house.push(ways.querent.house.get());
        self.querent_dignity
            .push(dignity_bits(ways.querent.dignity));
        self.quesited_house.push(ways.quesited.house.get());
        self.quesited_dignity
            .push(dignity_bits(ways.quesited.dignity));
        self.mutual_by_house.push(u8::from(ways.mutual_by_house));
        self.infortunes_between.push(
            bit_set(ways.infortunes_between.iter().map(|graha| graha.id()))
                .ok_or_else(|| no_code("the infortune"))?,
        );
        self.moon_relays.push(u8::from(ways.moon_relays));
        self.quesited_in_ascendant
            .push(u8::from(ways.quesited_in_ascendant));
        self.ways_held.push(
            ways.held
                .iter()
                .map(|way| TsWay::of(*way).map(|code| u16::from(code as u8)))
                .collect::<Option<Vec<u16>>>()
                .and_then(bit_set)
                .ok_or_else(|| no_code("the way"))?,
        );
        Ok(())
    }

    /// One translation's row.
    fn push_translation(&mut self, translation: &teistro::Translation) -> Result<(), Error> {
        self.translator.push(translation.translator.id());
        self.translated_from.push(translation.from.id());
        self.translated_to.push(translation.to.id());
        self.separating_aspect
            .push(aspect_code(translation.separating.aspect)?);
        self.separating_past_deg
            .push(translation.separating.past_deg);
        self.translation_aspect
            .push(aspect_code(translation.aspect)?);
        self.translation_days.push(translation.days);
        self.received.push(dignity_bits(translation.received));
        Ok(())
    }

    /// One collection's row.
    fn push_collection(&mut self, collection: &teistro::Collection) -> Result<(), Error> {
        self.collector.push(collection.collector.id());
        self.from_querent_aspect
            .push(aspect_code(collection.from_querent.aspect)?);
        self.from_querent_days.push(collection.from_querent.days);
        self.from_quesited_aspect
            .push(aspect_code(collection.from_quesited.aspect)?);
        self.from_quesited_days.push(collection.from_quesited.days);
        self.collector_in_querent
            .push(dignity_bits(collection.collector_in_querent));
        self.collector_in_quesited
            .push(dignity_bits(collection.collector_in_quesited));
        self.querent_in_collector
            .push(dignity_bits(collection.querent_in_collector));
        self.quesited_in_collector
            .push(dignity_bits(collection.quesited_in_collector));
        Ok(())
    }

    /// One impediment's row.
    fn push_impediment(&mut self, impediment: &teistro::Impediment) -> Result<(), Error> {
        self.impediment_kind.push(
            TsImpedimentKind::of(impediment.kind).ok_or_else(|| no_code("the impediment"))? as u8,
        );
        self.impediment_significator
            .push(impediment.significator.id());
        self.impediment_third_present
            .push(u8::from(impediment.third.is_some()));
        self.impediment_third.push(
            impediment
                .third
                .map_or(0, teistro_core::catalogue::Graha::id),
        );
        self.impediment_aspect.push(aspect_code(impediment.aspect)?);
        self.impediment_days.push(impediment.days);
        Ok(())
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "perfection",
            self.querent.len(),
            &[
                ColumnData::U16(&self.querent),
                ColumnData::U16(&self.quesited),
                ColumnData::F64(&self.horizon_days),
                ColumnData::F64(&self.horizon_rule_days),
                ColumnData::U8(&self.within_sign_rule),
                ColumnData::U8(&self.application_present),
                ColumnData::U8(&self.application_aspect),
                ColumnData::F64(&self.application_days),
                ColumnData::U16(&self.applying),
                ColumnData::U8(&self.application_kind),
                ColumnData::F64(&self.gap_deg),
                ColumnData::U8(&self.within_moieties),
                ColumnData::U8(&self.separation_present),
                ColumnData::U8(&self.separation_aspect),
                ColumnData::F64(&self.separation_past_deg),
                ColumnData::U8(&self.querent_house),
                ColumnData::U8(&self.querent_dignity),
                ColumnData::U8(&self.quesited_house),
                ColumnData::U8(&self.quesited_dignity),
                ColumnData::U8(&self.mutual_by_house),
                ColumnData::U8(&self.infortunes_between),
                ColumnData::U8(&self.moon_relays),
                ColumnData::U8(&self.quesited_in_ascendant),
                ColumnData::U8(&self.ways_held),
                ColumnData::U32(&self.impediment_count),
                ColumnData::U32(&self.translation_count),
                ColumnData::U32(&self.collection_count),
            ],
        )?;
        writer.columns(
            "perfection_impediments",
            self.impediment_kind.len(),
            &[
                ColumnData::U8(&self.impediment_kind),
                ColumnData::U16(&self.impediment_significator),
                ColumnData::U8(&self.impediment_third_present),
                ColumnData::U16(&self.impediment_third),
                ColumnData::U8(&self.impediment_aspect),
                ColumnData::F64(&self.impediment_days),
            ],
        )?;
        writer.columns(
            "perfection_translations",
            self.translator.len(),
            &[
                ColumnData::U16(&self.translator),
                ColumnData::U16(&self.translated_from),
                ColumnData::U16(&self.translated_to),
                ColumnData::U8(&self.separating_aspect),
                ColumnData::F64(&self.separating_past_deg),
                ColumnData::U8(&self.translation_aspect),
                ColumnData::F64(&self.translation_days),
                ColumnData::U8(&self.received),
            ],
        )?;
        writer.columns(
            "perfection_collections",
            self.collector.len(),
            &[
                ColumnData::U16(&self.collector),
                ColumnData::U8(&self.from_querent_aspect),
                ColumnData::F64(&self.from_querent_days),
                ColumnData::U8(&self.from_quesited_aspect),
                ColumnData::F64(&self.from_quesited_days),
                ColumnData::U8(&self.collector_in_querent),
                ColumnData::U8(&self.collector_in_quesited),
                ColumnData::U8(&self.querent_in_collector),
                ColumnData::U8(&self.quesited_in_collector),
            ],
        )?;
        writer.columns(
            "perfection_orbs",
            self.orb.len(),
            &[ColumnData::F64(&self.orb)],
        )
    }
}

/// Every chart's considerations before judgement: a row a chart in
/// `considerations`, the Moon's two perfections a chart in
/// `consideration_perfections` (by the sign's end, then within the
/// moieties), and the seven orbs a chart in `consideration_orbs`.
#[derive(Default)]
struct ConsiderationColumns {
    hour_lord: Vec<u16>,
    ascendant_lord: Vec<u16>,
    grounds: Vec<u8>,
    ascendant_sign: Vec<u16>,
    ascendant_degree: Vec<f64>,
    ascendant_early: Vec<u8>,
    ascendant_late: Vec<u8>,
    short_ascension: Vec<u8>,
    moon_sign: Vec<u16>,
    moon_degree: Vec<f64>,
    moon_late: Vec<u8>,
    moon_late_sign: Vec<u8>,
    via_combusta: Vec<u8>,
    days_in_sign: Vec<f64>,
    eased: Vec<u8>,
    seventh_cusp: Vec<f64>,
    seventh_lord: Vec<u16>,
    seventh_infortunes: Vec<u8>,
    lord_retrograde: Vec<u8>,
    lord_combust: Vec<u8>,
    lord_in_fall: Vec<u8>,
    lord_in_infortune_term: Vec<u8>,
    lord_net: Vec<i16>,
    saturn_house: Vec<u8>,
    saturn_retrograde: Vec<u8>,
    ascendant_lord_combust: Vec<u8>,
    moon_late_from: Vec<f64>,
    present: Vec<u8>,
    planet: Vec<u16>,
    aspect: Vec<u8>,
    days: Vec<f64>,
    gap: Vec<f64>,
    orb: Vec<f64>,
}

impl ConsiderationColumns {
    fn of(read: &[teistro::Considerations], charts: usize) -> Result<ConsiderationColumns, Error> {
        one_a_chart(read.len(), charts, "considerations")?;
        let mut columns = ConsiderationColumns::default();
        for one in read {
            let radicality = &one.radicality;
            columns.hour_lord.push(radicality.hour_lord.id());
            columns.ascendant_lord.push(radicality.ascendant_lord.id());
            let grounds = radicality
                .grounds
                .iter()
                .map(|ground| TsRadicalGround::of(*ground).map(|code| u16::from(code as u8)))
                .collect::<Option<Vec<u16>>>()
                .and_then(bit_set)
                .ok_or_else(|| no_code("the radical ground"))?;
            columns.grounds.push(grounds);
            let ascendant = &one.ascendant;
            columns.ascendant_sign.push(ascendant.sign.id());
            columns.ascendant_degree.push(ascendant.degree);
            columns.ascendant_early.push(u8::from(ascendant.early));
            columns.ascendant_late.push(u8::from(ascendant.late));
            columns
                .short_ascension
                .push(u8::from(ascendant.short_ascension));
            let moon = &one.moon;
            columns.moon_sign.push(moon.sign.id());
            columns.moon_degree.push(moon.degree);
            columns.moon_late.push(u8::from(moon.late));
            columns.moon_late_sign.push(u8::from(moon.late_sign));
            columns.via_combusta.push(u8::from(moon.via_combusta));
            columns.days_in_sign.push(moon.course.days_in_sign);
            columns.eased.push(u8::from(moon.course.eased));
            let seventh = &one.seventh;
            columns.seventh_cusp.push(seventh.cusp_deg);
            columns.seventh_lord.push(seventh.lord.id());
            let infortunes = bit_set(seventh.infortunes_in_house.iter().map(|graha| graha.id()))
                .ok_or_else(|| no_code("the infortune"))?;
            columns.seventh_infortunes.push(infortunes);
            columns
                .lord_retrograde
                .push(u8::from(seventh.lord_retrograde));
            columns.lord_combust.push(u8::from(seventh.lord_combust));
            columns.lord_in_fall.push(u8::from(seventh.lord_in_fall));
            columns
                .lord_in_infortune_term
                .push(u8::from(seventh.lord_in_infortune_term));
            columns.lord_net.push(seventh.lord_net);
            columns.saturn_house.push(one.saturn_house.get());
            columns
                .saturn_retrograde
                .push(u8::from(one.saturn_retrograde));
            columns
                .ascendant_lord_combust
                .push(u8::from(one.ascendant_lord_combust));
            columns.moon_late_from.push(one.rules.moon_late_from_deg);
            for perfection in [moon.course.next, moon.course.within_orb] {
                columns.push_perfection(perfection)?;
            }
            columns.orb.extend(one.rules.orbs_deg);
        }
        Ok(columns)
    }

    /// One of the Moon's perfections, or a row saying there is none.
    fn push_perfection(&mut self, perfection: Option<teistro::Perfection>) -> Result<(), Error> {
        if let Some(found) = perfection {
            self.present.push(1);
            self.planet.push(found.planet.id());
            self.aspect.push(
                TsPtolemaicAspect::of(found.aspect).ok_or_else(|| no_code("the aspect"))? as u8,
            );
            self.days.push(found.days);
            self.gap.push(found.gap_deg);
        } else {
            self.present.push(0);
            self.planet.push(0);
            self.aspect.push(0);
            self.days.push(f64::NAN);
            self.gap.push(f64::NAN);
        }
        Ok(())
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "considerations",
            self.hour_lord.len(),
            &[
                ColumnData::U16(&self.hour_lord),
                ColumnData::U16(&self.ascendant_lord),
                ColumnData::U8(&self.grounds),
                ColumnData::U16(&self.ascendant_sign),
                ColumnData::F64(&self.ascendant_degree),
                ColumnData::U8(&self.ascendant_early),
                ColumnData::U8(&self.ascendant_late),
                ColumnData::U8(&self.short_ascension),
                ColumnData::U16(&self.moon_sign),
                ColumnData::F64(&self.moon_degree),
                ColumnData::U8(&self.moon_late),
                ColumnData::U8(&self.moon_late_sign),
                ColumnData::U8(&self.via_combusta),
                ColumnData::F64(&self.days_in_sign),
                ColumnData::U8(&self.eased),
                ColumnData::F64(&self.seventh_cusp),
                ColumnData::U16(&self.seventh_lord),
                ColumnData::U8(&self.seventh_infortunes),
                ColumnData::U8(&self.lord_retrograde),
                ColumnData::U8(&self.lord_combust),
                ColumnData::U8(&self.lord_in_fall),
                ColumnData::U8(&self.lord_in_infortune_term),
                ColumnData::I16(&self.lord_net),
                ColumnData::U8(&self.saturn_house),
                ColumnData::U8(&self.saturn_retrograde),
                ColumnData::U8(&self.ascendant_lord_combust),
                ColumnData::F64(&self.moon_late_from),
            ],
        )?;
        writer.columns(
            "consideration_perfections",
            self.present.len(),
            &[
                ColumnData::U8(&self.present),
                ColumnData::U16(&self.planet),
                ColumnData::U8(&self.aspect),
                ColumnData::F64(&self.days),
                ColumnData::F64(&self.gap),
            ],
        )?;
        writer.columns(
            "consideration_orbs",
            self.orb.len(),
            &[ColumnData::F64(&self.orb)],
        )
    }
}

/// Every chart's lots: a row a chart in `lots`, and fourteen in
/// `lot_places`.
#[derive(Default)]
struct LotColumns {
    sect: Vec<u8>,
    sect_rule: Vec<u8>,
    fortune: Vec<u8>,
    fortune_reversed: Vec<u8>,
    lot: Vec<u8>,
    longitude: Vec<f64>,
    sign: Vec<u16>,
    lord: Vec<u16>,
    house: Vec<u8>,
}

impl LotColumns {
    fn of(read: &[teistro::LotReading], charts: usize) -> Result<LotColumns, Error> {
        one_a_chart(read.len(), charts, "lots")?;
        let mut columns = LotColumns::default();
        for one in read {
            columns.sect.push(TsSect::from(one.sect) as u8);
            columns.sect_rule.push(
                TsSectRule::of(one.request.sect_rule()).ok_or_else(|| no_code("the sect rule"))?
                    as u8,
            );
            columns.fortune.push(
                TsFortuneRule::of(one.request.fortune())
                    .ok_or_else(|| no_code("the Fortune rule"))? as u8,
            );
            columns
                .fortune_reversed
                .push(u8::from(one.fortune_reversed));
            for placed in &one.lots {
                columns
                    .lot
                    .push(TsLot::of(placed.lot).ok_or_else(|| no_code("the lot"))? as u8);
                columns.longitude.push(placed.place.longitude_deg);
                columns.sign.push(placed.place.sign.id());
                columns.lord.push(placed.place.lord.id());
                columns.house.push(placed.place.house.get());
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "lots",
            self.sect.len(),
            &[
                ColumnData::U8(&self.sect),
                ColumnData::U8(&self.sect_rule),
                ColumnData::U8(&self.fortune),
                ColumnData::U8(&self.fortune_reversed),
            ],
        )?;
        writer.columns(
            "lot_places",
            self.lot.len(),
            &[
                ColumnData::U8(&self.lot),
                ColumnData::F64(&self.longitude),
                ColumnData::U16(&self.sign),
                ColumnData::U16(&self.lord),
                ColumnData::U8(&self.house),
            ],
        )
    }
}

/// Lilly's accidental lines as the `fortitudes` section's `score_*`
/// columns carry them, in [`teistro::AccidentalScores`]' order after its
/// houses; the schema names each column from this list.
pub(crate) const ACCIDENTAL_LINES: [&str; 26] = [
    "direct",
    "retrograde",
    "swift",
    "slow",
    "superior_oriental",
    "superior_occidental",
    "inferior_oriental",
    "inferior_occidental",
    "increasing",
    "decreasing",
    "free_from_combustion",
    "cazimi",
    "combust",
    "under_beams",
    "conjunct_benefic",
    "conjunct_north_node",
    "trine_benefic",
    "sextile_benefic",
    "conjunct_malefic",
    "conjunct_south_node",
    "opposed_malefic",
    "square_malefic",
    "besieged",
    "regulus",
    "spica",
    "algol",
];

/// A table's line scores in [`ACCIDENTAL_LINES`]' order.
const fn accidental_lines(s: &teistro::AccidentalScores) -> [i8; 26] {
    [
        s.direct,
        s.retrograde,
        s.swift,
        s.slow,
        s.superior_oriental,
        s.superior_occidental,
        s.inferior_oriental,
        s.inferior_occidental,
        s.increasing,
        s.decreasing,
        s.free_from_combustion,
        s.cazimi,
        s.combust,
        s.under_beams,
        s.conjunct_benefic,
        s.conjunct_north_node,
        s.trine_benefic,
        s.sextile_benefic,
        s.conjunct_malefic,
        s.conjunct_south_node,
        s.opposed_malefic,
        s.square_malefic,
        s.besieged,
        s.regulus,
        s.spica,
        s.algol,
    ]
}

/// Every chart's accidental fortitudes: a row a chart in `fortitudes`,
/// twelve houses a chart in `fortitude_houses`, the seven planets a chart
/// in `fortitude_planets`, in the Chaldean order, and their accidents in
/// `fortitude_accidents`, ragged by `accident_count`; all empty when none
/// was asked for. The essential half is [`DignityColumns`]'.
#[derive(Default)]
struct FortitudeColumns {
    houses: Vec<u16>,
    /// The ascendant, the midheaven, the North Node, Regulus, Spica and
    /// Algol.
    points: [Vec<f64>; 6],
    /// The combustion, beams, cazimi, cusp and star orbs.
    orbs: [Vec<f64>; 5],
    combustion_in_sign: Vec<u8>,
    partile: Vec<u8>,
    partile_orb: Vec<f64>,
    siege: Vec<u8>,
    siege_span: Vec<f64>,
    almuten_place: Vec<u8>,
    almuten_fortune: Vec<u8>,
    fortune: Vec<f64>,
    /// The line scores, one column a line in [`ACCIDENTAL_LINES`]' order.
    scores: Vec<Vec<i8>>,
    /// The `fortitude_houses` section.
    cusp: Vec<f64>,
    house_score: Vec<i8>,
    /// Each house's almuten totals, a column a planet in the Chaldean
    /// order.
    house_almuten: [Vec<i16>; 7],
    /// The `fortitude_planets` section.
    planet: Vec<u16>,
    speed: Vec<f64>,
    mean_motion: Vec<f64>,
    house: Vec<u8>,
    fortitude: Vec<i16>,
    debility: Vec<i16>,
    places: Vec<i16>,
    accident_count: Vec<u8>,
    /// The `fortitude_accidents` section.
    accident: Vec<u8>,
    accident_points: Vec<i8>,
}

impl FortitudeColumns {
    fn of(read: &[teistro::Fortitudes], charts: usize) -> Result<FortitudeColumns, Error> {
        one_a_chart(read.len(), charts, "fortitudes")?;
        let mut columns = FortitudeColumns {
            scores: vec![Vec::new(); ACCIDENTAL_LINES.len()],
            ..FortitudeColumns::default()
        };
        for one in read {
            let (sky, rules) = (&one.sky, &one.rules);
            columns.houses.push(sky.houses.id());
            let points = [
                sky.ascendant_deg,
                sky.midheaven_deg,
                sky.north_node_deg,
                sky.regulus_deg,
                sky.spica_deg,
                sky.algol_deg,
            ];
            for (column, value) in columns.points.iter_mut().zip(points) {
                column.push(value);
            }
            let orbs = [
                rules.combustion_deg,
                rules.beams_deg,
                rules.cazimi_deg,
                rules.cusp_orb_deg,
                rules.star_orb_deg,
            ];
            for (column, value) in columns.orbs.iter_mut().zip(orbs) {
                column.push(value);
            }
            columns
                .combustion_in_sign
                .push(u8::from(rules.combustion_in_sign));
            let (partile, orb) =
                TsPartile::of(rules.partile).ok_or_else(|| no_code("the partile reading"))?;
            columns.partile.push(partile as u8);
            columns.partile_orb.push(orb);
            let (siege, span) =
                TsSiege::of(rules.siege).ok_or_else(|| no_code("the siege reading"))?;
            columns.siege.push(siege as u8);
            columns.siege_span.push(span);
            let almutens = &one.almutens;
            columns.almuten_place.push(
                TsPlaceReading::of(almutens.rules.place)
                    .ok_or_else(|| no_code("the place reading"))? as u8,
            );
            columns.almuten_fortune.push(
                TsFortuneRule::of(almutens.rules.fortune)
                    .ok_or_else(|| no_code("the Fortune rule"))? as u8,
            );
            columns.fortune.push(almutens.fortune_deg);
            for (column, value) in columns.scores.iter_mut().zip(accidental_lines(&one.scores)) {
                column.push(value);
            }
            columns.cusp.extend(sky.cusps_deg);
            columns.house_score.extend(one.scores.houses);
            for house in &almutens.houses {
                for (column, total) in columns.house_almuten.iter_mut().zip(house.totals) {
                    column.push(total);
                }
            }
            for (((at, speed), mean), places) in one
                .planets
                .iter()
                .zip(sky.speeds_deg_per_day)
                .zip(rules.mean_motion_deg)
                .zip(almutens.places.totals)
            {
                columns.planet.push(at.planet.id());
                columns.speed.push(speed);
                columns.mean_motion.push(mean);
                columns.house.push(at.house.get());
                columns.fortitude.push(at.fortitude);
                columns.debility.push(at.debility);
                columns.places.push(places);
                columns.accident_count.push(
                    u8::try_from(at.accidents.len())
                        .map_err(|_| Error::internal("more accidents than lines"))?,
                );
                for &accident in &at.accidents {
                    columns.accident.push(
                        TsAccident::of(accident).ok_or_else(|| no_code("the accident"))? as u8,
                    );
                    columns
                        .accident_points
                        .push(one.scores.points(at.planet, accident));
                }
            }
        }
        Ok(columns)
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        let [ascendant, midheaven, north_node, regulus, spica, algol] = &self.points;
        let [combustion, beams, cazimi, cusp_orb, star_orb] = &self.orbs;
        let mut chart = vec![
            ColumnData::U16(&self.houses),
            ColumnData::F64(ascendant),
            ColumnData::F64(midheaven),
            ColumnData::F64(north_node),
            ColumnData::F64(regulus),
            ColumnData::F64(spica),
            ColumnData::F64(algol),
            ColumnData::F64(combustion),
            ColumnData::U8(&self.combustion_in_sign),
            ColumnData::F64(beams),
            ColumnData::F64(cazimi),
            ColumnData::F64(cusp_orb),
            ColumnData::F64(star_orb),
            ColumnData::U8(&self.partile),
            ColumnData::F64(&self.partile_orb),
            ColumnData::U8(&self.siege),
            ColumnData::F64(&self.siege_span),
            ColumnData::U8(&self.almuten_place),
            ColumnData::U8(&self.almuten_fortune),
            ColumnData::F64(&self.fortune),
        ];
        chart.extend(self.scores.iter().map(|column| ColumnData::I8(column)));
        writer.columns("fortitudes", self.houses.len(), &chart)?;
        let mut houses = vec![
            ColumnData::F64(&self.cusp),
            ColumnData::I8(&self.house_score),
        ];
        houses.extend(
            self.house_almuten
                .iter()
                .map(|column| ColumnData::I16(column)),
        );
        writer.columns("fortitude_houses", self.cusp.len(), &houses)?;
        writer.columns(
            "fortitude_planets",
            self.planet.len(),
            &[
                ColumnData::U16(&self.planet),
                ColumnData::F64(&self.speed),
                ColumnData::F64(&self.mean_motion),
                ColumnData::U8(&self.house),
                ColumnData::I16(&self.fortitude),
                ColumnData::I16(&self.debility),
                ColumnData::I16(&self.places),
                ColumnData::U8(&self.accident_count),
            ],
        )?;
        writer.columns(
            "fortitude_accidents",
            self.accident.len(),
            &[
                ColumnData::U8(&self.accident),
                ColumnData::I8(&self.accident_points),
            ],
        )
    }
}

/// Every chart's Vaiseshikamsa, a row a graha, empty when it was not asked
/// for.
struct VaiseshikamsaColumns {
    graha: Vec<u16>,
    impaired: Vec<u8>,
    good: Vec<Vec<u8>>,
    name: Vec<Vec<u16>>,
}

impl VaiseshikamsaColumns {
    fn of(documents: &[Document]) -> VaiseshikamsaColumns {
        let grahas: Vec<_> = documents
            .iter()
            .filter_map(|d| d.vaiseshikamsa.as_ref())
            .flat_map(|reading| &reading.grahas)
            .collect();
        let rows = grahas.len();
        let mut columns = VaiseshikamsaColumns {
            graha: Vec::with_capacity(rows),
            impaired: Vec::with_capacity(rows),
            good: VAISESHIKAMSA_SCHEMES
                .iter()
                .map(|_| Vec::with_capacity(rows))
                .collect(),
            name: VAISESHIKAMSA_SCHEMES
                .iter()
                .map(|_| Vec::with_capacity(rows))
                .collect(),
        };
        for graha in grahas {
            columns.graha.push(graha.graha.id());
            columns.impaired.push(u8::from(graha.impaired));
            for ((good, name), (_, standing)) in columns
                .good
                .iter_mut()
                .zip(columns.name.iter_mut())
                .zip(VAISESHIKAMSA_SCHEMES)
            {
                let standing = standing(graha);
                good.push(standing.good_vargas);
                name.push(
                    standing
                        .name
                        .map_or(0, teistro_core::catalogue::Vaiseshikamsa::id),
                );
            }
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        let mut data = vec![ColumnData::U16(&self.graha), ColumnData::U8(&self.impaired)];
        for (good, name) in self.good.iter().zip(&self.name) {
            data.push(ColumnData::U8(good));
            data.push(ColumnData::U16(name));
        }
        writer.columns("vaiseshikamsa", self.graha.len(), &data)
    }
}

/// A Bhava bala value column: its name, what it holds, and where a bhava's
/// reading keeps it.
pub(crate) type BhavaBalaColumn = (
    &'static str,
    &'static str,
    fn(&teistro::strength::BhavaStrength) -> f64,
);

/// The `bhava_bala` section's value columns in order, which the section's
/// schema and its writer both read.
pub(crate) const BHAVA_BALA_COLUMNS: [BhavaBalaColumn; 5] = [
    ("adhipati", "The lord's Shadbala.", |b| b.adhipati),
    ("dig", "From its direction, 0 to 60.", |b| b.dig),
    (
        "drishti",
        "From the drishtis it receives, which may be negative.",
        |b| b.drishti,
    ),
    (
        "special",
        "From its occupants and its sign's rising, under BPHS's special rules.",
        |b| b.special,
    ),
    ("virupas", "The four together.", |b| b.virupas),
];

/// Every chart's Bhava bala, a row a bhava, empty when it was not asked for.
struct BhavaBalaColumns {
    lord: Vec<u16>,
    values: Vec<Vec<f64>>,
}

impl BhavaBalaColumns {
    fn of(documents: &[Document]) -> BhavaBalaColumns {
        let bhavas: Vec<_> = documents
            .iter()
            .filter_map(|d| d.bhava_bala.as_ref())
            .flat_map(|reading| &reading.bhavas)
            .collect();
        let mut columns = BhavaBalaColumns {
            lord: Vec::with_capacity(bhavas.len()),
            values: BHAVA_BALA_COLUMNS
                .iter()
                .map(|_| Vec::with_capacity(bhavas.len()))
                .collect(),
        };
        for bhava in bhavas {
            columns.lord.push(bhava.lord.id());
            for (column, (_, _, read)) in columns.values.iter_mut().zip(BHAVA_BALA_COLUMNS) {
                column.push(read(bhava));
            }
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        let mut data = Vec::with_capacity(self.values.len() + 1);
        data.push(ColumnData::U16(&self.lord));
        data.extend(self.values.iter().map(|column| ColumnData::F64(column)));
        writer.columns("bhava_bala", self.lord.len(), &data)
    }
}

/// Every chart's Vimshopaka, a row a graha, empty when it was not asked for.
struct VimshopakaColumns {
    graha: Vec<u16>,
    scoring: Vec<u8>,
    shadvarga: Vec<f64>,
    saptavarga: Vec<f64>,
    dashavarga: Vec<f64>,
    shodashavarga: Vec<f64>,
}

impl VimshopakaColumns {
    fn of(documents: &[Document]) -> VimshopakaColumns {
        let rows = documents
            .iter()
            .filter_map(|d| d.vimshopaka.as_ref())
            .map(|reading| reading.grahas.len())
            .sum();
        let mut columns = VimshopakaColumns {
            graha: Vec::with_capacity(rows),
            scoring: Vec::with_capacity(rows),
            shadvarga: Vec::with_capacity(rows),
            saptavarga: Vec::with_capacity(rows),
            dashavarga: Vec::with_capacity(rows),
            shodashavarga: Vec::with_capacity(rows),
        };
        for reading in documents.iter().filter_map(|d| d.vimshopaka.as_ref()) {
            for graha in &reading.grahas {
                columns.graha.push(graha.graha.id());
                columns
                    .scoring
                    .push(TsVimshopakaScoring::from(reading.scoring) as u8);
                columns.shadvarga.push(graha.shadvarga);
                columns.saptavarga.push(graha.saptavarga);
                columns.dashavarga.push(graha.dashavarga);
                columns.shodashavarga.push(graha.shodashavarga);
            }
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "vimshopaka",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::U8(&self.scoring),
                ColumnData::F64(&self.shadvarga),
                ColumnData::F64(&self.saptavarga),
                ColumnData::F64(&self.dashavarga),
                ColumnData::F64(&self.shodashavarga),
            ],
        )
    }
}

struct PointColumns {
    counts: Vec<u32>,
    point: Vec<u16>,
    longitude: Vec<f64>,
    sign: Vec<u16>,
    sign_deg: Vec<f64>,
    nakshatra_deg: Vec<f64>,
    pada_deg: Vec<f64>,
}

impl PointColumns {
    fn of(documents: &[Document]) -> PointColumns {
        let rows: usize = documents
            .iter()
            .map(|d| d.points.as_ref().map_or(0, |p| p.all().len()))
            .sum();
        let mut columns = PointColumns {
            counts: Vec::with_capacity(documents.len()),
            point: Vec::with_capacity(rows),
            longitude: Vec::with_capacity(rows),
            sign: Vec::with_capacity(rows),
            sign_deg: Vec::with_capacity(rows),
            nakshatra_deg: Vec::with_capacity(rows),
            pada_deg: Vec::with_capacity(rows),
        };
        for document in documents {
            let Some(points) = document.points.as_ref() else {
                columns.counts.push(0);
                continue;
            };
            columns
                .counts
                .push(u32::try_from(points.all().len()).unwrap_or(u32::MAX));
            for found in points.all() {
                columns.point.push(found.point.id());
                columns.longitude.push(found.longitude_deg);
                columns.sign.push(found.sign.id());
                columns.sign_deg.push(found.boundaries.sign_deg);
                columns.nakshatra_deg.push(found.boundaries.nakshatra_deg);
                columns.pada_deg.push(found.boundaries.pada_deg);
            }
        }
        columns
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "points",
            self.point.len(),
            &[
                ColumnData::U16(&self.point),
                ColumnData::F64(&self.longitude),
                ColumnData::U16(&self.sign),
                ColumnData::F64(&self.sign_deg),
                ColumnData::F64(&self.nakshatra_deg),
                ColumnData::F64(&self.pada_deg),
            ],
        )
    }
}

/// The divisional charts of a batch, as the two sections carry them.
///
/// **Charts outermost, then charts asked for, then grahas** — the same
/// ordering rule every per-chart section in this blob follows, so a
/// decoder slices by arithmetic rather than by searching.
///
/// A batch whose documents hold different numbers of divisional charts
/// is `INTERNAL` for the reason a batch of differing graha counts is:
/// the layout is one count for the batch, and only this crate could have
/// built such a value.
struct VargaColumns {
    /// How many divisional charts each document holds.
    count: u32,
    ids: Vec<u16>,
    lagna_rashi: Vec<u16>,
    lagna_part: Vec<u16>,
    lagna_sign: Vec<u16>,
    rashi: Vec<u16>,
    part: Vec<u16>,
    sign: Vec<u16>,
}

impl VargaColumns {
    fn of(documents: &[Document], graha_count: usize) -> Result<VargaColumns, Error> {
        let count = documents.first().map_or(0, |d| d.vargas.len());
        if let Some(odd) = documents.iter().find(|d| d.vargas.len() != count) {
            return Err(Error::new(
                Status::Internal,
                format!(
                    "the batch mixes divisional chart counts: {count} and {}, though every document was read from one request",
                    odd.vargas.len()
                ),
            ));
        }
        let charts = documents.len() * count;
        let mut columns = VargaColumns {
            count: u32::try_from(count).unwrap_or(u32::MAX),
            ids: Vec::with_capacity(charts),
            lagna_rashi: Vec::with_capacity(charts),
            lagna_part: Vec::with_capacity(charts),
            lagna_sign: Vec::with_capacity(charts),
            rashi: Vec::with_capacity(charts * graha_count),
            part: Vec::with_capacity(charts * graha_count),
            sign: Vec::with_capacity(charts * graha_count),
        };
        for document in documents {
            for varga in &document.vargas {
                // The axis's own chart, which for every axis this
                // boundary can ask for is one catalogued member: the
                // request takes `Varga` ids, so a mixed axis or an
                // arbitrary D-N is reachable in Rust and not here
                // (`03-design/chart-reading.md` §8).
                columns.ids.push(
                    varga
                        .axis
                        .grahas
                        .varga
                        .map_or(u16::MAX, teistro_core::catalogue::Catalogued::id),
                );
                columns.lagna_rashi.push(varga.lagna.rashi.id());
                columns.lagna_part.push(varga.lagna.part);
                columns.lagna_sign.push(varga.lagna.sign.id());
                if varga.grahas.len() != graha_count {
                    return Err(Error::new(
                        Status::Internal,
                        format!(
                            "a divisional chart holds {} grahas where the foundation holds {graha_count}",
                            varga.grahas.len()
                        ),
                    ));
                }
                for placed in &varga.grahas {
                    columns.rashi.push(placed.at.rashi.id());
                    columns.part.push(placed.at.part);
                    columns.sign.push(placed.at.sign.id());
                }
            }
        }
        Ok(columns)
    }

    /// Both sections, written where the schema declares them.
    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "vargas",
            self.ids.len(),
            &[
                ColumnData::U16(&self.ids),
                ColumnData::U16(&self.lagna_rashi),
                ColumnData::U16(&self.lagna_part),
                ColumnData::U16(&self.lagna_sign),
            ],
        )?;
        writer.columns(
            "varga_grahas",
            self.rashi.len(),
            &[
                ColumnData::U16(&self.rashi),
                ColumnData::U16(&self.part),
                ColumnData::U16(&self.sign),
            ],
        )
    }
}

/// Every chart's annual-chart instants, concatenated and ragged.
///
/// The count is per chart and not per batch for a reason the pass
/// measured: the request settles how many returns are *wanted* and the
/// ephemeris settles how many there *are*
/// (`03-design/annual-chart-measured.md`).
struct PraveshaColumns {
    counts: Vec<u32>,
    years: Vec<u16>,
    jds: Vec<f64>,
    muntha_signs: Vec<u16>,
    muntha_lords: Vec<u16>,
    muntha_degs: Vec<f64>,
    /// The `annual_charts` section, one row per return or none at all.
    annual: AnnualColumns,
    /// How many of `natal` each chart holds: `cast.natal_saham_count`.
    natal_counts: Vec<u32>,
    /// The births' own sahams, the `natal_sahams` section.
    natal: SahamColumns,
}

/// The years' own charts, parallel to `praveshas` row for row when a place
/// was asked for and empty when none was.
///
/// All or none, never some: every year of a batch is founded at the one
/// place the request named, so a year with no chart is a year whose
/// founding failed, and that is refused rather than written as a gap.
#[derive(Default)]
struct AnnualColumns {
    lagnas: Vec<f64>,
    daylight: Vec<u8>,
    janma_lagna: Vec<u16>,
    varsha_lagna: Vec<u16>,
    tri_rashi: Vec<u16>,
    dina_ratri: Vec<u16>,
    year_lord: Vec<u16>,
    chosen: Vec<u8>,
    year_lord_vishwa: Vec<i32>,
    moon_passed_over: Vec<u8>,
    claim_counts: Vec<u8>,
    yoga_counts: Vec<u8>,
    retrograde: Vec<u8>,
    combust: Vec<u8>,
    matter_counts: Vec<u8>,
    /// The `year_claims` section: every year's claimants concatenated, in
    /// the order each year ranks them, ragged by `claim_count`.
    claims: ClaimColumns,
    /// The `year_yogas` section: every year's yoga-making pairs, ragged by
    /// `yoga_count`.
    yogas: YogaColumns,
    /// The `year_matters` section and the two under it, ragged by
    /// `matter_count`.
    matters: MatterColumns,
    saham_counts: Vec<u8>,
    /// The `year_sahams` section, ragged by `saham_count`, and the seven
    /// rows under each.
    sahams: SahamColumns,
    /// The `year_harsha` section, seven rows a year.
    harsha: HarshaColumns,
    dasha_counts: Vec<u8>,
    /// The `year_dashas` section and the two under it, ragged by
    /// `dasha_count`; written after the births' sahams, whose ids come
    /// first.
    dashas: YearDashaColumns,
}

/// A chart's sahams, flat and ragged, each where it fell, what it fell in
/// and its strength clause by clause, with the seven planets' facts under
/// each (`03-design/tajika-saham-strength.md`).
///
/// One shape for the years' sahams and the births', so each binding
/// decodes a saham in one place.
#[derive(Default)]
struct SahamColumns {
    saham: Vec<u8>,
    longitude_deg: Vec<f64>,
    sign: Vec<u16>,
    lord: Vec<u16>,
    house: Vec<u8>,
    added_sign: Vec<u8>,
    strong: Vec<u16>,
    weak: Vec<u8>,
    lord_vishwa: Vec<i32>,
    lord_harsha: Vec<u8>,
    node_axis: Vec<u8>,
    /// Seven rows under each saham, the catalogue's order.
    seven_graha: Vec<u16>,
    seven_drishti: Vec<u8>,
    seven_relation: Vec<u8>,
    seven_company: Vec<u8>,
}

/// What a saham row's `node_axis` holds when the chart placed no nodes.
const NODE_AXIS_UNREAD: u8 = 2;

impl SahamColumns {
    fn push(&mut self, one: &teistro::SahamStrength) {
        let place = &one.place;
        self.saham.push(TsSaham::from(one.saham) as u8);
        self.longitude_deg.push(place.longitude_deg);
        self.sign.push(place.sign.id());
        self.lord.push(place.lord.id());
        self.house.push(place.house.get());
        self.added_sign.push(u8::from(place.added_sign));
        self.strong
            .push(one.strong().iter().fold(0, |bits, (clause, holds)| {
                if *holds {
                    bits | 1_u16 << TsSahamStrong::from(*clause) as u8
                } else {
                    bits
                }
            }));
        self.weak
            .push(one.weak().iter().fold(0, |bits, (clause, holds)| {
                if *holds {
                    bits | 1_u8 << TsSahamWeak::from(*clause) as u8
                } else {
                    bits
                }
            }));
        self.lord_vishwa.push(sub_sub(one.lord_vishwa));
        self.lord_harsha
            .push(TsHarshaGrade::from(one.lord_harsha) as u8);
        self.node_axis
            .push(one.in_node_axis.map_or(NODE_AXIS_UNREAD, u8::from));
        for (at, graha) in teistro::tajika::SEVEN.iter().enumerate() {
            self.seven_graha.push(graha.id());
            self.seven_drishti.push(
                one.aspects
                    .get(at)
                    .map_or(TsTajikaDrishti::None, |drishti| {
                        TsTajikaDrishti::from(*drishti)
                    }) as u8,
            );
            self.seven_relation.push(
                one.relations
                    .get(at)
                    .map_or(TsTajikaRelation::Neutral, |relation| {
                        TsTajikaRelation::from(*relation)
                    }) as u8,
            );
            self.seven_company
                .push(u8::from(one.company.get(at).copied().unwrap_or(false)));
        }
    }

    /// The sahams under `name`, and their seven rows under `seven`.
    fn write(
        &self,
        writer: &mut Writer<'_>,
        name: &str,
        seven: &str,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            name,
            self.saham.len(),
            &[
                ColumnData::U8(&self.saham),
                ColumnData::F64(&self.longitude_deg),
                ColumnData::U16(&self.sign),
                ColumnData::U16(&self.lord),
                ColumnData::U8(&self.house),
                ColumnData::U8(&self.added_sign),
                ColumnData::U16(&self.strong),
                ColumnData::U8(&self.weak),
                ColumnData::I32(&self.lord_vishwa),
                ColumnData::U8(&self.lord_harsha),
                ColumnData::U8(&self.node_axis),
            ],
        )?;
        writer.columns(
            seven,
            self.seven_graha.len(),
            &[
                ColumnData::U16(&self.seven_graha),
                ColumnData::U8(&self.seven_drishti),
                ColumnData::U8(&self.seven_relation),
                ColumnData::U8(&self.seven_company),
            ],
        )
    }
}

/// Every founded year's Harsha bala: seven rows each, the catalogue's
/// order (`03-design/tajika-harsha.md`).
#[derive(Default)]
struct HarshaColumns {
    graha: Vec<u16>,
    house: Vec<u8>,
    sthana: Vec<u8>,
    uchcha_swakshetra: Vec<u8>,
    stri_purusha: Vec<u8>,
    dina_ratri: Vec<u8>,
    total: Vec<u8>,
    grade: Vec<u8>,
}

impl HarshaColumns {
    fn push(&mut self, seven: &[teistro::Harsha; 7]) {
        for one in seven {
            self.graha.push(one.graha.id());
            self.house.push(one.house.get());
            self.sthana.push(u8::from(one.sthana));
            self.uchcha_swakshetra.push(u8::from(one.uchcha_swakshetra));
            self.stri_purusha.push(u8::from(one.stri_purusha));
            self.dina_ratri.push(u8::from(one.dina_ratri));
            self.total
                .push(u8::try_from(one.total.units()).unwrap_or(u8::MAX));
            self.grade.push(TsHarshaGrade::from(one.grade) as u8);
        }
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "year_harsha",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::U8(&self.house),
                ColumnData::U8(&self.sthana),
                ColumnData::U8(&self.uchcha_swakshetra),
                ColumnData::U8(&self.stri_purusha),
                ColumnData::U8(&self.dina_ratri),
                ColumnData::U8(&self.total),
                ColumnData::U8(&self.grade),
            ],
        )
    }
}

/// How two planets stand, as the matter sections carry it: `year_yogas`'
/// own columns and a presence flag on the yoga, because a pair the matter
/// sections carry may make none.
///
/// One shape for the matter's own pair and for every leg, so each binding
/// decodes a pair in one place.
#[derive(Default)]
struct PairColumns {
    faster: Vec<u16>,
    slower: Vec<u16>,
    drishti: Vec<u8>,
    yoga: Vec<u8>,
    yoga_present: Vec<u8>,
    orb_deg: Vec<f64>,
    apart_deg: Vec<f64>,
}

impl PairColumns {
    fn push(&mut self, pair: Option<&teistro::Between>) {
        let Some(pair) = pair else {
            // Absent: a row of noughts, read only when the section's own
            // flag says the pair is there.
            self.faster.push(0);
            self.slower.push(0);
            self.drishti.push(0);
            self.yoga.push(0);
            self.yoga_present.push(0);
            self.orb_deg.push(0.0);
            self.apart_deg.push(0.0);
            return;
        };
        self.faster.push(pair.faster.id());
        self.slower.push(pair.slower.id());
        self.drishti.push(TsTajikaDrishti::from(pair.drishti) as u8);
        self.yoga
            .push(pair.yoga.map_or(0, |yoga| TsTajikaYoga::from(yoga) as u8));
        self.yoga_present.push(u8::from(pair.yoga.is_some()));
        self.orb_deg.push(pair.orb_deg);
        self.apart_deg.push(pair.apart_deg);
    }

    /// The seven columns, in the order every pair-carrying section
    /// declares them.
    fn data(&self) -> [ColumnData<'_>; 7] {
        [
            ColumnData::U16(&self.faster),
            ColumnData::U16(&self.slower),
            ColumnData::U8(&self.drishti),
            ColumnData::U8(&self.yoga),
            ColumnData::U8(&self.yoga_present),
            ColumnData::F64(&self.orb_deg),
            ColumnData::F64(&self.apart_deg),
        ]
    }
}

/// Every year's matters, flat and ragged, with the yogas each holds and
/// the legs each of those stands on.
#[derive(Default)]
struct MatterColumns {
    house: Vec<u8>,
    sign: Vec<u16>,
    lagnesha: Vec<u16>,
    karyesha: Vec<u16>,
    same_lord: Vec<u8>,
    /// The lords' own relation, present unless `same_lord`.
    pair: PairColumns,
    unanswered: Vec<u16>,
    held_count: Vec<u8>,
    held: HeldColumns,
}

/// Every matter's held yogas, flat and ragged by `held_count`.
#[derive(Default)]
struct HeldColumns {
    yoga: Vec<u8>,
    by_pair: Vec<u8>,
    through: Vec<u16>,
    through_present: Vec<u8>,
    entering: Vec<u16>,
    entering_present: Vec<u8>,
    afflictions_present: Vec<u8>,
    lagnesha_afflictions: Vec<u8>,
    karyesha_afflictions: Vec<u8>,
    leg_count: Vec<u8>,
    /// The `matter_legs` section, ragged by `leg_count`.
    legs: PairColumns,
}

/// A graha as a column carries one where it may be absent: its id and a
/// presence flag, since graha id 0 is the Sun and no sentinel is free.
fn graha_or_absent(graha: Option<teistro::catalogue::Graha>) -> (u16, u8) {
    graha.map_or((0, 0), |graha| (graha.id(), 1))
}

/// Grahas as a bit set: bit `n` is the graha with catalogue id `n`, the
/// nine in sixteen bits.
fn graha_mask(grahas: &[teistro::catalogue::Graha]) -> u16 {
    grahas.iter().fold(0, |bits, graha| {
        bits | 1_u16.checked_shl(u32::from(graha.id())).unwrap_or(0)
    })
}

/// Planets as a bit set in a byte: only the seven cross here, ids 0 to 6.
fn graha_bits(grahas: &[teistro::catalogue::Graha]) -> u8 {
    u8::try_from(graha_mask(grahas) & 0x7F).unwrap_or(0)
}

impl MatterColumns {
    fn push(&mut self, matter: &teistro::YearYogas) -> Result<(), Error> {
        self.house.push(matter.house.get());
        self.sign.push(matter.sign.id());
        self.lagnesha.push(matter.lagnesha.id());
        self.karyesha.push(matter.karyesha.id());
        self.same_lord.push(u8::from(matter.same_lord));
        self.pair.push(matter.between.as_ref());
        self.unanswered
            .push(matter.unanswered.iter().fold(0, |bits, yoga| {
                bits | 1_u16 << TsYearYoga::from(*yoga) as u8
            }));
        self.held_count
            .push(u8::try_from(matter.held.len()).unwrap_or(u8::MAX));
        for held in &matter.held {
            // A yoga's `between` is the matter's own pair or nothing, which
            // is why it crosses as a flag; were that ever not so, the flag
            // would lie, and the batch is refused instead.
            if held.between.is_some() && held.between != matter.between {
                return Err(Error::new(
                    Status::Internal,
                    format!(
                        "{:?} for house {} carries a pair other than the matter's own",
                        held.yoga,
                        matter.house.get()
                    ),
                ));
            }
            let held_rows = &mut self.held;
            held_rows.yoga.push(TsYearYoga::from(held.yoga) as u8);
            held_rows.by_pair.push(u8::from(held.between.is_some()));
            let (through, present) = graha_or_absent(held.through);
            held_rows.through.push(through);
            held_rows.through_present.push(present);
            let (entering, present) = graha_or_absent(held.entering);
            held_rows.entering.push(entering);
            held_rows.entering_present.push(present);
            let [lagnesha, karyesha] = held
                .afflictions
                .map_or([0, 0], |both| both.map(TsAffliction::bits));
            held_rows
                .afflictions_present
                .push(u8::from(held.afflictions.is_some()));
            held_rows.lagnesha_afflictions.push(lagnesha);
            held_rows.karyesha_afflictions.push(karyesha);
            let legs = held.legs.as_ref().map_or(&[][..], |legs| &legs[..]);
            held_rows
                .leg_count
                .push(u8::try_from(legs.len()).unwrap_or(u8::MAX));
            for leg in legs {
                held_rows.legs.push(Some(leg));
            }
        }
        Ok(())
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        let mut columns = vec![
            ColumnData::U8(&self.house),
            ColumnData::U16(&self.sign),
            ColumnData::U16(&self.lagnesha),
            ColumnData::U16(&self.karyesha),
            ColumnData::U8(&self.same_lord),
        ];
        columns.extend(self.pair.data());
        columns.extend([
            ColumnData::U16(&self.unanswered),
            ColumnData::U8(&self.held_count),
        ]);
        writer.columns("year_matters", self.house.len(), &columns)?;
        let held = &self.held;
        writer.columns(
            "matter_yogas",
            held.yoga.len(),
            &[
                ColumnData::U8(&held.yoga),
                ColumnData::U8(&held.by_pair),
                ColumnData::U16(&held.through),
                ColumnData::U8(&held.through_present),
                ColumnData::U16(&held.entering),
                ColumnData::U8(&held.entering_present),
                ColumnData::U8(&held.afflictions_present),
                ColumnData::U8(&held.lagnesha_afflictions),
                ColumnData::U8(&held.karyesha_afflictions),
                ColumnData::U8(&held.leg_count),
            ],
        )?;
        writer.columns("matter_legs", held.legs.faster.len(), &held.legs.data())
    }
}

/// Every year's yoga-making pairs, flat and ragged.
#[derive(Default)]
struct YogaColumns {
    faster: Vec<u16>,
    slower: Vec<u16>,
    drishti: Vec<u8>,
    yoga: Vec<u8>,
    orb_deg: Vec<f64>,
    apart_deg: Vec<f64>,
}

impl YogaColumns {
    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "year_yogas",
            self.faster.len(),
            &[
                ColumnData::U16(&self.faster),
                ColumnData::U16(&self.slower),
                ColumnData::U8(&self.drishti),
                ColumnData::U8(&self.yoga),
                ColumnData::F64(&self.orb_deg),
                ColumnData::F64(&self.apart_deg),
            ],
        )
    }
}

/// Every year's claimants on the lordship, flat and ragged.
#[derive(Default)]
struct ClaimColumns {
    graha: Vec<u16>,
    vishwa: Vec<i32>,
    portfolios: Vec<u8>,
    aspects_lagna: Vec<u8>,
}

impl ClaimColumns {
    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "year_claims",
            self.graha.len(),
            &[
                ColumnData::U16(&self.graha),
                ColumnData::I32(&self.vishwa),
                ColumnData::U8(&self.portfolios),
                ColumnData::U8(&self.aspects_lagna),
            ],
        )
    }
}

impl AnnualColumns {
    fn push(&mut self, year: &teistro::AnnualChart) -> Result<(), Error> {
        let bearers = &year.bearers;
        self.lagnas.push(year.lagna_deg);
        self.daylight.push(u8::from(bearers.by_day));
        self.janma_lagna.push(bearers.janma_lagna.id());
        self.varsha_lagna.push(bearers.varsha_lagna.id());
        self.tri_rashi.push(bearers.tri_rashi.id());
        self.dina_ratri.push(bearers.dina_ratri.id());
        let lord = &year.year_lord;
        self.year_lord.push(lord.graha.id());
        self.chosen.push(TsVarsheshaChosen::from(lord.chosen) as u8);
        self.year_lord_vishwa.push(sub_sub(lord.vishwa));
        self.moon_passed_over.push(u8::from(lord.moon_passed_over));
        self.claim_counts
            .push(u8::try_from(lord.claims.len()).unwrap_or(u8::MAX));
        self.yoga_counts
            .push(u8::try_from(year.yogas.len()).unwrap_or(u8::MAX));
        for pair in &year.yogas {
            self.yogas.faster.push(pair.faster.id());
            self.yogas.slower.push(pair.slower.id());
            self.yogas
                .drishti
                .push(TsTajikaDrishti::from(pair.drishti) as u8);
            // Only pairs that make one are here, so the fallback is dead.
            self.yogas.yoga.push(
                pair.yoga
                    .map_or(u8::MAX, |yoga| TsTajikaYoga::from(yoga) as u8),
            );
            self.yogas.orb_deg.push(pair.orb_deg);
            self.yogas.apart_deg.push(pair.apart_deg);
        }
        for claim in &lord.claims {
            self.claims.graha.push(claim.graha.id());
            self.claims.vishwa.push(sub_sub(claim.vishwa));
            self.claims.portfolios.push(claim.portfolios);
            self.claims
                .aspects_lagna
                .push(u8::from(claim.aspects_lagna));
        }
        self.retrograde.push(graha_bits(&year.states.retrograde));
        self.combust.push(graha_bits(&year.states.combust));
        self.matter_counts
            .push(u8::try_from(year.matters.len()).unwrap_or(u8::MAX));
        self.saham_counts
            .push(u8::try_from(year.sahams.len()).unwrap_or(u8::MAX));
        for one in &year.sahams {
            self.sahams.push(one);
        }
        self.harsha.push(&year.harsha);
        self.dasha_counts
            .push(u8::try_from(year.dashas.len()).unwrap_or(u8::MAX));
        for dasha in &year.dashas {
            self.dashas.push(dasha);
        }
        year.matters
            .iter()
            .try_for_each(|matter| self.matters.push(matter))
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "annual_charts",
            self.lagnas.len(),
            &[
                ColumnData::F64(&self.lagnas),
                ColumnData::U8(&self.daylight),
                ColumnData::U16(&self.janma_lagna),
                ColumnData::U16(&self.varsha_lagna),
                ColumnData::U16(&self.tri_rashi),
                ColumnData::U16(&self.dina_ratri),
                ColumnData::U16(&self.year_lord),
                ColumnData::U8(&self.chosen),
                ColumnData::I32(&self.year_lord_vishwa),
                ColumnData::U8(&self.moon_passed_over),
                ColumnData::U8(&self.claim_counts),
                ColumnData::U8(&self.yoga_counts),
                ColumnData::U8(&self.retrograde),
                ColumnData::U8(&self.combust),
                ColumnData::U8(&self.matter_counts),
                ColumnData::U8(&self.saham_counts),
                ColumnData::U8(&self.dasha_counts),
            ],
        )?;
        self.claims.write(writer)?;
        self.yogas.write(writer)?;
        self.matters.write(writer)?;
        self.sahams
            .write(writer, "year_sahams", "year_saham_seven")?;
        self.harsha.write(writer)
    }
}

/// A strength as the boundary carries it: exact, in sub-sub units, of
/// which a unit holds 3600.
///
/// An integer and not a float, because the source works to sub-sub units
/// and two office-bearers a sub-sub unit apart decide a year between them.
fn sub_sub(bala: teistro::Bala) -> i32 {
    i32::try_from(bala.as_sub_sub()).unwrap_or(i32::MAX)
}

impl PraveshaColumns {
    fn of(varsha: &[teistro::Varsha]) -> Result<PraveshaColumns, Error> {
        let mut counts = Vec::with_capacity(varsha.len());
        let mut years = Vec::new();
        let mut jds = Vec::new();
        let mut muntha_signs = Vec::new();
        let mut muntha_lords = Vec::new();
        let mut muntha_degs = Vec::new();
        let mut annual = AnnualColumns::default();
        let mut natal_counts = Vec::with_capacity(varsha.len());
        let mut natal = SahamColumns::default();
        for chart in varsha {
            natal_counts.push(u32::try_from(chart.natal_sahams.len()).unwrap_or(u32::MAX));
            for one in &chart.natal_sahams {
                natal.push(one);
            }
            let found = &chart.years;
            counts.push(u32::try_from(found.len()).unwrap_or(u32::MAX));
            for one in found {
                years.push(one.pravesha.year);
                jds.push(one.pravesha.at.get());
                muntha_signs.push(one.muntha.sign.id());
                muntha_lords.push(one.muntha.lord.id());
                muntha_degs.push(one.muntha.longitude_deg);
                if let Some(year) = &one.annual {
                    annual.push(year)?;
                }
            }
        }
        Ok(PraveshaColumns {
            counts,
            years,
            jds,
            muntha_signs,
            muntha_lords,
            muntha_degs,
            annual,
            natal_counts,
            natal,
        })
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "praveshas",
            self.years.len(),
            &[
                ColumnData::U16(&self.years),
                ColumnData::F64(&self.jds),
                ColumnData::U16(&self.muntha_signs),
                ColumnData::U16(&self.muntha_lords),
                ColumnData::F64(&self.muntha_degs),
            ],
        )?;
        self.annual.write(writer)?;
        self.natal
            .write(writer, "natal_sahams", "natal_saham_seven")?;
        self.annual.dashas.write(writer)
    }
}

/// One row per chart, in the order `cast` declares its columns.
#[must_use]
fn chart_rows(
    charts: &[&ChartFoundation],
    point_counts: &[u32],
    aspect_counts: &[u32],
    pravesha_counts: &[u32],
    natal_saham_counts: &[u32],
    hit_counts: &[u32],
    sade_sati_counts: &[u32],
) -> Vec<Vec<FixedValue>> {
    charts
        .iter()
        .enumerate()
        .map(|(at, chart)| {
            vec![
                chart.instant.get().into(),
                chart.lagna_deg.into(),
                chart.day_lagna_deg.into(),
                chart.zodiac.offset_deg.into(),
                (TsDayPart::from(chart.day.part) as u64).into(),
                chart.day.elapsed.into(),
                u64::from(point_counts.get(at).copied().unwrap_or(0)).into(),
                u64::from(aspect_counts.get(at).copied().unwrap_or(0)).into(),
                u64::from(pravesha_counts.get(at).copied().unwrap_or(0)).into(),
                u64::from(natal_saham_counts.get(at).copied().unwrap_or(0)).into(),
                u64::from(hit_counts.get(at).copied().unwrap_or(0)).into(),
                u64::from(sade_sati_counts.get(at).copied().unwrap_or(0)).into(),
            ]
        })
        .collect()
}

/// Twelve bhavas per chart, charts outermost, as `houses` and `chalit`
/// both want them.
#[must_use]
fn bhava_columns(
    charts: &[&ChartFoundation],
    of: fn(&ChartFoundation) -> &teistro_chart::bhava::Bhavas,
) -> (Vec<f64>, Vec<f64>) {
    let madhya = charts.iter().flat_map(|c| of(c).madhya).collect();
    let sandhi = charts.iter().flat_map(|c| of(c).sandhi).collect();
    (madhya, sandhi)
}

/// The birth timing's values, in the order `timing` declares them.
#[must_use]
fn timing_values(timing: &teistro_chart::foundation::BirthTiming) -> Vec<FixedValue> {
    vec![
        u64::from(timing.ishtakaal.ghati).into(),
        u64::from(timing.ishtakaal.pala).into(),
        u64::from(timing.ishtakaal.vipala).into(),
        TsGhatiReckoning::of(timing.ghati_reckoning)
            .map_or(u64::from(u8::MAX), |g| g as u64)
            .into(),
        u64::from(timing.hora.number).into(),
        u64::from(timing.hora.lord.id()).into(),
        timing.hora.start.get().into(),
        timing.hora.end.get().into(),
        TsHoraReckoning::of(timing.hora_reckoning)
            .map_or(u64::from(u8::MAX), |h| h as u64)
            .into(),
    ]
}

/// What only founding can tell about a batch, gathered from its first
/// chart.
///
/// The place, the kind and the counts come from the request, so they are
/// not here; these four are decided while a chart is founded — the house
/// systems that actually built the bhavas, the frame the positions were
/// asked for, the solar model's own description, the completion steps —
/// and are the same for every chart of a batch, which is why the first
/// answers for all of them.
///
/// A batch of none has no first chart. The sections are still declared,
/// so they are written as zeroes and empty text rather than left out,
/// and `summary.chart_count` is what says they mean nothing; the
/// provenance envelope still carries the settings hash that would have
/// produced them.
/// The dashas of a batch, as the two sections carry them: one row a chart
/// a system, and every period of each, concatenated in the same order and
/// **ragged** by `period_count`, since a dasha's depth is the settings' and
/// an elapsed birth period has fewer children than a compressed one.
struct DashaColumns {
    /// How many systems each chart holds: one for the batch, since every
    /// chart answers the same request.
    count: u32,
    system: Vec<u16>,
    seeded: Vec<u8>,
    signed: Vec<u8>,
    seed: Vec<u16>,
    first_lord: Vec<u16>,
    overflow: Vec<u8>,
    balance: Vec<u8>,
    remaining: Vec<f64>,
    days: Vec<f64>,
    years: Vec<u32>,
    months: Vec<u8>,
    whole_days: Vec<u8>,
    hours: Vec<u8>,
    minutes: Vec<u8>,
    span_from: Vec<f64>,
    span_to: Vec<f64>,
    depth: Vec<u8>,
    period_count: Vec<u32>,
    /// The `dasha_periods` section.
    periods: PeriodColumns,
}

/// A dasha's periods as the boundary carries them, depth first in time
/// order: one shape for the births' `dasha_periods` and the years'
/// `year_dasha_periods`, so a period is laid out, and decoded, in one
/// place.
#[derive(Default)]
struct PeriodColumns {
    level: Vec<u8>,
    index: Vec<u8>,
    has_sign: Vec<u8>,
    sign: Vec<u16>,
    lord: Vec<u16>,
    from: Vec<f64>,
    to: Vec<f64>,
}

impl PeriodColumns {
    fn with_capacity(periods: usize) -> PeriodColumns {
        PeriodColumns {
            level: Vec::with_capacity(periods),
            index: Vec::with_capacity(periods),
            has_sign: Vec::with_capacity(periods),
            sign: Vec::with_capacity(periods),
            lord: Vec::with_capacity(periods),
            from: Vec::with_capacity(periods),
            to: Vec::with_capacity(periods),
        }
    }

    fn push(&mut self, period: &teistro::dasha::PeriodRow) {
        let places: Vec<u8> = period
            .path
            .split('/')
            .map(|step| step.parse().unwrap_or(u8::MAX))
            .collect();
        self.level
            .push(u8::try_from(places.len()).unwrap_or(u8::MAX));
        self.index.push(places.last().copied().unwrap_or(0));
        self.has_sign.push(u8::from(period.sign.is_some()));
        self.sign
            .push(period.sign.map_or(0, teistro_core::catalogue::Rashi::id));
        self.lord.push(period.lord.id());
        self.from.push(period.interval.from.get());
        self.to.push(period.interval.to.get());
    }

    fn write(
        &self,
        writer: &mut Writer<'_>,
        section: &str,
        signed_by: SignedBy,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        let mut columns = vec![ColumnData::U8(&self.level), ColumnData::U8(&self.index)];
        if let SignedBy::Period = signed_by {
            columns.push(ColumnData::U8(&self.has_sign));
        }
        columns.extend([
            ColumnData::U16(&self.sign),
            ColumnData::U16(&self.lord),
            ColumnData::F64(&self.from),
            ColumnData::F64(&self.to),
        ]);
        writer.columns(section, self.lord.len(), &columns)
    }
}

impl DashaColumns {
    fn of(
        documents: &[Document],
        registered: &teistro::dasha::DashaSystems,
    ) -> Result<DashaColumns, Error> {
        let count = documents.first().map_or(0, |d| d.dashas.len());
        if documents.iter().any(|d| d.dashas.len() != count) {
            return Err(Error::internal(
                "the batch's charts hold different numbers of dashas, though one request asked for them",
            ));
        }
        let rows = documents.len() * count;
        let periods: usize = documents
            .iter()
            .flat_map(|d| &d.dashas)
            .map(|reading| reading.periods.len())
            .sum();
        let mut columns = DashaColumns {
            count: u32::try_from(count).unwrap_or(u32::MAX),
            system: Vec::with_capacity(rows),
            seeded: Vec::with_capacity(rows),
            signed: Vec::with_capacity(rows),
            seed: Vec::with_capacity(rows),
            first_lord: Vec::with_capacity(rows),
            overflow: Vec::with_capacity(rows),
            balance: Vec::with_capacity(rows),
            remaining: Vec::with_capacity(rows),
            days: Vec::with_capacity(rows),
            years: Vec::with_capacity(rows),
            months: Vec::with_capacity(rows),
            whole_days: Vec::with_capacity(rows),
            hours: Vec::with_capacity(rows),
            minutes: Vec::with_capacity(rows),
            span_from: Vec::with_capacity(rows),
            span_to: Vec::with_capacity(rows),
            depth: Vec::with_capacity(rows),
            period_count: Vec::with_capacity(rows),
            periods: PeriodColumns::with_capacity(periods),
        };
        for reading in documents.iter().flat_map(|d| &d.dashas) {
            // A registered system crosses as the id its context gave it,
            // which is how a binding names it from the definitions it passed.
            let system = match &reading.system {
                DashaName::Catalogued(system) => system.id(),
                DashaName::Registered(key) => {
                    registered.id(key).map(KeyId::id).ok_or_else(|| {
                        Error::internal(format!("`{key}` is not registered with the context"))
                    })?
                }
            };
            columns.push(system, reading);
        }
        Ok(columns)
    }

    /// One dasha's row and its periods.
    fn push(&mut self, system: u16, reading: &teistro::DashaReading) {
        let columns = self;
        columns.system.push(system);
        columns.seeded.push(u8::from(reading.seed.is_some()));
        let signed = reading
            .periods
            .first()
            .is_some_and(|period| period.sign.is_some());
        columns.signed.push(u8::from(signed));
        columns.seed.push(
            reading
                .seed
                .map_or(0, teistro_core::catalogue::Nakshatra::id),
        );
        columns.first_lord.push(reading.first_lord.id());
        columns.overflow.push(u8::from(reading.overflow));
        let balance = reading.balance;
        columns
            .balance
            .push(balance.map_or(0, |balance| TsBalance::from(balance.method) as u8));
        columns
            .remaining
            .push(balance.map_or(0.0, |balance| balance.remaining));
        columns
            .days
            .push(balance.map_or(0.0, |balance| balance.days));
        let written = balance.map(|balance| balance.written);
        columns
            .years
            .push(written.map_or(0, |written| written.years));
        columns
            .months
            .push(written.map_or(0, |written| written.months));
        columns
            .whole_days
            .push(written.map_or(0, |written| written.days));
        columns
            .hours
            .push(written.map_or(0, |written| written.hours));
        columns
            .minutes
            .push(written.map_or(0, |written| written.minutes));
        columns
            .span_from
            .push(reading.moon_span.map_or(f64::NAN, |span| span.from.get()));
        columns
            .span_to
            .push(reading.moon_span.map_or(f64::NAN, |span| span.to.get()));
        columns.depth.push(reading.depth.get());
        columns
            .period_count
            .push(u32::try_from(reading.periods.len()).unwrap_or(u32::MAX));
        for period in &reading.periods {
            columns.periods.push(period);
        }
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "dashas",
            self.system.len(),
            &[
                ColumnData::U16(&self.system),
                ColumnData::U8(&self.seeded),
                ColumnData::U8(&self.signed),
                ColumnData::U16(&self.seed),
                ColumnData::U16(&self.first_lord),
                ColumnData::U8(&self.overflow),
                ColumnData::U8(&self.balance),
                ColumnData::F64(&self.remaining),
                ColumnData::F64(&self.days),
                ColumnData::U32(&self.years),
                ColumnData::U8(&self.months),
                ColumnData::U8(&self.whole_days),
                ColumnData::U8(&self.hours),
                ColumnData::U8(&self.minutes),
                ColumnData::F64(&self.span_from),
                ColumnData::F64(&self.span_to),
                ColumnData::U8(&self.depth),
                ColumnData::U32(&self.period_count),
            ],
        )?;
        self.periods.write(writer, "dasha_periods", SignedBy::Dasha)
    }
}

/// Every year's annual dashas (`03-design/annual-dashas.md`): a row a
/// year a system, ragged by `annual_charts.dasha_count`, with each one's
/// ring and periods ragged under it.
#[derive(Default)]
struct YearDashaColumns {
    system: Vec<u16>,
    seeded: Vec<u8>,
    seed: Vec<u16>,
    first: Vec<u8>,
    remaining: Vec<f64>,
    from: Vec<f64>,
    to: Vec<f64>,
    share_count: Vec<u8>,
    period_count: Vec<u32>,
    /// The `year_dasha_shares` section, ragged by `share_count`.
    shares: ShareColumns,
    /// The `year_dasha_periods` section, ragged by `period_count`.
    periods: PeriodColumns,
}

/// The lords a year's dasha runs round, each with its weight.
#[derive(Default)]
struct ShareColumns {
    lord: Vec<u16>,
    has_sign: Vec<u8>,
    sign: Vec<u16>,
    weight: Vec<f64>,
}

impl YearDashaColumns {
    fn push(&mut self, dasha: &teistro::AnnualDasha) {
        let ring = &dasha.ring;
        self.system.push(dasha.system.id());
        self.seeded.push(u8::from(dasha.seed.is_some()));
        self.seed
            .push(dasha.seed.map_or(0, teistro_core::catalogue::Nakshatra::id));
        self.first.push(u8::try_from(ring.first).unwrap_or(u8::MAX));
        self.remaining.push(ring.remaining.unwrap_or(f64::NAN));
        self.from.push(dasha.year.from.get());
        self.to.push(dasha.year.to.get());
        self.share_count
            .push(u8::try_from(ring.ring.len()).unwrap_or(u8::MAX));
        self.period_count
            .push(u32::try_from(dasha.periods.len()).unwrap_or(u32::MAX));
        for share in &ring.ring {
            self.shares.lord.push(share.lord.id());
            self.shares.has_sign.push(u8::from(share.sign.is_some()));
            self.shares
                .sign
                .push(share.sign.map_or(0, teistro_core::catalogue::Rashi::id));
            self.shares.weight.push(share.weight);
        }
        for period in &dasha.periods {
            self.periods.push(period);
        }
    }

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            "year_dashas",
            self.system.len(),
            &[
                ColumnData::U16(&self.system),
                ColumnData::U8(&self.seeded),
                ColumnData::U16(&self.seed),
                ColumnData::U8(&self.first),
                ColumnData::F64(&self.remaining),
                ColumnData::F64(&self.from),
                ColumnData::F64(&self.to),
                ColumnData::U8(&self.share_count),
                ColumnData::U32(&self.period_count),
            ],
        )?;
        writer.columns(
            "year_dasha_shares",
            self.shares.lord.len(),
            &[
                ColumnData::U16(&self.shares.lord),
                ColumnData::U8(&self.shares.has_sign),
                ColumnData::U16(&self.shares.sign),
                ColumnData::F64(&self.shares.weight),
            ],
        )?;
        self.periods
            .write(writer, "year_dasha_periods", SignedBy::Period)
    }
}

struct BatchOnce {
    readings: Vec<FixedValue>,
    /// The `zodiac` section's row: the frame's bits, the ayanamsha's kind
    /// and, when catalogued, its id.
    zodiac: [FixedValue; 3],
    model: String,
    steps: String,
}

impl BatchOnce {
    fn of(first: Option<&ChartFoundation>) -> BatchOnce {
        let Some(chart) = first else {
            return BatchOnce {
                readings: vec![FixedValue::Uint(0); 6],
                zodiac: [FixedValue::Uint(0); 3],
                model: String::new(),
                steps: String::from("[]"),
            };
        };
        let (ayanamsha_kind, ayanamsha) = match chart.zodiac.ayanamsha {
            None => (0_u64, 0_u64),
            Some(teistro_core::settings::AyanamshaChoice::Catalogued { id }) => {
                (1, u64::from(id.id()))
            }
            // A custom ayanamsha's coefficients are settings, and the
            // settings hash pins them: a result carries what it applied.
            Some(teistro_core::settings::AyanamshaChoice::Custom { .. }) => (2, 0),
        };
        BatchOnce {
            readings: vec![
                u64::from(chart.houses.chalit.method.id()).into(),
                u64::from(chart.houses.chalit.source.id()).into(),
                (TsReading::from(chart.houses.chalit.reading) as u64).into(),
                u64::from(chart.chalit.chalit.method.id()).into(),
                u64::from(chart.chalit.chalit.source.id()).into(),
                (TsReading::from(chart.chalit.chalit.reading) as u64).into(),
            ],
            zodiac: [
                u64::from(chart.zodiac.request.to_bits()).into(),
                ayanamsha_kind.into(),
                ayanamsha.into(),
            ],
            model: chart.day.day.model.clone(),
            steps: serde_json::to_string(&chart.steps).unwrap_or_else(|_| String::from("[]")),
        }
    }
}

/// A batch of founded charts as the blob its schema describes.
///
/// Every per-chart section runs charts outermost, so a batch of one is
/// the same blob a one-chart entry point would have written, with the
/// The blob's three JSON sections, each written once per chart and each
/// empty where the request did not ask for it.
///
/// They travel together because they are the same kind of thing — what a
/// chart was asked to say beyond its numbers — and because a fourth of them
/// is likelier than a fourth positional argument is welcome.
#[derive(Clone, Copy, Debug, Default)]
pub struct Composed<'a> {
    /// Every chart's drawings as SVG strings (`render-svg.md`).
    pub svgs: &'a str,
    /// What every chart answered by rule (`rules-at-the-boundary.md`).
    pub rules: &'a str,
    /// What every chart has to say (`plans-at-the-boundary.md`).
    pub plans: &'a str,
    /// Every chart's annual charts and its own sahams, in the batch's
    /// order (`annual-chart.md`); empty when none were asked for.
    pub praveshas: &'a [teistro::Varsha],
    /// Every chart's transits at the instants `gochar` asked for, in the
    /// batch's order, and those instants (`gochar.md`); both empty when
    /// none were asked for.
    pub gochar: &'a [Vec<teistro::gochar::GocharReading>],
    /// The instants every chart's transits were read at.
    pub gochar_instants: &'a [JulianDay<Utc>],
    /// Every chart's transit hit list, in the batch's order
    /// (`transit-hit-list.md`); empty when none was asked for.
    pub hits: &'a [Vec<teistro::Hit>],
    /// Every chart's Sade Sati report, in the batch's order
    /// (`sade-sati.md`); empty when none was asked for.
    pub sade_sati: &'a [teistro::sade_sati::Report],
    /// Every chart's KP reading as canonical JSON (`kp.md`); empty when
    /// none was asked for.
    pub kp: &'a str,
    /// Every chart's essential dignities, in the batch's order
    /// (`essential-dignities.md`); empty when none was asked for.
    pub dignities: &'a [teistro::Dignities],
    /// Every chart's accidental fortitudes with its essential dignities,
    /// in the batch's order; empty when none was asked for, and when
    /// given, `dignities` is empty and the dignity sections are filled
    /// from these.
    pub fortitudes: &'a [teistro::Fortitudes],
    /// Every chart's lots, all fourteen, in the batch's order
    /// (`hellenistic-lots.md`); empty when none was asked for.
    pub lots: &'a [teistro::LotReading],
    /// Every chart's considerations before judgement, in the batch's order
    /// (`hellenistic-considerations.md`); empty when none was asked for.
    pub considerations: &'a [teistro::Considerations],
    /// Every chart's perfection between its two significators, with the
    /// rules it was read under, in the batch's order
    /// (`hellenistic-perfection.md`); empty when none was asked for.
    pub perfections: &'a [(teistro::Matter, teistro::PerfectionRules)],
    /// Every chart's progressions, in the batch's order
    /// (`western-progressions.md`); empty when none were asked for.
    pub progressions: &'a [teistro::Progressions],
    /// Every chart's Western aspect table, in the batch's order
    /// (`western-aspects.md`); empty when none was asked for.
    pub western_aspects: &'a [Vec<teistro::WesternAspectRow>],
    /// Every chart's synastry with the record's partner, in the batch's
    /// order (`western-synastry.md`); empty when none was asked for.
    pub synastry: &'a [teistro::PartnerReading],
    /// Every chart's declinations, in the batch's order
    /// (`western-declinations.md`); empty when no parallels were asked
    /// for.
    pub declinations: &'a [teistro::Declinations],
    /// Every chart's parallels, in the batch's order; empty when none were
    /// asked for.
    pub parallels: &'a [Vec<teistro::ParallelRow>],
    /// Every chart's antiscia, in the batch's order
    /// (`western-antiscia.md`); empty when none were asked for.
    pub antiscia: &'a [teistro::Antiscia],
    /// Every chart's equal distances, in the batch's order
    /// (`western-midpoints.md`); empty when none were asked for.
    pub midpoints: &'a [Vec<teistro::MidpointRow>],
    /// Every chart's Davison birth with the synastry's partner, in the
    /// batch's order (`western-composites.md`); empty when none was asked
    /// for.
    pub davisons: &'a [teistro::Partner],
    /// Every chart's own content hash, in the batch's order: what a chart
    /// handed out alone is stamped with, where the provenance hashes the
    /// list.
    pub hashes: &'a [teistro::Hash],
}

/// counts saying so.
///
/// The sections that describe the batch rather than a chart — the place,
/// the kind, the frame, the house systems, the solar model, the
/// completion steps — are written from the request where the request
/// knows them and from the first chart where only founding can tell.
/// A batch of none therefore carries zeroes in the latter, and its
/// provenance envelope still carries the settings hash that would have
/// produced them.
///
/// # Errors
///
/// Whatever the writer refuses: a section the schema does not have, or a
/// column of the wrong length. Neither can happen for a value this crate
/// built, so a failure here is a schema that has drifted from this
/// function rather than a caller's mistake. Charts of differing graha
/// counts are `INTERNAL`, since the blob's layout is one count for the
/// batch.
pub fn encode(
    documents: &[Document],
    place: &Place,
    kind: ChartKind,
    provenance: &Provenance,
    composed: Composed<'_>,
    registered: &teistro::dasha::DashaSystems,
) -> Result<Vec<u8>, Error> {
    let Composed {
        svgs,
        rules,
        plans,
        praveshas,
        gochar,
        gochar_instants,
        hits,
        sade_sati,
        kp,
        hashes,
        ..
    } = composed;
    let hashes = crate::support::hashes_text(hashes, documents.len())?;
    let charts: Vec<&ChartFoundation> = documents.iter().map(|d| &d.foundation).collect();
    let charts = charts.as_slice();
    let schema = crate::schemas::charts();
    let mut writer = Writer::new(&schema);
    let chart_count = u32::try_from(charts.len()).unwrap_or(u32::MAX);
    let graha_count = one_size(charts)?;
    let day_rows: Vec<Vec<FixedValue>> = charts.iter().map(|c| day_values(&c.day.day)).collect();
    let timing_rows: Vec<Vec<FixedValue>> =
        charts.iter().map(|c| timing_values(&c.timing)).collect();
    let once = BatchOnce::of(charts.first().copied());
    let by = Sections::of(documents, graha_count, registered, praveshas)?;
    let transits = GocharColumns::of(gochar, gochar_instants)?;
    let searches = Searches::of(hits, sade_sati, charts.len())?;
    let hellenistic = HellenisticColumns::of(&composed, charts.len())?;
    let progressions = ProgressionColumns::of(composed.progressions, charts.len())?;
    let tables = AspectTables::of(&composed, charts.len())?;
    let summary = summary_values(
        place,
        kind,
        chart_count,
        u32::try_from(graha_count).unwrap_or(u32::MAX),
        by.vargas.count,
        by.dashas.count,
    );

    let write = || -> Result<Vec<u8>, teistro_idl::blob::BlobError> {
        writer.fixed("summary", &summary)?;
        writer.rows(
            "cast",
            &chart_rows(
                charts,
                &by.points.counts,
                &by.aspects.counts,
                &by.years.counts,
                &by.years.natal_counts,
                &searches.hits.counts,
                &searches.sade_sati.counts,
            ),
        )?;
        GrahaColumns::of(charts, |c| &c.grahas).write(&mut writer, "grahas")?;
        writer.fixed("readings", &once.readings)?;
        write_bhavas(&mut writer, "houses", charts, |c| &c.houses)?;
        write_bhavas(&mut writer, "chalit", charts, |c| &c.chalit)?;
        writer.fixed("zodiac", &once.zodiac)?;
        writer.rows("day", &day_rows)?;
        writer.rows("timing", &timing_rows)?;
        writer.bytes("model", once.model.as_bytes())?;
        writer.bytes("steps", once.steps.as_bytes())?;
        writer.bytes(
            "provenance_json",
            teistro_core::envelope::canonical_json(provenance).as_bytes(),
        )?;
        by.vargas.write(&mut writer)?;
        by.aspects.write(&mut writer)?;
        by.points.write(&mut writer)?;
        by.bhavas.write(&mut writer)?;
        by.states.write(&mut writer)?;
        writer.bytes("drawings", drawings_json(documents).as_bytes())?;
        writer.bytes("svgs", svgs.as_bytes())?;
        writer.bytes("rules", rules.as_bytes())?;
        writer.bytes("plans", plans.as_bytes())?;
        by.years.write(&mut writer)?;
        by.dashas.write(&mut writer)?;
        by.ashtakavarga.write(&mut writer)?;
        by.vimshopaka.write(&mut writer)?;
        by.shadbala.write(&mut writer)?;
        by.bhava_bala.write(&mut writer)?;
        by.vaiseshikamsa.write(&mut writer)?;
        by.dasha_phala.write(&mut writer)?;
        writer.bytes("content_hashes", hashes.as_bytes())?;
        by.jaimini.write(&mut writer)?;
        transits.write(&mut writer)?;
        searches.write(&mut writer)?;
        writer.bytes("kp", kp.as_bytes())?;
        hellenistic.write(&mut writer)?;
        progressions.write(&mut writer)?;
        GrahaColumns::of(charts, |c| &c.outer).write(&mut writer, "outer")?;
        tables.write(&mut writer)?;
        writer.finish()
    };
    write().map_err(|error| {
        Error::new(
            Status::Internal,
            format!("the chart blob could not be written: {error}"),
        )
    })
}

/// One of the two twelve-bhava sections, charts outermost.
fn write_bhavas(
    writer: &mut Writer<'_>,
    name: &str,
    charts: &[&ChartFoundation],
    pick: fn(&ChartFoundation) -> &teistro_chart::bhava::Bhavas,
) -> Result<(), teistro_idl::blob::BlobError> {
    let (madhya, sandhi) = bhava_columns(charts, pick);
    writer.columns(
        name,
        charts.len() * 12,
        &[ColumnData::F64(&madhya), ColumnData::F64(&sandhi)],
    )
}

/// Every chart's drawings as the canonical JSON the `drawings` section
/// carries: one array per chart, or nothing at all when none were asked for,
/// so a caller that drew nothing pays for no text.
fn drawings_json(documents: &[Document]) -> String {
    if documents
        .iter()
        .all(|document| document.drawings.is_empty())
    {
        return String::new();
    }
    let per_chart: Vec<&Vec<teistro_geometry::Drawing>> = documents
        .iter()
        .map(|document| &document.drawings)
        .collect();
    teistro_core::envelope::canonical_json(&per_chart)
}

/// Every chart's drawings written as SVG in one theme, as the canonical JSON
/// the `svgs` section carries: one array of strings per chart, in the order
/// the drawings were asked for.
fn svgs_json(
    sdk: &teistro::Context,
    documents: &[Document],
    theme: &Theme,
) -> Result<String, Error> {
    let mut per_chart = Vec::with_capacity(documents.len());
    for document in documents {
        let mut svgs = Vec::with_capacity(document.drawings.len());
        for index in 0..document.drawings.len() {
            svgs.push(sdk.chart().svg(document, index, theme)?);
        }
        per_chart.push(svgs);
    }
    Ok(teistro_core::envelope::canonical_json(&per_chart))
}

/// A chart layout this context can draw in, shipped or registered, as its
/// JSON row: the record `options.layouts_json` takes. Read a shipped row,
/// give it a key of its own, change what differs and register it
/// (`03-design/chart-geometry.md` §7f). `key` is the layout's key, bare
/// (`NORTH_INDIAN`) or full (`chart_layout.NORTH_INDIAN`); an unknown one is
/// `INVALID_ARG` with the keys the context knows as the hint.
///
/// # Safety
///
/// `context` must be a live handle; `key` a NUL-terminated string;
/// `out_json` valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_chart_layout_row(
    context: *const TsContext,
    key: *const c_char,
    out_json: *mut TsString,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the entry point's contract.
        let asked = unsafe { crate::support::text(key, "key") }?;
        let row = ctx.sdk().chart().layout(asked)?;
        let json = TsString::from_string(teistro_core::envelope::canonical_json(&row));
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_json, "out_json", json) }
    })
}

/// The name a refusal gives a chart request's plans: the record every
/// binding and the façade call `interpret`, not the C argument
/// (`interpret_json`) that carries it, so a caller reads back the field
/// they wrote (`interpret.readings`). Only a C caller can hand the
/// argument itself something unreadable, and that refusal names it.
const INTERPRET: &str = "interpret";

/// The name a refusal gives a chart request's rule set, as [`INTERPRET`]
/// (`rules.rules[0]`, not `rules_json.rules[0]`).
const RULES: &str = "rules";

/// The plans a request's `interpret_json` asks for, none of them for null; a
/// refusal is named from the request's root, `interpret.readings`.
///
/// # Safety
///
/// `interpret_json` null or a NUL-terminated string.
unsafe fn plan_request_of(interpret_json: *const c_char) -> Result<PlanRequest, Error> {
    // SAFETY: the caller's contract.
    let text = unsafe { optional_text(interpret_json, "interpret_json") }?;
    let Some(text) = text else {
        return Ok(PlanRequest::default());
    };
    PlanRequest::from_json(text).map_err(|error| error.under(INTERPRET))
}

/// Which step of the year lord's chain decided it
/// (`03-design/varshesha.md`).
///
/// Mirrors `teistro::Chosen` through an **exhaustive** match, which is
/// what stops the two drifting: a step added stops this crate compiling
/// rather than silently crossing as whatever was first.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsVarsheshaChosen {
    /// The strongest office-bearer that aspects the annual lagna: the
    /// ordinary answer.
    Strongest = 0,
    /// Tied on strength, and this one holds more portfolios.
    MostPortfolios = 1,
    /// The Muntha's lord, because no office-bearer aspects the lagna.
    MunthaLordUnaspected = 2,
    /// The Muntha's lord, because every office-bearer is under five units.
    MunthaLordAllWeak = 3,
    /// The Muntha's lord, on an outright tie of strength, aspect and
    /// portfolios.
    MunthaLordTied = 4,
    /// The Dina-Ratri Pati, on that same tie, under the other reading.
    DinaRatriTied = 5,
    /// The annual lagna's lord, because nobody aspects and the rules ask
    /// for that reading.
    AnnualLagnaLordUnaspected = 6,
    /// The strongest of the five, because nobody aspects and the rules ask
    /// for the *Nilakanthi*'s reading.
    StrongestUnaspected = 7,
    /// The planet in Ithasala with the Moon, the strongest of several, in
    /// the Moon's place.
    MoonsIthasala = 8,
    /// The lord of the Moon's sign, in the Moon's place: the Moon itself
    /// where it stands in Cancer.
    MoonsSignLord = 9,
}

/// The Tajika aspect between two signs (`03-design/tajika-aspects.md`).
///
/// Not the Parashari drishti, which crosses elsewhere: this one is a
/// relation between signs, and its neutral houses give no aspect at all.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsTajikaDrishti {
    /// Pratyaksha Mitra, at houses 5 and 9: openly friendly.
    Friendly = 0,
    /// Gupta Mitra, at houses 3 and 11: secretly friendly.
    SecretlyFriendly = 1,
    /// Pratyaksha Shatru, at houses 1 and 7: openly inimical, and an
    /// aspect.
    Inimical = 2,
    /// Gupta Shatru, at houses 4 and 10: secretly inimical.
    SecretlyInimical = 3,
    /// Sama, at houses 2, 6, 8 and 12: no aspect at all.
    None = 4,
}

impl From<teistro::TajikaDrishti> for TsTajikaDrishti {
    fn from(drishti: teistro::TajikaDrishti) -> TsTajikaDrishti {
        match drishti {
            teistro::TajikaDrishti::Friendly => TsTajikaDrishti::Friendly,
            teistro::TajikaDrishti::SecretlyFriendly => TsTajikaDrishti::SecretlyFriendly,
            teistro::TajikaDrishti::Inimical => TsTajikaDrishti::Inimical,
            teistro::TajikaDrishti::SecretlyInimical => TsTajikaDrishti::SecretlyInimical,
            teistro::TajikaDrishti::None => TsTajikaDrishti::None,
        }
    }
}

/// What two planets inside each other's orb are doing.
///
/// Three of the four are kinds of Ithasala, the coming-together, which is
/// how the source's Table X-3 enumerates them.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsTajikaYoga {
    /// Vartamana Ithasala: the faster is behind the slower by a degree or
    /// more, inside the orb, and coming to it.
    IthasalaVartamana = 0,
    /// Poorna Ithasala: as Vartamana but within a single degree, which
    /// the source marks as immediate fulfilment.
    IthasalaPoorna = 1,
    /// Bhavishyat Ithasala: the faster is past but stands at 29° or
    /// beyond, so it acts from the next sign, where it is behind again.
    IthasalaBhavishyat = 2,
    /// Ishrafa: the faster is a degree or more past the slower and
    /// drawing away.
    Ishrafa = 3,
}

impl From<teistro::TajikaYoga> for TsTajikaYoga {
    fn from(yoga: teistro::TajikaYoga) -> TsTajikaYoga {
        match yoga {
            teistro::TajikaYoga::IthasalaVartamana => TsTajikaYoga::IthasalaVartamana,
            teistro::TajikaYoga::IthasalaPoorna => TsTajikaYoga::IthasalaPoorna,
            teistro::TajikaYoga::IthasalaBhavishyat => TsTajikaYoga::IthasalaBhavishyat,
            teistro::TajikaYoga::Ishrafa => TsTajikaYoga::Ishrafa,
        }
    }
}

/// One of the sixteen Tajika yogas of the annual chart (K.S. Charak,
/// Table X-3; `03-design/tajika-yogas.md`), in the table's order.
///
/// Mirrors `teistro::YearYoga` through an **exhaustive** match, so a yoga
/// added there stops this crate compiling rather than crossing as another.
/// Its ids are also the bit positions of `year_matters.unanswered`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsYearYoga {
    /// Every planet in a kendra or a panaphara: a fact about the chart.
    Ikabala = 0,
    /// Every planet in an apoklima: a fact about the chart.
    Induvara = 1,
    /// The lagnesha and the karyesha are coming together, in one of the
    /// three kinds `TsTajikaYoga` enumerates.
    Ithasala = 2,
    /// The pair are drawing apart.
    Ishrafa = 3,
    /// The two do not aspect, and a planet faster than both carries the
    /// light between them: past one, coming to the other.
    Nakta = 4,
    /// The two do not aspect, and a planet slower than both gathers their
    /// light: both are coming to it.
    Yamaya = 5,
    /// An Ithasala a malefic destroys.
    Manau = 6,
    /// An Ithasala the Moon joins.
    Kamboola = 7,
    /// An Ithasala an unqualified Moon completes on entering the next sign.
    GairiKamboola = 8,
    /// An Ithasala an unqualified Moon negates by standing apart from it.
    Khallasara = 9,
    /// An Ithasala where either of the pair is afflicted.
    Rudda = 10,
    /// An Ithasala where the slower is strong and the faster weak.
    DuhphaliKuttha = 11,
    /// Both weak, and one in Ithasala with a third, strong planet.
    DutthotthaDavira = 12,
    /// No aspect and no Ithasala, the karyesha completing one from the
    /// next sign.
    Tambira = 13,
    /// Both powerful, in a kendra or a panaphara, under a benefic's aspect
    /// and no malefic's (crux C117).
    Kuttha = 14,
    /// Both weak, in the trika houses, combust or retrograde.
    Durapha = 15,
}

impl From<teistro::YearYoga> for TsYearYoga {
    fn from(yoga: teistro::YearYoga) -> TsYearYoga {
        match yoga {
            teistro::YearYoga::Ikabala => TsYearYoga::Ikabala,
            teistro::YearYoga::Induvara => TsYearYoga::Induvara,
            teistro::YearYoga::Ithasala => TsYearYoga::Ithasala,
            teistro::YearYoga::Ishrafa => TsYearYoga::Ishrafa,
            teistro::YearYoga::Nakta => TsYearYoga::Nakta,
            teistro::YearYoga::Yamaya => TsYearYoga::Yamaya,
            teistro::YearYoga::Manau => TsYearYoga::Manau,
            teistro::YearYoga::Kamboola => TsYearYoga::Kamboola,
            teistro::YearYoga::GairiKamboola => TsYearYoga::GairiKamboola,
            teistro::YearYoga::Khallasara => TsYearYoga::Khallasara,
            teistro::YearYoga::Rudda => TsYearYoga::Rudda,
            teistro::YearYoga::DuhphaliKuttha => TsYearYoga::DuhphaliKuttha,
            teistro::YearYoga::DutthotthaDavira => TsYearYoga::DutthotthaDavira,
            teistro::YearYoga::Tambira => TsYearYoga::Tambira,
            teistro::YearYoga::Kuttha => TsYearYoga::Kuttha,
            teistro::YearYoga::Durapha => TsYearYoga::Durapha,
        }
    }
}

/// One of the forty-one Tajika sahams (K.S. Charak, ch. XI;
/// `03-design/tajika-sahams.md`), in the source's order, each id its
/// number less one.
///
/// Mirrors `teistro::Saham` through an **exhaustive** match, so a saham
/// added there stops this crate compiling rather than crossing as another.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsSaham {
    /// **Punya**, general auspiciousness.
    Punya = 0,
    /// **Guru**, the preceptor.
    Guru = 1,
    /// **Vidya** (Jnana), knowledge.
    Vidya = 2,
    /// **Yasha**, fame.
    Yasha = 3,
    /// **Mitra**, friends.
    Mitra = 4,
    /// **Mahatmya**, the fruits of virtuous living.
    Mahatmya = 5,
    /// **Asha**, hope.
    Asha = 6,
    /// **Samarthya**, capability.
    Samarthya = 7,
    /// **Bhratri**, siblings.
    Bhratri = 8,
    /// **Gaurava**, dignity.
    Gaurava = 9,
    /// **Pitri** (Taata), the father.
    Pitri = 10,
    /// **Raja**, royal dignity.
    Raja = 11,
    /// **Matri**, the mother.
    Matri = 12,
    /// **Putra**, progeny.
    Putra = 13,
    /// **Jeeva**, life.
    Jeeva = 14,
    /// **Roga**, disease.
    Roga = 15,
    /// **Karma**, profession.
    Karma = 16,
    /// **Manmatha**, infatuation.
    Manmatha = 17,
    /// **Kali**, strife.
    Kali = 18,
    /// **Kshama**, forgiveness.
    Kshama = 19,
    /// **Shastra**, scriptures.
    Shastra = 20,
    /// **Bandhu**, relatives.
    Bandhu = 21,
    /// **Mrityu**, death.
    Mrityu = 22,
    /// **Deshantara**, foreign travel.
    Deshantara = 23,
    /// **Artha** (Dhana), wealth.
    Artha = 24,
    /// **Paradara**, adultery.
    Paradara = 25,
    /// **Anya-karma**, an additional vocation.
    AnyaKarma = 26,
    /// **Vanika**, trade.
    Vanika = 27,
    /// **Karya-siddhi**, success in a venture.
    KaryaSiddhi = 28,
    /// **Vivaha**, marriage.
    Vivaha = 29,
    /// **Prasava**, the delivery of a child.
    Prasava = 30,
    /// **Santaapa**, sorrow.
    Santaapa = 31,
    /// **Shraddha**, devotion.
    Shraddha = 32,
    /// **Preeti**, love.
    Preeti = 33,
    /// **Jadya**, stupidity.
    Jadya = 34,
    /// **Vyapara**, business.
    Vyapara = 35,
    /// **Paneeya-paata**, falling into water.
    PaneeyaPaata = 36,
    /// **Shatru**, enemies.
    Shatru = 37,
    /// **Jalapatha**, a sea voyage.
    Jalapatha = 38,
    /// **Bandhana**, imprisonment.
    Bandhana = 39,
    /// **Labha**, monetary gain.
    Labha = 40,
}

impl From<teistro::Saham> for TsSaham {
    fn from(saham: teistro::Saham) -> TsSaham {
        match saham {
            teistro::Saham::Punya => TsSaham::Punya,
            teistro::Saham::Guru => TsSaham::Guru,
            teistro::Saham::Vidya => TsSaham::Vidya,
            teistro::Saham::Yasha => TsSaham::Yasha,
            teistro::Saham::Mitra => TsSaham::Mitra,
            teistro::Saham::Mahatmya => TsSaham::Mahatmya,
            teistro::Saham::Asha => TsSaham::Asha,
            teistro::Saham::Samarthya => TsSaham::Samarthya,
            teistro::Saham::Bhratri => TsSaham::Bhratri,
            teistro::Saham::Gaurava => TsSaham::Gaurava,
            teistro::Saham::Pitri => TsSaham::Pitri,
            teistro::Saham::Raja => TsSaham::Raja,
            teistro::Saham::Matri => TsSaham::Matri,
            teistro::Saham::Putra => TsSaham::Putra,
            teistro::Saham::Jeeva => TsSaham::Jeeva,
            teistro::Saham::Roga => TsSaham::Roga,
            teistro::Saham::Karma => TsSaham::Karma,
            teistro::Saham::Manmatha => TsSaham::Manmatha,
            teistro::Saham::Kali => TsSaham::Kali,
            teistro::Saham::Kshama => TsSaham::Kshama,
            teistro::Saham::Shastra => TsSaham::Shastra,
            teistro::Saham::Bandhu => TsSaham::Bandhu,
            teistro::Saham::Mrityu => TsSaham::Mrityu,
            teistro::Saham::Deshantara => TsSaham::Deshantara,
            teistro::Saham::Artha => TsSaham::Artha,
            teistro::Saham::Paradara => TsSaham::Paradara,
            teistro::Saham::AnyaKarma => TsSaham::AnyaKarma,
            teistro::Saham::Vanika => TsSaham::Vanika,
            teistro::Saham::KaryaSiddhi => TsSaham::KaryaSiddhi,
            teistro::Saham::Vivaha => TsSaham::Vivaha,
            teistro::Saham::Prasava => TsSaham::Prasava,
            teistro::Saham::Santaapa => TsSaham::Santaapa,
            teistro::Saham::Shraddha => TsSaham::Shraddha,
            teistro::Saham::Preeti => TsSaham::Preeti,
            teistro::Saham::Jadya => TsSaham::Jadya,
            teistro::Saham::Vyapara => TsSaham::Vyapara,
            teistro::Saham::PaneeyaPaata => TsSaham::PaneeyaPaata,
            teistro::Saham::Shatru => TsSaham::Shatru,
            teistro::Saham::Jalapatha => TsSaham::Jalapatha,
            teistro::Saham::Bandhana => TsSaham::Bandhana,
            teistro::Saham::Labha => TsSaham::Labha,
        }
    }
}

/// How a planet stands to another by Tajika's friendship: the relation a
/// saham's company is read by (`03-design/tajika-saham-strength.md`).
///
/// Mirrors `teistro::TajikaRelation` through an **exhaustive** match.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsTajikaRelation {
    /// The planet is the other: its own.
    Own = 0,
    /// A friend.
    Friend = 1,
    /// Neither.
    Neutral = 2,
    /// An enemy.
    Enemy = 3,
}

impl From<teistro::TajikaRelation> for TsTajikaRelation {
    fn from(relation: teistro::TajikaRelation) -> TsTajikaRelation {
        match relation {
            teistro::TajikaRelation::Own => TsTajikaRelation::Own,
            teistro::TajikaRelation::Friend => TsTajikaRelation::Friend,
            teistro::TajikaRelation::Neutral => TsTajikaRelation::Neutral,
            teistro::TajikaRelation::Enemy => TsTajikaRelation::Enemy,
        }
    }
}

/// What the source calls a planet by its Harsha bala
/// (`03-design/tajika-harsha.md`).
///
/// Mirrors `teistro::HarshaGrade` through an **exhaustive** match.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsHarshaGrade {
    /// No part: without strength.
    Nirbala = 0,
    /// One part, five units: weak.
    Alpabali = 1,
    /// Two parts, ten units: of medium strength.
    MadhyaBali = 2,
    /// Three parts, fifteen units: fully strong.
    PoornaBali = 3,
    /// All four, twenty units: extraordinarily strong, and rare.
    Extraordinary = 4,
}

impl From<teistro::HarshaGrade> for TsHarshaGrade {
    fn from(grade: teistro::HarshaGrade) -> TsHarshaGrade {
        match grade {
            teistro::HarshaGrade::Nirbala => TsHarshaGrade::Nirbala,
            teistro::HarshaGrade::Alpabali => TsHarshaGrade::Alpabali,
            teistro::HarshaGrade::MadhyaBali => TsHarshaGrade::MadhyaBali,
            teistro::HarshaGrade::PoornaBali => TsHarshaGrade::PoornaBali,
            teistro::HarshaGrade::Extraordinary => TsHarshaGrade::Extraordinary,
        }
    }
}

/// A clause of the source's list of what makes a saham **strong**, in its
/// order (`03-design/tajika-saham-strength.md`). Its ids are the bit
/// positions of a saham row's `strong` column.
///
/// Mirrors `teistro::StrongClause` through an **exhaustive** match.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsSahamStrong {
    /// Its lord is exalted.
    LordExalted = 0,
    /// Its lord is in its own sign.
    LordOwnSign = 1,
    /// Its lord is in its own Hudda.
    LordOwnHudda = 2,
    /// Its lord is in its own Drekkana.
    LordOwnDrekkana = 3,
    /// Its lord is in its own Navamsha.
    LordOwnNavamsha = 4,
    /// Its lord is in a friend's sign.
    LordInFriendsSign = 5,
    /// It is with a friend of its lord.
    WithFriend = 6,
    /// It is with a natural benefic.
    WithBenefic = 7,
    /// It is with the year lord.
    WithYearLord = 8,
    /// Its lord conjoins it.
    LordConjoins = 9,
    /// Its lord aspects it.
    LordAspectsSaham = 10,
    /// Its lord aspects the lagna.
    LordAspectsLagna = 11,
}

impl From<teistro::StrongClause> for TsSahamStrong {
    fn from(clause: teistro::StrongClause) -> TsSahamStrong {
        match clause {
            teistro::StrongClause::LordExalted => TsSahamStrong::LordExalted,
            teistro::StrongClause::LordOwnSign => TsSahamStrong::LordOwnSign,
            teistro::StrongClause::LordOwnHudda => TsSahamStrong::LordOwnHudda,
            teistro::StrongClause::LordOwnDrekkana => TsSahamStrong::LordOwnDrekkana,
            teistro::StrongClause::LordOwnNavamsha => TsSahamStrong::LordOwnNavamsha,
            teistro::StrongClause::LordInFriendsSign => TsSahamStrong::LordInFriendsSign,
            teistro::StrongClause::WithFriend => TsSahamStrong::WithFriend,
            teistro::StrongClause::WithBenefic => TsSahamStrong::WithBenefic,
            teistro::StrongClause::WithYearLord => TsSahamStrong::WithYearLord,
            teistro::StrongClause::LordConjoins => TsSahamStrong::LordConjoins,
            teistro::StrongClause::LordAspectsSaham => TsSahamStrong::LordAspectsSaham,
            teistro::StrongClause::LordAspectsLagna => TsSahamStrong::LordAspectsLagna,
        }
    }
}

/// A clause of the source's list of what makes a saham **weak**, in its
/// order. Its ids are the bit positions of a saham row's `weak` column.
///
/// Mirrors `teistro::WeakClause` through an **exhaustive** match.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsSahamWeak {
    /// Its lord is under the Panchavargiya floor.
    LordWeakVishwa = 0,
    /// Its lord has no Harsha bala.
    LordLacksHarsha = 1,
    /// Its lord neither aspects nor conjoins it.
    LordApart = 2,
    /// It is with an enemy of its lord.
    WithEnemy = 3,
    /// It is with a natural malefic.
    WithMalefic = 4,
}

impl From<teistro::WeakClause> for TsSahamWeak {
    fn from(clause: teistro::WeakClause) -> TsSahamWeak {
        match clause {
            teistro::WeakClause::LordWeakVishwa => TsSahamWeak::LordWeakVishwa,
            teistro::WeakClause::LordLacksHarsha => TsSahamWeak::LordLacksHarsha,
            teistro::WeakClause::LordApart => TsSahamWeak::LordApart,
            teistro::WeakClause::WithEnemy => TsSahamWeak::WithEnemy,
            teistro::WeakClause::WithMalefic => TsSahamWeak::WithMalefic,
        }
    }
}

/// One of the five clauses of the source's **affliction**, which Rudda
/// and Durapha read. Its ids are the bit positions of
/// `matter_yogas.lagnesha_afflictions` and `karyesha_afflictions`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsAffliction {
    /// Going backwards through the zodiac.
    Retrograde = 0,
    /// Burnt by the Sun.
    Combust = 1,
    /// In its sign of debilitation.
    Debilitated = 2,
    /// In the 6th, 8th or 12th house from the annual lagna.
    Trika = 3,
    /// Conjunct or inimically aspected by one of Tajika's malefics.
    UnderMalefic = 4,
}

impl TsAffliction {
    /// An affliction's clauses as a bit set: bit `n` is the clause with
    /// id `n`. The struct is destructured whole, so a clause added to
    /// `teistro::Affliction` stops this compiling rather than crossing
    /// unset.
    fn bits(affliction: teistro::Affliction) -> u8 {
        let teistro::Affliction {
            graha: _,
            retrograde,
            combust,
            debilitated,
            trika,
            under_malefic,
        } = affliction;
        [
            (TsAffliction::Retrograde, retrograde),
            (TsAffliction::Combust, combust),
            (TsAffliction::Debilitated, debilitated),
            (TsAffliction::Trika, trika),
            (TsAffliction::UnderMalefic, under_malefic),
        ]
        .into_iter()
        .filter(|(_, holds)| *holds)
        .fold(0, |bits, (clause, _)| bits | 1 << clause as u8)
    }
}

impl From<teistro::Chosen> for TsVarsheshaChosen {
    fn from(chosen: teistro::Chosen) -> TsVarsheshaChosen {
        match chosen {
            teistro::Chosen::Strongest => TsVarsheshaChosen::Strongest,
            teistro::Chosen::MostPortfolios => TsVarsheshaChosen::MostPortfolios,
            teistro::Chosen::MunthaLordUnaspected => TsVarsheshaChosen::MunthaLordUnaspected,
            teistro::Chosen::MunthaLordAllWeak => TsVarsheshaChosen::MunthaLordAllWeak,
            teistro::Chosen::MunthaLordTied => TsVarsheshaChosen::MunthaLordTied,
            teistro::Chosen::DinaRatriTied => TsVarsheshaChosen::DinaRatriTied,
            teistro::Chosen::AnnualLagnaLordUnaspected => {
                TsVarsheshaChosen::AnnualLagnaLordUnaspected
            }
            teistro::Chosen::StrongestUnaspected => TsVarsheshaChosen::StrongestUnaspected,
            teistro::Chosen::MoonsIthasala => TsVarsheshaChosen::MoonsIthasala,
            teistro::Chosen::MoonsSignLord => TsVarsheshaChosen::MoonsSignLord,
        }
    }
}

/// The annual charts a request's `varsha_json` asks for, none for null; the
/// façade reads and checks the record ([`teistro::VarshaRequest::from_json`]),
/// naming a refusal from its root, `varsha.through`.
///
/// # Safety
///
/// `varsha_json` null or a NUL-terminated string.
unsafe fn varsha_request_of(
    varsha_json: *const c_char,
) -> Result<Option<teistro::VarshaRequest>, Error> {
    // SAFETY: the caller's contract.
    unsafe { optional_text(varsha_json, "varsha_json") }?
        .map(teistro::VarshaRequest::from_json)
        .transpose()
}

/// The transits a request's `gochar_json` asks for, none for null; the
/// façade reads and checks the record ([`teistro::GocharRequest::from_json`]),
/// naming a refusal from its root, `gochar.instants`.
///
/// # Safety
///
/// `gochar_json` null or a NUL-terminated string.
unsafe fn gochar_request_of(
    gochar_json: *const c_char,
) -> Result<Option<teistro::GocharRequest>, Error> {
    // SAFETY: the caller's contract.
    unsafe { optional_text(gochar_json, "gochar_json") }?
        .map(teistro::GocharRequest::from_json)
        .transpose()
}

/// The hit list a request's `hits_json` asks for, none for null; the
/// façade reads and checks the record ([`teistro::HitRequest::from_json`]),
/// naming a refusal from its root, `hits.to`.
///
/// # Safety
///
/// `hits_json` null or a NUL-terminated string.
unsafe fn hit_request_of(hits_json: *const c_char) -> Result<Option<teistro::HitRequest>, Error> {
    // SAFETY: the caller's contract.
    unsafe { optional_text(hits_json, "hits_json") }?
        .map(teistro::HitRequest::from_json)
        .transpose()
}

/// Every chart's hit list, empty when none was asked for: **one batch**
/// through the façade ([`teistro::ChartArea::hits_many`]), which scans the
/// sky once for every chart of the request. A batch of none asks the
/// façade nothing, which would refuse it by `natals`: the boundary answers
/// an empty batch with an empty blob.
fn hits_of(
    sdk: &teistro::Context,
    documents: &[Document],
    asked: Option<&teistro::HitRequest>,
) -> Result<Vec<Vec<teistro::Hit>>, Error> {
    match asked {
        Some(asked) if !documents.is_empty() => sdk
            .chart()
            .hits_many(documents, asked)
            .map(|found| found.value),
        _ => Ok(Vec::new()),
    }
}

/// The KP reading a request's `kp_json` asks for, none for null; the
/// façade reads the record ([`teistro::KpRequest::from_json`]), naming a
/// refusal from its root, `kp.number`. A record naming no clock takes the
/// chart request's own, which is the clock the charts were asked on.
///
/// # Safety
///
/// `kp_json` null or a NUL-terminated string.
unsafe fn kp_request_of(
    kp_json: *const c_char,
    clock: UtcOffset,
) -> Result<Option<teistro::KpRequest>, Error> {
    // SAFETY: the caller's contract.
    let Some(text) = unsafe { optional_text(kp_json, "kp_json") }? else {
        return Ok(None);
    };
    let request = teistro::KpRequest::from_json(text)?;
    Ok(Some(match request.clock() {
        Some(_) => request,
        None => request.on_clock(clock),
    }))
}

/// Every chart's KP reading as the canonical JSON the `kp` section carries:
/// an array with one reading a chart, or nothing at all when none was asked
/// for, as `drawings` is.
fn kp_json(
    sdk: &teistro::Context,
    documents: &[Document],
    asked: Option<&teistro::KpRequest>,
) -> Result<String, Error> {
    let Some(asked) = asked else {
        return Ok(String::new());
    };
    let readings = documents
        .iter()
        .map(|document| sdk.chart().kp_reading(document, asked))
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(teistro_core::envelope::canonical_json(&readings))
}

/// The dignities a request's `dignities_json` asks for, none for null; the
/// crate reads the record ([`teistro::DignityRequest::from_json`]), naming
/// a refusal from its root, `dignities.rules.terms`.
///
/// # Safety
///
/// `dignities_json` null or a NUL-terminated string.
unsafe fn dignity_request_of(
    dignities_json: *const c_char,
) -> Result<Option<teistro::DignityRequest>, Error> {
    // SAFETY: the caller's contract.
    unsafe { optional_text(dignities_json, "dignities_json") }?
        .map(teistro::DignityRequest::from_json)
        .transpose()
}

/// The fortitudes a request's `fortitudes_json` asks for, none for null;
/// the crate reads the record ([`teistro::FortitudeRequest::from_json`]),
/// naming a refusal from its root, `fortitudes.rules.beamsDeg`.
///
/// # Safety
///
/// `fortitudes_json` null or a NUL-terminated string.
unsafe fn fortitude_request_of(
    fortitudes_json: *const c_char,
) -> Result<Option<teistro::FortitudeRequest>, Error> {
    // SAFETY: the caller's contract.
    unsafe { optional_text(fortitudes_json, "fortitudes_json") }?
        .map(teistro::FortitudeRequest::from_json)
        .transpose()
}

/// The lots a request's `lots_json` asks for, none for null; the crate
/// reads the record ([`teistro::LotRequest::from_json`]), naming a
/// refusal from its root, `lots.fortune`.
///
/// # Safety
///
/// `lots_json` null or a NUL-terminated string.
unsafe fn lot_request_of(lots_json: *const c_char) -> Result<Option<teistro::LotRequest>, Error> {
    // SAFETY: the caller's contract.
    unsafe { optional_text(lots_json, "lots_json") }?
        .map(teistro::LotRequest::from_json)
        .transpose()
}

/// The considerations a request's `considerations_json` asks for, none
/// for null; the crate reads the record
/// ([`teistro::ConsiderationRules::from_json`]), naming a refusal from
/// its root, `considerations.moonLateFromDeg`.
///
/// # Safety
///
/// `considerations_json` null or a NUL-terminated string.
unsafe fn consideration_rules_of(
    considerations_json: *const c_char,
) -> Result<Option<teistro::ConsiderationRules>, Error> {
    // SAFETY: the caller's contract.
    unsafe { optional_text(considerations_json, "considerations_json") }?
        .map(teistro::ConsiderationRules::from_json)
        .transpose()
}

/// The progressions a request's `progressions_json` asks for, none for
/// null; the crate reads the record ([`teistro::ProgressionsRequest::from_json`]),
/// which names a refusal from its root, `progressions.year`.
///
/// # Safety
///
/// `progressions_json` null or a NUL-terminated string.
unsafe fn progressions_request_of(
    progressions_json: *const c_char,
) -> Result<Option<teistro::ProgressionsRequest>, Error> {
    // SAFETY: the caller's contract.
    unsafe { optional_text(progressions_json, "progressions_json") }?
        .map(teistro::ProgressionsRequest::from_json)
        .transpose()
}

/// Every chart's progressions, none when none was asked for: each birth
/// read through the request, its progressed chart founded by `request`.
///
/// `request` is the batch's **foundation only** (its place, clock and
/// kind): the boundary writes a progressed chart's grahas and angles and
/// nothing else, so founding it with every section the batch asked for
/// would compute drawings, strengths and dashas no column carries, twice
/// per birth (the progressed chart and the solar arc's).
fn progressions_of(
    sdk: &teistro::Context,
    documents: &[Document],
    asked: Option<&teistro::ProgressionsRequest>,
    request: &ChartRequest,
) -> Result<Vec<teistro::Progressions>, Error> {
    let Some(asked) = asked else {
        return Ok(Vec::new());
    };
    documents
        .iter()
        .enumerate()
        .map(|(at, document)| {
            sdk.chart()
                .progressions(document, asked, request)
                .map_err(|error| error.with_hint(format!("chart {at}")))
        })
        .collect()
}

/// The Western tables a batch was asked for, read once: each chart's own
/// aspects, its synastry with the record's partner, its declinations with
/// the parallels among its planets, its antiscia and its equal distances.
/// Each is empty when its record was null.
struct WesternTables {
    aspects: Vec<Vec<teistro::WesternAspectRow>>,
    synastry: Vec<teistro::PartnerReading>,
    declinations: Vec<teistro::Declinations>,
    parallels: Vec<Vec<teistro::ParallelRow>>,
    antiscia: Vec<teistro::Antiscia>,
    midpoints: Vec<Vec<teistro::MidpointRow>>,
    davisons: Vec<teistro::Partner>,
}

impl WesternTables {
    /// Every table `records` asks of `documents`; a refusal is named under
    /// its record's root, and one read chart by chart says which chart.
    /// `clock` is the batch's, which a Davison birth reads the charts on.
    fn of(
        sdk: &teistro::Context,
        documents: &[Document],
        records: &AskedRecords,
        clock: UtcOffset,
    ) -> Result<WesternTables, Error> {
        let each = |root: &'static str| {
            move |at: usize, error: Error| error.under(root).with_hint(format!("chart {at}"))
        };
        let aspects = chart_by_chart(
            records.western_aspects.as_ref(),
            documents,
            "westernAspects",
            |document, asked| sdk.chart().western_aspects(document, asked),
        )?;
        let synastry = records.synastry.as_ref().map_or_else(
            || Ok(Vec::new()),
            |asked| {
                sdk.chart()
                    .synastry_with(documents, asked)
                    .map_err(|error| error.under("synastry"))
            },
        )?;
        let (declinations, parallels) = match &records.parallels {
            None => (Vec::new(), Vec::new()),
            Some(asked) => documents
                .iter()
                .enumerate()
                .map(|(at, document)| {
                    let declined = sdk
                        .chart()
                        .declinations(document)
                        .map_err(|error| each("parallels")(at, error))?;
                    let parallels = teistro::western::parallels(&declined.grahas, asked)
                        .map_err(|error| each("parallels")(at, error))?;
                    Ok((declined, parallels))
                })
                .collect::<Result<Vec<_>, Error>>()?
                .into_iter()
                .unzip(),
        };
        Ok(WesternTables {
            aspects,
            synastry,
            declinations,
            parallels,
            antiscia: chart_by_chart(
                records.antiscia.as_ref(),
                documents,
                "antiscia",
                |document, asked| sdk.chart().antiscia(document, asked),
            )?,
            midpoints: chart_by_chart(
                records.midpoints.as_ref(),
                documents,
                "midpoints",
                |document, asked| sdk.chart().midpoints(document, asked),
            )?,
            davisons: records
                .synastry
                .as_ref()
                .map(|asked| asked.davisons(documents, clock))
                .transpose()
                .map_err(|error| error.under("synastry"))?
                .flatten()
                .unwrap_or_default(),
        })
    }
}

/// One table a chart, each read by `read` under the asked record, or none
/// when it was null; a refusal is named under the record's `root` and says
/// which chart.
fn chart_by_chart<A, T>(
    asked: Option<&A>,
    documents: &[Document],
    root: &'static str,
    read: impl Fn(&Document, &A) -> Result<T, Error>,
) -> Result<Vec<T>, Error> {
    let Some(asked) = asked else {
        return Ok(Vec::new());
    };
    documents
        .iter()
        .enumerate()
        .map(|(at, document)| {
            read(document, asked)
                .map_err(|error| error.under(root).with_hint(format!("chart {at}")))
        })
        .collect()
}

/// The perfection a request's `perfection_json` asks for, none for null;
/// the crate reads the record ([`teistro::PerfectionRequest::from_json`]),
/// naming a refusal from its root, `perfection.quesited`.
///
/// # Safety
///
/// `perfection_json` null or a NUL-terminated string.
unsafe fn perfection_request_of(
    perfection_json: *const c_char,
) -> Result<Option<teistro::PerfectionRequest>, Error> {
    // SAFETY: the caller's contract.
    unsafe { optional_text(perfection_json, "perfection_json") }?
        .map(teistro::PerfectionRequest::from_json)
        .transpose()
}

/// Every chart's perfection, none when none was asked for: weighed on the
/// fortitudes the request asked for, and Lilly's when it asked for none.
fn perfections_of(
    sdk: &teistro::Context,
    documents: &[Document],
    asked: Option<teistro::PerfectionRequest>,
    fortitudes: &[teistro::Fortitudes],
) -> Result<Vec<(teistro::Matter, teistro::PerfectionRules)>, Error> {
    let Some(asked) = asked else {
        return Ok(Vec::new());
    };
    let read = |document: &Document, fortitudes: &teistro::Fortitudes| {
        sdk.chart()
            .perfection_in(document, fortitudes, &asked)
            .map(|matter| (matter, asked.rules))
    };
    if fortitudes.is_empty() {
        let lilly = teistro::FortitudeRequest::default();
        return documents
            .iter()
            .map(|document| read(document, &sdk.chart().fortitudes(document, &lilly)?))
            .collect();
    }
    documents
        .iter()
        .zip(fortitudes)
        .map(|(document, fortitudes)| read(document, fortitudes))
        .collect()
}

/// Every chart's considerations, none when none was asked for: read from
/// the fortitudes the request asked for, and Lilly's when it asked for
/// none, so no chart's fortitudes are read twice.
fn considerations_of(
    sdk: &teistro::Context,
    documents: &[Document],
    asked: Option<teistro::ConsiderationRules>,
    fortitudes: &[teistro::Fortitudes],
) -> Result<Vec<teistro::Considerations>, Error> {
    let Some(rules) = asked else {
        return Ok(Vec::new());
    };
    if fortitudes.is_empty() {
        let lilly = teistro::FortitudeRequest::default();
        return documents
            .iter()
            .map(|document| sdk.chart().considerations(document, &lilly, rules))
            .collect();
    }
    documents
        .iter()
        .zip(fortitudes)
        .map(|(document, read)| {
            teistro::hellenistic::considerations(read, document.foundation.timing.hora.lord, rules)
        })
        .collect()
}

/// Every chart's fourteen lots, none when none was asked for.
fn lots_of(
    sdk: &teistro::Context,
    documents: &[Document],
    asked: Option<teistro::LotRequest>,
) -> Result<Vec<teistro::LotReading>, Error> {
    let Some(asked) = asked else {
        return Ok(Vec::new());
    };
    documents
        .iter()
        .map(|document| {
            sdk.chart()
                .lots_with_request(document, &teistro::Lot::ALL, asked)
        })
        .collect()
}

/// Every chart's accidental fortitudes, none when none was asked for.
fn fortitudes_of(
    sdk: &teistro::Context,
    documents: &[Document],
    asked: Option<&teistro::FortitudeRequest>,
) -> Result<Vec<teistro::Fortitudes>, Error> {
    let Some(asked) = asked else {
        return Ok(Vec::new());
    };
    documents
        .iter()
        .map(|document| sdk.chart().fortitudes(document, asked))
        .collect()
}

/// Every chart's essential dignities, none when none was asked for.
fn dignities_of(
    sdk: &teistro::Context,
    documents: &[Document],
    asked: Option<&teistro::DignityRequest>,
) -> Result<Vec<teistro::Dignities>, Error> {
    let Some(asked) = asked else {
        return Ok(Vec::new());
    };
    documents
        .iter()
        .map(|document| sdk.chart().dignities(document, asked))
        .collect()
}

/// The Sade Sati a request's `sade_sati_json` asks for, none for null; the
/// façade reads and checks the record
/// ([`teistro::SadeSatiRequest::from_json`]), naming a refusal from its
/// root, `sadeSati.to`.
///
/// # Safety
///
/// `sade_sati_json` null or a NUL-terminated string.
unsafe fn sade_sati_request_of(
    sade_sati_json: *const c_char,
) -> Result<Option<teistro::SadeSatiRequest>, Error> {
    // SAFETY: the caller's contract.
    unsafe { optional_text(sade_sati_json, "sade_sati_json") }?
        .map(teistro::SadeSatiRequest::from_json)
        .transpose()
}

impl GrahaColumns {
    /// The grahas, charts outermost, in the order `grahas` declares them.
    fn write(
        &self,
        writer: &mut Writer<'_>,
        name: &str,
    ) -> Result<(), teistro_idl::blob::BlobError> {
        writer.columns(
            name,
            self.ids.len(),
            &[
                ColumnData::U16(&self.ids),
                ColumnData::F64(&self.longitudes),
                ColumnData::F64(&self.tropicals),
                ColumnData::F64(&self.latitudes),
                ColumnData::F64(&self.distances),
                ColumnData::F64(&self.speeds),
                ColumnData::U8(&self.house_bhava),
                ColumnData::U16(&self.house_method),
                ColumnData::F64(&self.house_through),
                ColumnData::F64(&self.house_from),
                ColumnData::U8(&self.placed_bhava),
                ColumnData::U16(&self.placed_method),
                ColumnData::F64(&self.placed_through),
                ColumnData::F64(&self.placed_from),
            ],
        )
    }
}

/// Every section's columns, built from the documents in one place.
///
/// Thirteen `XColumns::of(documents)` lines in a row said nothing that
/// their own names did not, and pushed the writer past the length a
/// reader can hold. The **writes** stay together and in the schema's
/// declared order, because that order is the blob's layout and scattering
/// it would hide the one thing a reader of `encode` needs to see.
struct Sections {
    vargas: VargaColumns,
    aspects: AspectColumns,
    points: PointColumns,
    bhavas: BhavaColumns,
    states: StateColumns,
    dashas: DashaColumns,
    ashtakavarga: AshtakavargaColumns,
    vimshopaka: VimshopakaColumns,
    shadbala: ShadbalaColumns,
    bhava_bala: BhavaBalaColumns,
    vaiseshikamsa: VaiseshikamsaColumns,
    dasha_phala: DashaPhalaColumns,
    jaimini: JaiminiColumns,
    years: PraveshaColumns,
}

impl Sections {
    fn of(
        documents: &[Document],
        graha_count: usize,
        registered: &teistro::dasha::DashaSystems,
        praveshas: &[teistro::Varsha],
    ) -> Result<Sections, Error> {
        Ok(Sections {
            vargas: VargaColumns::of(documents, graha_count)?,
            aspects: AspectColumns::of(documents),
            points: PointColumns::of(documents),
            bhavas: BhavaColumns::of(documents),
            states: StateColumns::of(documents),
            dashas: DashaColumns::of(documents, registered)?,
            ashtakavarga: AshtakavargaColumns::of(documents),
            vimshopaka: VimshopakaColumns::of(documents),
            shadbala: ShadbalaColumns::of(documents),
            bhava_bala: BhavaBalaColumns::of(documents),
            vaiseshikamsa: VaiseshikamsaColumns::of(documents),
            dasha_phala: DashaPhalaColumns::of(documents),
            jaimini: JaiminiColumns::of(documents),
            years: PraveshaColumns::of(praveshas)?,
        })
    }
}

/// The façade's request for what `asked` lists beside its instants: its
/// sections, divisional charts, drawings and dasha systems, each refused
/// by the field the caller wrote.
///
/// # Safety
///
/// `asked.vargas`, `asked.dashas` and `asked.drawings` each null with a
/// zero count, or that many readable values.
unsafe fn chart_request_of(
    asked: &TsChartRequest,
    place: Place,
    kind: ChartKind,
    clock: UtcOffset,
) -> Result<ChartRequest, Error> {
    // SAFETY: the caller's contract, for `varga_count` readable `u16`s.
    let asked_vargas = unsafe { slice(asked.vargas, asked.varga_count, "vargas") }?;
    let mut vargas = Vec::with_capacity(asked_vargas.len());
    for id in asked_vargas {
        vargas.push(Varga::from_id(*id).ok_or_else(|| {
            Error::new(
                Status::InvalidArg,
                format!("no divisional chart with id {id}"),
            )
            .with_field("vargas")
        })?);
    }
    // SAFETY: as above, for `dasha_count` readable `u16`s.
    let asked_dashas = unsafe { slice(asked.dashas, asked.dasha_count, "dashas") }?;
    let mut dashas = Vec::with_capacity(asked_dashas.len());
    // A catalogued id or one the context registered; the façade refuses
    // any other by its place, with the systems it can compute.
    for id in asked_dashas {
        dashas.push(KeyId::new(Kind::DashaSystem, *id));
    }
    // SAFETY: as above, for `drawing_count` readable `u32`s.
    let asked_drawings = unsafe { slice(asked.drawings, asked.drawing_count, "drawings") }?;
    let mut drawings = Vec::with_capacity(asked_drawings.len());
    for (index, packed) in asked_drawings.iter().enumerate() {
        let (layout, varga) = (packed >> 16, packed & 0xFFFF);
        let varga = u16::try_from(varga)
            .ok()
            .and_then(Varga::from_id)
            .ok_or_else(|| {
                Error::invalid_arg(format!(
                    "drawing {index} names divisional chart id {varga}, which is none"
                ))
                .with_field(format!("drawings[{index}]"))
            })?;
        let layout = KeyId::new(Kind::ChartLayout, u16::try_from(layout).unwrap_or(u16::MAX));
        drawings.push((layout, varga));
    }
    Ok(sections_of(
        asked.sections,
        ChartRequest::at(place, clock).with_kind(kind),
    )
    .with_vargas(vargas)
    .with_drawings(drawings)
    .with_dashas(dashas))
}

/// Where a request casts its charts, of what kind, and against which
/// clock — each refused by the field the caller wrote.
fn where_and_when(asked: &TsChartRequest) -> Result<(Place, ChartKind, UtcOffset), Error> {
    let place = Place::new(
        Latitude::try_new(asked.latitude_deg)
            .map_err(|e| Error::from(e).with_field("latitude_deg"))?,
        Longitude::try_new(asked.longitude_deg)
            .map_err(|e| Error::from(e).with_field("longitude_deg"))?,
        Altitude::try_new(asked.altitude_m).map_err(|e| Error::from(e).with_field("altitude_m"))?,
    );
    let kind = ChartKind::from_id(asked.kind).ok_or_else(|| {
        Error::new(
            Status::InvalidArg,
            format!("no chart kind with id {}", asked.kind),
        )
        .with_field("kind")
    })?;
    let clock = UtcOffset::try_from_seconds(asked.utc_offset_seconds)
        .map_err(|e| Error::from(e).with_field("utc_offset_seconds"))?;
    Ok((place, kind, clock))
}

/// Every chart's annual charts, empty when none were asked for.
///
/// Composed by the façade ([`teistro::ChartArea::varsha`]), one birth at a
/// time; a refusal says which chart of the batch it was refused for.
fn praveshas_of(
    sdk: &teistro::Context,
    documents: &[Document],
    birth_clock: teistro::UtcOffset,
    asked: Option<&teistro::VarshaRequest>,
) -> Result<Vec<teistro::Varsha>, Error> {
    let Some(asked) = asked else {
        return Ok(Vec::new());
    };
    documents
        .iter()
        .enumerate()
        .map(|(at, document)| {
            sdk.chart()
                .varsha(document, birth_clock, asked)
                .map_err(|error| error.with_hint(format!("chart {at}")))
        })
        .collect()
}

/// Every chart's transits, empty when none were asked for: one batch per
/// chart through the façade ([`teistro::ChartArea::gochar`]), which places
/// the grahas at every instant in one request; a refusal says which chart
/// of the batch it was refused for.
fn gochar_of(
    sdk: &teistro::Context,
    documents: &[Document],
    asked: Option<&teistro::GocharRequest>,
) -> Result<Vec<Vec<teistro::gochar::GocharReading>>, Error> {
    let Some(asked) = asked else {
        return Ok(Vec::new());
    };
    documents
        .iter()
        .enumerate()
        .map(|(at, document)| {
            sdk.chart()
                .gochar(document, asked)
                .map(|read| read.value)
                .map_err(|error| error.with_hint(format!("chart {at}")))
        })
        .collect()
}

/// The rule set a request's `rules_json` names, or none for null; a refusal is
/// named from the request's root, `rules.rules[0]`.
///
/// # Safety
///
/// `rules_json` null or a NUL-terminated string.
unsafe fn rule_set_of(rules_json: *const c_char) -> Result<Option<RuleSet>, Error> {
    // SAFETY: the caller's contract.
    unsafe { optional_text(rules_json, "rules_json") }?
        .map(|text| RuleRequest::from_json(text).and_then(|request| request.rule_set()))
        .transpose()
        .map_err(|error| error.under(RULES))
}

/// The JSON records a chart request carries beside its sections, each read
/// and checked by the façade's own reader before anything is founded, so a
/// bad record is refused before a chart is paid for, and each refusal is
/// named from the record's root as every binding writes it.
struct AskedRecords {
    theme: Option<Theme>,
    rules: Option<RuleSet>,
    plans: PlanRequest,
    varsha: Option<teistro::VarshaRequest>,
    gochar: Option<teistro::GocharRequest>,
    hits: Option<teistro::HitRequest>,
    sade_sati: Option<teistro::SadeSatiRequest>,
    kp: Option<teistro::KpRequest>,
    dignities: Option<teistro::DignityRequest>,
    fortitudes: Option<teistro::FortitudeRequest>,
    lots: Option<teistro::LotRequest>,
    considerations: Option<teistro::ConsiderationRules>,
    perfection: Option<teistro::PerfectionRequest>,
    progressions: Option<teistro::ProgressionsRequest>,
    western_aspects: Option<teistro::AspectRequest>,
    synastry: Option<teistro::PartnerSynastry>,
    parallels: Option<teistro::ParallelRequest>,
    antiscia: Option<teistro::AntisciaRequest>,
    midpoints: Option<teistro::MidpointRequest>,
}

impl AskedRecords {
    /// Every record `asked` carries; `clock` is the request's own, which a
    /// KP record naming none takes.
    ///
    /// # Safety
    ///
    /// Each of `asked`'s record fields null or a NUL-terminated string.
    unsafe fn of(asked: &TsChartRequest, clock: UtcOffset) -> Result<AskedRecords, Error> {
        // SAFETY: the caller's contract, for every field read below. The
        // theme names its fields from its own root, `theme.style.ink`,
        // which is what every binding calls it, so its refusal stands.
        unsafe {
            Ok(AskedRecords {
                theme: optional_text(asked.theme_json, "theme_json")?
                    .map(Theme::from_json)
                    .transpose()?,
                rules: rule_set_of(asked.rules_json)?,
                plans: plan_request_of(asked.interpret_json)?,
                varsha: varsha_request_of(asked.varsha_json)?,
                gochar: gochar_request_of(asked.gochar_json)?,
                hits: hit_request_of(asked.hits_json)?,
                sade_sati: sade_sati_request_of(asked.sade_sati_json)?,
                kp: kp_request_of(asked.kp_json, clock)?,
                dignities: dignity_request_of(asked.dignities_json)?,
                fortitudes: fortitude_request_of(asked.fortitudes_json)?,
                lots: lot_request_of(asked.lots_json)?,
                considerations: consideration_rules_of(asked.considerations_json)?,
                perfection: perfection_request_of(asked.perfection_json)?,
                progressions: progressions_request_of(asked.progressions_json)?,
                western_aspects: optional_text(asked.western_aspects_json, "western_aspects_json")?
                    .map(teistro::AspectRequest::from_json)
                    .transpose()?,
                synastry: optional_text(asked.synastry_json, "synastry_json")?
                    .map(teistro::PartnerSynastry::from_json)
                    .transpose()?,
                parallels: optional_text(asked.parallels_json, "parallels_json")?
                    .map(teistro::ParallelRequest::from_json)
                    .transpose()?,
                antiscia: optional_text(asked.antiscia_json, "antiscia_json")?
                    .map(teistro::AntisciaRequest::from_json)
                    .transpose()?,
                midpoints: optional_text(asked.midpoints_json, "midpoints_json")?
                    .map(teistro::MidpointRequest::from_json)
                    .transpose()?,
            })
            .and_then(AskedRecords::one_table)
        }
    }

    /// The record with one request for the essential dignities: the
    /// fortitudes carry their own, so a `dignities` record beside them is
    /// refused rather than one of the two silently winning.
    fn one_table(self) -> Result<AskedRecords, Error> {
        if self.dignities.is_some() && self.fortitudes.is_some() {
            return Err(Error::invalid_arg(
                "the essential dignities asked for twice, by `dignities` and by `fortitudes`",
            )
            .with_field("dignities")
            .with_hint(
                "the fortitudes answer the essential dignities too: put this record under `fortitudes.dignities` and drop `dignities`",
            ));
        }
        Ok(self)
    }
}

/// What a chart request asks for beside its charts, as the façade takes it.
fn plan_inputs<'r>(
    rules: Option<&'r RuleSet>,
    sade_sati: Option<&'r teistro::SadeSatiRequest>,
) -> PlanInputs<'r> {
    let inputs = PlanInputs::from(rules);
    match sade_sati {
        Some(window) => inputs.with_sade_sati(window),
        None => inputs,
    }
}

/// What [`read_charts`] answers, ready to encode.
struct ReadCharts {
    /// The documents, the batch's provenance sealed over the list.
    founded: Envelope<Vec<Document>>,
    /// Each chart's own content hash, in the batch's order.
    hashes: Vec<teistro::Hash>,
    /// What every chart answered by rule, canonical JSON; empty for none.
    rules: String,
    /// What every chart has to say, canonical JSON; empty for none.
    plans: String,
    /// Every chart's Sade Sati report, empty when no window was asked for.
    sade_sati: Vec<teistro::sade_sati::Report>,
}

/// The charts a request asks for, each chart's own content hash, the
/// canonical JSON of what they answer by rule and of the plans they were
/// asked to say — each empty when the request asked for none — and each
/// chart's Sade Sati report.
///
/// The reading, the searching and the composing are the façade's
/// ([`teistro::ChartArea::interpreted`]), so a plan is composed in one place
/// for Rust and every binding, and Saturn is scanned once for the reports
/// the blob carries and the plan that says them; this only encodes what it
/// answered.
fn read_charts(
    sdk: &teistro::Context,
    instants: &[JulianDay<Utc>],
    request: &ChartRequest,
    inputs: PlanInputs<'_>,
    asked: PlanRequest,
) -> Result<ReadCharts, Error> {
    let rules = inputs.rules;
    let read = sdk.chart().interpreted(instants, request, inputs, asked)?;
    let hashes = read.value.iter().map(|chart| chart.content_hash).collect();
    let rules_json = if rules.is_some() {
        let readings: Vec<_> = read
            .value
            .iter()
            .filter_map(|chart| chart.reading.as_ref())
            .collect();
        teistro_core::envelope::canonical_json(&readings)
    } else {
        String::new()
    };
    let plans_json = if asked.asks_for_something() {
        let plans: Vec<_> = read.value.iter().map(|chart| &chart.plans).collect();
        teistro_core::envelope::canonical_json(&plans)
    } else {
        String::new()
    };
    let mut documents = Vec::with_capacity(read.value.len());
    let mut sade_sati = Vec::new();
    for chart in read.value {
        documents.push(chart.document);
        sade_sati.extend(chart.sade_sati);
    }
    Ok(ReadCharts {
        founded: Envelope::new(documents, read.provenance),
        hashes,
        rules: rules_json,
        plans: plans_json,
        sade_sati,
    })
}

/// Founds a chart at an instant and a place and answers with its blob:
/// where every graha stands, in which bhava under both readings, in
/// which zodiac, on which day, at what time of that day.
///
/// Everything but the request is the context's settings, so two calls
/// under one context are comparable and the settings hash says why. The
/// civil calendar the day's date is read in comes from
/// `calendars.civil_calendar`, which is what that knob was waiting for.
///
/// A context without an ephemeris is `CAPABILITY`; a provider failure is
/// `PROVIDER` with the provider's own code in the last error.
///
/// `api: blob=charts`
///
/// # Safety
///
/// `context` must be a live handle; `request` valid for a read; `out_blob`
/// valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_chart_found(
    context: *const TsContext,
    request: *const TsChartRequest,
    out_blob: *mut TsBlob,
) -> Status {
    with_context(context, |ctx| {
        // SAFETY: the caller promises a readable request; `read_in`
        // checks the handshake before anything else reads a field.
        let asked = *unsafe { read_in(request, "request") }?;
        let (place, kind, clock) = where_and_when(&asked)?;
        // SAFETY: the entry point's contract — the caller promises
        // `instant_count` readable doubles at `instants`, or null and zero.
        let instants: Vec<JulianDay<Utc>> =
            unsafe { slice(asked.instants, asked.instant_count, "instants") }?
                .iter()
                .map(|jd| JulianDay::<Utc>::literal(*jd))
                .collect();
        // SAFETY: the entry point's contract, for each list `asked` names.
        let request = unsafe { chart_request_of(&asked, place, kind, clock) }?;
        // **The façade reads it**, which is what the dependency inversion
        // was for: `rust-consumer-surface.md` moved the SDK's
        // composition into `teistro` and had this crate depend on it, and
        // `TsContext::build` became a call into the builder — but this
        // entry point went on resolving the calendar, substituting the
        // ayanamsha and building the solar model itself. That was the
        // second copy of the chart composition, kept equal to the first
        // by hand and by nothing else.
        //
        // It also seals, so there is nothing left for the boundary to do
        // but encode what it was given.
        // SAFETY: the entry point's contract — each record null, or a
        // NUL-terminated string.
        let records = unsafe { AskedRecords::of(&asked, clock) }?;
        // The lots a chart reports are the lots its time lords release
        // from, so one `lots` record sets both.
        let request = match records.lots {
            Some(rules) => request.with_lot_rules(rules),
            None => request,
        };
        let ReadCharts {
            founded,
            hashes,
            rules: rules_json,
            plans: plans_json,
            sade_sati,
        } = read_charts(
            ctx.sdk(),
            &instants,
            &request,
            plan_inputs(records.rules.as_ref(), records.sade_sati.as_ref()),
            records.plans,
        )?;
        let svgs = match &records.theme {
            Some(theme) => svgs_json(ctx.sdk(), &founded.value, theme)?,
            None => String::new(),
        };
        let praveshas = praveshas_of(
            ctx.sdk(),
            &founded.value,
            request.offset(),
            records.varsha.as_ref(),
        )?;
        let transits = gochar_of(ctx.sdk(), &founded.value, records.gochar.as_ref())?;
        let hits = hits_of(ctx.sdk(), &founded.value, records.hits.as_ref())?;
        let kp = kp_json(ctx.sdk(), &founded.value, records.kp.as_ref())?;
        let dignities = dignities_of(ctx.sdk(), &founded.value, records.dignities.as_ref())?;
        let fortitudes = fortitudes_of(ctx.sdk(), &founded.value, records.fortitudes.as_ref())?;
        let lots = lots_of(ctx.sdk(), &founded.value, records.lots)?;
        let considerations = considerations_of(
            ctx.sdk(),
            &founded.value,
            records.considerations,
            &fortitudes,
        )?;
        let perfections =
            perfections_of(ctx.sdk(), &founded.value, records.perfection, &fortitudes)?;
        let progressions = progressions_of(
            ctx.sdk(),
            &founded.value,
            records.progressions.as_ref(),
            &ChartRequest::at(place, clock).with_kind(kind),
        )?;
        let western = WesternTables::of(ctx.sdk(), &founded.value, &records, clock)?;
        let encoded = encode(
            &founded.value,
            &place,
            kind,
            &founded.provenance,
            Composed {
                svgs: &svgs,
                rules: &rules_json,
                plans: &plans_json,
                praveshas: &praveshas,
                gochar: &transits,
                gochar_instants: records
                    .gochar
                    .as_ref()
                    .map_or(&[], teistro::GocharRequest::instants),
                hits: &hits,
                sade_sati: &sade_sati,
                kp: &kp,
                dignities: &dignities,
                fortitudes: &fortitudes,
                lots: &lots,
                considerations: &considerations,
                perfections: &perfections,
                progressions: &progressions,
                western_aspects: &western.aspects,
                synastry: &western.synastry,
                declinations: &western.declinations,
                parallels: &western.parallels,
                antiscia: &western.antiscia,
                midpoints: &western.midpoints,
                davisons: &western.davisons,
                hashes: &hashes,
            },
            ctx.sdk().dashas(),
        )?;
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_blob, "out_blob", TsBlob::from_vec(encoded)) }
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        clippy::cast_possible_wrap,
        reason = "a test fails by panicking, indexes its own blob and compares the numbers it wrote"
    )]

    use super::{
        Altitude, ChartKind, JulianDay, Latitude, Longitude, Place, Provenance, TsDayPart,
        TsDayState, TsGhatiReckoning, TsHoraReckoning, TsPolarDayPolicy, TsPolarKind, TsReading,
        TsSunrise, Utc, UtcOffset,
    };
    // The founder and the solar model, which this module builds by hand:
    // the entry point above founds through the façade now, and a test of
    // the *encoder* wants a founding it can control rather than a
    // context.
    use teistro_astro::precession::PrecessionModel;
    use teistro_calendar::shipped;
    use teistro_calendar::solar::drik::DrikSun;
    use teistro_chart::bhava::Reading;
    use teistro_chart::day::DayPart;
    use teistro_chart::foundation::Founder;
    use teistro_core::catalogue::{Ayanamsha, Varga};
    use teistro_core::settings::{GhatiReckoning, HoraReckoning, PolarDayPolicy, Sunrise};
    use teistro_serial::Document;
    use teistro_time::local_day::{DayState, PolarKind};

    /// A batch encoded as a natal chart with no drawings and no registered
    /// dasha systems, which is every encoding these tests make.
    fn encoded(
        documents: &[Document],
        place: &Place,
        provenance: &Provenance,
    ) -> Result<Vec<u8>, teistro_core::error::Error> {
        let (_, hashes) = teistro_core::envelope::content_hashes(documents);
        super::encode(
            documents,
            place,
            ChartKind::Natal,
            provenance,
            super::Composed {
                hashes: &hashes,
                ..super::Composed::default()
            },
            &teistro::dasha::DashaSystems::new(),
        )
    }

    /// Every member of every knob crosses, and crosses to an id of its
    /// own.
    ///
    /// This is the guard the three knobs cannot get from a match. A
    /// settings knob is `#[non_exhaustive]`, so a member added later
    /// falls into a wildcard rather than breaking the build; iterating
    /// the knob's own `ALL` fails here instead, naming the member that
    /// has no id.
    #[test]
    fn every_knob_member_crosses_to_an_id_of_its_own() {
        use super::{TsAshtakavargaGoodFrom, TsGocharFrom, TsNodeObstruction, TsNodeVedha};

        let sunrises: Vec<u8> = Sunrise::ALL
            .iter()
            .map(|member| {
                TsSunrise::of(*member)
                    .unwrap_or_else(|| panic!("`{member}` has no id at the boundary"))
                    as u8
            })
            .collect();
        assert_eq!(sunrises, vec![0, 1, 2, 3], "one id each, in order");

        let ghatis: Vec<u8> = GhatiReckoning::ALL
            .iter()
            .map(|member| {
                TsGhatiReckoning::of(*member)
                    .unwrap_or_else(|| panic!("`{member}` has no id at the boundary"))
                    as u8
            })
            .collect();
        assert_eq!(ghatis, vec![0, 1]);

        let horas: Vec<u8> = HoraReckoning::ALL
            .iter()
            .map(|member| {
                TsHoraReckoning::of(*member)
                    .unwrap_or_else(|| panic!("`{member}` has no id at the boundary"))
                    as u8
            })
            .collect();
        assert_eq!(horas, vec![0, 1]);

        let vedhas: Vec<u8> = teistro_core::settings::NodeVedha::ALL
            .iter()
            .map(|member| {
                TsNodeVedha::of(*member)
                    .unwrap_or_else(|| panic!("`{member}` has no id at the boundary"))
                    as u8
            })
            .collect();
        assert_eq!(vedhas, vec![0, 1]);

        let obstructions: Vec<u8> = teistro_core::settings::NodeObstruction::ALL
            .iter()
            .map(|member| {
                TsNodeObstruction::of(*member)
                    .unwrap_or_else(|| panic!("`{member}` has no id at the boundary"))
                    as u8
            })
            .collect();
        assert_eq!(obstructions, vec![0, 1, 2]);

        let references: Vec<u8> = teistro::GocharFrom::ALL
            .iter()
            .map(|member| {
                TsGocharFrom::of(*member)
                    .unwrap_or_else(|| panic!("`{member:?}` has no id at the boundary"))
                    as u8
            })
            .collect();
        assert_eq!(references, vec![0, 1]);

        let thresholds: Vec<u8> = teistro_core::settings::AshtakavargaGoodFrom::ALL
            .iter()
            .map(|member| {
                TsAshtakavargaGoodFrom::of(*member)
                    .unwrap_or_else(|| panic!("`{member}` has no id at the boundary"))
                    as u8
            })
            .collect();
        assert_eq!(thresholds, vec![0, 1]);
    }

    /// Two charts founded over the analytic test provider, with the
    /// provenance the boundary would seal.
    ///
    /// Two rather than one: a batch of one cannot tell a charts-outermost
    /// layout from a scalar one, so every per-chart section here holds
    /// rows that differ.
    fn founded() -> (
        Vec<teistro_chart::foundation::ChartFoundation>,
        Place,
        Provenance,
    ) {
        use teistro_core::settings::{DEFAULT_PROFILE, Profile, SettingsPatch};
        use teistro_port_ephemeris::test_provider::TestProvider;

        let provider = TestProvider;
        let resolved = Profile::shipped(DEFAULT_PROFILE)
            .expect("the default profile")
            .resolve(&SettingsPatch::default())
            .expect("it resolves");
        let model = DrikSun::new(
            &provider,
            Ayanamsha::Lahiri,
            resolved.settings.day.sunrise,
            resolved.settings.provider.overrides,
            teistro_astro::delta_t::DeltaTModel::TableThenModel,
        );
        let clock = UtcOffset::try_from_seconds(5 * 3600 + 45 * 60).expect("in range");
        let calendar = shipped(resolved.settings.calendars.civil_calendar)
            .expect("the profile's calendar ships");
        let place = Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.3240),
            Altitude::literal(1400.0),
        );
        let founded = Founder::new(
            &provider,
            &resolved,
            &model,
            calendar,
            &clock,
            PrecessionModel::default(),
            teistro_astro::delta_t::DeltaTModel::TableThenModel,
        )
        .found(
            &[
                JulianDay::<Utc>::literal(2_460_482.5),
                JulianDay::<Utc>::literal(2_460_600.25),
            ],
            &place,
            ChartKind::Natal,
        )
        .expect("two founded charts");
        (founded.value, place, founded.provenance)
    }

    /// A batch of founded charts crosses, and reads back as what was
    /// founded.
    ///
    /// The whole point of the module: charts computed in Rust, written
    /// to their blob and read out of it again, compared field by field
    /// against the values. Nothing else checks that the encoder and the
    /// schema agree — a column written in the wrong order would still
    /// decode, into wrong numbers — nor that the second chart's rows sit
    /// where the layout says they do.
    #[test]
    fn a_batch_of_founded_charts_round_trips_through_its_blob() {
        use teistro_idl::blob::Reader;

        let (charts, place, provenance) = founded();
        let resolved =
            teistro_core::settings::Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
                .expect("the default profile")
                .resolve(&teistro_core::settings::SettingsPatch::default())
                .expect("it resolves");
        // The **navamsha asked for**, because a section that is only
        // ever empty is a section nothing tests: the blob's layout for a
        // divisional chart is charts outermost then charts asked for,
        // and one varga over two charts is the smallest grid that can
        // come out transposed.
        let documents: Vec<Document> = charts
            .iter()
            .map(|chart| {
                Document::of(chart.clone())
                    .with_varga(
                        teistro_vargas::chart::chart(
                            chart,
                            teistro_vargas::chart::Axis::of(Varga::D9),
                        )
                        .expect("a navamsha"),
                    )
                    .with_aspects(
                        teistro_aspect::Aspects::of(chart, &resolved.settings)
                            .expect("the drishti"),
                    )
            })
            .collect();
        let bytes = encoded(&documents, &place, &provenance).expect("it encodes");
        let schema = crate::schemas::charts();
        let reader = Reader::parse(&bytes, &schema).expect("a well-formed blob");
        let graha_count = charts[0].grahas.len();

        // **By name, not by position.** The summary has grown a count
        // twice in one session, and a positional read of it reported a
        // latitude of 1.0 both times — a plausible latitude, which is
        // the worst kind of wrong.
        let field = |name: &str| reader.field("summary", name).expect(name);
        assert_eq!(field("kind").as_i64(), i64::from(ChartKind::Natal.id()));
        assert_eq!(field("chart_count").as_i64(), charts.len() as i64);
        assert_eq!(field("graha_count").as_i64(), graha_count as i64);
        assert_eq!(field("varga_count").as_i64(), 1, "the one asked for");
        assert_eq!(field("latitude_deg").as_f64(), place.latitude.get());

        // The navamsha, read back: one row per chart and the graha rows
        // charts-outermost then vargas-outermost, which is the ordering
        // a decoder slices by.
        let asked = reader.column("vargas", "varga").expect("the vargas");
        assert_eq!(asked.len(), charts.len());
        assert!(
            asked
                .iter()
                .all(|id| id.as_i64() == i64::from(Varga::D9.id()))
        );
        let signs = reader.column("varga_grahas", "sign").expect("the signs");
        assert_eq!(signs.len(), charts.len() * graha_count);
        for (index, document) in documents.iter().enumerate() {
            let navamsha = &document.vargas[0];
            for (at, placed) in navamsha.grahas.iter().enumerate() {
                assert_eq!(
                    signs[index * graha_count + at].as_i64(),
                    i64::from(placed.at.sign.id()),
                    "chart {index}, graha {at}"
                );
            }
        }

        let instants = reader.column("cast", "instant").expect("the instants");
        let lagnas = reader.column("cast", "lagna_deg").expect("the lagnas");
        assert_eq!(instants.len(), charts.len());
        for (row, chart) in charts.iter().enumerate() {
            assert_eq!(instants[row].as_f64(), chart.instant.get());
            assert_eq!(lagnas[row].as_f64(), chart.lagna_deg);
        }
        assert_ne!(
            lagnas[0].as_f64(),
            lagnas[1].as_f64(),
            "two instants, two lagnas: a constant column would prove nothing"
        );

        // Charts outermost: row `i * graha_count + j` is chart `i`, graha
        // `j`. The second chart's grahas are what a scalar layout would
        // put in the first chart's rows.
        let longitudes = reader
            .column("grahas", "longitude_deg")
            .expect("the grahas");
        assert_eq!(longitudes.len(), charts.len() * graha_count);
        for (i, chart) in charts.iter().enumerate() {
            for (j, graha) in chart.grahas.iter().enumerate() {
                assert_eq!(
                    longitudes[i * graha_count + j].as_f64(),
                    graha.longitude_deg,
                    "chart {i}, graha {j}"
                );
            }
        }

        let madhya = reader.column("houses", "madhya_deg").expect("the houses");
        assert_eq!(madhya.len(), charts.len() * 12);
        for (i, chart) in charts.iter().enumerate() {
            for (j, bhava) in chart.houses.madhya.iter().enumerate() {
                assert_eq!(madhya[i * 12 + j].as_f64(), *bhava, "chart {i}, bhava {j}");
            }
        }

        let sunrise = reader.column("day", "sunrise").expect("the sunrises");
        let vara = reader.column("day", "vara").expect("the varas");
        let ghati = reader.column("timing", "ghati").expect("the ghatis");
        for (row, chart) in charts.iter().enumerate() {
            assert_eq!(sunrise[row].as_f64(), chart.day.day.sunrise.get());
            assert_eq!(vara[row].as_i64(), i64::from(chart.day.day.vara.id()));
            assert_eq!(ghati[row].as_i64(), i64::from(chart.timing.ishtakaal.ghati));
        }

        let model = reader.bytes("model").expect("the model");
        assert_eq!(model, charts[0].day.day.model.as_bytes());
    }

    /// **The drishti, and the ragged layout that carries them.**
    ///
    /// A section of its own because what it proves is its own: two
    /// charts of the same nine grahas hold *different* numbers of
    /// relations — 47 and 40 — which the check that would have enforced
    /// one count for the batch is what found out. A chart's drishti are
    /// a function of where the bodies stand rather than of how many
    /// there are, so the rows are concatenated and a reader prefix-sums
    /// `cast.aspect_count`, which is the panchanga blob's own rule for a
    /// ragged list.
    #[test]
    fn the_drishti_are_ragged_and_the_counts_say_where_each_chart_begins() {
        use teistro_idl::blob::Reader;

        let (charts, place, provenance) = founded();
        let resolved =
            teistro_core::settings::Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
                .expect("the default profile")
                .resolve(&teistro_core::settings::SettingsPatch::default())
                .expect("it resolves");
        let documents: Vec<Document> = charts
            .iter()
            .map(|chart| {
                Document::of(chart.clone()).with_aspects(
                    teistro_aspect::Aspects::of(chart, &resolved.settings).expect("the drishti"),
                )
            })
            .collect();
        let bytes = encoded(&documents, &place, &provenance).expect("it encodes");
        let schema = crate::schemas::charts();
        let reader = Reader::parse(&bytes, &schema).expect("a well-formed blob");

        // **The drishti, and the ragged layout that carries them.** Two
        // charts of the same nine grahas hold different numbers of
        // relations — the check that would have enforced one count for
        // the batch is what found that out — so the rows are
        // concatenated and a reader prefix-sums `cast.aspect_count`.
        let counts = reader.column("cast", "aspect_count").expect("the counts");
        let from = reader.column("aspects", "from").expect("the drishti");
        assert_eq!(counts.len(), charts.len());
        assert!(counts.iter().all(|n| n.as_i64() > 0), "a chart has drishti");
        assert_ne!(
            counts[0].as_i64(),
            counts[1].as_i64(),
            "these two charts differ, which is why the section is ragged"
        );
        let mut at = 0_usize;
        for (index, document) in documents.iter().enumerate() {
            let relations = document.aspects.as_ref().expect("asked for").all();
            assert_eq!(counts[index].as_i64(), relations.len() as i64);
            for (k, drishti) in relations.iter().enumerate() {
                assert_eq!(
                    from[at + k].as_i64(),
                    i64::from(drishti.from.id()),
                    "chart {index}, drishti {k}"
                );
            }
            at += relations.len();
        }
        assert_eq!(from.len(), at, "the rows are exactly the counts");
        assert!(
            !reader.bytes("drishti_table").expect("the table").is_empty(),
            "the table every relation was read under"
        );
    }

    /// A batch of none is a blob, not an error.
    ///
    /// A caller that filtered a list to nothing gets an empty answer
    /// rather than a refusal, which is what lets a binding pass a list
    /// straight through. What the request knows is still written; what
    /// only founding could tell is zero, and `chart_count` says so.

    #[test]
    fn a_batch_of_no_charts_is_still_a_well_formed_blob() {
        use teistro_idl::blob::Reader;

        let (_, place, provenance) = founded();
        let bytes = encoded(&[], &place, &provenance).expect("it encodes");
        let schema = crate::schemas::charts();
        let reader = Reader::parse(&bytes, &schema).expect("a well-formed blob");

        let field = |name: &str| reader.field("summary", name).expect(name);
        assert_eq!(field("kind").as_i64(), i64::from(ChartKind::Natal.id()));
        assert_eq!(field("chart_count").as_i64(), 0, "no charts");
        assert_eq!(field("graha_count").as_i64(), 0, "and so no grahas each");
        assert_eq!(field("varga_count").as_i64(), 0, "and no divisional charts");
        assert_eq!(
            field("latitude_deg").as_f64(),
            place.latitude.get(),
            "but a place"
        );
        assert!(
            reader.column("vargas", "varga").expect("empty").is_empty(),
            "a section nobody asked for is written and empty"
        );
        assert!(
            reader.column("cast", "instant").expect("empty").is_empty(),
            "{:?}",
            reader.column("cast", "instant").expect("empty")
        );
        assert!(
            reader.column("day", "sunrise").expect("empty").is_empty(),
            "{:?}",
            reader.column("day", "sunrise").expect("empty")
        );
        assert_eq!(reader.bytes("model").expect("the model"), b"");
        assert!(
            !reader
                .bytes("provenance_json")
                .expect("the envelope")
                .is_empty(),
            "the settings that founded nothing are still stamped"
        );
    }

    /// A day's state splits into the three scalars a blob carries.
    ///
    /// `DayState::Polar` carries two payload fields, which is what made
    /// the first version of §8's rule — a kind and one value — too
    /// narrow. A normal day leaves both at nought, so a reader that
    /// checks the kind first never looks at them.
    #[test]
    fn a_days_state_splits_into_a_kind_and_its_payload() {
        assert_eq!(
            TsDayState::split(DayState::Normal),
            (TsDayState::Normal, 0, 0)
        );
        for kind in [PolarKind::Day, PolarKind::Night] {
            for policy in PolarDayPolicy::ALL {
                let (state, polar, applied) = TsDayState::split(DayState::Polar {
                    kind,
                    policy: *policy,
                });
                assert_eq!(state, TsDayState::Polar);
                assert_eq!(polar, TsPolarKind::from(kind) as u8);
                assert_eq!(
                    applied,
                    TsPolarDayPolicy::of(*policy)
                        .unwrap_or_else(|| panic!("`{policy}` has no id at the boundary"))
                        as u8
                );
            }
        }
    }

    /// The two chart-layer enums cross exhaustively, so a variant added
    /// to either breaks the build rather than this test; what this holds
    /// is that no two share an id.
    #[test]
    fn the_chart_enums_cross_to_ids_of_their_own() {
        assert_eq!(TsReading::from(Reading::Sandhi) as u8, 0);
        assert_eq!(TsReading::from(Reading::Madhya) as u8, 1);
        assert_eq!(TsDayPart::from(DayPart::Daylight) as u8, 0);
        assert_eq!(TsDayPart::from(DayPart::Night) as u8, 1);
    }

    /// A knob is `non_exhaustive`, so its crossing needs a wildcard; this is
    /// what stops a rule added to `jaimini.brahma` crossing as the verses.
    #[test]
    fn every_brahma_rule_crosses_as_its_own_code() {
        use super::TsBrahmaRule;
        use teistro_core::settings::BrahmaRule;
        let mut codes: Vec<u8> = BrahmaRule::ALL
            .iter()
            .map(|rule| TsBrahmaRule::from(*rule) as u8)
            .collect();
        // Sorted first: `dedup` drops only neighbours, so an unsorted list
        // would pass two members that share a code apart.
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), BrahmaRule::ALL.len(), "{codes:?}");
    }

    /// The Brahma graha's two masks read back as the grahas they hold, the
    /// nodes too, where the planets' byte holds only the seven.
    #[test]
    fn the_nine_grahas_mask_into_sixteen_bits() {
        use super::{graha_bits, graha_mask};
        use teistro::catalogue::Graha;
        let nine = &Graha::ALL[..9];
        assert_eq!(graha_mask(nine), 0x01FF);
        assert_eq!(graha_mask(&[Graha::Ketu, Graha::Sun]), 0x0101);
        assert_eq!(graha_bits(nine), 0x7F);
    }
}
