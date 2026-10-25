//! The port itself: one required operation, and the overrides a provider
//! may declare.

use serde::Serialize;
use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{JulianDay, Place, Ut1};

use crate::angles::{Angles, AnglesRequest};
use crate::body::{Body, TimeScale};
use crate::capabilities::{Capabilities, Obliquity};
use crate::columns::{CellStatus, PositionColumns};
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    ///
    /// ```
    /// use teistro_port_ephemeris::{Body, Frame, PositionRequest, TimeScale};
    ///
    /// let request = PositionRequest::new(&[2_451_545.0], TimeScale::Ut1, &[Body::Sun, Body::Moon], Frame::CANONICAL);
    /// assert_eq!((request.observer, request.speeds), (None, true));
    /// assert_eq!(request.frame, Frame::CANONICAL);
    /// ```
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
    ///
    /// ```
    /// use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro_port_ephemeris::{Body, Centre, Frame, PositionRequest, TimeScale};
    ///
    /// let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
    /// let request = PositionRequest::new(&[2_451_545.0], TimeScale::Ut1, &[Body::Sun, Body::Moon], Frame::CANONICAL);
    /// let seen = request.from_place(kathmandu);
    /// assert_eq!(seen.frame.centre, Centre::Topocentric);
    /// assert_eq!(seen.observer, Some(kathmandu));
    /// ```
    #[must_use]
    pub const fn from_place(self, place: Place) -> PositionRequest<'a> {
        PositionRequest {
            observer: Some(place),
            frame: self.frame.with_centre(Centre::Topocentric),
            ..self
        }
    }

    /// The same request without speeds.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Body, Frame, PositionRequest, TimeScale};
    ///
    /// let request = PositionRequest::new(&[2_451_545.0], TimeScale::Ut1, &[Body::Sun, Body::Moon], Frame::CANONICAL);
    /// assert!(!request.without_speeds().speeds);
    /// ```
    #[must_use]
    pub const fn without_speeds(self) -> PositionRequest<'a> {
        PositionRequest {
            speeds: false,
            ..self
        }
    }

    /// The same request in another frame.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Body, Coordinates, Frame, PositionRequest, TimeScale};
    ///
    /// let request = PositionRequest::new(&[2_451_545.0], TimeScale::Ut1, &[Body::Sun, Body::Moon], Frame::CANONICAL);
    /// let equatorial = Frame::CANONICAL.with_coordinates(Coordinates::Equatorial);
    /// assert_eq!(request.in_frame(equatorial).frame, equatorial);
    /// ```
    #[must_use]
    pub const fn in_frame(self, frame: Frame) -> PositionRequest<'a> {
        PositionRequest { frame, ..self }
    }

    /// The number of cells: instants times bodies.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Body, Frame, PositionRequest, TimeScale};
    ///
    /// let request = PositionRequest::new(&[2_451_545.0, 2_451_546.0, 2_451_547.0], TimeScale::Ut1, &[Body::Sun, Body::Moon], Frame::CANONICAL);
    /// assert_eq!(request.cell_count(), 6);
    /// ```
    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.jds.len().saturating_mul(self.bodies.len())
    }
}

/// An ephemeris provider. `positions` is required; every other method is
/// an override the provider declares in its capabilities and the SDK uses
/// when the profile's policy allows (ADR-0013). A provider is shared
/// between threads by the context, so it is `Send + Sync`.
///
/// ```
/// use teistro_port_ephemeris::{
///     Capabilities, EphemerisProvider, PositionColumns, PositionRequest, ProviderError, TestProvider,
/// };
///
/// // A provider needs two methods; every override defaults to refusing.
/// struct Delegating(TestProvider);
///
/// impl EphemerisProvider for Delegating {
///     fn capabilities(&self) -> Capabilities {
///         self.0.capabilities()
///     }
///     fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
///         self.0.positions(request)
///     }
/// }
///
/// let provider = Delegating(TestProvider::new());
/// assert!(provider.delta_t_seconds(2_451_545.0).is_err());
/// assert_eq!(provider.capabilities().identity.name, "test-provider");
/// ```
pub trait EphemerisProvider: Send + Sync {
    /// What the provider is and can do; stable for its lifetime.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    ///
    /// let capabilities = TestProvider::new().capabilities();
    /// assert_eq!(capabilities.identity.to_string(), "test-provider 1 (analytic)");
    /// assert!(capabilities.deterministic && capabilities.native);
    /// ```
    fn capabilities(&self) -> Capabilities;

    /// Positions over a grid of instants and bodies, in the provider's
    /// native frame or any frame it can produce natively. A frame it cannot
    /// produce is [`ProviderError::Unsupported`]; the SDK's completion then
    /// asks for the native frame and completes the rest.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Body, CellStatus, EphemerisProvider, Frame, PositionRequest, TestProvider, TimeScale};
    ///
    /// // A failing cell never fails the batch: 1e9 is beyond the coverage.
    /// let jds = [2_451_545.0, 1.0e9];
    /// let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
    /// let columns = TestProvider::new().positions(&request)?;
    /// assert_eq!(columns.status, [CellStatus::Ok, CellStatus::OutOfRange]);
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A malformed request, a frame the provider cannot produce, or a
    /// failure that affects the whole batch; per-cell failures are cell
    /// statuses, not errors.
    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError>;

    /// The obliquity and nutation at an instant; an override.
    ///
    /// ```
    /// use teistro_port_ephemeris::TimeScale;
    /// use teistro_port_ephemeris::{EphemerisProvider, Overrides, ProviderError, TestProvider};
    ///
    /// // The test provider declares no overrides, so the SDK does the work itself.
    /// let provider = TestProvider::new();
    /// assert!(!provider.capabilities().has(Overrides::OBLIQUITY));
    /// assert!(matches!(provider.obliquity(2_451_545.0, TimeScale::Tt), Err(ProviderError::Unsupported { .. })));
    /// ```
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
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, Overrides, ProviderError, TestProvider};
    ///
    /// // The test provider declares no overrides, so the SDK does the work itself.
    /// let provider = TestProvider::new();
    /// assert!(!provider.capabilities().has(Overrides::DELTA_T));
    /// assert!(matches!(provider.delta_t_seconds(2_451_545.0), Err(ProviderError::Unsupported { .. })));
    /// ```
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
    /// ```
    /// use teistro_core::catalogue::Ayanamsha;
    /// use teistro_port_ephemeris::TimeScale;
    /// use teistro_port_ephemeris::{EphemerisProvider, Overrides, ProviderError, TestProvider};
    ///
    /// // The test provider declares no overrides, so the SDK does the work itself.
    /// let provider = TestProvider::new();
    /// assert!(!provider.capabilities().has(Overrides::AYANAMSHA));
    /// assert!(matches!(provider.ayanamsha_deg(2_451_545.0, TimeScale::Tt, Ayanamsha::Lahiri), Err(ProviderError::Unsupported { .. })));
    /// ```
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
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, Overrides, ProviderError, TestProvider};
    ///
    /// // The test provider declares no overrides, so the SDK does the work itself.
    /// let provider = TestProvider::new();
    /// assert!(!provider.capabilities().has(Overrides::DUT1));
    /// assert!(matches!(provider.dut1_seconds(2_460_310.5), Err(ProviderError::Unsupported { .. })));
    /// ```
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
    /// ```
    /// use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_port_ephemeris::{Body, Horizon, HorizonEventKind, HorizonRequest};
    /// use teistro_port_ephemeris::{EphemerisProvider, Overrides, ProviderError, TestProvider};
    ///
    /// let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
    /// let sunrise = HorizonRequest {
    ///     body: Body::Sun,
    ///     kind: HorizonEventKind::Rise,
    ///     place: kathmandu,
    ///     from: JulianDay::literal(2_460_310.5),
    ///     window_days: 1.0,
    ///     horizon: Horizon::UPPER_LIMB_REFRACTION,
    /// };
    ///
    /// // The test provider declares no overrides, so the SDK does the work itself.
    /// let provider = TestProvider::new();
    /// assert!(!provider.capabilities().has(Overrides::RISE_SET));
    /// assert!(matches!(provider.horizon_event(&sunrise), Err(ProviderError::Unsupported { .. })));
    /// ```
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

    /// The ascendant and the midheaven at an instant and a place from the
    /// provider's own reckoning, tropical; an override
    /// ([`Overrides::ANGLES`](crate::capabilities::Overrides::ANGLES)),
    /// which the SDK's spherical astronomy stands in for otherwise.
    ///
    /// ```
    /// use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro_port_ephemeris::{AnglesRequest, TimeScale};
    /// use teistro_port_ephemeris::{EphemerisProvider, Overrides, ProviderError, TestProvider};
    ///
    /// let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
    /// let request = AnglesRequest { jd: 2_460_310.5, scale: TimeScale::Ut1, place: kathmandu };
    ///
    /// // The test provider declares no overrides, so the SDK does the work itself.
    /// let provider = TestProvider::new();
    /// assert!(!provider.capabilities().has(Overrides::ANGLES));
    /// assert!(matches!(provider.angles(&request), Err(ProviderError::Unsupported { .. })));
    /// ```
    ///
    /// # Errors
    ///
    /// [`ProviderError::Unsupported`] unless declared, or at a place the
    /// provider's reckoning has no answer for (a day with no sunrise, for
    /// a text that counts the ascendant from one).
    fn angles(&self, request: &AnglesRequest) -> Result<Angles, ProviderError> {
        let _ = request;
        Err(ProviderError::unsupported("angles"))
    }

    /// Every crossing of a quantity over a lattice inside a window from the
    /// provider's own search, in time order; an override
    /// ([`Overrides::CROSSINGS`](crate::capabilities::Overrides::CROSSINGS)),
    /// which the SDK's kernel stands in for otherwise.
    ///
    /// ```
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_port_ephemeris::{Body, CrossingRequest, Frame, Lattice, Quantity};
    /// use teistro_port_ephemeris::{EphemerisProvider, Overrides, ProviderError, TestProvider};
    ///
    /// // The Sun's ingresses into the signs over 2024.
    /// let ingresses = CrossingRequest {
    ///     quantity: Quantity::Longitude(Body::Sun),
    ///     lattice: Lattice::SIGNS,
    ///     from: JulianDay::literal(2_460_310.5),
    ///     to: JulianDay::literal(2_460_676.5),
    ///     tolerance_days: 1e-6,
    ///     frame: Frame::CANONICAL,
    ///     observer: None,
    /// };
    ///
    /// // The test provider declares no overrides, so the SDK does the work itself.
    /// let provider = TestProvider::new();
    /// assert!(!provider.capabilities().has(Overrides::CROSSINGS));
    /// assert!(matches!(provider.crossings(&ingresses), Err(ProviderError::Unsupported { .. })));
    /// ```
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
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    ///
    /// let manifest: serde_json::Value = serde_json::from_str(&TestProvider::new().native_manifest()?).unwrap();
    /// assert_eq!(manifest["engine"], "test-provider");
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
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
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, ProviderError, TestProvider};
    ///
    /// let provider = TestProvider::new();
    /// let answer = provider.native_call("tp_sum", r#"{"values": [1, 2, 3.5]}"#)?;
    /// // Read as JSON: the key order is serde_json's, which a feature can change.
    /// let answer: serde_json::Value = serde_json::from_str(&answer).unwrap();
    /// assert_eq!(answer, serde_json::json!({ "count": 3, "total": 6.5 }));
    /// assert!(matches!(provider.native_call("tp_nope", "{}"), Err(ProviderError::Unsupported { .. })));
    /// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
    /// ```
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
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    ///
    /// // The test provider describes two operations of its own.
    /// assert!(TestProvider::new().native().is_some());
    /// ```
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

    fn angles(&self, request: &AnglesRequest) -> Result<Angles, ProviderError> {
        (**self).angles(request)
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

    fn angles(&self, request: &AnglesRequest) -> Result<Angles, ProviderError> {
        (**self).angles(request)
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
/// written on, because the SDK asks every provider through
/// [`ask_positions`], which makes this check first. That is also what
/// lets a refusal name what is missing — the check runs where the words
/// survive, rather than in each binding, where only a code crosses back.
/// A provider called directly may make it of itself.
///
/// The order is the order a reader would ask in: what the frame needs,
/// then what the provider answers, then what the instants are.
///
/// ```
/// use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, PositionRequest, ProviderError, TestProvider, TimeScale, validate};
///
/// let capabilities = TestProvider::new().capabilities();
/// let jds = [2_451_545.0];
/// let true_node = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::TrueNode], Frame::CANONICAL);
/// // The refusal names the body and what the provider does answer.
/// let Err(ProviderError::Unsupported { what }) = validate(&capabilities, &true_node) else { panic!() };
/// assert!(what.starts_with("TRUE_NODE; it answers SUN, MOON"));
/// let nan = [f64::NAN];
/// assert!(validate(&capabilities, &PositionRequest::new(&nan, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL)).is_err());
/// ```
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
    // the 364 days it can compute rather than losing all of them, and it
    // is [`ask_positions`] that reckons it.
    Ok(())
}

/// Asks `provider` for positions the way the SDK always asks: the request
/// [`validate`]d against `capabilities` first, then only the instants
/// they cover, with every other instant's cells
/// [`CellStatus::OutOfRange`].
///
/// These are the port's rules and not a provider's. The SDK asks every
/// provider through here, native or foreign, so a provider is **never
/// asked for a body it did not declare or an instant it said it does not
/// have**, and none has to keep its own copy of either check. A year-long
/// grid whose last day runs past the ephemeris keeps the days it can
/// compute. Before coverage was the port's, three bindings each refused
/// the whole batch with their own error where a native provider refused
/// one cell.
///
/// A request the provider covers entirely is passed through untouched,
/// with no copy. One it covers in part is asked as the covered instants
/// alone, in order, and the answer is scattered back into the rows they
/// came from; one it covers not at all never reaches it.
///
/// ```
/// use teistro_port_ephemeris::{Body, CellStatus, EphemerisProvider, Frame, PositionRequest, TestProvider, TimeScale, ask_positions};
///
/// let provider = TestProvider::new();
/// let capabilities = provider.capabilities();
/// // The second instant is past the coverage: its cells are out of range and
/// // the provider is asked for the first alone.
/// let jds = [2_451_545.0, 1.0e9];
/// let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
/// let columns = ask_positions(&provider, &capabilities, &request)?;
/// assert_eq!(columns.status, [CellStatus::Ok, CellStatus::OutOfRange]);
/// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
/// ```
///
/// # Errors
///
/// What [`validate`] refuses, and whatever the provider refuses.
pub fn ask_positions<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    request: &PositionRequest<'_>,
) -> Result<PositionColumns, ProviderError> {
    validate(capabilities, request)?;
    if request.jds.iter().all(|jd| capabilities.covers(*jd)) {
        return provider.positions(request);
    }
    let covered: Vec<f64> = request
        .jds
        .iter()
        .copied()
        .filter(|jd| capabilities.covers(*jd))
        .collect();
    let mut columns = PositionColumns::new(request.jds.len(), request.bodies.len(), request.frame);
    columns.status.fill(CellStatus::OutOfRange);
    if covered.is_empty() {
        return Ok(columns);
    }
    let answered = provider.positions(&PositionRequest {
        jds: &covered,
        ..*request
    })?;
    columns.frame = answered.frame;
    let rows = request
        .jds
        .iter()
        .enumerate()
        .filter(|(_, jd)| capabilities.covers(**jd))
        .map(|(row, _)| row);
    for (from, to) in rows.enumerate() {
        columns.copy_row(to, &answered, from);
    }
    Ok(columns)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::quantity::{Altitude, Latitude, Longitude};

    use super::*;
    use crate::columns::Cell;
    use crate::test_provider::TestProvider;

    /// The test provider, remembering every instant it was asked for.
    struct Recording(TestProvider, std::sync::Mutex<Vec<f64>>);

    impl EphemerisProvider for Recording {
        fn capabilities(&self) -> Capabilities {
            self.0.capabilities()
        }

        fn positions(
            &self,
            request: &PositionRequest<'_>,
        ) -> Result<PositionColumns, ProviderError> {
            self.1.lock().unwrap().extend_from_slice(request.jds);
            self.0.positions(request)
        }
    }

    /// Coverage is a cell's outcome and the port's to reckon: the instants
    /// outside the declared span come back `OutOfRange`, the rest come back
    /// as the provider answered them, and the provider is never asked for
    /// an instant it does not have -- nor for a body it did not declare.
    #[test]
    fn a_provider_is_asked_only_for_what_it_covers() {
        let recording = Recording(TestProvider::new(), std::sync::Mutex::default());
        let capabilities = recording.capabilities();
        let (first, last) = TestProvider::JD_RANGE;
        let bodies = [Body::Sun, Body::Moon];
        let ask = |jds: &[f64]| {
            ask_positions(
                &recording,
                &capabilities,
                &PositionRequest::new(jds, TimeScale::Ut1, &bodies, Frame::CANONICAL),
            )
        };
        let answer = ask(&[first - 1.0, 2_460_000.5, last + 1.0, 2_460_100.5]).unwrap();
        let inside = [2_460_000.5, 2_460_100.5];
        assert_eq!(*recording.1.lock().unwrap(), inside);
        let direct = TestProvider::new()
            .positions(&PositionRequest::new(
                &inside,
                TimeScale::Ut1,
                &bodies,
                Frame::CANONICAL,
            ))
            .unwrap();
        for (row, from) in [(0, None), (1, Some(0)), (2, None), (3, Some(1))] {
            for body in 0..bodies.len() {
                let cell = answer.at(row, body).unwrap();
                match from {
                    None => assert_eq!(cell, Cell::failed(CellStatus::OutOfRange)),
                    Some(from) => assert_eq!(cell, direct.at(from, body).unwrap()),
                }
            }
        }

        recording.1.lock().unwrap().clear();
        let answer = ask(&[first - 2.0, last + 2.0]).unwrap();
        assert!(recording.1.lock().unwrap().is_empty());
        assert!(
            answer
                .cells()
                .all(|cell| cell.status == CellStatus::OutOfRange)
        );

        // A covered request passes through whole, and a NaN is refused
        // rather than marked, though no coverage holds it.
        recording.1.lock().unwrap().clear();
        ask(&inside).unwrap();
        assert_eq!(*recording.1.lock().unwrap(), inside);
        assert!(matches!(
            ask(&[2_460_000.5, f64::NAN]),
            Err(ProviderError::Invalid { .. })
        ));
        recording.1.lock().unwrap().clear();
        let undeclared = ask_positions(
            &recording,
            &capabilities,
            &PositionRequest::new(&inside, TimeScale::Ut1, &[Body::TrueNode], Frame::CANONICAL),
        );
        assert!(matches!(undeclared, Err(ProviderError::Unsupported { .. })));
        assert!(recording.1.lock().unwrap().is_empty());
    }

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
        let true_node =
            PositionRequest::new(&jds, TimeScale::Ut1, &[Body::TrueNode], Frame::CANONICAL);
        assert!(matches!(
            validate(&capabilities, &true_node),
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
