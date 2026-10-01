//! Which ephemeris a context computes with, and the ordered chain that
//! chooses between them.

use teistro_astro::DeltaTModel;
use teistro_core::Status;
use teistro_core::error::Error;
use teistro_core::settings::{Siddhanta, SuryaSunrise};
use teistro_port_ephemeris::{Astronomy, EphemerisProvider, TestProvider};
use teistro_siddhanta::{ModernSunrise, SiddhantaProvider};

/// One entry of the chain: an ephemeris the SDK carries, or one a
/// consumer brings.
///
/// **Values, not paths.** ADR-0029 gives a Rust consumer the adapters as
/// `rlib`s and says they link directly rather than paying for a loader,
/// so an engine arrives here as a provider a consumer constructed, not
/// as a shared library to open. `ts_provider_load` stays a boundary
/// concern for consumers who are not in Rust.
pub enum Ephemeris {
    /// No ephemeris. Calendars, times and messages compute; a position
    /// refuses by capability, naming the option.
    None,
    /// The analytic ephemeris the SDK carries — VSOP87A, ELP2000-82B and
    /// a quadratic bija — which makes a chart compute with nothing else
    /// installed. **The fallback, not the intended path** (ADR-0029): in
    /// most cases a consumer belongs on a real engine.
    #[cfg(feature = "builtin-ephemeris")]
    Builtin,
    /// The Surya Siddhanta: a **classical astronomy**, the text the
    /// classical panchangas of Nepal and much of India compute from, in
    /// Burgess's 1860 translation. Its places, its precession, its sunrise
    /// and its Lagna are the text's own definitions, and a chart founded
    /// over it is the text's chart throughout
    /// (`03-design/classical-chart.md`); name its ayanamsha,
    /// `SURYASIDDHANTA`, for the text's zodiac.
    SuryaSiddhanta,
    /// The analytic test provider, whose positions are **not
    /// astronomy**: one periodic term per body. For a test that needs a
    /// number and not the sky.
    Test,
    /// An ephemeris of your own, or an adapter's: anything implementing
    /// the port, already built.
    Provider(Box<dyn EphemerisProvider>),
    /// One that is opened when the chain reaches it, built by
    /// [`Ephemeris::opening`] — so a later entry can be the fallback for
    /// this one failing.
    Opening(Opening),
}

/// How an entry that may fail is opened, with the name a refusal calls
/// it by.
pub struct Opening {
    name: String,
    open: Box<dyn FnOnce() -> Result<Box<dyn EphemerisProvider>, Error>>,
}

impl core::fmt::Debug for Opening {
    /// The name, which is the whole of what a reader can be told: a
    /// closure has nothing else to show.
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        out.debug_struct("Opening")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl core::fmt::Debug for Ephemeris {
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        out.write_str(&self.name())
    }
}

impl Ephemeris {
    /// The refusal a call that needs an ephemeris gives when the context
    /// has none: `CAPABILITY`, naming the `ephemeris` option and the three
    /// kinds of answer it takes.
    ///
    /// ```
    /// use teistro::{Ephemeris, Status};
    ///
    /// let refusal = Ephemeris::missing();
    /// assert_eq!(refusal.status, Status::Capability);
    /// assert_eq!(refusal.field(), Some("ephemeris"));
    /// ```
    #[must_use]
    pub fn missing() -> Error {
        no_ephemeris()
    }

    /// What this entry is called in a refusal, so a chain that opens
    /// nothing can say which entries it tried: the SDK's own by the key
    /// every binding names them with (`BUILTIN`).
    pub(crate) fn name(&self) -> String {
        match self {
            Ephemeris::None => String::from("NONE"),
            #[cfg(feature = "builtin-ephemeris")]
            Ephemeris::Builtin => String::from("BUILTIN"),
            Ephemeris::SuryaSiddhanta => String::from("SURYA_SIDDHANTA"),
            Ephemeris::Test => String::from("TEST"),
            // The provider's own name, because a chain of two adapters
            // that both refused has to say which was which, and neither
            // of them is called `provider`.
            Ephemeris::Provider(provider) => provider.capabilities().identity.name,
            Ephemeris::Opening(opening) => opening.name.clone(),
        }
    }

    /// An ephemeris opened when the chain reaches it, under a name a
    /// refusal can use.
    ///
    /// **Without this a Rust chain never falls back**, and that was the
    /// building correcting the design. In Node, Dart and Python an entry
    /// is a *description* — a name, or a descriptor holding a path — so
    /// opening it can fail and a later entry is the fallback. In Rust an
    /// entry was an already-built `Box<dyn EphemerisProvider>`, which
    /// cannot fail to open, so every chain of them succeeded on its
    /// first entry and the ordering was decoration. Clippy found it,
    /// reporting that `open` was `Result` and could not fail.
    ///
    /// So an entry a chain can fall back *from* is a recipe:
    ///
    /// ```no_run
    /// # use teistro::{Context, Ephemeris};
    /// # fn teimeris(_: &str) -> Result<Box<dyn teistro_port_ephemeris::EphemerisProvider>, teistro::Error> { unimplemented!() }
    /// # #[cfg(feature = "builtin-ephemeris")] fn run() -> Result<(), teistro::Error> {
    /// let sdk = Context::builder()
    ///     .ephemeris([
    ///         Ephemeris::opening("teimeris", || teimeris("./ephe")),
    ///         Ephemeris::Builtin,
    ///     ])
    ///     .build()?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn opening<F>(name: impl Into<String>, open: F) -> Ephemeris
    where
        F: FnOnce() -> Result<Box<dyn EphemerisProvider>, Error> + 'static,
    {
        Ephemeris::Opening(Opening {
            name: name.into(),
            open: Box::new(open),
        })
    }

    /// The provider this entry opens, or the refusal that says why it
    /// could not; the Surya Siddhanta opens as the text, without a bija.
    ///
    /// A chain opens its entries itself, so a consumer rarely needs
    /// this; what does is a caller holding one entry and wanting the
    /// provider it names -- the C boundary's selector, for one.
    ///
    /// # Errors
    ///
    /// Whatever a recipe's own opening refuses with.
    pub fn open(self) -> Result<Option<Box<dyn EphemerisProvider>>, Error> {
        self.open_under(Siddhanta::Drik)
    }

    /// The provider this entry opens under the settings' astronomy: the
    /// Surya Siddhanta entry opens the text with the bija
    /// `frame.siddhanta` names (**the settings ask; the chain
    /// supplies**), and every other entry is what it is.
    ///
    /// ```
    /// use teistro::Ephemeris;
    /// use teistro::settings::{Siddhanta, SuryaBija};
    ///
    /// let committee = Siddhanta::surya(SuryaBija::NepalCommittee);
    /// let provider = Ephemeris::SuryaSiddhanta.open_under(committee)?.expect("a provider");
    /// assert!(provider.capabilities().identity.data_version.contains("moon_apsis -4"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// A day at a modern sunrise (`SuryaSunrise::Modern`) takes it from the
    /// entries after this one in a context's chain; opened alone, as here,
    /// there are none, so it takes the built-in ephemeris's, and refuses
    /// without that feature.
    ///
    /// # Errors
    ///
    /// Whatever a recipe's own opening refuses with, and a modern sunrise
    /// with no modern ephemeris to give it.
    pub fn open_under(
        self,
        siddhanta: Siddhanta,
    ) -> Result<Option<Box<dyn EphemerisProvider>>, Error> {
        self.open_before(siddhanta, DeltaTModel::default(), Vec::new())
    }

    /// As [`Ephemeris::open_under`], with the entries that follow this one
    /// in the chain, which a modern sunrise beside the text is taken from.
    fn open_before(
        self,
        siddhanta: Siddhanta,
        delta_t: DeltaTModel,
        rest: Vec<Ephemeris>,
    ) -> Result<Option<Box<dyn EphemerisProvider>>, Error> {
        match self {
            Ephemeris::None => Ok(None),
            #[cfg(feature = "builtin-ephemeris")]
            Ephemeris::Builtin => Ok(Some(Box::new(
                teistro_ephemeris_builtin::provider::Builtin::new(),
            ))),
            Ephemeris::SuryaSiddhanta => {
                let Siddhanta::Surya { bija, sunrise } = siddhanta else {
                    return Ok(Some(Box::new(SiddhantaProvider::text())));
                };
                let text = SiddhantaProvider::with_bija(bija.revolutions());
                match sunrise {
                    SuryaSunrise::Text => Ok(Some(Box::new(text))),
                    SuryaSunrise::Modern => Ok(Some(Box::new(
                        ModernSunrise::new(text, modern_sunrise(rest)?).with_delta_t(delta_t),
                    ))),
                    other => Err(Error::unsupported(format!(
                        "the {other} sunrise; this build knows TEXT and MODERN"
                    ))
                    .with_field("settings.frame.siddhanta.sunrise")),
                }
            }
            Ephemeris::Test => Ok(Some(Box::new(TestProvider::new()))),
            Ephemeris::Provider(provider) => Ok(Some(provider)),
            Ephemeris::Opening(opening) => (opening.open)().map(Some),
        }
    }
}

/// The modern ephemeris a day's sunrise comes from beside the text: the
/// first of the remaining entries that opens a modern astronomy, else the
/// SDK's built-in one.
///
/// A classical entry is passed over rather than refused, because it gives
/// the very sunrise this setting asks to replace; an entry that fails to
/// open is passed over as the chain passes over it.
fn modern_sunrise(rest: Vec<Ephemeris>) -> Result<Box<dyn EphemerisProvider>, Error> {
    for entry in rest {
        if let Ok(Some(provider)) = entry.open_under(Siddhanta::Drik) {
            if provider.capabilities().astronomy != Astronomy::Classical {
                return Ok(provider);
            }
        }
    }
    built_in_sunrise()
}

/// The built-in ephemeris, as the modern sunrise's last resort.
#[cfg(feature = "builtin-ephemeris")]
#[allow(
    clippy::unnecessary_wraps,
    reason = "one signature for both builds; the other refuses"
)]
fn built_in_sunrise() -> Result<Box<dyn EphemerisProvider>, Error> {
    Ok(Box::new(teistro_ephemeris_builtin::provider::Builtin::new()))
}

/// Without the built-in ephemeris, a modern sunrise needs a modern entry
/// after the text's in the chain.
#[cfg(not(feature = "builtin-ephemeris"))]
fn built_in_sunrise() -> Result<Box<dyn EphemerisProvider>, Error> {
    Err(Error::unsupported(
        "a modern sunrise beside the Surya Siddhanta, with no modern ephemeris to give it",
    )
    .with_field("settings.frame.siddhanta.sunrise")
    .with_hint(
        "name a modern ephemeris after 'SURYA_SIDDHANTA' in the chain, or set the sunrise to TEXT",
    ))
}

/// The first entry of the chain that opens, or the refusal that names
/// every one that did not.
///
/// **A chain of one is not a chain**, and that was a defect in the other
/// three bindings before it was a rule: catching every refusal to try
/// the next entry turned a refusal carrying its status, its field and
/// its hint into a bare "nothing could be opened". With one entry there
/// is no next entry, so the refusal is the refusal.
///
/// The entries after the one that opens are handed to it: the Surya
/// Siddhanta at a modern sunrise takes that sunrise from them.
pub(crate) fn open(
    chain: Vec<Ephemeris>,
    siddhanta: Siddhanta,
    delta_t: DeltaTModel,
) -> Result<Option<Box<dyn EphemerisProvider>>, Error> {
    let mut refusals: Vec<String> = Vec::with_capacity(chain.len());
    let only = chain.len() == 1;
    let mut entries = chain.into_iter();
    while let Some(entry) = entries.next() {
        let name = entry.name();
        let rest = if matches!(entry, Ephemeris::SuryaSiddhanta) {
            entries.by_ref().collect()
        } else {
            Vec::new()
        };
        match entry.open_before(siddhanta, delta_t, rest) {
            Ok(opened) => return Ok(opened),
            Err(refusal) if only => return Err(refusal),
            Err(refusal) => refusals.push(format!("{name}: {refusal}")),
        }
    }
    Err(Error::unsupported(format!(
        "no ephemeris in the chain could be opened: {}",
        refusals.join("; ")
    ))
    .with_field("ephemeris"))
}

/// `CAPABILITY` when a call needs an ephemeris and the context has none.
///
/// One sentence in one place, which the boundary reaches through
/// [`Ephemeris::missing`]. Four entry points refuse this -- positions, a
/// chart, a panchanga and the engine passthrough -- and three of them once
/// carried a sentence written for a C caller (*pass a provider vtable to
/// `ts_context_new`*), which a Node, Dart or Python consumer has no way to
/// do. It names the **option**, which every binding spells `ephemeris`,
/// and the hint gives the three kinds of answer it takes.
pub(crate) fn no_ephemeris() -> Error {
    Error::new(
        Status::Capability,
        "the context has no ephemeris, and this call needs one",
    )
    .with_field("ephemeris")
    .with_hint(
        "name one when the context is built: `BUILTIN` for the ephemeris the SDK carries, \
         an adapter's descriptor for a real engine, or a provider of your own",
    )
}
