#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use std::collections::{BTreeSet, HashSet};

use teistro_core::catalogue::{Graha, Kaala, Nakshatra, Rashi, Tithi, Vara};
use teistro_core::interval::Interval;
use teistro_intl::Value;

use super::{Side, intents, of_window, watched};
use crate::activity::{ActivityRules, Bar};
use crate::clause::{Clause, ClauseKey, ClauseKind};
use crate::grade::Grade;
use crate::instant::{Limbs, Sky, clauses};
use crate::judge::Judgement;

const T: f64 = 2_460_000.5;

fn at(from: f64, to: f64) -> Interval {
    Interval::literal(T + from, T + to)
}

fn held(kind: ClauseKind) -> Clause {
    Clause {
        kind,
        at: at(0.0, 1.0),
    }
}

/// A window under Raman's marriage rules over the clauses given.
fn window(kinds: Vec<ClauseKind>) -> (Judgement, ActivityRules) {
    let rules = ActivityRules::raman_marriage();
    let clauses: Vec<Clause> = kinds.into_iter().map(held).collect();
    (Judgement::of(at(0.2, 0.3), &clauses, &rules), rules)
}

fn good_day() -> Vec<ClauseKind> {
    vec![
        ClauseKind::Vara {
            vara: Vara::Guruvara,
            grade: Grade::Best,
        },
        ClauseKind::Kaala {
            kaala: Kaala::GulikaKaala,
        },
        ClauseKind::Abhijit {},
        ClauseKind::Nakshatra {
            nakshatra: Nakshatra::Rohini,
            grade: Grade::Best,
        },
        ClauseKind::Tithi {
            tithi: Tithi::from_id(1).expect("a tithi"),
            grade: Grade::Best,
        },
    ]
}

/// The list runs `FOR`, then `DESPITE`, then `AVOIDED`, and the limbs lead
/// the `FOR` run in a panchanga's order whatever order they held in.
#[test]
fn the_list_runs_for_then_despite_then_avoided() {
    let (judgement, rules) = window(good_day());
    let said = of_window(&judgement, 0, &rules, &watched(&rules, false)).expect("reasons");
    let sides: Vec<Side> = said.reasons.iter().map(|r| r.side).collect();
    let mut sorted = sides.clone();
    sorted.sort_by_key(|s| *s as u8);
    assert_eq!(sides, sorted, "{sides:?}");
    let keys: Vec<ClauseKey> = said
        .reasons
        .iter()
        .filter(|r| r.side == Side::For)
        .map(|r| r.key)
        .collect();
    assert_eq!(
        keys,
        [
            ClauseKey::Nakshatra,
            ClauseKey::Tithi,
            ClauseKey::Vara,
            ClauseKey::Abhijit
        ]
    );
    let despite: Vec<ClauseKey> = said
        .reasons
        .iter()
        .filter(|r| r.side == Side::Despite)
        .map(|r| r.key)
        .collect();
    assert_eq!(despite, [ClauseKey::Kaala]);
}

/// An item that says a clause points at it in the window's list, and its
/// slots are the clause's members in full keys.
#[test]
fn a_said_clause_points_at_itself_and_carries_its_members() {
    let (judgement, rules) = window(good_day());
    let said = of_window(&judgement, 0, &rules, &watched(&rules, false)).expect("reasons");
    for reason in said.reasons.iter().filter(|r| r.side != Side::Avoided) {
        let index = reason.clause.expect("a held clause's index");
        assert_eq!(judgement.clauses[index].kind.key(), reason.key);
    }
    let star = said
        .reasons
        .iter()
        .find(|r| r.key == ClauseKey::Nakshatra)
        .expect("the star");
    assert_eq!(star.intent, "sdk.voice.muhurta.favours.NAKSHATRA");
    assert_eq!(
        star.params.get("nakshatra"),
        Some(&Value::Entity(String::from("nakshatra.ROHINI")))
    );
    assert_eq!(star.params.get("grade"), Some(&Value::from("BEST")));
    assert!(star.params.contains_key("variant"));
}

/// "Avoided" names each watched dosha that did not hold, and never one
/// that did, nor one that needs a native when none was read.
#[test]
fn avoided_names_only_watched_doshas_that_did_not_hold() {
    let mut kinds = good_day();
    kinds.push(ClauseKind::MarsInEighth {});
    let rules = ActivityRules::raman_marriage();
    let clauses: Vec<Clause> = kinds.into_iter().map(held).collect();
    // Mars in the eighth bars a marriage, so it is said as held, not avoided.
    let judgement = Judgement::of(at(0.2, 0.3), &clauses, &rules);
    let said = of_window(&judgement, 0, &rules, &watched(&rules, false)).expect("reasons");
    let avoided: HashSet<ClauseKey> = said
        .reasons
        .iter()
        .filter(|r| r.side == Side::Avoided)
        .map(|r| r.key)
        .collect();
    assert!(!avoided.contains(&ClauseKey::MarsInEighth));
    assert!(!avoided.contains(&ClauseKey::Kaala), "a kaala held");
    assert!(!avoided.contains(&ClauseKey::Ghata), "no native was read");
    assert!(avoided.contains(&ClauseKey::VenusInSixth));
    // Each is watched under these rules, or a bar the rite names.
    let watch = watched(&rules, false);
    assert!(avoided.iter().all(|key| {
        watch.contains(key)
            || rules.bars.iter().any(|bar| match bar {
                Bar::Key(k) => k == key,
                Bar::Clause(c) => c.key() == *key,
            })
    }));
}

/// A bar the rite names is avoided too, once, even when it is not watched.
#[test]
fn a_rites_bar_is_avoided_once() {
    let mut rules = ActivityRules::raman_marriage();
    rules.bars = vec![
        Bar::Key(ClauseKey::MoonInDusthana),
        Bar::Key(ClauseKey::SeventhOccupied),
        Bar::Clause(ClauseKind::Kaala {
            kaala: Kaala::RahuKaala,
        }),
    ];
    let clauses: Vec<Clause> = good_day().into_iter().map(held).collect();
    let judgement = Judgement::of(at(0.2, 0.3), &clauses, &rules);
    let said = of_window(&judgement, 0, &rules, &watched(&rules, false)).expect("reasons");
    let avoided: Vec<&str> = said
        .reasons
        .iter()
        .filter(|r| r.side == Side::Avoided)
        .map(|r| r.intent.as_str())
        .collect();
    let count = |intent: &str| avoided.iter().filter(|i| **i == intent).count();
    assert_eq!(count("sdk.voice.muhurta.avoided.MOON_IN_DUSTHANA"), 1);
    assert_eq!(count("sdk.voice.muhurta.avoided.SEVENTH_OCCUPIED"), 1);
    // Rahu kaala did not hold (Gulika did), so the clause-bar is avoided,
    // and said with its member.
    let rahu = said
        .reasons
        .iter()
        .find(|r| r.side == Side::Avoided && r.key == ClauseKey::Kaala)
        .expect("Rahu kaala avoided");
    assert_eq!(
        rahu.params.get("kaala"),
        Some(&Value::Entity(String::from("kaala.RAHU_KAALA")))
    );
}

/// The line names the qualifying limbs in a panchanga's order, and counts
/// what the list says.
#[test]
fn the_line_names_the_limbs_and_counts_the_list() {
    let (judgement, rules) = window(good_day());
    let said = of_window(&judgement, 3, &rules, &watched(&rules, false)).expect("reasons");
    let line = &said.line;
    assert_eq!(line.intent, "sdk.voice.muhurta.line");
    assert_eq!(
        line.params.get("limbs"),
        Some(&Value::List(vec![
            Value::catalogued(Nakshatra::Rohini),
            Value::catalogued(Tithi::from_id(1).expect("a tithi")),
            Value::catalogued(Vara::Guruvara),
        ]))
    );
    assert_eq!(line.params.get("count"), Some(&Value::Int(3)));
    assert_eq!(line.params.get("despite"), Some(&Value::Int(1)));
    let avoided = said
        .reasons
        .iter()
        .filter(|r| r.side == Side::Avoided)
        .count();
    assert_eq!(
        line.params.get("avoided"),
        Some(&Value::Int(i64::try_from(avoided).expect("small")))
    );
}

/// Neighbouring items and windows take different variants.
#[test]
fn neighbours_take_different_variants() {
    let (judgement, rules) = window(good_day());
    let watch = watched(&rules, false);
    let first = of_window(&judgement, 0, &rules, &watch).expect("reasons");
    let second = of_window(&judgement, 1, &rules, &watch).expect("reasons");
    let variant = |r: &super::Reason| r.params.get("variant").cloned();
    assert_ne!(variant(&first.reasons[0]), variant(&first.reasons[1]));
    assert_ne!(variant(&first.reasons[0]), variant(&second.reasons[0]));
}

/// Every intent a window can say is one `intents` lists: a sample of every
/// clause kind, each said on every side it can take.
#[test]
fn every_intent_said_is_listed() {
    let listed: BTreeSet<String> = intents().into_iter().collect();
    let skies = skies();
    let mut kinds: Vec<ClauseKind> = Vec::new();
    for (sky, limbs) in &skies {
        kinds.extend(
            clauses(sky, Some(limbs), Some(Rashi::Aries), at(0.0, 1.0))
                .into_iter()
                .map(|c| c.kind),
        );
    }
    kinds.extend(good_day());
    let (judgement, rules) = window(kinds);
    let said = of_window(&judgement, 0, &rules, &watched(&rules, true)).expect("reasons");
    for reason in &said.reasons {
        assert!(listed.contains(&reason.intent), "{}", reason.intent);
    }
    assert!(listed.contains(&said.line.intent));
    assert_eq!(listed.len(), intents().len(), "no intent listed twice");
}

/// Every watched dosha a window's sky decides is one the sky reports: a
/// grid of skies finds each of them held somewhere, so its absence in a
/// window is a check that was made. (The day's kaalas, ashubha yogas,
/// varjyam and ghatas are read off the day's own spans for every day,
/// `day::clauses`.)
#[test]
fn every_watched_dosha_of_the_sky_is_reported_somewhere() {
    let mut found = HashSet::new();
    for (sky, limbs) in skies() {
        for clause in clauses(&sky, Some(&limbs), Some(Rashi::Aries), at(0.0, 1.0)) {
            found.insert(clause.kind.key());
        }
    }
    let of_the_sky = [
        ClauseKey::Kartari,
        ClauseKey::MoonInDusthana,
        ClauseKey::MoonJoined,
        ClauseKey::VenusInSixth,
        ClauseKey::MarsInEighth,
        ClauseKey::Kunavamsa,
        ClauseKey::PanchakaRemainder,
        ClauseKey::MaleficInLagna,
        ClauseKey::AshtamaLagna,
    ];
    for key in of_the_sky {
        assert!(found.contains(&key), "{key:?} never reported");
    }
}

/// A deterministic grid of skies and limbs.
fn skies() -> Vec<(Sky, Limbs)> {
    let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut out = Vec::new();
    for _ in 0..400 {
        let mut grahas = [0.0; 9];
        for g in &mut grahas {
            *g = f64::from(u32::try_from(next() % 3600).expect("small")) / 10.0;
        }
        // The nodes stand opposite, as they do.
        grahas[Graha::Ketu as usize] = (grahas[Graha::Rahu as usize] + 180.0) % 360.0;
        let lagna_deg = f64::from(u32::try_from(next() % 3600).expect("small")) / 10.0;
        let limbs = Limbs {
            tithi: Tithi::from_id(u16::try_from(next() % 30).expect("small")).expect("a tithi"),
            vara: Vara::ALL[usize::try_from(next() % 7).expect("small")],
            nakshatra: Nakshatra::ALL[usize::try_from(next() % 27).expect("small")],
        };
        out.push((
            Sky {
                lagna_deg,
                grahas,
                speeds: [1.0; 9],
            },
            limbs,
        ));
    }
    out
}
