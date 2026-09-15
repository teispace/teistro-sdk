//! `sdk.keys`: a catalogue key to its packed id and back.

use teistro_core::catalogue::Kind;
use teistro_core::error::{Detail, Error};
use teistro_core::key::{KeyId, resolve};

use crate::context::Context;

/// `sdk.keys`: the boundary's own spellings — `graha.SUN` and its like —
/// to the packed id every fixture and pack carries, and back.
///
/// A Rust consumer has the catalogue's enums and mostly wants those;
/// this area is for the other direction, where a key arrived as text —
/// from a fixture, a settings document, a request.
#[derive(Clone, Copy, Debug)]
pub struct KeysArea<'a> {
    context: &'a Context,
}

impl<'a> KeysArea<'a> {
    pub(crate) fn of(context: &'a Context) -> KeysArea<'a> {
        KeysArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// The packed id of a full key (`graha.SUN`), an alias, a former key,
    /// or a member this context registered (`chart_layout.ACME_KERALA`).
    ///
    /// # Errors
    ///
    /// A key no catalogued or registered member has, with the nearest
    /// catalogued one as the hint.
    pub fn id(self, key: &str) -> Result<KeyId, Error> {
        resolve(key).or_else(|refusal| self.registered_id(key).ok_or_else(|| Error::from(refusal)))
    }

    /// The full key of a packed id (`graha.SUN`), catalogued or registered
    /// with this context.
    ///
    /// # Errors
    ///
    /// An id no catalogued or registered member has.
    pub fn name(self, id: KeyId) -> Result<String, Error> {
        if let (Some(kind), Some(key)) = (id.kind(), id.key()) {
            return Ok(format!("{}.{key}", kind.name()));
        }
        self.registered_name(id).ok_or_else(|| {
            Error::unsupported(format!(
                "no catalogued or registered member has id {:#010x}",
                id.bits()
            ))
            .with_detail(Detail::UnknownKey)
            .with_field("id")
        })
    }

    /// A registered member's id, from the context's registries: chart layouts
    /// and nakshatra-seeded dasha systems. A kind that gains a registry is one
    /// more arm here.
    fn registered_id(self, key: &str) -> Option<KeyId> {
        let (kind, name) = key.split_once('.')?;
        match Kind::from_name(kind)? {
            Kind::ChartLayout => self
                .context
                .layouts()
                .id(name)
                .filter(|id| id.is_registered()),
            Kind::DashaSystem => self.context.dashas().id(name),
            _ => None,
        }
    }

    /// A registered member's full key, from the context's registries.
    fn registered_name(self, id: KeyId) -> Option<String> {
        match id.kind()? {
            Kind::ChartLayout if id.is_registered() => self
                .context
                .layouts()
                .by_id(id)
                .map(|layout| format!("{}.{}", Kind::ChartLayout.name(), layout.key)),
            Kind::DashaSystem => self
                .context
                .dashas()
                .by_id(id)
                .map(|definition| format!("{}.{}", Kind::DashaSystem.name(), definition.key)),
            _ => None,
        }
    }
}
