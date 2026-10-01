//! The text's sky with a modern sunrise: the Surya Siddhanta for every
//! place, angle and zodiac, and the rise and set a modern ephemeris gives
//! under the convention asked for.
//!
//! Nepal's daily panchanga prints the text's limbs to the minute and a
//! sunrise the text does not give: the text's has no equation of time
//! (III.34 to 35, C37), and the print's is within 2.4 minutes of a modern
//! one (C39, `docs/03-design/nepal-day-measured.md`). A day's flags are
//! decided by where an end falls against the sunrises, so a limb ending
//! within minutes of sunrise is flagged by the print's sunrise and not by
//! the text's. This provider answers both questions the way the print
//! does, as one sky, so every reader of a sunrise (the almanac, the
//! calendar's day, the muhurta windows, the chart's day and horas) takes
//! the same one.

use teistro_astro::rise_set::Solver;
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{
    Angles, AnglesRequest, Body, Capabilities, EphemerisProvider, HorizonRequest, Obliquity,
    Overrides, PositionColumns, PositionRequest, ProviderError, TimeScale,
};

use crate::provider::SiddhantaProvider;

/// The Surya Siddhanta's sky with a modern ephemeris's sunrise.
///
/// Positions, the obliquity, the ayanamsha and the angles are the
/// text's, exactly as [`SiddhantaProvider`] gives them; a rise or set is
/// the SDK's solver over the modern ephemeris, under whatever convention
/// the request names, so a chart's `day.sunrise` applies to it as it
/// would over a modern sky.
///
/// ```
/// use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// use teistro_port_ephemeris::{
///     Body, EphemerisProvider, Horizon, HorizonEventKind, HorizonRequest, TestProvider,
/// };
/// use teistro_siddhanta::{ModernSunrise, SiddhantaProvider};
///
/// let sky = ModernSunrise::new(SiddhantaProvider::text(), Box::new(TestProvider::new()));
/// let kathmandu = Place::new(
///     Latitude::literal(27.7172),
///     Longitude::literal(85.324),
///     Altitude::literal(1400.0),
/// );
/// let rise = sky.horizon_event(&HorizonRequest {
///     body: Body::Sun,
///     kind: HorizonEventKind::Rise,
///     place: kathmandu,
///     from: JulianDay::literal(2_460_482.0),
///     window_days: 1.0,
///     horizon: Horizon::UPPER_LIMB_REFRACTION,
/// })?;
/// assert!(rise.is_some());
/// // And the stamp names both.
/// assert!(sky.capabilities().identity.data_version.contains("sunrise from"));
/// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
/// ```
pub struct ModernSunrise {
    text: SiddhantaProvider,
    modern: Box<dyn EphemerisProvider>,
    delta_t: DeltaTModel,
}

impl core::fmt::Debug for ModernSunrise {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ModernSunrise")
            .field("text", &self.text)
            .field("modern", &self.modern.capabilities().identity.name)
            .field("delta_t", &self.delta_t)
            .finish()
    }
}

impl ModernSunrise {
    /// The text's sky, with the rise and set the modern ephemeris gives.
    #[must_use]
    pub fn new(text: SiddhantaProvider, modern: Box<dyn EphemerisProvider>) -> ModernSunrise {
        ModernSunrise {
            text,
            modern,
            delta_t: DeltaTModel::default(),
        }
    }

    /// The same, with the Delta T the modern sunrise is solved under.
    #[must_use]
    pub fn with_delta_t(self, delta_t: DeltaTModel) -> ModernSunrise {
        ModernSunrise { delta_t, ..self }
    }

    /// The text this sky's places come from.
    #[must_use]
    pub const fn text(&self) -> &SiddhantaProvider {
        &self.text
    }

    /// The ephemeris this sky's sunrise comes from.
    #[must_use]
    pub fn modern(&self) -> &dyn EphemerisProvider {
        self.modern.as_ref()
    }
}

/// An SDK error as the port says it, keeping its kind: the solver's
/// refusal is the provider's.
fn refused(error: &Error, at: JulianDay<Ut1>) -> ProviderError {
    match error.status {
        Status::Unsupported => ProviderError::unsupported(error.message.clone()),
        Status::InvalidArg => ProviderError::invalid(error.message.clone()),
        Status::OutOfRange => ProviderError::OutOfRange { jd: at.get() },
        _ => ProviderError::Refused {
            detail: error.to_string(),
        },
    }
}

impl EphemerisProvider for ModernSunrise {
    fn capabilities(&self) -> Capabilities {
        let mut capabilities = self.text.capabilities();
        let modern = self.modern.capabilities().identity;
        capabilities.identity.data_version = format!(
            "{}, sunrise from {} {}",
            capabilities.identity.data_version, modern.name, modern.data_version
        );
        // The text's declaration of the rise and set stands: this sky
        // answers it, for any convention rather than only the text's.
        debug_assert!(capabilities.has(Overrides::RISE_SET));
        capabilities
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        self.text.positions(request)
    }

    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        self.text.obliquity(jd, scale)
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        self.text.delta_t_seconds(jd_ut1)
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: teistro_core::catalogue::Ayanamsha,
    ) -> Result<f64, ProviderError> {
        self.text.ayanamsha_deg(jd, scale, ayanamsha)
    }

    fn angles(&self, request: &AnglesRequest) -> Result<Angles, ProviderError> {
        self.text.angles(request)
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        if request.body != Body::Sun {
            return Err(ProviderError::unsupported(format!(
                "the rise and set of {}; this sky gives the Sun's",
                request.body.key()
            )));
        }
        // The route a modern sky's sunrise takes in the almanac: the SDK's
        // solver over the modern ephemeris's apparent Sun, completed by the
        // SDK. A modern provider's own rise and set is not consulted there
        // either, since only a classical astronomy defines its sunrise.
        let sky = Completion::new(
            self.modern.as_ref(),
            OverridePolicy::PreferNative,
            self.delta_t,
        );
        let solver = Solver::new(
            &sky,
            Body::Sun,
            request.place,
            request.horizon,
            self.delta_t,
        );
        solver
            .event(request.kind, request.from, request.window_days)
            .map(|found| found.map(|event| event.instant))
            .map_err(|error| refused(&error, request.from))
    }
}
