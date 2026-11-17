//! Ketaki's ayanamsha (`docs/03-design/ketaki.md` §4.1 and §9, T1 to T5;
//! `docs/01-research/ketaki.md` §2): the catalogue's `KETAKI` against the
//! `CUSTOM` form that restates it, against the values Ketkar's own texts
//! and the 1955 survey print, and against Lahiri and Spica.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::print_stdout,
    reason = "tests fail by panicking and print the measurement under --nocapture"
)]

use teistro_astro::ayanamsha::{Basis, Definition, KETAKI, definition, mean_deg, value_deg};
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_astro::stars::{Corrections, Options, place_of};
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::{Ayanamsha, Star};
use teistro_core::quantity::{JulianDay, Tt};
use teistro_core::settings::AyanamshaChoice;

const DELTA_T: DeltaTModel = DeltaTModel::TableThenModel;
const MODEL: PrecessionModel = PrecessionModel::Vondrak2011;
/// The text's sidereal year, 365 days 15 ghatis 22.9 palas (*Jyotirgaṇitam*,
/// 1937, p. 7).
const SIDEREAL_YEAR: f64 = 365.256_361;

fn ketaki() -> AyanamshaChoice {
    Ayanamsha::Ketaki.into()
}

/// The `CUSTOM` form with the plan's three numbers (§4.1).
fn custom() -> AyanamshaChoice {
    AyanamshaChoice::Custom {
        epoch_jd_tt: 2_407_086.5,
        value_deg: 22.1425,
        rate_deg_per_year: 0.013_952_138,
    }
}

/// The mean value at a TT Julian day, arcseconds.
fn arcsec(choice: &AyanamshaChoice, jd: f64) -> f64 {
    mean_deg(choice, JulianDay::<Tt>::literal(jd), MODEL, DELTA_T).unwrap() * 3600.0
}

fn dms(degrees: u32, minutes: u32, seconds: f64) -> f64 {
    f64::from(degrees * 3600 + minutes * 60) + seconds
}

/// The Mesha sankranti `n` of the text's sidereal years after the epoch's:
/// the instant the text's rule gives each year's value at.
fn mesha(n: u32) -> f64 {
    KETAKI.epoch_jd_tt + f64::from(n) * SIDEREAL_YEAR
}

/// T1: the member is the `CUSTOM` form to the bit, at 200 instants over
/// 1800 to 2400, on both bases and under every precession model (a linear
/// rule reads none).
#[test]
fn the_member_is_its_custom_form_to_the_bit() {
    let Definition::Linear(linear) = definition(Ayanamsha::Ketaki) else {
        panic!("KETAKI is a linear rule");
    };
    assert_eq!(linear, KETAKI);
    let (start, end) = (2_378_496.5, 2_597_641.5);
    let step = (end - start) / 199.0;
    let models = [
        PrecessionModel::Vondrak2011,
        PrecessionModel::Iau2006,
        PrecessionModel::Iau1976,
        PrecessionModel::Newcomb,
    ];
    for i in 0..200_u32 {
        let at = JulianDay::<Tt>::literal(start + f64::from(i) * step);
        for model in models {
            for basis in [Basis::Mean, Basis::True] {
                let member = value_deg(&ketaki(), at, basis, model, DELTA_T).unwrap();
                let restated = value_deg(&custom(), at, basis, model, DELTA_T).unwrap();
                assert_eq!(
                    member.to_bits(),
                    restated.to_bits(),
                    "{at:?} {model:?} {basis:?}"
                );
            }
        }
    }
}

/// T2 and T3: the values Ketkar's texts print, each within the plan's
/// tolerance.
#[test]
fn the_texts_own_values_are_reproduced() {
    let cases = [
        // KGG p. 12, the rule's example: Mesha of Shaka 1815.
        ("KGG Shaka 1815", mesha(15), dms(22, 21, 6.0), 0.5),
        // JG1937 p. 7: Mesha of Shaka 1800, the epoch.
        ("JG1937 Shaka 1800", mesha(0), dms(22, 8, 33.04), 0.5),
        // JG1937 p. 296: Chaitra shukla 15, Shaka 1806, 10 April 1884.
        ("JG1937 Shaka 1806", 2_409_276.5, dms(22, 13, 36.0), 2.5),
        // JG1937 p. 346: from Spica on 1 January 1892.
        ("JG1937 1892", 2_412_098.5, dms(22, 20, 2.9), 1.0),
    ];
    for (what, jd, printed, tolerance) in cases {
        let off = arcsec(&ketaki(), jd) - printed;
        println!("{what}: {off:+.2}″");
        assert!(
            off.abs() <= tolerance,
            "{what}: {off:+.2}″ against ±{tolerance}″"
        );
    }
}

/// T4: the 1955 survey's sixteen almanacs on Ketkar's method print
/// 23°12′06″ to 23°12′10″ for Shaka 1876 (Report of the Calendar Reform
/// Committee, Annexure VI); the member falls inside, to a second.
#[test]
fn the_survey_range_of_1954_holds_the_member() {
    let value = arcsec(&ketaki(), mesha(76));
    println!("Shaka 1876: 23°12′{:.2}″", value - dms(23, 12, 0.0));
    assert!(
        (dms(23, 12, 6.0) - 1.0..=dms(23, 12, 10.0) + 1.0).contains(&value),
        "{value}"
    );
}

/// T5: Ketaki stands 52″ to 67″ below mean Lahiri over 1878 to 2100, and
/// within 6″ of Spica's mean geometric longitude less 180°, read from the
/// SDK's own star table (research §2.4).
#[test]
fn the_gap_to_lahiri_and_to_spica_is_the_measured_one() {
    let lahiri: AyanamshaChoice = Ayanamsha::Lahiri.into();
    let geometric = Options {
        precession: MODEL,
        corrections: Corrections::GEOMETRIC,
    };
    let (start, end) = (mesha(0), 2_488_069.5);
    let (mut to_lahiri, mut to_spica) = ((f64::MAX, f64::MIN), (f64::MAX, f64::MIN));
    let mut jd = start;
    while jd <= end {
        let at = JulianDay::<Tt>::literal(jd);
        let k = mean_deg(&ketaki(), at, MODEL, DELTA_T).unwrap();
        let l = mean_deg(&lahiri, at, MODEL, DELTA_T).unwrap();
        let spica = place_of(Star::Spica, at, &geometric).unwrap().lon_deg - 180.0;
        let d_lahiri = difference_deg(k, l) * 3600.0;
        let d_spica = difference_deg(k, spica) * 3600.0;
        to_lahiri = (to_lahiri.0.min(d_lahiri), to_lahiri.1.max(d_lahiri));
        to_spica = (to_spica.0.min(d_spica), to_spica.1.max(d_spica));
        jd += 30.0;
    }
    println!("Ketaki − Lahiri: {to_lahiri:.2?}″; Ketaki − (Spica − 180°): {to_spica:.2?}″");
    assert!(
        to_lahiri.0 >= -67.0 && to_lahiri.1 <= -52.0,
        "{to_lahiri:?}"
    );
    assert!(to_spica.0 >= -6.0 && to_spica.1 <= 6.0, "{to_spica:?}");
}
