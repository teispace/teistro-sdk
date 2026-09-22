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
//! assert_eq!(plan.items[0].key, "sdk.reason.pointInRashi", "the lagna frames the rest");
//! assert_eq!(plan.items[1].key, "sdk.reason.grahaInRashi");
//! assert!(plan.items.iter().all(|item| KEYS.contains(&item.key.as_str())));
//! // Nothing but keys and slots: the words are the locale's.
//! let written = serde_json::to_string(&plan)?;
//! assert!(written.contains(r#"{"$entity":"graha.SUN"}"#));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_intl::messages::sdk::reason::dasha;
use teistro_intl::messages::sdk::reason::panchanga as panchanga_messages;
use teistro_intl::messages::sdk::{aspect, condition, karaka, reading, reason};
// The composer below is `phala` too, so the message module is named in
// full where its keys are listed.
use teistro_intl::messages::sdk::phala as phala_messages;
use teistro_intl::{Params, TypedMessage};

mod ashtakavarga;
mod aspects;
mod chalit;
mod conditions;
mod dasha_phala;
mod houses;
mod karakas;
mod panchanga;
mod phala;
mod placements;
mod positions;
mod readings;
mod states;
mod strength;
mod weights;

pub use ashtakavarga::ashtakavarga;
pub use aspects::aspects;
pub use chalit::chalit;
pub use conditions::conditions;
pub use dasha_phala::dasha_phala;
pub use houses::houses;
pub use karakas::karakas;
pub use panchanga::panchanga;
pub use phala::phala;
pub use placements::placements;
pub use positions::positions;
pub use readings::readings;
pub use states::states;
pub use strength::strength;
pub use weights::{bhava_bala, vimshopaka};

/// Every message key a composer of this module can emit.
///
/// The keys are the generated messages' own associated constants, so a
/// message that changes its name changes this list with it, and
/// `check-interpret` holds the list against the packs and against what the
/// composers emit over the corpus, both ways: a key no locale carries would
/// render as a visible fallback, and a key nothing emits is a message nobody
/// reads.
pub const KEYS: [&str; 69] = [
    <reason::PointInRashi as TypedMessage>::KEY,
    <reason::GrahaInRashi as TypedMessage>::KEY,
    <reason::GrahaInBhava as TypedMessage>::KEY,
    <reason::BhavaInRashi as TypedMessage>::KEY,
    <reason::ChalitShift as TypedMessage>::KEY,
    <reason::PointAt as TypedMessage>::KEY,
    <reason::GrahaAt as TypedMessage>::KEY,
    <reason::Occupants as TypedMessage>::KEY,
    <reason::Lordship as TypedMessage>::KEY,
    <reason::strength::Score as TypedMessage>::KEY,
    <reason::strength::Meets as TypedMessage>::KEY,
    <reading::Effect as TypedMessage>::KEY,
    <reading::Says as TypedMessage>::KEY,
    <reading::Timing as TypedMessage>::KEY,
    <reading::LifeSpan as TypedMessage>::KEY,
    <reading::LifeClass as TypedMessage>::KEY,
    <reading::Participants as TypedMessage>::KEY,
    <reading::Status as TypedMessage>::KEY,
    <reading::Severity as TypedMessage>::KEY,
    <aspect::Cast as TypedMessage>::KEY,
    <aspect::Mutual as TypedMessage>::KEY,
    <condition::Dignity as TypedMessage>::KEY,
    <condition::Navamsha as TypedMessage>::KEY,
    <condition::Vargottama as TypedMessage>::KEY,
    <condition::Friendship as TypedMessage>::KEY,
    <condition::Age as TypedMessage>::KEY,
    <condition::Wakefulness as TypedMessage>::KEY,
    <condition::Brightness as TypedMessage>::KEY,
    <condition::Lajjitadi as TypedMessage>::KEY,
    <condition::Undecided as TypedMessage>::KEY,
    <condition::Retrograde as TypedMessage>::KEY,
    <condition::Combust as TypedMessage>::KEY,
    <karaka::OfSeven as TypedMessage>::KEY,
    <karaka::OfEight as TypedMessage>::KEY,
    <phala_messages::GrahaInBhava as TypedMessage>::KEY,
    <phala_messages::LagnaRashi as TypedMessage>::KEY,
    <phala_messages::Tithi as TypedMessage>::KEY,
    <phala_messages::Vara as TypedMessage>::KEY,
    <phala_messages::Nakshatra as TypedMessage>::KEY,
    <phala_messages::Yoga as TypedMessage>::KEY,
    <phala_messages::Namakarana as TypedMessage>::KEY,
    <phala_messages::Gana as TypedMessage>::KEY,
    <phala_messages::Nadi as TypedMessage>::KEY,
    <phala_messages::Yoni as TypedMessage>::KEY,
    <phala_messages::Varna as TypedMessage>::KEY,
    <phala_messages::Tatwa as TypedMessage>::KEY,
    <phala_messages::DashaLord as TypedMessage>::KEY,
    <phala_messages::DashaActivation as TypedMessage>::KEY,
    <phala_messages::AvasthaBaladi as TypedMessage>::KEY,
    <phala_messages::AvasthaJagradadi as TypedMessage>::KEY,
    <phala_messages::AvasthaDeeptadi as TypedMessage>::KEY,
    <phala_messages::AvasthaLajjitadi as TypedMessage>::KEY,
    <phala_messages::Dignity as TypedMessage>::KEY,
    <phala_messages::State as TypedMessage>::KEY,
    <dasha::Phase as TypedMessage>::KEY,
    <dasha::Place as TypedMessage>::KEY,
    <dasha::Points as TypedMessage>::KEY,
    <dasha::Favour as TypedMessage>::KEY,
    <panchanga_messages::Tithi as TypedMessage>::KEY,
    <panchanga_messages::Vara as TypedMessage>::KEY,
    <panchanga_messages::Nakshatra as TypedMessage>::KEY,
    <panchanga_messages::Pada as TypedMessage>::KEY,
    <panchanga_messages::Yoga as TypedMessage>::KEY,
    <panchanga_messages::Karana as TypedMessage>::KEY,
    <panchanga_messages::ByDay as TypedMessage>::KEY,
    <reason::BhavaBala as TypedMessage>::KEY,
    <reason::Vimshopaka as TypedMessage>::KEY,
    <reason::Ashtakavarga as TypedMessage>::KEY,
    <reason::Sarvashtakavarga as TypedMessage>::KEY,
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

/// What a composer asks **the base locale** before choosing between a
/// general message and one written for a particular subject.
///
/// `readings` is the composer that chooses: a rule's own reading where a
/// locale has been given one, and the verse's cited words where it has not.
/// It asks the **base** locale and not the reader's, because the base
/// locale is a fact of the build and the reader's is not — a plan that
/// changed shape with the reader would not be language-neutral, and the
/// same chart would compose differently for two people
/// (`03-design/interpret-composers.md` §5).
///
/// It is a trait rather than an `&Intl` so a composer can be tested without
/// building a locale engine, and so the question a composer asks is
/// visible in its signature.
pub trait Vocabulary {
    /// Whether the base locale carries this form of the record at this
    /// **full** catalogue key (`nakshatra.ASHWINI`, `phala`).
    ///
    /// One question for every subject, because a composer asking after a
    /// graha in a bhava and one asking after a rule are asking the same
    /// thing of the same records. A reading of a rule is the `name` form
    /// of a `rule` record, which is what [`Vocabulary::has_reading`]
    /// spells.
    fn has_form(&self, key: &str, form: &str) -> bool;

    /// Whether the base locale carries a reading of this rule, named by
    /// the rule's own key (`RUCHAKA`, not `rule.RUCHAKA`).
    fn has_reading(&self, rule: &str) -> bool {
        self.has_form(&reading_key(rule), NAME_FORM)
    }
}

/// The form a record's summary is carried under, which is what a reading
/// is named by.
pub const NAME_FORM: &str = "name";

/// The catalogue key a rule's reading is carried under.
///
/// `rule` is the catalogue's only **open** kind, so this is a spelling and
/// not a lookup (`03-design/interpretation-records.md` §4).
#[must_use]
pub fn reading_key(rule: &str) -> String {
    format!("rule.{rule}")
}

/// A vocabulary carrying no reading at all: every rule composes from the
/// words its verse cites.
///
/// This is what a consumer that has not loaded a readings pack has, and it
/// is named rather than defaulted — a composer that silently assumed one
/// or the other would be making the consumer's choice for it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoReadings;

impl Vocabulary for NoReadings {
    fn has_form(&self, _key: &str, _form: &str) -> bool {
        false
    }
}

impl Vocabulary for teistro_intl::Intl {
    /// The base locale's own record, which is what decides the plan's
    /// shape. A reader's locale that lacks the record falls back along its
    /// chain when the item is rendered; that is the renderer's business
    /// and not the plan's.
    fn has_form(&self, key: &str, form: &str) -> bool {
        self.entity_from(teistro_intl::source::BASE_LOCALE, key)
            .is_some_and(|record| record.form(form).is_some())
    }
}
