//! Two charts matched through their Moons (`03-design/matching.md`): the
//! natives read from each founded chart's own sidereal Moon, the kootas
//! the kernel's, and a tropical chart refused by the role it was given.

#![allow(
    clippy::unwrap_used,
    clippy::float_cmp,
    reason = "tests fail by panicking and compare exact halves"
)]

use teistro::catalogue::{Graha, Koota};
use teistro::matching::ashta_koota;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    ChartRequest, Context, Document, Ephemeris, KootaReading, KootaRules, Native, UtcOffset,
};

fn context(profile: Option<&str>) -> Context {
    let builder = Context::builder().ephemeris([Ephemeris::Builtin]);
    match profile {
        Some(profile) => builder.profile(profile),
        None => builder,
    }
    .build()
    .unwrap()
}

fn founded(sdk: &Context, instant: f64) -> Document {
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    sdk.chart()
        .reading(
            JulianDay::<Utc>::try_new(instant).unwrap(),
            &ChartRequest::at(kathmandu, UtcOffset::UTC),
        )
        .unwrap()
        .value
}

fn moon(chart: &Document) -> Native {
    Native::of_moon(chart.foundation.graha(Graha::Moon).unwrap().longitude_deg).unwrap()
}

#[test]
fn a_match_reads_each_charts_own_sidereal_moon() {
    let sdk = context(None);
    // Every other day for four weeks: the Moon through every sign.
    let bride = founded(&sdk, 2_447_892.5);
    for day in 0..14 {
        let groom = founded(&sdk, 2_451_545.0 + 2.0 * f64::from(day));
        let koota = sdk
            .chart()
            .matching(&bride, &groom, KootaRules::default())
            .unwrap();
        assert_eq!(
            koota,
            ashta_koota(moon(&bride), moon(&groom), KootaRules::default())
        );
        assert!((0.0..=36.0).contains(&koota.total));
        let Some(KootaReading::Tara {
            bride_to_groom,
            groom_to_bride,
        }) = koota.row(Koota::Tara).map(|row| row.reading)
        else {
            unreachable!()
        };
        assert!((1..=9).contains(&bride_to_groom) && (1..=9).contains(&groom_to_bride));
    }
}

#[test]
fn a_tropical_chart_is_refused_by_its_role() {
    let sidereal = founded(&context(None), 2_447_892.5);
    let western = context(Some("western-tropical-default"));
    let tropical = founded(&western, 2_451_545.0);
    let refused = western
        .chart()
        .matching(&sidereal, &tropical, KootaRules::default())
        .unwrap_err();
    assert_eq!(refused.field(), Some("groom"));
    let refused = western
        .chart()
        .matching(&tropical, &sidereal, KootaRules::default())
        .unwrap_err();
    assert_eq!(refused.field(), Some("bride"));
}
