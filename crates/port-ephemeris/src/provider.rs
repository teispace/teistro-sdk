//! The port itself: one required operation, and the overrides a provider
//! may declare.

use serde::Serialize;
use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{JulianDay, Place, Ut1};

use crate::body::{Body, TimeScale};
use crate::capabilities::{Capabilities, Obliquity};
use crate::columns::PositionColumns;
use crate::crossing::{CrossingRequest, Event};
use crate::error::ProviderError;
use crate::frame::{Centre, Frame};
use crate::horizon::HorizonRequest;

/// The one required operation's input: a grid of instants and bodies in a
/// requested frame.
///
/// ```
/// use teistro_port_ephemeris::{Body, Frame, PositionRequest, TimeScale};
///
/// let request = PositionRequest::new(&[2_451_545.0, 2_451_546.0], TimeScale::Tt, &[Body::Sun], Frame::CANONICAL);
/// assert_eq!(request.cell_count(), 2);
/// assert!(request.speeds);
/// ```
#[derive(Clone, Copy, Debug, Serialize)]
pub struct PositionRequest<'a> {
    /// The instants.
    pub jds: &'a [f64],
    /// Their time scale.
    pub scale: TimeScale,
    /// The bodies.
    pub bodies: &'a [Body],
    /// The frame the caller wants.
    pub frame: Frame,
    /// The observer, required when the frame is topocentric.
    pub observer: Option<Place>,
    /// Whether speeds are wanted.
    pub speeds: bool,
}

impl<'a> PositionRequest<'a> {
    /// A geocentric request with speeds.
    #[must_use]
    pub const fn new(
        jds: &'a [f64],
        scale: TimeScale,
        bodies: &'a [Body],
        frame: Frame,
    ) -> PositionRequest<'a> {
        PositionRequest {
            jds,
            scale,
            bodies,
            frame,
            observer: None,
            speeds: true,
        }
    }

    /// The same request seen from a place, in the topocentric frame.
    #[must_use]
    pub const fn from_place(self, place: Place) -> PositionRequest<'a> {
        PositionRequest {
            observer: Some(place),
            frame: self.frame.with_centre(Centre::Topocentric),
            ..self
        }
    }

    /// The same request without speeds.
    #[must_use]
    pub const fn without_speeds(self) -> PositionRequest<'a> {
        PositionRequest {
            speeds: false,
            ..self
        }
    }

    /// The same request in another frame.
    #[must_use]
    pub const fn in_frame(self, frame: Frame) -> PositionRequest<'a> {
        PositionRequest { frame, ..self }
    }

    /// The number of cells: instants times bodies.
    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.jds.len().saturating_mul(self.bodies.len())
    }
}

/// An ephemeris provider. `positions` is required; every other method is
/// an override the provider declares in its capabilities and the SDK uses
/// when the profile's policy allows (ADR-0013). A provider is shared
/// between threads by the context, so it is `Send + Sync`.
pub trait EphemerisProvider: Send + Sync {
    /// What the provider is and can do; stable for its lifetime.
    fn capabilities(&self) -> Capabilities;

    /// Positions over a grid of instants and bodies, in the provider's
    /// native frame or any frame it can produce natively. A frame it cannot
    /// produce is [`ProviderError::Unsupported`]; the SDK's completion then
    /// asks for the native frame and completes the rest.
    ///
    /// # Errors
    ///
    /// A malformed request, a frame the provider cannot produce, or a
    /// failure that affects the whole batch; per-cell failures are cell
    /// statuses, not errors.
    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError>;

    /// The obliquity and nutation at an instant; an override.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Unsupported`] unless declared.
    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        let _ = (jd, scale);
        Err(ProviderError::unsupported("obliquity"))
    }

    /// Delta T in seconds at a UT1 instant; an override.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Unsupported`] unless declared.
    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        let _ = jd_ut1;
        Err(ProviderError::unsupported("delta_t"))
    }

    /// The mean ayanamsha in degrees at an instant, the value sidereal
    /// longitudes subtract (without the nutation in longitude); an
    /// override.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Unsupported`] unless declared, or for an ayanamsha
    /// the provider does not know.
    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: Ayanamsha,
    ) -> Result<f64, ProviderError> {
        let _ = (jd, scale, ayanamsha);
        Err(ProviderError::unsupported("ayanamsha"))
    }

    /// UT1 less UTC in seconds at a UTC instant, from the provider's own
    /// copy of the IERS bulletins; an override. By definition within
    /// ±0.9 s.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Unsupported`] unless declared, or an instant the
    /// provider's bulletins do not reach.
    fn dut1_seconds(&self, jd_utc: f64) -> Result<f64, ProviderError> {
        let _ = jd_utc;
        Err(ProviderError::unsupported("dut1"))
    }

    /// The next horizon event of a body at a place from the provider's own
    /// search, or `None` when the event does not happen inside the
    /// request's window (a polar day, a circumpolar body); an override.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Unsupported`] unless declared, or for a horizon
    /// convention the provider cannot search under.
    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        let _ = request;
        Err(ProviderError::unsupported("rise_set"))
    }

    /// Every crossing of a quantity over a lattice inside a window from the
    /// provider's own search, in time order; an override
    /// ([`Overrides::CROSSINGS`](crate::capabilities::Overrides::CROSSINGS)),
    /// which the SDK's kernel stands in for otherwise.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Unsupported`] unless declared, or for a frame the
    /// provider cannot search in (a topocentric one, say).
    fn crossings(&self, request: &CrossingRequest) -> Result<Vec<Event>, ProviderError> {
        let _ = request;
        Err(ProviderError::unsupported("crossings"))
    }

    /// What the engine says it offers beyond this port, as the engine
    /// wrote it: a JSON manifest of its own operations
    /// ([`crate::native`]).
    ///
    /// The SDK does not keep a list of an engine's operations, so an
    /// operation the engine gains after the SDK ships is callable the
    /// day it appears in this. A provider that offers none says so by
    /// leaving this alone.
    ///
    /// # Errors
    ///
    /// `Unsupported` when the provider offers no native operations, or
    /// the engine's own failure to describe itself.
    fn native_manifest(&self) -> Result<String, ProviderError> {
        Err(ProviderError::unsupported("native_manifest"))
    }

    /// Calls one of the engine's own operations by the name its manifest
    /// gives, with arguments as a JSON object keyed by parameter name,
    /// answering with the engine's own JSON.
    ///
    /// The marshalling belongs to the adapter, generated from the
    /// engine's manifest, because a typed C call per function is what
    /// keeps a double in a floating-point register. The SDK relays.
    ///
    /// # Errors
    ///
    /// `Unsupported` when the provider offers no native operations, and
    /// otherwise whatever the adapter says: an unknown function, an
    /// argument it cannot marshal, or the engine's own failure.
    fn native_call(&self, function: &str, arguments_json: &str) -> Result<String, ProviderError> {
        let _ = (function, arguments_json);
        Err(ProviderError::unsupported("native_call"))
    }

    /// The engine's own operations, when it offers any.
    ///
    /// `None` rather than a handle that fails on every call, so that a
    /// caller asks once instead of discovering it per operation.
    fn native(&self) -> Option<crate::native::Native<'_>>
    where
        Self: Sized,
    {
        self.capabilities()
            .native
            .then(|| crate::native::Native::new(self))
    }
}

impl<P: EphemerisProvider + ?Sized> EphemerisProvider for &P {
    fn capabilities(&self) -> Capabilities {
        (**self).capabilities()
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        (**self).positions(request)
    }

    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        (**self).obliquity(jd, scale)
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        (**self).delta_t_seconds(jd_ut1)
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: Ayanamsha,
    ) -> Result<f64, ProviderError> {
        (**self).ayanamsha_deg(jd, scale, ayanamsha)
    }

    fn dut1_seconds(&self, jd_utc: f64) -> Result<f64, ProviderError> {
        (**self).dut1_seconds(jd_utc)
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        (**self).horizon_event(request)
    }

    fn crossings(&self, request: &CrossingRequest) -> Result<Vec<Event>, ProviderError> {
        (**self).crossings(request)
    }
    fn native_manifest(&self) -> Result<String, ProviderError> {
        (**self).native_manifest()
    }

    fn native_call(&self, function: &str, arguments_json: &str) -> Result<String, ProviderError> {
        (**self).native_call(function, arguments_json)
    }
}

/// A boxed provider is a provider.
///
/// The boundary owns its provider behind a `Box<dyn EphemerisProvider>`
/// (`ffi::TsContext`), so without this nothing could wrap what a binding
/// consumer hands the SDK — a cache, a counter or an adapter of their own
/// would be reachable from Rust and from nowhere else.
impl<P: EphemerisProvider + ?Sized> EphemerisProvider for Box<P> {
    fn capabilities(&self) -> Capabilities {
        (**self).capabilities()
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        (**self).positions(request)
    }

    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        (**self).obliquity(jd, scale)
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        (**self).delta_t_seconds(jd_ut1)
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: Ayanamsha,
    ) -> Result<f64, ProviderError> {
        (**self).ayanamsha_deg(jd, scale, ayanamsha)
    }

    fn dut1_seconds(&self, jd_utc: f64) -> Result<f64, ProviderError> {
        (**self).dut1_seconds(jd_utc)
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        (**self).horizon_event(request)
    }

    fn crossings(&self, request: &CrossingRequest) -> Result<Vec<Event>, ProviderError> {
        (**self).crossings(request)
    }
    fn native_manifest(&self) -> Result<String, ProviderError> {
        (**self).native_manifest()
    }

    fn native_call(&self, function: &str, arguments_json: &str) -> Result<String, ProviderError> {
        (**self).native_call(function, arguments_json)
    }
}

/// Checks a request against a provider's capabilities before any work: a
/// topocentric frame needs an observer, every body must be offered, every
/// instant must be finite. Coverage is not checked, because an instant
/// outside the declared span is a per-cell [`crate::CellStatus`] and not
/// a reason to refuse the batch.
///
/// Every provider is held to this, whichever side of the boundary it is
/// written on: a native one calls this itself, and a foreign one is
/// checked by [`crate::VtableProvider`] before the call crosses out. That
/// is what lets a refusal name what is missing — the check runs where the
/// words survive, rather than in each binding, where only a code crosses
/// back.
///
/// The order is the order a reader would ask in: what the frame needs,
/// then what the provider answers, then what the instants are.
///
/// # Errors
///
/// [`ProviderError::Invalid`] or [`ProviderError::Unsupported`] naming the
/// first problem.
pub fn validate(
    capabilities: &Capabilities,
    request: &PositionRequest<'_>,
) -> Result<(), ProviderError> {
    if request.frame.centre == Centre::Topocentric && request.observer.is_none() {
        return Err(ProviderError::invalid(
            "a topocentric frame needs an observer",
        ));
    }
    if let Some(body) = request.bodies.iter().find(|b| !capabilities.has_body(**b)) {
        // Naming what it *does* answer turns a refusal into an
        // instruction: the caller can see the catalogue key it meant.
        let answers = capabilities
            .bodies
            .iter()
            .map(|offered| offered.key())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(ProviderError::unsupported(format!(
            "{}; it answers {answers}",
            body.key()
        )));
    }
    if let Some(jd) = request.jds.iter().find(|jd| !jd.is_finite()) {
        return Err(ProviderError::invalid(format!("non-finite instant {jd}")));
    }
    // Coverage is deliberately not checked here. An instant outside the
    // declared span is a *per-cell* outcome — `CellStatus::OutOfRange` —
    // so a year-long grid whose last day runs past the ephemeris keeps
    // the 364 days it can compute rather than losing all of them. A
    // provider that would rather refuse the whole batch says so itself.
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::quantity::{Altitude, Latitude, Longitude};

    use super::*;
    use crate::test_provider::TestProvider;

    #[test]
    fn validation_names_the_first_problem() {
        let capabilities = TestProvider::new().capabilities();
        let jds = [2_451_545.0];
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
        assert!(validate(&capabilities, &request).is_ok());
        let topocentric = request.in_frame(Frame::CANONICAL.with_centre(Centre::Topocentric));
        assert!(matches!(
            validate(&capabilities, &topocentric),
            Err(ProviderError::Invalid { .. })
        ));
        let place = Place::new(
            Latitude::literal(27.7),
            Longitude::literal(85.3),
            Altitude::literal(1400.0),
        );
        let placed = request.from_place(place);
        assert_eq!(placed.frame.centre, Centre::Topocentric);
        assert!(validate(&capabilities, &placed).is_ok());
        let pluto = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Pluto], Frame::CANONICAL);
        assert!(matches!(
            validate(&capabilities, &pluto),
            Err(ProviderError::Unsupported { .. })
        ));
        let nan = [f64::NAN];
        let bad = PositionRequest::new(&nan, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
        assert!(matches!(
            validate(&capabilities, &bad),
            Err(ProviderError::Invalid { .. })
        ));
        assert!(!request.without_speeds().speeds);
        let provider = TestProvider::new();
        let by_ref: &dyn EphemerisProvider = &provider;
        assert!(by_ref.obliquity(2_451_545.0, TimeScale::Tt).is_err());
        assert!(by_ref.delta_t_seconds(2_451_545.0).is_err());
        assert!(by_ref.dut1_seconds(2_451_545.0).is_err());
        assert!(
            by_ref
                .ayanamsha_deg(2_451_545.0, TimeScale::Tt, Ayanamsha::Lahiri)
                .is_err()
        );
    }
}
