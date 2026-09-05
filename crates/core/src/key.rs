//! Keys and ids. A key is `<kind>.<NAME>`; an id is a dense `u16` inside
//! its kind; the two travel together across the C boundary as a packed
//! [`KeyId`]. Ids at or above [`KeyId::REGISTERED_BASE`] belong to a
//! context's registries and are never serialised without their key.
//!
//! ```
//! use teistro_core::key::{resolve, KeyId};
//! use teistro_core::catalogue::{Graha, Kind};
//!
//! let id = resolve("graha.MARS").expect("a catalogued key");
//! assert_eq!(id, Graha::Mars.key_id());
//! assert_eq!(id.kind(), Some(Kind::Graha));
//! assert_eq!(id.to_string(), "graha.MARS");
//!
//! let wrong = resolve("graha.MARZ").unwrap_err();
//! assert_eq!(wrong.to_string(), "unknown graha key `MARZ`; did you mean `MARS`?");
//! ```

use core::fmt;

pub use crate::catalogue::Kind;
use crate::catalogue::{UnknownKey, key_of, resolve as resolve_in};

/// The longest key name the grammar allows.
pub const MAX_KEY_LEN: usize = 48;

/// A kind number and an id packed as `(kind << 16) | id`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyId(u32);

impl KeyId {
    /// The id meaning "none" inside a kind.
    pub const NONE_ID: u16 = 0xFFFF;
    /// The first id a context's registry hands out.
    pub const REGISTERED_BASE: u16 = 0x8000;

    /// A packed id.
    #[must_use]
    pub const fn new(kind: Kind, id: u16) -> KeyId {
        KeyId(((kind as u32) << 16) | id as u32)
    }

    /// The "none" id of a kind.
    #[must_use]
    pub const fn none(kind: Kind) -> KeyId {
        KeyId::new(kind, KeyId::NONE_ID)
    }

    /// The raw packed value, for the C boundary.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// From the raw packed value; the kind is checked on use, not here.
    #[must_use]
    pub const fn from_bits(bits: u32) -> KeyId {
        KeyId(bits)
    }

    /// The kind, when the number is one this build knows.
    #[must_use]
    pub const fn kind(self) -> Option<Kind> {
        Kind::from_number(self.kind_number())
    }

    /// The kind's number.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the high half is the kind byte"
    )]
    pub const fn kind_number(self) -> u8 {
        (self.0 >> 16) as u8
    }

    /// The id inside the kind.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, reason = "the low half is the id")]
    pub const fn id(self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }

    /// Whether this is the "none" id.
    #[must_use]
    pub const fn is_none(self) -> bool {
        self.id() == KeyId::NONE_ID
    }

    /// Whether the id was handed out by a registry rather than the catalogue.
    #[must_use]
    pub const fn is_registered(self) -> bool {
        self.id() >= KeyId::REGISTERED_BASE && !self.is_none()
    }

    /// The catalogued key, when the id is one.
    #[must_use]
    pub fn key(self) -> Option<&'static str> {
        key_of(self)
    }
}

impl fmt::Display for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.kind(), self.key()) {
            (Some(kind), Some(key)) => write!(f, "{}.{key}", kind.name()),
            (Some(kind), None) if self.is_none() => write!(f, "{}.NONE", kind.name()),
            (Some(kind), None) => write!(f, "{}#{}", kind.name(), self.id()),
            (None, _) => write!(f, "{}#{}", self.kind_number(), self.id()),
        }
    }
}

impl fmt::Debug for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyId({self})")
    }
}

impl serde::Serialize for KeyId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for KeyId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<KeyId, D::Error> {
        let text = String::deserialize(deserializer)?;
        parse_key_id(&text).map_err(serde::de::Error::custom)
    }
}

/// Whether `s` is a key name: `[A-Z][A-Z0-9_]{0,47}`.
#[must_use]
pub fn is_key_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= MAX_KEY_LEN
        && s.bytes().next().is_some_and(|b| b.is_ascii_uppercase())
        && s.bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
}

/// Splits `graha.SUN` into its kind name and key name.
#[must_use]
pub fn split_key(full: &str) -> Option<(&str, &str)> {
    full.split_once('.')
}

/// Resolves a full key (`graha.SUN`) to its packed id.
///
/// # Errors
///
/// An unknown kind or key, with a suggestion when one is close.
pub fn resolve(full: &str) -> Result<KeyId, UnknownKey> {
    let Some((kind_name, key)) = split_key(full) else {
        return Err(UnknownKey {
            kind: None,
            key: full.to_string(),
            suggestion: None,
        });
    };
    let kind = Kind::from_name(kind_name).ok_or_else(|| UnknownKey::kind_name(kind_name))?;
    resolve_in(kind, key).ok_or_else(|| UnknownKey {
        kind: Some(kind),
        key: key.to_string(),
        suggestion: suggest(kind, key),
    })
}

/// The nearest catalogued key inside a kind, within two edits.
#[must_use]
pub fn suggest(kind: Kind, key: &str) -> Option<&'static str> {
    let upper = key.to_ascii_uppercase();
    (0..u16::try_from(kind.count()).unwrap_or(u16::MAX))
        .filter_map(|id| key_of(KeyId::new(kind, id)))
        .map(|k| (crate::catalogue::distance(&upper, k), k))
        .filter(|(d, _)| *d <= 2)
        .min_by_key(|(d, _)| *d)
        .map(|(_, k)| k)
}

/// Parses either a full key or the `kind#id` form a registered id prints as.
///
/// # Errors
///
/// Neither form.
pub fn parse_key_id(text: &str) -> Result<KeyId, UnknownKey> {
    if let Some((kind_name, id)) = text.split_once('#') {
        let kind = Kind::from_name(kind_name).ok_or_else(|| UnknownKey::kind_name(kind_name))?;
        let id: u16 = id.parse().map_err(|_| UnknownKey {
            kind: Some(kind),
            key: text.to_string(),
            suggestion: None,
        })?;
        return Ok(KeyId::new(kind, id));
    }
    resolve(text)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;
    use crate::catalogue::{Graha, Rashi};

    #[test]
    fn ids_pack_and_unpack() {
        let id = KeyId::new(Kind::Rashi, 7);
        assert_eq!(id.kind(), Some(Kind::Rashi));
        assert_eq!(id.id(), 7);
        assert_eq!(KeyId::from_bits(id.bits()), id);
        assert!(!id.is_registered());
        assert!(KeyId::new(Kind::Varga, 0x8001).is_registered());
        assert!(KeyId::none(Kind::Graha).is_none());
        assert_eq!(KeyId::none(Kind::Graha).to_string(), "graha.NONE");
        assert_eq!(KeyId::new(Kind::Varga, 0x8001).to_string(), "varga#32769");
        assert_eq!(KeyId::from_bits(0x00FF_0001).to_string(), "255#1");
    }

    #[test]
    fn keys_resolve_with_suggestions() {
        assert_eq!(resolve("rashi.LEO"), Ok(Rashi::Leo.key_id()));
        assert_eq!(
            resolve("graha.SUN").map(|k| k.to_string()),
            Ok("graha.SUN".to_string())
        );
        let wrong = resolve("grahas.SUN").unwrap_err();
        assert_eq!(wrong.suggestion, Some("graha"));
        assert_eq!(resolve("SUN").unwrap_err().kind, None);
        assert_eq!(resolve("graha.sun").unwrap_err().suggestion, Some("SUN"));
        assert_eq!(
            parse_key_id("varga#32769"),
            Ok(KeyId::new(Kind::Varga, 32769))
        );
        let json = serde_json::to_string(&Graha::Mars.key_id()).unwrap_or_default();
        assert_eq!(json, "\"graha.MARS\"");
        let back: KeyId = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(back, Graha::Mars.key_id());
    }

    #[test]
    fn the_key_grammar() {
        assert!(is_key_name("SUN"));
        assert!(is_key_name("PURVA_PHALGUNI"));
        assert!(is_key_name("D9"));
        assert!(!is_key_name("sun"));
        assert!(!is_key_name("9D"));
        assert!(!is_key_name(""));
        assert!(!is_key_name(&"A".repeat(49)));
    }
}
