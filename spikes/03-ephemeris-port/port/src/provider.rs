//! The port itself: one required operation, optional overrides.

use crate::model::{
    AyanamshaId, Capabilities, Obliquity, PositionColumns, PositionRequest, ProviderError,
    TimeScale,
};

/// An ephemeris provider. `positions` is required; every other method is
/// an override the provider declares in its capabilities and the SDK uses
/// when the profile's policy allows (ADR-0013).
pub trait EphemerisProvider {
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
    /// longitudes subtract (without the nutation in longitude); an override.
    ///
    /// # Errors
    ///
    /// [`ProviderError::Unsupported`] unless declared, or for an ayanamsha
    /// the provider does not know.
    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        id: AyanamshaId,
    ) -> Result<f64, ProviderError> {
        let _ = (jd, scale, id);
        Err(ProviderError::unsupported("ayanamsha"))
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
        id: AyanamshaId,
    ) -> Result<f64, ProviderError> {
        (**self).ayanamsha_deg(jd, scale, id)
    }
}

/// Checks a request against a provider's capabilities before any work: a
/// topocentric frame needs an observer, every body must be offered.
///
/// # Errors
///
/// [`ProviderError::Invalid`] or [`ProviderError::Unsupported`] naming the
/// first problem.
pub fn validate(
    capabilities: &Capabilities,
    request: &PositionRequest<'_>,
) -> Result<(), ProviderError> {
    if request.frame.center == crate::model::Center::Topocentric && request.observer.is_none() {
        return Err(ProviderError::Invalid {
            detail: String::from("a topocentric frame needs an observer"),
        });
    }
    if let Some(body) = request.bodies.iter().find(|b| !capabilities.has_body(**b)) {
        return Err(ProviderError::Unsupported {
            what: format!("body {}", body.key()),
        });
    }
    if let Some(jd) = request.jds.iter().find(|jd| !jd.is_finite()) {
        return Err(ProviderError::Invalid {
            detail: format!("non-finite instant {jd}"),
        });
    }
    Ok(())
}
