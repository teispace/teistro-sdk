//! A day in sentences through the context (`03-design/day-in-sentences.md`):
//! the sentences in each language, read back against the day's answer, a
//! locale without templates refused, and a polar place across a year.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use serde_json::json;
use teistro::catalogue::Calendar;
use teistro::{CalendarDate, Context, Ephemeris, SentencesRequest, Status, UtcOffset};

fn context(settings: &str) -> Context {
    Context::builder()
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .settings_json(settings)
        .build()
        .unwrap()
}

fn asked(
    date: (i32, u8, u8),
    place: (f64, f64),
    offset: i32,
    locale: Option<&str>,
) -> SentencesRequest {
    let mut record = json!({
        "date": {"year": date.0, "month": date.1, "day": date.2},
        "latitudeDeg": place.0, "longitudeDeg": place.1, "utcOffsetSeconds": offset,
    });
    if let Some(locale) = locale {
        record["locale"] = json!(locale);
    }
    SentencesRequest::from_json(&record.to_string()).unwrap()
}

const KATHMANDU: (f64, f64) = (27.7172, 85.324);
const NEPAL: i32 = 20_700;

/// Every sentence reads a value of the day's answer: the opening day's
/// tithi and vara are named as the entity pages name them, in each
/// language, and the Nepali's digits are Nepal's.
#[test]
fn a_day_says_its_own_answer_in_english_and_nepali() {
    let sdk = context("{}");
    let date = CalendarDate::defined(Calendar::Gregorian, 2026, 3, 18);
    let place = asked((2026, 3, 18), KATHMANDU, NEPAL, None).place;
    let day = sdk
        .almanac()
        .day(&date, &place, UtcOffset::literal(5, 45, 0))
        .unwrap()
        .value;
    let en = sdk
        .almanac()
        .sentences(&asked((2026, 3, 18), KATHMANDU, NEPAL, Some("en-Latn")))
        .unwrap()
        .value;
    assert!(!en.draft);
    assert_eq!(en.withheld, Vec::<String>::new());
    assert_eq!(en.sentences[0].text, "The day is Wednesday.");
    let tithis: Vec<_> = en
        .sentences
        .iter()
        .filter(|s| s.key == "sdk.day.tithi")
        .collect();
    assert_eq!(tithis.len(), day.limbs.tithi.len());

    let ne = sdk
        .almanac()
        .sentences(&asked((2026, 3, 18), KATHMANDU, NEPAL, Some("ne-Deva-NP")))
        .unwrap()
        .value;
    assert!(ne.draft, "a Nepali template waits on its reviewer (Q42)");
    assert_eq!(ne.sentences.len(), en.sentences.len());
    assert_eq!(ne.sentences[0].text, "यो दिन बुधबार हो।");
    let sun = ne
        .sentences
        .iter()
        .find(|s| s.key == "sdk.day.sun")
        .unwrap();
    assert!(
        sun.text.chars().any(|c| ('०'..='९').contains(&c)),
        "{}",
        sun.text
    );
    assert!(
        !sun.text.chars().any(|c| c.is_ascii_digit()),
        "{}",
        sun.text
    );
}

/// A locale with no templates is refused by name rather than answered in
/// its fallback's language, and one not loaded is refused for that.
#[test]
fn a_locale_without_templates_is_refused_and_says_why() {
    let sdk = context("{}");
    for locale in ["hi-Deva-IN", "sa-Deva", "sa-Latn"] {
        let refused = sdk
            .almanac()
            .sentences(&asked((2026, 3, 18), KATHMANDU, NEPAL, Some(locale)))
            .unwrap_err();
        assert_eq!(refused.status, Status::Unsupported);
        assert_eq!(refused.field(), Some("locale"));
        assert!(refused.message.contains("Q42"), "{refused}");
    }
    let unknown = sdk
        .almanac()
        .sentences(&asked((2026, 3, 18), KATHMANDU, NEPAL, Some("xx-Latn")))
        .unwrap_err();
    assert_eq!(unknown.field(), Some("locale"));
    assert!(unknown.message.contains("not loaded"), "{unknown}");
}

/// Left out, the locale is the context's.
#[test]
fn the_locale_defaults_to_the_contexts() {
    let sdk = context("{}");
    let said = sdk
        .almanac()
        .sentences(&asked((2026, 3, 18), KATHMANDU, NEPAL, None))
        .unwrap();
    assert_eq!(said.value.locale, sdk.intl().locale());
    assert_ne!(said.provenance.content_hash.to_string(), "");
}

/// The polar sankranti: at Tromso on the Karka and Makara sankrantis, the
/// day under `NEAREST_EVENT` answers rather than refusing, says the Sun
/// neither rises nor sets, and withholds what its window holds elsewhere.
/// Under `UNDEFINED`, the day's own refusal names the policy.
#[test]
fn a_polar_sankranti_is_answered_under_a_policy_and_refused_without() {
    const TROMSO: (f64, f64) = (69.6492, 18.9553);
    let nearest = context(r#"{"day": {"polar_day_policy": "NEAREST_EVENT"}}"#);
    for (date, sentence) in [
        ((2026, 7, 16), "sdk.day.sunUp"),
        ((2026, 12, 21), "sdk.day.sunDown"),
    ] {
        let said = nearest
            .almanac()
            .sentences(&asked(date, TROMSO, 3_600, Some("en-Latn")))
            .unwrap()
            .value;
        assert!(said.sentences.iter().any(|s| s.key == sentence), "{date:?}");
        assert!(said.withheld.iter().any(|fact| fact == "tithi"));
        assert!(said.sentences.iter().all(|s| s.key != "sdk.day.tithi"));
    }
    let undefined = context(r#"{"day": {"polar_day_policy": "UNDEFINED"}}"#);
    let refused = undefined
        .almanac()
        .sentences(&asked((2026, 7, 16), TROMSO, 3_600, Some("en-Latn")))
        .unwrap_err();
    assert_eq!(refused.field(), Some("day.polar_day_policy"));
}

/// Tromso on 21 December 2026 under both polar policies. `NEAREST_EVENT`
/// runs from the last sunrise to the next, some fifty-five tithis, which
/// is its intended reading, so a sentence of one tithi would be false of
/// the date and the limbs are withheld. `CIVIL_MIDNIGHT` is what a
/// calendar shows: the date's own limbs, and still no sunrise.
#[test]
fn tromso_at_midwinter_reads_as_each_policy_means() {
    const TROMSO: (f64, f64) = (69.6492, 18.9553);
    let request = asked((2026, 12, 21), TROMSO, 3_600, Some("en-Latn"));
    let date = CalendarDate::defined(Calendar::Gregorian, 2026, 12, 21);

    let nearest = context(r#"{"day": {"polar_day_policy": "NEAREST_EVENT"}}"#);
    let span = nearest
        .almanac()
        .day(&date, &request.place, UtcOffset::literal(1, 0, 0))
        .unwrap()
        .value;
    assert!(
        (45..=65).contains(&span.limbs.tithi.len()),
        "{}",
        span.limbs.tithi.len()
    );
    let said = nearest.almanac().sentences(&request).unwrap().value;
    let keys: Vec<&str> = said.sentences.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(keys, ["sdk.day.vara", "sdk.day.sunDown", "sdk.day.ritu"]);
    assert!(said.withheld.iter().any(|fact| fact == "tithi"));

    let civil = context(r#"{"day": {"polar_day_policy": "CIVIL_MIDNIGHT"}}"#);
    let said = civil.almanac().sentences(&request).unwrap().value;
    let tithis = said.sentences.iter().filter(|s| s.key == "sdk.day.tithi");
    assert!((1..=3).contains(&tithis.count()));
    assert!(said.sentences.iter().any(|s| s.key == "sdk.day.sunDown"));
    assert!(said.sentences.iter().all(|s| s.key != "sdk.day.sun"));
    assert_eq!(said.withheld, Vec::<String>::new());
}

/// Each refusal of the request's own reading.
#[test]
fn a_sentence_request_is_read_strictly() {
    let unknown = SentencesRequest::from_json(
        r#"{"date": {"year": 2026, "month": 3, "day": 18}, "latitudeDeg": 0, "longitudeDeg": 0,
            "utcOffsetSeconds": 0, "language": "en"}"#,
    )
    .unwrap_err();
    assert_eq!(unknown.status, Status::InvalidArg);
    let far = SentencesRequest::from_json(
        r#"{"date": {"year": 2026, "month": 3, "day": 18}, "latitudeDeg": 91, "longitudeDeg": 0,
            "utcOffsetSeconds": 0}"#,
    )
    .unwrap_err();
    assert!(
        far.field().is_some_and(|f| f.starts_with("sentences")),
        "{far}"
    );
}
