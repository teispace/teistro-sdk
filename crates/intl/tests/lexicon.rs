//! `intl.lexicon` and `intl.lookup`: every member's names in a locale, and
//! the members a text may name (`docs/03-design/lexicon.md` §1, A2-3).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use std::time::Instant;

use teistro_core::Status;
use teistro_intl::lexicon::{Lexicon, LexiconRequest, Lookup, LookupRequest, fold};
use teistro_intl::source::Tree;
use teistro_intl::{Intl, sdk_root};

fn intl() -> Intl {
    Intl::from_tree(&Tree::load(&sdk_root()).expect("the sources")).expect("the engine")
}

fn lexicon(intl: &Intl, json: &str) -> Lexicon {
    intl.lexicon(&LexiconRequest::from_json(json).unwrap())
        .unwrap()
}

fn lookup(intl: &Intl, json: &str) -> Lookup {
    intl.lookup(&LookupRequest::from_json(json).unwrap())
        .unwrap()
}

/// Every span's text and its candidates' keys.
fn found(answer: &Lookup) -> Vec<(String, Vec<String>)> {
    answer
        .matches
        .iter()
        .map(|m| {
            (
                m.text.clone(),
                m.candidates.iter().map(|c| c.key.clone()).collect(),
            )
        })
        .collect()
}

fn keys(answer: &Lookup) -> Vec<Vec<String>> {
    found(answer).into_iter().map(|(_, keys)| keys).collect()
}

#[test]
fn every_named_member_is_listed_in_every_locale() {
    let intl = intl();
    let english = lexicon(&intl, r#"{"locale": "en-Latn"}"#);
    for tag in ["ne-Deva-NP", "hi-Deva-IN", "sa-Deva", "sa-Latn"] {
        let other = lexicon(&intl, &format!(r#"{{"locale": "{tag}"}}"#));
        assert_eq!(other.locale, tag);
        // A fallback to English names what the locale does not: the
        // lexicon is the chain's, so every member English names is there.
        assert_eq!(other.entries.len(), english.entries.len(), "{tag}");
    }
    assert!(english.entries.len() > 900, "{}", english.entries.len());
    // The catalogue's order: the Sun first.
    assert_eq!(english.entries[0].key, "graha.SUN");
}

#[test]
fn a_devanagari_name_carries_its_romanised_forms() {
    let intl = intl();
    let nepali = lexicon(&intl, r#"{"locale": "ne", "kinds": ["vara"]}"#);
    let monday = nepali
        .entries
        .iter()
        .find(|e| e.key == "vara.SOMAVARA")
        .unwrap();
    let forms: Vec<(&str, &str)> = monday
        .forms
        .iter()
        .map(|f| (f.form.as_str(), f.text.as_str()))
        .collect();
    assert_eq!(
        forms,
        [
            ("name", "सोमबार"),
            ("short", "सोम"),
            ("iast", "Somavāra"),
            ("romanised", "Somavara"),
            ("romanised", "Somavar"),
        ]
    );
    assert!(monday.forms.iter().all(|f| f.locale == "ne-Deva-NP"));
}

#[test]
fn a_fallback_says_which_locale_gave_the_form() {
    let intl = intl();
    // Hindi's packs name no festival kind, so English answers.
    let hindi = lexicon(
        &intl,
        r#"{"locale": "hi-Deva-IN", "kinds": ["festival_kind"]}"#,
    );
    assert_ne!(hindi.entries.len(), 0);
    for entry in &hindi.entries {
        assert!(
            entry.forms.iter().all(|f| f.locale == "en-Latn"),
            "{entry:?}"
        );
    }
}

#[test]
fn a_name_is_found_in_any_script_and_any_spelling() {
    let intl = intl();
    for text in [
        "today is Ekadashi",
        "aaja ekadasi ho",
        "आज एकादशी हो",
        "आज एकादशी है",
        "ekādaśī",
        "EKAADASHI",
    ] {
        let answer = lookup(
            &intl,
            &format!(r#"{{"text": "{text}", "kinds": ["tithi"]}}"#),
        );
        assert_eq!(
            keys(&answer),
            [["tithi.KRISHNA_EKADASHI", "tithi.SHUKLA_EKADASHI"]],
            "{text}"
        );
    }
}

#[test]
fn a_fused_postposition_is_read_off() {
    let intl = intl();
    let answer = lookup(
        &intl,
        r#"{"text": "चन्द्रमा रोहिणीमा छ, सोमबारदेखि", "kinds": ["nakshatra", "vara"]}"#,
    );
    assert_eq!(
        found(&answer),
        [
            (
                String::from("रोहिणी"),
                vec![String::from("nakshatra.ROHINI")]
            ),
            (String::from("सोमबार"), vec![String::from("vara.SOMAVARA")]),
        ]
    );
    // Scalar values: रोहिणी begins after चन्द्रमा and a space.
    assert_eq!(answer.matches[0].start, "चन्द्रमा ".chars().count());
    assert_eq!(
        answer.matches[0].end - answer.matches[0].start,
        "रोहिणी".chars().count()
    );
}

#[test]
fn the_longest_name_wins_and_spans_words() {
    let intl = intl();
    let answer = lookup(
        &intl,
        r#"{"text": "Moon in Purva Phalguni", "kinds": ["nakshatra", "graha"]}"#,
    );
    assert_eq!(
        found(&answer),
        [
            (String::from("Moon"), vec![String::from("graha.MOON")]),
            (
                String::from("Purva Phalguni"),
                vec![String::from("nakshatra.PURVA_PHALGUNI")]
            ),
        ]
    );
}

#[test]
fn an_ambiguous_span_answers_every_member() {
    let intl = intl();
    let answer = lookup(
        &intl,
        r#"{"text": "श्रावण", "kinds": ["masa", "nakshatra"]}"#,
    );
    assert_eq!(keys(&answer), [["masa.SHRAVANA", "nakshatra.SHRAVANA"]]);
    // Each candidate says which forms it met: with no locale asked, any
    // locale's form whose fold is the span's.
    let via = &answer.matches[0].candidates[0].via;
    assert_ne!(via.len(), 0);
    assert!(
        via.iter().all(|f| fold(&f.text) == fold("श्रावण")),
        "{via:?}"
    );
}

#[test]
fn a_locale_narrows_the_names_searched() {
    let intl = intl();
    // `सोमवार` is Hindi's spelling; Nepali writes `सोमबार`. The fold makes
    // them one (व and ब), so either locale finds both.
    for tag in ["ne-Deva-NP", "hi-Deva-IN"] {
        let answer = lookup(
            &intl,
            &format!(r#"{{"text": "सोमवार", "locale": "{tag}", "kinds": ["vara"]}}"#),
        );
        assert_eq!(keys(&answer), [["vara.SOMAVARA"]], "{tag}");
    }
    // A locale's chain gives each member the first name it has: Nepali
    // names Monday, so under `ne` the English name is not searched, and
    // with no locale it is.
    let under_nepali = lookup(
        &intl,
        r#"{"text": "Monday", "locale": "ne", "kinds": ["vara"]}"#,
    );
    assert!(under_nepali.matches.is_empty(), "{under_nepali:?}");
    let anywhere = lookup(&intl, r#"{"text": "Monday", "kinds": ["vara"]}"#);
    assert_eq!(keys(&anywhere), [["vara.SOMAVARA"]]);
}

#[test]
fn a_refusal_names_its_field() {
    let intl = intl();
    let kind = intl
        .lookup(&LookupRequest::from_json(r#"{"text": "x", "kinds": ["planet"]}"#).unwrap())
        .unwrap_err();
    assert_eq!(kind.status, Status::InvalidArg);
    assert_eq!(kind.field(), Some("lookup.kinds"));
    let locale = intl
        .lexicon(&LexiconRequest::from_json(r#"{"locale": "de"}"#).unwrap())
        .unwrap_err();
    assert_eq!(locale.status, Status::Unsupported);
    assert_eq!(locale.field(), Some("lexicon.locale"));
}

#[test]
fn text_naming_nothing_answers_nothing() {
    let intl = intl();
    for text in ["", "   ", "१२३ 456 !?", "the quick brown fox"] {
        let answer = lookup(
            &intl,
            &format!(r#"{{"text": "{text}", "kinds": ["tithi", "nakshatra"]}}"#),
        );
        assert!(answer.matches.is_empty(), "{text}: {answer:?}");
    }
}

/// The collisions the fold makes among one kind's members in the shipped
/// locales, listed so that a new one is a decision, not an accident
/// (`docs/03-design/lexicon.md` §1.3).
#[test]
fn the_folds_collisions_are_the_ones_listed() {
    let intl = intl();
    let english = lexicon(
        &intl,
        r#"{"locale": "en-Latn", "kinds": ["graha", "rashi", "nakshatra", "masa", "vara", "paksha"]}"#,
    );
    let mut by_fold: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();
    for entry in &english.entries {
        for form in entry.forms.iter().filter(|f| f.form != "short") {
            by_fold
                .entry(fold(&form.text))
                .or_default()
                .insert(entry.key.clone());
        }
    }
    let collisions: Vec<(String, Vec<String>)> = by_fold
        .into_iter()
        .filter(|(_, keys)| keys.len() > 1)
        .map(|(folded, keys)| (folded, keys.into_iter().collect()))
        .collect();
    let listed: Vec<(&str, Vec<&str>)> = collisions
        .iter()
        .map(|(f, k)| (f.as_str(), k.iter().map(String::as_str).collect()))
        .collect();
    assert_eq!(
        listed,
        [
            ("jest", vec!["masa.JYESHTHA", "nakshatra.JYESHTHA"]),
            ("mag", vec!["masa.MAGHA", "nakshatra.MAGHA"]),
            ("sraban", vec!["masa.SHRAVANA", "nakshatra.SHRAVANA"]),
        ],
        "a collision the design does not list"
    );
}

/// The lexicon is built once per engine and locale: the first lookup
/// builds it, the next ones read it (`docs/03-design/lexicon.md` §5).
#[test]
fn the_lexicon_is_built_once() {
    let intl = intl();
    let asked = LookupRequest::from_json(r#"{"text": "आज एकादशी, चन्द्रमा रोहिणीमा"}"#).unwrap();
    let cold = Instant::now();
    let first = intl.lookup(&asked).unwrap();
    let built = cold.elapsed();
    let warm = Instant::now();
    for _ in 0..100 {
        assert_eq!(intl.lookup(&asked).unwrap(), first);
    }
    let each = warm.elapsed() / 100;
    // Not a budget, which the bench keeps: a lookup that rebuilt the index
    // would cost about what the first did.
    assert!(
        each * 10 < built,
        "built in {built:?}, then {each:?} a lookup"
    );
}
