//! The unreviewed text: Hindi and Sanskrit `sdk.calendar` and composed
//! text (SR-38), Sanskrit's `sdk.amount`, and the rashifal (SR-37) and
//! rites (SR-31) ship marked, a reader sees the mark, and `check-intl`
//! holds it (`docs/03-design/intl-engine-and-packs.md`, "Text not yet
//! reviewed").
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro_intl::pack::{build, build_bundle};
use teistro_intl::source::Tree;
use teistro_intl::validate::{Severity, validate};
use teistro_intl::{ClockTime, Intl, Params, Value, params, sdk_root};

fn tree() -> Tree {
    Tree::load(&sdk_root()).expect("the sources")
}

fn errors(tree: &Tree) -> Vec<String> {
    validate(tree)
        .diagnostics
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .map(|finding| format!("{} {}: {}", finding.locale, finding.key, finding.message))
        .collect()
}

/// Every namespace Hindi and Sanskrit write is a draft: the calendar's
/// words, and the composed text SR-38 added.
const DRAFTED: [&str; 9] = [
    "sdk.aspect",
    "sdk.calendar",
    "sdk.condition",
    "sdk.day",
    "sdk.karaka",
    "sdk.naming",
    "sdk.phala",
    "sdk.reading",
    "sdk.reason",
];

/// The composed namespaces with one locale's own further draft, sorted as
/// `_meta.json` lists them.
fn drafted(extra: &str) -> Vec<String> {
    let mut all: Vec<String> = DRAFTED.iter().map(|name| (*name).to_owned()).collect();
    all.push(extra.to_owned());
    all.sort();
    all
}

/// This pin comes off only with the reviewer named in QUESTIONS.md Q42.
#[test]
fn hindi_and_sanskrit_text_ships_marked() {
    let tree = tree();
    // Hindi drafts the rashifal (SR-37) beside the composed text; Sanskrit
    // its currency words (SR-49). sa-Latn carries sa-Deva's drafts across
    // with the text it derives: transliterating a draft does not review it.
    assert_eq!(tree.locales["hi-Deva-IN"].meta.draft, drafted("sdk.rashifal"));
    for tag in ["sa-Deva", "sa-Latn"] {
        assert_eq!(tree.locales[tag].meta.draft, drafted("sdk.amount"), "{tag}");
    }
    // The strict locale drafts its day sentences, its rashifal (SR-37) and
    // its rites' names (SR-31), not its calendar words.
    assert_eq!(
        tree.locales["ne-Deva-NP"].meta.draft,
        ["sdk.day", "sdk.rashifal", "sdk.rite"]
    );
    assert_eq!(tree.locales["en-Latn"].meta.draft, Vec::<String>::new());
    let report = Intl::from_tree(&tree).unwrap().report();
    let hindi = report
        .locales
        .iter()
        .find(|l| l.tag == "hi-Deva-IN")
        .unwrap();
    assert_eq!(hindi.draft, drafted("sdk.rashifal"));
}

/// The parameters of one message of each composed namespace, by name.
fn composed(name: &str) -> Params {
    let entity = |key: &str| Value::Entity(key.into());
    match name {
        "jupiterInTheFifth" => {
            params([("graha", entity("graha.JUPITER")), ("bhava", Value::Int(5))])
        }
        "saturn" => params([("graha", entity("graha.SATURN"))]),
        "sunday" => params([("vara", entity("vara.RAVIVARA"))]),
        "long" => params([
            ("rule", Value::Str("R".into())),
            ("class", Value::Str("long".into())),
        ]),
        "atmakaraka" => params([
            ("graha", entity("graha.SUN")),
            ("karaka", entity("chara_karaka.ATMAKARAKA")),
        ]),
        "mutual" => params([
            ("first", entity("graha.MARS")),
            ("second", entity("graha.SATURN")),
        ]),
        "aries" => params([("rashi", entity("rashi.ARIES"))]),
        other => {
            let (name, n) = other.split_once('=').expect("a name=count");
            params([(name, Value::Int(n.parse().expect("a count")))])
        }
    }
}

/// Renders each case in its locale: the text, from that locale itself,
/// and marked a draft.
fn says(cases: &[(&str, &str, &str, &str)]) {
    let intl = Intl::from_tree(&tree()).unwrap();
    for (tag, key, name, text) in cases {
        let said = intl.render_in(tag, key, &composed(name));
        assert_eq!(said.text, *text, "{tag} {key}");
        assert_eq!(said.resolved_from.as_deref(), Some(*tag), "{tag} {key}");
        assert!(said.is_draft, "{tag} {key} is a draft");
    }
}

/// One message of each composed namespace (SR-38) as Hindi says it.
/// Houses and padas take the Sanskrit ordinal Hindi panchangas print; a
/// rank reads Hindi's ordinal categories, where 3 is `two` and 6 `many`.
#[test]
fn the_composed_text_speaks_hindi_and_says_it_is_a_draft() {
    says(&[
        (
            "hi-Deva-IN",
            "sdk.reason.grahaInBhava",
            "jupiterInTheFifth",
            "गुरु पंचम भाव में",
        ),
        (
            "hi-Deva-IN",
            "sdk.condition.retrograde",
            "saturn",
            "शनि वक्री है",
        ),
        // No locale's entity table has a `phala` form yet, so the name stands in.
        ("hi-Deva-IN", "sdk.phala.vara", "sunday", "रविवार"),
        ("hi-Deva-IN", "sdk.reading.lifeClass", "long", "दीर्घायु"),
        (
            "hi-Deva-IN",
            "sdk.karaka.ofSeven",
            "atmakaraka",
            "सात कारकों में सूर्य आत्मकारक है",
        ),
        (
            "hi-Deva-IN",
            "sdk.aspect.mutual",
            "mutual",
            "मंगल और शनि की परस्पर दृष्टि है",
        ),
        ("hi-Deva-IN", "sdk.naming.syllables", "count=2", "२ अक्षर"),
        ("hi-Deva-IN", "sdk.day.sunSign", "aries", "सूर्य मेष राशि में है।"),
        ("hi-Deva-IN", "sdk.reason.strength.rank", "rank=3", "३रा"),
        ("hi-Deva-IN", "sdk.reason.strength.rank", "rank=6", "६ठा"),
    ]);
}

/// One message of each composed namespace (SR-38) as Sanskrit says it,
/// and the derived Latin script. Sanskrit gives a stem its case through
/// its slot (`गुरुः`, `मेषराशौ`) and has a dual, which no plural category
/// names, so an exact `2` carries it.
#[test]
fn the_composed_text_speaks_sanskrit_and_says_it_is_a_draft() {
    says(&[
        (
            "sa-Deva",
            "sdk.reason.grahaInBhava",
            "jupiterInTheFifth",
            "गुरुः पञ्चमे भावे",
        ),
        ("sa-Deva", "sdk.condition.retrograde", "saturn", "शनिः वक्री"),
        ("sa-Deva", "sdk.phala.vara", "sunday", "रविवार"),
        ("sa-Deva", "sdk.reading.lifeClass", "long", "दीर्घायुः"),
        (
            "sa-Deva",
            "sdk.karaka.ofSeven",
            "atmakaraka",
            "सप्तसु कारकेषु सूर्यः आत्मकारकः",
        ),
        (
            "sa-Deva",
            "sdk.aspect.mutual",
            "mutual",
            "मंगल-शनिग्रहौ परस्परं पश्यतः",
        ),
        ("sa-Deva", "sdk.naming.syllables", "count=1", "१ अक्षरम्"),
        ("sa-Deva", "sdk.naming.syllables", "count=2", "२ अक्षरे"),
        ("sa-Deva", "sdk.naming.syllables", "count=3", "३ अक्षराणि"),
        ("sa-Deva", "sdk.day.sunSign", "aries", "सूर्यः मेषराशौ तिष्ठति।"),
        // Derived: the same text in IAST, the danda a full stop, a
        // sentence capitalised.
        (
            "sa-Latn",
            "sdk.reason.grahaInBhava",
            "jupiterInTheFifth",
            "Guruḥ pañcame bhāve",
        ),
        (
            "sa-Latn",
            "sdk.day.sunSign",
            "aries",
            "Sūryaḥ Meṣarāśau tiṣṭhati.",
        ),
    ]);
}

#[test]
fn a_render_says_when_draft_text_answered() {
    let mut intl = Intl::from_tree(&tree()).unwrap();
    let day = params([
        ("masa", Value::Entity("masa.CHAITRA".into())),
        ("kind", Value::Str("ADHIKA".into())),
    ]);
    let at = params([("at", Value::from(ClockTime::new(6, 5, 0)))]);
    assert!(
        intl.render_in("hi-Deva-IN", "sdk.calendar.lunarMonth", &day)
            .is_draft
    );
    assert!(
        !intl
            .render_in("ne-Deva-NP", "sdk.calendar.lunarMonth", &day)
            .is_draft
    );
    assert!(
        intl.render_in("hi-Deva-IN", "sdk.reason.appName", &Params::new())
            .is_draft
    );
    assert!(
        !intl
            .render_in("ne-Deva-NP", "sdk.reason.appName", &Params::new())
            .is_draft
    );

    // Reached through a function inside a message of a reviewed namespace.
    intl.set_locale("hi-Deva-IN").unwrap();
    let inside = intl.render_source("{$at :time hour12=true}", &at).unwrap();
    assert_eq!(inside.text, "सुबह ६:०५");
    assert!(inside.is_draft);

    // A consumer's own text is not the draft.
    intl.set_override("hi-Deva-IN", "sdk.calendar.lunarMonth", "x")
        .unwrap();
    assert!(
        !intl
            .render_in("hi-Deva-IN", "sdk.calendar.lunarMonth", &day)
            .is_draft
    );
}

#[test]
fn check_intl_holds_the_mark() {
    assert!(errors(&tree()).is_empty(), "{:?}", errors(&tree()));

    // A strict locale drafts a namespace it holds whole (C987): every key
    // is there, and the mark says no one has read them.
    let mut strict = tree();
    strict.locales.get_mut("ne-Deva-NP").unwrap().meta.draft = vec!["sdk.calendar".into()];
    assert!(errors(&strict).is_empty(), "{:?}", errors(&strict));

    let mut absent = tree();
    absent.locales.get_mut("sa-Deva").unwrap().meta.draft = vec!["sdk.amount".into()];
    assert!(
        errors(&absent)
            .iter()
            .any(|e| e.contains("no such namespace"))
    );

    let mut patched = tree();
    let namespace = patched
        .locales
        .get_mut("hi-Deva-IN")
        .unwrap()
        .namespaces
        .get_mut("sdk.calendar")
        .unwrap();
    namespace.entries.remove("weekdayName");
    assert!(
        errors(&patched)
            .iter()
            .any(|e| e.contains("missing from a drafted namespace"))
    );
}

#[test]
fn the_mark_travels_in_a_bundle_and_a_reviewers_pack_lifts_it() {
    let tree = tree();
    let hindi = &tree.locales["hi-Deva-IN"];
    let mut intl = Intl::from_tree(&tree).unwrap();

    // A bundle carries the metadata, mark and all.
    let bundle = build_bundle(hindi).unwrap();
    let mut other = tree.clone();
    other.locales.remove("hi-Deva-IN");
    let mut fresh = Intl::from_tree(&other).unwrap();
    fresh.load_pack(&bundle).unwrap();
    let report = fresh.report();
    let loaded = report
        .locales
        .iter()
        .find(|l| l.tag == "hi-Deva-IN")
        .unwrap();
    assert_eq!(loaded.draft, drafted("sdk.rashifal"));

    // The same namespace, signed off, carried by a pack with metadata.
    let mut reviewed = hindi.clone();
    reviewed.meta.draft.clear();
    intl.load_pack(&build(&reviewed, "sdk.calendar").unwrap())
        .unwrap();
    let day = params([
        ("masa", Value::Entity("masa.CHAITRA".into())),
        ("kind", Value::Str("ADHIKA".into())),
    ]);
    assert!(
        !intl
            .render_in("hi-Deva-IN", "sdk.calendar.lunarMonth", &day)
            .is_draft
    );

    // And one that still says draft keeps it.
    intl.load_pack(&build(hindi, "sdk.calendar").unwrap())
        .unwrap();
    assert!(
        intl.render_in("hi-Deva-IN", "sdk.calendar.lunarMonth", &day)
            .is_draft
    );
}

/// SR-31: a rite's name in Nepali says it is a draft; in English it does not,
/// and a day count takes its plural.
#[test]
fn a_rites_name_renders_and_the_nepali_says_it_is_a_draft() {
    let intl = Intl::from_tree(&tree()).unwrap();
    let verdict = params([("verdict", Value::Str("GOES_ON".into()))]);
    let english = intl.render_in("en-Latn", "sdk.rite.riteVerdict", &verdict);
    assert_eq!(english.text, "goes on");
    assert!(!english.is_draft);
    let nepali = intl.render_in("ne-Deva-NP", "sdk.rite.riteVerdict", &verdict);
    assert_eq!(nepali.text, "चल्छ");
    assert!(nepali.is_draft);
    let days = |n: i64| {
        let count = params([("days", Value::Int(n))]);
        intl.render_in("en-Latn", "sdk.rite.ashauchDays", &count)
            .text
    };
    assert_eq!(days(1), "1 day");
    assert_eq!(days(10), "10 days");
    let unknown = params([("verdict", Value::Str("NOT_A_MEMBER".into()))]);
    assert_eq!(
        intl.render_in("en-Latn", "sdk.rite.riteVerdict", &unknown)
            .text,
        "NOT_A_MEMBER",
        "a member no message names prints its own spelling"
    );
}
