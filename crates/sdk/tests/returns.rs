//! A body's returns (`03-design/western-returns.md`, C255 to C258):
//! the lunar return Morin defines (*Astrologia Gallica* XXIII, pp.
//! 633–634), timed as the recast that holds his Richelieu figure times it,
//! and the Sun's return read back as the Tajika pravesha, sidereal and
//! tropical; and the half- and quarter-returns (C433 to C437), each read
//! back on its offset from a chart founded at it.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Graha;
use teistro::gochar::hits::Motion;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::tajika::Reading;
use teistro::{ChartRequest, Context, Document, Ephemeris, OffsetReturn, UtcOffset};

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

/// The returns of `graha` at `offsets` over a window, as rows.
fn at_offsets(
    sdk: &Context,
    radix: &Document,
    (from, to): (f64, f64),
    graha: Graha,
    offsets: &[u16],
) -> Vec<OffsetReturn> {
    sdk.chart()
        .returns_at(radix, day(from), day(to), [graha], offsets.iter().copied())
        .unwrap()
        .value
}

#[test]
fn the_quarti_solar_returns_are_the_sun_at_its_quarters_a_season_apart() {
    let sdk = context(Some("western-tropical-default"));
    let radix = founded(&sdk, 2_451_545.0, paris());
    let natal = longitude(&radix, Graha::Sun);
    let window = (2_451_546.0, 2_451_546.0 + 366.0);
    let year = at_offsets(&sdk, &radix, window, Graha::Sun, &[0, 90, 180, 270]);
    // The circle in order from the birthday: the three quarters, then the
    // return a year on.
    let order: Vec<u16> = year.iter().map(|one| one.offset_deg).collect();
    assert_eq!(order, [90, 180, 270, 0], "{year:?}");
    for one in &year {
        assert_eq!((one.graha, one.motion), (Graha::Sun, Motion::Direct));
        let chart = founded(&sdk, one.at.get(), paris());
        let target = natal + f64::from(one.offset_deg);
        let off = apart(longitude(&chart, Graha::Sun), target) * 3600.0;
        assert!(off < 1.0, "{}°: {off}″ off", one.offset_deg);
    }
    // A season each, about a quarter of the year: longer through the
    // northern summer, when the Sun is farthest and slowest.
    let mut since = 2_451_545.0;
    for one in &year {
        let season = one.at.get() - since;
        assert!(
            (88.0..95.0).contains(&season),
            "{}°: {season} days",
            one.offset_deg
        );
        since = one.at.get();
    }
    // The demi-solar return asked alone is the 180° row of the four.
    let half = at_offsets(&sdk, &radix, window, Graha::Sun, &OffsetReturn::HALF);
    assert_eq!(half.len(), 1, "{half:?}");
    assert_eq!(half[0].offset_deg, 180);
    let seconds = (half[0].at.get() - year[1].at.get()).abs() * 86_400.0;
    assert!(seconds < 1e-3, "{seconds} s");
}

#[test]
fn offset_zero_is_the_return_instant_for_instant() {
    let sdk = context(Some("western-tropical-default"));
    let radix = founded(&sdk, 2_451_545.0, paris());
    let window = (2_451_546.0, 2_451_630.0);
    let returns = sdk
        .chart()
        .returns(&radix, day(window.0), day(window.1), [Graha::Moon])
        .unwrap()
        .value;
    let at_zero = at_offsets(&sdk, &radix, window, Graha::Moon, &[0]);
    assert_eq!(returns.len(), at_zero.len());
    for (one, zero) in returns.iter().zip(&at_zero) {
        assert_eq!(
            (one.graha, one.at.get().to_bits(), one.motion),
            (zero.graha, zero.at.get().to_bits(), zero.motion)
        );
        assert_eq!(zero.offset_deg, 0);
    }
}

#[test]
fn the_moons_quarter_returns_are_her_month_in_four() {
    let sdk = context(Some("western-tropical-default"));
    let radix = founded(&sdk, 2_451_545.0, paris());
    let natal = longitude(&radix, Graha::Moon);
    let offsets: Vec<u16> = [0].into_iter().chain(OffsetReturn::QUARTERS).collect();
    // A sidereal month from a day after birth: each quarter once, in the
    // order the circle passes them, the return last (Moshier's first).
    let month = at_offsets(
        &sdk,
        &radix,
        (2_451_546.0, 2_451_546.0 + 27.0),
        Graha::Moon,
        &offsets,
    );
    let order: Vec<u16> = month.iter().map(|one| one.offset_deg).collect();
    assert_eq!(order, [90, 180, 270, 0], "{month:?}");
    for pair in month.windows(2) {
        // A quarter of 27.32 days, the Moon's speed ranging 12° to 15° a day.
        let quarter = pair[1].at.get() - pair[0].at.get();
        assert!((5.5..8.5).contains(&quarter), "{quarter} days");
    }
    for one in &month {
        let chart = founded(&sdk, one.at.get(), paris());
        let target = natal + f64::from(one.offset_deg);
        assert!(apart(longitude(&chart, Graha::Moon), target) < 1.0 / 3600.0);
    }
}

#[test]
fn an_offset_is_refused_by_the_field_it_names() {
    let sdk = context(None);
    let radix = founded(&sdk, 2_451_545.0, paris());
    let ask = |grahas: &[Graha], offsets: &[u16], to: f64| {
        sdk.chart()
            .returns_at(
                &radix,
                day(2_451_546.0),
                day(to),
                grahas.iter().copied(),
                offsets.iter().copied(),
            )
            .unwrap_err()
    };
    let full_circle = ask(&[Graha::Sun], &[360], 2_451_600.0);
    assert_eq!(full_circle.field(), Some("offsets"));
    assert!(
        full_circle
            .hint()
            .is_some_and(|hint| hint.contains("write 0"))
    );
    assert_eq!(
        ask(&[Graha::Sun], &[], 2_451_600.0).field(),
        Some("offsets")
    );
    assert_eq!(
        ask(&[Graha::Sun], &[180, 180], 2_451_600.0).field(),
        Some("offsets")
    );
    // The window and the grahas are refused as the return's are.
    assert_eq!(ask(&[Graha::Sun], &[180], 2_451_500.0).field(), Some("to"));
    assert_eq!(ask(&[], &[180], 2_451_600.0).field(), Some("grahas"));
}
