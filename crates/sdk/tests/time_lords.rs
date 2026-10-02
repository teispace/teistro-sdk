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

use teistro::catalogue::{DashaSystem, Graha, Rashi};
use teistro::dasha::{DashaReading, Timeline, releasing_years};
use teistro::quantity::{Altitude, Depth, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    ChartRequest, Context, Document, Ephemeris, FortuneRule, Lot, LotPoint, LotRequest, UtcOffset,
};

const TIME_LORDS: [DashaSystem; 3] = [
    DashaSystem::ReleasingFortune,
    DashaSystem::ReleasingDaimon,
    DashaSystem::Profection,
];

fn tropical() -> Context {
    tropical_with("")
}

/// The tropical conformance profile, with a dasha group patch.
fn tropical_with(dasha: &str) -> Context {
    let patch = if dasha.is_empty() {
        r#"{"frame": {"zodiac": "TROPICAL"}}"#.to_owned()
    } else {
        format!(r#"{{"frame": {{"zodiac": "TROPICAL"}}, "dasha": {dasha}}}"#)
    };
    Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .settings_json(&patch)
        .build()
        .unwrap()
}

/// London at `jd`, with the three time lords asked for.
fn chart(sdk: &Context, jd: f64) -> Document {
    chart_under(sdk, jd, LotRequest::VALENS)
}

/// London at `jd`, its time lords released from lots read under `lots`.
fn chart_under(sdk: &Context, jd: f64, lots: LotRequest) -> Document {
    let place = Place::new(
        Latitude::literal(51.5),
        Longitude::literal(-0.12),
        Altitude::literal(0.0),
    );
    let request = ChartRequest::at(place, UtcOffset::UTC)
        .with_dashas(TIME_LORDS)
        .with_lot_rules(lots);
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
    assert_eq!(
        profection.periods.len(),
        usize::from(teistro::dasha::PROFECTION_YEARS)
    );
    assert_eq!(profection.depth, Depth::MIN);
    assert!((profection.periods[0].interval.days() - 365.25).abs() < 1e-6);

    // A cursor rebuilt from the document answers what its rows say, and
    // the profection keeps counting past the 120 years it carries.
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
    for ordinal in [35_u32, 121] {
        let into = (f64::from(ordinal) - 0.5) * 365.25;
        let year = cursor.at(JulianDay::literal(birth + into), Depth::MIN);
        let profected = cursor.profection().unwrap().sign_of_year(ordinal);
        assert_eq!(year.iter().next().unwrap().sign, profected, "{ordinal}");
    }
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

/// C223's other reading: a consumer who keeps the shared sign gets
/// Daimon's own.
#[test]
fn the_shared_sign_can_be_kept() {
    let sdk = tropical_with(r#"{"releasing_shared_sign": "SAME"}"#);
    let document = chart(&sdk, 2_451_550.26);
    let fortune = lot_sign(&sdk, &document, Lot::Fortune);
    let daimon = reading(&document, DashaSystem::ReleasingDaimon);
    assert_eq!(daimon.start_sign(), Some(fortune));
}

/// The lots releasing starts from are read under the request's rules, so
/// a night birth whose Fortune moves sign under Lilly's rule releases
/// from where Lilly puts it.
#[test]
fn releasing_reads_the_lots_under_the_requests_rules() {
    let sdk = tropical();
    // 1 January 2000, 18:00 UTC: night in London.
    let jd = 2_451_545.25;
    let lilly = LotRequest::VALENS.with_fortune(FortuneRule::DayAndNight);
    let valens = chart(&sdk, jd);
    let document = chart_under(&sdk, jd, lilly);
    let under = |request: LotRequest| {
        sdk.chart()
            .lots_with_request(&document, &[Lot::Fortune], request)
            .unwrap()
            .lots[0]
            .place
            .sign
    };
    assert_ne!(
        under(lilly),
        under(LotRequest::VALENS),
        "the test must bite"
    );
    let fortune = reading(&document, DashaSystem::ReleasingFortune);
    assert_eq!(fortune.start_sign(), Some(under(lilly)));
    assert_eq!(
        reading(&valens, DashaSystem::ReleasingFortune).start_sign(),
        Some(under(LotRequest::VALENS))
    );
}

/// Profection from a point: from the Ascendant it is the catalogue's
/// `PROFECTION` row for row, and from the Moon it starts at the Moon's
/// sign (IV.11's "every point").
#[test]
fn a_year_profects_from_any_point() {
    let sdk = tropical();
    let document = chart(&sdk, 2_451_545.25);
    let stored = reading(&document, DashaSystem::Profection);
    let from_ascendant = sdk
        .chart()
        .profection_from(&document, LotPoint::Ascendant, LotRequest::VALENS)
        .unwrap();
    assert_eq!(
        DashaReading::of_time_lord(
            DashaSystem::Profection,
            &from_ascendant,
            stored.rules,
            stored.depth
        ),
        *stored
    );
    let moon = LotPoint::Planet(Graha::Moon);
    let from_moon = sdk
        .chart()
        .profection_from(&document, moon, LotRequest::VALENS)
        .unwrap();
    let moon_sign = sdk
        .chart()
        .point_place(&document, moon, LotRequest::VALENS)
        .unwrap()
        .sign;
    assert_eq!(from_moon.start(), moon_sign);
    assert_ne!(
        from_moon.start(),
        from_ascendant.start(),
        "the test must bite"
    );
    // A planet outside the seven is refused by the point.
    let refused = sdk
        .chart()
        .profection_from(&document, LotPoint::Planet(Graha::Rahu), LotRequest::VALENS)
        .unwrap_err();
    assert_eq!(refused.field(), Some("point"));
}

/// London at `jd`, its firdaria asked for under `lots`.
fn firdaria_under(sdk: &Context, jd: f64, lots: LotRequest) -> Document {
    let place = Place::new(
        Latitude::literal(51.5),
        Longitude::literal(-0.12),
        Altitude::literal(0.0),
    );
    let request = ChartRequest::at(place, UtcOffset::UTC)
        .with_dashas([DashaSystem::Firdaria])
        .with_lot_rules(lots);
    sdk.chart()
        .reading(JulianDay::<Utc>::literal(jd), &request)
        .unwrap()
        .value
}

/// The first-level lords a reading stores, in order.
fn firdars(reading: &DashaReading) -> Vec<Graha> {
    reading
        .periods
        .iter()
        .filter(|row| row.level() == 1)
        .map(|row| row.lord)
        .collect()
}

/// The firdaria begin from the Sun by day and the Moon by night (al-Biruni
/// §395), the sect read under the request's lot rules; each firdar is
/// shared out in sevenths, two levels stored, and a stored document's
/// cursor answers what its rows say.
#[test]
fn the_firdaria_begin_from_the_luminary_of_the_sect() {
    use teistro::SectRule;
    use teistro::dasha::{FIRDARIA_PERIODS, FIRDARIA_ROUNDS, firdar_years};

    let sdk = tropical();
    // 1 January 2000, 18:00 UTC: night in London.
    let jd = 2_451_545.25;
    let night = firdaria_under(&sdk, jd, LotRequest::VALENS);
    let reading_of = |document: &Document| reading(document, DashaSystem::Firdaria).clone();
    let stored = reading_of(&night);
    assert_eq!(stored.first_lord, Graha::Moon);
    let lords = firdars(&stored);
    assert_eq!(lords.len(), FIRDARIA_PERIODS * FIRDARIA_ROUNDS);
    assert_eq!(
        lords[..FIRDARIA_PERIODS],
        [
            Graha::Moon,
            Graha::Saturn,
            Graha::Jupiter,
            Graha::Mars,
            Graha::Sun,
            Graha::Venus,
            Graha::Mercury,
            Graha::Rahu,
            Graha::Ketu,
        ]
    );
    assert_eq!(stored.depth.get(), 2);
    let year = stored.rules.year_length.days();
    assert!((year - 365.25).abs() < 1e-9);
    for row in stored.periods.iter().filter(|row| row.level() == 1) {
        let years = f64::from(firdar_years(row.lord));
        assert!((row.interval.days() - years * year).abs() < 1e-6, "{row:?}");
    }
    // The Moon's sevenths, from the Moon in descending order; the nodes'
    // firdars are not shared.
    let sevenths: Vec<_> = stored
        .periods
        .iter()
        .filter(|row| row.path.starts_with("0/"))
        .map(|row| row.lord)
        .collect();
    assert_eq!(sevenths[0], Graha::Moon);
    assert_eq!(sevenths[1], Graha::Saturn);
    assert_eq!(sevenths.len(), 7);
    for node in ["7/", "8/"] {
        assert!(!stored.periods.iter().any(|row| row.path.starts_with(node)));
    }

    // Forced to day, the same instant begins from the Sun.
    let day = firdaria_under(&sdk, jd, LotRequest::VALENS.with_sect_rule(SectRule::Day));
    assert_eq!(firdars(&reading_of(&day))[0], Graha::Sun);

    // A cursor rebuilt from either document answers its rows.
    for document in [&night, &day] {
        let cursor = sdk.chart().dasha(document, DashaSystem::Firdaria).unwrap();
        for row in &reading_of(document).periods {
            let middle = f64::midpoint(row.interval.from.get(), row.interval.to.get());
            let depth = Depth::try_new(u8::try_from(row.level()).unwrap()).unwrap();
            let chain = cursor.at(JulianDay::literal(middle), depth);
            assert_eq!(chain.iter().last().unwrap().lord, row.lord, "{row:?}");
        }
    }
}

/// C225's other reading: Bonatti's nodes after Mars by night, kept when a
/// stored document is rebuilt under a context that says otherwise.
#[test]
fn the_nodes_can_follow_mars() {
    let bonatti = tropical_with(r#"{"firdaria_nodes": "AFTER_MARS"}"#);
    let document = firdaria_under(&bonatti, 2_451_545.25, LotRequest::VALENS);
    let stored = reading(&document, DashaSystem::Firdaria);
    assert_eq!(
        firdars(stored)[3..6],
        [Graha::Mars, Graha::Rahu, Graha::Ketu]
    );
    let cursor = tropical()
        .chart()
        .dasha(&document, DashaSystem::Firdaria)
        .unwrap();
    assert_eq!(
        cursor.firdaria().unwrap().order(),
        teistro::dasha::firdaria_order(Graha::Moon, teistro::settings::FirdariaNodes::AfterMars)
    );
}
