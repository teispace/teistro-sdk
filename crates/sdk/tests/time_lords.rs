//! Valens's time lords through the façade: releasing from the lots and the
//! profected year, asked for as dashas and read back as their periods, and
//! a stored document's cursor rebuilt from its own first sign
//! (`docs/03-design/hellenistic-time-lords.md`).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    reason = "tests fail by panicking, index what they found, and compare whole days"
)]

use teistro::catalogue::{DashaSystem, Rashi};
use teistro::dasha::{DashaReading, Timeline, releasing_years};
use teistro::quantity::{Altitude, Depth, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, Lot, UtcOffset};

const TIME_LORDS: [DashaSystem; 3] = [
    DashaSystem::ReleasingFortune,
    DashaSystem::ReleasingDaimon,
    DashaSystem::Profection,
];

fn tropical() -> Context {
    Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .settings_json(r#"{"frame": {"zodiac": "TROPICAL"}}"#)
        .build()
        .unwrap()
}

/// London at `jd`, with the three time lords asked for.
fn chart(sdk: &Context, jd: f64) -> Document {
    let place = Place::new(
        Latitude::literal(51.5),
        Longitude::literal(-0.12),
        Altitude::literal(0.0),
    );
    let request = ChartRequest::at(place, UtcOffset::UTC).with_dashas(TIME_LORDS);
    sdk.chart()
        .reading(JulianDay::<Utc>::literal(jd), &request)
        .unwrap()
        .value
}

fn reading(document: &Document, system: DashaSystem) -> &DashaReading {
    document
        .dashas
        .iter()
        .find(|reading| reading.system == system)
        .unwrap()
}

/// The sign a lot fell in.
fn lot_sign(sdk: &Context, document: &Document, lot: Lot) -> Rashi {
    sdk.chart().lots(document, &[lot]).unwrap().lots[0]
        .place
        .sign
}

#[test]
fn the_time_lords_start_where_valens_counts_them() {
    let sdk = tropical();
    let document = chart(&sdk, 2_451_545.25);
    let birth = document.foundation.instant.get();

    // Releasing begins at Fortune's sign, at birth, each first-level sign
    // its years of 360 days (IV.4, IV.9).
    let fortune = reading(&document, DashaSystem::ReleasingFortune);
    assert_eq!(
        fortune.start_sign(),
        Some(lot_sign(&sdk, &document, Lot::Fortune))
    );
    assert_eq!(fortune.rules.year_length.days(), 360.0);
    let first: Vec<_> = fortune
        .periods
        .iter()
        .filter(|row| row.level() == 1)
        .collect();
    assert_eq!(first.len(), 12);
    assert_eq!(first[0].interval.from.get(), birth);
    for row in &first {
        let years = f64::from(releasing_years(row.sign.unwrap()));
        assert!(
            (row.interval.days() - years * 360.0).abs() < 1e-6,
            "{row:?}"
        );
    }

    // The profected year begins at the Ascendant's sign, one sign a year
    // of 365¼ days, one level deep (IV.11).
    let profection = reading(&document, DashaSystem::Profection);
    let ascendant = Rashi::of_longitude(sdk.chart().angles(&document).unwrap().ascendant_deg);
    assert_eq!(profection.start_sign(), Some(ascendant));
    assert_eq!(profection.periods.len(), 12);
    assert_eq!(profection.depth, Depth::MIN);
    assert!((profection.periods[0].interval.days() - 365.25).abs() < 1e-6);

    // A cursor rebuilt from the document answers what its rows say, and
    // the profection keeps counting past the twelve years it carries.
    for system in TIME_LORDS {
        let cursor = sdk.chart().dasha(&document, system).unwrap();
        let rows = &reading(&document, system).periods;
        for row in rows.iter().filter(|row| row.level() == 1) {
            let middle = f64::midpoint(row.interval.from.get(), row.interval.to.get());
            let chain = cursor.at(JulianDay::literal(middle), Depth::MIN);
            assert_eq!(chain.iter().next().unwrap().sign, row.sign, "{system:?}");
        }
    }
    let cursor = sdk
        .chart()
        .dasha(&document, DashaSystem::Profection)
        .unwrap();
    let thirty_fifth = JulianDay::literal(birth + 34.5 * 365.25);
    let year = cursor.at(thirty_fifth, Depth::MIN);
    let profected = cursor.profection().unwrap().sign_of_year(35);
    assert_eq!(year.iter().next().unwrap().sign, profected);
}

/// At a new moon Fortune and Daimon share the Ascendant's sign, and Valens
/// reads activity from the sign after it (IV.4, crux C223).
#[test]
fn daimon_in_fortunes_sign_releases_from_the_next() {
    let sdk = tropical();
    // The new moon of 6 January 2000, 18:14 UTC.
    let document = chart(&sdk, 2_451_550.26);
    let fortune = lot_sign(&sdk, &document, Lot::Fortune);
    assert_eq!(lot_sign(&sdk, &document, Lot::Daimon), fortune);
    let daimon = reading(&document, DashaSystem::ReleasingDaimon);
    let next = Rashi::of_longitude(fortune.start_deg() + 30.0);
    assert_eq!(daimon.start_sign(), Some(next));
    assert_eq!(
        reading(&document, DashaSystem::ReleasingFortune).start_sign(),
        Some(fortune)
    );
}
