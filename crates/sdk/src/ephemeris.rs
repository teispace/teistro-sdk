//! Which ephemeris a context computes with, and the ordered chain that
//! chooses between them.

use teistro_core::Status;
use teistro_core::error::Error;
use teistro_port_ephemeris::{EphemerisProvider, TestProvider};

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
    /// What this entry is called in a refusal, so a chain that opens
    /// nothing can say which entries it tried.
    pub(crate) fn name(&self) -> String {
        match self {
            Ephemeris::None => String::from("none"),
            #[cfg(feature = "builtin-ephemeris")]
            Ephemeris::Builtin => String::from("builtin"),
            Ephemeris::Test => String::from("test"),
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
    /// could not.
    ///
    /// A chain opens its entries itself, so a consumer rarely needs
    /// this; what does is a caller holding one entry and wanting the
    /// provider it names -- the C boundary's selector, for one.
    ///
    /// # Errors
    ///
    /// Whatever a recipe's own opening refuses with.
    pub fn open(self) -> Result<Option<Box<dyn EphemerisProvider>>, Error> {
        match self {
            Ephemeris::None => Ok(None),
            #[cfg(feature = "builtin-ephemeris")]
            Ephemeris::Builtin => Ok(Some(Box::new(
                teistro_ephemeris_builtin::provider::Builtin::new(),
            ))),
            Ephemeris::Test => Ok(Some(Box::new(TestProvider::new()))),
            Ephemeris::Provider(provider) => Ok(Some(provider)),
            Ephemeris::Opening(opening) => (opening.open)().map(Some),
        }
    }
}

/// The first entry of the chain that opens, or the refusal that names
/// every one that did not.
///
/// **A chain of one is not a chain**, and that was a defect in the other
/// three bindings before it was a rule: catching every refusal to try
/// the next entry turned a refusal carrying its status, its field and
/// its hint into a bare "nothing could be opened". With one entry there
/// is no next entry, so the refusal is the refusal.
pub(crate) fn open(chain: Vec<Ephemeris>) -> Result<Option<Box<dyn EphemerisProvider>>, Error> {
    let mut refusals: Vec<String> = Vec::with_capacity(chain.len());
    let only = chain.len() == 1;
    for entry in chain {
        let name = entry.name();
        match entry.open() {
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
/// **The same sentence as `crates/ffi`'s `support::no_ephemeris`**, and
/// the duplication is an inventory rather than an accident: the
/// composition is written twice until
/// `03-design/rust-consumer-surface.md`'s third step has the boundary
/// depend on this crate, and a refusal is part of a composition. That
/// step deletes the boundary's copy; it does not add a third.
pub(crate) fn no_ephemeris() -> Error {
    Error::new(
        Status::Capability,
        "the context has no ephemeris, and this call needs one",
    )
    .with_field("ephemeris")
    .with_hint(
        "name one when the context is built: `builtin` for the ephemeris the SDK carries, \
         an adapter's descriptor for a real engine, or a provider of your own",
    )
}
