//! `settings.explain` and `settings.diff` through the façade and the
//! records (`03-design/settings-and-profiles.md` §3.5): every knob has an
//! effect note in every locale the SDK ships, a derived context keeps its
//! parent's layers, and the records answer as the methods do.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::Context;
use teistro::settings::{Layer, Settings, SettingsQuery};

/// The locales whose packs carry the notes themselves; `sa-Latn` is
/// derived from `sa-Deva`.
const LOCALES: [&str; 5] = ["en-Latn", "ne-Deva-NP", "hi-Deva-IN", "sa-Deva", "sa-Latn"];

#[test]
fn every_knob_has_an_effect_note_in_every_shipped_locale() {
    let sdk = Context::builder().build().unwrap();
    let intl = sdk.locale_engine();
    for knob in Settings::knobs() {
        let key = knob.effect_key();
        for locale in LOCALES {
            let note = intl.render_in(locale, &key, &teistro::Params::new());
            assert_eq!(
                note.resolved_from.as_deref(),
                Some(locale),
                "{locale} has no note for {}",
                knob.path()
            );
            assert!(
                note.warnings.is_empty(),
                "{locale} {key}: {:?}",
                note.warnings
            );
            assert!(!note.text.is_empty() && note.text != key, "{locale} {key}");
        }
    }
}

#[test]
fn a_derived_context_keeps_its_parents_layers() {
    let parent = Context::builder()
        .profile("kp-default")
        .settings_json(r#"{"frame": {"node": "TRUE"}}"#)
        .locale("hi-Deva-IN")
        .build()
        .unwrap();
    let derived = parent
        .with_patch_json(r#"{"day": {"ghati_reckoning": "PROPORTIONAL"}}"#, None)
        .unwrap();
    let explained = derived.explain_settings(&SettingsQuery::default()).unwrap();
    let layer = |name: &str| {
        explained
            .knobs
            .iter()
            .find(|knob| knob.knob == name)
            .unwrap()
            .layer
    };
    assert_eq!(
        [
            layer("frame.node"),
            layer("day.ghati_reckoning"),
            layer("frame.ayanamsha"),
            layer("output.precision")
        ],
        [Layer::Context, Layer::Request, Layer::Profile, Layer::Root]
    );
    assert_eq!(explained.settings_hash, derived.settings_hash());
    // The note in the derived context's locale, the parent's Hindi.
    let node = explained
        .knobs
        .iter()
        .find(|knob| knob.knob == "frame.node")
        .unwrap();
    assert!(node.effect.contains("राहु"), "{}", node.effect);
}

#[test]
fn the_records_answer_as_the_methods_do() {
    let committee = Context::builder()
        .profile("nepali-committee")
        .build()
        .unwrap();
    let drik = Context::builder().profile("nepali-drik").build().unwrap();
    let asked = SettingsQuery::from_json(r#"{"knobs": ["frame", "day.sunrise"]}"#).unwrap();

    let explain = teistro::records::record("settings.explain").unwrap();
    assert!(!explain.compares());
    let answered = explain
        .answer(&committee, r#"{"knobs": ["frame", "day.sunrise"]}"#)
        .unwrap();
    assert!(
        answered.provenance.is_none(),
        "an explanation computes nothing to seal"
    );
    assert_eq!(
        answered.value,
        serde_json::to_value(committee.explain_settings(&asked).unwrap()).unwrap()
    );

    let diff = teistro::records::record("settings.diff").unwrap();
    assert!(diff.compares());
    let parted = diff.answer_against(&committee, &drik, "{}").unwrap();
    assert_eq!(
        parted.value,
        serde_json::to_value(
            committee
                .diff_settings(&drik, &SettingsQuery::default())
                .unwrap()
        )
        .unwrap()
    );
    assert_eq!(parted.value["otherProfile"], "nepali-drik");
    // Alone it compares the context with itself.
    assert_eq!(
        diff.answer(&drik, "{}").unwrap().value["knobs"],
        serde_json::json!([])
    );

    let refused = explain
        .answer(&drik, r#"{"knobs": ["frame.nod"]}"#)
        .unwrap_err();
    assert_eq!(refused.field(), Some("knobs[0]"));
    let refused = explain.answer(&drik, r#"{"knob": ["frame"]}"#).unwrap_err();
    assert_eq!(refused.status, teistro::Status::InvalidArg);
}

#[test]
fn a_receipt_of_an_explanation_holds_and_other_settings_are_a_documents() {
    use teistro::settings::{Profile, SettingsPatch, SettingsSnapshot};
    use teistro::verify::{IssueRequest, ReceiptVerdict};

    let sdk = Context::builder()
        .profile("kp-default")
        .settings_json(r#"{"frame": {"node": "TRUE"}}"#)
        .build()
        .unwrap();
    let asked = IssueRequest::from_json(
        r#"{"record": "settings.explain", "request": {"knobs": ["frame"]}}"#,
    )
    .unwrap();
    let issued = sdk.verify().issue(&asked).unwrap();
    let check = sdk.verify().receipt(&issued.receipt);
    assert_eq!(check.verdict, ReceiptVerdict::Held, "{check:?}");
    // The context's own settings handed back keep their layers ...
    let own = sdk
        .with_settings(&SettingsSnapshot::of(sdk.resolved()), None)
        .unwrap();
    assert_eq!(
        own.explain_settings(&SettingsQuery::default()).unwrap(),
        sdk.explain_settings(&SettingsQuery::default()).unwrap()
    );
    // ... and another profile's are a document's.
    let drik = Profile::shipped("nepali-drik")
        .unwrap()
        .resolve(&SettingsPatch::default())
        .unwrap();
    let other = sdk
        .with_settings(&SettingsSnapshot::of(&drik), None)
        .unwrap();
    let explained = other.explain_settings(&SettingsQuery::default()).unwrap();
    assert!(
        explained
            .knobs
            .iter()
            .all(|knob| knob.layer == Layer::Document)
    );
}
