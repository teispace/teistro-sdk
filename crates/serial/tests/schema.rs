//! The document schema against the documents it describes
//! (`docs/03-design/document-schema.md` §5).
//!
//! Two properties, and the direction of the second is the point:
//!
//! - **every sample validates**, sealed and written in the canonical form
//!   a consumer stores;
//! - **the schema is never stricter than the reader**: each kind of claim
//!   the schema makes is made to fire on a document the reader also
//!   refuses. The converse does not hold and is not asked for — a D9 that
//!   says it divides a sign eleven times is well-formed JSON the reader
//!   refuses on meaning, which no schema can express.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own fixtures"
)]

#[path = "../examples/documents.rs"]
#[allow(
    dead_code,
    unreachable_pub,
    reason = "the example is a program as well as a builder; this test uses the builder half"
)]
mod sample;

use serde_json::{Value, json};
use teistro_serial::canonical::{from_hash_form, to_hash_form};
use teistro_serial::schema::{DOCUMENT, DOCUMENT_ID};
use teistro_serial::{Document, Sealed};

/// The checked-in schema, compiled once per test.
struct Validator {
    schemas: boon::Schemas,
    index: boon::SchemaIndex,
}

impl Validator {
    fn new() -> Validator {
        let schema: Value = serde_json::from_str(DOCUMENT).expect("the schema is JSON");
        let mut schemas = boon::Schemas::new();
        let mut compiler = boon::Compiler::new();
        compiler
            .add_resource(DOCUMENT_ID, schema)
            .expect("the schema is a resource");
        let index = compiler
            .compile(DOCUMENT_ID, &mut schemas)
            .unwrap_or_else(|error| panic!("the schema does not compile: {error}"));
        Validator { schemas, index }
    }

    fn check(&self, instance: &Value) -> Result<(), String> {
        self.schemas
            .validate(instance, self.index)
            .map_err(|error| format!("{error:#}"))
    }
}

/// A sample sealed and written as a consumer stores it, parsed back into
/// a JSON value.
fn stored(document: Document) -> Value {
    let (_, provenance) = sample::whole();
    let written = to_hash_form(&Sealed::new(document, provenance));
    serde_json::from_str(&written).expect("the canonical form is JSON")
}

/// Whether this build reads a stored document.
fn reader_accepts(stored: &Value) -> bool {
    from_hash_form::<Sealed<Document>>(&stored.to_string()).is_ok()
}

#[test]
fn the_schema_is_this_release_s() {
    let schema: Value = serde_json::from_str(DOCUMENT).unwrap();
    assert_eq!(schema["$id"], DOCUMENT_ID);
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(schema["required"], json!(["value", "provenance"]));
}

#[test]
fn every_sample_validates_as_it_is_stored() {
    let validator = Validator::new();
    for (name, document) in sample::samples() {
        let stored = stored(document);
        validator
            .check(&stored)
            .unwrap_or_else(|error| panic!("{name} does not validate: {error}"));
        assert!(reader_accepts(&stored), "{name} does not read back");
    }
}

/// One way to break a stored document, and what it breaks.
type Breakage = (&'static str, fn(&mut Value));

#[test]
fn the_schema_is_never_stricter_than_the_reader() {
    let breakages: [Breakage; 6] = [
        ("a catalogue key no catalogue has", |doc| {
            doc["value"]["state"][0]["graha"] = json!("MARZ");
        }),
        ("a count past its integer's range", |doc| {
            doc["value"]["state"][0]["house"] = json!(300);
        }),
        ("a number written as a string", |doc| {
            doc["value"]["foundation"]["lagna_deg"] = json!("twelve");
        }),
        ("a tag no variant carries", |doc| {
            doc["value"]["houses"]["outcome"]["kind"] = json!("NEVER");
        }),
        ("a required section left out", |doc| {
            doc["value"].as_object_mut().unwrap().remove("foundation");
        }),
        ("a hash that is not sixty-four hex digits", |doc| {
            doc["provenance"]["content_hash"] = json!("not-a-hash");
        }),
    ];
    let validator = Validator::new();
    let (document, _) = sample::whole();
    let whole = stored(document);
    for (what, break_it) in breakages {
        let mut broken = whole.clone();
        break_it(&mut broken);
        assert_ne!(broken, whole, "{what}: the fixture has the path");
        assert!(
            validator.check(&broken).is_err(),
            "{what}: the schema let it through"
        );
        assert!(
            !reader_accepts(&broken),
            "{what}: the schema refuses what the reader reads"
        );
    }

    // And what neither refuses: a section a newer build wrote. A schema
    // that refused it would make every stored chart unreadable to the
    // tools of every release but its own.
    let mut newer = whole;
    newer["value"]["a_section_from_a_newer_build"] = json!(1);
    assert!(
        validator.check(&newer).is_ok(),
        "the schema refuses a newer section"
    );
    assert!(reader_accepts(&newer), "the reader refuses a newer section");
}

#[cfg(feature = "schema")]
#[test]
fn the_checked_in_schema_is_what_the_types_generate() {
    // `check-document-schema` is the gate; this is the same question
    // asked where a crate-only `cargo test --features schema` sees it.
    assert_eq!(teistro_serial::schema::generate().unwrap(), DOCUMENT);
}
