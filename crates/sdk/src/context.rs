//! The context: the settings a chart is computed under, the locale it is
//! described in, and the ephemeris it is computed with.

use core::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use serde::Serialize;
use teistro_astro::DeltaTModel;
use teistro_astro::completion::{Completed, Completion};
use teistro_core::envelope::{Envelope, Hash, PackStamp, Provenance, Version, content_hash};
use teistro_core::error::{Error, Status};
use teistro_core::settings::{
    DEFAULT_PROFILE, Diagnostic, Profile, Resolved, SHIPPED_PROFILES, Settings, SettingsPatch,
    Severity, Siddhanta,
};
#[cfg(feature = "chart")]
use teistro_dasha::{DashaDefinition, DashaSystems};
use teistro_geometry::{Layout, Layouts};
use teistro_intl::Intl;
use teistro_intl::pack::locales_from_packs;
use teistro_intl::source::LocaleSource;
#[cfg(feature = "chart")]
use teistro_matching::{BookId, NameBook};
use teistro_port_ephemeris::{
    Astronomy, CachingProvider, EphemerisProvider, Frame, PositionRequest,
};
use teistro_time::{EmbeddedTzdb, TimeZoneProvider, ZonePack};

use crate::area::{ZoneDatabase, ZoneDatabaseSource};

use crate::BUNDLES;
#[cfg(feature = "numerology")]
use crate::area::NumerologyArea;
#[cfg(feature = "research")]
use crate::area::ResearchArea;
use crate::area::{
    AlmanacArea, CalendarArea, EngineArea, FrameArea, IntlArea, KeysArea, TimeArea, VerifyArea,
};
#[cfg(feature = "chart")]
use crate::area::{ChartArea, InterpretArea, MatchingArea, NamingArea};
use crate::ephemeris::{self, Ephemeris, no_ephemeris};

/// One context: built once, read many times.
///
/// It carries what every answer is computed against — the resolved
/// settings, the locale engine, the ΔT model and the ephemeris — so that
/// two contexts with the same settings hash compute the same numbers and
/// the hash is a cache key.
///
/// **A context moves between threads and is used by one at a time**: it
/// is `Send` and not `Sync`, which is the rule every other binding states
/// (*one context serves one thread*) with the move it allows made safe. A
/// thread that wants its own takes a [`Context::fork`] — the same settings,
/// ephemeris, cache and registrations, at the cost of copying the locale
/// state — and a server keeps a [`SharedContext`] (`Send + Sync`, cheap to
/// clone) and asks it for a context per request or per worker
/// (`03-design/rust-consumer-surface.md`, "A context across threads").
///
/// Not `Sync` on purpose. What a context changes after it is built — the
/// locale, a runtime pack, a zone pack, a name book — sits in `RefCell`s,
/// and a lock in their place would be paid on every render of every
/// single-threaded caller. What is fixed when it is built is shared by
/// every fork behind an `Arc`, and reading it costs no atomic operation.
///
/// ```
/// # #[cfg(feature = "builtin-ephemeris")] {
/// use teistro::{Context, Ephemeris};
///
/// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
/// let shared = sdk.share();
/// let hashes: Vec<String> = (0..2)
///     .map(|_| {
///         let shared = shared.clone();
///         std::thread::spawn(move || shared.context().settings_hash().to_string())
///     })
///     .map(|worker| worker.join().unwrap_or_default())
///     .collect();
/// assert_eq!(hashes, [sdk.settings_hash().to_string(), sdk.settings_hash().to_string()]);
/// # }
/// # Ok::<(), teistro::Error>(())
/// ```
pub struct Context {
    /// What is fixed when the context is built, shared by its forks.
    core: Arc<Core>,
    intl: RefCell<Intl>,
    /// The zone database every resolution reads: the embedded one, until a
    /// signed pack replaces it (`03-design/zone-packs.md`).
    zones: RefCell<Zones>,
    /// The name books loaded into this context, by their identity: the
    /// one thing a context gains after it is built, and only by
    /// [`NamingArea::load`], which adds and never replaces (the same book
    /// is the same id).
    #[cfg(feature = "chart")]
    books: RefCell<Books>,
}

/// The name books a context holds, by their identity.
#[cfg(feature = "chart")]
type Books = std::collections::HashMap<BookId, Arc<NameBook>>;

/// What a context fixes when it is built, and every fork of it shares:
/// read through an `Arc`, which costs a pointer and no atomic operation.
struct Core {
    settings: Resolved,
    /// Behind the context's cache, so its forks share what it remembers.
    provider: Option<Box<dyn EphemerisProvider>>,
    /// Resolved once from `time.delta_t`, as the boundary resolves it,
    /// so the two agree on which model a context uses.
    delta_t: DeltaTModel,
    /// The layouts a chart can be drawn in: the shipped ones and any the
    /// builder was given, sealed, so nothing changes under a context that
    /// draws.
    layouts: Layouts,
    /// The consumer's nakshatra-seeded dasha systems, sealed as the layouts
    /// are.
    #[cfg(feature = "chart")]
    dashas: DashaSystems,
}

/// What a context may change after it is built, as one value a fork
/// copies.
#[derive(Clone)]
struct State {
    intl: Intl,
    zones: Zones,
    #[cfg(feature = "chart")]
    books: Books,
}

/// A context to share between threads: `Send + Sync`, and a clone is two
/// reference counts.
///
/// What [`Context::share`] answers. It holds the context's fixed core and
/// its locale state **as it was when shared**, and
/// [`SharedContext::context`] makes a context of them for the thread that
/// asks: the same settings, ephemeris and cache, and its own locale state
/// from there on. A locale set or a pack loaded on that context is its own
/// and reaches no other.
///
/// ```
/// # #[cfg(feature = "builtin-ephemeris")] {
/// use teistro::{Context, Ephemeris};
///
/// let sdk = Context::builder().locale("ne-Deva-NP").ephemeris([Ephemeris::Builtin]).build()?;
/// let shared = sdk.share();
/// let english = shared.context();
/// english.intl().set_locale("en-Latn")?;
/// // The change is that context's own.
/// assert_eq!(shared.context().intl().locale(), "ne-Deva-NP");
/// assert_eq!(english.intl().locale(), "en-Latn");
/// # }
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone)]
pub struct SharedContext {
    core: Arc<Core>,
    state: Arc<State>,
}

impl SharedContext {
    /// A context for this thread: the shared core, and a copy of the
    /// locale state as it was shared.
    #[must_use]
    pub fn context(&self) -> Context {
        Context::of(Arc::clone(&self.core), State::clone(&self.state))
    }
}

impl core::fmt::Debug for SharedContext {
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        out.debug_struct("SharedContext")
            .field("profile", &self.core.settings.profile.as_str())
            .field("locale", &self.state.intl.locale())
            .finish_non_exhaustive()
    }
}

// Held at compile time, so a field that would take either away fails the
// build here rather than in a consumer's: a context moves between threads,
// and a shared one is shared between them.
const _: () = {
    const fn moves<T: Send>() {}
    const fn shares<T: Send + Sync>() {}
    moves::<Context>();
    shares::<SharedContext>();
};

/// The zone database a context reads, and which it is.
#[derive(Clone)]
struct Zones {
    provider: Arc<dyn TimeZoneProvider>,
    source: ZoneDatabaseSource,
}

impl Zones {
    fn of(pack: Option<Arc<ZonePack>>) -> Zones {
        match pack {
            Some(pack) => Zones {
                provider: pack,
                source: ZoneDatabaseSource::Pack,
            },
            None => Zones {
                provider: Arc::new(EmbeddedTzdb::shared()),
                source: ZoneDatabaseSource::Embedded,
            },
        }
    }
}

impl core::fmt::Debug for Context {
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        out.debug_struct("Context")
            .field("profile", &self.profile())
            .field("settings_hash", &self.settings_hash().to_string())
            .field(
                "ephemeris",
                &self
                    .core
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

    /// A context over a core and a locale state.
    fn of(core: Arc<Core>, state: State) -> Context {
        Context {
            core,
            intl: RefCell::new(state.intl),
            zones: RefCell::new(state.zones),
            #[cfg(feature = "chart")]
            books: RefCell::new(state.books),
        }
    }

    /// This context's locale state as it stands, copied.
    fn state(&self) -> State {
        State {
            intl: self.intl.borrow().clone(),
            zones: self.zones.borrow().clone(),
            #[cfg(feature = "chart")]
            books: self.books.borrow().clone(),
        }
    }

    /// **Another context over this one's core**, for another thread or a
    /// second locale: the same settings, ephemeris, cache, layouts and
    /// dasha systems, shared and not copied, and a copy of the locale
    /// state as it stands — the locale, the packs, the zone database and
    /// the name books. What either changes from here on is its own.
    ///
    /// Far cheaper than building: nothing is resolved, opened or checked
    /// again, and the ephemeris's cache is the one both read.
    ///
    /// ```
    /// # #[cfg(feature = "builtin-ephemeris")] {
    /// use teistro::{Context, Ephemeris};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let worker = sdk.fork();
    /// let hash = std::thread::spawn(move || worker.settings_hash()).join().map_err(|_| "a worker")?;
    /// assert_eq!(hash, sdk.settings_hash());
    /// # }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn fork(&self) -> Context {
        Context::of(Arc::clone(&self.core), self.state())
    }

    /// This context **to share between threads**: a [`SharedContext`],
    /// which a server keeps in its state and asks for a context per
    /// request or per worker. It holds the locale state as it stands now.
    #[must_use]
    pub fn share(&self) -> SharedContext {
        SharedContext {
            core: Arc::clone(&self.core),
            state: Arc::new(self.state()),
        }
    }

    /// The resolved settings, every knob as this context will use it.
    #[must_use]
    pub fn settings(&self) -> &Settings {
        &self.core.settings.settings
    }

    /// The profile the settings were resolved from.
    #[must_use]
    pub fn profile(&self) -> &str {
        self.core.settings.profile.as_str()
    }

    /// The canonical settings document: the JSON the hash is taken of,
    /// and what a consumer stores beside a chart so it can be recomputed.
    ///
    /// Canonical, so two contexts that compute the same numbers write
    /// the same bytes — which is what makes the hash a cache key.
    #[must_use]
    pub fn settings_json(&self) -> String {
        self.core.settings.settings.canonical_json()
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
        self.core.settings.settings.hash()
    }

    /// The zone database this context reads: the embedded one, or the
    /// pack that replaced it.
    #[must_use]
    pub fn zones(&self) -> Arc<dyn TimeZoneProvider> {
        Arc::clone(&self.zones.borrow().provider)
    }

    /// Which database this context reads, for `sdk.time`.
    pub(crate) fn zone_database(&self) -> ZoneDatabase {
        let zones = self.zones.borrow();
        ZoneDatabase {
            source: zones.source,
            version: zones.provider.version().to_string(),
            zones: zones.provider.zones().len(),
            bundled: EmbeddedTzdb::bundled_version().to_string(),
        }
    }

    /// Replaces the zone database, for every answer from here on: a
    /// message's `timeZone` reads the one a resolution reads.
    #[cfg(feature = "signed-packs")]
    pub(crate) fn install_zones(&self, pack: ZonePack) {
        let zones = Zones::of(Some(Arc::new(pack)));
        self.intl
            .borrow_mut()
            .set_time_zones(Arc::clone(&zones.provider));
        *self.zones.borrow_mut() = zones;
    }

    /// The ephemeris this context computes positions with, when it has
    /// one.
    #[must_use]
    pub fn ephemeris(&self) -> Option<&dyn EphemerisProvider> {
        self.core.provider.as_deref()
    }

    /// The resolved settings **whole**: the knobs, the warnings the
    /// coherence rules raised, and the profile they came from.
    ///
    /// [`Context::settings`] is the knobs alone, which is what most
    /// callers want; this is what the chart founder and a consumer
    /// auditing a resolution want.
    #[must_use]
    pub fn resolved(&self) -> &Resolved {
        &self.core.settings
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
    /// as doubles. It comes in the same envelope as a chart's, stamped
    /// with what computed it, so a stored grid says which provider and
    /// which data answered.
    ///
    /// An instant outside the provider's coverage is not an error: its
    /// cells are `OutOfRange` and the rest of the grid is answered, and
    /// the provider is never asked for it.
    ///
    /// # Errors
    ///
    /// A context with no ephemeris, a frame the provider refuses and the
    /// SDK cannot complete, or the provider's own refusal, which crosses
    /// back as itself.
    pub fn positions(&self, request: &PositionRequest<'_>) -> Result<Envelope<Completed>, Error> {
        let completed = self.completion()?.positions(request).map_err(Error::from)?;
        let provenance = self.positions_provenance(request, &completed);
        Ok(Envelope::new(completed, provenance))
    }

    /// The completion a positions answer is made through: this context's
    /// ephemeris, override policy and Delta T model, with the speeds it
    /// reports derived from the places it reports
    /// ([`Completion::deriving_speeds`]). One place, so the façade and the
    /// C boundary's `ts_positions` cannot complete a grid two ways.
    ///
    /// # Errors
    ///
    /// A context with no ephemeris.
    pub fn completion(&self) -> Result<Completion<'_, dyn EphemerisProvider>, Error> {
        let provider = self.core.provider.as_deref().ok_or_else(no_ephemeris)?;
        Ok(Completion::new(
            provider,
            self.settings().provider.overrides,
            self.core.delta_t,
        )
        .deriving_speeds())
    }

    /// The provenance [`Context::positions`] stamps its answer with: this
    /// context's settings and profile, the request's hash, the provider's
    /// identity with the frame it answered in and the steps applied, the
    /// time scales' tables, and the hash of the columns.
    ///
    /// Public for a caller that completes a request itself and wants the
    /// answer stamped as this context would stamp it — which is what the C
    /// boundary does, since it reads a provider's own error code on the
    /// way that this façade's `Error` does not carry. A context with no
    /// ephemeris stamps the provider as unnamed.
    #[must_use]
    pub fn positions_provenance(
        &self,
        request: &PositionRequest<'_>,
        completed: &Completed,
    ) -> Provenance {
        let mut provenance = self.stamped(
            content_hash(&RequestRecord::of(request)),
            completed.columns.frame,
            completed.step_keys(),
        );
        provenance.content_hash = content_hash(&completed.columns);
        provenance
    }

    /// The provenance an answer of this context is stamped with before
    /// its value is sealed: the settings and profile, the input's hash,
    /// the provider's identity with the frame it answered in and the steps
    /// applied, and the time scales' tables. One place, so that every
    /// answer names its sources the same way.
    pub(crate) fn stamped(&self, input_hash: Hash, frame: Frame, steps: Vec<String>) -> Provenance {
        let mut provenance = self.core.settings.provenance(
            Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Version::new(0, 0, 0)),
            input_hash,
        );
        if let Some(provider) = self.core.provider.as_deref() {
            provenance.provider = provider.capabilities().identity.stamp(frame, steps);
        }
        provenance.time.delta_t_model = self.core.delta_t.key().to_string();
        provenance.time.leap_table = teistro_time::leap::version().to_string();
        provenance.time.tzdb_version = self.zones.borrow().provider.version().to_string();
        provenance.packs = self.pack_stamps(provenance.catalogue_version);
        provenance
    }

    /// Each pack loaded at run time, as an answer's provenance names it
    /// (ADR-0020): what shaped the words an answer may say. The embedded
    /// locales are named by the SDK's own version and are not listed. A
    /// pack built for another catalogue is refused at load, so the
    /// catalogue version the context reads under is the pack's.
    pub(crate) fn pack_stamps(&self, catalogue_version: u32) -> Vec<PackStamp> {
        let version = catalogue_version.to_string();
        self.locale_engine()
            .loaded()
            .iter()
            .filter_map(|loaded| {
                // The loader writes the digest it computed as hex, so one
                // that does not read back is a defect, not a pack.
                let hash = Hash::from_hex(&loaded.sha256);
                debug_assert!(hash.is_some(), "a pack digest that is not hex");
                Some(PackStamp {
                    id: format!("{}/{}", loaded.locale, loaded.namespaces.join(",")),
                    version: version.clone(),
                    hash: hash?,
                })
            })
            .collect()
    }

    /// The locale engine: a message, an entity's forms,
    /// transliteration, and a pack loaded at run time.
    #[must_use]
    pub fn intl(&self) -> IntlArea<'_> {
        IntlArea::of(self)
    }

    /// A chart founded from a birth record, and a batch of them founded
    /// in one crossing.
    #[cfg(feature = "chart")]
    #[must_use]
    pub fn chart(&self) -> ChartArea<'_> {
        ChartArea::of(self)
    }

    /// What a reading says, as a narrative plan of message keys and
    /// slots, which [`Context::intl`] renders in any locale.
    #[cfg(feature = "chart")]
    #[must_use]
    pub fn interpret(&self) -> InterpretArea<'_> {
        InterpretArea::of(self)
    }

    /// A panchanga: the five limbs of a day and its periods, for a day
    /// or a run of them.
    #[must_use]
    pub fn almanac(&self) -> AlmanacArea<'_> {
        AlmanacArea::of_context(self)
    }

    /// What a text claims of the time, each claim held, contradicted or
    /// unread, against what the almanac computes at an instant and a
    /// place.
    #[must_use]
    pub fn verify(&self) -> VerifyArea<'_> {
        VerifyArea::of(self)
    }

    /// The naming of a child (nāmakaraṇa): the syllables a pada names a
    /// child by, a name read and checked, the names a birth gives, and a
    /// consumer's name book.
    #[cfg(feature = "chart")]
    #[must_use]
    pub fn naming(&self) -> NamingArea<'_> {
        NamingArea::new(self)
    }

    /// The name book loaded under `id`, when one is.
    #[cfg(feature = "chart")]
    pub(crate) fn book(&self, id: BookId) -> Option<Arc<NameBook>> {
        self.books.borrow().get(&id).cloned()
    }

    /// Keeps a loaded book under its id; the same book loaded again is
    /// the one already kept.
    #[cfg(feature = "chart")]
    pub(crate) fn keep_book(&self, book: NameBook) -> Arc<NameBook> {
        Arc::clone(
            self.books
                .borrow_mut()
                .entry(book.id())
                .or_insert_with(|| Arc::new(book)),
        )
    }

    /// What matches without a chart: two names, star to star.
    #[cfg(feature = "chart")]
    #[must_use]
    pub fn matching(&self) -> MatchingArea<'_> {
        MatchingArea::of(self)
    }

    /// What a name and a birth date say under numerology's two systems.
    #[cfg(feature = "numerology")]
    #[must_use]
    pub fn numerology(&self) -> NumerologyArea<'_> {
        NumerologyArea::of(self)
    }

    /// Counts and permutation tests over a batch of births: how often each
    /// rule of a set holds in each group, whether groups differ, whether
    /// rules are delivered at life events more than chance, and whether a
    /// sample departs from its own recombined population.
    #[cfg(feature = "research")]
    #[must_use]
    pub fn research(&self) -> ResearchArea<'_> {
        ResearchArea::of(self)
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

    /// The ΔT model the settings chose.
    ///
    /// Declared rather than hidden: a choice the SDK makes on a
    /// consumer's behalf is a knob or is at least answerable, which is
    /// the "no dead ends" brief's rule.
    #[must_use]
    pub fn delta_t(&self) -> DeltaTModel {
        self.core.delta_t
    }

    /// The layouts this context can draw a chart in: the shipped ones, then
    /// any the builder was given.
    #[must_use]
    pub fn layouts(&self) -> &Layouts {
        &self.core.layouts
    }

    /// The dasha systems a consumer registered with this context, beside the
    /// ones the catalogue has.
    #[cfg(feature = "chart")]
    #[must_use]
    pub fn dashas(&self) -> &DashaSystems {
        &self.core.dashas
    }
}

/// The profile [`ContextBuilder::nepal_official`] resolves: the Nepali
/// committee's.
pub const NEPAL_OFFICIAL_PROFILE: &str = "nepali-committee";

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
    wrap: Option<Wrap>,
    layouts: Vec<Layout>,
    zones: Option<Arc<ZonePack>>,
    #[cfg(feature = "chart")]
    dashas: Vec<DashaDefinition>,
}

/// What [`ContextBuilder::wrapping`] lays over the opened provider.
type Wrap = Box<dyn FnOnce(Box<dyn EphemerisProvider>) -> Box<dyn EphemerisProvider>>;

impl core::fmt::Debug for ContextBuilder {
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        out.debug_struct("ContextBuilder")
            .field("profile", &self.profile)
            .field("locale", &self.locale)
            .field("ephemeris", &self.chain)
            .field("wrapped", &self.wrap.is_some())
            .finish_non_exhaustive()
    }
}

impl ContextBuilder {
    /// A chart layout of the consumer's own, a regional chart the SDK does
    /// not ship, to draw in beside the shipped ones (ADR-0026 §1).
    ///
    /// Checked when the context is built, by the same rules a shipped layout
    /// passes; a key the SDK ships is refused, so a layout is added and never
    /// replaced.
    #[must_use]
    pub fn layout(mut self, layout: Layout) -> ContextBuilder {
        self.layouts.push(layout);
        self
    }

    /// A nakshatra-seeded dasha system of the consumer's own, asked for by
    /// the id [`keys`](crate::Context::keys) gives its key
    /// (`dasha_system.ACME_SAPTAKA`) like any catalogued system.
    ///
    /// Checked when the context is built, by the rules a shipped row passes,
    /// and refused by its place and field (`dashas[0].span`); a key the
    /// catalogue has is refused, so a system is added and never replaced
    /// (`03-design/dasha-kernels.md`, "A consumer's own system").
    #[cfg(feature = "chart")]
    #[must_use]
    pub fn dasha_system(mut self, definition: impl Into<DashaDefinition>) -> ContextBuilder {
        self.dashas.push(definition.into());
        self
    }

    /// A zone pack this context reads in place of the embedded database
    /// (`03-design/zone-packs.md`). Shared, so a worker's context takes the
    /// pack one thread verified and read.
    ///
    /// ```
    /// use std::sync::Arc;
    /// use teistro::{Context, ZonePack};
    ///
    /// let utc = jiff_tzdb::get("UTC").ok_or("a zone")?.1;
    /// let pack = Arc::new(ZonePack::from_bytes(&ZonePack::write("2026f", [("UTC", utc)]))?);
    /// let sdk = Context::builder().zones(pack).build()?;
    /// assert_eq!(sdk.time().zone_database().version, "2026f");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn zones(mut self, pack: Arc<ZonePack>) -> ContextBuilder {
        self.zones = Some(pack);
        self
    }

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

    /// **Nepal's official days**: the Nepali committee's reckoning, the
    /// setup the committee's own panchanga is computed by. The profile is
    /// [`NEPAL_OFFICIAL_PROFILE`] and the chain the Surya Siddhanta, then
    /// `modern`: the text reckons the tithi, nakshatra and yoga with the
    /// committee's bija, and the first of `modern` that opens gives the
    /// day's sunrise and the outer planets. With none, the sunrise is the
    /// built-in ephemeris's.
    ///
    /// ```
    /// # #[cfg(feature = "builtin-ephemeris")] {
    /// use teistro::{Context, NEPAL_OFFICIAL_PROFILE};
    ///
    /// // An app passes its engine, `Ephemeris::Provider(teimeris)`.
    /// let official = Context::builder().nepal_official([]).build()?;
    /// assert_eq!(official.profile(), NEPAL_OFFICIAL_PROFILE);
    /// # }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn nepal_official(self, modern: impl IntoIterator<Item = Ephemeris>) -> ContextBuilder {
        self.profile(NEPAL_OFFICIAL_PROFILE)
            .ephemeris(core::iter::once(Ephemeris::SuryaSiddhanta).chain(modern))
    }

    /// A provider of the caller's own laid over the one the chain opens,
    /// behind the context's cache: one that counts, logs or stops what an
    /// engine is asked. Whichever entry opened, under whatever the
    /// settings name, `wrap` receives it, so a wrapper never has to open
    /// an engine itself; it must forward every method it does not change.
    ///
    /// ```
    /// # #[cfg(feature = "builtin-ephemeris")] {
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicBool, Ordering};
    /// use teistro::{Context, Ephemeris};
    /// use teistro_port_ephemeris::{CountingProvider, EphemerisProvider};
    ///
    /// let wrapped = Arc::new(AtomicBool::new(false));
    /// let seen = Arc::clone(&wrapped);
    /// let context = Context::builder()
    ///     .ephemeris([Ephemeris::Builtin])
    ///     .wrapping(move |opened| -> Box<dyn EphemerisProvider> {
    ///         seen.store(true, Ordering::Relaxed);
    ///         Box::new(CountingProvider::new(opened))
    ///     })
    ///     .build()?;
    /// assert!(wrapped.load(Ordering::Relaxed) && context.ephemeris().is_some());
    /// # }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn wrapping(
        mut self,
        wrap: impl FnOnce(Box<dyn EphemerisProvider>) -> Box<dyn EphemerisProvider> + 'static,
    ) -> ContextBuilder {
        self.wrap = Some(Box::new(wrap));
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
            // Named down to the knob, `settings.houses.chalit_system`, and
            // not only the patch — `settings` being what every binding
            // calls it; the patch refuses an unknown key itself.
            Some(json) => teistro_core::strict::deserialize_str(json, "settings")?,
            None => SettingsPatch::default(),
        };
        let settings = profile.resolve(&patch)?;

        let mut intl = Intl::shared(embedded_locales()?)
            .map_err(|e| Error::internal(format!("the locale engine did not start: {e}")))?;
        // A message's `timeZone` reads an instant in a zone through the
        // same database the context's own zone resolutions use.
        let zones = Zones::of(self.zones);
        intl.set_time_zones(Arc::clone(&zones.provider));
        if let Some(tag) = &self.locale {
            intl.set_locale(tag)
                .map(drop)
                .map_err(|e| crate::area::intl::refused_locale(&intl, tag, &e.to_string()))?;
        }

        let delta_t = DeltaTModel::from_knob(settings.settings.time.delta_t).unwrap_or_default();
        // No chain named is no ephemeris, and that is a decision rather
        // than a default: ADR-0029 refuses a context quietly given one.
        let chain = self.chain.unwrap_or_else(|| vec![Ephemeris::None]);
        let opened = ephemeris::open(chain, settings.settings.frame.siddhanta, delta_t)?;
        let mut settings = settings;
        astronomy_coherence(&mut settings, opened.as_deref())?;
        let opened = match (opened, self.wrap) {
            (Some(opened), Some(wrap)) => Some(wrap(opened)),
            (opened, _) => opened,
        };
        let provider = remembering(opened, settings.settings.provider.cache_cells);
        let mut layouts = Layouts::new();
        for (index, layout) in self.layouts.into_iter().enumerate() {
            // Named by its place, so a caller who gave several knows which.
            layouts
                .register(layout)
                .map_err(|error| error.under(&format!("layouts[{index}]")))?;
        }
        layouts.seal();
        #[cfg(feature = "chart")]
        let dashas = {
            let mut dashas = DashaSystems::new();
            for (index, definition) in self.dashas.into_iter().enumerate() {
                dashas
                    .register(definition)
                    .map_err(|error| error.under(&format!("dashas[{index}]")))?;
            }
            dashas.seal();
            dashas
        };
        let core = Core {
            settings,
            provider,
            delta_t,
            layouts,
            #[cfg(feature = "chart")]
            dashas,
        };
        Ok(Context::of(
            Arc::new(core),
            State {
                intl,
                zones,
                #[cfg(feature = "chart")]
                books: Books::new(),
            },
        ))
    }
}

/// The embedded bundles, decoded once for every context in the process.
/// Decoding them is about 22,000 allocations and 2 MB, which each context
/// paid before; every context's engine now shares these, and copies only
/// a locale its own runtime packs change.
fn embedded_locales() -> Result<BTreeMap<String, Arc<LocaleSource>>, Error> {
    type Decoded = Result<BTreeMap<String, Arc<LocaleSource>>, String>;
    static DECODED: OnceLock<Decoded> = OnceLock::new();
    DECODED
        .get_or_init(|| {
            let bundles: Vec<&[u8]> = BUNDLES.iter().map(|(_, bytes)| *bytes).collect();
            locales_from_packs(&bundles)
                .map(|locales| {
                    locales
                        .into_iter()
                        .map(|(tag, locale)| (tag, Arc::new(locale)))
                        .collect()
                })
                .map_err(|e| e.to_string())
        })
        .clone()
        .map_err(|e| {
            Error::new(
                Status::Pack,
                format!("the embedded bundles do not load: {e}"),
            )
        })
}

/// Holds the settings' astronomy (`frame.siddhanta`) to the provider the
/// chain opened.
///
/// **The settings ask; the chain supplies** (ADR-0029: a provider is
/// always declared, never chosen quietly). Settings that name the Surya
/// Siddhanta ask for the text's chart, which only a classical provider
/// defines (`03-design/classical-chart.md`), so a modern engine under them
/// is refused rather than answering a hybrid. The other way round is a
/// declared choice and not a contradiction — the text's provider under
/// settings that say modern astronomy still gives the text's grahas, day
/// and Lagna — but its zodiac is then the catalogue's unless the text's
/// is named, so it is a warning every result carries. No ephemeris at
/// all is neither: calendars and times need none.
fn astronomy_coherence(
    settings: &mut Resolved,
    provider: Option<&dyn EphemerisProvider>,
) -> Result<(), Error> {
    let Some(provider) = provider else {
        return Ok(());
    };
    let capabilities = provider.capabilities();
    let classical = capabilities.astronomy == Astronomy::Classical;
    match (settings.settings.frame.siddhanta, classical) {
        (Siddhanta::Surya { .. }, false) => Err(Error::unsupported(format!(
            "the settings name the Surya Siddhanta's astronomy and the ephemeris, {}, is a \
             modern one, so the chart would be neither",
            capabilities.identity.name
        ))
        .with_field("settings.frame.siddhanta")
        .with_hint(
            "open the context over 'SURYA_SIDDHANTA', or set frame.siddhanta to DRIK for \
             modern astronomy",
        )),
        (Siddhanta::Drik, true) => {
            settings.warnings.push(Diagnostic {
                severity: Severity::Warning,
                rule: "classical-provider-drik-settings",
                message: format!(
                    "{} is a classical astronomy and frame.siddhanta says DRIK; the parts \
                     it defines are its own, and the rest follow the settings",
                    capabilities.identity.name
                ),
                fields: vec!["frame.siddhanta"],
            });
            Ok(())
        }
        _ => Ok(()),
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

/// A positions request as its input hash sees it: every field that
/// changes the answer, named rather than numbered, so the hash is the
/// same whichever binding asked.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct RequestRecord<'a> {
    scale: &'static str,
    frame: String,
    bodies: Vec<&'static str>,
    jds: &'a [f64],
    observer: Option<[f64; 3]>,
    speeds: bool,
}

impl<'a> RequestRecord<'a> {
    fn of(request: &'a PositionRequest<'_>) -> RequestRecord<'a> {
        RequestRecord {
            scale: request.scale.name(),
            frame: request.frame.key(),
            bodies: request.bodies.iter().map(|b| b.key()).collect(),
            jds: request.jds,
            observer: request
                .observer
                .map(|p| [p.latitude.get(), p.longitude.get(), p.altitude.get()]),
            speeds: request.speeds,
        }
    }
}
