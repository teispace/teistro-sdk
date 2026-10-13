//! The twilight births of BPHS ch. 9 v. 13 and Brihat Jataka ch. 6 v. 1,
//! read on real skies: the night a chart carries, measured from the day's
//! own sunset and next sunrise, and the two shipped rules that read its
//! edges.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail by panicking"
)]

use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Ephemeris, RuleRequest, ShippedRules, UtcOffset};

fn sdk() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the built-in ephemeris")
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::try_new(27.7172).unwrap(),
        Longitude::try_new(85.324).unwrap(),
        Altitude::try_new(0.0).unwrap(),
    )
}

/// The keys of the rules that hold at each instant, in order.
fn held(sdk: &Context, instants: &[f64], rules: &RuleRequest) -> Vec<Vec<String>> {
    let instants: Vec<JulianDay<Utc>> = instants
        .iter()
        .map(|jd| JulianDay::<Utc>::literal(*jd))
        .collect();
    let request = ChartRequest::at(kathmandu(), UtcOffset::UTC);
    sdk.chart()
        .readings_with_rules(&instants, &request, &rules.rule_set().unwrap())
        .unwrap()
        .value
        .iter()
        .map(|(_, read)| {
            read.present
                .iter()
                .map(|held| held.rule.key.clone())
                .collect()
        })
        .collect()
}

/// A day at Kathmandu read every ten minutes against a rule of its own that
/// holds in either twilight: it holds in two runs, one after sunset and one
/// before sunrise, each three ghatikas (72 minutes) long, so the night
/// reaches a birth before dawn as well as one after dusk.
#[test]
fn a_day_has_two_twilights_of_three_ghatikas() {
    let rules = RuleRequest::from_json(
        r#"{"rules": [{
            "key": "TWILIGHT", "category": "test",
            "source": {"text": "BPHS", "chapter": "9", "verse": "14", "rank": 1},
            "conditions": [{"type": "or", "conditions": [
                {"type": "at-limb-edge", "limb": "night", "edge": "first", "ghatikas": 3.0},
                {"type": "at-limb-edge", "limb": "night", "edge": "last", "ghatikas": 3.0}
            ]}]
        }]}"#,
    )
    .unwrap();
    let step = 10.0 / 1440.0;
    // From 06:00 UT, before Kathmandu's sunset, to 06:00 the next day.
    let instants: Vec<f64> = (0..144)
        .map(|n| 2_460_616.75 + f64::from(n) * step)
        .collect();
    let twilight: Vec<bool> = held(&sdk(), &instants, &rules)
        .iter()
        .map(|keys| keys.iter().any(|key| key == "TWILIGHT"))
        .collect();
    let runs: Vec<usize> = twilight
        .chunk_by(|a, b| a == b)
        .filter(|run| run.first() == Some(&true))
        .map(<[bool]>::len)
        .collect();
    // 72 minutes sampled every ten is seven or eight samples.
    assert_eq!(runs.len(), 2, "{twilight:?}");
    assert!(runs.iter().all(|run| (7..=8).contains(run)), "{runs:?}");
    assert_eq!(
        (twilight.first(), twilight.last()),
        (Some(&false), Some(&false)),
        "noon is not a twilight"
    );
}

/// Found by reading three months of Kathmandu's twilights every two
/// minutes: on the evening of 2024 November 1 the Moon's hora rose in the
/// twilight at a gandanta, and on 2024 October 17 the lunar hora
/// rose in the twilight with a malefic in a sign's last navamsa. An hour
/// and a half later the night is past its twilight, and neither holds.
#[test]
fn the_twilight_evils_hold_in_the_twilight_and_not_after_it() {
    let rules = RuleRequest::shipped([ShippedRules::Arishtas]);
    let found = held(
        &sdk(),
        &[2_460_616.01, 2_460_616.07, 2_460_601.021, 2_460_601.08],
        &rules,
    );
    let has = |n: usize, key: &str| {
        found
            .get(n)
            .is_some_and(|keys| keys.iter().any(|held| held == key))
    };
    let bphs = "ARISHTA_TWILIGHT_MOONS_HORA_AND_GANDANTA";
    let bj = "BJ_TWILIGHT_LUNAR_HORA_MALEFIC_IN_LAST_NAVAMSA";
    assert!(has(0, bphs) && !has(1, bphs));
    assert!(has(2, bj) && !has(3, bj));
}
