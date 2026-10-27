//! Every part a façade section reads is answered: a refusal, or a part
//! read by a name the answer does not use, would leave the section silently
//! short and the gates counting less than they say.

#![allow(clippy::unwrap_used, reason = "a test fails by panicking")]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

use teistro_scenario::{ASKED, SECTIONS, section};

#[test]
fn every_part_a_facade_section_reads_is_answered() {
    let record = teistro::records::record("chart.found").unwrap();
    let sdk = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .unwrap();
    for asked in ASKED {
        assert!(SECTIONS.contains(&asked.name), "{}", asked.name);
        let answered = record.answer(&sdk, asked.request).unwrap();
        for part in asked.parts {
            let value = answered.value.get(*part);
            assert!(
                value.is_some_and(|v| match v {
                    serde_json::Value::Array(items) => !items.is_empty(),
                    serde_json::Value::Object(fields) => !fields.is_empty(),
                    _ => false,
                }),
                "{}: `{part}` is not answered",
                asked.name
            );
        }
        assert!(section(asked.name).is_some_and(|s| !s.values.is_empty()));
    }
}
