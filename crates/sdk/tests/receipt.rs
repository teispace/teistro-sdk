//! Receipts (`03-design/receipts-and-datasets.md` §1, SR-51): every
//! example record issued with a receipt holds when checked; a field
//! changed after issue breaks the seal; a change sealed again is caught by
//! the re-run, or reported as a move; and what cannot be re-run says why.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::Hash;
use teistro::settings::{Settings, SettingsSnapshot};
use teistro::verify::{
    IssueRequest, NotRerunCause, Receipt, ReceiptCheck, ReceiptProvider, ReceiptVerdict,
};
use teistro::{Context, Ephemeris, Status};
use teistro_core::envelope::{PackStamp, Version, content_hash};

fn context(profile: &str) -> Context {
    Context::builder()
        .profile(profile)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a context")
}

fn issued(sdk: &Context, record: &str, request: &serde_json::Value) -> Receipt {
    let asked = IssueRequest {
        record: record.to_owned(),
        request: request.clone(),
    };
    sdk.verify().issue(&asked).unwrap().receipt
}

/// A sankalpa at Kathmandu, which reads the ephemeris.
fn sankalpa(sdk: &Context) -> Receipt {
    issued(
        sdk,
        "almanac.sankalpa",
        &serde_json::json!({"at": 2_461_055.593_75, "latitudeDeg": 27.7172,
            "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}),
    )
}

/// A date converted, which reads no ephemeris.
fn converted(sdk: &Context) -> Receipt {
    issued(
        sdk,
        "calendar.convert",
        &serde_json::json!({"date": {"year": 2026, "month": 10, "day": 10},
            "into": "calendar.BIKRAM_SAMBAT"}),
    )
}

/// `receipt` with a knob of its settings changed by `edit`, and the
/// snapshot taken again: what a caller does to ask under other settings.
fn resettled(mut receipt: Receipt, edit: impl FnOnce(&mut Settings)) -> Receipt {
    let mut settings = receipt.settings.settings().unwrap();
    edit(&mut settings);
    receipt.settings = SettingsSnapshot::new(&receipt.settings.profile, &settings);
    receipt
}

fn cause(check: &ReceiptCheck) -> NotRerunCause {
    assert_eq!(check.verdict, ReceiptVerdict::NotRerun, "{check:?}");
    check.reason.as_ref().unwrap().cause
}

fn moved(check: &ReceiptCheck) -> Vec<&str> {
    check.moved.iter().map(|m| m.field.as_str()).collect()
}

/// Every example the SDK's own tests send, issued and checked, holds; a
/// record the build lacks is skipped, as is the engine's passthrough,
/// which the built-in ephemeris refuses, and the receipts' own records.
#[test]
fn every_example_record_holds_when_checked() {
    let sdk = context("nepali-drik");
    let examples: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("requests.json")).unwrap();
    let mut held = 0;
    for example in examples {
        let record = example["record"].as_str().unwrap();
        if record.starts_with("engine.")
            || record.starts_with("verify.issue")
            || record.starts_with("verify.receipt")
            || teistro::records::record(record).is_none()
        {
            continue;
        }
        let receipt = issued(&sdk, record, &example["request"]);
        assert!(receipt.broken().is_empty(), "{record}");
        let check = sdk.verify().receipt(&receipt);
        assert_eq!(check.verdict, ReceiptVerdict::Held, "{record}: {check:?}");
        assert!(check.moved.is_empty(), "{record}: {check:?}");
        held += 1;
    }
    assert!(held >= 30, "only {held} examples held");
}

/// What a receipt names, from the record and the context that issued it.
#[test]
fn a_receipt_names_what_it_was_asked_and_under_what() {
    let sdk = context("nepali-drik");
    let sky = sankalpa(&sdk);
    assert_eq!(sky.record, "almanac.sankalpa");
    assert_eq!(sky.settings.profile, "nepali-drik");
    assert_eq!(sky.settings.hash, sdk.settings_hash());
    assert_eq!(sky.locale, sdk.intl().locale());
    assert_eq!(sky.provider.as_ref().unwrap().name, "teistro-builtin");
    assert_eq!(
        sky.sdk_version,
        Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
    );
    // An answer that read no sky names no ephemeris.
    assert!(converted(&sdk).provider.is_none());
}

/// Each field changed after issue breaks the seal, and nothing is re-run.
#[test]
fn a_field_changed_after_issue_breaks_the_receipt() {
    let sdk = context("nepali-drik");
    let receipt = serde_json::to_value(sankalpa(&sdk)).unwrap();
    let edits: [(&str, serde_json::Value, &[&str]); 14] = [
        ("record", "almanac.digest".into(), &["digest"]),
        (
            "request",
            serde_json::json!({"at": 2_461_056.0, "latitudeDeg": 27.7172,
                "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}),
            &["requestDigest", "digest"],
        ),
        (
            "requestDigest",
            Hash::of(b"x").to_string().into(),
            &["requestDigest", "digest"],
        ),
        ("locale", "de-Latn-DE".into(), &["digest"]),
        (
            "packs",
            serde_json::json!([{"id": "x", "version": "1", "hash": Hash::of(b"x").to_string()}]),
            &["digest"],
        ),
        ("provider", serde_json::Value::Null, &["digest"]),
        ("tzdb", "1999a".into(), &["digest"]),
        (
            "sdkVersion",
            serde_json::json!({"major": 0, "minor": 0, "patch": 1}),
            &["digest"],
        ),
        ("calculationVersion", 99.into(), &["digest"]),
        ("catalogueVersion", 99.into(), &["digest"]),
        (
            "outputDigest",
            Hash::of(b"x").to_string().into(),
            &["digest"],
        ),
        ("digest", Hash::of(b"x").to_string().into(), &["digest"]),
        ("settings", serde_json::Value::Null, &[]),
        ("format", serde_json::Value::Null, &[]),
    ];
    for (field, value, broken) in edits {
        let mut edited = receipt.clone();
        if field == "settings" {
            edited["settings"]["profile"] = "vedic-default".into();
        } else if field == "format" {
            edited["format"] = "teistro.receipt/2".into();
            let refused = Receipt::from_value(&edited, "receipt").unwrap_err();
            assert_eq!(refused.status, Status::SchemaVersion);
            assert_eq!(refused.field(), Some("receipt.format"));
            continue;
        } else {
            edited[field] = value;
        }
        let edited = Receipt::from_value(&edited, "receipt").unwrap();
        let check = sdk.verify().receipt(&edited);
        assert_eq!(check.verdict, ReceiptVerdict::Broken, "{field}");
        let expected: &[&str] = if field == "settings" {
            &["digest"]
        } else {
            broken
        };
        assert_eq!(check.broken, expected, "{field}");
    }
    // A knob changed under the snapshot's hash breaks the hash too.
    let receipt = sankalpa(&sdk);
    let hash = receipt.settings.hash;
    let mut knob = resettled(receipt, |settings| {
        settings.frame.node = if settings.frame.node == teistro::settings::Node::True {
            teistro::settings::Node::Mean
        } else {
            teistro::settings::Node::True
        };
    });
    knob.settings.hash = hash;
    assert_eq!(
        sdk.verify().receipt(&knob).broken,
        ["settings.hash", "digest"]
    );
}

/// A change sealed again is caught by the re-run: another value, another
/// request, other settings, another record.
#[test]
fn a_change_sealed_again_is_caught_by_the_rerun() {
    let sdk = context("nepali-drik");
    let mut output = sankalpa(&sdk);
    output.output_digest = Hash::of(b"another value");
    let check = sdk.verify().receipt(&output.sealed());
    assert_eq!(check.verdict, ReceiptVerdict::Differs);
    assert_eq!(moved(&check), ["output"]);
    assert_eq!(
        check.moved[0].issued,
        Hash::of(b"another value").to_string()
    );

    let mut request = sankalpa(&sdk);
    request.request["at"] = 2_461_056.25.into();
    assert_eq!(
        sdk.verify().receipt(&request.sealed()).verdict,
        ReceiptVerdict::Differs
    );

    // Another ayanamsha, the snapshot hashed again: every sidereal
    // longitude of a chart moves. (A sankalpa's names may well not.)
    let examples: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("requests.json")).unwrap();
    let found = examples
        .iter()
        .find(|example| example["record"] == "chart.found")
        .unwrap();
    let settings = resettled(issued(&sdk, "chart.found", &found["request"]), |settings| {
        settings.frame.ayanamsha = teistro::catalogue::Ayanamsha::Raman.into();
    });
    assert_eq!(
        sdk.verify().receipt(&settings.sealed()).verdict,
        ReceiptVerdict::Differs
    );

    let mut record = converted(&sdk);
    record.record = String::from("calendar.nothing");
    assert_eq!(
        cause(&sdk.verify().receipt(&record.sealed())),
        NotRerunCause::Record
    );
}

/// A receipt from another SDK version that computes the same value holds,
/// and says the version moved, with both sides (C1171).
#[test]
fn another_sdk_version_is_reported_as_such() {
    let sdk = context("nepali-drik");
    let mut older = sankalpa(&sdk);
    older.sdk_version = Version::new(0, 0, 1);
    older.calculation_version = 0;
    let check = sdk.verify().receipt(&older.sealed());
    assert_eq!(check.verdict, ReceiptVerdict::Held);
    assert_eq!(moved(&check), ["sdkVersion", "calculationVersion"]);
    assert_eq!(check.moved[0].issued, "0.0.1");
    // A pre-release keeps its label, as Cargo wrote it.
    assert_eq!(check.moved[0].now, env!("CARGO_PKG_VERSION"));
    assert_eq!(check.issued.sdk_version, Version::new(0, 0, 1));
    assert_eq!(
        check.now.sdk_version,
        Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
    );

    // A provider of another version moves too, and still computes.
    let mut provider = sankalpa(&sdk);
    provider.provider.as_mut().unwrap().version = String::from("0.0.0-old");
    let check = sdk.verify().receipt(&provider.sealed());
    assert_eq!(check.verdict, ReceiptVerdict::Held);
    assert_eq!(moved(&check), ["provider"]);
}

/// A receipt is re-run under its own settings, not the verifier's.
#[test]
fn a_receipt_is_answered_again_under_its_own_settings() {
    let kp = context("kp-default");
    let receipt = sankalpa(&kp);
    let other = context("nepali-drik");
    assert_ne!(other.settings_hash(), receipt.settings.hash);
    assert_eq!(
        other.verify().receipt(&receipt).verdict,
        ReceiptVerdict::Held
    );
}

/// What cannot be re-run says why.
#[test]
fn what_cannot_be_answered_again_says_why() {
    let sdk = context("nepali-drik");
    let sky = sankalpa(&sdk);

    let blind = Context::builder().profile("nepali-drik").build().unwrap();
    let check = blind.verify().receipt(&sky);
    assert_eq!(cause(&check), NotRerunCause::Provider);
    assert!(check.now.provider.is_none());
    // A receipt that read no sky is answered without one.
    assert_eq!(
        blind.verify().receipt(&converted(&sdk)).verdict,
        ReceiptVerdict::Held
    );

    let mut named = sankalpa(&sdk);
    named.provider = Some(ReceiptProvider {
        name: String::from("another-engine"),
        version: String::from("1"),
        data_version: String::from("1"),
    });
    assert_eq!(
        cause(&sdk.verify().receipt(&named.sealed())),
        NotRerunCause::Provider
    );

    let mut packs = sankalpa(&sdk);
    packs.packs.push(PackStamp {
        id: String::from("ne-Deva-NP/extra"),
        version: String::from("1"),
        hash: Hash::of(b"a pack"),
    });
    assert_eq!(
        cause(&sdk.verify().receipt(&packs.sealed())),
        NotRerunCause::Packs
    );

    let mut locale = sankalpa(&sdk);
    locale.locale = String::from("de-Latn-DE");
    assert_eq!(
        cause(&sdk.verify().receipt(&locale.sealed())),
        NotRerunCause::Locale
    );

    let mut refused = sankalpa(&sdk);
    refused.request["latitudeDeg"] = 97.into();
    let check = sdk.verify().receipt(&refused.sealed());
    assert_eq!(cause(&check), NotRerunCause::Refused);
    assert!(check.reason.unwrap().detail.contains("latitudeDeg"));

    let mut far = issued(
        &sdk,
        "almanac.digest",
        &serde_json::json!({"first": {"year": 2025, "month": 10, "day": 1}, "days": 1,
            "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700}),
    );
    far.request["days"] = 0.into();
    assert_eq!(
        cause(&sdk.verify().receipt(&far.sealed())),
        NotRerunCause::OutOfRange
    );
}

/// The Surya Siddhanta is opened for its settings, so a receipt naming
/// another bija is not put under it (C1173).
#[test]
fn a_classical_ephemeris_opened_otherwise_is_not_shared() {
    let official = Context::builder().nepal_official([]).build().unwrap();
    let receipt = converted(&context("nepali-drik"));
    let check = official.verify().receipt(&receipt);
    assert_eq!(cause(&check), NotRerunCause::Settings, "{check:?}");

    // Its own receipt is answered again on it: the same siddhanta and
    // Delta T open nothing anew.
    let own = converted(&official);
    assert_eq!(
        official.verify().receipt(&own).verdict,
        ReceiptVerdict::Held
    );
    // Another Delta T under the same siddhanta would need it opened anew.
    let other = resettled(own, |settings| {
        let knob = &mut settings.time.delta_t;
        *knob = if *knob == teistro::settings::DeltaT::EspenakMeeus2006 {
            teistro::settings::DeltaT::StephensonMorrisonHohenkerk2016
        } else {
            teistro::settings::DeltaT::EspenakMeeus2006
        };
    });
    let check = official.verify().receipt(&other.sealed());
    assert_eq!(cause(&check), NotRerunCause::Settings, "{check:?}");
    assert!(
        check
            .reason
            .unwrap()
            .detail
            .contains("`settings.time.delta_t`"),
        "the knob is named"
    );
}

/// A receipt from a later build, whose settings carry a knob this build
/// does not have, sealed as that build seals it, is no broken receipt: its
/// digests hold to the bytes it carries, and the knob is named as why it
/// was not answered again. So is one from an earlier build that lacks a
/// knob this build has.
#[test]
fn a_receipt_from_another_builds_settings_is_not_broken() {
    let sdk = context("nepali-drik");
    let mut later = sankalpa(&sdk);
    later.settings.settings["frame"]["futureKnob"] = "LATER".into();
    later.settings.hash = content_hash(&later.settings.settings);
    let later = later.sealed();
    // Through its JSON, as a verifier is handed it.
    let read = Receipt::from_json(&teistro_core::envelope::canonical_json(&later)).unwrap();
    assert_eq!(read, later);
    let check = sdk.verify().receipt(&read);
    assert!(check.broken.is_empty(), "{check:?}");
    assert_eq!(cause(&check), NotRerunCause::Settings);
    assert!(
        check
            .reason
            .unwrap()
            .detail
            .contains("settings.frame.futureKnob"),
        "the knob is named"
    );

    let mut earlier = sankalpa(&sdk);
    earlier.settings.settings["frame"]
        .as_object_mut()
        .unwrap()
        .remove("node");
    earlier.settings.hash = content_hash(&earlier.settings.settings);
    let check = sdk.verify().receipt(&earlier.sealed());
    assert!(check.broken.is_empty(), "{check:?}");
    assert_eq!(cause(&check), NotRerunCause::Settings);
    let detail = check.reason.unwrap().detail;
    assert!(
        detail.contains("settings.frame") && detail.contains("`node`"),
        "{detail}"
    );
}

/// The receipt crosses as JSON and back without a byte moving, and its
/// records answer as the area does.
#[test]
fn the_records_answer_as_the_area_does() {
    let sdk = context("nepali-drik");
    let issue = teistro::records::record("verify.issue").unwrap();
    let asked = r#"{"record": "calendar.convert", "request": {"date": {"year": 2026, "month": 10, "day": 10}, "into": "calendar.BIKRAM_SAMBAT"}}"#;
    let answered = issue.answer(&sdk, asked).unwrap();
    assert!(answered.provenance.is_none());
    let receipt = Receipt::from_value(&answered.value["receipt"], "receipt").unwrap();
    assert_eq!(receipt, converted(&sdk));
    let check = teistro::records::record("verify.receipt")
        .unwrap()
        .answer(&sdk, &answered.value["receipt"].to_string())
        .unwrap();
    assert_eq!(check.value["verdict"], "HELD");

    let refused = issue
        .answer(&sdk, r#"{"record": "calendar.convert", "request": []}"#)
        .unwrap_err();
    assert_eq!(refused.field(), Some("issue.request"));
}

/// A dataset with three births, one rated too poorly for its study, and
/// one study under the snapshot of `sdk`'s settings.
fn dataset(sdk: &Context) -> teistro::research::dataset::Dataset {
    let snapshot = teistro_core::settings::SettingsSnapshot::of(sdk.resolved());
    let birth = |jd: f64| serde_json::json!({"instant": jd, "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20700});
    teistro::research::dataset::Dataset::from_value(
        &serde_json::json!({
            "format": "teistro.dataset/1", "name": "three births",
            "records": [
                {"id": "a", "birth": birth(2_447_000.25), "rating": "AA"},
                {"id": "b", "birth": birth(2_448_000.75), "rating": "DD"},
                {"id": "c", "birth": birth(2_449_000.5), "rating": "A"}],
            "studies": [{"id": "counts", "settings": snapshot, "minimumRating": "A",
                "groups": {"a": 0, "c": 1},
                "request": {"study": "COUNTS", "rules": {"shipped": ["YOGAS"]}}}]
        }),
        "dataset",
    )
    .unwrap()
}

/// A dataset's study is run under its own settings, not the caller's,
/// and answers as `research.study` answers the request it means.
#[test]
fn a_dataset_study_runs_under_its_own_settings() {
    let written = context("kp-default");
    let dataset = dataset(&written);
    let other = context("nepali-drik");
    let studied = other.research().dataset(&dataset, "counts").unwrap();
    assert_eq!(studied.admitted, ["a", "c"]);
    assert_eq!(studied.excluded[0].id, "b");
    let asked = dataset.study_request("counts").unwrap();
    let request =
        teistro::ResearchRequest::from_json(&teistro::canonical_json(&asked.request)).unwrap();
    let direct = written.research().request(&request).unwrap();
    assert_eq!(studied.answer, direct);

    assert_eq!(
        other
            .research()
            .dataset(&dataset, "none")
            .unwrap_err()
            .field(),
        Some("study")
    );
    let official = Context::builder().nepal_official([]).build().unwrap();
    let refused = official.research().dataset(&dataset, "counts").unwrap_err();
    assert_eq!(refused.status, Status::Unsupported);
    assert!(
        refused
            .field()
            .is_some_and(|field| field.starts_with("dataset.studies[0].settings.")),
        "{refused}"
    );
}
