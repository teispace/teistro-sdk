//! The Teistro SDK's composers (`docs/03-design/interpret-composers.md`).
//!
//! A composer turns what the SDK computed into a **narrative plan**: an
//! ordered list of [`Item`]s, each a message key and its slots. A plan holds
//! no words, so one plan renders in every locale the engine carries, and a
//! golden plan is a test of the composer rather than of the translator.
//! `sdk.intl` renders an item — `render(&item.key, &item.params)` — and a
//! consumer may render, reorder or drop items as its own page needs.
//!
//! Every key a composer can emit is in [`KEYS`], and `check-interpret` holds
//! that list against the message packs both ways: a key no locale carries
//! would render as a visible fallback, and a key no composer emits is a
//! message nobody reads.
//!
//! ```
//! use teistro_core::catalogue::{Dignity, Rashi};
//! use teistro_interpret::{KEYS, placements};
//! use teistro_rules::{House, Placement, RuleChart};
//!
//! // Every body at 15° Aries in the first house.
//! let placement = Placement {
//!     longitude: 15.0,
//!     sign: Rashi::Aries,
//!     house: House::try_new(1)?,
//!     dignity: Dignity::Neutral,
//!     retrograde: false,
//!     combust: false,
//!     karaka7: None,
//!     karaka8: None,
//!     navamsha: Rashi::Aries,
//! };
//! let chart = RuleChart { placements: [placement; 10], panchanga: None, strengths: None };
//!
//! let plan = placements(&chart);
//! assert_eq!(plan.items[0].key, "sdk.reason.grahaInRashi");
//! assert!(plan.items.iter().all(|item| KEYS.contains(&item.key.as_str())));
//! // Nothing but keys and slots: the words are the locale's.
//! let written = serde_json::to_string(&plan)?;
//! assert!(written.contains(r#"{"$entity":"graha.SUN"}"#));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_intl::messages::sdk::{reading, reason};
use teistro_intl::{Params, TypedMessage};

mod placements;
mod readings;
mod strength;

pub use placements::placements;
pub use readings::readings;
pub use strength::strength;

/// Every message key a composer of this module can emit.
///
/// The keys are the generated messages' own associated constants, so a
/// message that changes its name changes this list with it, and
/// `check-interpret` holds the list against the packs and against what the
/// composers emit over the corpus, both ways: a key no locale carries would
/// render as a visible fallback, and a key nothing emits is a message nobody
/// reads.
pub const KEYS: [&str; 10] = [
    <reason::GrahaInRashi as TypedMessage>::KEY,
    <reason::GrahaInBhava as TypedMessage>::KEY,
    <reason::Occupants as TypedMessage>::KEY,
    <reason::strength::Score as TypedMessage>::KEY,
    <reading::Effect as TypedMessage>::KEY,
    <reading::LifeSpan as TypedMessage>::KEY,
    <reading::LifeClass as TypedMessage>::KEY,
    <reading::Participants as TypedMessage>::KEY,
    <reading::Status as TypedMessage>::KEY,
    <reading::Severity as TypedMessage>::KEY,
];

/// One thing to say: a message key and the slots it is said with.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    /// The message, as `sdk.reason.grahaInRashi`.
    pub key: String,
    /// Its slots, in the order a message names them being irrelevant: the
    /// map is ordered, so a plan is the same bytes every time.
    pub params: Params,
}

impl Item {
    /// An item from a **typed** message — `messages::sdk::reason::GrahaInRashi`
    /// and its like, generated from `i18n/` — which is how a composer says
    /// anything: the key is the message's own, the slots are its fields, and
    /// a message that gains or loses one stops the composer compiling.
    #[must_use]
    pub fn of<M: TypedMessage>(message: &M) -> Item {
        Item {
            key: String::from(M::KEY),
            params: message.params(),
        }
    }

    /// An item from a key and slots, for a consumer composing its own.
    #[must_use]
    pub fn new(key: impl Into<String>, params: Params) -> Item {
        Item {
            key: key.into(),
            params,
        }
    }
}

/// What a composer says, in the order it says it.
///
/// ```
/// use teistro_interpret::{Item, Plan};
/// use teistro_intl::{Value, params};
///
/// let mut plan = Plan::default();
/// plan.push("sdk.reason.appName", params([]));
/// assert_eq!(plan.items.len(), 1);
/// assert_eq!(plan.items[0], Item::new("sdk.reason.appName", params([])));
/// // A plan is written down as the list it is.
/// let written = serde_json::to_string(&plan)?;
/// assert_eq!(written, r#"[{"key":"sdk.reason.appName","params":{}}]"#);
/// assert_eq!(serde_json::from_str::<Plan>(&written)?, plan);
/// # let _ = Value::Int(1);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Plan {
    /// The items, in the order they are said.
    pub items: Vec<Item>,
}

impl Plan {
    /// Says a typed message.
    pub fn say<M: TypedMessage>(&mut self, message: &M) {
        self.items.push(Item::of(message));
    }

    /// Adds an item, for a consumer composing its own.
    pub fn push(&mut self, key: impl Into<String>, params: Params) {
        self.items.push(Item::new(key, params));
    }

    /// Whether it says nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// How many things it says.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Its items, borrowed.
    pub fn iter(&self) -> std::slice::Iter<'_, Item> {
        self.items.iter()
    }

    /// Every key it uses, in the order it uses them, each once.
    #[must_use]
    pub fn keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = Vec::new();
        for item in &self.items {
            if !keys.contains(&item.key.as_str()) {
                keys.push(&item.key);
            }
        }
        keys
    }
}

impl FromIterator<Item> for Plan {
    fn from_iter<I: IntoIterator<Item = Item>>(items: I) -> Plan {
        Plan {
            items: items.into_iter().collect(),
        }
    }
}

impl IntoIterator for Plan {
    type Item = Item;
    type IntoIter = std::vec::IntoIter<Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'p> IntoIterator for &'p Plan {
    type Item = &'p Item;
    type IntoIter = std::slice::Iter<'p, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
