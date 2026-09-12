//! `sdk.keys`: a catalogue key to its packed id and back.

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
    pub fn context(&self) -> &'a Context {
        self.context
    }

    /// The packed id of a full key (`graha.SUN`), an alias, or a former
    /// key.
    ///
    /// # Errors
    ///
    /// A key no catalogued member has, with the nearest one as the
    /// hint.
    pub fn id(&self, key: &str) -> Result<KeyId, Error> {
        resolve(key).map_err(Error::from)
    }

    /// The full key of a packed id (`graha.SUN`).
    ///
    /// # Errors
    ///
    /// An id no catalogued member has.
    pub fn name(&self, id: KeyId) -> Result<String, Error> {
        let (Some(kind), Some(key)) = (id.kind(), id.key()) else {
            return Err(Error::unsupported(format!(
                "no catalogued member has id {:#010x}",
                id.bits()
            ))
            .with_detail(Detail::UnknownKey)
            .with_field("id"));
        };
        Ok(format!("{}.{key}", kind.name()))
    }
}
