//! A consumer's own dasha system, end to end through the façade: registered on
//! the builder, named by its key, asked for by its id, carried in the document
//! with its definition, and rebuilt from a stored document by a context that
//! never registered it (`docs/03-design/dasha-kernels.md`, "A consumer's own
//! system").
//!
//! Both kernels: the nakshatra-seeded system registered here has
//! Vimshottari's table under a consumer's key and the sign-based one has
//! Chara's row, so every answer has an exact twin — the catalogued system
//! read beside it on the same chart. A twin is the strongest thing a test
//! of a registration path can assert: the periods are not merely plausible,
//! they are a shipped system's to the last digit.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{DashaSystem, Nakshatra};
use teistro::dasha::{DashaName, RashiDefinition, UduDefinition, VIMSHOTTARI};
use teistro::quantity::{Altitude, Depth, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, Timeline, UtcOffset};

/// Vimshottari's table under a consumer's key.
fn twin() -> UduDefinition {
    UduDefinition {
        lords: VIMSHOTTARI.lords.to_vec(),
        sources: vec![String::from("Vimshottari's table, for the test")],
        ..UduDefinition::of("ACME_VIMSHOTTARI", Nakshatra::Ashwini)
    }
}

/// Chara's row under a consumer's key: from the lagna, every sign in turn,
/// each counted to its stronger lord.
fn sign_twin() -> RashiDefinition {
    RashiDefinition {
        sources: vec![String::from("Chara's row, for the test")],
        ..RashiDefinition::of("ACME_CHARA")
    }
}

fn context(register: bool) -> Context {
    let builder = Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin]);
    let builder = if register {
        builder.dasha_system(twin()).dasha_system(sign_twin())
    } else {
        builder
    };
    builder.build().expect("a built context")
}

fn request() -> ChartRequest {
    ChartRequest::at(
        Place::new(
            Latitude::try_new(27.7172).unwrap(),
            Longitude::try_new(85.324).unwrap(),
            Altitude::try_new(1400.0).unwrap(),
        ),
        UtcOffset::try_from_seconds(20_700).unwrap(),
    )
}

const BIRTH: f64 = 2_447_995.489_583_333_5;

#[test]
fn a_registered_system_is_read_as_its_twin_under_its_own_name() {
    let sdk = context(true);
    let id = sdk.keys().id("dasha_system.ACME_VIMSHOTTARI").unwrap();
    assert!(id.is_registered());
    assert_eq!(
        sdk.keys().name(id).unwrap(),
        "dasha_system.ACME_VIMSHOTTARI"
    );
    let asked = request().with_dashas([id, DashaSystem::Vimshottari.key_id()]);
    let document = sdk
        .chart()
        .reading(JulianDay::<Utc>::literal(BIRTH), &asked)
        .unwrap()
        .value;
    let [consumer, shipped] = [&document.dashas[0], &document.dashas[1]];
    assert_eq!(
        consumer.system,
        DashaName::Registered(String::from("ACME_VIMSHOTTARI"))
    );
    assert_eq!(
        consumer.definition.as_ref().and_then(|d| d.udu()),
        Some(&twin())
    );
    assert_eq!(shipped.system, DashaSystem::Vimshottari);
    assert!(shipped.definition.is_none());
    assert_eq!(consumer.balance, shipped.balance);
    assert_eq!(consumer.periods, shipped.periods);
    assert_eq!(consumer.depth, twin().depth);
}

#[test]
fn a_stored_document_rebuilds_the_system_without_its_registration() {
    let sdk = context(true);
    let id = sdk.keys().id("dasha_system.ACME_VIMSHOTTARI").unwrap();
    let document = sdk
        .chart()
        .reading(
            JulianDay::<Utc>::literal(BIRTH),
            &request().with_dashas([id, DashaSystem::Vimshottari.key_id()]),
        )
        .unwrap()
        .value;
    let stored = serde_json::to_string(&document).unwrap();
    assert!(
        stored.contains(r#""system":"ACME_VIMSHOTTARI""#),
        "the key, bare"
    );
    let restored: Document = serde_json::from_str(&stored).unwrap();
    assert_eq!(restored.dashas, document.dashas);

    // A context that registered nothing still rebuilds it, from the
    // definition the document carries.
    let plain = context(false);
    let consumer = plain.chart().dasha(&restored, "ACME_VIMSHOTTARI").unwrap();
    let shipped = plain
        .chart()
        .dasha(&restored, DashaSystem::Vimshottari)
        .unwrap();
    let depth = Depth::try_new(5).unwrap();
    for years in [0.0, 12.3, 47.9, 101.5] {
        let at = JulianDay::<Utc>::literal(BIRTH + years * 365.25);
        assert_eq!(
            consumer.at(at, depth),
            shipped.at(at, depth),
            "{years} years on"
        );
    }
    // A registered name the document does not carry is refused by name.
    let missing = plain.chart().dasha(&restored, "ACME_OTHER").unwrap_err();
    assert_eq!(missing.field(), Some("system"));
}

/// The other kernel, by the same route: a sign-based system a consumer
/// registers answers exactly as the catalogued row it copies.
///
/// This is what `dasha-coverage-measured.md` found missing. Without it a
/// school's Sthira or Varnada is shut to a consumer who **has** the text, as
/// firmly as it is to this build — a dead end in the SDK rather than a gap
/// in the sources.
#[test]
fn a_registered_sign_based_system_is_read_as_its_twin() {
    let sdk = context(true);
    let id = sdk.keys().id("dasha_system.ACME_CHARA").unwrap();
    assert!(id.is_registered());
    let asked = request().with_dashas([id, DashaSystem::Chara.key_id()]);
    let document = sdk
        .chart()
        .reading(JulianDay::<Utc>::literal(BIRTH), &asked)
        .unwrap()
        .value;
    let [consumer, shipped] = [&document.dashas[0], &document.dashas[1]];
    assert_eq!(
        consumer.system,
        DashaName::Registered(String::from("ACME_CHARA"))
    );
    assert_eq!(
        consumer.definition.as_ref().and_then(|d| d.rashi()),
        Some(&sign_twin())
    );
    assert_eq!(shipped.system, DashaSystem::Chara);
    assert!(shipped.definition.is_none());
    // A sign-based dasha has no seed and no balance, whoever registered it.
    assert!(consumer.seed.is_none() && consumer.balance.is_none());
    assert_eq!(consumer.periods, shipped.periods);
    assert_eq!(consumer.first_lord, shipped.first_lord);
}

/// A sign-based definition rebuilds through the sign-based kernel, which is
/// what stating the kernel buys: guessed from its fields, a row with no
/// lords would go to the nakshatra-seeded kernel and be refused for having
/// none.
#[test]
fn a_stored_sign_based_document_rebuilds_without_its_registration() {
    let sdk = context(true);
    let id = sdk.keys().id("dasha_system.ACME_CHARA").unwrap();
    let document = sdk
        .chart()
        .reading(
            JulianDay::<Utc>::literal(BIRTH),
            &request().with_dashas([id, DashaSystem::Chara.key_id()]),
        )
        .unwrap()
        .value;
    let stored = serde_json::to_string(&document).unwrap();
    assert!(stored.contains(r#""kernel":"RASHI""#), "the kernel, stated");
    let restored: Document = serde_json::from_str(&stored).unwrap();
    assert_eq!(restored.dashas, document.dashas);

    let plain = context(false);
    let consumer = plain.chart().dasha(&restored, "ACME_CHARA").unwrap();
    let shipped = plain.chart().dasha(&restored, DashaSystem::Chara).unwrap();
    let depth = Depth::try_new(3).unwrap();
    for years in [0.0, 12.3, 47.9, 101.5] {
        let at = JulianDay::<Utc>::literal(BIRTH + years * 365.25);
        assert_eq!(
            consumer.at(at, depth),
            shipped.at(at, depth),
            "{years} years on"
        );
    }
}

/// A sign-based row is refused by the field that is wrong, as a
/// nakshatra-seeded one is, and by its place in the builder's list.
#[test]
fn a_sign_based_definition_is_refused_by_its_place_and_field() {
    let refused = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .dasha_system(RashiDefinition {
            length: teistro::dasha::Length::Fixed(0),
            ..RashiDefinition::of("ACME_NOTHING")
        })
        .build()
        .unwrap_err();
    assert_eq!(refused.field(), Some("dashas[0].length"));

    let houses = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .dasha_system(twin())
        .dasha_system(RashiDefinition {
            stronger_of: vec![1, 13],
            ..RashiDefinition::of("ACME_THIRTEEN")
        })
        .build()
        .unwrap_err();
    assert_eq!(refused.field(), Some("dashas[0].length"));
    assert_eq!(houses.field(), Some("dashas[1].stronger_of[1]"));

    // One key space over both kernels: a name one took the other cannot.
    let taken = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .dasha_system(twin())
        .dasha_system(RashiDefinition::of("ACME_VIMSHOTTARI"))
        .build()
        .unwrap_err();
    assert_eq!(taken.field(), Some("dashas[1].key"));
}

#[test]
fn a_definition_is_refused_by_its_place_and_field_and_an_unregistered_id_by_its_place() {
    let refused = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .dasha_system(twin())
        .dasha_system(UduDefinition { span: 0, ..twin() })
        .build()
        .unwrap_err();
    // The second has a taken key before a bad span; the key is refused first.
    assert_eq!(refused.field(), Some("dashas[1].key"));
    let narrow = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .dasha_system(UduDefinition { span: 0, ..twin() })
        .build()
        .unwrap_err();
    assert_eq!(narrow.field(), Some("dashas[0].span"));

    let sdk = context(false);
    let stray = teistro::KeyId::new(teistro::catalogue::Kind::DashaSystem, 0x8000);
    let error = sdk
        .chart()
        .reading(
            JulianDay::<Utc>::literal(BIRTH),
            &request().with_dashas([stray]),
        )
        .unwrap_err();
    assert_eq!(error.field(), Some("dashas[0]"));
    assert!(error.hint().unwrap_or_default().contains("VIMSHOTTARI"));
}
