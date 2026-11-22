//! The text's sky with modern parts: the Surya Siddhanta for every
//! place, angle and zodiac it is asked for, and from a modern ephemeris
//! the sunrise, the star planets, or both.
//!
//! Nepal's daily panchanga prints the text's limbs to the minute and a
//! sunrise the text does not give: the text's has no equation of time
//! (III.34 to 35, C37), and the print's is within 1.7 minutes of a modern
//! one, the disc's centre with standard refraction (C39,
//! `docs/03-design/nepal-day-measured.md`). A day's flags are
//! decided by where an end falls against the sunrises, so a limb ending
//! within minutes of sunrise is flagged by the print's sunrise and not by
//! the text's. Its star planets and node are not the text's either: a
//! modern sky in Lahiri's zodiac misses them by 9′ to 33′ over 49
//! printed mornings, the text by 94′ to 459′ (C38,
//! `docs/03-design/committee-star-planets.md`). This provider answers
//! each part the way the print does, as one sky, so every reader of a
//! sunrise or a planet takes the same one.

use teistro_astro::ayanamsha;
use teistro_astro::precession::PrecessionModel;
use teistro_astro::rise_set::Solver;
use teistro_astro::scale::tt_of;
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::angle::normalise_deg;
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Tt, Ut1};
use teistro_core::settings::{AyanamshaBasis, AyanamshaChoice, OverridePolicy};
use teistro_port_ephemeris::{
    Angles, AnglesRequest, Body, Capabilities, Conventions, DeltaTEstimate, EphemerisProvider,
    Frame, HorizonRequest, Obliquity, Overrides, PositionColumns, PositionRequest, ProviderError,
    TimeScale,
};

use crate::provider::SiddhantaProvider;

/// How far inside the modern ephemeris's coverage a sky taking parts of
/// it ends, days: an hour, more than the spread of every Delta T model the
/// SDK offers at either end of six centuries.
const MODERN_SLACK_DAYS: f64 = 1.0 / 24.0;

/// The Surya Siddhanta's sky with the parts asked for taken from a modern
/// ephemeris.
///
/// The obliquity, the ayanamsha, the angles and every place not asked
/// for are the text's, exactly as [`SiddhantaProvider`] gives them.
/// [`ModernParts::with_sunrise`] makes a rise or set the SDK's solver over
/// the modern ephemeris, under whatever convention the request names, so
/// a chart's `day.sunrise` applies to it as it would over a modern sky.
/// [`ModernParts::with_planets`] makes Mars, Mercury, Jupiter, Venus,
/// Saturn and the mean node the modern ephemeris's apparent geocentric
/// places less the named ayanamsha's mean value, in the text's frame.
///
/// ```
/// use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// use teistro_port_ephemeris::{
///     Body, Conventions, EphemerisProvider, Horizon, HorizonEventKind, HorizonRequest, TestProvider,
/// };
/// use teistro_siddhanta::{ModernParts, SiddhantaProvider};
///
/// let sky = ModernParts::new(SiddhantaProvider::text(), Box::new(TestProvider::new())).with_sunrise();
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
///     conventions: Conventions::STANDARD,
/// })?;
/// assert!(rise.is_some());
/// // And the stamp names both.
/// assert!(sky.capabilities().identity.data_version.contains("sunrise from"));
/// # Ok::<(), teistro_port_ephemeris::ProviderError>(())
/// ```
pub struct ModernParts {
    text: SiddhantaProvider,
    modern: Box<dyn EphemerisProvider>,
    sunrise: bool,
    planets: Option<AyanamshaChoice>,
    basis: AyanamshaBasis,
}

impl core::fmt::Debug for ModernParts {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ModernParts")
            .field("text", &self.text)
            .field("modern", &self.modern.capabilities().identity.name)
            .field("sunrise", &self.sunrise)
            .field("planets", &self.planets)
            .field("basis", &self.basis)
            .finish()
    }
}

impl ModernParts {
    /// The bodies [`ModernParts::with_planets`] takes from the modern
    /// ephemeris.
    ///
    /// ```
    /// use teistro_port_ephemeris::Body;
    /// use teistro_siddhanta::ModernParts;
    ///
    /// // The Sun and the Moon stay the text's.
    /// assert!(!ModernParts::PLANETS.contains(&Body::Sun) && !ModernParts::PLANETS.contains(&Body::Moon));
    /// ```
    pub const PLANETS: [Body; 6] = [
        Body::Mars,
        Body::Mercury,
        Body::Jupiter,
        Body::Venus,
        Body::Saturn,
        Body::MeanNode,
    ];

    /// The text's sky beside a modern ephemeris, taking nothing from it
    /// until a part is asked for.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    /// use teistro_siddhanta::{ModernParts, SiddhantaProvider};
    ///
    /// // Nothing taken yet: the stamp is the text's own.
    /// let sky = ModernParts::new(SiddhantaProvider::text(), Box::new(TestProvider::new()));
    /// let text = SiddhantaProvider::text().capabilities().identity.data_version;
    /// assert_eq!(sky.capabilities().identity.data_version, text);
    /// ```
    #[must_use]
    pub fn new(text: SiddhantaProvider, modern: Box<dyn EphemerisProvider>) -> ModernParts {
        ModernParts {
            text,
            modern,
            sunrise: false,
            planets: None,
            basis: AyanamshaBasis::True,
        }
    }

    /// The same, with the rise and set the modern ephemeris gives.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    /// use teistro_siddhanta::{ModernParts, SiddhantaProvider};
    ///
    /// let sky = ModernParts::new(SiddhantaProvider::text(), Box::new(TestProvider::new())).with_sunrise();
    /// assert!(sky.capabilities().identity.data_version.contains("sunrise from"));
    /// ```
    #[must_use]
    pub fn with_sunrise(self) -> ModernParts {
        ModernParts {
            sunrise: true,
            ..self
        }
    }

    /// The same, with [`ModernParts::PLANETS`] from the modern ephemeris
    /// in the zodiac of `ayanamsha`.
    ///
    /// ```
    /// use teistro_core::catalogue::Ayanamsha;
    /// use teistro_core::settings::AyanamshaChoice;
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    /// use teistro_siddhanta::{ModernParts, SiddhantaProvider};
    ///
    /// let lahiri = AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri };
    /// let sky = ModernParts::new(SiddhantaProvider::text(), Box::new(TestProvider::new())).with_planets(lahiri);
    /// let text = SiddhantaProvider::text().capabilities().identity.data_version;
    /// assert_ne!(sky.capabilities().identity.data_version, text);
    /// ```
    #[must_use]
    pub fn with_planets(self, ayanamsha: AyanamshaChoice) -> ModernParts {
        ModernParts {
            planets: Some(ayanamsha),
            ..self
        }
    }

    /// The same, with the star planets' ayanamsha on `basis`: the true
    /// one until this says otherwise, since their modern places are
    /// apparent, of the true equinox of date, and a nirayana longitude is
    /// the apparent one less the true ayanamsha (SR-100, crux C1750).
    ///
    /// ```
    /// use teistro_core::settings::AyanamshaBasis;
    /// use teistro_port_ephemeris::TestProvider;
    /// use teistro_siddhanta::{ModernParts, SiddhantaProvider};
    ///
    /// let sky = ModernParts::new(SiddhantaProvider::text(), Box::new(TestProvider::new()));
    /// assert!(format!("{sky:?}").contains("basis: True"));
    /// let mean = sky.with_ayanamsha_basis(AyanamshaBasis::Mean);
    /// assert!(format!("{mean:?}").contains("basis: Mean"));
    /// ```
    #[must_use]
    pub fn with_ayanamsha_basis(self, basis: AyanamshaBasis) -> ModernParts {
        ModernParts { basis, ..self }
    }

    /// The text this sky's places come from.
    ///
    /// ```
    /// use teistro_port_ephemeris::TestProvider;
    /// use teistro_siddhanta::{ModernParts, SiddhantaProvider, SuryaSiddhanta};
    ///
    /// let sky = ModernParts::new(SiddhantaProvider::text(), Box::new(TestProvider::new()));
    /// assert_eq!(sky.text().model(), &SuryaSiddhanta::text());
    /// ```
    #[must_use]
    pub const fn text(&self) -> &SiddhantaProvider {
        &self.text
    }

    /// The ephemeris this sky's modern parts come from.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    /// use teistro_siddhanta::{ModernParts, SiddhantaProvider};
    ///
    /// let sky = ModernParts::new(SiddhantaProvider::text(), Box::new(TestProvider::new()));
    /// let name = TestProvider::new().capabilities().identity.name;
    /// assert_eq!(sky.modern().capabilities().identity.name, name);
    /// ```
    #[must_use]
    pub fn modern(&self) -> &dyn EphemerisProvider {
        self.modern.as_ref()
    }

    /// The modern places of the bodies `wanted` at the request's
    /// instants, as the text's frame holds them: apparent and geocentric,
    /// less the ayanamsha on this sky's basis and its rate.
    fn modern_places(
        &self,
        request: &PositionRequest<'_>,
        wanted: &[Body],
        ayanamsha: &AyanamshaChoice,
    ) -> Result<PositionColumns, ProviderError> {
        let conventions = request.conventions;
        let sky = self.completion(conventions);
        let precession =
            PrecessionModel::from_knob(conventions.models.precession).ok_or_else(|| {
                ProviderError::unsupported(format!(
                    "the {} precession in the SDK's ayanamsha",
                    conventions.models.precession.key()
                ))
            })?;
        let modern = PositionRequest {
            bodies: wanted,
            frame: Frame::CANONICAL,
            observer: None,
            ..*request
        };
        let mut columns = sky
            .positions(&modern)
            .map_err(|error| refused_at(&Error::from(error), request.jds.first().copied()))?
            .columns;
        for (jd_index, jd) in request.jds.iter().enumerate() {
            let at = ModernParts::tt_at(*jd, request.scale, conventions.delta_t)?;
            let offset =
                ayanamsha::value_deg(ayanamsha, at, self.basis, precession, conventions.delta_t)
                    .map_err(|error| refused_at(&error, Some(*jd)))?;
            let rate = if request.speeds {
                ayanamsha::speed_deg_per_day(ayanamsha, at, precession, conventions.delta_t)
                    .map_err(|error| refused_at(&error, Some(*jd)))?
            } else {
                0.0
            };
            for body_index in 0..wanted.len() {
                if let Some(mut cell) = columns.at(jd_index, body_index) {
                    cell.lon = normalise_deg(cell.lon - offset);
                    if request.speeds {
                        cell.lon_speed -= rate;
                    }
                    columns.set_at(jd_index, body_index, cell);
                }
            }
        }
        Ok(columns)
    }

    /// The SDK's completion over the modern ephemeris, under a request's
    /// conventions: its Delta T and its models.
    fn completion(&self, conventions: Conventions) -> Completion<'_, dyn EphemerisProvider> {
        Completion::new(
            self.modern.as_ref(),
            OverridePolicy::PreferNative,
            conventions.delta_t,
        )
        .with_models(conventions.models)
    }

    /// An instant of the request on TT, through the request's Delta T.
    fn tt_at(
        jd: f64,
        scale: TimeScale,
        model: DeltaTModel,
    ) -> Result<JulianDay<Tt>, ProviderError> {
        let invalid = |error: &dyn core::fmt::Display| ProviderError::invalid(error.to_string());
        match scale {
            TimeScale::Tt => JulianDay::try_new(jd).map_err(|e| invalid(&e)),
            TimeScale::Ut1 => {
                let ut1 = JulianDay::<Ut1>::try_new(jd).map_err(|e| invalid(&e))?;
                tt_of(ut1, model).map(|(tt, _)| tt).map_err(|e| invalid(&e))
            }
        }
    }
}

/// An SDK error as the port says it, keeping its kind: the solver's
/// refusal is the provider's.
fn refused(error: &Error, at: JulianDay<Ut1>) -> ProviderError {
    refused_at(error, Some(at.get()))
}

/// As [`refused`], at an instant on any scale, if one is known.
fn refused_at(error: &Error, jd: Option<f64>) -> ProviderError {
    match (error.status, jd) {
        (Status::Unsupported, _) => ProviderError::unsupported(error.message.clone()),
        (Status::InvalidArg, _) => ProviderError::invalid(error.message.clone()),
        (Status::OutOfRange, Some(jd)) => ProviderError::OutOfRange { jd },
        _ => ProviderError::Refused {
            detail: error.to_string(),
        },
    }
}

impl EphemerisProvider for ModernParts {
    fn capabilities(&self) -> Capabilities {
        let mut capabilities = self.text.capabilities();
        let modern = self.modern.capabilities().identity;
        let from = format!("{} {}", modern.name, modern.data_version);
        if self.sunrise {
            capabilities.identity.data_version = format!(
                "{}, sunrise from {from}",
                capabilities.identity.data_version
            );
        }
        if let Some(ayanamsha) = self.planets {
            capabilities.identity.data_version = format!(
                "{}, star planets from {from} in {}",
                capabilities.identity.data_version,
                zodiac_name(&ayanamsha)
            );
        }
        // A part taken from the modern ephemeris is answered only where
        // it is (SR-136, C2245): the sky's coverage is the text's cut to
        // the modern one's, carried to UT1 (this sky is asked on UT1, a
        // classical astronomy's scale) and an hour inside it, so that no
        // Delta T model's minutes between the scales reach past it.
        if self.sunrise || self.planets.is_some() {
            let modern = Completion::new(
                self.modern.as_ref(),
                OverridePolicy::PreferNative,
                DeltaTModel::default(),
            )
            .coverage();
            let slack = if modern.is_bounded() {
                MODERN_SLACK_DAYS
            } else {
                0.0
            };
            let (from, to) = capabilities.jd_range;
            capabilities.jd_range = (from.max(modern.from + slack), to.min(modern.to - slack));
        }
        // The text's declaration of the rise and set stands: this sky
        // answers it, for any convention rather than only the text's.
        debug_assert!(capabilities.has(Overrides::RISE_SET));
        capabilities
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        let mut columns = self.text.positions(request)?;
        let Some(ayanamsha) = self.planets else {
            return Ok(columns);
        };
        let wanted: Vec<(usize, Body)> = request
            .bodies
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, body)| ModernParts::PLANETS.contains(body))
            .collect();
        if wanted.is_empty() {
            return Ok(columns);
        }
        let bodies: Vec<Body> = wanted.iter().map(|(_, body)| *body).collect();
        let modern = self.modern_places(request, &bodies, &ayanamsha)?;
        for jd_index in 0..request.jds.len() {
            for (from, (into, _)) in wanted.iter().enumerate() {
                if let Some(cell) = modern.at(jd_index, from) {
                    columns.set_at(jd_index, *into, cell);
                }
            }
        }
        Ok(columns)
    }

    fn obliquity(
        &self,
        jd: f64,
        scale: TimeScale,
        conventions: Conventions,
    ) -> Result<Obliquity, ProviderError> {
        self.text.obliquity(jd, scale, conventions)
    }

    fn delta_t(&self, jd_ut1: f64, model: DeltaTModel) -> Result<DeltaTEstimate, ProviderError> {
        self.text.delta_t(jd_ut1, model)
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: teistro_core::catalogue::Ayanamsha,
        basis: teistro_core::settings::AyanamshaBasis,
        conventions: Conventions,
    ) -> Result<f64, ProviderError> {
        self.text
            .ayanamsha_deg(jd, scale, ayanamsha, basis, conventions)
    }

    fn angles(&self, request: &AnglesRequest) -> Result<Angles, ProviderError> {
        self.text.angles(request)
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        if !self.sunrise {
            return self.text.horizon_event(request);
        }
        if request.body != Body::Sun {
            return Err(ProviderError::unsupported(format!(
                "the rise and set of {}; this sky gives the Sun's",
                request.body.key()
            )));
        }
        // The route a modern sky's sunrise takes in the almanac: the SDK's
        // solver over the modern ephemeris's apparent Sun, completed by the
        // SDK, or the engine's own search where it declares one and the
        // policy lets it answer (boundary audit, rank 6), as there.
        let sky = self.completion(request.conventions);
        let solver = Solver::new(
            &sky,
            Body::Sun,
            request.place,
            request.horizon,
            request.conventions.delta_t,
        );
        solver
            .event(request.kind, request.from, request.window_days)
            .map(|found| found.map(|event| event.instant))
            .map_err(|error| refused(&error, request.from))
    }
}

/// An ayanamsha as a stamp names it.
fn zodiac_name(choice: &AyanamshaChoice) -> String {
    choice.describe()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::float_cmp,
        reason = "tests fail by panicking, and compare a cell passed through"
    )]

    use teistro_core::catalogue::Ayanamsha;
    use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
    use teistro_port_ephemeris::{Horizon, HorizonEventKind, TestProvider};

    use super::*;

    fn parts() -> ModernParts {
        ModernParts::new(SiddhantaProvider::text(), Box::new(TestProvider::new()))
    }

    fn sunrise_of(
        sky: &dyn EphemerisProvider,
        body: Body,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        sky.horizon_event(&HorizonRequest {
            body,
            kind: HorizonEventKind::Rise,
            place: Place::new(
                Latitude::literal(27.7172),
                Longitude::literal(85.324),
                Altitude::literal(1400.0),
            ),
            from: JulianDay::literal(2_460_413.5),
            window_days: 1.0,
            horizon: Horizon::CENTRE_NO_REFRACTION,
            conventions: Conventions::STANDARD,
        })
    }

    /// Without a modern part the sky is the text's: its identity, its span
    /// and its sunrise.
    #[test]
    fn a_sky_with_no_modern_part_is_the_texts() {
        let sky = parts();
        let text = SiddhantaProvider::text();
        assert_eq!(sky.capabilities().identity, text.capabilities().identity);
        assert_eq!(sky.capabilities().jd_range, text.capabilities().jd_range);
        assert_eq!(
            sunrise_of(&sky, Body::Sun).unwrap(),
            sunrise_of(&text, Body::Sun).unwrap()
        );
        assert_eq!(
            sky.text().capabilities().identity,
            text.capabilities().identity
        );
        assert_eq!(
            sky.modern().capabilities().identity,
            TestProvider::new().capabilities().identity
        );
    }

    /// The modern sunrise is the Sun's: another body's rise is refused by
    /// name rather than taken from the text.
    #[test]
    fn a_modern_sunrise_is_the_suns_alone() {
        let sky = parts().with_sunrise();
        assert!(
            sky.capabilities()
                .identity
                .data_version
                .contains("sunrise from")
        );
        assert!(sunrise_of(&sky, Body::Sun).unwrap().is_some());
        let refused = sunrise_of(&sky, Body::Moon).unwrap_err();
        assert!(
            matches!(refused, ProviderError::Unsupported { .. }),
            "{refused:?}"
        );
    }

    /// The star planets come from the modern ephemeris and the Sun and the
    /// Moon from the text, with or without speeds.
    #[test]
    fn the_star_planets_alone_are_modern() {
        let lahiri = AyanamshaChoice::Catalogued {
            id: Ayanamsha::Lahiri,
        };
        let sky = parts().with_planets(lahiri);
        let version = sky.capabilities().identity.data_version;
        assert!(
            version.contains("star planets from") && !version.contains("sunrise"),
            "{version}"
        );
        let text = SiddhantaProvider::text();
        let jds = [2_460_413.5];
        let bodies = [Body::Sun, Body::Mars];
        for speeds in [true, false] {
            let request = PositionRequest {
                speeds,
                ..PositionRequest::new(&jds, TimeScale::Ut1, &bodies, SiddhantaProvider::FRAME)
            };
            let ours = sky.positions(&request).unwrap();
            let theirs = text.positions(&request).unwrap();
            assert_eq!(ours.at(0, 0).unwrap().lon, theirs.at(0, 0).unwrap().lon);
            let mars = ours.at(0, 1).unwrap();
            assert_ne!(mars.lon, theirs.at(0, 1).unwrap().lon);
            assert!((0.0..360.0).contains(&mars.lon));
        }
        // A request with no star planet is the text's answer whole.
        let moon = [Body::Moon];
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &moon, SiddhantaProvider::FRAME);
        assert_eq!(
            sky.positions(&request).unwrap().at(0, 0).unwrap().lon,
            text.positions(&request).unwrap().at(0, 0).unwrap().lon
        );
    }
}
