//! The purifier: BPHS ch. 2 vv. 67–78 in the Subodhini prints
//! (`03-design/rectification.md`, "The purifiers").
//!
//! A human birth's lagna stands in the sign of a purifier or in a trine of
//! it (v. 77). The purifiers are the pranapada (vv. 71–74), Gulika
//! (vv. 67–70) and the Moon (v. 75), any one of them sufficing (v. 75's
//! *vā*); v. 76 adds Gulika's 7th, its navamsha and that navamsha's 7th.
//! Every clause is reported, held or not, so a caller who reads v. 76 as a
//! precedence can apply it (X7).

use std::cell::Cell;

use serde::{Deserialize, Serialize};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Rashi, Varga};
use teistro_core::error::Error;
use teistro_core::quantity::{Degrees, JulianDay, Utc};
use teistro_vargas::Scheme;

use crate::{Day, Rules, Sky};

/// What may purify a lagna.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Purifier {
    /// The pranapada, a sign every fifteen palas from sunrise (vv. 71–74).
    Pranapada,
    /// Gulika, the lagna at Saturn's eighth of the day or night (vv. 67–70).
    Gulika,
    /// The Moon (v. 75).
    Moon,
}

/// Which point of a purifier a clause reads: the purifier itself, or one
/// of the three v. 76 attaches to Gulika.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Reference {
    /// The purifier's own sign.
    Itself,
    /// The sign seventh from it (v. 76, Gulika only).
    Seventh,
    /// Its navamsha (v. 76, Gulika only).
    Navamsha,
    /// The sign seventh from its navamsha (v. 76, Gulika only).
    NavamshaSeventh,
}

/// The species vv. 77–78 assign the twelve houses to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Native {
    /// A human: the sign and its trines, houses 1, 5 and 9 (v. 77).
    #[default]
    Human,
    /// A beast: houses 2, 6 and 10 (v. 77).
    Beast,
    /// A bird: houses 3, 7 and 11 (v. 78).
    Bird,
    /// A creeping or water creature: houses 4, 8 and 12 (v. 78).
    Creeper,
}

impl Native {
    /// The houses, counted from the purifier to the lagna, that purify
    /// this native's birth.
    #[must_use]
    pub const fn houses(self) -> [u8; 3] {
        match self {
            Native::Human => [1, 5, 9],
            Native::Beast => [2, 6, 10],
            Native::Bird => [3, 7, 11],
            Native::Creeper => [4, 8, 12],
        }
    }
}

/// Whether a lagna no purifier holds is removed or only weighed (X8).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PurifyAs {
    /// Removed: v. 75 calls it "the birth of a plant".
    #[default]
    Bar,
    /// Kept, its verdict saying it is impure.
    Weight,
}

/// How the pranapada is reckoned (X2, X3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PranapadaRule {
    /// The verse: the Sun's longitude, shifted by v. 74 to itself, its
    /// 9th or its 5th for a movable, fixed or dual sign, then a sign
    /// every fifteen palas since sunrise, 300° an hour.
    #[default]
    Verse,
    /// The reading that reproduces the gloss's printed answer (p. 13):
    /// the signs taken mod 15 and counted inclusively from the movable
    /// sign of the Sun's triplicity, two degrees for each left-over pala.
    PrintedExample,
    /// The SDK's `points::lagna::pranapada`, settled against the
    /// conformance corpus at 60° an hour.
    SdkPoint,
}

/// Which end of Saturn's eighth is Gulika (X5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GulikaAt {
    /// Its end, where the gloss's multipliers put it (16;37 ghatis in its
    /// worked case).
    #[default]
    End,
    /// Its start, the SDK's `Gulika` point.
    Start,
}

/// When v. 76's extension of Gulika (its 7th, its navamsha and that
/// navamsha's 7th) counts toward a verdict (X7).
///
/// v. 76 opens "when the two are weak": the extension is the verse's
/// fallback. No print defines weak, so the default reads it as "neither
/// purifies", which keeps exactly the instants [`Always`](Self::Always)
/// keeps (an instant either of the two holds is pure already) and differs
/// only in what the verdict says counted. That reading is the SDK's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GulikaExtension {
    /// Counted when neither the pranapada nor the Moon holds.
    #[default]
    WhenTwoFail,
    /// Counted at every instant, without precedence.
    Always,
    /// Not judged: Gulika purifies by its own sign alone.
    Never,
}

/// One test of the lagna against one point of one purifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Clause {
    /// The purifier read.
    pub purifier: Purifier,
    /// Which of its points.
    pub reference: Reference,
    /// The sign that point stands in.
    pub sign: Rashi,
    /// The lagna's sign.
    pub lagna: Rashi,
    /// The lagna's house counted from that sign, one to twelve.
    pub house: u8,
    /// Whether the house is one that purifies this native.
    pub held: bool,
    /// Whether the clause counts toward the verdict: false only for v.
    /// 76's extension while the pranapada or the Moon holds, under
    /// [`GulikaExtension::WhenTwoFail`].
    pub counted: bool,
}

impl Clause {
    /// The verses the clause reads, in the Subodhini prints' numbering.
    #[must_use]
    pub const fn source(&self) -> &'static str {
        match (self.purifier, self.reference) {
            (Purifier::Pranapada, _) => "BPHS ch. 2 vv. 71–75, 77",
            (Purifier::Gulika, Reference::Itself) => "BPHS ch. 2 vv. 67–70, 75, 77",
            (Purifier::Gulika, _) => "BPHS ch. 2 vv. 76–77",
            (Purifier::Moon, _) => "BPHS ch. 2 vv. 75, 77",
        }
    }
}

/// What the purifier finds at an instant: every clause, and whether any
/// held.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Verdict {
    /// Every clause judged, in the order pranapada, Gulika, Moon.
    pub clauses: Vec<Clause>,
    /// Whether at least one counted clause held (v. 75).
    pub pure: bool,
}

impl Verdict {
    /// The clauses that held, counted or not.
    pub fn held(&self) -> impl Iterator<Item = &Clause> {
        self.clauses.iter().filter(|clause| clause.held)
    }

    /// The clauses that held and counted: what made the instant pure.
    pub fn purified_by(&self) -> impl Iterator<Item = &Clause> {
        self.held().filter(|clause| clause.counted)
    }
}

/// The verse's count for a time since sunrise, figure for figure as the
/// 1899 print works it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PranapadaWorking {
    /// Whole signs: a sign for every fifteen palas.
    pub signs: u32,
    /// The palas left over.
    pub palas_left: u32,
    /// Those palas doubled, as degrees.
    pub degrees: u32,
    /// The signs mod 12, as the verse takes them.
    pub mod_twelve: u32,
    /// The signs mod 15, as the gloss's printed answer needs them.
    pub mod_fifteen: u32,
}

/// The verse's count for a time since sunrise in ghatis and palas.
///
/// ```
/// use teistro_rectification::pranapada_working;
///
/// // The gloss's example: 6;17 ghatis after sunrise.
/// let working = pranapada_working(6, 17);
/// assert_eq!((working.signs, working.palas_left, working.degrees), (25, 2, 4));
/// assert_eq!((working.mod_twelve, working.mod_fifteen), (1, 10));
/// ```
#[must_use]
pub const fn pranapada_working(ghatis: u32, palas: u32) -> PranapadaWorking {
    let total = ghatis.saturating_mul(PALAS_PER_GHATI).saturating_add(palas);
    let signs = total / PALAS_PER_SIGN;
    let palas_left = total % PALAS_PER_SIGN;
    PranapadaWorking {
        signs,
        palas_left,
        degrees: palas_left * 2,
        mod_twelve: signs % 12,
        mod_fifteen: signs % 15,
    }
}

/// Palas in a ghati.
const PALAS_PER_GHATI: u32 = 60;

/// Palas the pranapada spends in a sign (v. 71).
const PALAS_PER_SIGN: u32 = 15;

/// The verse's rate: a sign every fifteen palas, six minutes.
const VERSE_DEG_PER_HOUR: f64 = 300.0;

/// The pranapada's longitude under a rule, degrees in `[0, 360)`.
///
/// ```
/// use teistro_rectification::{PranapadaRule, pranapada_deg};
///
/// // The gloss's example: the Sun at 2s 4°28′1″, 6;17 ghatis after sunrise.
/// let sun = 64.0 + 28.0 / 60.0 + 1.0 / 3600.0;
/// let hours = (6.0 * 60.0 + 17.0) * 24.0 / 3600.0;
/// // The verse gives Scorpio 8°28′.
/// let verse = pranapada_deg(PranapadaRule::Verse, sun, hours)?;
/// assert!((verse - (210.0 + 8.0 + 28.0 / 60.0)).abs() < 1e-3);
/// // The printed reading gives the print's 3s 4°.
/// let printed = pranapada_deg(PranapadaRule::PrintedExample, sun, hours)?;
/// assert!((printed - 94.0).abs() < 1e-9);
/// # Ok::<(), teistro_core::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for a longitude or an elapsed time that is not finite,
/// naming the field, and `OUT_OF_RANGE` for an elapsed time outside 0 to
/// [`crate::LONGEST_HOURS`].
pub fn pranapada_deg(
    rule: PranapadaRule,
    sun_deg: f64,
    hours_since_sunrise: f64,
) -> Result<f64, Error> {
    if rule == PranapadaRule::SdkPoint {
        return Ok(teistro_points::lagna::pranapada(sun_deg, hours_since_sunrise)?.longitude_deg);
    }
    if !sun_deg.is_finite() {
        return Err(Error::invalid_arg("the Sun's longitude must be finite")
            .with_field("rectification.sun"));
    }
    if !hours_since_sunrise.is_finite()
        || !(0.0..=crate::LONGEST_HOURS).contains(&hours_since_sunrise)
    {
        return Err(Error::new(
            teistro_core::error::Status::OutOfRange,
            format!(
                "the pranapada counts 0 to {} hours from sunrise, not {hours_since_sunrise}",
                crate::LONGEST_HOURS
            ),
        )
        .with_field("rectification.hoursSinceSunrise"));
    }
    let travelled = VERSE_DEG_PER_HOUR * hours_since_sunrise;
    let sun = Rashi::of_longitude(sun_deg);
    let longitude = match rule {
        PranapadaRule::Verse => sun_deg + shift_deg(sun) + travelled,
        // Signs mod 15, counted inclusively: the fifteenth sign lands on
        // the movable sign itself, so the count starts one sign behind it.
        _ => movable_of_triplicity(sun).start_deg() + travelled.rem_euclid(450.0) - 30.0,
    };
    Ok(longitude.rem_euclid(360.0))
}

/// A sign's place in the cycle movable, fixed, dual: 0, 1 or 2. The
/// zodiac runs that cycle from Aries, so the place is the sign's index
/// mod 3.
fn modality_place(sign: Rashi) -> u8 {
    u8::try_from(index(sign) % 3).unwrap_or_default()
}

/// What v. 74 adds for the Sun's sign: nothing for a movable sign, its
/// 9th for a fixed one, its 5th for a dual one.
fn shift_deg(sun: Rashi) -> f64 {
    match modality_place(sun) {
        0 => 0.0,
        1 => 240.0,
        _ => 120.0,
    }
}

/// The movable sign of a sign's triplicity: Aries for Leo, Libra for
/// Gemini, four signs back for each step past movable.
fn movable_of_triplicity(sign: Rashi) -> Rashi {
    Rashi::of_longitude(sign.start_deg() - 120.0 * f64::from(modality_place(sign)))
}

/// Gulika's instant for the arc holding an instant: the end or the start
/// of Saturn's eighth.
///
/// ```
/// use teistro_core::catalogue::Vara;
/// use teistro_core::quantity::JulianDay;
/// use teistro_rectification::{Day, GulikaAt, gulika_instant};
///
/// // The gloss's case: a Wednesday of 33;14 ghatis of daylight.
/// let ghati = 1.0 / 60.0;
/// let day = Day::literal(0.0, (33.0 + 14.0 / 60.0) * ghati, 60.0 * ghati, Vara::Budhavara);
/// let at = gulika_instant(&day, JulianDay::literal(0.1), GulikaAt::End)?;
/// // 16;37 ghatis after sunrise, to the nearest pala.
/// assert!((at.get() / ghati - (16.0 + 37.0 / 60.0)).abs() < 1.0 / 120.0);
/// # Ok::<(), teistro_core::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for a malformed day, and `UNSUPPORTED` for an arc with no
/// length (a polar day or night has no Gulika).
pub fn gulika_instant(
    day: &Day,
    at: JulianDay<Utc>,
    which: GulikaAt,
) -> Result<JulianDay<Utc>, Error> {
    let (arc, is_day) = day.arc(at)?;
    let portion = teistro_points::eighth::saturns(arc, day.vara, is_day)?;
    Ok(match which {
        GulikaAt::End => portion.at.to,
        GulikaAt::Start => portion.at.from,
    })
}

/// A sign so many signs on.
pub(crate) fn nth(sign: Rashi, on: u8) -> Rashi {
    Rashi::of_longitude(sign.start_deg() + 30.0 * f64::from(on))
}

/// A sign's index from Aries, 0 to 11.
pub(crate) fn index(sign: Rashi) -> usize {
    Rashi::ALL
        .iter()
        .position(|one| *one == sign)
        .unwrap_or_default()
}

/// The house the lagna stands in counted from a sign, one to twelve.
pub(crate) fn house(from: Rashi, lagna: Rashi) -> u8 {
    u8::try_from((index(lagna) + 12 - index(from)) % 12 + 1).unwrap_or(1)
}

/// The navamsha of a longitude.
pub(crate) fn navamsha(longitude_deg: f64) -> Result<Rashi, Error> {
    let at = Nas::from_degrees(Degrees::try_new(longitude_deg.rem_euclid(360.0))?);
    Ok(teistro_vargas::sign(&Scheme::of(Varga::D9), at))
}

/// Judges instants against the purifier, holding the day and Gulika it
/// last read: both change only at a sunrise or a sunset.
pub(crate) struct Judge<'a> {
    sky: &'a dyn Sky,
    rules: &'a Rules,
    day: Cell<Option<Day>>,
    /// The arc Gulika was read for, by its start, and Gulika's longitude.
    gulika: Cell<Option<(f64, f64)>>,
}

impl<'a> Judge<'a> {
    pub(crate) fn new(sky: &'a dyn Sky, rules: &'a Rules) -> Judge<'a> {
        Judge {
            sky,
            rules,
            day: Cell::new(None),
            gulika: Cell::new(None),
        }
    }

    /// The day an instant belongs to.
    pub(crate) fn day(&self, at: JulianDay<Utc>) -> Result<Day, Error> {
        if let Some(day) = self.day.get().filter(|day| day.holds(at)) {
            return Ok(day);
        }
        let day = self.sky.day(at)?;
        if !day.holds(at) {
            return Err(Error::invalid_arg(format!(
                "the sky answered a day from {} to {} for the instant {}, which it does not hold",
                day.sunrise.get(),
                day.next_sunrise.get(),
                at.get()
            ))
            .with_field("rectification.day"));
        }
        self.day.set(Some(day));
        Ok(day)
    }

    /// Gulika's longitude for the arc holding an instant.
    fn gulika_deg(&self, day: &Day, at: JulianDay<Utc>) -> Result<f64, Error> {
        let (arc, _) = day.arc(at)?;
        if let Some((from, deg)) = self.gulika.get()
            && from.to_bits() == arc.from.get().to_bits()
        {
            return Ok(deg);
        }
        let deg = self
            .sky
            .ascendant_deg(gulika_instant(day, at, self.rules.gulika_at)?)?;
        self.gulika.set(Some((arc.from.get(), deg)));
        Ok(deg)
    }

    /// The verdict at an instant.
    pub(crate) fn verdict(&self, at: JulianDay<Utc>) -> Result<Verdict, Error> {
        let day = self.day(at)?;
        let lagna = Rashi::of_longitude(self.sky.ascendant_deg(at)?);
        let houses = self.rules.native.houses();
        let mut clauses = Vec::with_capacity(6);
        let mut judge = |purifier, reference, sign: Rashi| {
            let house = house(sign, lagna);
            clauses.push(Clause {
                purifier,
                reference,
                sign,
                lagna,
                house,
                held: houses.contains(&house),
                counted: true,
            });
        };
        if self.rules.pranapada {
            let hours = self.sky.ishtakaal_hours(at, &day)?;
            let deg = pranapada_deg(self.rules.pranapada_rule, self.sky.sun_deg(at)?, hours)?;
            judge(
                Purifier::Pranapada,
                Reference::Itself,
                Rashi::of_longitude(deg),
            );
        }
        if self.rules.gulika {
            let deg = self.gulika_deg(&day, at)?;
            let sign = Rashi::of_longitude(deg);
            judge(Purifier::Gulika, Reference::Itself, sign);
            if self.rules.gulika_extension != GulikaExtension::Never {
                let amsha = navamsha(deg)?;
                judge(Purifier::Gulika, Reference::Seventh, nth(sign, 6));
                judge(Purifier::Gulika, Reference::Navamsha, amsha);
                judge(Purifier::Gulika, Reference::NavamshaSeventh, nth(amsha, 6));
            }
        }
        if self.rules.moon {
            let sign = Rashi::of_longitude(self.sky.moon_deg(at)?);
            judge(Purifier::Moon, Reference::Itself, sign);
        }
        if self.rules.gulika_extension == GulikaExtension::WhenTwoFail {
            let two_hold = clauses
                .iter()
                .any(|c| c.held && c.purifier != Purifier::Gulika);
            for clause in &mut clauses {
                if clause.reference != Reference::Itself {
                    clause.counted = !two_hold;
                }
            }
        }
        let pure = clauses.iter().any(|clause| clause.held && clause.counted);
        Ok(Verdict { clauses, pure })
    }
}
