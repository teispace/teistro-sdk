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
use teistro::tajika::{
    MasaDashaRules, MasaDashaSign, MasaPhalaRules, MaseshaReading, MaseshaRules, MonthClock,
    Reading, VarsheshaRules,
};
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

#[test]
fn a_place_founds_each_month_and_carries_its_lord() {
    let sdk = context();
    let natal = chart(&sdk, BIRTH);
    let clock = UtcOffset::try_from_seconds(20_700).unwrap();
    let asked =
        teistro::MasaPraveshaRequest::from_json(r#"{"year": 30, "place": "birth"}"#).unwrap();
    let rows = sdk.chart().masa_pravesha(&natal, clock, &asked).unwrap();
    let months = sdk
        .chart()
        .masa_praveshas(&natal, Reading::Sidereal, 30)
        .unwrap();
    assert_eq!(rows.len(), 12);
    let year = chart(&sdk, months[0].at.get());
    for (row, entry) in rows.iter().zip(&months) {
        assert_eq!(row.entry, teistro::tajika::DinaPravesha::from(*entry));
        let month = chart(&sdk, entry.at.get());
        let lord = sdk
            .chart()
            .masesha(&natal, &year, &month, entry, MaseshaRules::default())
            .unwrap();
        assert_eq!(row.masesha.as_ref(), Some(&lord), "month {}", entry.month);
    }
    // Without a place the rows are the instants alone.
    let bare = teistro::MasaPraveshaRequest::new(30);
    let rows = sdk.chart().masa_pravesha(&natal, clock, &bare).unwrap();
    assert!(rows.iter().all(|row| row.masesha.is_none()));

    // The section's wire spelling, camelCase to the leaves.
    let wire = serde_json::to_value(&rows_with_lords(&sdk, &natal, clock)[0]).unwrap();
    assert!(wire["masesha"]["bearers"]["masaLagna"].is_string());
    assert!(wire["masesha"]["chain"]["moonPassedOver"].is_boolean());
    assert!(wire["masesha"]["chain"]["claims"][0]["aspectsLagna"].is_boolean());
}

/// v. 14: each month's dasha runs the five Huddas of its stronger lord's
/// sign from the month's opening to the next's, a Hudda's degree a
/// degree of the Sun, and is what the façade reads from the month's
/// charts and knots.
#[test]
fn each_months_dasha_runs_its_huddas_on_the_suns_degrees() {
    let sdk = context();
    let natal = chart(&sdk, BIRTH);
    let clock = UtcOffset::try_from_seconds(20_700).unwrap();
    let rows = rows_with_lords(&sdk, &natal, clock);
    let months = sdk
        .chart()
        .masa_praveshas(&natal, Reading::Sidereal, 30)
        .unwrap();
    let year = chart(&sdk, months[0].at.get());
    for (index, (row, entry)) in rows.iter().zip(&months).enumerate() {
        let dasha = row.dasha.as_ref().unwrap();
        let knots = sdk
            .chart()
            .month_knots(&natal, Reading::Sidereal, entry)
            .unwrap();
        assert_eq!(knots.len(), 31);
        let month = chart(&sdk, entry.at.get());
        let read = sdk
            .chart()
            .masa_dasha(
                &natal,
                &year,
                &month,
                entry,
                &knots,
                MaseshaRules::default().muntha,
                MasaDashaRules::default(),
            )
            .unwrap();
        assert_eq!(dasha, &read, "month {}", entry.month);
        let lord = row.masesha.as_ref().unwrap();
        assert!(
            [lord.bearers.muntha, lord.bearers.masa_lagna].contains(&dasha.graha),
            "the Muntha's lord or the lagna's"
        );
        assert_eq!(
            dasha
                .periods
                .iter()
                .map(|one| u32::from(one.degrees))
                .sum::<u32>(),
            30
        );
        let (first, last) = (&dasha.periods[0], &dasha.periods[4]);
        assert_eq!(first.starts, entry.at);
        if let Some(next) = months.get(index + 1) {
            assert!(
                (last.ends.get() - next.at.get()).abs() < 1e-6,
                "ends as the next opens"
            );
        }
        // A period's edge is where the Sun has moved its degrees on.
        let moved = sun(&sdk, first.ends.get());
        assert!(apart(moved, entry.sun_deg + f64::from(first.degrees)) < 1e-4);
    }
    // Thirty civil days need no search, and end thirty days on.
    let days = teistro::MasaPraveshaRequest::new(30)
        .at(teistro::AnnualPlace::Birth)
        .with_dasha(MasaDashaRules {
            clock: MonthClock::Days,
            ..MasaDashaRules::default()
        });
    let rows = sdk.chart().masa_pravesha(&natal, clock, &days).unwrap();
    let dasha = rows[3].dasha.as_ref().unwrap();
    assert!((dasha.periods[4].ends.get() - months[3].at.get() - 30.0).abs() < 1e-9);
    // The record's wire spelling, and its readings by name.
    let asked = teistro::MasaPraveshaRequest::from_json(
        r#"{"year": 30, "place": "birth", "dasha": {"sign": "OWNED", "clock": "EVEN", "table": "CHARAK"}}"#,
    )
    .unwrap();
    assert_eq!(asked.dasha.sign, MasaDashaSign::Owned);
    let wire = serde_json::to_value(&rows[0]).unwrap();
    assert!(wire["dasha"]["office"].is_string());
    assert!(wire["dasha"]["periods"][0]["degrees"].is_number());
    assert_eq!(wire["dasha"]["rules"]["clock"], "DAYS");
    let refused =
        teistro::MasaPraveshaRequest::from_json(r#"{"year": 30, "dasha": {"clock": "HOURS"}}"#)
            .unwrap_err();
    assert!(refused.field().unwrap().starts_with("masaPravesha.dasha"));
}

fn rows_with_lords(sdk: &Context, natal: &Document, clock: UtcOffset) -> Vec<teistro::MasaRow> {
    let asked = teistro::MasaPraveshaRequest::new(30).at(teistro::AnnualPlace::Birth);
    sdk.chart().masa_pravesha(natal, clock, &asked).unwrap()
}

#[test]
fn each_months_results_are_read_from_its_chart_beside_its_lord() {
    let sdk = context();
    let natal = chart(&sdk, BIRTH);
    let clock = UtcOffset::try_from_seconds(20_700).unwrap();
    let rows = rows_with_lords(&sdk, &natal, clock);
    let months = sdk
        .chart()
        .masa_praveshas(&natal, Reading::Sidereal, 30)
        .unwrap();
    let year = chart(&sdk, months[0].at.get());
    let year_lord = sdk
        .chart()
        .varshesha(&natal, &year, 30, VarsheshaRules::default())
        .unwrap()
        .graha;
    let rules = MasaPhalaRules::default();
    for (row, entry) in rows.iter().zip(&months) {
        let month = chart(&sdk, entry.at.get());
        let lord = row.masesha.as_ref().unwrap();
        let results = sdk
            .chart()
            .masa_phala(&month, lord, year_lord, rules)
            .unwrap();
        assert_eq!(
            row.results.as_ref(),
            Some(&results),
            "month {}",
            entry.month
        );
        assert_eq!(results.houses.len(), 12);
        // The first house is v. 7's: the lagna's own navamsha lord.
        assert_eq!(
            results.houses[0].amsha_lord,
            teistro::tajika::navamsha_lord(month.foundation.lagna_deg)
        );
        let offices: Vec<_> = results.lords.iter().map(|one| one.office).collect();
        assert_eq!(offices, teistro::tajika::MonthLord::ALL);
        assert_eq!(results.lords[1].graha, lord.graha);
        assert_eq!(results.lords[2].graha, year_lord);
    }
    // Another month's lord handed in with this month's chart is refused
    // when the two lagnas have different lords.
    let first = rows[0].masesha.as_ref().unwrap();
    let wrong = months
        .iter()
        .map(|entry| chart(&sdk, entry.at.get()))
        .find(|month| {
            teistro::catalogue::Rashi::of_longitude(month.foundation.lagna_deg)
                .attributes()
                .lord
                != first.bearers.masa_lagna
        });
    if let Some(month) = wrong {
        let refused = sdk
            .chart()
            .masa_phala(&month, first, year_lord, rules)
            .unwrap_err();
        assert_eq!(refused.field(), Some("month"));
    }
    // The floors are refused by their wire name under the record.
    let refused = teistro::MasaPraveshaRequest::from_json(
        r#"{"year": 30, "place": "birth", "results": {"weakBelow": 9000, "strongFrom": 10}}"#,
    )
    .unwrap_err();
    assert_eq!(refused.field(), Some("masaPravesha.results.strongFrom"));
    let wire = serde_json::to_value(&rows[0]).unwrap();
    assert!(wire["results"]["houses"][0]["amshaLord"].is_string());
    assert!(wire["results"]["houses"][0]["seenByMoon"].is_array());
    assert!(wire["results"]["lords"][0]["office"].is_string());
}

#[test]
fn a_place_beside_a_month_founds_each_day_its_lord_and_its_results() {
    let sdk = context();
    let natal = chart(&sdk, BIRTH);
    let clock = UtcOffset::try_from_seconds(20_700).unwrap();
    let asked =
        teistro::MasaPraveshaRequest::from_json(r#"{"year": 30, "month": 4, "place": "birth"}"#)
            .unwrap();
    let rows = sdk.chart().masa_pravesha(&natal, clock, &asked).unwrap();
    assert_eq!(rows.len(), 30);
    let months = sdk
        .chart()
        .masa_praveshas(&natal, Reading::Sidereal, 30)
        .unwrap();
    let year = chart(&sdk, months[0].at.get());
    let month = chart(&sdk, months[3].at.get());
    let month_lord = sdk
        .chart()
        .masesha(&natal, &year, &month, &months[3], MaseshaRules::default())
        .unwrap();
    let year_lord = sdk
        .chart()
        .varshesha(&natal, &year, 30, VarsheshaRules::default())
        .unwrap()
        .graha;
    for row in &rows {
        assert!(row.masesha.is_none() && row.results.is_none());
        let lord = row.dinesha.as_ref().unwrap();
        let day = chart(&sdk, row.entry.at.get());
        let read = sdk
            .chart()
            .dinesha(
                &natal,
                &year,
                &month,
                &day,
                &row.entry,
                MaseshaRules::default(),
            )
            .unwrap();
        assert_eq!(lord, &read, "day {}", row.entry.day);
        assert_eq!(lord.graha, lord.chain.graha);
        assert_eq!(lord.bearers.masa_lagna, month_lord.bearers.masa_lagna);
        assert_eq!(lord.bearers.varsha_lagna, month_lord.bearers.varsha_lagna);
        let results = sdk
            .chart()
            .dina_phala(
                &day,
                lord,
                year_lord,
                month_lord.graha,
                MasaPhalaRules::default(),
            )
            .unwrap();
        assert_eq!(
            row.day_results.as_ref(),
            Some(&results),
            "day {}",
            row.entry.day
        );
        let offices: Vec<_> = results.lords.iter().map(|one| one.graha).collect();
        assert_eq!(
            offices,
            [lord.graha, year_lord, month_lord.graha, lord.bearers.muntha]
        );
    }
    // The Moon walks some thirteen degrees a day, so the thirty days
    // pass through more than one of her states.
    let states: std::collections::BTreeSet<_> = rows
        .iter()
        .map(|row| row.day_results.as_ref().unwrap().moon_state)
        .collect();
    assert!(states.len() > 1);
    // The first day is the month's own chart, so its lord is the month's.
    assert_eq!(rows[0].dinesha.as_ref().unwrap().graha, month_lord.graha);
    // The month's chart handed in as the fifth day's is refused by name.
    let refused = sdk
        .chart()
        .dinesha(
            &natal,
            &year,
            &month,
            &month,
            &rows[4].entry,
            MaseshaRules::default(),
        )
        .unwrap_err();
    assert_eq!(refused.field(), Some("day"));
    let wire = serde_json::to_value(&rows[1]).unwrap();
    assert!(wire["dinesha"]["bearers"]["dinaLagna"].is_string());
    assert!(wire["dayResults"]["moonState"].is_string());
    assert_eq!(wire["dayResults"]["houses"].as_array().unwrap().len(), 12);
    assert!(wire.get("masesha").is_none());
}
