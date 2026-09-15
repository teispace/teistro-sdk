//! Sripati's Shadbala against B.V. Raman's worked Standard Horoscope
//! (*Graha and Bhava Balas*, thirteenth edition, Examples 3 to 56): every
//! component of every graha, from the inputs his Chapter I gives, under
//! [`ShadbalaRules::SRIPATI`].
//!
//! Raman works in degrees and minutes and rounds to one or two decimals, so
//! a component matches within [`ROUNDING`]; the few cells where his own
//! arithmetic slips are named where they are compared, with the value his
//! working implies.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they built"
)]

use teistro_core::angle::Nas;
use teistro_core::catalogue::{Graha, Rashi, Varga};
use teistro_core::quantity::Degrees;
use teistro_strength::ashtakavarga::GRAHAS;
use teistro_strength::shadbala::{
    SAPTAVARGAJA_VARGAS, ShadbalaChart, ShadbalaGraha, ShadbalaReading, ShadbalaRules,
};
use teistro_vargas::{Scheme, sign};

/// How far Raman's degrees-and-minutes arithmetic moves a component.
const ROUNDING: f64 = 0.15;

/// Degrees, minutes and seconds.
fn dms(degrees: f64, minutes: f64, seconds: f64) -> f64 {
    degrees + minutes / 60.0 + seconds / 3600.0
}

/// Raman's Standard Horoscope, from his Chapter I and the examples that use it.
fn standard_horoscope() -> ShadbalaChart {
    // Art. 6: nirayana longitudes, Sun to Saturn.
    let longitudes = [
        dms(180.0, 53.0, 55.0),
        dms(311.0, 17.0, 19.0),
        dms(229.0, 30.0, 34.0),
        dms(181.0, 31.0, 34.0),
        dms(84.0, 0.0, 49.0),
        dms(171.0, 9.0, 56.0),
        dms(124.0, 22.0, 41.0),
    ];
    // Example 32: the ayanamsha, 21° 16′.
    let ayanamsha = dms(21.0, 16.0, 0.0);
    // Art. 7: Makara rising, so each graha's house counts from Capricorn.
    let lagna = Rashi::Capricorn as u8;
    let varga_sign = |varga: Varga, longitude: f64| {
        sign(
            &Scheme::of(varga),
            Nas::from_degrees(Degrees::try_new(longitude).unwrap()),
        )
    };
    let house = |longitude: f64| (varga_sign(Varga::D1, longitude) as u8 + 12 - lagna) % 12 + 1;
    // 16 October 1918, 2h 6m 16s p.m. LMT at 77° 35′ E: 8h 55m 56s UT.
    let day = 2_421_882.5;
    let ut = |hours: f64, minutes: f64, seconds: f64| day + dms(hours, minutes, seconds) / 24.0;
    ShadbalaChart {
        grahas: longitudes.map(|longitude| ShadbalaGraha {
            longitude,
            tropical: longitude + ayanamsha,
            latitude: 0.0,
            house: house(longitude),
        }),
        // Art. 6: Rahu.
        rahu: Some(dms(234.0, 23.0, 47.0)),
        vargas: SAPTAVARGAJA_VARGAS.map(|varga| longitudes.map(|l| varga_sign(varga, l))),
        instant: ut(8.0, 55.0, 56.0),
        // Art. 5: sunrise 5h 54m LMT, a day of 29 ghatis 20 vighatis.
        sunrise: ut(0.0, 43.0, 40.0),
        sunset: ut(0.0, 43.0, 40.0) + 29.333_333 / 60.0,
        next_sunrise: ut(0.0, 43.0, 40.0) + 1.0,
        after_midnight: false,
        civil_day: 2_421_883,
        // Examples 22 and 31: a Wednesday, in the Moon's hora.
        weekday_lord: Graha::Mercury,
        hora_lord: Graha::Moon,
        sankranti_lord: None,
        // Art. 29: the first and tenth bhava madhyas.
        ascendant: dms(298.0, 27.0, 0.0),
        midheaven: dms(216.0, 36.0, 0.0),
        ayanamsha,
        obliquity: 24.0,
    }
}

/// One component's worked values, Sun to Saturn.
type Row = (
    &'static str,
    [f64; 7],
    fn(&teistro_strength::GrahaShadbala) -> f64,
);

#[test]
fn raman_s_standard_horoscope_reproduces() {
    let reading = ShadbalaReading::of(&standard_horoscope(), ShadbalaRules::SRIPATI);
    let rows: [Row; 15] = [
        // Example 3; Mars's 111° 30′ 34″ over 3 is 37.17, printed 37.06.
        (
            "uchcha",
            [3.00, 32.75, 37.06, 54.50, 56.33, 1.95, 34.80],
            |g| g.sthana.uchcha,
        ),
        // Example 9, the Moon's row summing to 48.75 (its total misprinted 49.75).
        (
            "saptavargaja",
            [90.0, 48.75, 90.0, 135.0, 71.25, 116.25, 97.5],
            |g| g.sthana.saptavargaja,
        ),
        (
            "ojayugma",
            [30.0, 15.0, 15.0, 30.0, 15.0, 30.0, 15.0],
            |g| g.sthana.ojayugma,
        ),
        (
            "kendradi",
            [60.0, 30.0, 30.0, 60.0, 15.0, 15.0, 30.0],
            |g| g.sthana.kendradi,
        ),
        ("drekkana", [15.0, 0.0, 0.0, 0.0, 0.0, 15.0, 0.0], |g| {
            g.sthana.drekkana
        }),
        (
            "dig",
            [48.10, 31.56, 55.70, 21.09, 11.50, 15.15, 58.02],
            |g| g.dig,
        ),
        (
            "nathonnatha",
            [48.32, 11.68, 11.68, 60.0, 48.32, 48.32, 11.68],
            |g| g.kaala.nathonnatha,
        ),
        (
            "paksha",
            [16.54, 86.92, 16.54, 16.54, 43.46, 43.46, 16.54],
            |g| g.kaala.paksha,
        ),
        ("tribhaga", [0.0, 0.0, 0.0, 0.0, 60.0, 0.0, 60.0], |g| {
            g.kaala.tribhaga
        }),
        ("abda", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 15.0], |g| {
            g.kaala.abda
        }),
        ("masa and vara", [0.0, 0.0, 0.0, 75.0, 0.0, 0.0, 0.0], |g| {
            g.kaala.masa + g.kaala.vara
        }),
        ("hora", [0.0, 60.0, 0.0, 0.0, 0.0, 0.0, 0.0], |g| {
            g.kaala.hora
        }),
        // Example 33: Jupiter's kranti of 23° 05′ is printed 23.5 and Saturn's
        // 13° 10′ as 13.03, so those two stand further from the table's.
        (
            "ayana",
            [38.12, 43.44, 1.84, 41.25, 58.86, 23.75, 13.55],
            |g| g.kaala.ayana,
        ),
        (
            "cheshta",
            [0.0, 0.0, 22.23, 2.30, 35.26, 5.95, 21.14],
            |g| g.cheshta,
        ),
        (
            "drik",
            [15.86, -21.73, 0.95, 15.64, -16.04, 18.47, 7.21],
            |g| g.drik,
        ),
    ];
    let mut far = Vec::new();
    for (name, raman, ours) in rows {
        for ((graha, expected), strength) in GRAHAS.iter().zip(raman).zip(&reading.grahas) {
            let got = ours(strength);
            if (got - expected).abs() > ROUNDING {
                far.push(format!("{graha:?} {name}: {got:.2} against {expected:.2}"));
            }
        }
    }
    assert!(
        far.is_empty(),
        "outside Raman's rounding:\n{}",
        far.join("\n")
    );
}

#[test]
fn raman_s_totals_and_requirements_reproduce() {
    let reading = ShadbalaReading::of(&standard_horoscope(), ShadbalaRules::SRIPATI);
    // Example 56: the Shadbala pindas in virupas. Three are his own slips
    // corrected: the Sun's printed 424.24 is 424.94 by its own components,
    // and Jupiter's and Saturn's carry Example 33's kranti slips of 0.54 and
    // 0.20. (Mercury's 537.02 he prints as 8.85 rupas; it is 8.95.)
    let virupas = [424.94, 389.80, 298.14, 537.02, 433.17, 376.15, 389.01];
    for ((graha, expected), strength) in GRAHAS.iter().zip(virupas).zip(&reading.grahas) {
        assert!(
            (strength.virupas - expected).abs() < 0.5,
            "{graha:?}: {:.2} against {expected}",
            strength.virupas
        );
    }
    // Example 57: every graha reaches Sripati's requirement but Mars.
    for strength in &reading.grahas {
        assert_eq!(
            strength.strong,
            strength.graha != Graha::Mars,
            "{:?}",
            strength.graha
        );
    }
}
