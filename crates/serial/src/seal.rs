//! A value and its provenance, sealed: the one field a caller cannot
//! fill by hand is filled by the constructor.
//!
//! `Provenance` carries every field ADR-0020 asks for, and the field the
//! whole envelope exists for — the hash of the value — is set to the
//! hash of nothing by `Provenance::new` and replaced by exactly one
//! producer of three (`03-design/serial-measured.md` §2). A founded
//! chart and a daily panchanga both go out claiming a hash of the empty
//! string.
//!
//! That is a shape problem rather than a bug in a producer: a value and
//! its stamp are built separately and joined at the end, so the one
//! field that **cannot** be filled until the value exists is the one
//! everybody forgets.
//!
//! So [`Sealed::new`] is the only way to make one, it computes the hash
//! from the value it is given, and there is no setter. A stale hash is
//! not representable.
//!
//! ```
//! use teistro_core::envelope::{Hash, Provenance, Version};
//! use teistro_serial::seal::Sealed;
//!
//! let provenance = Provenance::new(
//!     Version::new(0, 1, 0), 1, 1, "parashari-classical",
//!     Hash::of(b"settings"), Hash::of(b"input"),
//! );
//! // The provenance goes in with an empty hash and comes out with the
//! // value's own.
//! assert_eq!(provenance.content_hash, Hash::of(&[]));
//! let sealed = Sealed::new(vec![1.5_f64, 2.5], provenance);
//! assert_ne!(sealed.content_hash(), Hash::of(&[]));
//! assert_eq!(sealed.content_hash(), sealed.provenance().content_hash);
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::envelope::{Envelope, Hash, Provenance};

use crate::canonical;

/// A value with a provenance whose content hash is the value's own.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sealed<T> {
    value: T,
    provenance: Provenance,
}

impl<T: Serialize> Sealed<T> {
    /// Seals a value with its provenance, computing the content hash.
    ///
    /// This is the only constructor, and it takes the provenance by
    /// value, so the pair cannot be built with a hash that belongs to
    /// something else.
    #[must_use]
    pub fn new(value: T, mut provenance: Provenance) -> Sealed<T> {
        provenance.content_hash = canonical::hash_of(&value);
        Sealed { value, provenance }
    }

    /// Seals what a producer built, which is how a module's own
    /// `Envelope` becomes something publishable.
    #[must_use]
    pub fn of(envelope: Envelope<T>) -> Sealed<T> {
        Sealed::new(envelope.value, envelope.provenance)
    }

    /// The value.
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// How it was produced, with the hash filled in.
    pub const fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// The hash of the value's canonical bytes.
    #[must_use]
    pub const fn content_hash(&self) -> Hash {
        self.provenance.content_hash
    }

    /// The cache key every binding documents.
    #[must_use]
    pub fn cache_key(&self) -> (Hash, Hash, u32) {
        self.provenance.cache_key()
    }

    /// The canonical bytes the hash was taken over.
    #[must_use]
    pub fn to_hash_form(&self) -> String {
        canonical::to_hash_form(&self.value)
    }

    /// Whether the hash still belongs to the value, which is true by
    /// construction and is asserted rather than assumed.
    #[must_use]
    pub fn is_intact(&self) -> bool {
        self.provenance.content_hash == canonical::hash_of(&self.value)
    }

    /// The value and its provenance, given up.
    #[must_use]
    pub fn into_parts(self) -> (T, Provenance) {
        (self.value, self.provenance)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]

    use super::Sealed;
    use crate::canonical;
    use teistro_core::envelope::{Envelope, Hash, Provenance, Version};

    fn provenance() -> Provenance {
        Provenance::new(
            Version::new(0, 1, 0),
            1,
            1,
            "parashari-classical",
            Hash::of(b"settings"),
            Hash::of(b"input"),
        )
    }

    #[test]
    fn sealing_fills_the_hash_the_producer_left_empty() {
        let bare = provenance();
        assert_eq!(bare.content_hash, Hash::of(&[]), "the placeholder");
        let sealed = Sealed::new(vec![1.5_f64, 2.5], bare);
        assert_eq!(sealed.content_hash(), canonical::hash_of(sealed.value()));
        assert_ne!(sealed.content_hash(), Hash::of(&[]));
        assert!(sealed.is_intact());
    }

    #[test]
    fn a_producers_envelope_becomes_publishable() {
        let envelope = Envelope::new(vec![1_u8, 2, 3], provenance());
        assert_eq!(envelope.provenance.content_hash, Hash::of(&[]));
        let sealed = Sealed::of(envelope);
        assert!(sealed.is_intact());
        assert_eq!(sealed.value(), &vec![1_u8, 2, 3]);
    }

    #[test]
    fn the_hash_is_the_values_and_moves_with_it() {
        let one = Sealed::new(vec![1.0_f64], provenance());
        let same = Sealed::new(vec![1.0_f64], provenance());
        let other = Sealed::new(vec![2.0_f64], provenance());
        assert_eq!(one.content_hash(), same.content_hash());
        assert_ne!(one.content_hash(), other.content_hash());
        // Two doubles that differ at all are two answers. The grammar
        // used to round them together below twelve decimals, which made
        // the form lossy on purpose — and not a fixed point, which is
        // the property a content hash actually rests on. "Would these
        // compute the same" is the **settings** hash's question; this
        // one asks whether the bytes are the same bytes.
        let near = Sealed::new(vec![1.000_000_000_000_01_f64], provenance());
        assert_ne!(one.content_hash(), near.content_hash());
    }

    #[test]
    fn the_cache_key_is_the_three_the_bindings_document() {
        let sealed = Sealed::new(1_u8, provenance());
        let (input, settings, calculation) = sealed.cache_key();
        assert_eq!(input, Hash::of(b"input"));
        assert_eq!(settings, Hash::of(b"settings"));
        assert_eq!(calculation, 1);
        // The content hash is not part of it: it is what a caller
        // compares once it has two results.
        assert_ne!(sealed.content_hash(), input);
    }

    #[test]
    fn the_bytes_the_hash_was_taken_over_are_the_ones_it_gives_back() {
        let sealed = Sealed::new(vec![1.5_f64, 2.0], provenance());
        assert_eq!(sealed.to_hash_form(), "[1.5,2]");
        assert_eq!(
            sealed.content_hash(),
            Hash::of(sealed.to_hash_form().as_bytes())
        );
        let (value, provenance) = sealed.into_parts();
        assert_eq!(value, vec![1.5, 2.0]);
        assert_ne!(provenance.content_hash, Hash::of(&[]));
    }
}
