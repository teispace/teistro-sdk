//! Modern astronomy (drik) as a solar model: the Sun's apparent sidereal
//! longitude through the ephemeris port and the frame completion, and the
//! day's arc from the rise and set solver under a sunrise convention
//! (`docs/03-design/calendar-bikram-sambat.md`, §2). The classical model
//! and this one answer the same trait, so the engine, the sankranti finder
//! and the month-start rules run unchanged over either, and the source
//! memo's comparison of the two comes from the SDK's own code.

use core::sync::atomic::{AtomicBool, Ordering};

use teistro_astro::ayanamsha;
use teistro_astro::lockstep::Lockstep;
use teistro_astro::rise_set::Solver;
use teistro_astro::scale::tt_from_ut1;
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::angle::normalise_deg;
use teistro_core::catalogue::Ayanamsha;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_core::settings::{
    AyanamshaBasis, AyanamshaChoice, HorizonModel, OverridePolicy, SunriseConvention,
};
use teistro_port_ephemeris::{
    Body, EphemerisProvider, Frame, Horizon, HorizonEventKind, HorizonRequest, Models, Overrides,
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
    /// A custom definition the zodiac is measured by instead of the
    /// catalogued member (SR-112): the port's frame names only catalogued
    /// members, so the Sun is asked tropical and the SDK's value taken
    /// from it.
    custom: Option<AyanamshaChoice>,
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
    ///
    /// ```
    /// use teistro_astro::DeltaTModel;
    /// use teistro_calendar::solar::DrikSun;
    /// use teistro_core::catalogue::Ayanamsha;
    /// use teistro_core::settings::{OverridePolicy, Sunrise};
    /// use teistro_port_ephemeris::TestProvider;
    ///
    /// let provider = TestProvider::new();
    /// let drik = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, DeltaTModel::TableThenModel);
    /// use teistro_calendar::solar::SolarModel;
    ///
    /// // Not a classical astronomy: the day's sunrise is the sky's.
    /// assert!(!drik.defines_sunrise());
    /// ```
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
            custom: None,
            convention,
            horizon: Horizon::from_convention(convention),
            delta_t,
            refused: AtomicBool::new(false),
        }
    }

    /// The model with the zodiac a settings document chooses: a catalogued
    /// member takes the place of the one it was built with, and a custom
    /// definition (a linear rate, a precessed value or a star held at a
    /// longitude, SR-112) measures the Sun from the tropical one by the
    /// SDK's value of it, on the model's basis and precession.
    ///
    /// ```
    /// use teistro_astro::DeltaTModel;
    /// use teistro_calendar::solar::{DrikSun, SolarModel};
    /// use teistro_core::catalogue::{Ayanamsha, Star};
    /// use teistro_core::settings::{AyanamshaChoice, OverridePolicy, Sunrise};
    /// use teistro_port_ephemeris::TestProvider;
    ///
    /// let provider = TestProvider::new();
    /// let new = || DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, DeltaTModel::TableThenModel);
    /// let chitra = new().with_ayanamsha_choice(Ayanamsha::TrueChitra.into());
    /// let spica = new().with_ayanamsha_choice(AyanamshaChoice::Anchored { star: Star::Spica, longitude_deg: 180.0 });
    /// assert_eq!(chitra.ayanamsha(), Ayanamsha::TrueChitra);
    /// // Spica held at 180° is True Chitra, by the port's frame or by the SDK's value.
    /// let (a, b) = (chitra.sidereal_sun_deg(2_451_545.0).unwrap(), spica.sidereal_sun_deg(2_451_545.0).unwrap());
    /// assert!((a - b).abs() < 1e-9, "{a} {b}");
    /// assert!(spica.describe().contains("SPICA held at 180°"));
    /// ```
    #[must_use]
    pub fn with_ayanamsha_choice(mut self, choice: AyanamshaChoice) -> DrikSun<'p, P> {
        match choice {
            AyanamshaChoice::Catalogued { id } => {
                self.ayanamsha = id;
                self.custom = None;
            }
            custom => self.custom = Some(custom),
        }
        self
    }

    /// The model with the ayanamsha on another basis: [`AyanamshaBasis::True`]
    /// carries the nutation in longitude, so the sidereal Sun is the
    /// apparent one less the nutated ayanamsha, the Indian Astronomical
    /// Ephemeris's nirayana Sun; [`AyanamshaBasis::Mean`] leaves the
    /// nutation in it. The basis is the true one, the root profile's,
    /// until this says otherwise (SR-100).
    ///
    /// ```
    /// use teistro_astro::DeltaTModel;
    /// use teistro_calendar::solar::{DrikSun, SolarModel};
    /// use teistro_core::catalogue::Ayanamsha;
    /// use teistro_core::settings::{AyanamshaBasis, OverridePolicy, Sunrise};
    /// use teistro_port_ephemeris::TestProvider;
    ///
    /// let provider = TestProvider::new();
    /// let new = || DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// let mean = new().with_ayanamsha_basis(AyanamshaBasis::Mean);
    /// assert_eq!(mean.completion().ayanamsha_basis(), AyanamshaBasis::Mean);
    /// assert!(new().describe().contains("LAHIRI (true basis)"));
    /// assert!(!mean.describe().contains("true basis"));
    /// // The two bases part by the nutation in longitude, under 20″.
    /// let gap = new().sidereal_sun_deg(2_451_545.0).unwrap() - mean.sidereal_sun_deg(2_451_545.0).unwrap();
    /// assert!(gap.abs() > 1e-4 && gap.abs() < 20.0 / 3600.0, "{gap}");
    /// ```
    #[must_use]
    pub fn with_ayanamsha_basis(mut self, basis: AyanamshaBasis) -> DrikSun<'p, P> {
        self.completion = self.completion.with_ayanamsha_basis(basis);
        self
    }

    /// The model under a context's conventions: the engine's models
    /// (`frame.precession`, `frame.nutation`, `frame.frame_bias`,
    /// `time.sidereal_time`) its positions are asked under, and the
    /// horizon model (`day.horizon`) its sunrise is reckoned by.
    ///
    /// ```
    /// use teistro_astro::DeltaTModel;
    /// use teistro_calendar::solar::DrikSun;
    /// use teistro_core::catalogue::Ayanamsha;
    /// use teistro_core::settings::{Dip, HorizonModel, OverridePolicy, Precession, Sunrise};
    /// use teistro_port_ephemeris::{Models, TestProvider};
    ///
    /// let provider = TestProvider::new();
    /// let models = Models { precession: Precession::Iau2006, ..Models::STANDARD };
    /// let dipped = HorizonModel { dip: Dip::Geometric, ..HorizonModel::STANDARD };
    /// let drik = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::UpperLimbRefraction.into(), OverridePolicy::SdkOnly, DeltaTModel::TableThenModel)
    ///     .under(models, dipped);
    /// assert_eq!(drik.completion().conventions().models, models);
    /// ```
    #[must_use]
    pub fn under(mut self, models: Models, horizon: HorizonModel) -> DrikSun<'p, P> {
        self.completion = self.completion.with_models(models);
        self.horizon = self.horizon.with_model(horizon);
        self
    }

    /// The frame completion the model reads positions through.
    ///
    /// ```
    /// use teistro_astro::DeltaTModel;
    /// use teistro_calendar::solar::DrikSun;
    /// use teistro_core::catalogue::Ayanamsha;
    /// use teistro_core::settings::{OverridePolicy, Sunrise};
    /// use teistro_port_ephemeris::TestProvider;
    ///
    /// let provider = TestProvider::new();
    /// let drik = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, DeltaTModel::TableThenModel);
    ///
    /// assert_eq!(drik.completion().policy(), OverridePolicy::PreferNative);
    /// ```
    #[must_use]
    pub const fn completion(&self) -> &Completion<'p, P> {
        &self.completion
    }

    /// The catalogued ayanamsha the port's frame names; a custom zodiac is
    /// [`DrikSun::ayanamsha_choice`].
    ///
    /// ```
    /// use teistro_astro::DeltaTModel;
    /// use teistro_calendar::solar::DrikSun;
    /// use teistro_core::catalogue::Ayanamsha;
    /// use teistro_core::settings::{OverridePolicy, Sunrise};
    /// use teistro_port_ephemeris::TestProvider;
    ///
    /// let provider = TestProvider::new();
    /// let drik = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, DeltaTModel::TableThenModel);
    ///
    /// assert_eq!(drik.ayanamsha(), Ayanamsha::Lahiri);
    /// ```
    #[must_use]
    pub const fn ayanamsha(&self) -> Ayanamsha {
        self.ayanamsha
    }

    /// The zodiac the model measures the Sun in: the catalogued member, or
    /// the custom definition [`DrikSun::with_ayanamsha_choice`] gave it.
    ///
    /// ```
    /// use teistro_astro::DeltaTModel;
    /// use teistro_calendar::solar::DrikSun;
    /// use teistro_core::catalogue::{Ayanamsha, Star};
    /// use teistro_core::settings::{AyanamshaChoice, OverridePolicy, Sunrise};
    /// use teistro_port_ephemeris::TestProvider;
    ///
    /// let provider = TestProvider::new();
    /// let drik = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, DeltaTModel::TableThenModel);
    /// assert_eq!(drik.ayanamsha_choice(), Ayanamsha::Lahiri.into());
    /// let revati = AyanamshaChoice::Anchored { star: Star::Revati, longitude_deg: 359.5 };
    /// assert_eq!(drik.with_ayanamsha_choice(revati).ayanamsha_choice(), revati);
    /// ```
    #[must_use]
    pub fn ayanamsha_choice(&self) -> AyanamshaChoice {
        self.custom.unwrap_or_else(|| self.ayanamsha.into())
    }

    /// Whether the provider **defines** the day's sunrise, which a
    /// classical astronomy does (`Completion::defines`): the text's
    /// sunrise is its own reckoning, with no SDK answer to agree with
    /// (`03-design/classical-chart.md` §4).
    fn provider_defines_sunrise(&self) -> Result<bool, Error> {
        Ok(self.completion.defines(Overrides::RISE_SET, "rise_set")?)
    }

    /// The solver a day's sunrise and sunset are searched by.
    fn solver(&self, place: &Place) -> Solver<'_> {
        Solver::new(
            &self.completion,
            Body::Sun,
            *place,
            self.horizon,
            self.delta_t,
        )
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
                conventions: self.completion.conventions(),
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

/// The light a day's sunrise and sunset searches found.
fn light_of(events: &teistro_astro::rise_set::DayEvents) -> DayLight {
    match events.arc() {
        Some((sunrise, sunset)) => DayLight::Arc(DayArc {
            sunrise: sunrise.relabel(),
            sunset: sunset.relabel(),
        }),
        None if events.above_at_midday => DayLight::AlwaysUp,
        None => DayLight::NeverUp,
    }
}

impl<P: EphemerisProvider + ?Sized> SolarModel for DrikSun<'_, P> {
    fn sidereal_sun_deg(&self, jd_ut: f64) -> Result<f64, Error> {
        let jds = [jd_ut];
        let bodies = [Body::Sun];
        let zodiac = match self.custom {
            Some(_) => Zodiac::Tropical,
            None => Zodiac::sidereal(self.ayanamsha),
        };
        let request = PositionRequest::new(
            &jds,
            TimeScale::Ut1,
            &bodies,
            Frame::CANONICAL.with_zodiac(zodiac),
        )
        .without_speeds();
        let done = self.completion.positions(&request)?;
        let cell = done
            .columns
            .at(0, 0)
            .ok_or_else(|| Error::internal("a one-cell grid has a cell"))?;
        if !cell.is_ok() {
            return Err(Error::new(
                cell.status.status(),
                format!(
                    "the Sun at JD {jd_ut}: the provider answered {:?}",
                    cell.status
                ),
            )
            .with_field("jd"));
        }
        let Some(custom) = self.custom else {
            return Ok(cell.lon);
        };
        // The custom zodiac, as the completion's own sidereal step takes a
        // catalogued one: at the instant in TT from Delta T's owner, on
        // the model's basis, under its precession.
        let ut1 = JulianDay::<Ut1>::try_new(jd_ut)?;
        let (delta_t, _) = self.completion.delta_t_at(ut1)?;
        let offset = ayanamsha::value_deg(
            &custom,
            tt_from_ut1(ut1, &delta_t),
            self.completion.ayanamsha_basis(),
            self.completion.precession(),
            self.delta_t,
        )?;
        Ok(normalise_deg(cell.lon - offset))
    }

    fn day_light(&self, day: FixedDay, place: &Place) -> Result<DayLight, Error> {
        // A civil day at a place is its local-mean-time day, as for the
        // classical model, so the two answer the same question.
        let local_midnight = local_mean_midnight(day, place)?;
        if let Some(light) = self.defined_day(local_midnight, place)? {
            return Ok(light);
        }
        Ok(light_of(&self.solver(place).day(local_midnight)?))
    }

    fn day_lights(&self, days: &[FixedDay], place: &Place) -> Vec<Result<DayLight, Error>> {
        // A provider that defines the sunrise answers each day itself, so
        // there is no search to share.
        if !matches!(self.provider_defines_sunrise(), Ok(false)) {
            return days.iter().map(|day| self.day_light(*day, place)).collect();
        }
        let solver = self.solver(place);
        let round = Lockstep::new();
        round.run(days.iter().map(|day| {
            let solver = &solver;
            let round = &round;
            async move {
                let midnight = local_mean_midnight(*day, place)?;
                Ok(light_of(&solver.day_in(round, midnight).await?))
            }
        }))
    }

    fn describe(&self) -> String {
        format!(
            "drik: {} through the port ({} overrides), ayanamsha {}{}, sunrise {}{}",
            self.completion.capabilities().identity,
            self.completion.policy().key(),
            self.custom.map_or_else(
                || self.ayanamsha.key().to_owned(),
                |custom| custom.describe()
            ),
            if self.completion.ayanamsha_basis() == AyanamshaBasis::True {
                " (true basis)"
            } else {
                ""
            },
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::error::Status;
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
            _basis: teistro_core::settings::AyanamshaBasis,
            _conventions: teistro_port_ephemeris::Conventions,
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

    /// A custom zodiac (SR-112) measures the Sun from the tropical one by
    /// the SDK's value, so its sankranti is the catalogued member's when it
    /// restates one; a catalogued choice puts the member back in the port's
    /// frame.
    #[test]
    fn a_custom_zodiac_finds_the_catalogued_members_sankranti() {
        use teistro_core::catalogue::Star;
        use teistro_core::settings::{AyanamshaBasis, Precession};

        let provider = TestProvider::new();
        let new = || {
            DrikSun::new(
                &provider,
                Ayanamsha::Lahiri,
                Sunrise::UpperLimbRefraction.into(),
                OverridePolicy::PreferNative,
                DeltaTModel::TableThenModel,
            )
            .with_ayanamsha_basis(AyanamshaBasis::True)
        };
        let from = Gregorian
            .to_fixed_ymd(2024, 3, 1)
            .unwrap()
            .jd_at_midnight()
            .unwrap();
        let chitra = new().with_ayanamsha_choice(Ayanamsha::TrueChitra.into());
        let spica = new().with_ayanamsha_choice(AyanamshaChoice::Anchored {
            star: Star::Spica,
            longitude_deg: 180.0,
        });
        assert_eq!(chitra.ayanamsha_choice(), Ayanamsha::TrueChitra.into());
        assert!(
            spica.describe().contains("SPICA held at 180°"),
            "{}",
            spica.describe()
        );
        let (a, b) = (
            find_sankranti(&chitra, 0, from).unwrap(),
            find_sankranti(&spica, 0, from).unwrap(),
        );
        let seconds = (a.instant.get() - b.instant.get()).abs() * 86_400.0;
        assert!(seconds < 0.01, "{seconds} s");
        // A precessed Lahiri, stated where the catalogue stands at its epoch.
        let lahiri = new();
        let precessed = new().with_ayanamsha_choice(AyanamshaChoice::Precessed {
            epoch_jd_tt: 2_451_545.0,
            value_deg: teistro_astro::ayanamsha::mean_deg(
                &Ayanamsha::Lahiri.into(),
                teistro_core::quantity::JulianDay::literal(2_451_545.0),
                teistro_astro::precession::PrecessionModel::default(),
                DeltaTModel::TableThenModel,
            )
            .unwrap(),
            basis: AyanamshaBasis::Mean,
            precession: Precession::Iau2006,
        });
        let (a, b) = (
            find_sankranti(&lahiri, 0, from).unwrap(),
            find_sankranti(&precessed, 0, from).unwrap(),
        );
        let seconds = (a.instant.get() - b.instant.get()).abs() * 86_400.0;
        assert!(seconds < 0.01, "{seconds} s");
        // Back to a catalogued member.
        let back = spica.with_ayanamsha_choice(Ayanamsha::Raman.into());
        assert_eq!(back.ayanamsha_choice(), Ayanamsha::Raman.into());
        assert!(back.describe().contains("RAMAN"));
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
