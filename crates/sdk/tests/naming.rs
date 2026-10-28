//! The naming area through the façade and its records
//! (`03-design/namakarana.md` §4.2, §4.3): each request read strictly and
//! refused by its field under `naming`, a batch that never fails as a
//! whole, and a book loaded into the context or carried in the request.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Deity, Nakshatra, Rashi, Sex};
use teistro::matching::{
    Birth, Book, BookName, Clause, InitialBy, NamingRules, SuggestFilter, SyllablesOf,
};
use teistro::{
    Context, NamingCheckRequest, NamingLoadRequest, NamingOfRequest, NamingReadRequest,
    NamingSuggestRequest, NamingSyllablesRequest,
};

fn context() -> Context {
    Context::builder().build().unwrap()
}

fn book(names: &[&str]) -> Book {
    Book {
        names: names
            .iter()
            .map(|name| BookName {
                devanagari: (*name).to_owned(),
                ..BookName::default()
            })
            .collect(),
    }
}

const ASHWINI_1: Birth = Birth {
    nakshatra: Nakshatra::Ashwini,
    pada: 1,
};

#[test]
fn the_syllables_of_a_pada_a_star_and_a_sign() {
    let sdk = context();
    let rules = NamingRules::default();
    let one = sdk
        .naming()
        .syllables(
            SyllablesOf::Pada {
                nakshatra: Nakshatra::Ashwini,
                pada: 4,
            },
            rules,
        )
        .unwrap();
    assert_eq!(one[0].devanagari, "ला");
    let star = sdk
        .naming()
        .syllables(
            SyllablesOf::Nakshatra {
                nakshatra: Nakshatra::Ardra,
            },
            rules,
        )
        .unwrap();
    assert_eq!(star.len(), 4);
    // Makara's nine and Abhijit's four, whose pada is none.
    let makara = sdk
        .naming()
        .syllables(
            SyllablesOf::Rashi {
                rashi: Rashi::Capricorn,
            },
            rules,
        )
        .unwrap();
    assert_eq!(makara.iter().filter(|one| one.pada.is_none()).count(), 4);

    let asked = NamingSyllablesRequest::from_json(
        r#"{"of": {"of": "PADA", "nakshatra": "ASHWINI", "pada": 9}}"#,
    )
    .unwrap();
    assert_eq!(
        asked.answer(&sdk).unwrap_err().field(),
        Some("naming.of.pada")
    );
    let typo = NamingSyllablesRequest::from_json(r#"{"of": {"of": "PADA"}, "rule": {}}"#);
    assert!(typo.unwrap_err().field().unwrap().starts_with("naming"));
}

#[test]
fn a_batch_of_names_is_read_one_by_one() {
    let sdk = context();
    let asked = NamingReadRequest::from_json(
        r#"{"names": ["राम", "Shiva", "ऐश्वर्य"], "rules": {"name": {"latin": "CASUAL"}}}"#,
    )
    .unwrap();
    let read = asked.answer(&sdk);
    assert_eq!(read.len(), 3);
    assert!(read[0].refused.is_none());
    assert!(read[1].reading.as_ref().unwrap().ambiguous);
    let refused = read[2].refused.as_ref().unwrap();
    assert_eq!(refused.field(), Some("naming.names[2]"));
    assert!(read[2].reading.is_none());
    // The area answers the reader's own refusal, unrenamed.
    let own = sdk.naming().read(&["ऐश्वर्य"], NamingRules::default());
    assert_eq!(own[0].as_ref().unwrap_err().field(), Some("name"));
    assert_eq!(
        NamingReadRequest::from_json(r#"{"name": ["राम"]}"#)
            .unwrap_err()
            .field(),
        Some("naming.name")
    );
}

#[test]
fn a_batch_of_names_is_checked_against_one_birth() {
    let sdk = context();
    let asked = NamingCheckRequest::from_json(
        r#"{"names": ["चूडामणि", "चेदीश", "ऐश्वर्य"], "birth": {"nakshatra": "ASHWINI", "pada": 1}}"#,
    )
    .unwrap();
    let checked = asked.answer(&sdk).unwrap();
    let same: Vec<Option<bool>> = checked
        .iter()
        .map(|one| one.check.as_ref().map(|check| check.same_pada))
        .collect();
    assert_eq!(same, [Some(true), Some(false), None]);
    assert_eq!(
        checked[2].refused.as_ref().unwrap().field(),
        Some("naming.names[2]")
    );
    // A birth outside the four padas refuses the whole batch, even an
    // empty one.
    let wrong = NamingCheckRequest::from_json(
        r#"{"names": [], "birth": {"nakshatra": "ASHWINI", "pada": 0}}"#,
    )
    .unwrap();
    assert_eq!(
        wrong.answer(&sdk).unwrap_err().field(),
        Some("naming.birth.pada")
    );
    let sex = NamingCheckRequest::from_json(
        r#"{"names": [], "birth": {"nakshatra": "ASHWINI", "pada": 1, "sex": "MALE"}}"#,
    );
    assert_eq!(sex.unwrap_err().field(), Some("naming.birth.sex"));
}

#[test]
fn the_names_a_birth_gives() {
    let sdk = context();
    let asked =
        NamingOfRequest::from_json(r#"{"birth": {"nakshatra": "KRITTIKA", "pada": 1}}"#).unwrap();
    let names = asked.answer(&sdk).unwrap();
    assert_eq!(
        (names.syllable.devanagari.as_str(), names.deity),
        ("आ", Deity::Agni)
    );
    let wrong =
        NamingOfRequest::from_json(r#"{"birth": {"nakshatra": "KRITTIKA", "pada": 5}}"#).unwrap();
    assert_eq!(
        wrong.answer(&sdk).unwrap_err().field(),
        Some("naming.birth.pada")
    );
}

#[test]
fn a_book_is_loaded_once_and_suggested_from_by_its_id() {
    let sdk = context();
    let loaded = sdk
        .naming()
        .load(book(&["चूडामणि", "चूडा", "राम", "औषधि"]))
        .unwrap();
    assert_eq!((loaded.names, loaded.refused.len()), (4, 1));
    // The same book, spelt as JSON, is the same book.
    let again = NamingLoadRequest::from_json(
        r#"{"book": {"names": [{"devanagari": "चूडामणि"}, {"devanagari": "चूडा"}, {"devanagari": "राम"}, {"devanagari": "औषधि"}]}}"#,
    )
    .unwrap()
    .answer(&sdk)
    .unwrap();
    assert_eq!(again.book, loaded.book);

    let found = sdk
        .naming()
        .suggest(
            loaded.book,
            ASHWINI_1,
            &SuggestFilter::default(),
            NamingRules::default(),
        )
        .unwrap();
    let indices: Vec<u32> = found.names.iter().map(|name| name.index).collect();
    assert_eq!(indices, [0, 1]);
    assert_eq!(found.refused[0].index, 3);

    let by_id = NamingSuggestRequest::from_json(&format!(
        r#"{{"bookId": "{}", "birth": {{"nakshatra": "ASHWINI", "pada": 1}}, "filter": {{"syllables": [2]}}}}"#,
        loaded.book
    ))
    .unwrap()
    .answer(&sdk)
    .unwrap();
    assert_eq!(by_id.names[0].index, 1);

    // Another context has not loaded it.
    let other = context();
    let refused = NamingSuggestRequest::from_json(&format!(
        r#"{{"bookId": "{}", "birth": {{"nakshatra": "ASHWINI", "pada": 1}}}}"#,
        loaded.book
    ))
    .unwrap()
    .answer(&other)
    .unwrap_err();
    assert_eq!(refused.field(), Some("naming.bookId"));

    // Text that is no id at all is refused the same way, with the hint.
    let malformed = NamingSuggestRequest::from_json(
        r#"{"bookId": "my-book", "birth": {"nakshatra": "ASHWINI", "pada": 1}}"#,
    )
    .unwrap()
    .answer(&sdk)
    .unwrap_err();
    assert_eq!(malformed.field(), Some("naming.bookId"));
    assert!(malformed.hint().is_some_and(|hint| hint.contains("hex")));
}

#[test]
fn a_book_carried_in_the_request_is_used_once() {
    let sdk = context();
    let asked = NamingSuggestRequest::from_json(
        r#"{"book": {"names": [{"devanagari": "गणेश", "sex": "MALE"}, {"devanagari": "गीता", "sex": "FEMALE"}]},
            "birth": {"nakshatra": "ARDRA", "pada": 3}, "filter": {"sex": "FEMALE"}}"#,
    )
    .unwrap();
    let found = asked.answer(&sdk).unwrap();
    assert_eq!(found.names.len(), 1);
    assert_eq!(
        (found.names[0].index, found.names[0].by),
        (1, InitialBy::Substitute)
    );
    assert_eq!(asked.filter.sex, Some(Sex::Female));
}

#[test]
fn a_suggestion_is_refused_by_its_field() {
    let sdk = context();
    let field = |json: &str| {
        NamingSuggestRequest::from_json(json)
            .and_then(|asked| asked.answer(&sdk))
            .unwrap_err()
            .field()
            .map(str::to_owned)
    };
    let birth = r#""birth": {"nakshatra": "ASHWINI", "pada": 1}"#;
    assert_eq!(
        field(&format!("{{{birth}}}")).as_deref(),
        Some("naming.book")
    );
    let id = sdk.naming().load(book(&["राम"])).unwrap().book;
    assert_eq!(
        field(&format!(
            r#"{{"bookId": "{id}", "book": {{"names": []}}, {birth}}}"#
        ))
        .as_deref(),
        Some("naming.bookId")
    );
    assert_eq!(
        field(&format!(
            r#"{{"book": {{"names": [{{"devanagari": "Ram"}}]}}, {birth}}}"#
        ))
        .as_deref(),
        Some("naming.book.names[0].devanagari")
    );
    assert_eq!(
        field(&format!(
            r#"{{"book": {{"names": [{{"devanagari": "राम", "sex": "BOY"}}]}}, {birth}}}"#
        ))
        .as_deref(),
        Some("naming.book.names[0].sex")
    );
    assert_eq!(
        field(r#"{"book": {"names": []}, "birth": {"nakshatra": "ASHWINI", "pada": 7}}"#)
            .as_deref(),
        Some("naming.birth.pada")
    );
    assert_eq!(
        field(&format!(
            r#"{{"book": {{"names": []}}, {birth}, "filter": {{"clauses": ["EVEN_FOR_A_BOY"]}}}}"#
        ))
        .as_deref(),
        Some("naming.filter.clauses")
    );
    // The same clause is Ashvalayana's, so under it the filter reads.
    let ashvalayana = NamingSuggestRequest::from_json(&format!(
        r#"{{"book": {{"names": [{{"devanagari": "चूडा"}}]}}, {birth}, "filter": {{"clauses": ["EVEN_FOR_A_BOY"]}}, "rules": {{"sutra": "ASHVALAYANA"}}}}"#
    ))
    .unwrap();
    assert_eq!(ashvalayana.filter.clauses, [Clause::EvenForABoy]);
    assert_eq!(ashvalayana.answer(&sdk).unwrap().names.len(), 1);
}

#[test]
fn a_load_is_refused_by_its_field() {
    let sdk = context();
    let refused = NamingLoadRequest::from_json(r#"{"book": {"names": [{"devanagari": " "}]}}"#)
        .unwrap()
        .answer(&sdk)
        .unwrap_err();
    assert_eq!(refused.field(), Some("naming.book.names[0].devanagari"));
    let unknown =
        NamingLoadRequest::from_json(r#"{"book": {"names": [], "meaning": "x"}}"#).unwrap_err();
    assert_eq!(unknown.field(), Some("naming.book.meaning"));
}

#[test]
fn every_naming_record_answers_as_its_request_does() {
    let sdk = context();
    let answered = |name: &str, json: &str| {
        let record = teistro::records::record(name).unwrap();
        record.answer(&sdk, json).unwrap().value
    };
    let syllables = answered(
        "naming.syllables",
        r#"{"of": {"of": "NAKSHATRA", "nakshatra": "ASHWINI"}}"#,
    );
    assert_eq!(syllables[0]["devanagari"], "चू");
    let read = answered("naming.read", r#"{"names": ["राम"]}"#);
    assert_eq!(read[0]["refused"], serde_json::Value::Null);
    let checked = answered(
        "naming.check",
        r#"{"names": ["चूडामणि"], "birth": {"nakshatra": "ASHWINI", "pada": 1}}"#,
    );
    assert_eq!(checked[0]["check"]["samePada"], true);
    let of = answered(
        "naming.of",
        r#"{"birth": {"nakshatra": "KRITTIKA", "pada": 1}}"#,
    );
    assert_eq!(of["deity"], "AGNI");
    let suggested = answered(
        "naming.suggest",
        r#"{"book": {"names": [{"devanagari": "चूडामणि"}]}, "birth": {"nakshatra": "ASHWINI", "pada": 1}}"#,
    );
    assert_eq!(suggested["names"][0]["index"], 0);
}

/// A key as the wire spells it.
fn wire<T: serde::Serialize>(member: &T) -> String {
    serde_json::to_value(member)
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned()
}

/// Every key a naming answer carries is said in words of its own in each
/// strict locale (`03-design/namakarana.md` §4.4): from that locale's own
/// message, never the catch-all, and no two keys alike. The keys are read
/// from the answers, every clause both sutras report and every way an
/// initial reaches a cell, so a member added to the kernel without its
/// words fails here rather than reading as "another clause".
#[test]
fn every_key_an_answer_carries_has_words_of_its_own_in_each_strict_locale() {
    use std::collections::BTreeSet;
    use teistro::Rendered;
    use teistro::matching::{FinalA, GrhyaSutra, name_clauses};
    use teistro::messages::sdk::naming;

    let sdk = context();
    let mut clauses = BTreeSet::new();
    let mut sources = BTreeSet::new();
    for sutra in [GrhyaSutra::Paraskara, GrhyaSutra::Ashvalayana] {
        let rules = NamingRules {
            sutra,
            ..NamingRules::default()
        };
        for row in name_clauses("राम", rules).rows {
            clauses.insert(wire(&row.clause));
            sources.insert(wire(&row.source));
        }
    }
    let mut by = BTreeSet::new();
    for rashi in Rashi::ALL {
        let of = SyllablesOf::Rashi { rashi };
        for one in sdk.naming().syllables(of, NamingRules::default()).unwrap() {
            by.extend(one.initials.iter().map(|initial| wire(&initial.by)));
        }
    }
    let counted: BTreeSet<String> = [FinalA::Sounded, FinalA::Silent].iter().map(wire).collect();
    assert_eq!(
        (clauses.len(), sources.len(), by.len(), counted.len()),
        (8, 3, 3, 2)
    );

    for locale in ["en-Latn", "ne-Deva-NP"] {
        sdk.intl().set_locale(locale).unwrap();
        let said = |message: &dyn Fn(String) -> Rendered, keys: &BTreeSet<String>| {
            let other = message(String::from("NOT_A_KEY")).text;
            let mut words = BTreeSet::new();
            for key in keys {
                let one = message(key.clone());
                assert_eq!(one.resolved_from.as_deref(), Some(locale), "{key}");
                assert!(
                    !one.is_fallback && one.warnings.is_empty(),
                    "{key} in {locale}"
                );
                assert_ne!(one.text, other, "{key} falls to the catch-all in {locale}");
                assert!(
                    words.insert(one.text),
                    "{key} says another key's words in {locale}"
                );
            }
        };
        let intl = sdk.intl();
        said(
            &|clause| intl.render_typed(&naming::Clause { clause }),
            &clauses,
        );
        said(
            &|source| intl.render_typed(&naming::Source { source }),
            &sources,
        );
        said(&|by| intl.render_typed(&naming::By { by }), &by);
        said(
            &|counted| intl.render_typed(&naming::Counted { counted }),
            &counted,
        );
        // A clause met or not: `yes`, and anything else is not.
        let holds = ["yes", "no"].map(|holds| {
            intl.render_typed(&naming::Holds {
                holds: holds.to_owned(),
            })
        });
        assert!(
            holds
                .iter()
                .all(|one| one.resolved_from.as_deref() == Some(locale) && one.warnings.is_empty())
        );
        assert_ne!(holds[0].text, holds[1].text);
        // The pada's four ordinals, each its own words.
        let padas: BTreeSet<String> = (1..=4)
            .map(|pada| {
                let one = sdk.intl().render_typed(&naming::Pada {
                    nakshatra: Nakshatra::Ardra,
                    pada,
                    syllable: String::from("ङ"),
                });
                assert_eq!(one.resolved_from.as_deref(), Some(locale));
                assert!(one.warnings.is_empty(), "{:?}", one.warnings);
                one.text
            })
            .collect();
        assert_eq!(padas.len(), 4, "{padas:?}");
    }
}
