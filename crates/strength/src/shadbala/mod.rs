//! The Shadbala: each graha's six strengths, in virupas
//! (`03-design/shadbala-measured.md`).
//!
//! BPHS ch. 27 names the six — Sthana, Dig, Kaala (Ayana and Yuddha inside
//! it), Cheshta, Naisargika and Drik — for the seven grahas from the Sun to
//! Saturn. Three readings of them are published or recorded, and they part at
//! fifteen forks, each a setting in [`ShadbalaRules`]:
//!
//! - [`ShadbalaRules::BPHS`], the chapter as translated, the default;
//! - [`ShadbalaRules::SRIPATI`], Sripati's as B.V. Raman works it on his
//!   Standard Horoscope, the one reading every number of which can be checked;
//! - [`ShadbalaRules::RECORDING_ENGINE`], the conformance corpus's engine,
//!   which this module reproduces on every recorded component.
//!
//! Each strength has its own module — [`sthana`], [`dig`], [`kaala`],
//! [`cheshta`], [`drik`] and [`yuddha`] — and this one composes them.
//! Everything here is arithmetic on values the caller supplies in a
//! [`ShadbalaChart`]; nothing searches an ephemeris.

pub mod cheshta;
pub mod dig;
pub mod drik;
pub mod ishta;
pub mod kaala;
pub mod sthana;
pub mod yuddha;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{BalaScheme, Graha, Rashi, Varga};
use teistro_core::error::Error;
use teistro_core::settings::{
    Benefics, Cheshta, DigKendras, Drekkana, Drik, IshtaKashta, KaalaLords, Kranti,
    LuminaryCheshta, Naisargika, Nathonnatha, PreDawnNight, RequiredRupas, Saptavargaja, Settings,
    SunAyana, Yuddha,
};

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

/// BPHS ch. 27 vv. 32 and 33's requirements in rupas, Sun to Saturn.
const BPHS_REQUIRED: [f64; 7] = [6.5, 6.0, 5.0, 7.0, 6.5, 5.5, 5.0];
/// Sripati's, as B.V. Raman gives them (Art. 122), the Sun's 5.
const SRIPATI_REQUIRED: [f64; 7] = [5.0, 6.0, 5.0, 7.0, 6.5, 5.5, 5.0];
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
    /// Rahu's sidereal longitude, degrees, for Mercury's association under
    /// [`Benefics::Conditional`]; `None` leaves the nodes out of it.
    pub rahu: Option<f64>,
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
    /// The ayanamsha, degrees, for the mean Sun of the recording engine's
    /// seeghra kendra.
    pub ayanamsha: f64,
    /// The date's true obliquity, degrees, which [`Kranti::True`] reads.
    pub obliquity: f64,
}

/// The choices a Shadbala is computed under, each a fork between BPHS ch. 27,
/// Sripati and the recording engine (cruxes C64 to C72).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ShadbalaRules {
    /// How the Saptavargaja scores a varga.
    pub saptavargaja: Saptavargaja,
    /// Which decanate gives each gender its Drekkana bala.
    pub drekkana: Drekkana,
    /// The kendras the Dig measures from.
    pub dig: DigKendras,
    /// How the Nathonnatha measures the hour.
    pub nathonnatha: Nathonnatha,
    /// Which night a birth before dawn is measured in.
    pub pre_dawn_night: PreDawnNight,
    /// Which grahas are benefics for Paksha and Drik.
    pub benefics: Benefics,
    /// Whose weekdays the Abda and Masa lords are.
    pub kaala_lords: KaalaLords,
    /// The declination the Ayana reads.
    pub kranti: Kranti,
    /// Whether the Sun's Ayana is doubled in Kaala.
    pub sun_ayana: SunAyana,
    /// Whether grahas at war gain and lose the Yuddha bala.
    pub yuddha: Yuddha,
    /// The Sun's and the Moon's Cheshta.
    pub luminary_cheshta: LuminaryCheshta,
    /// The mean elements the Cheshta of Mars to Saturn reads.
    pub cheshta: Cheshta,
    /// The natural strengths.
    pub naisargika: Naisargika,
    /// How the Drik weighs the drishtis received.
    pub drik: Drik,
    /// The rupas a graha must reach.
    pub required_rupas: RequiredRupas,
    /// How the Ishta and Kashta phalas are read.
    pub ishta_kashta: IshtaKashta,
}

impl ShadbalaRules {
    /// BPHS ch. 27's reading as translated, at every fork it decides, and
    /// Sripati's where it does not (the Cheshta's mean elements, the Yuddha).
    pub const BPHS: ShadbalaRules = ShadbalaRules {
        saptavargaja: Saptavargaja::Compound,
        drekkana: Drekkana::MaleFemaleNeuter,
        dig: DigKendras::Angles,
        nathonnatha: Nathonnatha::Midnight,
        pre_dawn_night: PreDawnNight::PreviousEvening,
        benefics: Benefics::Conditional,
        kaala_lords: KaalaLords::Ahargana,
        kranti: Kranti::True,
        sun_ayana: SunAyana::Doubled,
        yuddha: Yuddha::Sripati,
        luminary_cheshta: LuminaryCheshta::AyanaAndPaksha,
        cheshta: Cheshta::Sripati,
        naisargika: Naisargika::Exact,
        drik: Drik::QuarterWithJupiterMercury,
        required_rupas: RequiredRupas::Bphs,
        ishta_kashta: IshtaKashta::Rays,
    };

    /// Sripati's reading, as B.V. Raman works it in *Graha and Bhava Balas*.
    pub const SRIPATI: ShadbalaRules = ShadbalaRules {
        saptavargaja: Saptavargaja::Compound,
        drekkana: Drekkana::MaleNeuterFemale,
        dig: DigKendras::Angles,
        nathonnatha: Nathonnatha::Midnight,
        pre_dawn_night: PreDawnNight::PreviousEvening,
        benefics: Benefics::Conditional,
        kaala_lords: KaalaLords::Ahargana,
        kranti: Kranti::HinduTable,
        sun_ayana: SunAyana::Doubled,
        yuddha: Yuddha::Sripati,
        luminary_cheshta: LuminaryCheshta::None,
        cheshta: Cheshta::Sripati,
        naisargika: Naisargika::Exact,
        drik: Drik::Quarter,
        required_rupas: RequiredRupas::Sripati,
        ishta_kashta: IshtaKashta::SquareRoots,
    };

    /// The conformance corpus's recording engine's reading at every fork.
    pub const RECORDING_ENGINE: ShadbalaRules = ShadbalaRules {
        saptavargaja: Saptavargaja::Natural,
        drekkana: Drekkana::MaleFemaleNeuter,
        dig: DigKendras::LagnaProjection,
        nathonnatha: Nathonnatha::Arc,
        pre_dawn_night: PreDawnNight::SameEvening,
        benefics: Benefics::Fixed,
        kaala_lords: KaalaLords::Sankranti,
        kranti: Kranti::Ecliptic,
        sun_ayana: SunAyana::NotInKaala,
        yuddha: Yuddha::None,
        luminary_cheshta: LuminaryCheshta::AyanaAndElongation,
        cheshta: Cheshta::RecordingEngine,
        naisargika: Naisargika::Hundredths,
        drik: Drik::Full,
        required_rupas: RequiredRupas::Sripati,
        ishta_kashta: IshtaKashta::ShadbalaCheshta,
    };

    /// The rules a context's settings name.
    ///
    /// # Errors
    ///
    /// `UNSUPPORTED` for a bala scheme other than the six strengths of
    /// BPHS ch. 27, which is the one the catalogue defines.
    pub fn of(settings: &Settings) -> Result<ShadbalaRules, Error> {
        if settings.strength.bala_scheme != BalaScheme::Parashara {
            return Err(Error::unsupported(format!(
                "the {} bala scheme defines no components the SDK can compute; it computes {}, the six strengths of BPHS ch. 27",
                settings.strength.bala_scheme.key(),
                BalaScheme::Parashara.key()
            ))
            .with_field("strength.bala_scheme"));
        }
        Ok(ShadbalaRules {
            saptavargaja: settings.strength.saptavargaja,
            drekkana: settings.strength.drekkana,
            dig: settings.strength.dig,
            nathonnatha: settings.strength.nathonnatha,
            pre_dawn_night: settings.strength.pre_dawn_night,
            benefics: settings.strength.benefics,
            kaala_lords: settings.strength.kaala_lords,
            kranti: settings.strength.kranti,
            sun_ayana: settings.strength.sun_ayana,
            yuddha: settings.strength.yuddha,
            luminary_cheshta: settings.strength.luminary_cheshta,
            cheshta: settings.strength.cheshta,
            naisargika: settings.strength.naisargika,
            drik: settings.strength.drik,
            required_rupas: settings.strength.required_rupas,
            ishta_kashta: settings.strength.ishta_kashta,
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
    /// Gained by the victor and lost by the vanquished of a planetary war.
    #[serde(default)]
    pub yuddha: f64,
}

impl KaalaBala {
    /// The components up to the Hora bala, which a Yuddha bala weighs.
    #[must_use]
    pub fn to_hora(&self) -> f64 {
        self.nathonnatha
            + self.paksha
            + self.tribhaga
            + self.abda
            + self.masa
            + self.vara
            + self.hora
    }

    /// The nine together.
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
            + self.yuddha
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
    /// How far it tends to good, 0 to 60 (BPHS ch. 28).
    #[serde(default)]
    pub ishta: f64,
    /// How far it tends to harm, 0 to 60.
    #[serde(default)]
    pub kashta: f64,
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
pub(crate) fn apart(a: f64, b: f64) -> f64 {
    let diff = (a.rem_euclid(360.0) - b.rem_euclid(360.0)).abs();
    if diff > 180.0 { 360.0 - diff } else { diff }
}

/// A graha's entry in a row kept Sun to Saturn.
pub(crate) fn seat<T: Copy>(row: &[T; 7], graha: Graha) -> T {
    let [first, ..] = *row;
    GRAHAS
        .iter()
        .zip(row)
        .find_map(|(g, value)| (*g == graha).then_some(*value))
        .unwrap_or(first)
}

/// The sign a longitude falls in.
pub(crate) fn sign_of(longitude: f64) -> Rashi {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude over 30 degrees is 0 to 11"
    )]
    let index = (longitude.rem_euclid(360.0) / 30.0) as u16;
    Rashi::from_id(index.min(11)).unwrap_or(Rashi::Aries)
}

/// The Moon's elongation from the Sun, 0 at new moon and 180 at full.
pub(crate) fn elongation(chart: &ShadbalaChart) -> f64 {
    let [sun, moon, ..] = chart.grahas;
    apart(moon.longitude, sun.longitude)
}

/// Whether a graha counts as a benefic for Paksha and Drik under `rule`.
///
/// Under [`Benefics::Conditional`] the Moon is a benefic from the eighth day
/// of the bright half to the eighth of the dark, her elongation 84° to 276°,
/// and Mercury unless it shares its sign with the Sun, Mars, Saturn, a node or
/// a malefic Moon (B.V. Raman, Arts. 53 and 117–118).
pub(crate) fn is_benefic(graha: Graha, chart: &ShadbalaChart, rule: Benefics) -> bool {
    match (graha, rule) {
        (Graha::Jupiter | Graha::Venus, _) | (Graha::Moon | Graha::Mercury, Benefics::Fixed) => {
            true
        }
        (Graha::Moon, _) => waxing(chart),
        (Graha::Mercury, _) => {
            let sign = sign_of(seat(&chart.grahas, Graha::Mercury).longitude);
            let beside = |longitude: f64| sign_of(longitude) == sign;
            let [sun, moon, mars, _, _, _, saturn] = chart.grahas;
            let malefic_moon = !waxing(chart) && beside(moon.longitude);
            let node = chart
                .rahu
                .is_some_and(|rahu| beside(rahu) || beside(rahu + 180.0));
            !(beside(sun.longitude)
                || beside(mars.longitude)
                || beside(saturn.longitude)
                || malefic_moon
                || node)
        }
        _ => false,
    }
}

/// Whether the Moon is in her benefic half: from the eighth tithi of the
/// bright fortnight through the eighth of the dark.
fn waxing(chart: &ShadbalaChart) -> bool {
    let [sun, moon, ..] = chart.grahas;
    let ahead = (moon.longitude - sun.longitude).rem_euclid(360.0);
    (84.0..276.0).contains(&ahead)
}

impl ShadbalaReading {
    /// A chart's Shadbala under `rules`.
    #[must_use]
    pub fn of(chart: &ShadbalaChart, rules: ShadbalaRules) -> ShadbalaReading {
        let lords = kaala::Lords::of(chart, rules.kaala_lords);
        let mut grahas: Vec<GrahaShadbala> = GRAHAS
            .iter()
            .map(|graha| {
                let graha = *graha;
                let sthana = sthana::of(graha, chart, &rules);
                let kaala = kaala::of(graha, chart, &rules, &lords);
                let naisargika = if rules.naisargika == Naisargika::Exact {
                    seat(&NAISARGIKA_SEVENTHS, graha) * VIRUPAS_PER_RUPA / 7.0
                } else {
                    seat(&NAISARGIKA_HUNDREDTHS, graha)
                };
                let required_rupas = if rules.required_rupas == RequiredRupas::Bphs {
                    seat(&BPHS_REQUIRED, graha)
                } else {
                    seat(&SRIPATI_REQUIRED, graha)
                };
                GrahaShadbala {
                    graha,
                    sthana,
                    dig: dig::of(graha, chart, rules.dig),
                    kaala,
                    cheshta: cheshta::of(graha, chart, &rules),
                    naisargika,
                    drik: drik::of(graha, chart, &rules),
                    virupas: 0.0,
                    rupas: 0.0,
                    required_rupas,
                    strong: false,
                    ishta: 0.0,
                    kashta: 0.0,
                }
            })
            .collect();
        if rules.yuddha == Yuddha::Sripati {
            yuddha::apply(chart, &mut grahas);
        }
        for graha in &mut grahas {
            graha.virupas = graha.sthana.total()
                + graha.dig
                + graha.kaala.total()
                + graha.cheshta
                + graha.naisargika
                + graha.drik;
            graha.rupas = graha.virupas / VIRUPAS_PER_RUPA;
            graha.strong = graha.rupas >= graha.required_rupas;
            (graha.ishta, graha.kashta) = ishta::of(graha, chart, rules.ishta_kashta);
        }
        ShadbalaReading { rules, grahas }
    }
}
