//! Serialise-deserialise fixed points over real skies
//! (`docs/03-design/property-coverage.md` §4 step 3): the stored chart, the
//! document a consumer keeps, read back from its own canonical bytes for
//! every birth of the golden corpus with every section and every
//! divisional chart, where the serial crate's own test holds three samples.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

mod common;

use teistro::{ChartRequest, Context, Document, Ephemeris, canonical_json};

/// The corpus's tags for a birth at an edge of the data's years, where a
/// section may need an instant past the ephemeris's span.
const EDGES: [&str; 2] = ["range-start", "range-end"];

#[test]
fn every_corpus_chart_reads_back_as_the_document_it_was() {
    // The corpus's polar birth has a day only under a polar policy, and
    // a document without its day would hold less than the widest.
    let sdk = Context::builder()
        .profile("conformance-baseline")
        .settings_json(r#"{"day": {"polar_day_policy": "CIVIL_MIDNIGHT"}}"#)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the conformance profile and the built-in ephemeris");
    let (mut edges, mut refused) = (Vec::new(), Vec::new());
    for (name, chart) in common::charts() {
        let at_edge = chart["tags"]
            .as_array()
            .is_some_and(|tags| tags.iter().any(|t| EDGES.iter().any(|e| t == e)));
        if at_edge {
            edges.push(name.clone());
        }
        let document = match common::reading_of(&sdk, &chart, ChartRequest::with_everything) {
            Ok(document) => document,
            Err(error) => {
                assert!(
                    at_edge,
                    "{name} refused away from the range's edges: {error}"
                );
                refused.push(name);
                continue;
            }
        };
        let written = canonical_json(&document);
        let back: Document = serde_json::from_str(&written)
            .unwrap_or_else(|error| panic!("{name} did not read back: {error}"));
        // The bytes, which a stored chart's content hash is taken over.
        assert_eq!(canonical_json(&back), written, "{name}: the bytes moved");
        // The value, which is stronger: nothing was dropped on the way.
        assert_eq!(back, document, "{name}: the value moved");
    }
    // The widest document asks the Sun for instants away from the birth
    // (a year before c047's, six weeks after c048's), so the two edge
    // births are refused, and every other birth of the corpus read back.
    assert_eq!(refused, edges);
    assert_eq!(edges.len(), 2, "{edges:?}");
}
