//! A bundle a writer sealed but did not write as this reader reads it: a
//! later format version, an index naming another locale than its packs,
//! bytes past the last pack. Each is refused for what it is, the checksum
//! resealed so that it is not the checksum that refuses it.

#![allow(clippy::unwrap_used, reason = "a test fails by panicking")]
#![allow(
    clippy::indexing_slicing,
    reason = "a test rewrites the bytes it was given"
)]

use teistro_intl::pack::{Bundle, build_bundle};
use teistro_intl::sdk_root;
use teistro_intl::source::Tree;

/// The Nepali bundle and where its checksum is: after the magic, the
/// version, the flags, the locale and the count.
fn nepali() -> (Vec<u8>, usize) {
    let tree = Tree::load(&sdk_root()).unwrap();
    let bytes = build_bundle(&tree.locales["ne-Deva-NP"]).unwrap();
    (bytes, 4 + 2 + 2 + 2 + "ne-Deva-NP".len() + 4)
}

/// The bundle with its checksum written again.
fn resealed(mut bytes: Vec<u8>, at: usize) -> Vec<u8> {
    let mut crc = crc32fast::Hasher::new();
    crc.update(&bytes[..at]);
    crc.update(&bytes[at + 4..]);
    let sum = crc.finalize().to_le_bytes();
    bytes[at..at + 4].copy_from_slice(&sum);
    bytes
}

fn refusal(bytes: &[u8]) -> String {
    Bundle::parse(bytes)
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default()
}

#[test]
fn a_bundle_this_reader_does_not_write_is_refused_for_what_it_is() {
    let (bytes, crc_at) = nepali();
    let bundle = Bundle::parse(&bytes).unwrap();
    assert_eq!(bundle.locale(), "ne-Deva-NP");

    let mut later = bytes.clone();
    later[4..6].copy_from_slice(&99_u16.to_le_bytes());
    assert!(
        refusal(&later).contains("format version 99; this reader knows"),
        "{}",
        refusal(&later)
    );

    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(refusal(&trailing).contains("checksum mismatch"));
    let trailing = resealed(trailing, crc_at);
    assert!(
        refusal(&trailing).contains("trailing bytes after the last pack"),
        "{}",
        refusal(&trailing)
    );

    // The header's locale, changed to another of the same length.
    let mut other = bytes;
    let at = 4 + 2 + 2 + 2;
    other[at..at + 10].copy_from_slice(b"ne-Deva-IN");
    let other = resealed(other, crc_at);
    let why = refusal(&other);
    assert!(
        why.contains("the bundle's index names ne-Deva-IN ")
            && why.contains("the pack says ne-Deva-NP "),
        "{why}"
    );
}
