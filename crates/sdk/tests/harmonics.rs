//! A chart's harmonics (`03-design/western-harmonics.md`, C252 to C254):
//! Churchill's 9th harmonic as Addey reads it (*Harmonics in Astrology*,
//! pp. 97–98) on the built-in ephemeris, and the 9th harmonic of a
//! sidereal chart read back as the navamsa the SDK's own vargas give.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::{Graha, Rashi, Varga};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    ChartRequest, Context, Document, Ephemeris, HarmonicChart, HarmonicPoint, HarmonicRequest,
    UtcOffset,
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

fn founded(sdk: &Context, instant: f64, request: &ChartRequest) -> Document {
    let instant = JulianDay::<Utc>::try_new(instant).unwrap();
    sdk.chart().reading(instant, request).unwrap().value
}

const fn graha(graha: Graha) -> HarmonicPoint {
    HarmonicPoint::Graha { graha }
}

fn house(chart: &HarmonicChart, point: HarmonicPoint) -> u8 {
    chart
        .points
        .iter()
        .find(|one| one.point == point)
        .unwrap()
        .house
        .get()
}

#[test]
fn churchills_ninth_harmonic_reads_as_addey_read_it() {
    // Blenheim, 30 November 1874, "about 2 minutes before" 1.30 a.m.
    // local (p. 98, note 5): 1.33 a.m. at Greenwich.
    let sdk = context(Some("western-tropical-default"));
    let blenheim = Place::new(
        Latitude::literal(51.8414),
        Longitude::literal(-1.3611),
        Altitude::literal(0.0),
    );
    let request = ChartRequest::at(blenheim, UtcOffset::UTC).with_outer_planets();
    let chart = founded(&sdk, 2_405_857.564_892, &request);
    let ninth = sdk
        .chart()
        .harmonic(&chart, &HarmonicRequest::of(9))
        .unwrap();
    assert_eq!(ninth.harmonic, 9);
    // The Moon on Saturn in the third, 160° apart in the radix.
    let row = ninth
        .rows
        .iter()
        .find(|row| (row.first, row.second) == (graha(Graha::Moon), graha(Graha::Saturn)))
        .unwrap();
    assert!(row.apart_deg < 0.6, "{}", row.apart_deg);
    assert_eq!(row.multiple, 4);
    assert_eq!(house(&ninth, graha(Graha::Moon)), 3);
    // Venus rising and Pluto in the tenth.
    assert_eq!(house(&ninth, graha(Graha::Venus)), 1);
    assert_eq!(house(&ninth, graha(Graha::Pluto)), 10);
    // Ten planets and the two angles.
    assert_eq!(ninth.points.len(), 12);
}

#[test]
fn a_sidereal_ninth_harmonic_is_the_navamsa() {
    // Under the default sidereal profile every graha's and the lagna's
    // 9th-harmonic sign is the sign the SDK's own D9 gives it.
    let sdk = context(None);
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let mut instant = 2_451_545.0;
    for _ in 0..40 {
        let request =
            ChartRequest::at(kathmandu, UtcOffset::literal(5, 45, 0)).with_vargas([Varga::D9]);
        let chart = founded(&sdk, instant, &request);
        let ninth = sdk
            .chart()
            .harmonic(&chart, &HarmonicRequest::of(9))
            .unwrap();
        let navamsa = chart
            .vargas
            .iter()
            .find(|one| one.axis.grahas.varga == Some(Varga::D9))
            .unwrap();
        let sign_of = |point: HarmonicPoint| {
            Rashi::of_longitude(
                ninth
                    .points
                    .iter()
                    .find(|one| one.point == point)
                    .unwrap()
                    .longitude_deg,
            )
        };
        for placed in &navamsa.grahas {
            if matches!(placed.graha, Graha::Rahu | Graha::Ketu) {
                continue;
            }
            assert_eq!(
                sign_of(graha(placed.graha)),
                placed.at.sign,
                "{instant} {:?}",
                placed.graha
            );
        }
        assert_eq!(
            sign_of(HarmonicPoint::Ascendant),
            navamsa.lagna.sign,
            "{instant}"
        );
        instant += 97.31;
    }
}
