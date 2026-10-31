//! The unreviewed text: Hindi and Sanskrit `sdk.calendar` ship marked, a
//! reader sees the mark, and `check-intl` holds it
//! (`docs/03-design/intl-engine-and-packs.md`, "Text not yet reviewed").
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

/// This pin comes off only with the reviewer named in QUESTIONS.md Q42.
#[test]
fn hindi_and_sanskrit_calendar_text_ships_marked() {
    let tree = tree();
    for tag in ["hi-Deva-IN", "sa-Deva"] {
        assert_eq!(tree.locales[tag].meta.draft, ["sdk.calendar"], "{tag}");
    }
    for tag in ["en-Latn", "ne-Deva-NP", "sa-Latn"] {
        assert!(tree.locales[tag].meta.draft.is_empty(), "{tag}");
    }
    let report = Intl::from_tree(&tree).unwrap().report();
    let hindi = report
        .locales
        .iter()
        .find(|l| l.tag == "hi-Deva-IN")
        .unwrap();
    assert_eq!(hindi.draft, ["sdk.calendar"]);
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
        !intl
            .render_in("hi-Deva-IN", "sdk.reason.appName", &Params::new())
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

    let mut strict = tree();
    strict.locales.get_mut("ne-Deva-NP").unwrap().meta.draft = vec!["sdk.calendar".into()];
    assert!(
        errors(&strict)
            .iter()
            .any(|e| e.contains("strict locale ships no draft"))
    );

    let mut absent = tree();
    absent.locales.get_mut("sa-Deva").unwrap().meta.draft = vec!["sdk.reading".into()];
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
    assert_eq!(loaded.draft, ["sdk.calendar"]);

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
