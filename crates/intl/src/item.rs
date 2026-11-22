//! One thing to say: a message key and its slots, the unit a composer's
//! plan is made of and a render takes (`03-design/plans-at-the-boundary.md`).

use serde::{Deserialize, Serialize};

use crate::{Params, TypedMessage};

/// One thing to say: a message key and the slots it is said with.
///
/// ```
/// use teistro_intl::Item;
/// use teistro_intl::{Value, params};
///
/// let item = Item::new("my.app.greeting", params([("count", Value::Int(3))]));
/// assert_eq!(serde_json::to_string(&item)?, r#"{"key":"my.app.greeting","params":{"count":3}}"#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "words-schema", derive(schemars::JsonSchema))]
// Named as the bindings name it: a plan's item, whose wire form their own
// hand layers declared until this record crossed (`intl.render`).
#[cfg_attr(feature = "words-schema", schemars(rename = "PlanItem"))]
#[serde(deny_unknown_fields)]
pub struct Item {
    /// The message, as `sdk.reason.grahaInRashi`.
    pub key: String,
    /// Its slots, in the order a message names them being irrelevant: the
    /// map is ordered, so a plan is the same bytes every time.
    #[cfg_attr(
        feature = "words-schema",
        schemars(with = "std::collections::BTreeMap<String, serde_json::Value>")
    )]
    pub params: Params,
}

impl Item {
    /// An item from a **typed** message — `messages::sdk::reason::GrahaInRashi`
    /// and its like, generated from `i18n/` — which is how a composer says
    /// anything: the key is the message's own, the slots are its fields, and
    /// a message that gains or loses one stops the composer compiling.
    ///
    /// ```
    /// use teistro_intl::Item;
    /// use teistro_intl::messages::sdk::reason;
    /// use teistro_intl::{TypedMessage, Value};
    ///
    /// let item = Item::of(&reason::BhavaBala { bhava: 1, virupas: 473.39 });
    /// assert_eq!(item.key, <reason::BhavaBala as TypedMessage>::KEY);
    /// assert_eq!(item.params.get("bhava"), Some(&Value::Int(1)));
    /// ```
    #[must_use]
    pub fn of<M: TypedMessage>(message: &M) -> Item {
        Item {
            key: String::from(M::KEY),
            params: message.params(),
        }
    }

    /// An item from a key and slots, for a consumer composing its own.
    ///
    /// ```
    /// use teistro_intl::Item;
    /// use teistro_intl::params;
    ///
    /// let item = Item::new("my.app.footer", params([]));
    /// assert_eq!(item.key, "my.app.footer");
    /// assert!(item.params.is_empty());
    /// ```
    #[must_use]
    pub fn new(key: impl Into<String>, params: Params) -> Item {
        Item {
            key: key.into(),
            params,
        }
    }
}
