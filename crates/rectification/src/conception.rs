//! The reports a candidate birth time gives beside the purifier
//! (`03-design/rectification.md`, step 4): the pranapada's house as Jha's
//! print judges a birth (`PRANAPADA_HOUSE`), the conception BPHS counts back
//! to (`NISHEKA`), and the birth Moon *Brihat Jataka* IV.21 reads from that
//! conception (`CONCEPTION_MOON`).
//!
//! None of them bars. The first is a judgement of the birth, not of its
//! time; the other two compose two texts no text composes (X10). And they
//! are read at one instant, not over a run, for a measured reason: the
//! conception moves by a day for every degree its arcs move, so its own
//! lagna turns many times while the birth moves by minutes, and a verdict
//! "constant over the run" would be false.
//! [`NishekaCount::days_per_birth_minute`] says how fast. The conception is
//! a chart of its own, so it is read in its own zodiac: [`conception`]
//! founds its sky once its instant is known.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Nakshatra, Rashi};
use teistro_core::error::Error;
use teistro_core::house::{House, house_of, sripati_mid_points};
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::YearLength;

use crate::purifier::{
    GulikaAt, Judge, PranapadaRule, Verdict, gulika_instant, house, navamsha, nth, pranapada_deg,
};
use crate::{EDGE_TOLERANCE_DAYS, Rules, Sky};

/// What the conception reports ask of an ephemeris beyond the purifier's
/// [`Sky`]: a graha's longitude and the midheaven, for Saturn, the lagna's
/// lord and Sripati's 9th bhava.
pub trait ConceptionSky: Sky {
    /// A graha's longitude at an instant, in the chart's own zodiac,
    /// degrees.
    ///
    /// # Errors
    ///
    /// Whatever the implementation cannot answer, unchanged.
    fn graha_deg(&self, graha: Graha, at: JulianDay<Utc>) -> Result<f64, Error>;

    /// The midheaven at an instant, in the chart's own zodiac, degrees.
    ///
    /// # Errors
    ///
    /// Whatever the implementation cannot answer, unchanged.
    fn midheaven_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error>;

    /// Several grahas' longitudes at one instant, in the order asked. By
    /// default one [`ConceptionSky::graha_deg`] each; an implementation
    /// whose ephemeris answers several bodies in one request answers them
    /// so.
    ///
    /// # Errors
    ///
    /// Whatever [`ConceptionSky::graha_deg`] refuses.
    fn grahas_deg(&self, grahas: &[Graha], at: JulianDay<Utc>) -> Result<Vec<f64>, Error> {
        grahas
            .iter()
            .map(|graha| self.graha_deg(*graha, at))
            .collect()
    }

    /// A graha's daily motion at an instant, degrees a day, negative while
    /// it is retrograde. By default the difference of its longitude over
    /// [`SPEED_STEP_DAYS`] either side, across 0° the short way; an
    /// implementation whose ephemeris gives speeds answers them instead.
    ///
    /// # Errors
    ///
    /// Whatever [`ConceptionSky::graha_deg`] refuses.
    fn graha_speed_deg(&self, graha: Graha, at: JulianDay<Utc>) -> Result<f64, Error> {
        let before = self.graha_deg(graha, at.plus_days(-SPEED_STEP_DAYS)?)?;
        let after = self.graha_deg(graha, at.plus_days(SPEED_STEP_DAYS)?)?;
        Ok(((after - before + 540.0).rem_euclid(360.0) - 180.0) / (2.0 * SPEED_STEP_DAYS))
    }
}

/// Half the span [`ConceptionSky::graha_speed_deg`] differences a
/// longitude over by default, days: an hour, short enough that no graha's
/// station falls inside it unnoticed by more than its own hour.
pub const SPEED_STEP_DAYS: f64 = 1.0 / 24.0;

// ── PRANAPADA_HOUSE ────────────────────────────────────────────────────

/// How a house is counted from the lagna to a point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HouseCount {
    /// By sign from the lagna's sign. Jha's worked example forces it: its
    /// pranapada stands 23°09′ past the lagna and is printed in the 2nd.
    #[default]
    Sign,
    /// By Sripati's bhava, the sandhis halfway between the mid-points.
    SripatiBhava,
}

/// How [`pranapada_house`] judges (Jha's print ch. 3 vv. 71–74).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct PranapadaHouseRules {
    /// How the pranapada is reckoned. Jha's v. 72 is the verse's: the
    /// Sun's full longitude, the movable sign of its triplicity, a sign
    /// every fifteen palas.
    pub pranapada: PranapadaRule,
    /// How its house is counted from the lagna.
    pub count: HouseCount,
    /// Whether the 1st is auspicious. The gloss lists 2, 5, 9, 4, 10 and
    /// 11 and not the lagna's own sign, so not by default; a reader who
    /// takes v. 73's *dvikoṇe* as the trine 1, 5, 9 turns it on.
    pub first_auspicious: bool,
}

impl Default for PranapadaHouseRules {
    fn default() -> PranapadaHouseRules {
        PranapadaHouseRules {
            pranapada: PranapadaRule::Verse,
            count: HouseCount::Sign,
            first_auspicious: false,
        }
    }
}

/// The houses from the lagna Jha's gloss calls an auspicious birth.
pub const AUSPICIOUS_HOUSES: [u8; 6] = [2, 4, 5, 9, 10, 11];

/// The pranapada's house from the lagna, and the birth it judges.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PranapadaHouse {
    /// The pranapada, degrees.
    pub pranapada_deg: f64,
    /// The lagna, degrees.
    pub lagna_deg: f64,
    /// Its house from the lagna, one to twelve.
    pub house: u8,
    /// Whether that house is auspicious (Jha's ch. 3 vv. 73–74).
    pub auspicious: bool,
}

impl PranapadaHouse {
    /// The verses it reads.
    #[must_use]
    pub const fn source(&self) -> &'static str {
        "BPHS (Jha 1952) ch. 3 vv. 71–74"
    }
}

/// The pranapada's house at a candidate instant.
///
/// # Errors
///
/// Whatever the [`Sky`] refuses.
pub fn pranapada_house(
    sky: &dyn ConceptionSky,
    at: JulianDay<Utc>,
    rules: PranapadaHouseRules,
) -> Result<PranapadaHouse, Error> {
    let day = sky.day(at)?;
    let hours = sky.ishtakaal_hours(at, &day)?;
    let pranapada = pranapada_deg(rules.pranapada, sky.sun_deg(at)?, hours)?;
    let lagna = sky.ascendant_deg(at)?;
    let counted = match rules.count {
        HouseCount::Sign => house(Rashi::of_longitude(lagna), Rashi::of_longitude(pranapada)),
        HouseCount::SripatiBhava => sripati_house(pranapada, lagna, sky.midheaven_deg(at)?).get(),
    };
    Ok(judged_house(pranapada, lagna, counted, rules))
}

/// A counted house judged, apart from the sky so the printed example can
/// be asserted from its printed figures.
fn judged_house(
    pranapada_deg: f64,
    lagna_deg: f64,
    house: u8,
    rules: PranapadaHouseRules,
) -> PranapadaHouse {
    PranapadaHouse {
        pranapada_deg,
        lagna_deg,
        house,
        auspicious: AUSPICIOUS_HOUSES.contains(&house) || (house == 1 && rules.first_auspicious),
    }
}

/// The Sripati bhava a point stands in: the house whose sandhis, halfway
/// between its mid-point and its neighbours', hold it.
fn sripati_house(point_deg: f64, lagna_deg: f64, midheaven_deg: f64) -> House {
    let mids = sripati_mid_points(lagna_deg, midheaven_deg);
    let mut sandhis = [0.0; 12];
    let befores = mids.iter().cycle().skip(11);
    for ((sandhi, this), before) in sandhis.iter_mut().zip(&mids).zip(befores) {
        *sandhi = (before + (this - before).rem_euclid(360.0) / 2.0).rem_euclid(360.0);
    }
    house_of(point_deg, &sandhis, 0.0)
}

// ── NISHEKA ────────────────────────────────────────────────────────────

/// How long the "month" v. 28 reads a sign as.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NishekaMonth {
    /// Thirty days, so a degree is a day and the arc is the days: Jha's
    /// units, "months, days, ghatis, palas".
    #[default]
    ThirtyDays,
    /// A twelfth of the sidereal year.
    Solar,
    /// A synodic month.
    Synodic,
}

impl NishekaMonth {
    /// Its length, in days.
    #[must_use]
    pub const fn days(self) -> f64 {
        match self {
            NishekaMonth::ThirtyDays => 30.0,
            NishekaMonth::Solar => YearLength::Sidereal.days() / 12.0,
            NishekaMonth::Synodic => 29.530_588_853,
        }
    }
}

/// Which point v. 27's "the lagna to the 9th bhava" measures to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NinthBhava {
    /// Sripati's mid-point of the 9th: Jha's bhava table, and the figure
    /// his worked example uses.
    #[default]
    Sripati,
    /// The start of the 9th sign from the lagna's.
    WholeSign,
    /// Eight signs past the lagna's degree.
    Equal,
}

/// Which point of Saturn v. 27's "where Saturn stands" measures from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SaturnTerm {
    /// Saturn's longitude, as Jha's example takes it.
    #[default]
    Longitude,
    /// The Sripati mid-point of the bhava Saturn stands in, a literal
    /// reading of *yasmin bhāve sthitaḥ* no worked example takes.
    BhavaMadhya,
}

/// How v. 28's "the invisible half" is read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvisibleHalf {
    /// The half-circle ahead of the lagna's degree: Jha's "within the six
    /// signs ahead of the lagna", by longitude.
    #[default]
    ByLongitude,
    /// The 1st to the 6th signs from the lagna's.
    BySign,
}

/// How [`nisheka`] counts back (BPHS ch. 3 vv. 25–29).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct NishekaRules {
    /// The month a sign is read as.
    pub month: NishekaMonth,
    /// Which end of Saturn's eighth Mandi is. Its own setting, not the
    /// purifier's: Jha's gloss to v. 70 takes the start and rejects the
    /// end, and his worked nisheka closes only with it.
    pub mandi_at: NishekaMandi,
    /// The 9th bhava's point.
    pub ninth: NinthBhava,
    /// Saturn's point.
    pub saturn: SaturnTerm,
    /// The invisible half.
    pub invisible_half: InvisibleHalf,
}

/// Which end of Saturn's eighth Mandi is, for the conception: the start by
/// default, as Jha (X5 decides the end for the purifier, after the
/// Subodhini's gloss).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NishekaMandi {
    /// The start of Saturn's eighth (Jha ch. 3 v. 70).
    #[default]
    Start,
    /// Its end (the Subodhini's Gulika).
    End,
}

impl NishekaMandi {
    const fn gulika_at(self) -> GulikaAt {
        match self {
            NishekaMandi::Start => GulikaAt::Start,
            NishekaMandi::End => GulikaAt::End,
        }
    }
}

/// The arc v. 28 reads, written as v. 28 reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MonthsBefore {
    /// Signs, read as months.
    pub months: u32,
    /// Degrees, read as days.
    pub days: u32,
    /// Arc-minutes, read as ghatis.
    pub ghatis: u32,
    /// Arc-seconds, read as palas.
    pub palas: u32,
}

/// v. 27's two arcs, their sum, and the span before birth they give.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NishekaSpan {
    /// From Saturn forward to Mandi, degrees.
    pub saturn_to_mandi_deg: f64,
    /// From the lagna forward to the 9th bhava, degrees.
    pub lagna_to_ninth_deg: f64,
    /// The Moon's degrees elapsed in her sign, added when the lagna's lord
    /// is in the invisible half (v. 28), or none.
    pub moon_added_deg: Option<f64>,
    /// The whole arc, degrees.
    pub arc_deg: f64,
    /// The arc written as months, days, ghatis and palas, to the second.
    pub written: MonthsBefore,
    /// The span before birth, in days, under the month length.
    pub days_before: f64,
}

/// The points v. 27 reads, at the birth.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NishekaPoints {
    /// Mandi, degrees.
    pub mandi_deg: f64,
    /// Saturn's point, degrees.
    pub saturn_deg: f64,
    /// The lagna, degrees.
    pub lagna_deg: f64,
    /// The 9th bhava's point, degrees.
    pub ninth_deg: f64,
    /// The lagna's lord's longitude, degrees.
    pub lagna_lord_deg: f64,
    /// The Moon, degrees.
    pub moon_deg: f64,
}

/// The span v. 27 and v. 28 give from the points.
///
/// ```
/// use teistro_rectification::{NishekaPoints, NishekaRules, nisheka_span};
///
/// // Jha's worked example (p. 36).
/// let dms = |s: f64, d: f64, m: f64, x: f64| s * 30.0 + d + m / 60.0 + x / 3600.0;
/// let span = nisheka_span(
///     &NishekaPoints {
///         mandi_deg: dms(9.0, 29.0, 36.0, 53.0),
///         saturn_deg: dms(7.0, 13.0, 24.0, 27.0),
///         lagna_deg: dms(10.0, 26.0, 28.0, 5.0),
///         ninth_deg: dms(6.0, 29.0, 0.0, 36.0),
///         lagna_lord_deg: dms(7.0, 13.0, 24.0, 27.0),
///         moon_deg: 0.0,
///     },
///     NishekaRules::default(),
/// );
/// assert_eq!((span.written.months, span.written.days), (10, 18));
/// assert_eq!((span.written.ghatis, span.written.palas), (44, 57));
/// assert_eq!(span.moon_added_deg, None);
/// ```
#[must_use]
pub fn nisheka_span(points: &NishekaPoints, rules: NishekaRules) -> NishekaSpan {
    let forward = |from: f64, to: f64| (to - from).rem_euclid(360.0);
    let saturn_to_mandi_deg = forward(points.saturn_deg, points.mandi_deg);
    let lagna_to_ninth_deg = forward(points.lagna_deg, points.ninth_deg);
    let invisible = match rules.invisible_half {
        InvisibleHalf::ByLongitude => forward(points.lagna_deg, points.lagna_lord_deg) < 180.0,
        InvisibleHalf::BySign => {
            house(
                Rashi::of_longitude(points.lagna_deg),
                Rashi::of_longitude(points.lagna_lord_deg),
            ) <= 6
        }
    };
    let moon_added_deg = invisible.then(|| points.moon_deg.rem_euclid(30.0));
    let arc_deg = saturn_to_mandi_deg + lagna_to_ninth_deg + moon_added_deg.unwrap_or(0.0);
    let written = written(arc_deg);
    let whole_months = (arc_deg / 30.0).floor();
    let days_before = whole_months * rules.month.days() + (arc_deg - 30.0 * whole_months);
    NishekaSpan {
        saturn_to_mandi_deg,
        lagna_to_ninth_deg,
        moon_added_deg,
        arc_deg,
        written,
        days_before,
    }
}

/// An arc written in signs, degrees, minutes and seconds, to the nearest
/// second.
fn written(arc_deg: f64) -> MonthsBefore {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a sum of two forward arcs and a sign's degrees is 0 to 750 degrees"
    )]
    let seconds = (arc_deg * 3600.0).round() as u32;
    MonthsBefore {
        months: seconds / 108_000,
        days: seconds / 3600 % 30,
        ghatis: seconds / 60 % 60,
        palas: seconds % 60,
    }
}

/// The 9th bhava's point from the lagna and the midheaven.
#[must_use]
pub fn ninth_bhava_deg(lagna_deg: f64, midheaven_deg: f64, rule: NinthBhava) -> f64 {
    match rule {
        NinthBhava::Sripati => sripati_mid_points(lagna_deg, midheaven_deg)[8],
        NinthBhava::WholeSign => nth(Rashi::of_longitude(lagna_deg), 8).start_deg(),
        NinthBhava::Equal => (lagna_deg + 240.0).rem_euclid(360.0),
    }
}

/// The conception a candidate birth counts back to: the points read at the
/// birth, the span they give, and the instant.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NishekaCount {
    /// The points read at the birth.
    pub points: NishekaPoints,
    /// The span they give.
    pub span: NishekaSpan,
    /// The conception: the birth less the span.
    pub instant: JulianDay<Utc>,
    /// How many days the conception moves when the birth moves a minute
    /// later, measured across the minute: why it is read at an instant.
    pub days_per_birth_minute: f64,
}

/// The conception, and its lagna judged.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Nisheka {
    /// The count back from the birth.
    pub count: NishekaCount,
    /// The conception's lagna, degrees, in the conception chart's own
    /// zodiac.
    pub lagna_deg: f64,
    /// That lagna under the purifier, at the birth's place (v. 29: "purify
    /// it as before").
    pub verdict: Verdict,
}

impl Nisheka {
    /// The verses it reads.
    #[must_use]
    pub const fn source(&self) -> &'static str {
        "BPHS ch. 3 vv. 25–29 (Jha 1952 ch. 4 vv. 25–30)"
    }
}

/// The points v. 27 reads at an instant.
fn read_points(
    sky: &dyn ConceptionSky,
    at: JulianDay<Utc>,
    rules: NishekaRules,
) -> Result<NishekaPoints, Error> {
    let day = sky.day(at)?;
    let mandi_deg = sky.ascendant_deg(gulika_instant(&day, at, rules.mandi_at.gulika_at())?)?;
    let lagna_deg = sky.ascendant_deg(at)?;
    let midheaven = sky.midheaven_deg(at)?;
    let saturn = sky.graha_deg(Graha::Saturn, at)?;
    let saturn_deg = match rules.saturn {
        SaturnTerm::Longitude => saturn,
        SaturnTerm::BhavaMadhya => {
            let bhava = sripati_house(saturn, lagna_deg, midheaven);
            sripati_mid_points(lagna_deg, midheaven)
                .get(usize::from(bhava.get() - 1))
                .copied()
                .unwrap_or(saturn)
        }
    };
    let lord = Rashi::of_longitude(lagna_deg).attributes().lord;
    Ok(NishekaPoints {
        mandi_deg,
        saturn_deg,
        lagna_deg,
        ninth_deg: ninth_bhava_deg(lagna_deg, midheaven, rules.ninth),
        lagna_lord_deg: sky.graha_deg(lord, at)?,
        moon_deg: sky.moon_deg(at)?,
    })
}

/// The conception BPHS counts back to from a candidate birth (vv. 27–28).
///
/// # Errors
///
/// Whatever the [`Sky`] refuses at the birth or a minute after it.
pub fn nisheka_count(
    sky: &dyn ConceptionSky,
    birth: JulianDay<Utc>,
    rules: NishekaRules,
) -> Result<NishekaCount, Error> {
    let points = read_points(sky, birth, rules)?;
    let span = nisheka_span(&points, rules);
    let later = JulianDay::literal(birth.get() + 1.0 / 1440.0);
    let later_span = nisheka_span(&read_points(sky, later, rules)?, rules);
    Ok(NishekaCount {
        points,
        span,
        instant: JulianDay::literal(birth.get() - span.days_before),
        days_per_birth_minute: (later_span.days_before - span.days_before).abs(),
    })
}

/// The conception's lagna judged under the purifier (v. 29), over the sky
/// of the conception chart: its own zodiac, not the birth's.
///
/// # Errors
///
/// What [`Rules::check`] refuses, and whatever the [`Sky`] refuses at the
/// conception.
pub fn nisheka_judged(
    count: NishekaCount,
    conceived: &dyn Sky,
    purifier: &Rules,
) -> Result<Nisheka, Error> {
    purifier.check()?;
    Ok(Nisheka {
        count,
        lagna_deg: conceived.ascendant_deg(count.instant)?,
        verdict: Judge::new(conceived, purifier).verdict(count.instant)?,
    })
}

// ── CONCEPTION_MOON ────────────────────────────────────────────────────

/// How BJ IV.21's count runs from the Moon's dvadashamsha at conception to
/// the sign the Moon holds at birth (X11).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConceptionCount {
    /// From the sign after the dvadashamsha's, as many signs as its
    /// number: Bhattotpala's *sādhvī vyākhyā* after Garga, and Iyer.
    #[default]
    NextAfterDvadashamsha,
    /// The dvadashamsha's own sign, counted from the Moon's: the 1912
    /// main text, after Gargi.
    FromMoonSign,
    /// As many signs from Aries: the view Bhattotpala gives to "some",
    /// after Saravali.
    FromAries,
}

/// What of the conception's rising point BJ IV.21 classes as a day or a
/// night sign.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConceptionRising {
    /// The rising sign: the verse and the 1912 main text.
    #[default]
    Sign,
    /// The rising navamsha: Iyer, the 1912 notes, Bhattotpala's
    /// "division".
    Navamsha,
}

/// What Pisces is, which the prints class two ways.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PiscesIs {
    /// Strong by day or by night, so a birth may be either (1912).
    #[default]
    Either,
    /// A day sign (Iyer's BJ I.10 note).
    Day,
}

/// A day or a night birth, or either.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DayOrNight {
    /// By day.
    Day,
    /// By night.
    Night,
    /// Either.
    Either,
}

/// How [`conception_moon`] reads BJ IV.21.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct ConceptionMoonRules {
    /// The count to the birth Moon's sign.
    pub count: ConceptionCount,
    /// What of the rising point is classed.
    pub rising: ConceptionRising,
    /// Pisces's class.
    pub pisces: PiscesIs,
}

/// The day and night signs of BJ I.10: Aries, Taurus, Gemini, Cancer,
/// Sagittarius and Capricorn are night signs.
#[must_use]
pub const fn day_or_night(sign: Rashi, pisces: PiscesIs) -> DayOrNight {
    match sign {
        Rashi::Aries
        | Rashi::Taurus
        | Rashi::Gemini
        | Rashi::Cancer
        | Rashi::Sagittarius
        | Rashi::Capricorn => DayOrNight::Night,
        Rashi::Pisces => match pisces {
            PiscesIs::Either => DayOrNight::Either,
            PiscesIs::Day => DayOrNight::Day,
        },
        // Leo, Virgo, Libra, Scorpio and Aquarius.
        _ => DayOrNight::Day,
    }
}

/// What BJ IV.21 predicts from the Moon at conception.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MoonCount {
    /// The dvadashamsha the Moon occupies in her sign, one to twelve.
    pub dvadashamsha: u8,
    /// The sign the Moon holds at birth.
    pub sign: Rashi,
    /// The nakshatra Bhattotpala's proportion places her in, under
    /// [`ConceptionCount::NextAfterDvadashamsha`] only.
    pub nakshatra: Option<Nakshatra>,
}

/// BJ IV.21's count from a Moon at conception.
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, Rashi};
/// use teistro_rectification::{ConceptionCount, moon_count};
///
/// // Iyer's example: the middle of the 8th dvadashamsha of Aquarius.
/// let count = moon_count(300.0 + 18.75, ConceptionCount::NextAfterDvadashamsha);
/// assert_eq!((count.dvadashamsha, count.sign), (8, Rashi::Taurus));
/// assert_eq!(count.nakshatra, Some(Nakshatra::Rohini));
/// // The 1912 main text counts from the Moon's sign.
/// assert_eq!(moon_count(318.75, ConceptionCount::FromMoonSign).sign, Rashi::Virgo);
/// ```
#[must_use]
pub fn moon_count(moon_deg: f64, count: ConceptionCount) -> MoonCount {
    let moon = moon_deg.rem_euclid(360.0);
    let sign = Rashi::of_longitude(moon);
    let within = moon - sign.start_deg();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a sign's degrees over two and a half are 0 to 11"
    )]
    let past = ((within / 2.5).floor() as u8).min(11);
    let ordinal = past + 1;
    let birth = match count {
        ConceptionCount::NextAfterDvadashamsha => nth(sign, 2 * ordinal - 1),
        ConceptionCount::FromMoonSign => nth(sign, ordinal - 1),
        ConceptionCount::FromAries => nth(Rashi::Aries, ordinal - 1),
    };
    let nakshatra = (count == ConceptionCount::NextAfterDvadashamsha).then(|| {
        let at = birth.start_deg() + (within - 2.5 * f64::from(past)) * 12.0;
        nakshatra_at(at)
    });
    MoonCount {
        dvadashamsha: ordinal,
        sign: birth,
        nakshatra: nakshatra.flatten(),
    }
}

fn nakshatra_at(longitude_deg: f64) -> Option<Nakshatra> {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a normalised longitude over a nakshatra's width is 0 to 26"
    )]
    let id = (longitude_deg.rem_euclid(360.0) / (360.0 / 27.0)) as u16;
    Nakshatra::from_id(id.min(26))
}

/// BJ IV.21 read at a conception and set against the candidate birth.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ConceptionMoon {
    /// The count from the conception's Moon.
    pub predicted: MoonCount,
    /// The candidate's Moon sign.
    pub moon_sign: Rashi,
    /// The candidate's Moon nakshatra.
    pub moon_nakshatra: Option<Nakshatra>,
    /// Whether the candidate's Moon is in the predicted sign.
    pub sign_agrees: bool,
    /// Whether it is in the predicted nakshatra, where one is predicted.
    pub nakshatra_agrees: Option<bool>,
    /// The conception's rising sign or navamsha.
    pub rising: Rashi,
    /// Its class: a day or a night birth.
    pub predicted_part: DayOrNight,
    /// Whether the candidate is born by day.
    pub born_by_day: bool,
    /// Whether the candidate's day or night is the predicted one.
    pub part_agrees: bool,
    /// How much of the rising sign or navamsha had risen at conception,
    /// by rising time.
    pub risen_fraction: f64,
    /// How much of the candidate's day or night had passed at birth.
    pub elapsed_fraction: f64,
}

impl ConceptionMoon {
    /// The verse it reads.
    #[must_use]
    pub const fn source(&self) -> &'static str {
        "Brihat Jataka IV.21, with Bhattotpala"
    }
}

/// BJ IV.21 read at a conception against a candidate birth: the
/// conception's Moon and rising point over `conceived`, the conception
/// chart's sky, and the candidate's Moon and day over `sky`. The verse's
/// fraction is reported and not weighed: no print gives a tolerance.
///
/// # Errors
///
/// Whatever the [`Sky`] refuses, at the birth or the conception.
pub fn conception_moon(
    sky: &dyn Sky,
    conceived: &dyn Sky,
    birth: JulianDay<Utc>,
    conception: JulianDay<Utc>,
    rules: ConceptionMoonRules,
) -> Result<ConceptionMoon, Error> {
    let predicted = moon_count(conceived.moon_deg(conception)?, rules.count);
    let moon = sky.moon_deg(birth)?;
    let moon_sign = Rashi::of_longitude(moon);
    let moon_nakshatra = nakshatra_at(moon);
    let rising_deg = conceived.ascendant_deg(conception)?;
    let rising = match rules.rising {
        ConceptionRising::Sign => Rashi::of_longitude(rising_deg),
        ConceptionRising::Navamsha => navamsha(rising_deg)?,
    };
    let predicted_part = day_or_night(rising, rules.pisces);
    let day = sky.day(birth)?;
    let (arc, born_by_day) = day.arc(birth)?;
    let part_agrees = match predicted_part {
        DayOrNight::Either => true,
        DayOrNight::Day => born_by_day,
        DayOrNight::Night => !born_by_day,
    };
    let width = match rules.rising {
        ConceptionRising::Sign => 30.0,
        ConceptionRising::Navamsha => 30.0 / 9.0,
    };
    Ok(ConceptionMoon {
        predicted,
        moon_sign,
        moon_nakshatra,
        sign_agrees: moon_sign == predicted.sign,
        nakshatra_agrees: predicted.nakshatra.map(|n| Some(n) == moon_nakshatra),
        rising,
        predicted_part,
        born_by_day,
        part_agrees,
        risen_fraction: risen_fraction(conceived, conception, width)?,
        elapsed_fraction: arc.fraction_at(birth),
    })
}

/// How much of the rising unit (a sign or a navamsha, `width` degrees)
/// had risen at an instant, by the time it takes to rise: its entry and
/// exit found by bisection on the lagna.
fn risen_fraction(sky: &dyn Sky, at: JulianDay<Utc>, width: f64) -> Result<f64, Error> {
    let unit = |t: f64| -> Result<i64, Error> {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a longitude over a unit's width is 0 to 107"
        )]
        let index =
            (sky.ascendant_deg(JulianDay::literal(t))?.rem_euclid(360.0) / width).floor() as i64;
        Ok(index)
    };
    let here = unit(at.get())?;
    let edge = |direction: f64| -> Result<f64, Error> {
        // A step well inside the briefest unit, and a day's bound.
        let step = 2.0 / 1440.0;
        let mut inside = at.get();
        let mut outside = inside;
        for _ in 0..720 {
            outside = inside + direction * step;
            if unit(outside)? != here {
                break;
            }
            inside = outside;
        }
        while (outside - inside).abs() > EDGE_TOLERANCE_DAYS {
            let mid = f64::midpoint(inside, outside);
            if unit(mid)? == here {
                inside = mid;
            } else {
                outside = mid;
            }
        }
        Ok(inside)
    };
    let (entered, leaves) = (edge(-1.0)?, edge(1.0)?);
    let span = leaves - entered;
    Ok(if span > 0.0 {
        (at.get() - entered) / span
    } else {
        0.0
    })
}

// ── all three at once ──────────────────────────────────────────────────

/// What [`conception`] reads under: the purifier's rules, which judge the
/// conception's lagna, and each report's own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct ConceptionRules {
    /// The purifier's rules (v. 29: the conception's lagna is purified
    /// "as before").
    pub purifier: Rules,
    /// `PRANAPADA_HOUSE`'s.
    pub pranapada_house: PranapadaHouseRules,
    /// `NISHEKA`'s.
    pub nisheka: NishekaRules,
    /// `CONCEPTION_MOON`'s.
    pub moon: ConceptionMoonRules,
}

/// The three reports a candidate birth time gives beside the purifier.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Conception {
    /// The candidate.
    pub birth: JulianDay<Utc>,
    /// The pranapada's house, as Jha's print judges the birth.
    pub pranapada_house: PranapadaHouse,
    /// The conception BPHS counts back to.
    pub nisheka: Nisheka,
    /// BJ IV.21 read at that conception against the candidate.
    pub moon: ConceptionMoon,
}

/// The three reports at a candidate birth. `conceived` founds the sky
/// of the conception chart once its instant is known, so the conception
/// is read in its own zodiac; a sky valid at every instant answers with
/// itself.
///
/// # Errors
///
/// What [`Rules::check`] refuses, what `conceived` refuses, and whatever
/// either sky refuses.
pub fn conception<S: Sky>(
    sky: &dyn ConceptionSky,
    birth: JulianDay<Utc>,
    rules: &ConceptionRules,
    conceived: impl FnOnce(JulianDay<Utc>) -> Result<S, Error>,
) -> Result<Conception, Error> {
    rules.purifier.check()?;
    let count = nisheka_count(sky, birth, rules.nisheka)?;
    let at_conception = conceived(count.instant)?;
    Ok(Conception {
        birth,
        pranapada_house: pranapada_house(sky, birth, rules.pranapada_house)?,
        moon: conception_moon(sky, &at_conception, birth, count.instant, rules.moon)?,
        nisheka: nisheka_judged(count, &at_conception, &rules.purifier)?,
    })
}

#[cfg(test)]
pub(crate) mod tests_support {
    use super::{PranapadaHouse, PranapadaHouseRules, Rashi, house, judged_house};

    /// A pranapada judged by sign from printed figures.
    pub(crate) fn judged(
        pranapada_deg: f64,
        lagna_deg: f64,
        rules: PranapadaHouseRules,
    ) -> PranapadaHouse {
        let counted = house(
            Rashi::of_longitude(lagna_deg),
            Rashi::of_longitude(pranapada_deg),
        );
        judged_house(pranapada_deg, lagna_deg, counted, rules)
    }
}
