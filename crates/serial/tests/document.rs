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

use teistro_aspect::Aspects;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::Gregorian;
use teistro_calendar::solar::drik::DrikSun;
use teistro_chart::foundation::{ChartFoundation, Founder};
use teistro_core::catalogue::{Ayanamsha, ChartKind, Varga};
use teistro_core::envelope::{Envelope, Hash, Provenance};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{OverridePolicy, Profile, Settings, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_houses::Houses;
use teistro_points::Points;
use teistro_port_ephemeris::test_provider::TestProvider;
use teistro_serial::canonical::{hash_of, to_hash_form, to_rendered};
use teistro_serial::{Document, Sealed};
use teistro_state::state;
use teistro_vargas::chart::{Axis, chart as varga_chart};

fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

fn founded() -> (Envelope<ChartFoundation>, Settings) {
    let provider = TestProvider;
    let resolved = Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
        .expect("the default profile")
        .resolve(&SettingsPatch::default())
        .expect("it resolves");
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let founded = Founder::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    )
    .found_one(
        JulianDay::<Utc>::literal(2_460_482.5),
        &place(),
        ChartKind::Natal,
    )
    .expect("a founded chart");
    (founded, resolved.settings)
}

/// Every section the layer can produce, on one chart.
fn whole() -> (Document, Provenance, Settings) {
    let (envelope, settings) = founded();
    let provenance = envelope.provenance.clone();
    let foundation = envelope.value;
    let document = Document::of(foundation.clone())
        .with_varga(varga_chart(&foundation, Axis::of(Varga::D9)).expect("a navamsha"))
        .with_state(state(&foundation, &settings).expect("a state"))
        .with_aspects(Aspects::of(&foundation, &settings).expect("the aspects"))
        .with_points(Points::from_longitudes(&foundation).expect("the points"))
        .with_houses(Houses::of(&foundation).expect("the houses"));
    (document, provenance, settings)
}

#[test]
fn every_section_of_the_layer_serialises() {
    let (document, _, _) = whole();
    let sections = document.sections();
    println!("{} sections: {sections:?}", sections.len());
    assert_eq!(
        sections,
        vec![
            "foundation",
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
    let (document, provenance, _) = whole();
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
    let (document, provenance, settings) = whole();
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
    let (again, _, _) = whole();
    assert_eq!(whole_hash, hash_of(&again), "the determinism contract");
    let _ = provenance;
}

#[test]
fn the_canonical_form_of_a_real_document_is_canonical() {
    let (document, _, _) = whole();
    let text = to_hash_form(&document);
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("it reads back");

    // The form is a **fixed point**: writing what it wrote gives the
    // same bytes. That is the invariant, and it subsumes "no whitespace
    // between tokens" without mistaking a space inside a string value
    // for one.
    assert_eq!(to_hash_form(&parsed), text, "a fixed point");
    assert_eq!(text, to_hash_form(&document), "and stable");

    assert!(sorted(&parsed), "keys in code-point order at every depth");

    // And every number in it is written without an exponent, which is
    // the rule a binding has to match.
    let mut numbers = 0;
    walk(&parsed, &mut |number| {
        numbers += 1;
        let written = teistro_serial::canonical::decimal(number, 12);
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
    let (document, _, settings) = whole();
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
    let (document, provenance, _) = whole();
    let sealed = Sealed::new(document, provenance);
    let hash = sealed.content_hash();
    let (value, provenance) = sealed.into_parts();
    assert_eq!(provenance.content_hash, hash);
    assert_eq!(hash_of(&value), hash, "the hash still belongs to it");
}
