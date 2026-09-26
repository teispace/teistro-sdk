//! Jaimini's significators through the façade: the settings reach them,
//! and what the verses cannot find is said rather than guessed
//! (`docs/03-design/jaimini-significators.md`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Graha;
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
