//! The Shadbala: each graha's six strengths, in virupas
//! (`03-design/shadbala-measured.md`).
//!
//! BPHS ch. 27 names the six — Sthana, Dig, Kaala (Ayana inside it), Cheshta,
//! Naisargika and Drik — for the seven grahas from the Sun to Saturn. The
//! conformance corpus's recording engine computes every one of them, and
//! departs from the chapter at eleven forks; each fork is a setting in
//! [`ShadbalaRules`], BPHS's reading the default and the engine's the other
//! value, under which this module reproduces every recorded component.
//!
//! Two things neither the text nor the engine settles are the same under
//! both readings, and said so: the Drik bala reads the graded whole-sign
//! drishti ([`teistro_aspect::drishti`]), because ch. 26's sphuta drishti is
//! garbled in the translation read (crux C69); and the Cheshta of Mars to
//! Saturn reads the engine's J2000 mean elements, because the chapter gives
//! none (crux C70). The Yuddha bala is not built, and a scheme that counts
//! it is refused.
//!
//! Everything here is arithmetic on values the caller supplies in a
//! [`ShadbalaChart`]; nothing searches an ephemeris.

use serde::{Deserialize, Serialize};
use teistro_aspect::drishti::quarters;
use teistro_core::catalogue::{BalaScheme, Gender, Graha, Nature, Rashi, Relationship, Varga};
use teistro_core::error::Error;
use teistro_core::settings::{
    DigKendras, Drik, KaalaLords, Kranti, MoonCheshta, Naisargika, Nathonnatha, PreDawnNight,
    RequiredRupas, Saptavargaja, Settings, SunAyana,
};
use teistro_state::dignity::{FRIENDLY_HOUSES, compound, natural};

use crate::ashtakavarga::GRAHAS;

/// The seven vargas the Saptavargaja reads, in the order a
/// [`ShadbalaChart`] holds them.
pub const SAPTAVARGAJA_VARGAS: [Varga; 7] = [
    Varga::D1,
    Varga::D2,
    Varga::D3,
    Varga::D7,
    Varga::D9,
    Varga::D12,
    Varga::D30,
];

/// Virupas in a rupa.
pub const VIRUPAS_PER_RUPA: f64 = 60.0;

/// The recording engine's obliquity, which [`Kranti::Ecliptic`] reads.
pub const ENGINE_OBLIQUITY_DEG: f64 = 23.4393;

/// Days elapsed from creation to 1 January 1860 (Burgess's 714,404,108,573
/// less the day itself), so that the count is a Sunday at nought as the
/// Surya Siddhanta's creation was; the corpus confirms it on every recorded
/// weekday the engine resolved correctly (crux C67).
const AHARGANA_1860: i64 = 714_404_108_572;
/// The Julian day number of 1 January 1860.
const JDN_1860: i64 = 2_400_411;
/// The Julian day of J2000, the engine's mean elements' epoch.
const J2000: f64 = 2_451_545.0;

/// BPHS ch. 27 vv. 32 and 33's requirements in rupas, Sun to Saturn.
const BPHS_REQUIRED: [f64; 7] = [6.5, 6.0, 5.0, 7.0, 6.5, 5.5, 5.0];
/// The recording engine's, the Sun's 5.
const ENGINE_REQUIRED: [f64; 7] = [5.0, 6.0, 5.0, 7.0, 6.5, 5.5, 5.0];
/// The natural strength's sevenths of a rupa, Sun to Saturn (v. 14).
const NAISARGIKA_SEVENTHS: [f64; 7] = [7.0, 6.0, 2.0, 3.0, 4.0, 5.0, 1.0];
/// The recording engine's natural strengths, rounded to hundredths.
const NAISARGIKA_HUNDREDTHS: [f64; 7] = [60.0, 51.43, 17.14, 25.71, 34.29, 42.86, 8.57];

/// One graha as the Shadbala reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadbalaGraha {
    /// Its sidereal longitude, degrees.
    pub longitude: f64,
    /// Its tropical longitude, degrees, for its declination.
    pub tropical: f64,
    /// Its ecliptic latitude, degrees, for its declination.
    pub latitude: f64,
    /// The house it stands in, 1 to 12.
    pub house: u8,
}

/// What a Shadbala reads of a chart.
///
/// The day is **the Hindu day** the instant belongs to: from the sunrise at
/// or before it to the next, so a birth before dawn belongs to the previous
/// morning's day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadbalaChart {
    /// The seven grahas, Sun to Saturn.
    pub grahas: [ShadbalaGraha; 7],
    /// Each of [`SAPTAVARGAJA_VARGAS`]' seven signs, the grahas Sun to Saturn.
    pub vargas: [[Rashi; 7]; 7],
    /// The instant, a Julian day (UT).
    pub instant: f64,
    /// The day's sunrise, a Julian day (UT).
    pub sunrise: f64,
    /// The day's sunset.
    pub sunset: f64,
    /// The next sunrise, which ends the day.
    pub next_sunrise: f64,
    /// Whether the instant falls after the local midnight that ends the
    /// day's civil date: a birth before dawn.
    pub after_midnight: bool,
    /// The Julian day number of the day's civil date, for the ahargana.
    pub civil_day: i64,
    /// The day's weekday lord.
    pub weekday_lord: Graha,
    /// The lord of the planetary hour holding the instant.
    pub hora_lord: Graha,
    /// The UT weekday lord of the last Mesha sankranti at or before the
    /// instant, which [`KaalaLords::Sankranti`] reads; `None` gives no graha
    /// the Abda bala under it.
    pub sankranti_lord: Option<Graha>,
    /// The sidereal ascendant, degrees.
    pub ascendant: f64,
    /// The sidereal midheaven, degrees.
    pub midheaven: f64,
    /// The ayanamsha, degrees, for the mean Sun of the seeghra kendra.
    pub ayanamsha: f64,
    /// The date's true obliquity, degrees, which [`Kranti::True`] reads.
    pub obliquity: f64,
}

/// The choices a Shadbala is computed under, one a fork of BPHS ch. 27 and
/// the recording engine (cruxes C64 to C71).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ShadbalaRules {
    /// How the Saptavargaja scores a varga.
    pub saptavargaja: Saptavargaja,
    /// How the Nathonnatha measures the hour.
    pub nathonnatha: Nathonnatha,
    /// Which night a birth before dawn is measured in.
    pub pre_dawn_night: PreDawnNight,
    /// Where the Sun's Ayana is counted.
    pub sun_ayana: SunAyana,
    /// The Moon's Cheshta.
    pub moon_cheshta: MoonCheshta,
    /// The declination the Ayana reads.
    pub kranti: Kranti,
    /// Whose weekdays the Abda and Masa lords are.
    pub kaala_lords: KaalaLords,
    /// The kendras the Dig measures from.
    pub dig: DigKendras,
    /// How the Drik weighs the drishtis received.
    pub drik: Drik,
    /// The natural strengths.
    pub naisargika: Naisargika,
    /// The rupas a graha must reach.
    pub required_rupas: RequiredRupas,
}

impl ShadbalaRules {
    /// BPHS ch. 27's reading at every fork.
    pub const BPHS: ShadbalaRules = ShadbalaRules {
        saptavargaja: Saptavargaja::Compound,
        nathonnatha: Nathonnatha::Midnight,
        pre_dawn_night: PreDawnNight::PreviousEvening,
        sun_ayana: SunAyana::Doubled,
        moon_cheshta: MoonCheshta::Paksha,
        kranti: Kranti::True,
        kaala_lords: KaalaLords::Ahargana,
        dig: DigKendras::Angles,
        drik: Drik::Quarter,
        naisargika: Naisargika::Exact,
        required_rupas: RequiredRupas::Bphs,
    };

    /// The conformance corpus's recording engine's reading at every fork.
    pub const RECORDING_ENGINE: ShadbalaRules = ShadbalaRules {
        saptavargaja: Saptavargaja::Natural,
        nathonnatha: Nathonnatha::Arc,
        pre_dawn_night: PreDawnNight::SameEvening,
        sun_ayana: SunAyana::CheshtaOnly,
        moon_cheshta: MoonCheshta::Elongation,
        kranti: Kranti::Ecliptic,
        kaala_lords: KaalaLords::Sankranti,
        dig: DigKendras::LagnaProjection,
        drik: Drik::Full,
        naisargika: Naisargika::Hundredths,
        required_rupas: RequiredRupas::RecordingEngine,
    };

    /// The rules a context's settings name.
    ///
    /// # Errors
    ///
    /// `UNSUPPORTED` for a bala scheme other than BPHS's six strengths: the
    /// extended scheme counts the Yuddha bala, which is not built.
    pub fn of(settings: &Settings) -> Result<ShadbalaRules, Error> {
        if settings.strength.bala_scheme != BalaScheme::Parashara {
            return Err(Error::unsupported(format!(
                "the {} bala scheme counts the Yuddha bala, which is not built; the SDK computes {}",
                settings.strength.bala_scheme.key(),
                BalaScheme::Parashara.key()
            ))
            .with_field("strength.bala_scheme"));
        }
        Ok(ShadbalaRules {
            saptavargaja: settings.strength.saptavargaja,
            nathonnatha: settings.strength.nathonnatha,
            pre_dawn_night: settings.strength.pre_dawn_night,
            sun_ayana: settings.strength.sun_ayana,
            moon_cheshta: settings.strength.moon_cheshta,
            kranti: settings.strength.kranti,
            kaala_lords: settings.strength.kaala_lords,
            dig: settings.strength.dig,
            drik: settings.strength.drik,
            naisargika: settings.strength.naisargika,
            required_rupas: settings.strength.required_rupas,
        })
    }
}

/// A graha's Sthana bala by component, virupas.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SthanaBala {
    /// From its distance to its debilitation point, 0 to 60.
    pub uchcha: f64,
    /// From its dignity in the seven vargas.
    pub saptavargaja: f64,
    /// From its rasi's and navamsha's parity, 0, 15 or 30.
    pub ojayugma: f64,
    /// From its house: 60, 30 or 15.
    pub kendradi: f64,
    /// From its decanate: 0 or 15.
    pub drekkana: f64,
}

impl SthanaBala {
    /// The five together.
    #[must_use]
    pub fn total(&self) -> f64 {
        self.uchcha + self.saptavargaja + self.ojayugma + self.kendradi + self.drekkana
    }
}

/// A graha's Kaala bala by component, virupas.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct KaalaBala {
    /// From the hour's distance to midnight or noon, 0 to 60.
    pub nathonnatha: f64,
    /// From the Moon's elongation, the Moon's doubled.
    pub paksha: f64,
    /// 60 to the lord of the third of the day or night, and to Jupiter.
    pub tribhaga: f64,
    /// 15 to the year's lord.
    pub abda: f64,
    /// 30 to the month's lord.
    pub masa: f64,
    /// 45 to the weekday's lord.
    pub vara: f64,
    /// 60 to the hour's lord.
    pub hora: f64,
    /// From its declination.
    pub ayana: f64,
}

impl KaalaBala {
    /// The eight together.
    #[must_use]
    pub fn total(&self) -> f64 {
        self.nathonnatha
            + self.paksha
            + self.tribhaga
            + self.vara
            + self.hora
            + self.ayana
            + self.abda
            + self.masa
    }
}

/// One graha's Shadbala.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GrahaShadbala {
    /// Which graha.
    pub graha: Graha,
    /// Positional strength.
    pub sthana: SthanaBala,
    /// Directional strength, 0 to 60.
    pub dig: f64,
    /// Temporal strength.
    pub kaala: KaalaBala,
    /// Motional strength.
    pub cheshta: f64,
    /// Natural strength.
    pub naisargika: f64,
    /// Aspectual strength, which may be negative.
    pub drik: f64,
    /// The six together, virupas.
    pub virupas: f64,
    /// The same in rupas.
    pub rupas: f64,
    /// The rupas it must reach to be strong.
    pub required_rupas: f64,
    /// Whether it reaches them.
    pub strong: bool,
}

/// A chart's Shadbala.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ShadbalaReading {
    /// The choices it was computed under.
    pub rules: ShadbalaRules,
    /// Each graha's, Sun to Saturn.
    pub grahas: Vec<GrahaShadbala>,
}

// ── Shared arithmetic ─────────────────────────────────────────────────────

/// The shorter arc between two longitudes, 0 to 180.
fn apart(a: f64, b: f64) -> f64 {
    let diff = (a.rem_euclid(360.0) - b.rem_euclid(360.0)).abs();
    if diff > 180.0 { 360.0 - diff } else { diff }
}

/// A graha's entry in a row kept Sun to Saturn.
fn seat<T: Copy>(row: &[T; 7], graha: Graha) -> T {
    let [first, ..] = *row;
    GRAHAS
        .iter()
        .zip(row)
        .find_map(|(g, value)| (*g == graha).then_some(*value))
        .unwrap_or(first)
}

fn sign_of(longitude: f64) -> Rashi {
    // The quotient is 0 to 11, which a u16 holds.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude over 30 degrees is 0 to 11"
    )]
    let index = (longitude.rem_euclid(360.0) / 30.0) as u16;
    Rashi::from_id(index.min(11)).unwrap_or(Rashi::Aries)
}

/// The grahas whose Paksha grows with the Moon, and which add to Drik under
/// the chapter's quarters.
const fn waxing_benefic(graha: Graha) -> bool {
    matches!(
        graha,
        Graha::Moon | Graha::Mercury | Graha::Jupiter | Graha::Venus
    )
}

/// The Moon's elongation from the Sun, 0 at new moon and 180 at full.
fn elongation(chart: &ShadbalaChart) -> f64 {
    let [sun, moon, ..] = chart.grahas;
    apart(moon.longitude, sun.longitude)
}

// ── Sthana ────────────────────────────────────────────────────────────────

fn uchcha(graha: Graha, at: &ShadbalaGraha) -> f64 {
    let exaltation = graha.attributes().exaltation.map_or(0.0, |e| {
        f64::from(e.sign as u8) * 30.0 + f64::from(e.degree)
    });
    ((180.0 - apart(at.longitude, exaltation)) / 3.0).max(0.0)
}

fn saptavargaja(graha: Graha, chart: &ShadbalaChart, rule: Saptavargaja) -> f64 {
    let at = seat(&chart.grahas, graha);
    let [rasi, ..] = chart.vargas;
    let degrees = at.longitude.rem_euclid(30.0);
    chart
        .vargas
        .iter()
        .enumerate()
        .map(|(k, row)| {
            let sign = seat(row, graha);
            if rule == Saptavargaja::Natural {
                natural_virupas(graha, sign)
            } else {
                compound_virupas(graha, sign, (k == 0).then_some(degrees), &rasi)
            }
        })
        .sum()
}

/// The recording engine's virupas for a graha in a varga sign.
fn natural_virupas(graha: Graha, sign: Rashi) -> f64 {
    let attributes = graha.attributes();
    if attributes.exaltation.is_some_and(|e| e.sign == sign) {
        45.0
    } else if attributes.debilitation.is_some_and(|e| e.sign == sign) {
        0.0
    } else if attributes.moolatrikona.is_some_and(|m| m.sign == sign)
        || attributes.own.contains(&sign)
    {
        30.0
    } else {
        match natural(graha, sign) {
            Relationship::Friend => 15.0,
            Relationship::Enemy => 3.75,
            _ => 7.5,
        }
    }
}

/// The compound relationship's virupas; `degrees` the graha's degrees in the
/// sign in the rasi chart, where the moolatrikona is read by its span, and
/// `rasi` every graha's rasi sign, for the temporary relationship.
fn compound_virupas(graha: Graha, sign: Rashi, degrees: Option<f64>, rasi: &[Rashi; 7]) -> f64 {
    let attributes = graha.attributes();
    if let Some(span) = attributes.moolatrikona.filter(|m| m.sign == sign) {
        let inside =
            degrees.is_none_or(|d| (f64::from(span.from)..=f64::from(span.to)).contains(&d));
        if inside {
            return 45.0;
        }
    }
    if attributes.own.contains(&sign) {
        return 30.0;
    }
    let lord = sign.attributes().lord;
    let from = seat(rasi, graha) as u8;
    let to = seat(rasi, lord) as u8;
    let distance = (to + 12 - from) % 12 + 1;
    let temporary = if FRIENDLY_HOUSES.contains(&distance) {
        Relationship::Friend
    } else {
        Relationship::Enemy
    };
    match compound(natural(graha, sign), temporary) {
        Relationship::GreatFriend => 22.5,
        Relationship::Friend => 15.0,
        Relationship::Enemy => 3.75,
        Relationship::GreatEnemy => 1.875,
        _ => 7.5,
    }
}

fn ojayugma(graha: Graha, rasi: Rashi, navamsha: Rashi) -> f64 {
    let wants_odd = !matches!(graha, Graha::Moon | Graha::Venus);
    let odd = |sign: Rashi| (sign as u8) % 2 == 0;
    [rasi, navamsha]
        .into_iter()
        .map(|sign| if odd(sign) == wants_odd { 15.0 } else { 0.0 })
        .sum()
}

const fn kendradi(house: u8) -> f64 {
    match house % 3 {
        1 => 60.0,
        2 => 30.0,
        _ => 15.0,
    }
}

fn drekkana(graha: Graha, longitude: f64) -> f64 {
    let degrees = longitude.rem_euclid(30.0);
    let decanate = if degrees < 10.0 {
        Gender::Male
    } else if degrees < 20.0 {
        Gender::Female
    } else {
        Gender::Neutral
    };
    if graha.attributes().descriptors.map(|d| d.gender) == Some(decanate) {
        15.0
    } else {
        0.0
    }
}

// ── Dig ───────────────────────────────────────────────────────────────────

fn dig(graha: Graha, chart: &ShadbalaChart, rule: DigKendras) -> f64 {
    // The house where the graha's direction is full: the Sun and Mars the
    // tenth, Mercury and Jupiter the first, Saturn the seventh, the Moon and
    // Venus the fourth.
    let (house, angle) = match graha {
        Graha::Sun | Graha::Mars => (10.0, chart.midheaven),
        Graha::Mercury | Graha::Jupiter => (1.0, chart.ascendant),
        Graha::Saturn => (7.0, chart.ascendant + 180.0),
        _ => (4.0, chart.midheaven + 180.0),
    };
    let full = if rule == DigKendras::Angles {
        angle
    } else {
        chart.ascendant + (house - 1.0) * 30.0
    };
    apart(seat(&chart.grahas, graha).longitude, full + 180.0) / 3.0
}

// ── Kaala ─────────────────────────────────────────────────────────────────

/// A triangle over an arc: nothing at its ends or outside, 60 at its middle.
fn peak(elapsed: f64, length: f64) -> f64 {
    let half = length / 2.0;
    ((1.0 - (elapsed - half).abs() / half) * 60.0).max(0.0)
}

fn is_daylight(chart: &ShadbalaChart) -> bool {
    chart.instant >= chart.sunrise && chart.instant < chart.sunset
}

/// Whether the engine's reading loses the night a pre-dawn birth falls in.
fn night_lost(chart: &ShadbalaChart, rules: &ShadbalaRules) -> bool {
    chart.after_midnight && rules.pre_dawn_night == PreDawnNight::SameEvening
}

fn nathonnatha(graha: Graha, chart: &ShadbalaChart, rules: &ShadbalaRules) -> f64 {
    let night_graha = matches!(graha, Graha::Moon | Graha::Mars | Graha::Saturn);
    if graha == Graha::Mercury {
        return 60.0;
    }
    let daylight = is_daylight(chart);
    if rules.nathonnatha == Nathonnatha::Arc {
        return match (night_graha, daylight) {
            (false, true) => peak(chart.instant - chart.sunrise, chart.sunset - chart.sunrise),
            (true, false) if !night_lost(chart, rules) => peak(
                chart.instant - chart.sunset,
                chart.next_sunrise - chart.sunset,
            ),
            _ => 0.0,
        };
    }
    // The unnata: the ghatis (60 a day) from midnight, 0 to 30.
    let unnata = if daylight {
        30.0 - (chart.instant - f64::midpoint(chart.sunrise, chart.sunset)).abs() * 60.0
    } else {
        (chart.instant - f64::midpoint(chart.sunset, chart.next_sunrise)).abs() * 60.0
    }
    .clamp(0.0, 30.0);
    let nata = 2.0 * (30.0 - unnata);
    if night_graha { nata } else { 60.0 - nata }
}

fn paksha(graha: Graha, chart: &ShadbalaChart) -> f64 {
    let bright = elongation(chart) / 3.0;
    let value = if waxing_benefic(graha) {
        bright
    } else {
        60.0 - bright
    };
    if graha == Graha::Moon {
        2.0 * value
    } else {
        value
    }
}

fn tribhaga(graha: Graha, chart: &ShadbalaChart, rules: &ShadbalaRules) -> f64 {
    if graha == Graha::Jupiter {
        return 60.0;
    }
    let (start, end, lords) = if is_daylight(chart) {
        (
            chart.sunrise,
            chart.sunset,
            [Graha::Mercury, Graha::Sun, Graha::Saturn],
        )
    } else if night_lost(chart, rules) {
        return 0.0;
    } else {
        (
            chart.sunset,
            chart.next_sunrise,
            [Graha::Moon, Graha::Venus, Graha::Mars],
        )
    };
    let third = ((chart.instant - start) / (end - start) * 3.0)
        .floor()
        .clamp(0.0, 2.0);
    // The third is 0, 1 or 2.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamped to 0 to 2"
    )]
    let lord = lords.get(third as usize);
    if lord == Some(&graha) { 60.0 } else { 0.0 }
}

/// The weekday lord of the ahargana's day `n`, Sunday at nought.
fn weekday_of_count(n: i64) -> Graha {
    const FROM_SUNDAY: [Graha; 7] = [
        Graha::Sun,
        Graha::Moon,
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
    ];
    // rem_euclid of 7 is 0 to 6.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "0 to 6"
    )]
    let index = n.rem_euclid(7) as usize;
    FROM_SUNDAY.get(index).copied().unwrap_or(Graha::Sun)
}

/// The Abda and Masa lords.
fn kaala_lords(chart: &ShadbalaChart, rule: KaalaLords) -> (Option<Graha>, Graha) {
    if rule == KaalaLords::Sankranti {
        let [sun, ..] = chart.grahas;
        return (
            chart.sankranti_lord,
            sign_of(sun.longitude).attributes().lord,
        );
    }
    let n = AHARGANA_1860 + chart.civil_day - JDN_1860;
    (
        Some(weekday_of_count(n.div_euclid(360) * 3)),
        weekday_of_count(n.div_euclid(30) * 2),
    )
}

/// A graha's Ayana bala, before the Sun's is doubled.
fn ayana(graha: Graha, chart: &ShadbalaChart, rule: Kranti) -> f64 {
    let at = seat(&chart.grahas, graha);
    let (obliquity, latitude) = if rule == Kranti::True {
        (chart.obliquity, at.latitude)
    } else {
        (ENGINE_OBLIQUITY_DEG, 0.0)
    };
    let (e, b, l) = (
        obliquity.to_radians(),
        latitude.to_radians(),
        at.tropical.to_radians(),
    );
    let declination = (b.sin() * e.cos() + b.cos() * e.sin() * l.sin())
        .asin()
        .to_degrees();
    let signed = match graha {
        Graha::Mercury => declination.abs(),
        Graha::Sun | Graha::Mars | Graha::Jupiter | Graha::Venus => declination,
        _ => -declination,
    };
    ((obliquity + signed) / (2.0 * obliquity) * 60.0).clamp(0.0, 60.0)
}

// ── Cheshta ───────────────────────────────────────────────────────────────

/// The recording engine's J2000 mean elements: sidereal longitude and daily
/// motion, degrees (crux C70).
const fn mean_elements(graha: Graha) -> Option<(f64, f64)> {
    match graha {
        Graha::Mars => Some((311.293, 0.524_039)),
        Graha::Mercury => Some((226.704, 4.092_339)),
        Graha::Jupiter => Some((10.7163, 0.083_091)),
        Graha::Venus => Some((157.247, 1.602_136)),
        Graha::Saturn => Some((25.1077, 0.033_461)),
        _ => None,
    }
}

fn cheshta(graha: Graha, chart: &ShadbalaChart, rules: &ShadbalaRules) -> f64 {
    match graha {
        Graha::Sun => ayana(graha, chart, rules.kranti),
        Graha::Moon if rules.moon_cheshta == MoonCheshta::Paksha => paksha(graha, chart),
        Graha::Moon => elongation(chart) / 3.0,
        _ => mean_elements(graha).map_or(0.0, |(epoch, motion)| {
            let days = chart.instant - J2000;
            let mean = (epoch + motion * days).rem_euclid(360.0);
            let mut arc = (seat(&chart.grahas, graha).longitude - mean).rem_euclid(360.0);
            if arc > 180.0 {
                arc -= 360.0;
            }
            let madhyama = (mean + arc / 2.0).rem_euclid(360.0);
            let seeghrochcha = if matches!(graha, Graha::Mercury | Graha::Venus) {
                mean
            } else {
                // The mean Sun, sidereal.
                (280.46646 + 0.985_647_4 * days - chart.ayanamsha).rem_euclid(360.0)
            };
            (apart(seeghrochcha, madhyama) / 3.0).clamp(0.0, 60.0)
        }),
    }
}

// ── Drik ──────────────────────────────────────────────────────────────────

fn drik(graha: Graha, chart: &ShadbalaChart, rule: Drik) -> f64 {
    let target = seat(&chart.grahas, graha).house;
    let received = GRAHAS
        .iter()
        .zip(&chart.grahas)
        .filter(|(from, _)| **from != graha)
        .map(|(from, at)| {
            let count = (target + 12 - at.house) % 12 + 1;
            let virupas = f64::from(quarters(*from, count).quarters()) * 15.0;
            (*from, virupas)
        });
    if rule == Drik::Full {
        let sum: f64 = received
            .map(|(from, v)| {
                let benefic =
                    from.attributes().descriptors.map(|d| d.nature) == Some(Nature::Benefic);
                if benefic { v } else { -v }
            })
            .sum();
        return sum.clamp(-60.0, 60.0);
    }
    received
        .map(|(from, v)| {
            let quarter = if waxing_benefic(from) {
                v / 4.0
            } else {
                -v / 4.0
            };
            let full = if matches!(from, Graha::Mercury | Graha::Jupiter) {
                v
            } else {
                0.0
            };
            quarter + full
        })
        .sum()
}

impl ShadbalaReading {
    /// A chart's Shadbala under `rules`.
    #[must_use]
    pub fn of(chart: &ShadbalaChart, rules: ShadbalaRules) -> ShadbalaReading {
        let (year_lord, month_lord) = kaala_lords(chart, rules.kaala_lords);
        let [_, _, _, _, navamsha, ..] = chart.vargas;
        let grahas = GRAHAS
            .iter()
            .zip(chart.grahas)
            .map(|(graha, at)| {
                let graha = *graha;
                let sthana = SthanaBala {
                    uchcha: uchcha(graha, &at),
                    saptavargaja: saptavargaja(graha, chart, rules.saptavargaja),
                    ojayugma: ojayugma(graha, sign_of(at.longitude), seat(&navamsha, graha)),
                    kendradi: kendradi(at.house),
                    drekkana: drekkana(graha, at.longitude),
                };
                let own_ayana = ayana(graha, chart, rules.kranti);
                let kaala = KaalaBala {
                    nathonnatha: nathonnatha(graha, chart, &rules),
                    paksha: paksha(graha, chart),
                    tribhaga: tribhaga(graha, chart, &rules),
                    abda: if year_lord == Some(graha) { 15.0 } else { 0.0 },
                    masa: if month_lord == graha { 30.0 } else { 0.0 },
                    vara: if chart.weekday_lord == graha {
                        45.0
                    } else {
                        0.0
                    },
                    hora: if chart.hora_lord == graha { 60.0 } else { 0.0 },
                    ayana: match (graha, rules.sun_ayana) {
                        (Graha::Sun, SunAyana::Doubled) => 2.0 * own_ayana,
                        (Graha::Sun, _) => 0.0,
                        _ => own_ayana,
                    },
                };
                let naisargika = if rules.naisargika == Naisargika::Exact {
                    seat(&NAISARGIKA_SEVENTHS, graha) * VIRUPAS_PER_RUPA / 7.0
                } else {
                    seat(&NAISARGIKA_HUNDREDTHS, graha)
                };
                let dig = dig(graha, chart, rules.dig);
                let cheshta = cheshta(graha, chart, &rules);
                let drik = drik(graha, chart, rules.drik);
                let virupas = sthana.total() + dig + kaala.total() + cheshta + naisargika + drik;
                let rupas = virupas / VIRUPAS_PER_RUPA;
                let required_rupas = if rules.required_rupas == RequiredRupas::Bphs {
                    seat(&BPHS_REQUIRED, graha)
                } else {
                    seat(&ENGINE_REQUIRED, graha)
                };
                GrahaShadbala {
                    graha,
                    sthana,
                    dig,
                    kaala,
                    cheshta,
                    naisargika,
                    drik,
                    virupas,
                    rupas,
                    required_rupas,
                    strong: rupas >= required_rupas,
                }
            })
            .collect();
        ShadbalaReading { rules, grahas }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::indexing_slicing,
        clippy::float_cmp,
        clippy::unwrap_used,
        reason = "tests index what they built, compare exact values and fail by panicking"
    )]

    use teistro_core::settings::{Profile, SettingsPatch, root};

    use super::*;

    /// A chart at 0° Aries tropical for every graha, everything in the first
    /// house of a Mesha lagna, at noon of a twelve-hour day.
    fn chart() -> ShadbalaChart {
        let at = ShadbalaGraha {
            longitude: 0.0,
            tropical: 0.0,
            latitude: 0.0,
            house: 1,
        };
        ShadbalaChart {
            grahas: [at; 7],
            vargas: [[Rashi::Aries; 7]; 7],
            instant: 2_451_545.0,
            sunrise: 2_451_544.75,
            sunset: 2_451_545.25,
            next_sunrise: 2_451_545.75,
            after_midnight: false,
            civil_day: 2_451_545,
            weekday_lord: Graha::Saturn,
            hora_lord: Graha::Sun,
            sankranti_lord: Some(Graha::Venus),
            ascendant: 0.0,
            midheaven: 270.0,
            ayanamsha: 0.0,
            obliquity: 23.44,
        }
    }

    fn of(chart: &ShadbalaChart, rules: ShadbalaRules, graha: Graha) -> GrahaShadbala {
        ShadbalaReading::of(chart, rules).grahas[slot_of(graha)]
    }

    fn slot_of(graha: Graha) -> usize {
        GRAHAS.iter().position(|g| *g == graha).unwrap()
    }

    #[test]
    fn the_rules_follow_the_settings_and_refuse_an_unbuilt_scheme() {
        assert_eq!(ShadbalaRules::of(&root()).unwrap(), ShadbalaRules::BPHS);
        let engine = Profile::shipped("conformance-baseline")
            .unwrap()
            .resolve(&SettingsPatch::default())
            .unwrap()
            .settings;
        assert_eq!(
            ShadbalaRules::of(&engine).unwrap(),
            ShadbalaRules::RECORDING_ENGINE
        );
        let mut extended = root();
        extended.strength.bala_scheme = BalaScheme::ParasharaExtended;
        let refused = ShadbalaRules::of(&extended).unwrap_err();
        assert_eq!(refused.field(), Some("strength.bala_scheme"));
    }

    #[test]
    fn the_ahargana_s_lords_are_the_chapter_s() {
        // Burgess's day, 1 January 1860, was a Sunday.
        assert_eq!(weekday_of_count(AHARGANA_1860), Graha::Sun);
        // v. 13's worked month: 2176 months completed, times 2 plus 1 over 7
        // leaves 6, a Friday.
        assert_eq!(weekday_of_count(2176 * 2), Graha::Venus);
        let day = ShadbalaChart {
            civil_day: JDN_1860,
            ..chart()
        };
        let (year, month) = kaala_lords(&day, KaalaLords::Ahargana);
        let n = AHARGANA_1860;
        assert_eq!(year, Some(weekday_of_count(n / 360 * 3)));
        assert_eq!(month, weekday_of_count(n / 30 * 2));
        // The engine's reads the sankranti and the Sun's sign.
        assert_eq!(
            kaala_lords(&day, KaalaLords::Sankranti),
            (Some(Graha::Venus), Graha::Mars)
        );
    }

    #[test]
    fn nathonnatha_runs_from_midnight_under_the_text() {
        let rules = ShadbalaRules::BPHS;
        let noon = chart();
        assert_eq!(nathonnatha(Graha::Sun, &noon, &rules), 60.0);
        assert_eq!(nathonnatha(Graha::Moon, &noon, &rules), 0.0);
        assert_eq!(nathonnatha(Graha::Mercury, &noon, &rules), 60.0);
        let midnight = ShadbalaChart {
            instant: 2_451_545.5,
            ..chart()
        };
        assert_eq!(nathonnatha(Graha::Saturn, &midnight, &rules), 60.0);
        assert_eq!(nathonnatha(Graha::Jupiter, &midnight, &rules), 0.0);
        // Six hours from midnight is fifteen ghatis: half strength either way.
        let dawn = ShadbalaChart {
            instant: 2_451_545.75 - 1e-9,
            ..chart()
        };
        assert!((nathonnatha(Graha::Mars, &dawn, &rules) - 30.0).abs() < 1e-5);
    }

    #[test]
    fn the_engine_loses_a_pre_dawn_night_and_the_text_does_not() {
        let pre_dawn = ShadbalaChart {
            instant: 2_451_545.6,
            after_midnight: true,
            ..chart()
        };
        let engine = ShadbalaRules::RECORDING_ENGINE;
        assert_eq!(nathonnatha(Graha::Moon, &pre_dawn, &engine), 0.0);
        assert_eq!(tribhaga(Graha::Mars, &pre_dawn, &engine), 0.0);
        let previous = ShadbalaRules {
            pre_dawn_night: PreDawnNight::PreviousEvening,
            ..engine
        };
        assert!(nathonnatha(Graha::Moon, &pre_dawn, &previous) > 0.0);
        // 0.35 of a half-day night: its last third, Mars's.
        assert_eq!(tribhaga(Graha::Mars, &pre_dawn, &previous), 60.0);
    }

    #[test]
    fn the_luminaries_ayana_and_cheshta_follow_the_chapter() {
        // At the equinox with no latitude every graha's Ayana is 30.
        let equinox = chart();
        for kranti in [Kranti::True, Kranti::Ecliptic] {
            assert!((ayana(Graha::Moon, &equinox, kranti) - 30.0).abs() < 1e-12);
        }
        let text = of(&equinox, ShadbalaRules::BPHS, Graha::Sun);
        assert_eq!(text.kaala.ayana, 2.0 * text.cheshta);
        let engine = of(&equinox, ShadbalaRules::RECORDING_ENGINE, Graha::Sun);
        assert_eq!(engine.kaala.ayana, 0.0);
        // Opposite the Sun: a full Moon, whose Paksha is doubled.
        let mut full = chart();
        full.grahas[1].longitude = 180.0;
        let moon = of(&full, ShadbalaRules::BPHS, Graha::Moon);
        assert_eq!(moon.kaala.paksha, 120.0);
        assert_eq!(moon.cheshta, moon.kaala.paksha);
        let moon = of(&full, ShadbalaRules::RECORDING_ENGINE, Graha::Moon);
        assert_eq!(moon.cheshta, 60.0);
        // A graha's true declination lifts with its latitude.
        let mut north = chart();
        north.grahas[3].latitude = 3.0;
        assert!(ayana(Graha::Mercury, &north, Kranti::True) > 30.0);
        assert!((ayana(Graha::Mercury, &north, Kranti::Ecliptic) - 30.0).abs() < 1e-12);
    }

    #[test]
    fn the_natural_strengths_are_sevenths_or_the_engine_s_hundredths() {
        let text = ShadbalaReading::of(&chart(), ShadbalaRules::BPHS);
        let sum: f64 = text.grahas.iter().map(|g| g.naisargika).sum();
        assert!((sum - 240.0).abs() < 1e-12, "28 sevenths of 60");
        let engine = ShadbalaReading::of(&chart(), ShadbalaRules::RECORDING_ENGINE);
        assert_eq!(engine.grahas[1].naisargika, 51.43);
        assert_eq!(text.grahas[6].required_rupas, 5.0);
        assert_eq!(text.grahas[0].required_rupas, 6.5);
        assert_eq!(engine.grahas[0].required_rupas, 5.0);
    }

    #[test]
    fn dig_is_full_at_the_graha_s_own_angle() {
        let mut chart = chart();
        // The Sun at the midheaven, Mercury at the ascendant.
        chart.grahas[0].longitude = chart.midheaven;
        assert_eq!(dig(Graha::Sun, &chart, DigKendras::Angles), 60.0);
        assert_eq!(dig(Graha::Mercury, &chart, DigKendras::Angles), 60.0);
        // A midheaven 30° from the lagna's tenth is 10 virupas apart.
        chart.midheaven = 300.0;
        chart.grahas[0].longitude = 300.0;
        assert_eq!(dig(Graha::Sun, &chart, DigKendras::Angles), 60.0);
        assert_eq!(dig(Graha::Sun, &chart, DigKendras::LagnaProjection), 50.0);
    }

    #[test]
    fn drik_weighs_quarters_and_mercury_and_jupiter_in_full() {
        let mut chart = chart();
        // Jupiter in the seventh looks fully at the Sun in the first.
        chart.grahas[4].house = 7;
        // Everyone else stands with the Sun and looks at nothing.
        assert_eq!(drik(Graha::Sun, &chart, Drik::Quarter), 15.0 + 60.0);
        assert_eq!(drik(Graha::Sun, &chart, Drik::Full), 60.0);
        // Saturn in the seventh instead takes a quarter away under the text.
        chart.grahas[4].house = 1;
        chart.grahas[6].house = 7;
        assert_eq!(drik(Graha::Sun, &chart, Drik::Quarter), -15.0);
        assert_eq!(drik(Graha::Sun, &chart, Drik::Full), -60.0);
    }

    #[test]
    fn the_saptavargaja_s_compound_reading_reads_the_moolatrikona_by_degree_in_the_rasi() {
        let mut chart = chart();
        // The Sun at 25° Leo, past its moolatrikona: its own sign in the rasi
        // and its moolatrikona by sign in every other varga.
        chart.grahas[0].longitude = 145.0;
        chart.vargas = [[Rashi::Leo; 7]; 7];
        assert_eq!(
            saptavargaja(Graha::Sun, &chart, Saptavargaja::Compound),
            30.0 + 6.0 * 45.0
        );
        assert_eq!(
            saptavargaja(Graha::Sun, &chart, Saptavargaja::Natural),
            7.0 * 30.0
        );
    }

    #[test]
    fn every_graha_s_total_is_its_six_strengths() {
        for rules in [ShadbalaRules::BPHS, ShadbalaRules::RECORDING_ENGINE] {
            for graha in ShadbalaReading::of(&chart(), rules).grahas {
                let six = graha.sthana.total()
                    + graha.dig
                    + graha.kaala.total()
                    + graha.cheshta
                    + graha.naisargika
                    + graha.drik;
                assert_eq!(graha.virupas, six);
                assert_eq!(graha.rupas, six / 60.0);
                assert_eq!(graha.strong, graha.rupas >= graha.required_rupas);
            }
        }
    }
}
