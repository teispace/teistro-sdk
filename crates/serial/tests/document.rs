//! A whole chart document, sealed, over the analytic test provider.
//!
//! This is the first thing that puts every Phase 4 value in one place:
//! the foundation, the divisional charts, the planetary state, the
//! aspects, the derived points and the houses. What it holds is that
//! they all serialise, that the whole seals to its own hash, and that
//! the hash moves when any section does.
//!
//! It is also the test that would have caught what the falsification
//! pass found by reading the source: before this crate, the foundation
//! every other value is computed from **could not be serialised at
//! all**, and the hash a founded chart carried was the hash of the empty
//! string.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index their own results and print counts under --nocapture"
)]

// The chart every sample is founded on is built **once**, in the example
// the schema pass runs, and included here as a module rather than built
// a second time. Two copies of a founding would drift, and the pass and
// this test would then be measuring different charts.
#[path = "../examples/documents.rs"]
#[allow(
    dead_code,
    unreachable_pub,
    reason = "the example is a program as well as a builder; this test uses the builder half"
)]
mod sample;

use sample::{founded, whole};
use teistro_core::envelope::{Digits, Hash};
use teistro_serial::canonical::{hash_of, to_hash_form, to_rendered};
use teistro_serial::{Document, Sealed};
use teistro_state::state;

#[test]
fn every_section_of_the_layer_serialises() {
    let (document, _) = whole();
    let sections = document.sections();
    println!("{} sections: {sections:?}", sections.len());
    assert_eq!(
        sections,
        vec![
            "foundation",
            "panchanga",
            "vargas",
            "state",
            "aspects",
            "points",
            "houses"
        ]
    );
    let text = to_hash_form(&document);
    assert!(!text.is_empty(), "the document writes");
    // Every section's name appears as a key, so nothing was quietly
    // dropped by a `skip_serializing_if`.
    for section in &sections {
        assert!(text.contains(&format!("\"{section}\"")), "{section}");
    }
    // And the foundation, which before this crate could not serialise at
    // all, carries its own contents.
    assert!(text.contains("\"grahas\""), "the foundation's own bodies");
    println!("{} bytes of canonical form", text.len());
}

#[test]
fn a_sealed_document_carries_its_own_hash() {
    let (document, provenance) = whole();
    assert_eq!(
        provenance.content_hash,
        Hash::of(&[]),
        "the founder leaves the placeholder, which is what the pass found"
    );
    let sealed = document.seal(provenance);
    assert!(sealed.is_intact());
    assert_ne!(sealed.content_hash(), Hash::of(&[]));
    assert_eq!(sealed.content_hash(), hash_of(sealed.value()));
    // And the bytes it hashed are the ones it gives back.
    assert_eq!(
        sealed.content_hash(),
        Hash::of(sealed.to_hash_form().as_bytes())
    );
}

#[test]
fn the_hash_moves_when_any_section_does() {
    let (document, provenance) = whole();
    let (_, settings) = founded();
    let whole_hash = hash_of(&document);
    // A document with one section fewer is a different answer.
    let (envelope, _) = founded();
    let fewer = Document::of(envelope.value.clone())
        .with_state(state(&envelope.value, &settings).expect("a state"));
    assert_ne!(whole_hash, hash_of(&fewer));
    // Only the foundation is a different answer again.
    let bare = Document::of(envelope.value);
    assert_ne!(hash_of(&fewer), hash_of(&bare));
    assert_eq!(bare.sections(), vec!["foundation"]);
    // And two builds of the same document agree.
    let (again, _) = whole();
    assert_eq!(whole_hash, hash_of(&again), "the determinism contract");
    let _ = provenance;
}

#[test]
fn the_canonical_form_of_a_real_document_is_canonical() {
    let (document, _) = whole();
    let text = to_hash_form(&document);
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("it reads back");

    // The form is **stable**: writing the same value twice gives the same
    // bytes.
    assert_eq!(text, to_hash_form(&document), "stable");

    // It is a **fixed point**: every number it writes reads back as the
    // very same double, so a consumer that stores a document and hashes
    // it again gets the producer's hash. The grammar earned that by
    // writing the shortest decimal that round-trips rather than a fixed
    // twelve — at a Julian day's magnitude an `f64` resolves about nine,
    // and the three extra were the expansion of a binary value rather
    // than information (`03-design/schema-measured.md` §9).
    //
    // Measured against a **correctly rounded** parser, which is what
    // `str::parse`, JavaScript's `JSON.parse` and Python's `json` all
    // are. `serde_json`'s own number path is not one: it lands a unit in
    // the last place low on about one number in fifteen, which is why
    // this walks the text rather than a `serde_json::Value`.
    let mut checked = 0;
    for token in text.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')) {
        let Ok(value) = token.parse::<f64>() else {
            continue;
        };
        checked += 1;
        let written = teistro_serial::canonical::decimal(value, Digits::Shortest);
        let read: f64 = written.parse().expect("what the grammar wrote parses");
        assert_eq!(
            read.to_bits(),
            value.to_bits(),
            "{written} did not read back as itself"
        );
    }
    assert!(checked > 100, "only {checked} numbers were checked");

    assert!(sorted(&parsed), "keys in code-point order at every depth");

    // And every number in it is written without an exponent, which is
    // the rule a binding has to match.
    let mut numbers = 0;
    walk(&parsed, &mut |number| {
        numbers += 1;
        let written = teistro_serial::canonical::decimal(number, Digits::Shortest);
        assert!(
            !written.contains('e') && !written.contains('E'),
            "{number} wrote as {written}"
        );
    });
    println!(
        "{numbers} numbers, none with an exponent, in {} bytes",
        text.len()
    );
    assert!(numbers > 100, "a real chart carries a good many");
}

/// Every number in a value.
fn walk(value: &serde_json::Value, seen: &mut impl FnMut(f64)) {
    match value {
        serde_json::Value::Number(number) => {
            if let Some(double) = number.as_f64() {
                seen(double);
            }
        }
        serde_json::Value::Array(items) => items.iter().for_each(|item| walk(item, seen)),
        serde_json::Value::Object(fields) => {
            fields.values().for_each(|field| walk(field, seen));
        }
        _ => {}
    }
}

fn sorted(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(fields) => {
            let keys: Vec<&String> = fields.keys().collect();
            keys.windows(2).all(|pair| pair[0] <= pair[1]) && fields.values().all(sorted)
        }
        serde_json::Value::Array(items) => items.iter().all(sorted),
        _ => true,
    }
}

#[test]
fn a_rendering_reads_the_precision_knob_the_hash_form_ignores() {
    let (document, _) = whole();
    let (_, settings) = founded();
    let rendered = to_rendered(&document, &settings.output.precision).expect("a rendering");
    let hashed = to_hash_form(&document);
    // The default profile asks for nine decimals of an angle; the hash
    // form writes twelve, so the two differ on a real chart.
    assert_ne!(rendered, hashed, "the knob does something");
    assert!(rendered.len() < hashed.len(), "and it shortens");
    // Both read back as the same shape.
    let a: serde_json::Value = serde_json::from_str(&rendered).expect("it reads");
    let b: serde_json::Value = serde_json::from_str(&hashed).expect("it reads");
    assert_eq!(
        a.as_object().map(serde_json::Map::len),
        b.as_object().map(serde_json::Map::len)
    );
    println!(
        "{} bytes rendered against {} hashed",
        rendered.len(),
        hashed.len()
    );
}

#[test]
fn a_sealed_document_gives_up_its_parts_intact() {
    let (document, provenance) = whole();
    let sealed = Sealed::new(document, provenance);
    let hash = sealed.content_hash();
    let (value, provenance) = sealed.into_parts();
    assert_eq!(provenance.content_hash, hash);
    assert_eq!(hash_of(&value), hash, "the hash still belongs to it");
}
