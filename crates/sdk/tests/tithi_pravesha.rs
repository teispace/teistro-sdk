//! The tithi pravesha through the façade: each year's phase return chosen
//! around its solar return, read back through charts founded at it
//! (`docs/03-design/tithi-pravesha.md`, C443 to C447).
//!
//! The birth is the annual chart's: the Sun at 0.05° of sidereal Aries,
//! about an hour into the sign. That makes the two rules part company in
//! a way the test can predict: every phase return before a solar return
//! falls with the Sun still in Pisces, so the natal-sign rule never takes
//! one, and the nearest-return rule takes one whenever it is the nearer.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{ChartKind, Graha};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::tajika::Reading;
use teistro::{
    ChartRequest, Context, Document, Ephemeris, TITHI_WINDOW_DAYS, TithiPravesha, TithiRule,
    UtcOffset,
};

const BIRTH: f64 = 2_447_995.489_583_333_5;

fn context() -> Context {
    Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a built context")
}

fn place() -> Place {
    Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    )
}

fn natal(sdk: &Context) -> Document {
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(BIRTH),
            &ChartRequest::at(place(), UtcOffset::try_from_seconds(20_700).unwrap()),
        )
        .unwrap()
        .value
}

/// The Moon less the Sun, `[0, 360)`.
fn elongation(chart: &teistro::ChartFoundation) -> f64 {
    (chart.graha(Graha::Moon).unwrap().longitude_deg
        - chart.graha(Graha::Sun).unwrap().longitude_deg)
        .rem_euclid(360.0)
}

/// Signed, in (-180, 180].
fn signed(apart: f64) -> f64 {
    let apart = apart.rem_euclid(360.0);
    if apart > 180.0 { apart - 360.0 } else { apart }
}

/// Every year's instant, read back through a chart founded a second
/// either side of it, finds the elongation just short of the birth's
/// before and just past it after; and under the natal-sign rule the Sun
/// of a chart founded at the instant stands in the birth's sign.
fn read_back(sdk: &Context, birth: &Document, years: &[TithiPravesha]) {
    let born = elongation(&birth.foundation);
    let second = 1.0 / 86_400.0;
    let instants: Vec<JulianDay<Utc>> = years
        .iter()
        .flat_map(|one| {
            [
                JulianDay::<Utc>::literal(one.at.get() - second),
                JulianDay::<Utc>::literal(one.at.get()),
                JulianDay::<Utc>::literal(one.at.get() + second),
            ]
        })
        .collect();
    let founded = sdk
        .chart()
        .found_many(&instants, &place(), UtcOffset::UTC, ChartKind::Natal)
        .unwrap()
        .value;
    let natal_sign = (birth.foundation.graha(Graha::Sun).unwrap().longitude_deg / 30.0).floor();
    for (one, three) in years.iter().zip(founded.chunks(3)) {
        assert!((one.elongation_deg - born).abs() < 1e-12, "{one:?}");
        let before = signed(elongation(&three[0]) - born);
        let after = signed(elongation(&three[2]) - born);
        // The Moon gains about half an arcsecond on the Sun in a second.
        assert!(before < 0.0 && before > -0.01, "{one:?}: {before}");
        assert!(after > 0.0 && after < 0.01, "{one:?}: {after}");
        if one.picked == TithiRule::NatalSign {
            let sun = three[1].graha(Graha::Sun).unwrap().longitude_deg;
            assert_eq!(
                (sun / 30.0).floor(),
                natal_sign,
                "{one:?}: the Sun at {sun}°"
            );
        }
    }
}

/// One a year, in order, each within the window of its own solar return
/// and a lunar year or a little more from the last: twelve or thirteen
/// synodic months, about 354 or 384 days.
#[test]
fn one_tithi_pravesha_a_year_reads_back_from_its_own_chart() {
    let sdk = context();
    let birth = natal(&sdk);
    for rule in TithiRule::ALL {
        let years = sdk.chart().tithi_praveshas(&birth, rule, 40).unwrap();
        assert!(years.iter().map(|one| one.year).eq(1..=40), "{rule:?}");
        assert!(years.iter().all(|one| one.rule == rule));
        let solar = sdk
            .chart()
            .praveshas(&birth, Reading::Sidereal, 40)
            .unwrap();
        for (one, sun) in years.iter().zip(&solar) {
            assert_eq!(one.solar_return, sun.at, "the year's own solar return");
            let off = one.at.get() - sun.at.get();
            assert!(off.abs() < TITHI_WINDOW_DAYS, "{one:?}: {off} days");
        }
        for pair in years.windows(2) {
            let gap = pair[1].at.get() - pair[0].at.get();
            assert!(
                (353.0..=385.5).contains(&gap),
                "{rule:?} {pair:?}: {gap} days"
            );
        }
        read_back(&sdk, &birth, &years);
    }
}

/// The nearest-return rule is never more than half a synodic month from
/// the solar return; the natal-sign rule, for a birth an hour into its
/// sign, never takes a return before it, and so the two part company in
/// the years the nearer return fell while the Sun was still in Pisces.
#[test]
fn the_rules_part_company_where_the_sign_says_so() {
    let sdk = context();
    let birth = natal(&sdk);
    let sign = sdk
        .chart()
        .tithi_praveshas(&birth, TithiRule::NatalSign, 40)
        .unwrap();
    let nearest = sdk
        .chart()
        .tithi_praveshas(&birth, TithiRule::NearestReturn, 40)
        .unwrap();
    let mut parted = 0;
    for (by_sign, by_nearest) in sign.iter().zip(&nearest) {
        let solar = by_sign.solar_return.get();
        assert!((by_nearest.at.get() - solar).abs() < 15.0, "{by_nearest:?}");
        assert_eq!(by_nearest.picked, TithiRule::NearestReturn);
        // An hour into Aries: the sojourn opened just before the solar
        // return and holds one return after it, never none.
        assert!(by_sign.in_sign >= 1, "{by_sign:?}");
        assert!(!by_sign.fell_back(), "{by_sign:?}");
        assert!(by_sign.at.get() > solar - 0.1, "{by_sign:?}");
        assert_eq!(by_sign.in_sign, by_nearest.in_sign, "counted alike");
        if by_sign.at != by_nearest.at {
            parted += 1;
            assert!(by_nearest.at.get() < solar, "{by_nearest:?}");
        }
    }
    // Before or after the solar return is about even over forty years.
    assert!((8..=32).contains(&parted), "{parted} of 40 years parted");
}

/// A year outside the cap is refused by the field that names it, before
/// anything is searched, as the solar returns refuse it; a context with
/// no ephemeris cannot search at all.
#[test]
fn a_year_that_cannot_be_asked_is_refused_by_name() {
    let sdk = context();
    let birth = natal(&sdk);
    for through in [0, 201] {
        let why = sdk
            .chart()
            .tithi_praveshas(&birth, TithiRule::NatalSign, through)
            .unwrap_err();
        assert_eq!(why.field(), Some("through"), "{why}");
        assert!(why.to_string().contains("1 to 200"), "{why}");
    }
    let bare = Context::builder()
        .build()
        .expect("a context with no ephemeris");
    assert!(
        bare.chart()
            .tithi_praveshas(&birth, TithiRule::NatalSign, 1)
            .is_err()
    );
}
