//! Jaimini's significators through the façade: the settings reach them,
//! and what the verses cannot find is said rather than guessed
//! (`docs/03-design/jaimini-significators.md`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "tests fail by panicking and index what they found"
)]

use teistro::catalogue::{DashaSystem, Graha, Rashi};
use teistro::dasha::jaimini::NoBrahma;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::settings::BrahmaRule;
use teistro::{ChartRequest, Context, Document, Ephemeris, UtcOffset};

/// A Kathmandu birth, 14 April 1990, the corpus's first.
const BIRTH: f64 = 2_447_995.489_583_333_5;

fn context(patch: &str) -> Context {
    Context::builder()
        .settings_json(patch)
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
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(BIRTH),
            &ChartRequest::at(place, UtcOffset::literal(5, 45, 0)),
        )
        .unwrap()
        .value
}

#[test]
fn the_rule_the_settings_name_is_the_rule_that_answers() {
    for (patch, rule) in [
        ("{}", BrahmaRule::Verses),
        (
            r#"{"jaimini": {"brahma": "TRANSLATORS_NOTE"}}"#,
            BrahmaRule::TranslatorsNote,
        ),
    ] {
        let sdk = context(patch);
        let reading = sdk.chart().jaimini(&chart(&sdk)).unwrap();
        assert_eq!(reading.brahma.rule, rule);
        // Found, or refused with a reason: never neither, never both.
        assert_eq!(
            reading.brahma.graha.is_none(),
            reading.brahma.none.is_some()
        );
        // A found Brahma is one of the planets that met the marks, or the
        // heir of the Saturn or node that did.
        if let Some(graha) = reading.brahma.graha {
            let from = reading.brahma.passed_from.unwrap_or(graha);
            assert!(reading.brahma.qualified.contains(&from), "{reading:?}");
        }
    }
}

/// The reading's `jaimini` section is the answer `sdk.chart().jaimini` gives
/// the bare chart, under every rule and co-lordship; a stored document
/// answers with its own section; and a request for everything carries it.
#[test]
fn the_section_is_the_answer_and_a_stored_chart_keeps_its_own() {
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let request = ChartRequest::at(place, UtcOffset::literal(5, 45, 0));
    for patch in [
        "{}",
        r#"{"jaimini": {"brahma": "TRANSLATORS_NOTE", "node_co_lordship": "BOTH"}}"#,
    ] {
        let sdk = context(patch);
        for step in 0..12 {
            let at = JulianDay::<Utc>::literal(BIRTH + f64::from(step) * 0.37);
            let bare = sdk.chart().reading(at, &request).unwrap().value;
            assert!(bare.jaimini.is_none());
            let read = sdk
                .chart()
                .reading(at, &request.clone().with_jaimini())
                .unwrap()
                .value;
            let section = read.jaimini.clone().unwrap();
            assert_eq!(section, sdk.chart().jaimini(&bare).unwrap(), "{patch}");
            assert!(read.sections().contains(&"jaimini"));
            // Read back under other settings, a stored chart keeps the
            // answer it was cast with.
            let other =
                context(r#"{"jaimini": {"chara_karakas": "EIGHT", "brahma": "TRANSLATORS_NOTE"}}"#);
            assert_eq!(other.chart().jaimini(&read).unwrap(), section);
        }
    }
    let sdk = context("{}");
    let everything = sdk
        .chart()
        .reading(JulianDay::<Utc>::literal(BIRTH), &request.with_everything())
        .unwrap()
        .value;
    assert!(everything.jaimini.is_some());
}

#[test]
fn the_karakamsha_is_the_atmakarakas_navamsha_under_either_scheme() {
    for patch in [
        r#"{"jaimini": {"chara_karakas": "SEVEN"}}"#,
        r#"{"jaimini": {"chara_karakas": "EIGHT"}}"#,
    ] {
        let sdk = context(patch);
        let document = chart(&sdk);
        let reading = sdk.chart().jaimini(&document).unwrap();
        let karakamsha = reading.karakamsha;
        // The Atmakaraka's own navamsha is the karakamsha, so it is the 1st
        // from it in the navamsha.
        assert_eq!(
            karakamsha
                .in_navamsha
                .get(karakamsha.atmakaraka as usize)
                .copied(),
            Some(1)
        );
        // Eight karakas may rank Rahu; seven never do.
        if patch.contains("SEVEN") {
            assert_ne!(karakamsha.atmakaraka, Graha::Rahu);
        }
    }
}

#[test]
fn every_reading_of_the_dual_lords_answers() {
    for co in ["NONE", "STRONGER_LORD", "BOTH"] {
        let sdk = context(&format!(r#"{{"jaimini": {{"node_co_lordship": "{co}"}}}}"#));
        let reading = sdk.chart().jaimini(&chart(&sdk)).unwrap();
        if let Some(why) = reading.brahma.none {
            assert!(
                matches!(
                    why,
                    NoBrahma::NoLordQualifies | NoBrahma::NoPlanetInTheSixth
                ),
                "{why:?} is the note's reason, not the verses'"
            );
        }
    }
}

/// The Sthira dasa starts from the Brahma graha's sign and runs forward,
/// seven, eight or nine years a sign by modality; over a chart the verses
/// find no Brahma for, asking for it is refused on the knob that supplies
/// one (BPHS ch. 46 vv. 168 to 173).
#[test]
fn the_sthira_dasa_starts_from_brahma_or_refuses_by_name() {
    let sdk = context("{}");
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let request = ChartRequest::at(place, UtcOffset::literal(5, 45, 0));
    let (mut found, mut refused) = (false, false);
    for step in 0..200 {
        let at = JulianDay::<Utc>::literal(BIRTH + f64::from(step) * 0.37);
        let plain = sdk.chart().reading(at, &request).unwrap().value;
        let brahma = sdk.chart().jaimini(&plain).unwrap().brahma;
        let with_sthira = sdk
            .chart()
            .reading(at, &request.clone().with_dashas([DashaSystem::Sthira]));
        if let Some(graha) = brahma.graha {
            found = true;
            let document = with_sthira.unwrap().value;
            let sthira = document.dashas.first().unwrap();
            let mahadashas: Vec<_> = sthira
                .periods
                .iter()
                .filter(|row| !row.path.contains('/'))
                .collect();
            let signs: Vec<Rashi> = mahadashas.iter().filter_map(|row| row.sign).collect();
            let start = plain.foundation.graha(graha).unwrap().longitude_deg;
            let expected_start = Rashi::ALL[(start / 30.0) as usize % 12];
            assert_eq!(signs.first().copied(), Some(expected_start));
            for pair in signs.windows(2) {
                assert_eq!(
                    (pair[1] as usize + 12 - pair[0] as usize) % 12,
                    1,
                    "{signs:?}"
                );
            }
            // Seven, eight and nine by modality: the spans' ratios.
            let days = |row: &&teistro::PeriodRow| row.interval.to.get() - row.interval.from.get();
            let years = |sign: Rashi| f64::from([7, 8, 9][sign as usize % 3]);
            let unit = days(&mahadashas[0]) / years(signs[0]);
            for (row, sign) in mahadashas.iter().zip(&signs) {
                assert!((days(row) / years(*sign) - unit).abs() < 1e-6, "{sign:?}");
            }
        } else {
            refused = true;
            let why = with_sthira.unwrap_err();
            assert!(why.to_string().contains("Brahma"), "{why}");
            assert!(why.to_string().contains("TRANSLATORS_NOTE"), "{why}");
            // A request for everything still answers such a chart: it asks
            // for every system but those a chart may refuse.
            let everything = sdk
                .chart()
                .reading(at, &request.clone().with_everything())
                .unwrap()
                .value;
            let asked: Vec<DashaSystem> = everything
                .dashas
                .iter()
                .filter_map(|dasha| dasha.system.catalogued())
                .collect();
            let every: Vec<DashaSystem> = teistro::dasha::systems()
                .filter(|system| *system != DashaSystem::Sthira)
                .collect();
            assert_eq!(asked, every);
        }
        if found && refused {
            return;
        }
    }
    panic!("200 charts gave found={found} refused={refused}; the test needs both");
}
