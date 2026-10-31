//! `sdk.intl`: the locale engine — a message, an entity's forms,
//! transliteration, and a pack loaded at run time.

use teistro_core::Status;
use teistro_core::error::Error;
use teistro_intl::civil::{CivilText, DateTextRequest, TimeTextRequest};
use teistro_intl::lexicon::{Lexicon, LexiconRequest, Lookup, LookupRequest};
use teistro_intl::source::Entity;
use teistro_intl::translit::{Script, transliterate};
use teistro_intl::words::{InWords, InWordsRequest};
use teistro_intl::{Intl, Loaded, Params, Rendered, TypedMessage};

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

/// The refusal of a locale tag, the same from the builder and from a change:
/// a consumer who mistyped a tag wants the same sentence wherever they did.
pub(crate) fn refused_locale(engine: &Intl, tag: &str, why: &str) -> Error {
    let known: Vec<&str> = engine.locales().map(|l| l.tag.as_str()).collect();
    Error::unsupported(why)
        .with_field("locale")
        .with_hint(format!(
            "`{tag}` is not loaded; the locales are {}",
            known.join(", ")
        ))
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

    /// The loaded locale `tag` means, by RFC 4647 lookup: `ne` is
    /// `ne-Deva-NP`. Nothing changes.
    ///
    /// ```
    /// use teistro::Context;
    ///
    /// let sdk = Context::builder().build()?;
    /// assert_eq!(sdk.intl().lookup_locale("hi")?, "hi-Deva-IN");
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`IntlArea::set_locale`].
    pub fn lookup_locale(self, tag: &str) -> Result<String, Error> {
        let engine = self.context.locale_engine();
        engine
            .lookup_locale(tag)
            .map(str::to_owned)
            .map_err(|e| refused_locale(&engine, tag, &e.to_string()))
    }

    /// Changes the locale every render resolves from, by RFC 4647 lookup,
    /// and answers the one chosen, which may be longer than `tag`: `ne`
    /// selects `ne-Deva-NP`.
    ///
    /// # Errors
    ///
    /// A tag the embedded bundles and the loaded packs do not answer, or
    /// several answer with nothing to settle which, with the locales they
    /// carry as the hint. Nothing falls back to a default.
    pub fn set_locale(self, tag: &str) -> Result<String, Error> {
        let mut engine = self.context.locale_engine_mut();
        engine
            .set_locale(tag)
            .map(str::to_owned)
            .map_err(|e| refused_locale(&engine, tag, &e.to_string()))
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

    /// A number or an amount in words, by CLDR's rule-based number
    /// formats, with its figures (`03-design/amounts-in-words.md`): in
    /// the request's locale, or this area's when it names none.
    ///
    /// ```
    /// # use teistro::{Context, Ephemeris};
    /// # fn main() -> Result<(), teistro::Error> {
    /// let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
    /// let asked = teistro::InWordsRequest::from_json(
    ///     r#"{"amount": {"currency": "NPR", "value": {"decimal": "10000.50"}, "only": true}, "grouping": "INDIAN"}"#,
    /// )?;
    /// let words = sdk.intl().in_words(&asked)?;
    /// assert_eq!(words.text, "ten thousand rupees and fifty paisa only");
    /// assert_eq!(words.figure, "10,000.50");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`teistro_intl::Intl::in_words`], named under `inWords`.
    pub fn in_words(self, request: &InWordsRequest) -> Result<InWords, Error> {
        self.context.locale_engine().in_words(request)
    }

    /// A time of day as text and parts, through the locale's pattern and
    /// parts of the day: in the request's locale, or this area's when it
    /// names none (`03-design/intl-engine-and-packs.md`).
    ///
    /// ```
    /// # use teistro::{Context, Ephemeris};
    /// # fn main() -> Result<(), teistro::Error> {
    /// let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
    /// let asked = teistro::TimeTextRequest::from_json(
    ///     r#"{"at": {"hour": 18, "minute": 30}, "hour12": true, "locale": "hi-Deva-IN"}"#,
    /// )?;
    /// let dusk = sdk.intl().time(&asked)?;
    /// assert_eq!(dusk.text, "शाम ६:३०");
    /// assert!(dusk.is_draft);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`teistro_intl::Intl::time`], named under `time`.
    pub fn time(self, request: &TimeTextRequest) -> Result<CivilText, Error> {
        self.context.locale_engine().time(request)
    }

    /// A date as text and parts, in its own calendar: in the request's
    /// locale, or this area's when it names none.
    ///
    /// ```
    /// # use teistro::{Context, Ephemeris};
    /// # fn main() -> Result<(), teistro::Error> {
    /// let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
    /// let asked = teistro::DateTextRequest::from_json(
    ///     r#"{"at": {"calendar": "calendar.GREGORIAN", "year": 2026, "month": 10, "day": 31}, "style": "LONG"}"#,
    /// )?;
    /// assert_eq!(sdk.intl().date(&asked)?.text, "31 October 2026");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`teistro_intl::Intl::date`], named under `date`.
    pub fn date(self, request: &DateTextRequest) -> Result<CivilText, Error> {
        self.context.locale_engine().date(request)
    }

    /// Every catalogue member's names in a locale, romanised forms too
    /// (`03-design/lexicon.md` §1): the request's locale, or this area's
    /// when it names none. Built once per context and locale.
    ///
    /// ```
    /// # use teistro::{Context, Ephemeris};
    /// # fn main() -> Result<(), teistro::Error> {
    /// let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
    /// let asked = teistro::LexiconRequest::from_json(r#"{"locale": "ne", "kinds": ["graha"]}"#)?;
    /// let grahas = sdk.intl().lexicon(&asked)?;
    /// assert_eq!(grahas.entries[0].forms[0].text, "सूर्य");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`teistro_intl::Intl::lexicon`], named under `lexicon`.
    pub fn lexicon(self, request: &LexiconRequest) -> Result<Lexicon, Error> {
        self.context.locale_engine().lexicon(request)
    }

    /// Every span of a text that may name a catalogue member, each with
    /// every member it may name (`03-design/lexicon.md` §1.3).
    ///
    /// ```
    /// # use teistro::{Context, Ephemeris};
    /// # fn main() -> Result<(), teistro::Error> {
    /// let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
    /// let asked = teistro::LookupRequest::from_json(r#"{"text": "the Moon is in Rohini", "kinds": ["graha", "nakshatra"]}"#)?;
    /// let found = sdk.intl().lookup(&asked)?;
    /// let keys: Vec<&str> = found.matches.iter().map(|m| m.candidates[0].key.as_str()).collect();
    /// assert_eq!(keys, ["graha.MOON", "nakshatra.ROHINI"]);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`teistro_intl::Intl::lookup`], named under `lookup`.
    pub fn lookup(self, request: &LookupRequest) -> Result<Lookup, Error> {
        self.context.locale_engine().lookup(request)
    }

    /// A locale pack loaded at run time, whose locales join the embedded
    /// ones. The bytes are a `.tpack` or `.tbundle` written by
    /// `teistro-intl build`, from wherever the consumer got them — a file
    /// beside the binary, a download, an asset in an application bundle.
    ///
    /// This is how a corpus too large to embed reaches a consumer. The
    /// SDK's readings — one for each yoga and dosha, in four languages —
    /// ship this way rather than inside the library, because a consumer
    /// computing a Julian day should not carry them to do it
    /// (`03-design/interpretation-records.md` §3). Loading one changes
    /// what a composer **says**, not how it is called.
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
