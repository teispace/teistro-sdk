//! `sdk.intl`: the locale engine — a message, an entity's forms,
//! transliteration, and a pack loaded at run time.

use teistro_core::Status;
use teistro_core::error::Error;
use teistro_intl::source::Entity;
use teistro_intl::translit::{Script, transliterate};
use teistro_intl::{Loaded, Params, Rendered, TypedMessage};

use crate::context::Context;

/// `sdk.intl`: what the SDK says, in the locale it was asked for.
///
/// Every operation borrows the engine for its own call, so a render and a
/// `set_locale` from two places cannot overlap. That is the
/// single-threaded rule the context already carries, enforced one call
/// at a time rather than for the context's whole life.
///
/// **`messages` is not here**, and that is the shape rather than an
/// omission: the typed accessor tree every other binding spells
/// `sdk.intl.messages.sdk.reason.grahaInBhava({ … })` is a *module* tree
/// in Rust, `teistro::messages::sdk::reason::GrahaInBhava { … }`, which
/// is what a namespace is in this language. Hand one to
/// [`IntlArea::render_typed`].
#[derive(Clone, Copy, Debug)]
pub struct IntlArea<'a> {
    context: &'a Context,
}

impl<'a> IntlArea<'a> {
    pub(crate) fn of(context: &'a Context) -> IntlArea<'a> {
        IntlArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// The locale every render resolves from.
    #[must_use]
    pub fn locale(self) -> String {
        self.context.locale_engine().locale().to_owned()
    }

    /// Changes the locale every render resolves from.
    ///
    /// # Errors
    ///
    /// A locale the embedded bundles and the loaded packs do not carry,
    /// with the ones they do as the hint.
    pub fn set_locale(self, tag: &str) -> Result<(), Error> {
        let mut engine = self.context.locale_engine_mut();
        engine.set_locale(tag).map_err(|e| {
            let known: Vec<&str> = engine.locales().map(|l| l.tag.as_str()).collect();
            Error::unsupported(e.to_string())
                .with_field("locale")
                .with_hint(format!(
                    "`{tag}` is not loaded; the locales are {}",
                    known.join(", ")
                ))
        })
    }

    /// Whether the locale carries a message.
    #[must_use]
    pub fn has(self, key: &str) -> bool {
        self.context.locale_engine().has(key)
    }

    /// A message, by its key and its parameters.
    ///
    /// Prefer [`IntlArea::render_typed`], which cannot be given a
    /// parameter the message does not have.
    #[must_use]
    pub fn render(self, key: &str, params: &Params) -> Rendered {
        self.context.locale_engine().render(key, params)
    }

    /// A message by its **typed** accessor, which is a value of the
    /// message's own parameters — `messages::sdk::reason::GrahaInBhava`
    /// and its like, generated from `i18n/`.
    #[must_use]
    pub fn render_typed<M: TypedMessage>(&self, message: &M) -> Rendered {
        self.context.locale_engine().render_typed(message)
    }

    /// A catalogued entity's forms in this locale — `short`, `name`,
    /// `prose`, `iast`, the glyph, and any the locale adds.
    ///
    /// # Errors
    ///
    /// A key the locale and its fallbacks carry no entity for.
    pub fn entity(self, key: &str) -> Result<Entity, Error> {
        let engine = self.context.locale_engine();
        let locale = engine.locale();
        // Cloned rather than lent: the engine is behind a borrow that
        // ends with this call, and an `Entity` is a handful of strings.
        engine.entity_from(locale, key).cloned().ok_or_else(|| {
            Error::unsupported(format!("no entity `{key}` in `{locale}`"))
                .with_field("key")
                .with_hint(format!(
                    "the catalogue's key, as `graha.SUN`; `{locale}` and its fallbacks carry none"
                ))
        })
    }

    /// A term written in one script, read in another.
    ///
    /// # Errors
    ///
    /// A script pair the transliterator does not carry.
    pub fn transliterate(self, text: &str, from: Script, to: Script) -> Result<String, Error> {
        // The same refusal the boundary gives, naming the field a
        // caller can change.
        transliterate(text, from, to)
            .map_err(|e| Error::unsupported(e.to_string()).with_field("to"))
    }

    /// A locale pack loaded at run time, whose locales join the embedded
    /// ones.
    ///
    /// # Errors
    ///
    /// Bytes that are not a pack, or a pack built for another catalogue.
    pub fn load_pack(self, bytes: &[u8]) -> Result<Loaded, Error> {
        self.context
            .locale_engine_mut()
            .load_pack(bytes)
            .map_err(|e| Error::new(Status::Pack, e.to_string()).with_field("bytes"))
    }
}
