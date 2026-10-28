//! The gochar sidecar (`docs/03-design/gochar-sidecar.md` §9): the same
//! chart gives the same bytes, a stored sidecar reads back as itself in any
//! JSON spelling, and one that does not belong is refused by its field. The
//! chart is the corpus's first birth with every section, as `cargo xtask
//! dossier` writes it.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use serde_json::Value;
use teistro_core::envelope::{Hash, Provenance, Version};
use teistro_core::error::Status;
use teistro_serial::canonical::from_hash_form;
use teistro_serial::schema::{GOCHAR_SIDECAR, GOCHAR_SIDECAR_ID};
use teistro_serial::{Document, GocharSidecar, Sealed, hash_of};

fn chart() -> Document {
    from_hash_form(include_str!("dossier-chart.json")).unwrap()
}

fn sealed(document: Document) -> Sealed<Document> {
    document.seal(Provenance::new(
        Version::new(0, 1, 0),
        1,
        1,
        "conformance-baseline",
        Hash::of(b"settings"),
        Hash::of(b"input"),
    ))
}

fn cut(document: Document) -> (Sealed<Document>, GocharSidecar) {
    let sealed = sealed(document);
    let sidecar = GocharSidecar::cut(sealed.value(), sealed.provenance()).unwrap();
    (sealed, sidecar)
}

/// The sidecar's text with one top-level member replaced.
fn with(text: &str, key: &str, value: Value) -> String {
    let mut parsed: Value = serde_json::from_str(text).unwrap();
    parsed[key] = value;
    parsed.to_string()
}

#[test]
fn the_same_chart_gives_the_same_bytes() {
    let (_, once) = cut(chart());
    let (_, twice) = cut(chart());
    assert_eq!(once.to_text(), twice.to_text());
    assert_eq!(once, twice);
}

#[test]
fn it_ties_itself_to_its_chart_and_settings() {
    let (sealed, sidecar) = cut(chart());
    assert_eq!(sidecar.chart(), sealed.content_hash());
    assert_eq!(sidecar.settings(), Hash::of(b"settings"));
    assert_eq!(sidecar.natal_hash(), hash_of(&sidecar.natal()));
    // The natal document is the chart's foundation and panchanga, whole.
    let natal = sidecar.natal();
    assert_eq!(natal.foundation, sealed.value().foundation);
    assert_eq!(natal.panchanga, sealed.value().panchanga);
    assert_eq!(natal.sections(), GocharSidecar::SECTIONS);
}

#[test]
fn a_stored_sidecar_reads_back_as_itself() {
    let (_, sidecar) = cut(chart());
    let text = sidecar.to_text();
    let read = GocharSidecar::read(&text).unwrap();
    assert_eq!(read, sidecar);
    assert_eq!(read.to_text(), text);
}

#[test]
fn any_json_spelling_reads_back_to_the_same_bytes() {
    // A database's binary JSON column gives keys back in its own order
    // and with its own spacing; the hash, not the bytes, decides.
    let (_, sidecar) = cut(chart());
    let text = sidecar.to_text();
    let parsed: Value = serde_json::from_str(&text).unwrap();
    let mut keys: Vec<&String> = parsed.as_object().unwrap().keys().collect();
    keys.sort_by(|a, b| b.cmp(a));
    let respelled = format!(
        "{{\n  {}\n}}",
        keys.iter()
            .map(|key| format!(
                "\"{key}\" : {}",
                serde_json::to_string_pretty(&parsed[key.as_str()]).unwrap()
            ))
            .collect::<Vec<_>>()
            .join(",\n  ")
    );
    assert_ne!(respelled, text);
    assert_eq!(GocharSidecar::read(&respelled).unwrap().to_text(), text);
}

#[test]
fn a_sidecar_of_another_chart_is_refused() {
    let (sealed, sidecar) = cut(chart());
    assert!(sidecar.check_chart(sealed.content_hash()).is_ok());
    let stale = sidecar
        .check_chart(Hash::of(b"the chart recomputed"))
        .unwrap_err();
    assert_eq!(stale.status, Status::InvalidArg);
    assert_eq!(stale.field(), Some("sidecar.chart"));
}

#[test]
fn a_provenance_of_another_document_cuts_nothing() {
    let (sealed, _) = cut(chart());
    let mut document = sealed.value().clone();
    document.panchanga = None;
    let refused = GocharSidecar::cut(&document, sealed.provenance()).unwrap_err();
    assert_eq!(refused.field(), Some("provenance.content_hash"));
}

#[test]
fn an_edited_foundation_is_refused_by_its_hash() {
    let (_, sidecar) = cut(chart());
    let text = sidecar.to_text();
    let mut foundation = serde_json::to_value(sidecar.foundation()).unwrap();
    let lagna = foundation["lagna_deg"].as_f64().unwrap();
    foundation["lagna_deg"] = Value::from(lagna + 1.0);
    let refused = GocharSidecar::read(&with(&text, "foundation", foundation)).unwrap_err();
    assert_eq!(refused.status, Status::InvalidArg);
    assert_eq!(refused.field(), Some("sidecar.natal_hash"));
}

#[test]
fn another_format_is_refused_as_another_shape() {
    let (_, sidecar) = cut(chart());
    let later = with(&sidecar.to_text(), "format", Value::from(2));
    let refused = GocharSidecar::read(&later).unwrap_err();
    assert_eq!(refused.status, Status::SchemaVersion);
    assert_eq!(refused.field(), Some("sidecar.format"));
}

#[test]
fn a_key_it_does_not_have_is_refused_by_its_path() {
    let (_, sidecar) = cut(chart());
    let extra = with(&sidecar.to_text(), "extra", Value::from(1));
    let refused = GocharSidecar::read(&extra).unwrap_err();
    assert_eq!(refused.field(), Some("sidecar.extra"));
    let missing = GocharSidecar::read(r#"{"format": 1}"#).unwrap_err();
    assert_eq!(missing.status, Status::InvalidArg);
}

#[test]
fn a_chart_without_a_panchanga_cuts_a_sidecar_without_one() {
    let bare = Document::of(chart().foundation);
    let (_, sidecar) = cut(bare);
    assert!(sidecar.panchanga().is_none());
    assert_eq!(sidecar.natal().sections(), ["foundation"]);
    let text = sidecar.to_text();
    assert!(!text.contains(r#""panchanga":"#));
    assert_eq!(GocharSidecar::read(&text).unwrap(), sidecar);
}

#[test]
fn the_text_validates_against_its_schema() {
    let schema: Value = serde_json::from_str(GOCHAR_SIDECAR).unwrap();
    assert_eq!(schema["$id"], GOCHAR_SIDECAR_ID);
    let mut schemas = boon::Schemas::new();
    let mut compiler = boon::Compiler::new();
    compiler.add_resource(GOCHAR_SIDECAR_ID, schema).unwrap();
    let index = compiler.compile(GOCHAR_SIDECAR_ID, &mut schemas).unwrap();
    for document in [chart(), Document::of(chart().foundation)] {
        let (_, sidecar) = cut(document);
        let instance: Value = serde_json::from_str(&sidecar.to_text()).unwrap();
        assert!(
            schemas.validate(&instance, index).is_ok(),
            "the sidecar does not validate against its schema"
        );
    }
}

#[cfg(feature = "schema")]
#[test]
fn the_checked_in_schema_is_what_the_type_generates() {
    assert_eq!(
        teistro_serial::schema::generate_sidecar().unwrap(),
        GOCHAR_SIDECAR
    );
}
