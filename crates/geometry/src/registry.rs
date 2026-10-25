//! The layouts a context can draw in: the six the SDK ships, and any a
//! consumer registers (ADR-0026 §1).
//!
//! A regional layout is a row, not a fork. A consumer builds a [`Layout`]
//! and registers it. It passes the same checks a shipped row does
//! ([`Layout::validate`]) and is drawn by the same placement, and a key a
//! shipped layout already has is refused, so a registered row can add a
//! layout and never quietly replace one.
//!
//! ```
//! use teistro_core::catalogue::ChartLayout;
//! use teistro_geometry::{Layouts, rows};
//!
//! let mut layouts = Layouts::new();
//! assert!(layouts.get("NORTH_INDIAN").is_some());
//!
//! // A consumer's own regional chart: here the South Indian grid under a
//! // new name, as a stand-in for a row of their own.
//! let mut kerala = rows::south_indian();
//! kerala.key = String::from("ACME_KERALA");
//! let id = layouts.register(kerala)?;
//! assert!(id.is_registered());
//! assert!(layouts.get("ACME_KERALA").is_some());
//! assert_eq!(layouts.id("ACME_KERALA"), Some(id));
//! assert_eq!(layouts.id("NORTH_INDIAN"), Some(ChartLayout::NorthIndian.key_id()));
//!
//! // A shipped key cannot be taken over.
//! let mut impostor = rows::east_indian();
//! impostor.key = String::from(ChartLayout::NorthIndian.key());
//! assert!(layouts.register(impostor).is_err());
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use teistro_core::catalogue::{ChartLayout, Kind};
use teistro_core::error::Error;
use teistro_core::key::KeyId;
use teistro_core::registry::{Definition, Registry};

use crate::layout::Layout;
use crate::rows;

impl Definition for Layout {
    fn key(&self) -> &str {
        &self.key
    }

    fn validate(&self) -> Result<(), Error> {
        Layout::validate(self)
    }
}

/// The shipped layouts and the registered ones, looked up by key.
///
/// ```
/// use teistro_geometry::Layouts;
///
/// let layouts = Layouts::default();
/// assert_eq!(layouts.iter().count(), 6);
/// ```
#[derive(Clone, Debug)]
pub struct Layouts {
    shipped: Vec<Layout>,
    registered: Registry<Layout>,
}

impl Default for Layouts {
    fn default() -> Layouts {
        Layouts::new()
    }
}

impl Layouts {
    /// The shipped layouts, with nothing registered yet.
    ///
    /// ```
    /// use teistro_geometry::Layouts;
    ///
    /// assert!(Layouts::new().get("SUDARSHAN_CHAKRA").is_some());
    /// ```
    #[must_use]
    pub fn new() -> Layouts {
        Layouts {
            shipped: rows::shipped(),
            registered: Registry::new(Kind::ChartLayout),
        }
    }

    /// Registers a consumer's layout.
    ///
    /// ```
    /// use teistro_geometry::{Layouts, rows};
    ///
    /// let mut layouts = Layouts::new();
    /// let mut own = rows::east_indian();
    /// own.key = String::from("ACME_ODIA");
    /// assert!(layouts.register(own.clone()).is_ok());
    /// // Its key is taken now.
    /// assert!(layouts.register(own).is_err());
    /// // A row the checks refuse is refused by the field.
    /// let mut broken = rows::east_indian();
    /// broken.key = String::from("ACME_BROKEN");
    /// broken.sources.clear();
    /// assert_eq!(layouts.register(broken).unwrap_err().field(), Some("sources"));
    /// ```
    ///
    /// # Errors
    ///
    /// A layout the checks refuse, naming the field; a key a shipped layout
    /// has or another registered one took; a sealed registry.
    pub fn register(&mut self, layout: Layout) -> Result<KeyId, Error> {
        if ChartLayout::from_key(&layout.key).is_some() {
            return Err(Error::invalid_arg(format!(
                "`{}` is a layout the SDK ships; register yours under a key of its own",
                layout.key
            ))
            .with_field("key"));
        }
        self.registered.register(layout)
    }

    /// Refuses further registrations, so nothing changes under a context
    /// that has started drawing.
    ///
    /// ```
    /// use teistro_geometry::{Layouts, rows};
    ///
    /// let mut layouts = Layouts::new();
    /// layouts.seal();
    /// let mut late = rows::south_indian();
    /// late.key = String::from("ACME_LATE");
    /// assert!(layouts.register(late).is_err());
    /// ```
    pub fn seal(&mut self) {
        self.registered.seal();
    }

    /// The layout with a key, shipped or registered.
    ///
    /// ```
    /// use teistro_geometry::Layouts;
    ///
    /// let layouts = Layouts::new();
    /// assert_eq!(layouts.get("NEPALI_LOTUS").map(|layout| layout.key.as_str()), Some("NEPALI_LOTUS"));
    /// assert!(layouts.get("ACME_UNKNOWN").is_none());
    /// ```
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Layout> {
        self.shipped
            .iter()
            .find(|layout| layout.key == key)
            .or_else(|| self.registered.get(key).map(|(layout, _)| layout))
    }

    /// The id of the layout with a key: its catalogue id when the SDK ships
    /// it, or the one this registry gave it.
    ///
    /// ```
    /// use teistro_core::catalogue::ChartLayout;
    /// use teistro_geometry::Layouts;
    ///
    /// let layouts = Layouts::new();
    /// assert_eq!(layouts.id("WESTERN_WHEEL"), Some(ChartLayout::WesternWheel.key_id()));
    /// assert_eq!(layouts.id("ACME_UNKNOWN"), None);
    /// ```
    #[must_use]
    pub fn id(&self, key: &str) -> Option<KeyId> {
        match ChartLayout::from_key(key) {
            Some(member) => Some(member.key_id()),
            None => self.registered.get(key).map(|(_, id)| id),
        }
    }

    /// The layout with an id: a shipped member's, or one this registry gave.
    ///
    /// ```
    /// use teistro_core::catalogue::ChartLayout;
    /// use teistro_geometry::{Layouts, rows};
    ///
    /// let mut layouts = Layouts::new();
    /// assert_eq!(layouts.by_id(ChartLayout::SouthIndian.key_id()).map(|layout| layout.key.as_str()), Some("SOUTH_INDIAN"));
    /// let mut own = rows::south_indian();
    /// own.key = String::from("ACME_TAMIL");
    /// let id = layouts.register(own)?;
    /// assert_eq!(layouts.by_id(id).map(|layout| layout.key.as_str()), Some("ACME_TAMIL"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn by_id(&self, id: KeyId) -> Option<&Layout> {
        match ChartLayout::try_from(id) {
            Ok(member) => self.get(member.key()),
            Err(_) => self.registered.by_id(id),
        }
    }

    /// Every layout, the shipped ones first in catalogue order.
    ///
    /// ```
    /// use teistro_geometry::{Layouts, rows};
    ///
    /// let mut layouts = Layouts::new();
    /// let mut own = rows::north_indian();
    /// own.key = String::from("ACME_OWN");
    /// layouts.register(own)?;
    /// let keys: Vec<&str> = layouts.iter().map(|layout| layout.key.as_str()).collect();
    /// // The shipped ones first, in catalogue order; then the registered.
    /// assert_eq!(keys.first(), Some(&"NORTH_INDIAN"));
    /// assert_eq!(keys.last(), Some(&"ACME_OWN"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &Layout> {
        self.shipped
            .iter()
            .chain(self.registered.iter().map(|(_, layout)| layout))
    }
}
