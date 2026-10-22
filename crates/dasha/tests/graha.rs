//! The Karaka dasha against the text's worked example (BPHS ch. 46 v. 178;
//! `docs/03-design/dasha-kernels.md`, cruxes C737 and C739).
//!
//! Santhanam's example chart (vol. 2, p. 558, its longitudes read from the
//! page image) has an Aquarius lagna, and his table under the verse
//! (p. 564) runs the eight karakas Sun 11, Venus 12, Rahu 4, Jupiter 12,
//! Saturn 9, Mercury 12, Moon 4 and Mars 3 years.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test fails by panicking and indexes what it built"
)]

use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::quantity::{Depth, JulianDay};
use teistro_core::settings::{AfterCycle, GrahaDivision, YearLength};
use teistro_dasha::{GrahaDasha, Timeline, karaka_years};

/// Santhanam's karakas in his order, each in the sign the page places it.
const KARAKAS: [(Graha, Rashi); 8] = [
    (Graha::Sun, Rashi::Capricorn),
    (Graha::Venus, Rashi::Aquarius),
    (Graha::Rahu, Rashi::Gemini),
    (Graha::Jupiter, Rashi::Aquarius),
    (Graha::Saturn, Rashi::Scorpio),
    (Graha::Mercury, Rashi::Aquarius),
    (Graha::Moon, Rashi::Gemini),
    (Graha::Mars, Rashi::Taurus),
];

/// His table's years, in the same order.
const YEARS: [u8; 8] = [11, 12, 4, 12, 9, 12, 4, 3];

const BIRTH: f64 = 2_451_545.0;

fn karaka(division: GrahaDivision) -> GrahaDasha {
    let lords = KARAKAS
        .iter()
        .map(|&(graha, sign)| (graha, f64::from(karaka_years(Rashi::Aquarius, sign))))
        .collect();
    GrahaDasha::new(
        lords,
        JulianDay::literal(BIRTH),
        YearLength::Julian36525,
        division,
        AfterCycle::End,
    )
    .unwrap()
}

#[test]
fn every_karaka_runs_the_years_santhanams_table_gives() {
    let years = KARAKAS.map(|(_, sign)| karaka_years(Rashi::Aquarius, sign));
    assert_eq!(years, YEARS);
    let dasha = karaka(GrahaDivision::Proportional);
    let mahadashas: Vec<_> = dasha.mahadashas().collect();
    assert_eq!(
        mahadashas
            .iter()
            .map(|period| period.lord)
            .collect::<Vec<_>>(),
        KARAKAS.map(|(graha, _)| graha)
    );
    // Sixty-seven years in all, and the cycle then ends.
    let end = mahadashas.last().unwrap().interval.to.get();
    assert!((end - (BIRTH + 67.0 * 365.25)).abs() < 1e-6);
    let after = JulianDay::literal(end + 1.0);
    assert!(dasha.at(after, Depth::MIN).deepest().is_none());
}

#[test]
fn the_antardashas_run_from_the_periods_own_karaka_in_proportion() {
    // Rahu's four years: Rahu's own antardasha first (ch. 51 v. 2), each
    // its own years of the sixty-seven (v. 1).
    let dasha = karaka(GrahaDivision::Proportional);
    let rahu = dasha.mahadasha(0, 2).unwrap();
    let antardashas: Vec<_> = dasha.children(&rahu).collect();
    assert_eq!(
        antardashas
            .iter()
            .map(|period| period.lord)
            .collect::<Vec<_>>(),
        [
            Graha::Rahu,
            Graha::Jupiter,
            Graha::Saturn,
            Graha::Mercury,
            Graha::Moon,
            Graha::Mars,
            Graha::Sun,
            Graha::Venus,
        ]
    );
    let year = 365.25;
    let ours = |at: usize| antardashas[at].interval.days();
    assert!((ours(0) - 4.0 * year * 4.0 / 67.0).abs() < 1e-6);
    assert!((ours(6) - 4.0 * year * 11.0 / 67.0).abs() < 1e-6);
    // Under the equal division, eight eighths of the four years.
    let equal = karaka(GrahaDivision::Equal);
    let rahu = equal.mahadasha(0, 2).unwrap();
    for part in equal.children(&rahu) {
        assert!((part.interval.days() - 4.0 * year / 8.0).abs() < 1e-6);
    }
}
