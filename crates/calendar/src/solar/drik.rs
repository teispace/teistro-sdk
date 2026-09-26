//! Modern astronomy (drik) as a solar model: the Sun's apparent sidereal
//! longitude through the ephemeris port and the frame completion, and the
//! day's arc from the rise and set solver under a sunrise convention
//! (`docs/03-design/calendar-bikram-sambat.md`, §2). The classical model
//! and this one answer the same trait, so the engine, the sankranti finder
//! and the month-start rules run unchanged over either, and the source
//! memo's comparison of the two comes from the SDK's own code.

use core::sync::atomic::{AtomicBool, Ordering};

use teistro_astro::rise_set::Solver;
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::catalogue::Ayanamsha;
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_core::settings::{OverridePolicy, SunriseConvention};
use teistro_port_ephemeris::{
    Body, EphemerisProvider, Frame, Horizon, HorizonEventKind, HorizonRequest, Overrides,
    PositionRequest, ProviderError, TimeScale, Zodiac,
};

use crate::fixed::FixedDay;
use crate::solar::{DayArc, DayLight, SolarModel, local_mean_midnight};

/// The drik Sun over a provider: positions through the port, the zodiac
/// by a catalogued ayanamsha (the provider's own value under the override
/// policy, or the SDK's own catalogue), the day's arc from the SDK's
/// solver under the profile's sunrise convention.
///
/// ```
/// use teistro_astro::DeltaTModel;
/// use teistro_calendar::solar::{DrikSun, SolarModel};
/// use teistro_core::catalogue::Ayanamsha;
/// use teistro_core::settings::{OverridePolicy, Sunrise};
/// use teistro_port_ephemeris::TestProvider;
///
/// let provider = TestProvider::new();
/// let drik = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, DeltaTModel::TableThenModel);
/// assert!(drik.describe().starts_with("drik"));
/// // The test provider has no ayanamsha of its own, so the SDK's catalogue
/// // supplies Lahiri's: the sidereal Sun at J2000.0 is the tropical one less 23.86°.
/// let sidereal = drik.sidereal_sun_deg(2_451_545.0).unwrap();
/// assert!((0.0..360.0).contains(&sidereal));
/// ```
pub struct DrikSun<'p, P: EphemerisProvider + ?Sized> {
    completion: Completion<'p, P>,
    ayanamsha: Ayanamsha,
    convention: SunriseConvention,
    horizon: Horizon,
    delta_t: DeltaTModel,
    /// Whether the provider refused this model's convention, so its days
    /// fall back to the solver: the convention is fixed, so one refusal
    /// is every day's, and the model stops claiming the provider's day.
    refused: AtomicBool,
}

impl<P: EphemerisProvider + ?Sized> core::fmt::Debug for DrikSun<'_, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&SolarModel::describe(self))
    }
}

impl<'p, P: EphemerisProvider + ?Sized> DrikSun<'p, P> {
    /// The model over a provider.
    pub fn new(
        provider: &'p P,
        ayanamsha: Ayanamsha,
        convention: SunriseConvention,
        policy: OverridePolicy,
        delta_t: DeltaTModel,
    ) -> DrikSun<'p, P> {
        DrikSun {
            completion: Completion::new(provider, policy, delta_t),
            ayanamsha,
            convention,
            horizon: Horizon::from_convention(convention),
            delta_t,
            refused: AtomicBool::new(false),
        }
    }

    /// The frame completion the model reads positions through.
    #[must_use]
    pub const fn completion(&self) -> &Completion<'p, P> {
        &self.completion
    }

    /// The ayanamsha.
    #[must_use]
    pub const fn ayanamsha(&self) -> Ayanamsha {
        self.ayanamsha
    }

    /// Whether the provider **defines** the day's sunrise, which a
    /// classical astronomy does (`Completion::defines`): the text's
    /// sunrise is its own reckoning, with no SDK answer to agree with
    /// (`03-design/classical-chart.md` §4).
    fn provider_defines_sunrise(&self) -> Result<bool, Error> {
        Ok(self.completion.defines(Overrides::RISE_SET, "rise_set")?)
    }

    /// The day's arc from the provider's own sunrise and sunset, where it
    /// defines them and answers this convention; `None` where the SDK's
    /// solver answers instead.
    ///
    /// A convention the provider refuses falls back to the solver under
    /// `prefer-native`, as an ayanamsha it does not list does, and is
    /// refused under `native-only`. A day with no sunrise falls back too,
    /// so the solver says which polar state it is: the port answers an
    /// event or its absence, not whether the Sun stayed up.
    fn defined_day(
        &self,
        local_midnight: JulianDay<Ut1>,
        place: &Place,
    ) -> Result<Option<DayLight>, Error> {
        if !self.provider_defines_sunrise()? {
            return Ok(None);
        }
        let native_only = self.completion.policy() == OverridePolicy::NativeOnly;
        let ask = |kind: HorizonEventKind, from: JulianDay<Ut1>| match self
            .completion
            .provider()
            .horizon_event(&HorizonRequest {
                body: Body::Sun,
                kind,
                place: *place,
                from,
                window_days: 1.0,
                horizon: self.horizon,
            }) {
            Ok(found) => Ok(found),
            Err(ProviderError::Unsupported { .. }) if !native_only => {
                self.refused.store(true, Ordering::Relaxed);
                Ok(None)
            }
            Err(error) => Err(Error::from(error)),
        };
        let Some(sunrise) = ask(HorizonEventKind::Rise, local_midnight)? else {
            return Ok(None);
        };
        let Some(sunset) = ask(HorizonEventKind::Set, sunrise)? else {
            return Ok(None);
        };
        Ok(Some(DayLight::Arc(DayArc {
            sunrise: sunrise.relabel(),
            sunset: sunset.relabel(),
        })))
    }
}

impl<P: EphemerisProvider + ?Sized> SolarModel for DrikSun<'_, P> {
    fn sidereal_sun_deg(&self, jd_ut: f64) -> Result<f64, Error> {
        let jds = [jd_ut];
        let bodies = [Body::Sun];
        let request = PositionRequest::new(
            &jds,
            TimeScale::Ut1,
            &bodies,
            Frame::CANONICAL.with_zodiac(Zodiac::sidereal(self.ayanamsha)),
        )
        .without_speeds();
        let done = self.completion.positions(&request)?;
        let cell = done
            .columns
            .at(0, 0)
            .ok_or_else(|| Error::internal("a one-cell grid has a cell"))?;
        if !cell.is_ok() {
            return Err(Error::new(
                Status::Provider,
                format!(
                    "the Sun at JD {jd_ut}: the provider answered {:?}",
                    cell.status
                ),
            )
            .with_field("jd"));
        }
        Ok(cell.lon)
    }

    fn day_light(&self, day: FixedDay, place: &Place) -> Result<DayLight, Error> {
        // A civil day at a place is its local-mean-time day, as for the
        // classical model, so the two answer the same question.
        let local_midnight = local_mean_midnight(day, place)?;
        if let Some(light) = self.defined_day(local_midnight, place)? {
            return Ok(light);
        }
        let solver = Solver::new(
            &self.completion,
            Body::Sun,
            *place,
            self.horizon,
            self.delta_t,
        );
        let events = solver.day(local_midnight)?;
        Ok(match events.arc() {
            Some((sunrise, sunset)) => DayLight::Arc(DayArc {
                sunrise: sunrise.relabel(),
                sunset: sunset.relabel(),
            }),
            None if events.above_at_midday => DayLight::AlwaysUp,
            None => DayLight::NeverUp,
        })
    }

    fn describe(&self) -> String {
        format!(
            "drik: {} through the port ({} overrides), ayanamsha {}, sunrise {}{}",
            self.completion.capabilities().identity,
            self.completion.policy().key(),
            self.ayanamsha.key(),
            self.horizon,
            if SolarModel::defines_sunrise(self) {
                " by the provider"
            } else {
                ""
            }
        )
    }

    fn convention(&self) -> SunriseConvention {
        self.convention
    }

    /// Whether the provider gives this model's days: it defines the
    /// sunrise, and it has not refused this model's convention.
    fn defines_sunrise(&self) -> bool {
        !self.refused.load(Ordering::Relaxed) && self.provider_defines_sunrise().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::quantity::{Altitude, Latitude, Longitude};
    use teistro_core::settings::Sunrise;
    use teistro_port_ephemeris::{
        Capabilities, Overrides, PositionColumns, ProviderError, TestProvider,
    };

    use super::*;
    use crate::CalendarSystem;
    use crate::gregorian::Gregorian;
    use crate::solar::find_sankranti;

    /// The test provider with a constant ayanamsha of 24°, so a sidereal
    /// request is answered.
    struct WithAyanamsha;

    impl EphemerisProvider for WithAyanamsha {
        fn capabilities(&self) -> Capabilities {
            Capabilities {
                overrides: Overrides::AYANAMSHA,
                ayanamshas: vec![Ayanamsha::Lahiri],
                ..TestProvider::new().capabilities()
            }
        }

        fn positions(
            &self,
            request: &PositionRequest<'_>,
        ) -> Result<PositionColumns, ProviderError> {
            TestProvider::new().positions(request)
        }

        fn ayanamsha_deg(
            &self,
            _jd: f64,
            _scale: TimeScale,
            _ayanamsha: Ayanamsha,
        ) -> Result<f64, ProviderError> {
            Ok(24.0)
        }
    }

    /// A classical provider **defines** the day's sunrise, so the drik
    /// model over it is the text's day to the last bit under
    /// `prefer-native`; `sdk-only` asks the SDK's solver instead; a
    /// convention the text refuses falls back under `prefer-native` and
    /// is refused under `native-only`.
    #[test]
    fn a_classical_provider_gives_the_day_its_own_sunrise() {
        use teistro_siddhanta::{SiddhantaProvider, SuryaSiddhanta};

        let provider = SiddhantaProvider::text();
        let kathmandu = Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        );
        let day = Gregorian.to_fixed_ymd(2024, 4, 13).unwrap();
        let drik = |sunrise: Sunrise, policy: OverridePolicy| {
            DrikSun::new(
                &provider,
                Ayanamsha::Suryasiddhanta,
                sunrise.into(),
                policy,
                DeltaTModel::TableThenModel,
            )
        };
        let text = SuryaSiddhanta::text().day_light(day, &kathmandu).unwrap();
        let native = drik(Sunrise::CentreNoRefraction, OverridePolicy::PreferNative);
        assert_eq!(native.day_light(day, &kathmandu).unwrap(), text);
        assert!(native.describe().ends_with("by the provider"));

        let sdk = drik(Sunrise::CentreNoRefraction, OverridePolicy::SdkOnly);
        assert_ne!(sdk.day_light(day, &kathmandu).unwrap(), text);
        assert!(!sdk.describe().ends_with("by the provider"));

        let refracted = drik(Sunrise::UpperLimbRefraction, OverridePolicy::PreferNative);
        let fallen_back = refracted.day_light(day, &kathmandu).unwrap();
        assert!(fallen_back.arc().is_some());
        assert_ne!(fallen_back, text);
        assert!(
            !refracted.defines_sunrise(),
            "a refused convention is not the text's day"
        );
        assert!(native.defines_sunrise());
        let refused = drik(Sunrise::UpperLimbRefraction, OverridePolicy::NativeOnly)
            .day_light(day, &kathmandu)
            .unwrap_err();
        assert_eq!(refused.status, Status::Unsupported, "{refused}");
    }

    #[test]
    fn the_drik_model_answers_the_trait_over_a_provider() {
        let provider = WithAyanamsha;
        let drik = DrikSun::new(
            &provider,
            Ayanamsha::Lahiri,
            Sunrise::UpperLimbRefraction.into(),
            OverridePolicy::PreferNative,
            DeltaTModel::TableThenModel,
        );
        let model: &dyn SolarModel = &drik;
        let tropical = TestProvider::new()
            .positions(&PositionRequest::new(
                &[2_451_545.0],
                TimeScale::Ut1,
                &[Body::Sun],
                Frame::CANONICAL,
            ))
            .unwrap()
            .at(0, 0)
            .unwrap()
            .lon;
        let sidereal = model.sidereal_sun_deg(2_451_545.0).unwrap();
        assert!(
            teistro_core::angle::difference_deg(tropical, sidereal) - 24.0 < 1e-9,
            "{tropical} {sidereal}"
        );
        assert_eq!(model.convention(), Sunrise::UpperLimbRefraction.into());
        assert!(model.describe().contains("LAHIRI") && model.describe().contains("UPPER_LIMB"));
        assert_eq!(drik.ayanamsha(), Ayanamsha::Lahiri);
        assert!(format!("{drik:?}").starts_with("drik"));
        // A sankranti is found through the same finder as the classical model.
        let from = Gregorian
            .to_fixed_ymd(2024, 3, 1)
            .unwrap()
            .jd_at_midnight()
            .unwrap();
        let mesha = find_sankranti(model, 0, from).unwrap();
        let sun = model.sidereal_sun_deg(mesha.instant.get()).unwrap();
        assert!(sun < 1e-3 || sun > 360.0 - 1e-3, "{sun}");
        // The day's arc comes from the solver: a Kathmandu June day.
        let kathmandu = Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        );
        let day = Gregorian.to_fixed_ymd(2024, 6, 21).unwrap();
        let arc = model.day_arc(day, &kathmandu).unwrap().unwrap();
        let hours = (arc.sunset.get() - arc.sunrise.get()) * 24.0;
        assert!(hours > 12.0 && hours < 15.0, "{hours}");
        let tromso = Place::new(
            Latitude::literal(69.6),
            Longitude::literal(18.9),
            Altitude::literal(0.0),
        );
        assert!(matches!(
            model.day_light(day, &tromso).unwrap(),
            DayLight::AlwaysUp | DayLight::Arc(_)
        ));
        assert!(drik.completion().capabilities().has(Overrides::AYANAMSHA));
    }
}
