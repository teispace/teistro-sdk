//! A published set read from a signed pack
//! (`03-design/published-overrides.md` §5): the set a valid signature
//! covers, and a refusal by name for a tampered byte, another key, a
//! replayed older version and each malformed part. The key is made here
//! from a fixed seed; the SDK embeds none.

#![cfg(feature = "signed-packs")]
#![allow(
    clippy::unwrap_used,
    missing_docs,
    reason = "a test fails by panicking"
)]

use std::fmt::Write as _;

use ed25519_dalek::{Signer as _, SigningKey};
use teistro::festival::PublishedSet;

const SET: &str = r#"{"format":"teistro.published/1","publisher":"a patro printed in Nepal","id":"np-2083","version":3,"entries":[{"action":"SET","rule":"GHATASTHAPANA","day":{"calendar":"GREGORIAN","year":2026,"month":10,"day":11},"source":"p. 41"}]}"#;

/// A publisher's key, made from a seed for the test alone.
fn publisher(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn signed(pack: &[u8], by: &SigningKey) -> [u8; 64] {
    by.sign(pack).to_bytes()
}

fn refused(pack: &[u8], signature: &[u8], key: &[u8], at_least: u32) -> (Option<String>, String) {
    let error = PublishedSet::from_signed(pack, signature, key, at_least).unwrap_err();
    (error.field().map(str::to_owned), error.message)
}

#[test]
fn a_valid_signature_answers_the_set_it_covers() {
    let key = publisher(7);
    let signature = signed(SET.as_bytes(), &key);
    let set = PublishedSet::from_signed(
        SET.as_bytes(),
        &signature,
        key.verifying_key().as_bytes(),
        3,
    )
    .unwrap();
    assert_eq!(set, PublishedSet::from_json(SET).unwrap());
    assert_eq!((set.id.as_str(), set.version), ("np-2083", 3));
}

/// The almanac's own door reads the same set, and refuses as it refuses.
#[test]
fn the_almanac_reads_a_signed_pack_as_the_set_does() {
    let sdk = teistro::Context::builder().build().unwrap();
    let key = publisher(7);
    let signature = signed(SET.as_bytes(), &key);
    let public = key.verifying_key().to_bytes();
    let read = sdk
        .almanac()
        .published_from_signed(SET.as_bytes(), &signature, &public, 3)
        .unwrap();
    assert_eq!(read, PublishedSet::from_json(SET).unwrap());
    let replayed = sdk
        .almanac()
        .published_from_signed(SET.as_bytes(), &signature, &public, 4)
        .unwrap_err();
    assert_eq!(replayed.field(), Some("version"));
}

#[test]
fn a_tampered_byte_or_another_key_is_refused_on_the_signature() {
    let key = publisher(7);
    let signature = signed(SET.as_bytes(), &key);
    let public = key.verifying_key().to_bytes();
    // One byte changed: the 11th of October becomes the 12th.
    let tampered = SET.replace(r#""day":11"#, r#""day":12"#);
    let (field, message) = refused(tampered.as_bytes(), &signature, &public, 0);
    assert_eq!(field.as_deref(), Some("signature"));
    assert!(message.contains("altered"), "{message}");
    // The right bytes under another publisher's key.
    let other = publisher(8).verifying_key().to_bytes();
    let (field, _) = refused(SET.as_bytes(), &signature, &other, 0);
    assert_eq!(field.as_deref(), Some("signature"));
    // A signature with a bit flipped.
    let mut flipped = signature;
    flipped[0] ^= 1;
    let (field, _) = refused(SET.as_bytes(), &flipped, &public, 0);
    assert_eq!(field.as_deref(), Some("signature"));
}

#[test]
fn a_replayed_older_version_is_refused() {
    let key = publisher(7);
    let signature = signed(SET.as_bytes(), &key);
    let public = key.verifying_key().to_bytes();
    let (field, message) = refused(SET.as_bytes(), &signature, &public, 4);
    assert_eq!(field.as_deref(), Some("version"));
    assert!(message.contains("older than 4"), "{message}");
}

#[test]
fn each_malformed_part_is_refused_by_name() {
    let key = publisher(7);
    let signature = signed(SET.as_bytes(), &key);
    let public = key.verifying_key().to_bytes();
    assert_eq!(
        refused(SET.as_bytes(), &signature, &public[..31], 0)
            .0
            .as_deref(),
        Some("key")
    );
    // 32 bytes that decompress to no curve point: about half of all
    // y-coordinates have none, so the first such small one is found.
    let off_curve = (0_u8..=255)
        .map(|y| {
            let mut bytes = [0_u8; 32];
            bytes[0] = y;
            bytes
        })
        .find(|bytes| ed25519_dalek::VerifyingKey::from_bytes(bytes).is_err())
        .unwrap();
    assert_eq!(
        refused(SET.as_bytes(), &signature, &off_curve, 0)
            .0
            .as_deref(),
        Some("key")
    );
    assert_eq!(
        refused(SET.as_bytes(), &signature[..63], &public, 0)
            .0
            .as_deref(),
        Some("signature")
    );
    // Signed bytes that are not UTF-8, and signed UTF-8 that is no set.
    let latin1 = b"\xff{}";
    assert_eq!(
        refused(latin1, &signed(latin1, &key), &public, 0)
            .0
            .as_deref(),
        Some("pack")
    );
    let stray = SET.replace("teistro.published/1", "teistro.published/9");
    assert_eq!(
        refused(
            stray.as_bytes(),
            &signed(stray.as_bytes(), &key),
            &public,
            0
        )
        .0
        .as_deref(),
        Some("format")
    );
}

/// The vector every binding's test verifies: [`SET`]'s bytes signed by the
/// key of seed 7, so a binding needs no signer of its own.
#[test]
fn the_bindings_vector_is_the_seed_s_signature() {
    let key = publisher(7);
    let hex = |bytes: &[u8]| {
        bytes.iter().fold(String::new(), |mut text, b| {
            let _ = write!(text, "{b:02x}");
            text
        })
    };
    assert_eq!(
        hex(key.verifying_key().as_bytes()),
        "ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c"
    );
    assert_eq!(
        hex(&signed(SET.as_bytes(), &key)),
        "cae87f1a99d4869d5ba41086cd6ceba8cdb035fd11225f53e09882e98423e21332fb40c9ad564380fb9abcdf4c21cb2726f80fb692511b9593f0f9e60fbe320e"
    );
}
