//! Custom ayanamshas beyond a linear rate (SR-112,
//! `docs/03-design/astro-custom-ayanamsha.md`): a value at an epoch
//! carried by a named precession model, and a star of the catalogue held
//! at a sidereal longitude, each held to the catalogued member it can
//! restate. The bounds are the page's measured agreement.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::print_stdout,
    reason = "tests fail by panicking and print the measurement under --nocapture"
)]

use teistro_astro::ayanamsha::{
    Basis, Definition, Reading, definition, mean_deg, speed_deg_per_day, value_deg,
};
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::iau::{RAD2DEG, nut00b};
use teistro_astro::precession::PrecessionModel;
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::{Ayanamsha, Star};
use teistro_core::error::Status;
use teistro_core::quantity::{JulianDay, Tt};
use teistro_core::settings::{AyanamshaBasis, AyanamshaChoice, Precession, root};

const DELTA_T: DeltaTModel = DeltaTModel::TableThenModel;
/// Lahiri's epoch, 1956 March 21 0h TT.
const LAHIRI_EPOCH: f64 = 2_435_553.5;
/// The Indian Astronomical Ephemeris's 23°15′00.658″ there, a true value.
const IAE_TRUE_DEG: f64 = 23.250_182_778;
/// The nutation the Swiss Ephemeris takes from it to carry it as a mean
/// value (its IAU 1980 series's, 16.769″).
const SWISS_NUTATION_DEG: f64 = 0.004_658_035;

/// 1800-01-01 to 2200-01-01 in TT, every `step` days.
fn span(step: f64) -> impl Iterator<Item = JulianDay<Tt>> {
    let (start, end) = (2_378_496.5, 2_524_593.5);
    (0..)
        .map(move |i| start + f64::from(i) * step)
        .take_while(move |jd| *jd <= end)
        .map(JulianDay::<Tt>::literal)
}

/// The largest difference, arcseconds, between two choices over the span
/// on both bases, the sign kept.
fn worst(a: &AyanamshaChoice, b: &AyanamshaChoice, model: PrecessionModel, step: f64) -> f64 {
    let mut worst = 0.0_f64;
    for at in span(step) {
        for basis in [Basis::Mean, Basis::True] {
            let d = difference_deg(
                value_deg(a, at, basis, model, DELTA_T).unwrap(),
                value_deg(b, at, basis, model, DELTA_T).unwrap(),
            ) * 3600.0;
            if d.abs() > worst.abs() {
                worst = d;
            }
        }
    }
    worst
}

fn precessed(value_deg: f64, basis: AyanamshaBasis, precession: Precession) -> AyanamshaChoice {
    AyanamshaChoice::Precessed {
        epoch_jd_tt: LAHIRI_EPOCH,
        value_deg,
        basis,
        precession,
    }
}

/// SR-112's acceptance: an epoch-plus-precession definition equal to
/// Lahiri at Lahiri's epoch, carried by IAU 2006 precession, reproduces
/// the catalogued Lahiri under the root's precession (Vondrák 2011) to
/// 0.01″ over 1800–2200, on either basis. Carried by the root's own model
/// it is the same angle to the rounding.
#[test]
fn a_precessed_lahiri_reproduces_lahiri_from_1800_to_2200() {
    let lahiri: AyanamshaChoice = Ayanamsha::Lahiri.into();
    let model = PrecessionModel::from_knob(root().frame.precession).unwrap();
    let at_epoch = mean_deg(&lahiri, JulianDay::literal(LAHIRI_EPOCH), model, DELTA_T).unwrap();
    // The number the façade's test and the design page state.
    assert!(
        (at_epoch - 23.245_560_954_594_648).abs() < 1e-12,
        "{at_epoch}"
    );
    let by_iau2006 = worst(
        &precessed(at_epoch, AyanamshaBasis::Mean, Precession::Iau2006),
        &lahiri,
        model,
        10.0,
    );
    let by_root = worst(
        &precessed(at_epoch, AyanamshaBasis::Mean, root().frame.precession),
        &lahiri,
        model,
        10.0,
    );
    println!("carried by IAU 2006: {by_iau2006:.6}″; by the root's model: {by_root:e}″");
    assert!(by_iau2006.abs() < 0.01, "{by_iau2006}″");
    assert!(by_root.abs() < 1e-6, "{by_root}″");
}

/// The basis a value is stated on: a true value is the mean one with the
/// epoch's nutation in longitude, so the two statements of one definition
/// give one angle everywhere; and Lahiri's own number, the IAE's true
/// 23°15′00.658″, carried by IAU 1976 under IAU 1976 is the catalogued
/// Lahiri less the two nutation series' difference at the epoch (IAU
/// 2000B's 16.777″ against the 16.769″ the Swiss Ephemeris subtracts),
/// and stated as Swiss states it, mean, it is the catalogued Lahiri.
#[test]
fn a_value_states_its_basis() {
    let (date1, date2) = JulianDay::<Tt>::literal(LAHIRI_EPOCH).split();
    let nutation_deg = nut00b(date1, date2).dpsi * RAD2DEG;
    let mean = precessed(23.2, AyanamshaBasis::Mean, Precession::Iau2006);
    let stated_true = precessed(
        23.2 + nutation_deg,
        AyanamshaBasis::True,
        Precession::Iau2006,
    );
    let same = worst(&mean, &stated_true, PrecessionModel::Vondrak2011, 365.25);
    assert!(same.abs() < 1e-9, "{same}″");

    let lahiri: AyanamshaChoice = Ayanamsha::Lahiri.into();
    let iau1976 = PrecessionModel::Iau1976;
    let swiss = precessed(
        IAE_TRUE_DEG - SWISS_NUTATION_DEG,
        AyanamshaBasis::Mean,
        Precession::Iau1976,
    );
    let as_swiss = worst(&swiss, &lahiri, iau1976, 365.25);
    let iae = worst(
        &precessed(IAE_TRUE_DEG, AyanamshaBasis::True, Precession::Iau1976),
        &lahiri,
        iau1976,
        365.25,
    );
    let series = (SWISS_NUTATION_DEG - nutation_deg) * 3600.0;
    println!(
        "nutation at the epoch {:.4}″; Swiss's statement {as_swiss:e}″; the IAE's {iae:.5}″ (series {series:.5}″)",
        nutation_deg * 3600.0
    );
    assert!(as_swiss.abs() < 1e-6, "{as_swiss}″");
    assert!((iae - series).abs() < 1e-6, "{iae}″ against {series}″");
}

/// A precessed definition is the construction every catalogued epoch
/// member takes, which the Teimeris table holds to 1e-7″: one with no
/// fitted model, restated, is the member to the bit under its model.
#[test]
fn a_precessed_definition_is_the_catalogues_construction() {
    let Definition::Epoch(epoch) = definition(Ayanamsha::Ushashashi) else {
        panic!("epoch-defined");
    };
    for (knob, model) in [
        (Precession::Vondrak2011, PrecessionModel::Vondrak2011),
        (Precession::Iau2006, PrecessionModel::Iau2006),
        (Precession::Newcomb, PrecessionModel::Newcomb),
    ] {
        let own = AyanamshaChoice::Precessed {
            epoch_jd_tt: epoch.jd,
            value_deg: epoch.value_deg,
            basis: AyanamshaBasis::Mean,
            precession: knob,
        };
        for at in span(3_652.5) {
            // The context's model does not carry a precessed value: its own does.
            let got = mean_deg(&own, at, PrecessionModel::Iau1976, DELTA_T).unwrap();
            let want = mean_deg(&Ayanamsha::Ushashashi.into(), at, model, DELTA_T).unwrap();
            assert_eq!(got.to_bits(), want.to_bits(), "{model:?} at {}", at.get());
        }
    }
}

/// Every catalogued member anchored to a star's longitude, or a
/// direction's geometric one, restated as a star held at a longitude, is
/// the member on either basis: to the bit where the author wrote one
/// term, within the sum's rounding where two. Wilhelm's galactic centre is
/// read by right ascension, which a held longitude does not express.
#[test]
fn a_held_star_restates_every_anchored_member() {
    let mut restated = 0;
    for id in &Ayanamsha::ALL {
        let Definition::Object {
            anchor,
            held_deg: [first, second],
            reading,
        } = definition(*id)
        else {
            continue;
        };
        if reading == Reading::RightAscension {
            continue;
        }
        restated += 1;
        let held = AyanamshaChoice::Anchored {
            star: anchor,
            longitude_deg: first + second,
        };
        let bound = if second == 0.0 { 0.0 } else { 1e-9 };
        let w = worst(&held, &(*id).into(), PrecessionModel::Vondrak2011, 3_652.5);
        assert!(w.abs() <= bound, "{}: {w}″", id.key());
    }
    assert_eq!(restated, 11);
}

/// A held star turns with the precession, about 50.3″ a year, and with
/// the star's own motion; its rate on a day is True Chitra's, the annual
/// aberration's swing of up to 130″ a year riding on the precession; a
/// custom definition with a number that is not finite is refused by name.
#[test]
fn a_held_star_moves_and_bad_numbers_are_refused() {
    let j2000 = JulianDay::<Tt>::literal(2_451_545.0);
    let model = PrecessionModel::Vondrak2011;
    let spica = AyanamshaChoice::Anchored {
        star: Star::Spica,
        longitude_deg: 180.0,
    };
    let year_later = JulianDay::<Tt>::literal(2_451_545.0 + 365.25);
    let yearly = difference_deg(
        mean_deg(&spica, year_later, model, DELTA_T).unwrap(),
        mean_deg(&spica, j2000, model, DELTA_T).unwrap(),
    ) * 3600.0;
    assert!((yearly - 50.3).abs() < 0.2, "{yearly}″ in a year");
    let speed = speed_deg_per_day(&spica, j2000, model, DELTA_T).unwrap();
    let chitra = speed_deg_per_day(&Ayanamsha::TrueChitra.into(), j2000, model, DELTA_T).unwrap();
    assert_eq!(speed.to_bits(), chitra.to_bits());
    for bad in [
        AyanamshaChoice::Anchored {
            star: Star::Spica,
            longitude_deg: f64::NAN,
        },
        precessed(f64::INFINITY, AyanamshaBasis::Mean, Precession::Iau2006),
        AyanamshaChoice::Precessed {
            epoch_jd_tt: f64::NAN,
            value_deg: 23.0,
            basis: AyanamshaBasis::True,
            precession: Precession::Iau2006,
        },
        AyanamshaChoice::Custom {
            epoch_jd_tt: 2_451_545.0,
            value_deg: 23.0,
            rate_deg_per_year: f64::NAN,
        },
    ] {
        let error = mean_deg(&bad, j2000, model, DELTA_T).unwrap_err();
        assert_eq!(error.status, Status::InvalidArg, "{bad:?}");
        assert_eq!(error.field(), Some("frame.ayanamsha"));
    }
}
