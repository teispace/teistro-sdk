//! The nakshatra-seeded dasha systems a context can compute beyond the
//! catalogue's: the ones a consumer registers (`03-design/dasha-kernels.md`,
//! "A consumer's own system").
//!
//! A registered system is a [`UduDefinition`], checked by the rules a shipped
//! row passes, with a key the catalogue does not have, so it adds a system and
//! never replaces one.
//!
//! ```
//! use teistro_core::catalogue::{DashaSystem, Graha, Nakshatra};
//! use teistro_dasha::{DashaSystems, Lord, UduDefinition};
//!
//! let mut systems = DashaSystems::new();
//! let lords = [Graha::Sun, Graha::Moon, Graha::Mars]
//!     .map(|graha| Lord { graha, years: 12 })
//!     .to_vec();
//! let definition = UduDefinition {
//!     key: String::from("ACME_TRAYA"),
//!     lords,
//!     ..UduDefinition::of("ACME_TRAYA", Nakshatra::Ashwini)
//! };
//! let id = systems.register(definition)?;
//! assert!(id.is_registered());
//! assert_eq!(systems.id("ACME_TRAYA"), Some(id));
//! assert_eq!(systems.by_id(id).map(|d| d.lords.len()), Some(3));
//!
//! // A catalogued key cannot be taken over.
//! let impostor = UduDefinition::of(DashaSystem::Vimshottari.key(), Nakshatra::Ashwini);
//! assert!(systems.register(impostor).is_err());
//! # use teistro_core::catalogue::Catalogued;
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use teistro_core::catalogue::Kind;
use teistro_core::error::Error;
use teistro_core::key::KeyId;
use teistro_core::registry::Registry;

use crate::row::UduDefinition;

/// The consumer's nakshatra-seeded systems, looked up by key or id.
#[derive(Clone, Debug)]
pub struct DashaSystems {
    registered: Registry<UduDefinition>,
}

impl Default for DashaSystems {
    fn default() -> DashaSystems {
        DashaSystems::new()
    }
}

impl DashaSystems {
    /// Nothing registered yet.
    #[must_use]
    pub const fn new() -> DashaSystems {
        DashaSystems {
            registered: Registry::new(Kind::DashaSystem),
        }
    }

    /// Registers a consumer's system.
    ///
    /// # Errors
    ///
    /// A definition the row's checks refuse, naming the field; a key the
    /// catalogue has or another registered system took; a sealed registry.
    pub fn register(&mut self, definition: UduDefinition) -> Result<KeyId, Error> {
        self.registered.register(definition)
    }

    /// Refuses further registrations, so nothing changes under a context that
    /// has started computing.
    pub fn seal(&mut self) {
        self.registered.seal();
    }

    /// The definition registered under a key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&UduDefinition> {
        self.registered.get(key).map(|(definition, _)| definition)
    }

    /// The id a key was registered under.
    #[must_use]
    pub fn id(&self, key: &str) -> Option<KeyId> {
        self.registered.get(key).map(|(_, id)| id)
    }

    /// The definition registered under an id.
    #[must_use]
    pub fn by_id(&self, id: KeyId) -> Option<&UduDefinition> {
        self.registered.by_id(id)
    }

    /// Every registered definition with its id, in registration order.
    pub fn iter(&self) -> impl Iterator<Item = (KeyId, &UduDefinition)> {
        self.registered.iter()
    }

    /// Whether nothing is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.registered.is_empty()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::catalogue::{Graha, Nakshatra};

    use super::*;
    use crate::row::Lord;

    fn definition(key: &str) -> UduDefinition {
        UduDefinition {
            lords: vec![Lord {
                graha: Graha::Sun,
                years: 12,
            }],
            ..UduDefinition::of(key, Nakshatra::Ashwini)
        }
    }

    #[test]
    fn a_definition_is_refused_by_the_field_its_row_gets_wrong() {
        let mut systems = DashaSystems::new();
        let wide = UduDefinition {
            span: 0,
            ..definition("ACME_WIDE")
        };
        assert_eq!(systems.register(wide).unwrap_err().field(), Some("span"));
        let empty = UduDefinition {
            lords: Vec::new(),
            ..definition("ACME_EMPTY")
        };
        assert_eq!(systems.register(empty).unwrap_err().field(), Some("lords"));
        let badly = definition("acme lower");
        assert_eq!(systems.register(badly).unwrap_err().field(), Some("key"));
        assert!(systems.is_empty());
    }

    #[test]
    fn a_key_registers_once_and_a_sealed_registry_takes_no_more() {
        let mut systems = DashaSystems::new();
        let first = systems.register(definition("ACME_ONE")).unwrap();
        assert!(systems.register(definition("ACME_ONE")).is_err());
        let second = systems.register(definition("ACME_TWO")).unwrap();
        assert_eq!(second.id(), first.id() + 1);
        systems.seal();
        assert!(systems.register(definition("ACME_THREE")).is_err());
        let keys: Vec<&str> = systems.iter().map(|(_, d)| d.key.as_str()).collect();
        assert_eq!(keys, ["ACME_ONE", "ACME_TWO"]);
    }

    #[test]
    fn a_definition_round_trips_through_json_with_its_defaults() {
        let json =
            r#"{"key":"ACME_ONE","lords":[{"graha":"SUN","years":12}],"reference":"ASHWINI"}"#;
        let parsed: UduDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(parsed, definition("ACME_ONE"));
        let back: UduDefinition =
            serde_json::from_str(&serde_json::to_string(&parsed).unwrap()).unwrap();
        assert_eq!(back, parsed);
    }
}
