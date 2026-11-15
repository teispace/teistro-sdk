//! The checks of the conventions the port states (boundary audit, rank 7):
//! the ayanamsha on both bases against the SDK's catalogue, Delta T under
//! each model a provider answers, rise and set under every horizon the
//! port names (the lower limb, a stated atmosphere, twilight, the Moon),
//! sidereal crossings, topocentric positions, and positions against a
//! reference provider.
//!
//! Each compares the provider with the SDK's own routine over the same
//! question, so what a check measures is the provider's convention and
//! nothing the two sides reckon differently.

use teistro_astro::ayanamsha;
use teistro_astro::events::Search;
use teistro_astro::precession::PrecessionModel;
use teistro_astro::{Completion, DeltaTModel, delta_t};
use teistro_core::angle::difference_deg;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Tt, Ut1};
use teistro_core::settings::{Atmosphere, AyanamshaBasis, AyanamshaChoice, OverridePolicy};
use teistro_port_ephemeris::{
    Astronomy, Body, Capabilities, Conventions, CrossingRequest, DiscPoint, EphemerisProvider,
    Frame, Horizon, Lattice, Overrides, PositionRequest, ProviderError, Quantity, Refraction,
    TimeScale, Zodiac,
};

use super::{Bounds, Check};

/// The horizons the rise and set override is checked under, each with the
/// body it is asked of and its bound: the geometric and the almanac's
/// sunrise the kit always held, and the lower limb, a stated atmosphere
/// (Bennett's formula through the standard air), astronomical twilight
/// and the Moon's rising, which the port now states in full.
///
/// ```
/// use teistro_ephemeris_kit::Bounds;
///
/// let report = teistro_ephemeris_kit::kit::run(&teistro_port_ephemeris::TestProvider::new(), &Bounds::DEFAULT);
/// for name in ["override_rise_set_lower_limb", "override_rise_set_atmosphere", "override_rise_set_twilight", "override_rise_set_moon"] {
///     assert!(report.check(name).is_some(), "{name}");
/// }
/// ```
pub(crate) fn horizons(bounds: &Bounds) -> [(&'static str, (Body, Horizon), f64); 6] {
    let lower_limb = Horizon {
        disc: DiscPoint::LowerLimb,
        ..Horizon::UPPER_LIMB_REFRACTION
    };
    let atmosphere = Horizon {
        refraction: Refraction::Atmosphere(Atmosphere::STANDARD),
        ..Horizon::UPPER_LIMB_REFRACTION
    };
    let twilight = Horizon {
        altitude_deg: -12.0,
        ..Horizon::CENTRE_NO_REFRACTION
    };
    [
        (
            "override_rise_set_geometric",
            (Body::Sun, Horizon::CENTRE_NO_REFRACTION),
            bounds.rise_set_geometric_seconds,
        ),
        (
            "override_rise_set_refracted",
            (Body::Sun, Horizon::UPPER_LIMB_REFRACTION),
            bounds.rise_set_refracted_seconds,
        ),
        (
            "override_rise_set_lower_limb",
            (Body::Sun, lower_limb),
            bounds.rise_set_refracted_seconds,
        ),
        (
            "override_rise_set_atmosphere",
            (Body::Sun, atmosphere),
            bounds.rise_set_refracted_seconds,
        ),
        (
            "override_rise_set_twilight",
            (Body::Sun, twilight),
            bounds.rise_set_geometric_seconds,
        ),
        (
            "override_rise_set_moon",
            (Body::Moon, Horizon::UPPER_LIMB_REFRACTION),
            bounds.rise_set_refracted_seconds,
        ),
    ]
}

/// The instants the conventions are checked at inside a provider's
/// coverage: 1900, 2000 and 2050, those it covers.
fn epochs(capabilities: &Capabilities) -> Vec<f64> {
    [2_415_020.5, teistro_astro::iau::DJ00, 2_469_807.5]
        .into_iter()
        .filter(|jd| capabilities.covers(*jd))
        .collect()
}

/// Delta T under each model the port names: a provider answers a model
/// or refuses it, and every answer agrees with the SDK's routine for that
/// model, a custom value exactly. A model the SDK itself does not compute
/// (one registered and not yet cited) has nothing to be held to, so it
/// is listed and not asked. A provider that declares the override and
/// answers no model fails.
pub(crate) fn check_delta_t<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    bounds: &Bounds,
) -> Check {
    const NAME: &str = "override_delta_t";
    if !capabilities.has(Overrides::DELTA_T) {
        return Check::skipped(NAME, "not declared");
    }
    let at = epochs(capabilities);
    if at.is_empty() {
        return Check::skipped(NAME, "the coverage misses 1900, 2000 and 2050");
    }
    let models = [
        DeltaTModel::TableThenModel,
        DeltaTModel::EspenakMeeus2006,
        DeltaTModel::StephensonMorrisonHohenkerk2016,
        DeltaTModel::Custom { seconds: 69.184 },
    ];
    let mut worst = (0.0f64, String::new());
    let (mut answered, mut refused, mut uncompared) = (Vec::new(), Vec::new(), Vec::new());
    'models: for model in models {
        let mut supported = true;
        for &jd in &at {
            let Ok(sdk) = delta_t(JulianDay::<Ut1>::literal(jd), model) else {
                uncompared.push(model.key());
                continue 'models;
            };
            let sdk = sdk.seconds;
            let native = match provider.delta_t(jd, model) {
                Ok(native) => native,
                Err(ProviderError::Unsupported { .. }) => {
                    supported = false;
                    break;
                }
                Err(error) => {
                    return Check::fail(
                        NAME,
                        format!("{}: declared, but failed at JD {jd}: {error}", model.key()),
                    );
                }
            };
            if !native.is_valid() {
                return Check::fail(
                    NAME,
                    format!("{} at JD {jd}: {native:?} is not a value", model.key()),
                );
            }
            // A custom value is the value, to the bit; a model's is the
            // model's within the bound.
            let gap = (native.seconds - sdk).abs();
            if matches!(model, DeltaTModel::Custom { .. }) && gap != 0.0 {
                return Check::fail(
                    NAME,
                    format!("a custom {sdk} s came back as {} s", native.seconds),
                );
            }
            if gap > worst.0 {
                worst = (gap, format!("{} at JD {jd}", model.key()));
            }
        }
        if supported {
            answered.push(model.key());
        } else {
            refused.push(model.key());
        }
    }
    if answered.is_empty() {
        return Check::fail(NAME, "declared, but every model was refused");
    }
    let listed = |keys: &[&str]| {
        if keys.is_empty() {
            String::from("none")
        } else {
            keys.join(", ")
        }
    };
    Check::measured(
        NAME,
        worst.0,
        bounds.delta_t_seconds,
        "s",
        format!(
            "answered {}; refused {}; no SDK model to compare {}; worst {}",
            answered.join(", "),
            listed(&refused),
            listed(&uncompared),
            worst.1
        ),
    )
}

/// A native ayanamsha on both bases against the SDK's catalogue under
/// the standard models, for every listed member the catalogue reckons,
/// at 1900, 2000 and 2050. A classical text's member is its own
/// definition, which [`super::check_ayanamsha`] holds to the text.
pub(crate) fn check_ayanamsha_values<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    bounds: &Bounds,
) -> Check {
    const NAME: &str = "override_ayanamsha_value";
    if !capabilities.has(Overrides::AYANAMSHA) {
        return Check::skipped(NAME, "not declared");
    }
    if capabilities.astronomy == Astronomy::Classical {
        return Check::skipped(NAME, "a classical text's ayanamsha is its own definition");
    }
    let mut worst = (0.0f64, String::new());
    let mut compared = 0u32;
    for &member in &capabilities.ayanamshas {
        let choice = AyanamshaChoice::Catalogued { id: member };
        for jd in epochs(capabilities) {
            for basis in [AyanamshaBasis::Mean, AyanamshaBasis::True] {
                let native = match provider.ayanamsha_deg(
                    jd,
                    TimeScale::Tt,
                    member,
                    basis,
                    Conventions::STANDARD,
                ) {
                    Ok(native) => native,
                    // The mean basis alone: the catalogue answers the true one.
                    Err(ProviderError::Unsupported { .. }) if basis == AyanamshaBasis::True => {
                        continue;
                    }
                    Err(error) => {
                        return Check::fail(
                            NAME,
                            format!("{}: declared, but failed: {error}", member.key()),
                        );
                    }
                };
                let Ok(sdk) = ayanamsha::value_deg(
                    &choice,
                    JulianDay::<Tt>::literal(jd),
                    basis,
                    PrecessionModel::default(),
                    DeltaTModel::TableThenModel,
                ) else {
                    // A member the catalogue does not reckon: nothing to hold it to.
                    break;
                };
                compared += 1;
                let gap = difference_deg(native, sdk).abs();
                if gap > worst.0 {
                    worst = (gap, format!("{} {basis:?} at JD {jd}", member.key()));
                }
            }
        }
    }
    if compared == 0 {
        return Check::skipped(NAME, "no listed member the catalogue reckons");
    }
    Check::measured(
        NAME,
        worst.0,
        bounds.ayanamsha_value_deg,
        "deg",
        format!(
            "{compared} values on both bases against the catalogue; worst {}",
            worst.1
        ),
    )
}

/// A native crossing search in a sidereal frame against the SDK's kernel
/// in the same frame: the Sun's sign ingresses over a year, in the first
/// listed ayanamsha's zodiac on the mean basis (an engine's sidereal
/// positions are on the mean basis, the port's sidereal frame's).
pub(crate) fn check_sidereal_crossings<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    jds: &[f64],
    bounds: &Bounds,
) -> Check {
    const NAME: &str = "override_crossings_sidereal";
    if !capabilities.has(Overrides::CROSSINGS) {
        return Check::skipped(NAME, "not declared");
    }
    if !capabilities.has_body(Body::Sun) {
        return Check::skipped(NAME, "the Sun is not offered");
    }
    let Some(&member) = capabilities.ayanamshas.first() else {
        return Check::skipped(NAME, "no ayanamsha listed");
    };
    let frame = Frame {
        zodiac: Zodiac::Sidereal { ayanamsha: member },
        ..Frame::CANONICAL
    };
    let Some(from) = jds
        .first()
        .and_then(|jd| JulianDay::<Ut1>::try_new(*jd).ok())
    else {
        return Check::skipped(NAME, "no instant to start from");
    };
    let Ok(to) = from.plus_days(366.0) else {
        return Check::skipped(NAME, "the window runs off the calendar");
    };
    let quantity = Quantity::Longitude(Body::Sun);
    let request = CrossingRequest {
        quantity,
        lattice: Lattice::SIGNS,
        from,
        to,
        tolerance_days: 1e-7,
        frame,
        observer: None,
        conventions: Conventions::STANDARD,
    };
    let native = match provider.crossings(&request) {
        Ok(events) => events,
        Err(ProviderError::Unsupported { what }) => {
            return Check::skipped(NAME, format!("the provider does not offer {what}"));
        }
        Err(error) => return Check::fail(NAME, format!("declared, but failed: {error}")),
    };
    let completion = Completion::new(
        provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    )
    .with_ayanamsha_basis(AyanamshaBasis::Mean);
    let longitudes = completion.longitudes(frame);
    let sdk = match Search::new(&longitudes, quantity, Lattice::SIGNS).between(from, to) {
        Ok(events) => events,
        Err(error) => return Check::fail(NAME, format!("the SDK's kernel failed: {error}")),
    };
    if native.len() != sdk.len() {
        return Check::fail(
            NAME,
            format!(
                "the provider found {} sidereal ingresses where the SDK's kernel found {}",
                native.len(),
                sdk.len()
            ),
        );
    }
    let mut worst = 0.0f64;
    for (theirs, ours) in native.iter().zip(&sdk) {
        if theirs.direction != ours.direction
            || difference_deg(theirs.boundary_deg, ours.boundary_deg).abs() > 1e-6
        {
            return Check::fail(
                NAME,
                format!(
                    "at JD {}: the provider crossed {}° where the SDK's kernel crossed {}°",
                    theirs.instant.get(),
                    theirs.boundary_deg,
                    ours.boundary_deg
                ),
            );
        }
        worst = worst.max((theirs.instant.get() - ours.instant.get()).abs() * 86_400.0);
    }
    Check::measured(
        NAME,
        worst,
        bounds.crossings_seconds,
        "s",
        format!(
            "{} ingresses of the Sun in the {} zodiac (mean basis) over a year",
            native.len(),
            member.key()
        ),
    )
}

/// The place topocentric positions are checked from: Kathmandu at its
/// height, where the Moon's parallax is near its greatest.
const KATHMANDU: (f64, f64, f64) = (27.7172, 85.324, 1400.0);

/// A provider's own topocentric positions of the Sun and the Moon against
/// the SDK's correction of its geocentric ones, arcseconds, asked on TT so
/// no Delta T enters either side.
pub(crate) fn check_topocentric<P: EphemerisProvider + ?Sized>(
    provider: &P,
    capabilities: &Capabilities,
    jds: &[f64],
    bounds: &Bounds,
) -> Check {
    const NAME: &str = "override_topocentric";
    if !capabilities.has(Overrides::TOPOCENTRIC) {
        return Check::skipped(NAME, "not declared");
    }
    let bodies: Vec<Body> = [Body::Sun, Body::Moon]
        .into_iter()
        .filter(|body| capabilities.has_body(*body))
        .collect();
    if bodies.is_empty() {
        return Check::skipped(NAME, "neither the Sun nor the Moon is offered");
    }
    let (latitude, longitude, height) = KATHMANDU;
    let place = Place::new(
        Latitude::literal(latitude),
        Longitude::literal(longitude),
        Altitude::literal(height),
    );
    // The topocentric frame, so the SDK's side applies its centre step to
    // the provider's geocentric answer rather than passing the question
    // through to it.
    let request = PositionRequest::new(jds, TimeScale::Tt, &bodies, Frame::CANONICAL)
        .from_place(place)
        .without_speeds();
    let native = match provider.positions(&request) {
        Ok(columns) => columns,
        Err(error) => return Check::fail(NAME, format!("declared, but failed: {error}")),
    };
    let completion = Completion::new(
        provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let sdk = match completion.positions(&request) {
        Ok(done) => done.columns,
        Err(error) => return Check::fail(NAME, format!("the SDK's correction failed: {error}")),
    };
    let worst = worst_arcsec(native.cells().zip(sdk.cells()));
    Check::measured(
        NAME,
        worst,
        bounds.topocentric_arcsec,
        "arcsec",
        format!("the Sun and the Moon from Kathmandu at {height} m against the SDK's correction"),
    )
}

/// The greatest separation in longitude or latitude of paired cells that
/// both answered, arcseconds.
fn worst_arcsec<'a>(
    pairs: impl Iterator<Item = (teistro_port_ephemeris::Cell, teistro_port_ephemeris::Cell)> + 'a,
) -> f64 {
    pairs
        .filter(|(theirs, ours)| theirs.is_ok() && ours.is_ok())
        .map(|(theirs, ours)| {
            (difference_deg(theirs.lon, ours.lon).abs() * 3600.0)
                .max((theirs.lat - ours.lat).abs() * 3600.0)
        })
        .fold(0.0, f64::max)
}

/// A provider's positions against a reference provider's over the bodies
/// both offer, apparent and geocentric on the kit's instants, arcseconds:
/// an engine against the built-in, or a tier against a fuller one.
///
/// ```
/// use teistro_ephemeris_kit::Bounds;
/// use teistro_ephemeris_kit::kit::check_reference;
/// use teistro_port_ephemeris::TestProvider;
///
/// let check = check_reference(&TestProvider::new(), &TestProvider::new(), &Bounds::DEFAULT);
/// assert!(check.passed && check.measured == Some(0.0), "{}", check.detail);
/// ```
#[must_use]
pub fn check_reference<P, R>(provider: &P, reference: &R, bounds: &Bounds) -> Check
where
    P: EphemerisProvider + ?Sized,
    R: EphemerisProvider + ?Sized,
{
    const NAME: &str = "positions_reference";
    let ours = provider.capabilities();
    let theirs = reference.capabilities();
    let bodies: Vec<Body> = ours
        .bodies
        .iter()
        .copied()
        .filter(|body| theirs.has_body(*body))
        .collect();
    let jds: Vec<f64> = super::instants(&ours)
        .into_iter()
        .filter(|jd| theirs.covers(*jd))
        .collect();
    if bodies.is_empty() || jds.is_empty() {
        return Check::skipped(NAME, "no body or instant both providers cover");
    }
    let request =
        PositionRequest::new(&jds, TimeScale::Tt, &bodies, Frame::CANONICAL).without_speeds();
    let completed = |source: &dyn EphemerisProvider| {
        Completion::new(source, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel)
            .positions(&request)
            .map(|done| done.columns)
    };
    let (native, other) = match (
        completed(&Forward(provider)),
        completed(&Forward(reference)),
    ) {
        (Ok(native), Ok(other)) => (native, other),
        (Err(error), _) => return Check::fail(NAME, format!("the provider failed: {error}")),
        (_, Err(error)) => return Check::fail(NAME, format!("the reference failed: {error}")),
    };
    let worst = worst_arcsec(native.cells().zip(other.cells()));
    Check::measured(
        NAME,
        worst,
        bounds.reference_arcsec,
        "arcsec",
        format!(
            "{} bodies on {} instants against {} {}",
            bodies.len(),
            jds.len(),
            theirs.identity.name,
            theirs.identity.data_version
        ),
    )
}

/// A provider as a trait object, for a provider that is not `Sized`.
struct Forward<'a, P: EphemerisProvider + ?Sized>(&'a P);

impl<P: EphemerisProvider + ?Sized> EphemerisProvider for Forward<'_, P> {
    fn capabilities(&self) -> Capabilities {
        self.0.capabilities()
    }

    fn positions(
        &self,
        request: &PositionRequest<'_>,
    ) -> Result<teistro_port_ephemeris::PositionColumns, ProviderError> {
        self.0.positions(request)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::quantity::Ut1;
    use teistro_port_ephemeris::{CrossingEvent, DeltaTEstimate, PositionColumns, TestProvider};

    use super::*;

    /// What a [`Probe`] gets wrong, one fault at a time.
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Fault {
        /// Nothing.
        None,
        /// Delta T fails for a reason other than a refusal.
        DeltaTFails,
        /// Delta T answers a value that is not one.
        DeltaTNotANumber,
        /// A custom Delta T comes back a second off.
        CustomOff,
        /// Every Delta T model refused.
        DeltaTRefused,
        /// A model's Delta T two seconds off.
        DeltaTLate,
        /// The ayanamsha fails outright.
        AyanamshaFails,
        /// The true basis refused, the mean one answered.
        MeanOnly,
        /// A topocentric question answered from the centre of the Earth.
        Geocentric,
        /// Positions fail.
        PositionsFail,
        /// Every longitude a hundredth of a degree east.
        East,
        /// No Sun and no Moon offered.
        NoLuminaries,
        /// Crossings refused as a frame it does not offer.
        CrossingsRefused,
        /// Crossings fail.
        CrossingsFail,
        /// One sidereal ingress missing.
        CrossingDropped,
        /// The first ingress through the wrong boundary.
        CrossingMoved,
        /// Every ingress ten minutes late.
        CrossingsLate,
    }

    /// The test sky, declaring every override this module checks and
    /// answering each with the SDK's own routine unless its fault says
    /// otherwise.
    struct Probe(Fault);

    impl Probe {
        /// The test sky's answer, with any topocentric frame completed by
        /// the SDK from the geocentric one.
        fn sky(request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
            if request.frame == Frame::CANONICAL {
                return TestProvider::new().positions(request);
            }
            Completion::new(
                &TestProvider::new(),
                OverridePolicy::SdkOnly,
                DeltaTModel::TableThenModel,
            )
            .positions(request)
            .map(|done| done.columns)
            .map_err(|e| ProviderError::invalid(e.to_string()))
        }
    }

    impl EphemerisProvider for Probe {
        fn capabilities(&self) -> Capabilities {
            let plain = TestProvider::new().capabilities();
            Capabilities {
                overrides: Overrides::DELTA_T
                    .with(Overrides::AYANAMSHA)
                    .with(Overrides::TOPOCENTRIC)
                    .with(Overrides::CROSSINGS),
                ayanamshas: vec![teistro_core::catalogue::Ayanamsha::Lahiri],
                bodies: if self.0 == Fault::NoLuminaries {
                    plain
                        .bodies
                        .iter()
                        .copied()
                        .filter(|b| !matches!(b, Body::Sun | Body::Moon))
                        .collect()
                } else {
                    plain.bodies.clone()
                },
                ..plain
            }
        }

        fn positions(
            &self,
            request: &PositionRequest<'_>,
        ) -> Result<PositionColumns, ProviderError> {
            match self.0 {
                Fault::PositionsFail => Err(ProviderError::invalid("the table is torn")),
                Fault::Geocentric if request.observer.is_some() => {
                    let mut columns =
                        TestProvider::new().positions(&request.in_frame(Frame::CANONICAL))?;
                    columns.frame = request.frame;
                    Ok(columns)
                }
                Fault::East => {
                    let mut columns = Probe::sky(request)?;
                    for lon in &mut columns.lon {
                        *lon = (*lon + 0.01).rem_euclid(360.0);
                    }
                    Ok(columns)
                }
                _ => Probe::sky(request),
            }
        }

        fn delta_t(
            &self,
            jd_ut1: f64,
            model: DeltaTModel,
        ) -> Result<DeltaTEstimate, ProviderError> {
            let mut estimate = delta_t(JulianDay::<Ut1>::literal(jd_ut1), model)
                .map(|d| d.estimate())
                .map_err(|e| ProviderError::invalid(e.to_string()))?;
            let custom = matches!(model, DeltaTModel::Custom { .. });
            match self.0 {
                Fault::DeltaTFails => return Err(ProviderError::invalid("no bulletin")),
                Fault::DeltaTRefused => return Err(ProviderError::unsupported("any Delta T")),
                Fault::DeltaTNotANumber => estimate.seconds = f64::NAN,
                Fault::CustomOff if custom => estimate.seconds += 1.0,
                Fault::DeltaTLate if !custom => estimate.seconds += 2.0,
                _ => {}
            }
            Ok(estimate)
        }

        fn ayanamsha_deg(
            &self,
            jd: f64,
            _scale: TimeScale,
            a: teistro_core::catalogue::Ayanamsha,
            basis: AyanamshaBasis,
            _conventions: Conventions,
        ) -> Result<f64, ProviderError> {
            match (self.0, basis) {
                (Fault::AyanamshaFails, _) => Err(ProviderError::invalid("no table")),
                (Fault::MeanOnly, AyanamshaBasis::True) => {
                    Err(ProviderError::unsupported("the true basis"))
                }
                _ => ayanamsha::value_deg(
                    &a.into(),
                    JulianDay::literal(jd),
                    basis,
                    PrecessionModel::default(),
                    DeltaTModel::TableThenModel,
                )
                .map_err(|e| ProviderError::invalid(e.to_string())),
            }
        }

        fn crossings(
            &self,
            request: &CrossingRequest,
        ) -> Result<Vec<CrossingEvent>, ProviderError> {
            match self.0 {
                Fault::CrossingsRefused => {
                    return Err(ProviderError::unsupported("a sidereal zodiac"));
                }
                Fault::CrossingsFail => return Err(ProviderError::invalid("the search diverged")),
                _ => {}
            }
            let completion =
                Completion::new(self, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel)
                    .with_ayanamsha_basis(AyanamshaBasis::Mean);
            let longitudes = completion.longitudes(request.frame);
            let mut events = Search::new(&longitudes, request.quantity, request.lattice)
                .between(request.from, request.to)
                .map_err(|e| ProviderError::invalid(e.to_string()))?;
            match self.0 {
                Fault::CrossingDropped => {
                    events.pop();
                }
                Fault::CrossingMoved => {
                    if let Some(first) = events.first_mut() {
                        first.boundary_deg += 30.0;
                    }
                }
                Fault::CrossingsLate => {
                    for event in &mut events {
                        event.instant = event.instant.plus_days(600.0 / 86_400.0).unwrap();
                    }
                }
                _ => {}
            }
            Ok(events)
        }
    }

    fn delta_t_of(fault: Fault) -> Check {
        let probe = Probe(fault);
        check_delta_t(&probe, &probe.capabilities(), &Bounds::DEFAULT)
    }

    /// The kit's own instants for the probe.
    fn jds() -> Vec<f64> {
        super::super::instants(&Probe(Fault::None).capabilities())
    }

    #[test]
    fn delta_t_is_held_model_by_model_and_a_custom_value_to_the_bit() {
        let honest = delta_t_of(Fault::None);
        assert!(
            honest.passed && honest.measured == Some(0.0),
            "{}",
            honest.detail
        );
        assert!(honest.detail.contains("refused none"), "{}", honest.detail);
        for (fault, says) in [
            (Fault::DeltaTFails, "declared, but failed"),
            (Fault::DeltaTNotANumber, "is not a value"),
            (Fault::CustomOff, "came back as"),
            (Fault::DeltaTRefused, "every model was refused"),
        ] {
            let check = delta_t_of(fault);
            assert!(
                !check.passed && check.detail.contains(says),
                "{fault:?}: {}",
                check.detail
            );
        }
        let late = delta_t_of(Fault::DeltaTLate);
        assert!(!late.passed, "{}", late.detail);
        assert!(
            late.measured.is_some_and(|m| (m - 2.0).abs() < 1e-9),
            "{}",
            late.detail
        );
        // Undeclared, it is not asked.
        let plain = TestProvider::new();
        assert!(!check_delta_t(&plain, &plain.capabilities(), &Bounds::DEFAULT).ran());
    }

    #[test]
    fn ayanamsha_values_are_held_on_both_bases() {
        let of = |fault| {
            let probe = Probe(fault);
            check_ayanamsha_values(&probe, &probe.capabilities(), &Bounds::DEFAULT)
        };
        let honest = of(Fault::None);
        assert!(
            honest.passed && honest.measured == Some(0.0),
            "{}",
            honest.detail
        );
        assert!(honest.detail.starts_with("6 values"), "{}", honest.detail);
        // The mean basis alone: the true values are the catalogue's.
        let mean = of(Fault::MeanOnly);
        assert!(
            mean.passed && mean.detail.starts_with("3 values"),
            "{}",
            mean.detail
        );
        let failed = of(Fault::AyanamshaFails);
        assert!(
            !failed.passed && failed.detail.contains("declared, but failed"),
            "{}",
            failed.detail
        );
    }

    #[test]
    fn topocentric_positions_are_held_to_the_sdks_correction() {
        let jds = jds();
        let of = |fault| {
            let probe = Probe(fault);
            check_topocentric(&probe, &probe.capabilities(), &jds, &Bounds::DEFAULT)
        };
        let honest = of(Fault::None);
        assert!(honest.passed && honest.ran(), "{}", honest.detail);
        assert!(
            honest.measured.is_some_and(|m| m < 1e-6),
            "{}",
            honest.detail
        );
        // From the centre of the Earth the Moon is a parallax away: the
        // check sees it, past its bound.
        let geocentric = of(Fault::Geocentric);
        assert!(!geocentric.passed, "{}", geocentric.detail);
        let bound = Bounds::DEFAULT.topocentric_arcsec;
        assert!(
            geocentric.measured.is_some_and(|m| m > bound),
            "{}",
            geocentric.detail
        );
        let failed = of(Fault::PositionsFail);
        assert!(
            !failed.passed && failed.detail.contains("declared, but failed"),
            "{}",
            failed.detail
        );
        assert!(!of(Fault::NoLuminaries).ran());
        let plain = TestProvider::new();
        assert!(!check_topocentric(&plain, &plain.capabilities(), &jds, &Bounds::DEFAULT).ran());
    }

    #[test]
    fn a_provider_is_held_to_a_reference_both_ways() {
        let plain = TestProvider::new();
        let same = check_reference(&Probe(Fault::None), &plain, &Bounds::DEFAULT);
        assert!(same.passed && same.measured == Some(0.0), "{}", same.detail);
        // A hundredth of a degree is 36 arcseconds, past the bound.
        let east = check_reference(&Probe(Fault::East), &plain, &Bounds::DEFAULT);
        assert!(!east.passed, "{}", east.detail);
        assert!(
            east.measured.is_some_and(|m| (m - 36.0).abs() < 1e-6),
            "{}",
            east.detail
        );
        let failed = check_reference(&Probe(Fault::PositionsFail), &plain, &Bounds::DEFAULT);
        assert!(
            failed.detail.contains("the provider failed"),
            "{}",
            failed.detail
        );
        let reference = check_reference(&plain, &Probe(Fault::PositionsFail), &Bounds::DEFAULT);
        assert!(
            reference.detail.contains("the reference failed"),
            "{}",
            reference.detail
        );
        // As a trait object too.
        let dynamic: &dyn EphemerisProvider = &plain;
        assert!(check_reference(dynamic, dynamic, &Bounds::DEFAULT).passed);
    }

    #[test]
    fn sidereal_crossings_are_held_ingress_by_ingress() {
        let jds = jds();
        let of = |fault| {
            let probe = Probe(fault);
            check_sidereal_crossings(&probe, &probe.capabilities(), &jds, &Bounds::DEFAULT)
        };
        let honest = of(Fault::None);
        assert!(
            honest.passed && honest.measured == Some(0.0),
            "{}",
            honest.detail
        );
        assert!(
            honest.detail.contains("ingresses of the Sun"),
            "{}",
            honest.detail
        );
        assert!(!of(Fault::CrossingsRefused).ran());
        for (fault, says) in [
            (Fault::CrossingsFail, "declared, but failed"),
            (
                Fault::CrossingDropped,
                "sidereal ingresses where the SDK's kernel found",
            ),
            (Fault::CrossingMoved, "where the SDK's kernel crossed"),
        ] {
            let check = of(fault);
            assert!(
                !check.passed && check.detail.contains(says),
                "{fault:?}: {}",
                check.detail
            );
        }
        let late = of(Fault::CrossingsLate);
        assert!(
            !late.passed && late.measured.is_some_and(|m| (m - 600.0).abs() < 1e-3),
            "{}",
            late.detail
        );
        assert!(!of(Fault::NoLuminaries).ran());
        assert!(
            !check_sidereal_crossings(
                &Probe(Fault::None),
                &Probe(Fault::None).capabilities(),
                &[],
                &Bounds::DEFAULT
            )
            .ran()
        );
    }
}
