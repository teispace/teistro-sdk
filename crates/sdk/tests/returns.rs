//! A body's returns (`03-design/western-returns.md`, C255 to C258):
//! the lunar return Morin defines (*Astrologia Gallica* XXIII, pp.
//! 633–634), timed as the recast that holds his Richelieu figure times it,
//! and the Sun's return read back as the Tajika pravesha, sidereal and
//! tropical.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Graha;
use teistro::gochar::hits::Motion;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::tajika::Reading;
use teistro::{ChartRequest, Context, Document, Ephemeris, UtcOffset};

fn context(profile: Option<&str>) -> Context {
    let builder = Context::builder().ephemeris([Ephemeris::Builtin]);
    match profile {
        Some(profile) => builder.profile(profile),
        None => builder,
    }
    .build()
    .unwrap()
}

fn day(jd: f64) -> JulianDay<Utc> {
    JulianDay::<Utc>::try_new(jd).unwrap()
}

fn founded(sdk: &Context, instant: f64, place: Place) -> Document {
    sdk.chart()
        .reading(day(instant), &ChartRequest::at(place, UtcOffset::UTC))
        .unwrap()
        .value
}

fn longitude(chart: &Document, graha: Graha) -> f64 {
    chart.foundation.graha(graha).unwrap().longitude_deg
}

/// The arc from `a` to `b`, either way round, degrees.
fn apart(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

fn paris() -> Place {
    Place::new(
        Latitude::literal(48.8534),
        Longitude::literal(2.3488),
        Altitude::literal(0.0),
    )
}

/// The lunar returns of a radix at Paris, 1 January 2000 12h UT, as
/// pyswisseph's Moshier series times them (the recast that confirmed
/// Morin's figure; the built-in ephemeris begins in 1800, after it).
const MOSHIER: [f64; 3] = [2_451_572.291_914, 2_451_599.641_567, 2_451_627.016_457];

#[test]
fn a_lunar_return_is_the_one_the_recast_times() {
    let sdk = context(Some("western-tropical-default"));
    let radix = founded(&sdk, 2_451_545.0, paris());
    // Moshier's Moon at the radix: 223.323775°.
    assert!(apart(longitude(&radix, Graha::Moon), 223.323_775) < 2.0 / 3600.0);
    let returns = sdk
        .chart()
        .returns(&radix, day(2_451_546.0), day(2_451_630.0), [Graha::Moon])
        .unwrap()
        .value;
    assert_eq!(returns.len(), 3, "{returns:?}");
    for (one, moshier) in returns.iter().zip(MOSHIER) {
        assert_eq!((one.graha, one.motion), (Graha::Moon, Motion::Direct));
        let seconds = (one.at.get() - moshier).abs() * 86_400.0;
        assert!(seconds < 10.0, "{seconds} s from {moshier}");
    }
}

#[test]
fn every_return_of_a_lunar_year_is_on_the_radical_place() {
    let sdk = context(Some("western-tropical-default"));
    let radix = founded(&sdk, 2_451_545.0, paris());
    let radical = longitude(&radix, Graha::Moon);
    let returns = sdk
        .chart()
        .returns(
            &radix,
            day(2_451_546.0),
            day(2_451_546.0 + 365.25),
            [Graha::Moon],
        )
        .unwrap()
        .value;
    // A sidereal month of 27.32 days: thirteen in a year.
    assert_eq!(returns.len(), 13);
    for pair in returns.windows(2) {
        let month = pair[1].at.get() - pair[0].at.get();
        assert!((27.0..27.7).contains(&month), "{month}");
    }
    for one in &returns {
        let chart = founded(&sdk, one.at.get(), paris());
        assert!(apart(longitude(&chart, Graha::Moon), radical) < 1.0 / 3600.0);
    }
}

#[test]
fn a_retrograde_planet_returns_three_times_and_another_place_is_not_a_return() {
    let sdk = context(Some("western-tropical-default"));
    let radix = founded(&sdk, 2_451_545.0, paris());
    // Saturn's first return, about 29.5 years on; the Moon asked with it.
    let from = 2_451_545.0 + 28.0 * 365.25;
    let returns = sdk
        .chart()
        .returns(
            &radix,
            day(from),
            day(from + 3.0 * 365.25),
            [Graha::Saturn, Graha::Moon],
        )
        .unwrap()
        .value;
    let saturn: Vec<Motion> = returns
        .iter()
        .filter(|one| one.graha == Graha::Saturn)
        .map(|one| one.motion)
        .collect();
    assert!(
        saturn == [Motion::Direct]
            || saturn == [Motion::Direct, Motion::Retrograde, Motion::Direct],
        "{saturn:?}"
    );
    // The Moon crosses Saturn's natal place every month, and none of it
    // is a return: every row is a graha back on its own place.
    for one in &returns {
        let chart = founded(&sdk, one.at.get(), paris());
        let back = longitude(&chart, one.graha);
        assert!(apart(back, longitude(&radix, one.graha)) < 1.0 / 3600.0);
    }
    assert!(
        returns
            .windows(2)
            .all(|pair| pair[0].at.get() <= pair[1].at.get())
    );
}

#[test]
fn the_suns_return_is_the_tajika_pravesha() {
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    for (profile, reading) in [
        (None, Reading::Sidereal),
        (Some("western-tropical-default"), Reading::Tropical),
    ] {
        let sdk = context(profile);
        let birth = 2_447_892.5;
        let radix = founded(&sdk, birth, kathmandu);
        let years = sdk.chart().praveshas(&radix, reading, 3).unwrap();
        let returns = sdk
            .chart()
            .returns(
                &radix,
                day(birth + 1.0),
                day(birth + 3.0 * 365.25 + 10.0),
                [Graha::Sun],
            )
            .unwrap()
            .value;
        assert_eq!(returns.len(), 3, "{reading:?}");
        for (one, year) in returns.iter().zip(&years) {
            let seconds = (one.at.get() - year.at.get()).abs() * 86_400.0;
            assert!(seconds < 1.0, "{reading:?} year {}: {seconds} s", year.year);
        }
    }
}

#[test]
fn a_return_is_refused_by_the_field_it_names() {
    let sdk = context(None);
    let radix = founded(&sdk, 2_451_545.0, paris());
    let backwards = sdk
        .chart()
        .returns(&radix, day(2_451_600.0), day(2_451_546.0), [Graha::Moon])
        .unwrap_err();
    assert_eq!(backwards.field(), Some("to"));
    let none = sdk
        .chart()
        .returns(&radix, day(2_451_546.0), day(2_451_600.0), [])
        .unwrap_err();
    assert_eq!(none.field(), Some("grahas"));
}
