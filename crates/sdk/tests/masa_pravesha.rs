//! The Maasa and Dina Pravesha through the façade: each month's and each
//! day's instant read back through a chart founded at it
//! (`docs/03-design/annual-chart.md`, *Tajika Nilakanthi*, Saṃjñā Tantra,
//! the monthly and daily entry, v. 1).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::tajika::{MaseshaReading, MaseshaRules, Reading, VarsheshaRules};
use teistro::{ChartRequest, Context, Document, Ephemeris, UtcOffset};

const BIRTH: f64 = 2_447_995.489_583_333_5;

fn context() -> Context {
    Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a built context")
}

fn request() -> ChartRequest {
    let place = Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(1400.0).unwrap(),
    );
    ChartRequest::at(place, UtcOffset::try_from_seconds(20_700).unwrap())
}

fn chart(sdk: &Context, at: f64) -> Document {
    sdk.chart()
        .reading(JulianDay::<Utc>::literal(at), &request())
        .unwrap()
        .value
}

/// The Sun's sidereal longitude in a chart founded at `at`.
fn sun(sdk: &Context, at: f64) -> f64 {
    chart(sdk, at)
        .foundation
        .graha(Graha::Sun)
        .unwrap()
        .longitude_deg
}

/// How far two longitudes are apart, degrees, across Aries.
fn apart(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

#[test]
fn each_month_opens_with_the_sun_a_sign_on_from_the_years() {
    let sdk = context();
    let natal = chart(&sdk, BIRTH);
    let born = natal.foundation.graha(Graha::Sun).unwrap().longitude_deg;
    let year = sdk
        .chart()
        .praveshas(&natal, Reading::Sidereal, 30)
        .unwrap()
        .pop()
        .unwrap();
    let months = sdk
        .chart()
        .masa_praveshas(&natal, Reading::Sidereal, 30)
        .unwrap();
    assert_eq!(
        months.iter().map(|one| one.month).collect::<Vec<_>>(),
        (1..=12).collect::<Vec<_>>()
    );
    assert_eq!(
        months[0].at, year.at,
        "the first month is the year's own chart"
    );
    for (index, month) in months.iter().enumerate() {
        assert_eq!(month.year, 30);
        let target = born + 30.0 * f64::from(month.month - 1);
        assert!(apart(month.sun_deg, target) < 1e-9, "month {}", month.month);
        let read = sun(&sdk, month.at.get());
        assert!(
            apart(read, target) < 1e-4,
            "month {} reads {read}",
            month.month
        );
        if let Some(next) = months.get(index + 1) {
            let days = next.at.get() - month.at.get();
            assert!(
                (29.0..32.0).contains(&days),
                "month {}: {days} days",
                month.month
            );
        }
    }
    // The twelfth month ends where the next year's return opens.
    let next = sdk
        .chart()
        .masa_praveshas(&natal, Reading::Sidereal, 31)
        .unwrap();
    let last = months[11].at.get();
    assert!((29.0..32.0).contains(&(next[0].at.get() - last)));
}

#[test]
fn each_day_opens_with_the_sun_a_degree_on_from_the_months() {
    let sdk = context();
    let natal = chart(&sdk, BIRTH);
    let days = sdk
        .chart()
        .dina_praveshas(&natal, Reading::Sidereal, 30, 4)
        .unwrap();
    let month = sdk
        .chart()
        .masa_praveshas(&natal, Reading::Sidereal, 30)
        .unwrap()[3];
    assert_eq!(
        days.iter().map(|one| one.day).collect::<Vec<_>>(),
        (1..=30).collect::<Vec<_>>()
    );
    assert_eq!(
        days[0].at, month.at,
        "the first day is the month's own chart"
    );
    for (index, day) in days.iter().enumerate() {
        assert_eq!((day.year, day.month), (30, 4));
        let target = month.sun_deg + f64::from(day.day - 1);
        assert!(
            apart(sun(&sdk, day.at.get()), target) < 1e-4,
            "day {}",
            day.day
        );
        if let Some(next) = days.get(index + 1) {
            let apart_days = next.at.get() - day.at.get();
            assert!(
                (0.95..1.06).contains(&apart_days),
                "day {}: {apart_days}",
                day.day
            );
        }
    }
}

#[test]
fn a_tropical_year_divides_on_the_tropical_sun() {
    let sdk = context();
    let natal = chart(&sdk, BIRTH);
    let tropical = sdk
        .chart()
        .masa_praveshas(&natal, Reading::Tropical, 10)
        .unwrap();
    let sidereal = sdk
        .chart()
        .masa_praveshas(&natal, Reading::Sidereal, 10)
        .unwrap();
    assert_eq!(tropical.len(), 12);
    // Ten years on, the two returns stand hours apart, and so do their months.
    let hours = (tropical[5].at.get() - sidereal[5].at.get()).abs() * 24.0;
    assert!(hours > 1.0, "{hours} hours");
}

#[test]
fn what_cannot_be_divided_is_refused_by_name() {
    let sdk = context();
    let natal = chart(&sdk, BIRTH);
    let area = sdk.chart();
    let field = |error: teistro::Error| error.field().map(str::to_owned);
    assert_eq!(
        field(area.masa_praveshas(&natal, Reading::Mean, 3).unwrap_err()).as_deref(),
        Some("reading")
    );
    assert_eq!(
        field(
            area.masa_praveshas(&natal, Reading::Sidereal, 0)
                .unwrap_err()
        )
        .as_deref(),
        Some("year")
    );
    assert_eq!(
        field(
            area.dina_praveshas(&natal, Reading::Sidereal, 3, 13)
                .unwrap_err()
        )
        .as_deref(),
        Some("month")
    );
    // Past the two hundred years every return is counted to.
    assert_eq!(
        field(
            area.masa_praveshas(&natal, Reading::Sidereal, 201)
                .unwrap_err()
        )
        .as_deref(),
        Some("year")
    );
}

#[test]
fn each_month_has_its_lord_and_the_first_is_the_years() {
    let sdk = context();
    let natal = chart(&sdk, BIRTH);
    let months = sdk
        .chart()
        .masa_praveshas(&natal, Reading::Sidereal, 30)
        .unwrap();
    let year = chart(&sdk, months[0].at.get());
    let year_lord = sdk
        .chart()
        .varshesha(&natal, &year, 30, VarsheshaRules::default())
        .unwrap();
    for entry in &months {
        let month = chart(&sdk, entry.at.get());
        let lord = sdk
            .chart()
            .masesha(&natal, &year, &month, entry, MaseshaRules::default())
            .unwrap();
        assert_eq!(lord.reading, MaseshaReading::Chain);
        assert_eq!(lord.graha, lord.chain.graha);
        let rashi = teistro::catalogue::Rashi::of_longitude;
        assert_eq!(
            lord.bearers.masa_lagna,
            rashi(month.foundation.lagna_deg).attributes().lord
        );
        assert_eq!(
            lord.bearers.varsha_lagna,
            rashi(year.foundation.lagna_deg).attributes().lord
        );
        if entry.month == 1 {
            assert_eq!(lord.graha, year_lord.graha, "the first month is the year");
        }
        let others = sdk
            .chart()
            .masesha(
                &natal,
                &year,
                &month,
                entry,
                MaseshaRules {
                    reading: MaseshaReading::LagnaLord,
                    ..MaseshaRules::default()
                },
            )
            .unwrap();
        assert_eq!(others.graha, lord.bearers.masa_lagna);
    }
    // The year's chart handed in as the third month's is refused by name.
    let refused = sdk
        .chart()
        .masesha(&natal, &year, &year, &months[2], MaseshaRules::default())
        .unwrap_err();
    assert_eq!(refused.field(), Some("month"));
}
