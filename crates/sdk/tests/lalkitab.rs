//! Lal Kitab through the façade: a chart's teva read off its lagna and
//! grahas, and the life the kernel reads from it
//! (`docs/03-design/lalkitab.md`).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking on what they asked for"
)]

use teistro::catalogue::{Graha, Rashi};
use teistro::lalkitab::tables::PLANETS;
use teistro::lalkitab::{CycleStart, LifeRules, Teva, life};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    ChartRequest, Context, Document, Ephemeris, LalKitabRequest, UtcOffset, VarshphalRows,
};

/// Kathmandu, 14 April 1990, the corpus's first chart.
const BIRTH: f64 = 2_447_995.489_583_333_5;

fn sdk() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn chart(sdk: &Context) -> Document {
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let request = ChartRequest::at(place, UtcOffset::literal(5, 45, 0));
    sdk.chart()
        .reading(JulianDay::<Utc>::literal(BIRTH), &request)
        .unwrap()
        .value
}

/// A list with the book's structure and none of its numbers: year `y`
/// sends natal house `h` to `h + y − 1`, round the twelve.
fn rotation() -> VarshphalRows {
    let rows = (0..120_u8)
        .map(|year| core::array::from_fn(|column| (u8::try_from(column).unwrap() + year) % 12 + 1))
        .collect();
    VarshphalRows { rows }
}

#[test]
fn a_charts_teva_is_its_whole_sign_houses_from_the_lagna() {
    let sdk = sdk();
    let document = chart(&sdk);
    let read = sdk
        .chart()
        .lalkitab(&document, &LalKitabRequest::default())
        .unwrap();
    // The chart's own signs, counted from the lagna's, by hand.
    let foundation = &document.foundation;
    let lagna = Rashi::of_longitude(foundation.lagna_deg);
    let index = |sign: Rashi| Rashi::ALL.iter().position(|each| *each == sign).unwrap();
    for planet in &read.reading.planets {
        let sign = Rashi::of_longitude(foundation.graha(planet.graha).unwrap().longitude_deg);
        let house = (index(sign) + 12 - index(lagna)) % 12 + 1;
        assert_eq!(usize::from(planet.house), house, "{:?}", planet.graha);
    }
    assert_eq!(read.reading.planets.len(), PLANETS.len());
    assert_eq!(read.cycle, CycleStart::GENERAL);
    // The cycle's periods cover every year of life once, in order.
    assert_eq!(read.periods.first().unwrap().from, 1);
    assert_eq!(read.periods.last().unwrap().to, 120);
    for pair in read.periods.windows(2) {
        assert_eq!(pair[0].to + 1, pair[1].from);
    }
    assert_eq!(read.year, None);
}

#[test]
fn a_year_is_read_with_its_ruler_and_the_annual_teva_a_list_gives() {
    let sdk = sdk();
    let document = chart(&sdk);
    let asked = LalKitabRequest {
        cycle: CycleStart::new(Graha::Venus, 17).unwrap(),
        year: Some(43),
        varshphal: Some(rotation()),
    };
    let read = sdk.chart().lalkitab(&document, &asked).unwrap();
    let year = read.year.unwrap();
    // Venus from the 17th year gives the 43rd to Jupiter (1952 p. ~219).
    assert_eq!(year.ruler, Graha::Jupiter);
    assert_eq!(year.thirds, [Graha::Ketu, Graha::Jupiter, Graha::Sun]);
    // Under the rotation every planet moves 42 houses on, so 6.
    let annual = year.annual.unwrap();
    for (natal, moved) in read.reading.planets.iter().zip(&annual.planets) {
        assert_eq!(moved.graha, natal.graha);
        assert_eq!(moved.house, (natal.house + 42 - 1) % 12 + 1);
    }
    // The façade reads what the kernel reads from the same teva.
    let teva = Teva::from_houses(read.reading.planets.iter().map(|p| (p.graha, p.house))).unwrap();
    let kernel = life(
        &teva,
        &LifeRules {
            cycle: asked.cycle,
            year: asked.year,
            varshphal: None,
        },
    )
    .unwrap();
    assert_eq!(kernel.reading, read.reading);
    assert_eq!(kernel.periods, read.periods);
}

#[test]
fn a_refusal_is_named_under_the_record() {
    let sdk = sdk();
    let document = chart(&sdk);
    let refused = |asked: LalKitabRequest| {
        sdk.chart()
            .lalkitab(&document, &asked)
            .unwrap_err()
            .field()
            .map(str::to_owned)
    };
    let year = LalKitabRequest {
        year: Some(121),
        ..LalKitabRequest::default()
    };
    assert_eq!(refused(year).as_deref(), Some("lalkitab.year"));
    let mut list = rotation();
    list.rows[13] = list.rows[12];
    let echoed = LalKitabRequest {
        varshphal: Some(list),
        ..LalKitabRequest::default()
    };
    assert_eq!(
        refused(echoed).as_deref(),
        Some("lalkitab.varshphal.rows[13][0]")
    );
    let cycle = LalKitabRequest::from_json(r#"{"cycle": {"planet": "VENUS", "year": 0}}"#).unwrap();
    assert_eq!(refused(cycle).as_deref(), Some("lalkitab.cycle.year"));
    let uranus =
        LalKitabRequest::from_json(r#"{"cycle": {"planet": "URANUS", "year": 1}}"#).unwrap();
    assert_eq!(refused(uranus).as_deref(), Some("lalkitab.cycle.planet"));
}
