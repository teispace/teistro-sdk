//! The context: the settings a chart is computed under, the locale it is
//! described in, and the ephemeris it is computed with.

use core::cell::{Ref, RefCell, RefMut};

use teistro_astro::DeltaTModel;
use teistro_astro::completion::{Completed, Completion};
use teistro_core::envelope::Hash;
use teistro_core::error::{Error, Status};
use teistro_core::settings::{
    DEFAULT_PROFILE, Profile, Resolved, SHIPPED_PROFILES, Settings, SettingsPatch,
};
use teistro_intl::Intl;
use teistro_intl::pack::locales_from_packs;
use teistro_port_ephemeris::{CachingProvider, EphemerisProvider, PositionRequest};

use crate::BUNDLES;
use crate::area::{CalendarArea, EngineArea, FrameArea, IntlArea, KeysArea, TimeArea};
use crate::ephemeris::{self, Ephemeris, no_ephemeris};

/// One context: built once, read many times.
///
/// It carries what every answer is computed against — the resolved
/// settings, the locale engine, the ΔT model and the ephemeris — so that
/// two contexts with the same settings hash compute the same numbers and
/// the hash is a cache key.
///
/// **One context serves one thread, and a worker builds its own** — the
/// same rule the other three bindings state, and in Rust it is enforced
/// rather than advised: a `Context` is neither `Send` nor `Sync`.
///
/// The reason is worth knowing, because it is not the one a reader would
/// guess. The ephemeris cannot be it: the port requires `Send + Sync` of
/// a provider. It is the **locale engine** — `teistro_intl`'s plural
/// rules hold an `icu_plurals::PluralRules`, whose data payload is
/// reference-counted with `Rc`. `tests/surface.rs` builds one per worker,
/// which is the pattern this leaves, and
/// `03-design/rust-consumer-surface.md` §9 records what it would take to
/// lift (`icu_provider`'s `sync` feature, which exists).
pub struct Context {
    settings: Resolved,
    provider: Option<Box<dyn EphemerisProvider>>,
    intl: RefCell<Intl>,
    /// Resolved once from `time.delta_t`, as the boundary resolves it,
    /// so the two agree on which model a context uses.
    ///
    /// Read by `time()`, and by `positions` when it lands.
    delta_t: DeltaTModel,
}

impl core::fmt::Debug for Context {
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        out.debug_struct("Context")
            .field("profile", &self.profile())
            .field("settings_hash", &self.settings_hash().to_string())
            .field(
                "ephemeris",
                &self
                    .provider
                    .as_ref()
                    .map(|p| p.capabilities().identity.name.clone()),
            )
            .finish_non_exhaustive()
    }
}

impl Context {
    /// A builder for a context.
    #[must_use]
    pub fn builder() -> ContextBuilder {
        ContextBuilder::default()
    }

    /// The resolved settings, every knob as this context will use it.
    #[must_use]
    pub fn settings(&self) -> &Settings {
        &self.settings.settings
    }

    /// The profile the settings were resolved from.
    #[must_use]
    pub fn profile(&self) -> &str {
        self.settings.profile.as_str()
    }

    /// The SHA-256 of the canonical settings document: the
    /// `settings_hash` every result's provenance carries, and the cache
    /// key two contexts share when they compute the same numbers.
    ///
    /// The core's own `Hash`, not thirty-two bytes: it knows how to
    /// write itself as hex, and a second type with the same invariant
    /// would be a type with no new one (ADR-0023).
    #[must_use]
    pub fn settings_hash(&self) -> Hash {
        self.settings.settings.hash()
    }

    /// The ephemeris this context computes positions with, when it has
    /// one.
    #[must_use]
    pub fn ephemeris(&self) -> Option<&dyn EphemerisProvider> {
        self.provider.as_deref()
    }

    /// The locale engine itself, for a render.
    ///
    /// **Not `intl()`**, and the name matters: `intl` is an *area* in
    /// every other binding — `sdk.intl.render`, `sdk.intl.entity` and
    /// five more — so a root accessor of that name would be an operation
    /// the other three do not have, and `check-areas` holds the four to
    /// one list. The area is the next increment; these two are what a
    /// consumer uses until it lands, and may become private when it
    /// does.
    #[must_use]
    pub fn locale_engine(&self) -> Ref<'_, Intl> {
        self.intl.borrow()
    }

    /// The locale engine, to change the locale or load a pack. See
    /// [`Context::locale_engine`] for why it is not called `intl_mut`.
    #[must_use]
    pub fn locale_engine_mut(&self) -> RefMut<'_, Intl> {
        self.intl.borrow_mut()
    }

    /// Dates in the calendars the SDK carries, and the arithmetic over
    /// them.
    ///
    /// An **area**: a value you can hold, borrowing this context and
    /// unable to outlive it, which is what `sdk.calendar` is in the
    /// other three bindings (`03-design/surface-areas.md`).
    #[must_use]
    pub fn calendar(&self) -> CalendarArea<'_> {
        CalendarArea::of(self)
    }

    /// Civil times to instants and back, the time scales, and ΔT.
    #[must_use]
    pub fn time(&self) -> TimeArea<'_> {
        TimeArea::of(self)
    }

    /// **Positions**: a grid of instants by bodies, in the frame asked
    /// for, with every step the SDK applied to get there.
    ///
    /// A root operation and not an area's, because an operation whose
    /// name is its own area's name is one — nobody writes
    /// `sdk.positions.positions` (`03-design/surface-areas.md`).
    ///
    /// The answer is the astronomy crate's own `Completed`: a Rust
    /// consumer reads `JulianDay` and `Longitude` off it, where every
    /// other binding decodes a result blob to get the same numbers back
    /// as doubles.
    ///
    /// # Errors
    ///
    /// A context with no ephemeris, a frame the provider refuses and the
    /// SDK cannot complete, an instant outside the provider's coverage,
    /// or the provider's own refusal, which crosses back as itself.
    pub fn positions(&self, request: &PositionRequest<'_>) -> Result<Completed, Error> {
        let provider = self.provider.as_deref().ok_or_else(no_ephemeris)?;
        let completion =
            Completion::new(provider, self.settings().provider.overrides, self.delta_t);
        completion.positions(request).map_err(Error::from)
    }

    /// The locale engine: a message, an entity's forms,
    /// transliteration, and a pack loaded at run time.
    #[must_use]
    pub fn intl(&self) -> IntlArea<'_> {
        IntlArea::of(self)
    }

    /// The operations your **ephemeris** brings with it, beyond the ones
    /// the SDK names.
    #[must_use]
    pub fn engine(&self) -> EngineArea<'_> {
        EngineArea::of(self)
    }

    /// A catalogue key to its packed id and back.
    #[must_use]
    pub fn keys(&self) -> KeysArea<'_> {
        KeysArea::of(self)
    }

    /// The canonical frame, and packing one to carry it.
    #[must_use]
    pub fn frame(&self) -> FrameArea<'_> {
        FrameArea::of(self)
    }

    /// The ΔT model the settings chose, which `time()` reads.
    pub(crate) fn delta_t(&self) -> DeltaTModel {
        self.delta_t
    }
}

/// A context under construction.
///
/// A builder rather than an options struct for two reasons the project
/// already holds: a constructor that cannot be half-built (ADR-0023),
/// and a `build` that can answer with **what it resolved** rather than
/// leaving a consumer to ask — the "no dead ends" brief's *what was
/// applied is reported*.
#[derive(Default)]
pub struct ContextBuilder {
    profile: Option<String>,
    settings_json: Option<String>,
    locale: Option<String>,
    chain: Option<Vec<Ephemeris>>,
}

impl core::fmt::Debug for ContextBuilder {
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        out.debug_struct("ContextBuilder")
            .field("profile", &self.profile)
            .field("locale", &self.locale)
            .field("ephemeris", &self.chain)
            .finish_non_exhaustive()
    }
}

impl ContextBuilder {
    /// The shipped profile to resolve the settings from.
    #[must_use]
    pub fn profile(mut self, id: impl Into<String>) -> ContextBuilder {
        self.profile = Some(id.into());
        self
    }

    /// A JSON settings patch over the profile: an object whose groups
    /// and knobs are the settings document's, every one optional.
    #[must_use]
    pub fn settings_json(mut self, json: impl Into<String>) -> ContextBuilder {
        self.settings_json = Some(json.into());
        self
    }

    /// The locale every render resolves from.
    #[must_use]
    pub fn locale(mut self, tag: impl Into<String>) -> ContextBuilder {
        self.locale = Some(tag.into());
        self
    }

    /// Which ephemeris to compute with, or an **ordered chain** of them
    /// tried in order (ADR-0029).
    ///
    /// Nothing is silent: a context asked for an engine and quietly
    /// given a fallback is what the chain refuses, and when no entry
    /// opens the refusal names each one that failed.
    #[must_use]
    pub fn ephemeris(mut self, chain: impl IntoIterator<Item = Ephemeris>) -> ContextBuilder {
        self.chain = Some(chain.into_iter().collect());
        self
    }

    /// The context, or the refusal that says why there is none.
    ///
    /// # Errors
    ///
    /// An unknown profile, a patch that does not parse or contradicts
    /// the profile, an unknown locale, or an ephemeris chain where
    /// nothing opened.
    pub fn build(self) -> Result<Context, Error> {
        let id = self.profile.as_deref().unwrap_or(DEFAULT_PROFILE);
        let profile = Profile::shipped(id).ok_or_else(|| {
            Error::unsupported(format!("no shipped profile `{id}`"))
                .with_field("profile")
                .with_hint(format!(
                    "the shipped profiles are {}",
                    SHIPPED_PROFILES.join(", ")
                ))
        })?;
        let patch: SettingsPatch = match self.settings_json.as_deref() {
            Some(json) => serde_json::from_str(json).map_err(|e| {
                Error::invalid_arg(format!("the settings patch does not parse: {e}"))
                    .with_field("settings_json")
            })?,
            None => SettingsPatch::default(),
        };
        let settings = profile.resolve(&patch)?;

        let bundles: Vec<&[u8]> = BUNDLES.iter().map(|(_, bytes)| *bytes).collect();
        let locales = locales_from_packs(&bundles).map_err(|e| {
            Error::new(
                Status::Pack,
                format!("the embedded bundles do not load: {e}"),
            )
        })?;
        let mut intl = Intl::new(locales)
            .map_err(|e| Error::internal(format!("the locale engine did not start: {e}")))?;
        if let Some(tag) = &self.locale {
            intl.set_locale(tag).map_err(|e| {
                // The same refusal the boundary gives, because a
                // consumer who mistyped a tag wants the same sentence in
                // whichever language they mistyped it in.
                let known: Vec<&str> = intl.locales().map(|l| l.tag.as_str()).collect();
                Error::unsupported(e.to_string())
                    .with_field("locale")
                    .with_hint(format!(
                        "`{tag}` is not loaded; the locales are {}",
                        known.join(", ")
                    ))
            })?;
        }

        let delta_t = DeltaTModel::from_knob(settings.settings.time.delta_t).unwrap_or_default();
        // No chain named is no ephemeris, and that is a decision rather
        // than a default: ADR-0029 refuses a context quietly given one.
        let chain = self.chain.unwrap_or_else(|| vec![Ephemeris::None]);
        let opened = ephemeris::open(chain)?;
        let provider = remembering(opened, settings.settings.provider.cache_cells);
        Ok(Context {
            settings,
            provider,
            intl: RefCell::new(intl),
            delta_t,
        })
    }
}

/// The provider behind its cache, when the settings ask for one.
///
/// `provider.cache_cells` is the knob, and zero turns the cache off —
/// the same reading the C boundary makes, because the knob is the
/// settings' and not a boundary's.
fn remembering(
    provider: Option<Box<dyn EphemerisProvider>>,
    cells: u32,
) -> Option<Box<dyn EphemerisProvider>> {
    let inner = provider?;
    if cells == 0 {
        return Some(inner);
    }
    Some(Box::new(CachingProvider::with_capacity(
        inner,
        cells as usize,
    )))
}
