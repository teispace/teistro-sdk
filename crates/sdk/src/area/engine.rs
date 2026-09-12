//! `sdk.engine`: the operations your **ephemeris** brings with it,
//! beyond the ones the SDK names.

use teistro_core::error::Error;
use teistro_port_ephemeris::native::{Native, NativeFunction, NativeManifest};

use crate::context::Context;
use crate::ephemeris::no_ephemeris;

/// `sdk.engine`: what a real engine offers that the SDK has not ported.
///
/// Named for what it is — *this particular engine*, not the ephemeris
/// port — which is ADR-0030's fourth point and why it is not
/// `sdk.ephemeris`. An engine describes itself, so these operations are
/// the same however many functions it gains after the SDK ships.
///
/// **The adapter's own package carries a typed façade over them**, which
/// is a value a consumer takes rather than a method the SDK promises:
/// nothing here type-checks an operation the engine may not have.
#[derive(Clone, Copy, Debug)]
pub struct EngineArea<'a> {
    context: &'a Context,
}

impl<'a> EngineArea<'a> {
    pub(crate) fn of(context: &'a Context) -> EngineArea<'a> {
        EngineArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// The engine behind this context, or the refusal that says why
    /// there is none to ask.
    ///
    /// One reading, because every operation here needs it and each would
    /// otherwise spell both refusals itself.
    fn native(self) -> Result<Native<'a>, Error> {
        let provider = self.context.ephemeris().ok_or_else(no_ephemeris)?;
        let capabilities = provider.capabilities();
        if !capabilities.native {
            return Err(Error::unsupported(format!(
                "{} describes no operations of its own",
                capabilities.identity.name
            ))
            .with_hint("an adapter offers these by answering the port's native manifest"));
        }
        Ok(Native::new(provider))
    }

    /// Everything the engine describes about itself: every operation it
    /// offers, with its parameters and what each answers.
    ///
    /// # Errors
    ///
    /// A context with no ephemeris, an ephemeris that describes no
    /// operations of its own, or the engine's own refusal.
    pub fn manifest(self) -> Result<NativeManifest, Error> {
        self.native()?.manifest().map_err(Error::from)
    }

    /// The same manifest as the JSON the engine wrote, for a consumer
    /// passing it on rather than reading it.
    ///
    /// # Errors
    ///
    /// As [`EngineArea::manifest`].
    pub fn manifest_json(self) -> Result<String, Error> {
        self.native()?.manifest_json().map_err(Error::from)
    }

    /// The names of the operations the engine offers.
    ///
    /// # Errors
    ///
    /// As [`EngineArea::manifest`].
    pub fn names(self) -> Result<Vec<String>, Error> {
        Ok(self
            .manifest()?
            .functions
            .into_iter()
            .map(|function| function.name)
            .collect())
    }

    /// One operation's signature, by the name its manifest gives.
    ///
    /// # Errors
    ///
    /// As [`EngineArea::manifest`], or a name the engine does not
    /// offer.
    pub fn signature(self, name: &str) -> Result<NativeFunction, Error> {
        let manifest = self.manifest()?;
        manifest.function(name).cloned().ok_or_else(|| {
            Error::unsupported(format!("the engine offers no `{name}`"))
                .with_field("name")
                .with_hint(format!("it offers {} operation(s)", manifest.len()))
        })
    }

    /// Calls one of the engine's own operations by name, with its
    /// arguments as JSON values.
    ///
    /// # Errors
    ///
    /// As [`EngineArea::manifest`], a name the engine does not offer, an
    /// argument of the wrong shape, or the engine's own refusal.
    pub fn call(
        self,
        name: &str,
        arguments: &serde_json::Value,
    ) -> Result<serde_json::Value, Error> {
        self.native()?.call(name, arguments).map_err(Error::from)
    }

    /// The same call with its arguments and its answer as JSON text,
    /// for a consumer passing them through.
    ///
    /// # Errors
    ///
    /// As [`EngineArea::call`].
    pub fn call_json(self, name: &str, arguments_json: &str) -> Result<String, Error> {
        self.native()?
            .call_json(name, arguments_json)
            .map_err(Error::from)
    }
}
