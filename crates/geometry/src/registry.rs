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
    #[must_use]
    pub fn new() -> Layouts {
        Layouts {
            shipped: rows::shipped(),
            registered: Registry::new(Kind::ChartLayout),
        }
    }

    /// Registers a consumer's layout.
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
    pub fn seal(&mut self) {
        self.registered.seal();
    }

    /// The layout with a key, shipped or registered.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Layout> {
        self.shipped
            .iter()
            .find(|layout| layout.key == key)
            .or_else(|| self.registered.get(key).map(|(layout, _)| layout))
    }

    /// Every layout, the shipped ones first in catalogue order.
    pub fn iter(&self) -> impl Iterator<Item = &Layout> {
        self.shipped
            .iter()
            .chain(self.registered.iter().map(|(_, layout)| layout))
    }
}
