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
///
/// ```
/// use teistro_core::catalogue::Kind;
/// use teistro_core::error::Error;
/// use teistro_core::registry::{Definition, Registry};
///
/// /// A consumer's point: a key and a fixed longitude.
/// #[derive(Clone)]
/// struct FixedPoint { key: &'static str, longitude_deg: f64 }
///
/// impl Definition for FixedPoint {
///     fn key(&self) -> &str { self.key }
///     fn validate(&self) -> Result<(), Error> {
///         if (0.0..360.0).contains(&self.longitude_deg) { Ok(()) } else { Err(Error::invalid_arg("a longitude is 0 to 360").with_field("longitude_deg")) }
///     }
/// }
///
/// let mut points = Registry::new(Kind::Point);
/// assert!(points.register(FixedPoint { key: "GALACTIC_CENTRE", longitude_deg: 266.86 }).is_ok());
/// assert!(points.register(FixedPoint { key: "NOWHERE", longitude_deg: 400.0 }).is_err());
/// ```
pub trait Definition: Clone {
    /// The key, matching the key grammar and unique in the registry.
    ///
    /// ```
    /// use teistro_core::catalogue::Kind;
    /// use teistro_core::registry::Registry;
    /// # use teistro_core::error::Error;
    /// # use teistro_core::registry::Definition;
    /// # #[derive(Clone)]
    /// # struct CustomVarga { key: String, divisions: u16 }
    /// # impl Definition for CustomVarga {
    /// #     fn key(&self) -> &str { &self.key }
    /// #     fn validate(&self) -> Result<(), Error> {
    /// #         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions are 1 to 300")) }
    /// #     }
    /// # }
    /// # let varga = |key: &str, divisions: u16| CustomVarga { key: key.into(), divisions };
    ///
    /// let mut registry: Registry<CustomVarga> = Registry::new(Kind::Varga);
    /// registry.register(varga("ACME_D7", 7))?;
    /// assert!(registry.get("ACME_D7").is_some());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    fn key(&self) -> &str;

    /// The kind's whole-table invariants for one definition (ADR-0017).
    ///
    /// ```
    /// use teistro_core::catalogue::Kind;
    /// use teistro_core::registry::Registry;
    /// # use teistro_core::error::Error;
    /// # use teistro_core::registry::Definition;
    /// # #[derive(Clone)]
    /// # struct CustomVarga { key: String, divisions: u16 }
    /// # impl Definition for CustomVarga {
    /// #     fn key(&self) -> &str { &self.key }
    /// #     fn validate(&self) -> Result<(), Error> {
    /// #         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions are 1 to 300")) }
    /// #     }
    /// # }
    /// # let varga = |key: &str, divisions: u16| CustomVarga { key: key.into(), divisions };
    ///
    /// let mut registry: Registry<CustomVarga> = Registry::new(Kind::Varga);
    /// // The definition's own check runs at registration.
    /// assert_eq!(registry.register(varga("ACME_D0", 0)).unwrap_err().message, "divisions are 1 to 300");
    /// ```
    ///
    /// # Errors
    ///
    /// Why the definition is refused.
    fn validate(&self) -> Result<(), Error>;
}

/// The registry of one open kind.
///
/// ```
/// use teistro_core::catalogue::Kind;
/// use teistro_core::registry::Registry;
/// # use teistro_core::error::Error;
/// # use teistro_core::registry::Definition;
/// # #[derive(Clone)]
/// # struct CustomVarga { key: String, divisions: u16 }
/// # impl Definition for CustomVarga {
/// #     fn key(&self) -> &str { &self.key }
/// #     fn validate(&self) -> Result<(), Error> {
/// #         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions are 1 to 300")) }
/// #     }
/// # }
/// # let varga = |key: &str, divisions: u16| CustomVarga { key: key.into(), divisions };
///
/// let mut registry: Registry<CustomVarga> = Registry::new(Kind::Varga);
/// let id = registry.register(varga("ACME_D7", 7))?;
/// assert_eq!(id.to_string(), "varga#32768");
/// registry.seal();
/// assert!(registry.is_sealed());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct Registry<D: Definition> {
    kind: Kind,
    entries: Vec<D>,
    by_key: BTreeMap<String, u16>,
    sealed: bool,
}

impl<D: Definition> Registry<D> {
    /// An empty, unsealed registry for a kind.
    ///
    /// ```
    /// use teistro_core::catalogue::Kind;
    /// use teistro_core::registry::Registry;
    /// # use teistro_core::error::Error;
    /// # use teistro_core::registry::Definition;
    /// # #[derive(Clone)]
    /// # struct CustomVarga { key: String, divisions: u16 }
    /// # impl Definition for CustomVarga {
    /// #     fn key(&self) -> &str { &self.key }
    /// #     fn validate(&self) -> Result<(), Error> {
    /// #         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions are 1 to 300")) }
    /// #     }
    /// # }
    /// # let varga = |key: &str, divisions: u16| CustomVarga { key: key.into(), divisions };
    ///
    /// let mut registry: Registry<CustomVarga> = Registry::new(Kind::Varga);
    /// assert!(registry.is_empty() && !registry.is_sealed());
    /// ```
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
    ///
    /// ```
    /// use teistro_core::catalogue::Kind;
    /// use teistro_core::registry::Registry;
    /// # use teistro_core::error::Error;
    /// # use teistro_core::registry::Definition;
    /// # #[derive(Clone)]
    /// # struct CustomVarga { key: String, divisions: u16 }
    /// # impl Definition for CustomVarga {
    /// #     fn key(&self) -> &str { &self.key }
    /// #     fn validate(&self) -> Result<(), Error> {
    /// #         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions are 1 to 300")) }
    /// #     }
    /// # }
    /// # let varga = |key: &str, divisions: u16| CustomVarga { key: key.into(), divisions };
    ///
    /// let mut registry: Registry<CustomVarga> = Registry::new(Kind::Varga);
    /// assert_eq!(registry.kind(), Kind::Varga);
    /// ```
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// Registers a definition and returns its id.
    ///
    /// ```
    /// use teistro_core::error::Detail;
    /// use teistro_core::catalogue::Kind;
    /// use teistro_core::registry::Registry;
    /// # use teistro_core::error::Error;
    /// # use teistro_core::registry::Definition;
    /// # #[derive(Clone)]
    /// # struct CustomVarga { key: String, divisions: u16 }
    /// # impl Definition for CustomVarga {
    /// #     fn key(&self) -> &str { &self.key }
    /// #     fn validate(&self) -> Result<(), Error> {
    /// #         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions are 1 to 300")) }
    /// #     }
    /// # }
    /// # let varga = |key: &str, divisions: u16| CustomVarga { key: key.into(), divisions };
    ///
    /// let mut registry: Registry<CustomVarga> = Registry::new(Kind::Varga);
    /// let first = registry.register(varga("ACME_D7", 7))?;
    /// let second = registry.register(varga("ACME_D8", 8))?;
    /// assert_eq!(second.id(), first.id() + 1);
    /// // A catalogued key, a taken one and a malformed one are each refused.
    /// assert!(registry.register(varga("D9", 9)).is_err());
    /// assert!(registry.register(varga("ACME_D7", 7)).is_err());
    /// assert_eq!(registry.register(varga("acme", 7)).unwrap_err().field(), Some("key"));
    /// registry.seal();
    /// assert_eq!(registry.register(varga("ACME_D5", 5)).unwrap_err().detail, Some(Detail::Sealed));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
    ///
    /// ```
    /// use teistro_core::catalogue::Kind;
    /// use teistro_core::registry::Registry;
    /// # use teistro_core::error::Error;
    /// # use teistro_core::registry::Definition;
    /// # #[derive(Clone)]
    /// # struct CustomVarga { key: String, divisions: u16 }
    /// # impl Definition for CustomVarga {
    /// #     fn key(&self) -> &str { &self.key }
    /// #     fn validate(&self) -> Result<(), Error> {
    /// #         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions are 1 to 300")) }
    /// #     }
    /// # }
    /// # let varga = |key: &str, divisions: u16| CustomVarga { key: key.into(), divisions };
    ///
    /// let mut registry: Registry<CustomVarga> = Registry::new(Kind::Varga);
    /// registry.seal();
    /// assert!(registry.register(varga("ACME_D7", 7)).is_err());
    /// ```
    pub fn seal(&mut self) {
        self.sealed = true;
    }

    /// Whether the registry is sealed.
    ///
    /// ```
    /// use teistro_core::catalogue::Kind;
    /// use teistro_core::registry::Registry;
    /// # use teistro_core::error::Error;
    /// # use teistro_core::registry::Definition;
    /// # #[derive(Clone)]
    /// # struct CustomVarga { key: String, divisions: u16 }
    /// # impl Definition for CustomVarga {
    /// #     fn key(&self) -> &str { &self.key }
    /// #     fn validate(&self) -> Result<(), Error> {
    /// #         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions are 1 to 300")) }
    /// #     }
    /// # }
    /// # let varga = |key: &str, divisions: u16| CustomVarga { key: key.into(), divisions };
    ///
    /// let mut registry: Registry<CustomVarga> = Registry::new(Kind::Varga);
    /// assert!(!registry.is_sealed());
    /// registry.seal();
    /// assert!(registry.is_sealed());
    /// ```
    #[must_use]
    pub const fn is_sealed(&self) -> bool {
        self.sealed
    }

    /// The definition with a key, and its id.
    ///
    /// ```
    /// use teistro_core::catalogue::Kind;
    /// use teistro_core::registry::Registry;
    /// # use teistro_core::error::Error;
    /// # use teistro_core::registry::Definition;
    /// # #[derive(Clone)]
    /// # struct CustomVarga { key: String, divisions: u16 }
    /// # impl Definition for CustomVarga {
    /// #     fn key(&self) -> &str { &self.key }
    /// #     fn validate(&self) -> Result<(), Error> {
    /// #         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions are 1 to 300")) }
    /// #     }
    /// # }
    /// # let varga = |key: &str, divisions: u16| CustomVarga { key: key.into(), divisions };
    ///
    /// let mut registry: Registry<CustomVarga> = Registry::new(Kind::Varga);
    /// let id = registry.register(varga("ACME_D7", 7))?;
    /// let (definition, found) = registry.get("ACME_D7").unwrap();
    /// assert_eq!((definition.divisions, found), (7, id));
    /// assert!(registry.get("ACME_D8").is_none());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn get(&self, key: &str) -> Option<(&D, KeyId)> {
        let id = *self.by_key.get(key)?;
        self.by_id(KeyId::new(self.kind, id))
            .map(|d| (d, KeyId::new(self.kind, id)))
    }

    /// The definition with an id.
    ///
    /// ```
    /// use teistro_core::catalogue::Varga;
    /// use teistro_core::catalogue::Kind;
    /// use teistro_core::registry::Registry;
    /// # use teistro_core::error::Error;
    /// # use teistro_core::registry::Definition;
    /// # #[derive(Clone)]
    /// # struct CustomVarga { key: String, divisions: u16 }
    /// # impl Definition for CustomVarga {
    /// #     fn key(&self) -> &str { &self.key }
    /// #     fn validate(&self) -> Result<(), Error> {
    /// #         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions are 1 to 300")) }
    /// #     }
    /// # }
    /// # let varga = |key: &str, divisions: u16| CustomVarga { key: key.into(), divisions };
    ///
    /// let mut registry: Registry<CustomVarga> = Registry::new(Kind::Varga);
    /// let id = registry.register(varga("ACME_D7", 7))?;
    /// assert_eq!(registry.by_id(id).map(|d| d.divisions), Some(7));
    /// // A catalogued id is the catalogue's, not the registry's.
    /// assert!(registry.by_id(Varga::D9.key_id()).is_none());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn by_id(&self, id: KeyId) -> Option<&D> {
        if id.kind() != Some(self.kind) || !id.is_registered() {
            return None;
        }
        self.entries
            .get(usize::from(id.id() - KeyId::REGISTERED_BASE))
    }

    /// The number of definitions.
    ///
    /// ```
    /// use teistro_core::catalogue::Kind;
    /// use teistro_core::registry::Registry;
    /// # use teistro_core::error::Error;
    /// # use teistro_core::registry::Definition;
    /// # #[derive(Clone)]
    /// # struct CustomVarga { key: String, divisions: u16 }
    /// # impl Definition for CustomVarga {
    /// #     fn key(&self) -> &str { &self.key }
    /// #     fn validate(&self) -> Result<(), Error> {
    /// #         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions are 1 to 300")) }
    /// #     }
    /// # }
    /// # let varga = |key: &str, divisions: u16| CustomVarga { key: key.into(), divisions };
    ///
    /// let mut registry: Registry<CustomVarga> = Registry::new(Kind::Varga);
    /// registry.register(varga("ACME_D7", 7))?;
    /// assert_eq!(registry.len(), 1);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is registered.
    ///
    /// ```
    /// use teistro_core::catalogue::Kind;
    /// use teistro_core::registry::Registry;
    /// # use teistro_core::error::Error;
    /// # use teistro_core::registry::Definition;
    /// # #[derive(Clone)]
    /// # struct CustomVarga { key: String, divisions: u16 }
    /// # impl Definition for CustomVarga {
    /// #     fn key(&self) -> &str { &self.key }
    /// #     fn validate(&self) -> Result<(), Error> {
    /// #         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions are 1 to 300")) }
    /// #     }
    /// # }
    /// # let varga = |key: &str, divisions: u16| CustomVarga { key: key.into(), divisions };
    ///
    /// let mut registry: Registry<CustomVarga> = Registry::new(Kind::Varga);
    /// assert!(registry.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Every definition with its id, in registration order.
    ///
    /// ```
    /// use teistro_core::catalogue::Kind;
    /// use teistro_core::registry::Registry;
    /// # use teistro_core::error::Error;
    /// # use teistro_core::registry::Definition;
    /// # #[derive(Clone)]
    /// # struct CustomVarga { key: String, divisions: u16 }
    /// # impl Definition for CustomVarga {
    /// #     fn key(&self) -> &str { &self.key }
    /// #     fn validate(&self) -> Result<(), Error> {
    /// #         if (1..=300).contains(&self.divisions) { Ok(()) } else { Err(Error::invalid_arg("divisions are 1 to 300")) }
    /// #     }
    /// # }
    /// # let varga = |key: &str, divisions: u16| CustomVarga { key: key.into(), divisions };
    ///
    /// let mut registry: Registry<CustomVarga> = Registry::new(Kind::Varga);
    /// registry.register(varga("ACME_D7", 7))?;
    /// registry.register(varga("ACME_D8", 8))?;
    /// let keys: Vec<String> = registry.iter().map(|(id, d)| format!("{id}={}", d.divisions)).collect();
    /// assert_eq!(keys, ["varga#32768=7", "varga#32769=8"]);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (KeyId, &D)> {
        self.entries.iter().enumerate().map(|(i, d)| {
            #[allow(clippy::cast_possible_truncation, reason = "bounded at registration")]
            let id = KeyId::REGISTERED_BASE + i as u16;
            (KeyId::new(self.kind, id), d)
        })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
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
