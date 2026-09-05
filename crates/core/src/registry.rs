//! A registry: what a context registers for an open kind (custom vargas,
//! consumer dasha systems, consumer points, pack rules) beside the
//! catalogue, validated at registration, ids from `0x8000`, sealed before
//! the first computation so nothing changes under a running context.
//!
//! ```
//! use teistro_core::registry::{Definition, Registry};
//! use teistro_core::catalogue::Kind;
//! use teistro_core::error::Error;
//!
//! #[derive(Clone)]
//! struct CustomVarga { key: String, divisions: u16 }
//! impl Definition for CustomVarga {
//!     fn key(&self) -> &str { &self.key }
//!     fn validate(&self) -> Result<(), Error> {
//!         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions")) }
//!     }
//! }
//!
//! let mut registry = Registry::new(Kind::Varga);
//! let id = registry.register(CustomVarga { key: "ACME_D7".into(), divisions: 7 }).expect("valid");
//! assert!(id.is_registered());
//! registry.seal();
//! assert!(registry.register(CustomVarga { key: "ACME_D8".into(), divisions: 8 }).is_err());
//! assert_eq!(registry.get("ACME_D7").map(|(d, _)| d.divisions), Some(7));
//! ```

use std::collections::BTreeMap;

use crate::catalogue::Kind;
use crate::error::{Detail, Error};
use crate::key::{KeyId, is_key_name};

/// What a registered definition provides.
pub trait Definition: Clone {
    /// The key, matching the key grammar and unique in the registry.
    fn key(&self) -> &str;

    /// The kind's whole-table invariants for one definition (ADR-0017).
    ///
    /// # Errors
    ///
    /// Why the definition is refused.
    fn validate(&self) -> Result<(), Error>;
}

/// The registry of one open kind.
#[derive(Clone, Debug)]
pub struct Registry<D: Definition> {
    kind: Kind,
    entries: Vec<D>,
    by_key: BTreeMap<String, u16>,
    sealed: bool,
}

impl<D: Definition> Registry<D> {
    /// An empty, unsealed registry for a kind.
    #[must_use]
    pub const fn new(kind: Kind) -> Registry<D> {
        Registry {
            kind,
            entries: Vec::new(),
            by_key: BTreeMap::new(),
            sealed: false,
        }
    }

    /// The kind.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// Registers a definition and returns its id.
    ///
    /// # Errors
    ///
    /// A sealed registry, a key outside the grammar or already taken, a
    /// definition its own validation refuses, or a full registry.
    pub fn register(&mut self, definition: D) -> Result<KeyId, Error> {
        if self.sealed {
            return Err(Error::invalid_arg(format!(
                "the {} registry is sealed; register before the context is created",
                self.kind
            ))
            .with_detail(Detail::Sealed));
        }
        let key = definition.key().to_string();
        if !is_key_name(&key) {
            return Err(Error::invalid_arg(format!(
                "`{key}` is not a key name: [A-Z][A-Z0-9_], at most 48 characters"
            ))
            .with_field("key"));
        }
        if crate::catalogue::resolve(self.kind, &key).is_some() || self.by_key.contains_key(&key) {
            return Err(
                Error::invalid_arg(format!("`{}.{key}` is already defined", self.kind))
                    .with_field("key"),
            );
        }
        definition.validate()?;
        let offset = u16::try_from(self.entries.len())
            .ok()
            .filter(|n| *n < KeyId::NONE_ID - KeyId::REGISTERED_BASE)
            .ok_or_else(|| Error::limit(format!("the {} registry is full", self.kind)))?;
        let id = KeyId::REGISTERED_BASE + offset;
        self.entries.push(definition);
        self.by_key.insert(key, id);
        Ok(KeyId::new(self.kind, id))
    }

    /// Refuses further registrations.
    pub fn seal(&mut self) {
        self.sealed = true;
    }

    /// Whether the registry is sealed.
    #[must_use]
    pub const fn is_sealed(&self) -> bool {
        self.sealed
    }

    /// The definition with a key, and its id.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<(&D, KeyId)> {
        let id = *self.by_key.get(key)?;
        self.by_id(KeyId::new(self.kind, id))
            .map(|d| (d, KeyId::new(self.kind, id)))
    }

    /// The definition with an id.
    #[must_use]
    pub fn by_id(&self, id: KeyId) -> Option<&D> {
        if id.kind() != Some(self.kind) || !id.is_registered() {
            return None;
        }
        self.entries
            .get(usize::from(id.id() - KeyId::REGISTERED_BASE))
    }

    /// The number of definitions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Every definition with its id, in registration order.
    pub fn iter(&self) -> impl Iterator<Item = (KeyId, &D)> {
        self.entries.iter().enumerate().map(|(i, d)| {
            #[allow(clippy::cast_possible_truncation, reason = "bounded at registration")]
            let id = KeyId::REGISTERED_BASE + i as u16;
            (KeyId::new(self.kind, id), d)
        })
    }
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

    #[derive(Clone, Debug, PartialEq)]
    struct Def(&'static str, bool);

    impl Definition for Def {
        fn key(&self) -> &str {
            self.0
        }

        fn validate(&self) -> Result<(), Error> {
            if self.1 {
                Ok(())
            } else {
                Err(Error::invalid_arg("refused"))
            }
        }
    }

    #[test]
    fn registration_rules() {
        let mut registry: Registry<Def> = Registry::new(Kind::Varga);
        let id = registry
            .register(Def("ACME_D7", true))
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(id.id(), KeyId::REGISTERED_BASE);
        assert!(registry.register(Def("acme", true)).is_err());
        assert!(
            registry
                .register(Def("D9", true))
                .unwrap_err()
                .message
                .contains("already defined")
        );
        assert!(registry.register(Def("ACME_D7", true)).is_err());
        assert!(registry.register(Def("ACME_BAD", false)).is_err());
        assert_eq!(registry.by_id(id).map(|d| d.0), Some("ACME_D7"));
        assert_eq!(
            registry.by_id(KeyId::new(Kind::Graha, KeyId::REGISTERED_BASE)),
            None
        );
        assert_eq!(registry.iter().count(), 1);
        registry.seal();
        let sealed = registry.register(Def("ACME_D8", true)).unwrap_err();
        assert_eq!(sealed.detail, Some(Detail::Sealed));
        assert!(registry.is_sealed());
        assert_eq!(registry.len(), 1);
    }
}
