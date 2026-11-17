//! Settings and profiles (`docs/03-design/settings-and-profiles.md`): one
//! complete value every module reads, built by applying an explicit
//! patch to a named, versioned profile; a canonical serialisation and a
//! hash that go into every result; coherence validated once on the
//! resolved value, every finding returned.
//!
//! ```
//! use teistro_core::settings::{Profile, SettingsPatch, knobs::Node};
//! use teistro_core::catalogue::Ayanamsha;
//!
//! let profile = Profile::shipped("nepali-drik").expect("shipped");
//! let mut patch = SettingsPatch::default();
//! patch.frame.node = Some(Node::True);
//! patch.frame.ayanamsha = Some(Ayanamsha::Raman.into());
//! let resolved = profile.resolve(&patch).expect("coherent");
//! assert_eq!(resolved.settings.frame.node, Node::True);
//! assert_eq!(resolved.settings.hash().to_string().len(), 64);
//! assert!(resolved.warnings.is_empty(), "{:?}", resolved.warnings);
//! ```

mod explain;
pub mod knobs;
mod profiles;
pub mod recorded;
mod snapshot;

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::catalogue::{
    Ayanamsha, BalaScheme, Calendar, DashaSystem, Era, HouseSystem, Mark, ScoreBand, Source, Star,
    Varga,
};
use crate::envelope::{CALCULATION_VERSION, Hash, Provenance, Version};
use crate::error::{Error, Status};
use crate::quantity::Depth;
pub use crate::varga_rows::VargaChoice;
pub use explain::{
    Knob, KnobDifference, KnobExplained, KnobSource, Layer, Moves, Origin, Origins, SettingsDiff,
    SettingsDiffQuery, SettingsExplained, SettingsQuery,
};
pub use knobs::{
    AfterCycle, AmritKalam, AntardashaPlace, AshtakavargaGoodFrom, AshtottariGrouping,
    AyanamshaBasis, AyurdayaReadings, AyurdayaSpan, Balance, Benefics, BhavaBalaDivision, BhavaDig,
    BhavaDrishti, BhavaSpecialRules, BirthPeriod, BrahmaRule, Centre, CharaKarakas, Cheshta,
    DayBoundary, DayLordDay, DecennialDivision, DeltaT, DigKendras, Dip, Drekkana, Drik, DstGap,
    DstOverlap, DualLord, DurmuhurtaTable, EclipseShadow, EclipseVedha, EightKarakaOrder,
    Ekadhipatya, FirdariaNodes, FrameBias, GhatiReckoning, GrahaArudhaException, GrahaDivision,
    HoraReckoning, IshtaKashta, KaalaLords, KalachakraAfterNinth, KalachakraBalance,
    KalachakraMembership, KiratYeleEpoch, Kranti, LuminaryCheshta, LunarMonth, MaranaSaturn,
    MaranaVenus, MoonBinduFromJupiter, MoonEvents, MoonPaksha, Naisargika, NaisargikaDivision,
    NakshatraScheme, Nathonnatha, Node, NodeAspects, NodeCoLordship, NodeObstruction, NodeRulers,
    NodeVedha, Nutation, OverridePolicy, PanchakaName, PanchakaStart, PolarDayPolicy, PolarPolicy,
    Positions, PreDawnNight, Precession, PunyaKalaTable, PushkaraBhaga, RashiStart,
    RefractionFormula, ReleasingSharedSign, RequiredRupas, RetrogradeRejection, RisingLatitude,
    RituReckoning, RulingCount, SamvatsaraCount, Saptavargaja, SayanadiGhatis, SayanadiNodes,
    SeedOverflow, SevenKarakaOrder, ShantaSign, ShivaVasaReading, Shodhana, SiderealTime,
    SolarMonthStart, StrongestGraha, SunAyana, Sunrise, SuryaSunrise, SuryaTrig, Tier,
    TransitCentre, UnattestedDn, UnknownTime, VargaVariant, Varjyam, Vimshopaka, YearLength,
    Yuddha, Zodiac,
};
pub use profiles::{DEFAULT_PROFILE, PROFILE_ALIASES, Profile, ProfileId, SHIPPED_PROFILES, root};
pub use snapshot::SettingsSnapshot;

/// The settings document's schema version; a later knob appends and an
/// old document still hashes the same under the old schema.
///
/// ```
/// use teistro_core::settings::{SCHEMA, root};
///
/// assert_eq!(root().schema, SCHEMA);
/// ```
pub const SCHEMA: u16 = 1;

/// Which ayanamsha: a catalogued one, or a custom definition
/// (`03-design/astro-custom-ayanamsha.md`): a value at an epoch moving at
/// a linear rate, a value at an epoch carried by a named precession model,
/// or a star of the catalogue held at a sidereal longitude (SR-112).
///
/// ```
/// use teistro_core::catalogue::{Ayanamsha, Star};
/// use teistro_core::settings::{AyanamshaBasis, AyanamshaChoice, Precession};
///
/// let lahiri: AyanamshaChoice = Ayanamsha::Lahiri.into();
/// assert_eq!(serde_json::to_value(lahiri).unwrap(), serde_json::json!({"kind": "CATALOGUED", "id": "LAHIRI"}));
/// // A consumer's own: 23.85° at J2000, moving 50.29″ a year.
/// let own = AyanamshaChoice::Custom { epoch_jd_tt: 2_451_545.0, value_deg: 23.85, rate_deg_per_year: 50.29 / 3600.0 };
/// assert_eq!(serde_json::to_value(own).unwrap()["kind"], "CUSTOM");
/// // The Indian Astronomical Ephemeris's 23°15′00.658″, a true value on
/// // 21 March 1956, carried by IAU 2006 precession.
/// let precessed = AyanamshaChoice::Precessed {
///     epoch_jd_tt: 2_435_553.5,
///     value_deg: 23.250_182_778,
///     basis: AyanamshaBasis::True,
///     precession: Precession::Iau2006,
/// };
/// assert_eq!(serde_json::to_value(precessed).unwrap()["kind"], "PRECESSED");
/// // Spica held at 180°, True Chitra's form.
/// let anchored = AyanamshaChoice::Anchored { star: Star::Spica, longitude_deg: 180.0 };
/// assert_eq!(
///     serde_json::to_value(anchored).unwrap(),
///     serde_json::json!({"kind": "ANCHORED", "star": "SPICA", "longitude_deg": 180.0})
/// );
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AyanamshaChoice {
    /// A catalogued ayanamsha.
    Catalogued {
        /// The member.
        id: Ayanamsha,
    },
    /// A custom ayanamsha: a value at an epoch and a yearly rate.
    Custom {
        /// The epoch, a Julian day in TT.
        epoch_jd_tt: f64,
        /// The value at the epoch, degrees.
        value_deg: f64,
        /// The rate, degrees per Julian year.
        rate_deg_per_year: f64,
    },
    /// A value at an epoch carried to any date by a named precession
    /// model, as the Swiss Ephemeris's `SE_SIDM_USER` carries one: the
    /// vernal point of the date carried back to the epoch and read on the
    /// ecliptic of the epoch, the construction every catalogued epoch
    /// definition takes (crux C2160).
    Precessed {
        /// The epoch, a Julian day in TT.
        epoch_jd_tt: f64,
        /// The value at the epoch, degrees.
        value_deg: f64,
        /// The basis the value at the epoch is stated on: `TRUE` when it
        /// carries the epoch's nutation in longitude, as the Indian
        /// Astronomical Ephemeris's Lahiri value does, `MEAN` when not.
        /// The value is subtracted on `frame.ayanamsha_basis` whichever
        /// it is stated on (crux C2161).
        basis: AyanamshaBasis,
        /// The precession model that carries the value, whatever model
        /// `frame.precession` names for the rest of the sky.
        precession: Precession,
    },
    /// A star of the catalogue held at a sidereal longitude, as True
    /// Chitra holds Spica at 180°: the star's apparent longitude of date
    /// (a fixed direction's geometric one) less the longitude it is held
    /// at. The nutation in longitude moves the star and the equinox
    /// alike, so the definition holds on either basis (crux C2162).
    Anchored {
        /// The star.
        star: Star,
        /// The sidereal longitude it is held at, degrees.
        longitude_deg: f64,
    },
}

impl AyanamshaChoice {
    /// Whether every number of the definition is finite, the one thing
    /// the settings can check without an ephemeris.
    ///
    /// ```
    /// use teistro_core::catalogue::{Ayanamsha, Star};
    /// use teistro_core::settings::AyanamshaChoice;
    ///
    /// assert!(AyanamshaChoice::from(Ayanamsha::Lahiri).is_finite());
    /// assert!(!AyanamshaChoice::Anchored { star: Star::Spica, longitude_deg: f64::NAN }.is_finite());
    /// ```
    #[must_use]
    pub const fn is_finite(&self) -> bool {
        match *self {
            AyanamshaChoice::Catalogued { .. } => true,
            AyanamshaChoice::Custom {
                epoch_jd_tt,
                value_deg,
                rate_deg_per_year,
            } => epoch_jd_tt.is_finite() && value_deg.is_finite() && rate_deg_per_year.is_finite(),
            AyanamshaChoice::Precessed {
                epoch_jd_tt,
                value_deg,
                ..
            } => epoch_jd_tt.is_finite() && value_deg.is_finite(),
            AyanamshaChoice::Anchored { longitude_deg, .. } => longitude_deg.is_finite(),
        }
    }

    /// How a stamp names the definition: the catalogued member's key, or
    /// the kind of custom definition.
    ///
    /// ```
    /// use teistro_core::catalogue::{Ayanamsha, Star};
    /// use teistro_core::settings::AyanamshaChoice;
    ///
    /// assert_eq!(AyanamshaChoice::from(Ayanamsha::Lahiri).describe(), "LAHIRI");
    /// let spica = AyanamshaChoice::Anchored { star: Star::Spica, longitude_deg: 180.0 };
    /// assert_eq!(spica.describe(), "SPICA held at 180°");
    /// ```
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            AyanamshaChoice::Catalogued { id } => id.key().to_owned(),
            AyanamshaChoice::Custom { .. } => String::from("a custom ayanamsha"),
            AyanamshaChoice::Precessed { precession, .. } => {
                format!("a custom ayanamsha carried by {}", precession.key())
            }
            AyanamshaChoice::Anchored {
                star,
                longitude_deg,
            } => format!("{} held at {longitude_deg}°", star.key()),
        }
    }
}

/// Which ascendant a day's lagna table reads (crux C680,
/// `03-design/amrit-kalam-and-lagnas.md` §2).
///
/// ```
/// use teistro_core::catalogue::Ayanamsha;
/// use teistro_core::settings::LagnaTable;
///
/// let lahiri = LagnaTable::Sphere { ayanamsha: Ayanamsha::Lahiri.into() };
/// assert_eq!(
///     serde_json::to_value(lahiri).unwrap(),
///     serde_json::json!({"kind": "SPHERE", "ayanamsha": {"kind": "CATALOGUED", "id": "LAHIRI"}})
/// );
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LagnaTable {
    /// The context's own chart ascendant, in the chart's zodiac: the
    /// provider's own reckoning where it defines the angles (the Surya
    /// Siddhanta's rising times), the sphere's otherwise.
    Chart,
    /// The sphere's ascendant, from the sidereal time, in this
    /// ayanamsha, whatever the provider defines: what a patro printed in
    /// Nepal tabulates beside the text's Sun and Moon.
    Sphere {
        /// The zodiac the lagna is read in, on `frame.ayanamsha_basis`.
        ayanamsha: AyanamshaChoice,
    },
}

impl From<Ayanamsha> for AyanamshaChoice {
    fn from(id: Ayanamsha) -> AyanamshaChoice {
        AyanamshaChoice::Catalogued { id }
    }
}

/// The sunrise convention, with a custom altitude when asked.
///
/// ```
/// use teistro_core::settings::{Sunrise, SunriseConvention};
///
/// let named: SunriseConvention = Sunrise::UpperLimbRefraction.into();
/// assert_eq!(serde_json::to_value(named).unwrap(), serde_json::json!({"kind": "NAMED", "which": "UPPER_LIMB_REFRACTION"}));
/// // The Sun's centre six degrees down: civil twilight.
/// let dawn = SunriseConvention::Custom { altitude_deg: -6.0 };
/// assert_eq!(serde_json::to_value(dawn).unwrap()["altitude_deg"], -6.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SunriseConvention {
    /// One of the named conventions.
    Named {
        /// The convention.
        which: Sunrise,
    },
    /// The centre of the disc at a given altitude.
    Custom {
        /// The altitude of the centre, degrees; negative below the horizon.
        altitude_deg: f64,
    },
    /// A named convention that refracts, refracted by the given air rather
    /// than by the almanac's 34 arcminutes (`03-design/horizon-atmosphere.md`).
    ///
    /// ```
    /// use teistro_core::settings::{Atmosphere, Sunrise, SunriseConvention};
    ///
    /// // The engines' own convention: the standard air at the place.
    /// let engines = SunriseConvention::Atmospheric {
    ///     which: Sunrise::UpperLimbRefraction,
    ///     air: Atmosphere::STANDARD,
    /// };
    /// // A weather report's.
    /// let observed = SunriseConvention::Atmospheric {
    ///     which: Sunrise::UpperLimbRefraction,
    ///     air: Atmosphere::given(987.0, 4.5),
    /// };
    /// # let _ = (engines, observed);
    /// ```
    Atmospheric {
        /// The convention, which must refract.
        which: Sunrise,
        /// The air.
        air: Atmosphere,
    },
}

impl From<Sunrise> for SunriseConvention {
    fn from(which: Sunrise) -> SunriseConvention {
        SunriseConvention::Named { which }
    }
}

impl Sunrise {
    /// Whether the convention refracts, and so has an air to give it.
    ///
    /// ```
    /// use teistro_core::settings::Sunrise;
    ///
    /// // The disc's centre with refraction, the convention Nepal's daily
    /// // print is nearest, refracts as the limbs do.
    /// assert!(Sunrise::CentreRefraction.refracts() && Sunrise::UpperLimbRefraction.refracts());
    /// assert!(!Sunrise::CentreNoRefraction.refracts());
    /// ```
    #[must_use]
    pub const fn refracts(self) -> bool {
        matches!(
            self,
            Sunrise::UpperLimbRefraction | Sunrise::LowerLimbRefraction | Sunrise::CentreRefraction
        )
    }
}

/// The air a refracted horizon is seen through, as a convention names it:
/// each part given, or left to the engines' standard at the place
/// (`03-design/horizon-atmosphere.md` §5).
///
/// Left out, the pressure is the ICAO standard atmosphere's at the
/// observer's height and the temperature is 15 °C, which is what the
/// engines assume; so [`Atmosphere::STANDARD`] names their convention, and
/// a weather report gives both.
///
/// ```
/// use teistro_core::settings::Atmosphere;
///
/// // Either part may be left to the standard.
/// let cold: Atmosphere = serde_json::from_str(r#"{"temperature_c": -10}"#).unwrap();
/// assert_eq!((cold.pressure_hpa, cold.temperature_c), (None, Some(-10.0)));
/// assert_eq!(serde_json::from_str::<Atmosphere>("{}").unwrap(), Atmosphere::STANDARD);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Atmosphere {
    /// The pressure at the observer, hectopascals (millibars); absent for
    /// the standard atmosphere's at the observer's height.
    pub pressure_hpa: Option<f64>,
    /// The temperature at the observer, degrees Celsius; absent for 15 °C.
    pub temperature_c: Option<f64>,
}

/// The pressures an air may name, hectopascals: the highest recorded at
/// sea level is 1084, and 100 is far above any observer.
///
/// ```
/// use teistro_core::settings::{AIR_PRESSURE_HPA, Atmosphere};
///
/// assert!(AIR_PRESSURE_HPA.contains(&1084.0));
/// assert!(!Atmosphere::given(*AIR_PRESSURE_HPA.end() + 1.0, 15.0).is_valid());
/// ```
pub const AIR_PRESSURE_HPA: core::ops::RangeInclusive<f64> = 100.0..=1100.0;

/// The temperatures an air may name, degrees Celsius: the recorded
/// extremes at the surface are −89 and 57.
///
/// ```
/// use teistro_core::settings::{AIR_TEMPERATURE_C, Atmosphere};
///
/// assert!(AIR_TEMPERATURE_C.contains(&-89.0));
/// assert!(!Atmosphere::given(1013.25, *AIR_TEMPERATURE_C.start() - 1.0).is_valid());
/// ```
pub const AIR_TEMPERATURE_C: core::ops::RangeInclusive<f64> = -90.0..=60.0;

/// The standard atmosphere's temperature at sea level, and the engines'
/// at every height, degrees Celsius.
///
/// ```
/// use teistro_core::quantity::Altitude;
/// use teistro_core::settings::{Atmosphere, STANDARD_TEMPERATURE_C};
///
/// // The engines' temperature at every height.
/// assert_eq!(Atmosphere::STANDARD.at(Altitude::literal(4000.0)).temperature_c, STANDARD_TEMPERATURE_C);
/// ```
pub const STANDARD_TEMPERATURE_C: f64 = 15.0;

impl Atmosphere {
    /// The engines' standard air at the place: both parts left out.
    ///
    /// ```
    /// use teistro_core::settings::Atmosphere;
    ///
    /// assert_eq!((Atmosphere::STANDARD.pressure_hpa, Atmosphere::STANDARD.temperature_c), (None, None));
    /// assert!(Atmosphere::STANDARD.is_valid());
    /// ```
    pub const STANDARD: Atmosphere = Atmosphere {
        pressure_hpa: None,
        temperature_c: None,
    };

    /// An air with both parts given, as a weather report gives them.
    ///
    /// ```
    /// use teistro_core::quantity::Altitude;
    /// use teistro_core::settings::Atmosphere;
    ///
    /// // Given parts are kept whatever the height.
    /// let air = Atmosphere::given(987.0, 4.5).at(Altitude::literal(1400.0));
    /// assert_eq!((air.pressure_hpa, air.temperature_c), (987.0, 4.5));
    /// ```
    #[must_use]
    pub const fn given(pressure_hpa: f64, temperature_c: f64) -> Atmosphere {
        Atmosphere {
            pressure_hpa: Some(pressure_hpa),
            temperature_c: Some(temperature_c),
        }
    }

    /// The air at a height above sea level: what is given, and the
    /// standard for what is not.
    ///
    /// The pressure is the ICAO standard atmosphere's,
    /// `1013.25 · (1 − 0.0065 h / 288.15)^5.25588` hPa, whose exponent is
    /// `g₀M / (R L)`.
    ///
    /// ```
    /// use teistro_core::quantity::Altitude;
    /// use teistro_core::settings::Atmosphere;
    ///
    /// let sea = Atmosphere::STANDARD.at(Altitude::try_new(0.0).unwrap());
    /// assert_eq!((sea.pressure_hpa, sea.temperature_c), (1013.25, 15.0));
    /// let kathmandu = Atmosphere::STANDARD.at(Altitude::try_new(1400.0).unwrap());
    /// assert!((kathmandu.pressure_hpa - 856.0).abs() < 0.5);
    /// ```
    #[must_use]
    pub fn at(self, height: crate::quantity::Altitude) -> Air {
        const SEA_LEVEL_HPA: f64 = 1013.25;
        const SEA_LEVEL_K: f64 = 288.15;
        const LAPSE_K_PER_M: f64 = 0.0065;
        const EXPONENT: f64 = 5.25588;
        Air {
            pressure_hpa: self.pressure_hpa.unwrap_or_else(|| {
                SEA_LEVEL_HPA
                    * crate::math::powf(1.0 - LAPSE_K_PER_M * height.get() / SEA_LEVEL_K, EXPONENT)
            }),
            temperature_c: self.temperature_c.unwrap_or(STANDARD_TEMPERATURE_C),
        }
    }

    /// Whether each part given lies in its range (`AIR_PRESSURE_HPA`,
    /// `AIR_TEMPERATURE_C`).
    ///
    /// ```
    /// use teistro_core::settings::Atmosphere;
    ///
    /// assert!(Atmosphere::given(860.0, 12.0).is_valid());
    /// assert!(!Atmosphere::given(f64::NAN, 12.0).is_valid());
    /// assert!(!Atmosphere::given(860.0, 70.0).is_valid());
    /// ```
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.pressure_hpa
            .is_none_or(|p| AIR_PRESSURE_HPA.contains(&p))
            && self
                .temperature_c
                .is_none_or(|t| AIR_TEMPERATURE_C.contains(&t))
    }
}

/// An air resolved at a place: the pressure and the temperature a
/// refraction is computed from.
///
/// ```
/// use teistro_core::quantity::Altitude;
/// use teistro_core::settings::Atmosphere;
///
/// let air = Atmosphere::STANDARD.at(Altitude::literal(0.0));
/// assert_eq!(serde_json::to_value(air).unwrap(), serde_json::json!({"pressure_hpa": 1013.25, "temperature_c": 15.0}));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Air {
    /// Hectopascals.
    pub pressure_hpa: f64,
    /// Degrees Celsius.
    pub temperature_c: f64,
}

/// UT1 less UTC: none, the IERS's measured value, or a value the consumer
/// gives (`03-design/time-and-timezone.md` §3.1).
///
/// UTC is kept within 0.9 s of UT1 by leap seconds, so the default, none,
/// moves a civil instant by at most that; a precise lagna takes the IERS
/// value, which an engine reads from the IERS Earth orientation series
/// (EOP 14 C04 and `finals`) and answers through the port.
///
/// ```
/// use teistro_core::settings::Dut1;
///
/// assert_eq!(serde_json::to_value(Dut1::None).unwrap(), serde_json::json!({"kind": "NONE"}));
/// let own: Dut1 = serde_json::from_str(r#"{"kind": "CUSTOM", "seconds": -0.12}"#).unwrap();
/// assert_eq!(own, Dut1::Custom { seconds: -0.12 });
/// assert_eq!(Dut1::default(), Dut1::None);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Dut1 {
    /// UT1 taken as UTC: the default, within the 0.9 s a leap second
    /// keeps them.
    #[default]
    None,
    /// The IERS's UT1 less UTC (IERS Earth orientation parameters, EOP 14
    /// C04 and Bulletin A's `finals`), from the ephemeris engine; zero,
    /// and stamped so, outside the series it holds.
    Iers,
    /// A value the consumer gives, within ±0.9 s.
    Custom {
        /// UT1 less UTC, seconds.
        seconds: f64,
    },
}

/// The bound on UT1 less UTC a leap second keeps, seconds.
///
/// ```
/// use teistro_core::settings::DUT1_BOUND_SECONDS;
///
/// assert_eq!(DUT1_BOUND_SECONDS, 0.9);
/// ```
pub const DUT1_BOUND_SECONDS: f64 = 0.9;

/// The size of the disc whose limb rises or sets.
///
/// ```
/// use teistro_core::settings::DiscSize;
///
/// assert_eq!(DiscSize::default(), DiscSize::Actual);
/// // An almanac's fixed sixteen minutes of arc.
/// let fixed: DiscSize = serde_json::from_str(r#"{"kind": "FIXED", "semidiameter_arcmin": 16.0}"#).unwrap();
/// assert_eq!(fixed, DiscSize::Fixed { semidiameter_arcmin: 16.0 });
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiscSize {
    /// The disc at the body's distance at the instant: the default.
    #[default]
    Actual,
    /// The disc at the body's mean distance (the Sun at 1 au, the Moon at
    /// 384 400 km), so every day's rise is defined alike (the Swiss
    /// Ephemeris's `SE_BIT_FIXED_DISC_SIZE`).
    MeanDistance,
    /// A semidiameter the convention fixes, as an almanac's sixteen minutes
    /// of arc.
    Fixed {
        /// The semidiameter, arcminutes.
        semidiameter_arcmin: f64,
    },
}

/// The radii a disc's size is reckoned from.
///
/// ```
/// use teistro_core::settings::Radii;
///
/// assert_eq!(Radii::default(), Radii::Iau2015);
/// // The older solar radius the Swiss Ephemeris keeps.
/// let older: Radii = serde_json::from_str(r#"{"kind": "CUSTOM", "sun_km": 696000.0, "moon_km": 1737.4}"#).unwrap();
/// assert_eq!(older, Radii::Custom { sun_km: 696_000.0, moon_km: 1_737.4 });
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Radii {
    /// The Sun's nominal radius, 695 700 km (IAU 2015 Resolution B3), and
    /// the IAU Working Group on Cartographic Coordinates' mean radii for
    /// the Moon and the planets (Archinal et al. 2018, *Celest. Mech. Dyn.
    /// Astr.* 130, 22): the default.
    #[default]
    #[serde(rename = "IAU_2015")]
    Iau2015,
    /// The Sun's and the Moon's radii a convention gives; the planets keep
    /// the IAU's.
    Custom {
        /// The Sun's radius, kilometres.
        sun_km: f64,
        /// The Moon's radius, kilometres.
        moon_km: f64,
    },
}

/// How a horizon is reckoned beyond which point of the disc and whether it
/// refracts (`day.sunrise`): the refraction formula an air is read through,
/// the disc's size and radii, the rising body's latitude, the dip and a
/// raised or lowered horizon.
///
/// One value for every horizon a context reads (the sunrise, the eclipse
/// horizon, the Moon's rise and set), so that the Sun has one radius and
/// the air one formula wherever the SDK meets the horizon. Every part
/// defaults to what the SDK has always reckoned, so a document that leaves
/// them out answers as before.
///
/// ```
/// use teistro_core::settings::{Dip, HorizonModel, RefractionFormula};
///
/// // Every part may be left out.
/// let raised: HorizonModel = serde_json::from_str(r#"{"dip": "GEOMETRIC", "altitude_deg": 0.5}"#).unwrap();
/// assert_eq!((raised.dip, raised.refraction), (Dip::Geometric, RefractionFormula::Bennett));
/// assert_eq!(serde_json::from_str::<HorizonModel>("{}").unwrap(), HorizonModel::STANDARD);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct HorizonModel {
    /// The formula a named air refracts through (an `ATMOSPHERIC`
    /// sunrise); the standard refraction is the almanac's stated 34′.
    pub refraction: RefractionFormula,
    /// The disc's size.
    pub disc_size: DiscSize,
    /// The radii the disc is reckoned from.
    pub radii: Radii,
    /// The rising body's latitude: its own, or zero.
    pub latitude: RisingLatitude,
    /// The dip of the horizon from the observer's height.
    pub dip: Dip,
    /// The apparent altitude of the horizon the point reaches, degrees:
    /// positive for a horizon raised above the ideal one, as hills raise
    /// it, negative for one below. Added to a custom sunrise's altitude.
    pub altitude_deg: f64,
}

impl HorizonModel {
    /// What the SDK has always reckoned: Bennett's formula for a named
    /// air, the actual disc of the IAU 2015 radii, the body's own latitude,
    /// no dip and the ideal horizon.
    ///
    /// ```
    /// use teistro_core::settings::{HorizonModel, root};
    ///
    /// assert_eq!(root().day.horizon, HorizonModel::STANDARD);
    /// assert_eq!(HorizonModel::default(), HorizonModel::STANDARD);
    /// ```
    pub const STANDARD: HorizonModel = HorizonModel {
        refraction: RefractionFormula::Bennett,
        disc_size: DiscSize::Actual,
        radii: Radii::Iau2015,
        latitude: RisingLatitude::Body,
        dip: Dip::None,
        altitude_deg: 0.0,
    };
}

impl HorizonModel {
    /// The key stamped in provenance: formula, disc, radii, latitude, dip
    /// and altitude.
    ///
    /// ```
    /// use teistro_core::settings::{DiscSize, HorizonModel};
    ///
    /// assert_eq!(HorizonModel::STANDARD.key(), "BENNETT/ACTUAL/IAU_2015/BODY/NONE/0");
    /// let fixed = HorizonModel { disc_size: DiscSize::Fixed { semidiameter_arcmin: 16.0 }, ..HorizonModel::STANDARD };
    /// assert_eq!(fixed.key(), "BENNETT/FIXED[16']/IAU_2015/BODY/NONE/0");
    /// ```
    #[must_use]
    pub fn key(&self) -> String {
        let disc = match self.disc_size {
            DiscSize::Actual => String::from("ACTUAL"),
            DiscSize::MeanDistance => String::from("MEAN_DISTANCE"),
            DiscSize::Fixed {
                semidiameter_arcmin,
            } => format!("FIXED[{semidiameter_arcmin}']"),
        };
        let radii = match self.radii {
            Radii::Iau2015 => String::from("IAU_2015"),
            Radii::Custom { sun_km, moon_km } => format!("CUSTOM[{sun_km},{moon_km}]"),
        };
        format!(
            "{}/{disc}/{radii}/{}/{}/{}",
            self.refraction.key(),
            self.latitude.key(),
            self.dip.key(),
            self.altitude_deg
        )
    }
}

impl Default for HorizonModel {
    fn default() -> HorizonModel {
        HorizonModel::STANDARD
    }
}

/// Which astronomy: modern, the Surya Siddhanta under a bija, or the
/// *Vakyakarana*'s Sun and Moon as the Tamil vakya almanacs print them.
///
/// The settings **ask** and the ephemeris chain **supplies** (ADR-0029):
/// `SURYA` and `VAKYA` need a context opened over `SURYA_SIDDHANTA`, and
/// the model that entry opens is the text with this bija applied, or the
/// vakya sky beside it.
///
/// ```
/// use teistro_core::settings::{Siddhanta, SuryaBija};
///
/// assert_eq!(serde_json::to_value(Siddhanta::Drik).unwrap(), serde_json::json!({"kind": "DRIK"}));
/// assert_eq!(serde_json::to_value(Siddhanta::Vakya).unwrap(), serde_json::json!({"kind": "VAKYA"}));
/// // The text's sunrise and planets are the defaults, so a document may leave them out.
/// let surya: Siddhanta = serde_json::from_str(r#"{"kind": "SURYA", "bija": {"kind": "NEPAL_COMMITTEE"}}"#).unwrap();
/// assert_eq!(surya, Siddhanta::surya(SuryaBija::NepalCommittee));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Siddhanta {
    /// Modern astronomy.
    Drik,
    /// The Surya Siddhanta, with the revolution counts a bija gives.
    Surya {
        /// Which bija the text's revolution counts take.
        bija: SuryaBija,
        /// Whose sunrise the day begins at: the text's (the default) or a
        /// modern ephemeris's beside the text's sky.
        #[serde(default)]
        sunrise: SuryaSunrise,
        /// Where the star planets and the node come from: the text (the
        /// default) or a modern ephemeris beside it.
        #[serde(default)]
        planets: SuryaPlanets,
        /// The trigonometry the text's equations take: its table of
        /// twenty-four sines (the default) or the true sine (C1950).
        #[serde(default)]
        trig: SuryaTrig,
    },
    /// The *Vakyakarana*'s Sun and its Moon with the almanac makers'
    /// manyadi, as the Tamil vakya almanacs reckon the tithi and the
    /// nakshatra (`docs/calendars/vakya.md`): their places at each
    /// sunrise at Chennai, joined between sunrises by proportion, the day
    /// counted from a modern ephemeris's sunrise under `day.sunrise`. The
    /// star planets, the angles and the zodiac are the Surya Siddhanta's.
    Vakya,
}

/// Where the Surya Siddhanta's star planets and node come from (crux
/// C38, `03-design/committee-star-planets.md`).
///
/// Nepal's national panchanga committee prints the text's Sun and Moon
/// beside modern star planets in Lahiri's zodiac: over 49 printed
/// mornings a modern sky misses its Mars, Mercury, Jupiter, Venus and
/// Saturn by 9′ to 33′, the text by 94′ to 459′.
///
/// ```
/// use teistro_core::catalogue::Ayanamsha;
/// use teistro_core::settings::SuryaPlanets;
///
/// let modern = SuryaPlanets::Modern { ayanamsha: Ayanamsha::Lahiri.into() };
/// assert_eq!(
///     serde_json::to_value(modern)?,
///     serde_json::json!({"kind": "MODERN", "ayanamsha": {"kind": "CATALOGUED", "id": "LAHIRI"}})
/// );
/// assert_eq!(SuryaPlanets::default(), SuryaPlanets::Text);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SuryaPlanets {
    /// The text's, by its four steps (II.39 to 45), each from its own
    /// mean motion under the bija. The Makaranda's three steps are the
    /// same four folded, and Mithila's almanac prints them (C1227).
    #[default]
    Text,
    /// Mars, Mercury, Jupiter, Venus, Saturn and the mean node from the
    /// modern ephemeris a modern sunrise comes from (the next entry of the
    /// chain that opens, else the SDK's built-in one), apparent and
    /// geocentric, less this ayanamsha's mean value. The Sun, the Moon,
    /// the angles and the zodiac stay the text's.
    Modern {
        /// The zodiac the modern places are measured in.
        ayanamsha: AyanamshaChoice,
    },
}

impl Siddhanta {
    /// The Surya Siddhanta under a bija, its day beginning at the text's
    /// own sunrise, its equations by the text's sine table.
    ///
    /// ```
    /// use teistro_core::settings::{Siddhanta, SuryaBija, SuryaPlanets, SuryaSunrise, SuryaTrig};
    ///
    /// let text = Siddhanta::surya(SuryaBija::None);
    /// assert_eq!(
    ///     text,
    ///     Siddhanta::Surya {
    ///         bija: SuryaBija::None,
    ///         sunrise: SuryaSunrise::Text,
    ///         planets: SuryaPlanets::Text,
    ///         trig: SuryaTrig::Table,
    ///     }
    /// );
    /// ```
    #[must_use]
    pub const fn surya(bija: SuryaBija) -> Siddhanta {
        Siddhanta::Surya {
            bija,
            sunrise: SuryaSunrise::Text,
            planets: SuryaPlanets::Text,
            trig: SuryaTrig::Table,
        }
    }
}

/// Which bija (seed) corrections the Surya Siddhanta's revolution counts
/// take.
///
/// The later commentators correct the text's counts and their sets
/// differ (C28). What ships is the text without one, the set
/// **measured** from Nepal's national panchanga committee, the set the
/// Makarandasarini's tables print (C1221), and a consumer's own.
///
/// ```
/// use teistro_core::settings::{Bija, SuryaBija};
///
/// assert_eq!(SuryaBija::None.revolutions(), Bija::default());
/// assert_eq!(SuryaBija::NepalCommittee.revolutions().moon_apsis, -4);
/// let own = Bija { jupiter: -8, ..Bija::default() };
/// assert_eq!(SuryaBija::Custom { revolutions: own }.revolutions(), own);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum SuryaBija {
    /// The text as Burgess (1860) prints it.
    #[default]
    None,
    /// The set Nepal's national panchanga committee computes with,
    /// **measured** rather than cited: the Moon's apsis makes four
    /// revolutions fewer in an age, and nothing else moves
    /// ([`Bija::NEPAL_COMMITTEE`]).
    NepalCommittee,
    /// The Makarandasarini's set, read from its daily-motion tables
    /// ([`Bija::MAKARANDA`]): what Mithila's almanacs compute with.
    Makaranda,
    /// A consumer's own set.
    Custom {
        /// The revolutions per age it adds to or takes from the text's.
        revolutions: Bija,
    },
}

impl SuryaBija {
    /// The revolutions this bija adds to or takes from the text's counts.
    ///
    /// ```
    /// use teistro_core::settings::{Bija, SuryaBija};
    ///
    /// assert_eq!(SuryaBija::None.revolutions(), Bija::NONE);
    /// assert_eq!(SuryaBija::NepalCommittee.revolutions(), Bija::NEPAL_COMMITTEE);
    /// assert_eq!(SuryaBija::Makaranda.revolutions(), Bija::MAKARANDA);
    /// ```
    #[must_use]
    pub const fn revolutions(self) -> Bija {
        match self {
            SuryaBija::None => Bija::NONE,
            SuryaBija::NepalCommittee => Bija::NEPAL_COMMITTEE,
            SuryaBija::Makaranda => Bija::MAKARANDA,
            SuryaBija::Custom { revolutions } => revolutions,
        }
    }
}

/// A bija (seed) correction: the whole revolutions per age a tradition
/// adds to or takes from the Surya Siddhanta's counts.
///
/// ```
/// use teistro_core::settings::Bija;
///
/// // A part left out is no correction.
/// let own: Bija = serde_json::from_str(r#"{"jupiter": -8}"#).unwrap();
/// assert_eq!(own, Bija { jupiter: -8, ..Bija::NONE });
/// assert!(serde_json::from_str::<Bija>(r#"{"pluto": 1}"#).is_err());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct Bija {
    /// The Moon's revolutions.
    pub moon: i64,
    /// The Moon's apsis.
    pub moon_apsis: i64,
    /// The Moon's node.
    pub moon_node: i64,
    /// Mars.
    pub mars: i64,
    /// Mercury's conjunction.
    pub mercury: i64,
    /// Jupiter.
    pub jupiter: i64,
    /// Venus's conjunction.
    pub venus: i64,
    /// Saturn.
    pub saturn: i64,
}

impl Bija {
    /// No correction: the text's own counts.
    ///
    /// ```
    /// use teistro_core::settings::Bija;
    ///
    /// assert_eq!(Bija::NONE, Bija::default());
    /// ```
    pub const NONE: Bija = Bija {
        moon: 0,
        moon_apsis: 0,
        moon_node: 0,
        mars: 0,
        mercury: 0,
        jupiter: 0,
        venus: 0,
        saturn: 0,
    };

    /// Nepal's national panchanga committee's set, **measured**: the
    /// Moon's apsis makes 488 199 revolutions in an age against the
    /// text's 488 203. It puts the committee's printed Moon within 0.3′
    /// at two sunrises and its eight printed tithi ends within 0.5′
    /// (`docs/calendars/bikram-sambat.md`, R2), and 1 039 of 1 138
    /// daily tithi, nakshatra and yoga ends printed from it within a
    /// minute and a half (`03-design/nepal-day-measured.md`), where the
    /// text without it leaves them a quarter of an hour either way. The
    /// committee has not named the set (C28).
    ///
    /// ```
    /// use teistro_core::settings::Bija;
    ///
    /// assert_eq!(Bija::NEPAL_COMMITTEE, Bija { moon_apsis: -4, ..Bija::NONE });
    /// ```
    pub const NEPAL_COMMITTEE: Bija = Bija {
        moon_apsis: -4,
        ..Bija::NONE
    };

    /// The Makarandasarini's set (1478), **read** from its own tables
    /// (`03-design/makaranda.md`): each daily-motion table (*vatika*)
    /// steps by the text's motion with whole revolutions added or taken
    /// away in an age. The Moon's apsis makes four fewer (the Nepal
    /// committee's measured set, here printed), its node four more,
    /// Mercury's conjunction sixteen more, Jupiter eight fewer (the
    /// footer's own "kalpa years over 1500, subtracted"), Venus's
    /// conjunction twelve fewer and Saturn twelve more; the Sun, the
    /// Moon and Mars keep the text's. Every count is a multiple of four,
    /// so all the planets still meet at the first of Mesha when the Kali
    /// age began (I.57).
    ///
    /// ```
    /// use teistro_core::settings::Bija;
    ///
    /// let set = Bija::MAKARANDA;
    /// assert_eq!((set.moon_apsis, set.moon_node, set.jupiter), (-4, 4, -8));
    /// assert_eq!((set.moon, set.mars), (0, 0));
    /// ```
    pub const MAKARANDA: Bija = Bija {
        moon: 0,
        moon_apsis: -4,
        moon_node: 4,
        mars: 0,
        mercury: 16,
        jupiter: -8,
        venus: -12,
        saturn: 12,
    };
}

/// The rounding contract of serialised output.
///
/// ```
/// use teistro_core::settings::root;
///
/// let precision = root().output.precision;
/// assert!(precision.angle_decimals > 0 && precision.instant_decimals > 0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Precision {
    /// Decimals of a degree value beside its exact integer.
    pub angle_decimals: u8,
    /// Decimals of a Julian day.
    pub instant_decimals: u8,
    /// Decimals of a score.
    pub score_decimals: u8,
}

/// The three thresholds a rashifal's overall score is banded by,
/// descending: a score at or above the first is
/// [`ScoreBand::Excellent`], at or above the second
/// [`ScoreBand::Good`], at or above the third [`ScoreBand::Mixed`], and
/// below it [`ScoreBand::Care`].
///
/// No text bands a score, which is the baseline engine's own number
/// (C361). [`ScoreBands::DEFAULT`], 75, 50 and 30, is Tithipatra's
/// design (SR-124), and the knob `rashifal.score_bands` sets another
/// (`03-design/rashifal.md`, "The score band"). The settings refuse
/// thresholds that are not strictly descending or exceed 100.
///
/// ```
/// use teistro_core::catalogue::ScoreBand;
/// use teistro_core::settings::{ScoreBands, root};
///
/// let bands = root().rashifal.score_bands;
/// assert_eq!(bands, ScoreBands::DEFAULT);
/// assert_eq!((bands.band(75), bands.band(74)), (ScoreBand::Excellent, ScoreBand::Good));
/// assert_eq!((bands.band(30), bands.band(29)), (ScoreBand::Mixed, ScoreBand::Care));
/// // The JSON form is the three numbers.
/// assert_eq!(serde_json::to_string(&bands).unwrap(), "[75,50,30]");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct ScoreBands(pub [u8; 3]);

impl ScoreBands {
    /// Tithipatra's thresholds (SR-124): 75, 50 and 30.
    pub const DEFAULT: ScoreBands = ScoreBands([75, 50, 30]);

    /// The band an overall score from 0 to 100 falls in.
    ///
    /// ```
    /// use teistro_core::catalogue::ScoreBand;
    /// use teistro_core::settings::ScoreBands;
    ///
    /// let strict = ScoreBands([90, 60, 40]);
    /// assert_eq!(strict.band(100), ScoreBand::Excellent);
    /// assert_eq!(strict.band(75), ScoreBand::Good);
    /// assert_eq!(strict.band(0), ScoreBand::Care);
    /// ```
    #[must_use]
    pub const fn band(self, overall: u8) -> ScoreBand {
        let [excellent, good, mixed] = self.0;
        if overall >= excellent {
            ScoreBand::Excellent
        } else if overall >= good {
            ScoreBand::Good
        } else if overall >= mixed {
            ScoreBand::Mixed
        } else {
            ScoreBand::Care
        }
    }
}

impl Default for ScoreBands {
    fn default() -> ScoreBands {
        ScoreBands::DEFAULT
    }
}

/// How many ephemeris cells a context remembers by default.
///
/// A fifty-day almanac range asks for 53 935 cells of which 21 446 are
/// distinct (`03-design/batch-and-parallelism-measured.md`), so this
/// holds such a range whole with room over; a cell and its key are about
/// a hundred bytes, which puts it at roughly six megabytes. The port's
/// own default is this number — one value, in the place a consumer sets
/// it.
///
/// ```
/// use teistro_core::settings::{DEFAULT_CACHE_CELLS, root};
///
/// assert_eq!(root().provider.cache_cells, DEFAULT_CACHE_CELLS);
/// ```
pub const DEFAULT_CACHE_CELLS: u32 = 65_536;

/// How near a sankranti a lunar month's new moon may fall, minutes,
/// before the month is called marginal by default: seven hours.
///
/// The Surya Siddhanta's Sun and a drik Sun in Lahiri's zodiac decide a
/// month differently in nineteen years of AD 1900–2100, and in each of
/// them a month the two part on has a new moon within 0.3 to 6.3 hours
/// of a sankranti (tithipatra's `sdk-gap-research.md` §3.11 and §A.14;
/// reproduced from the field in `crates/sdk/tests/month_margin.rs`).
/// Seven hours flags every one of those years, the widest by 0.7 hours
/// (C1902).
///
/// ```
/// use teistro_core::settings::{DEFAULT_MARGINAL_MONTH_MINUTES, root};
///
/// assert_eq!(root().calendars.marginal_month_minutes, DEFAULT_MARGINAL_MONTH_MINUTES);
/// ```
pub const DEFAULT_MARGINAL_MONTH_MINUTES: u32 = 420;

macro_rules! group {
    ($(#[$m:meta])* $name:ident, $patch:ident, $path:ident {
        $(#[$ffm:meta])* $first:ident : $fty:ty => $fmoves:ident
        $(, $(#[$fm:meta])* $field:ident : $ty:ty => $moves:ident )* $(,)?
    }) => {
        $(#[$m])*
        ///
        #[doc = concat!("```\nuse teistro_core::settings::{", stringify!($name), ", root};\n\nlet group: ", stringify!($name), " = root().", stringify!($path), ";\n// The canonical document writes every knob.\nlet written = serde_json::to_value(&group).unwrap();\nassert_eq!(written.as_object().map(|knobs| knobs.len()), Some(", stringify!($name), "::KNOBS.len()));\n```")]
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            $(#[$ffm])* pub $first: $fty,
            $( $(#[$fm])* pub $field: $ty ),*
        }

        #[doc = concat!("The patch of `", stringify!($name), "`: every knob optional.")]
        ///
        #[doc = concat!("```\nuse teistro_core::settings::", stringify!($patch), ";\n\nlet patch: ", stringify!($patch), " = serde_json::from_str(\"{}\").unwrap();\nassert!(patch.is_empty());\n// A knob the group does not have is refused, not ignored.\nassert!(serde_json::from_str::<", stringify!($patch), ">(r#\"{\"not_a_knob\": 1}\"#).is_err());\n```")]
        #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[serde(default, deny_unknown_fields)]
        pub struct $patch {
            $(#[$ffm])* pub $first: Option<$fty>,
            $( $(#[$fm])* pub $field: Option<$ty> ),*
        }

        impl $name {
            /// Applies every knob the patch sets.
            ///
            #[doc = concat!("```\nuse teistro_core::settings::{", stringify!($patch), ", root};\n\nlet mut group = root().", stringify!($path), ";\n// An empty patch changes nothing; a set knob takes the patch's value.\ngroup.apply(&", stringify!($patch), "::default());\nassert_eq!(group, root().", stringify!($path), ");\nlet patch = ", stringify!($patch), " { ", stringify!($first), ": Some(group.", stringify!($first), ".clone()), ..Default::default() };\ngroup.apply(&patch);\nassert_eq!(Some(group.", stringify!($first), "), patch.", stringify!($first), ");\n```")]
            pub fn apply(&mut self, patch: &$patch) {
                self.apply_noting(patch, &mut |_| {});
            }

            /// Applies every knob the patch sets, telling `note` the name of
            /// each as it is set: the one merge, with the accounting
            /// [`Origins`] keeps of it (SR-110), so where a knob came from is
            /// what the merge did and not a second reading of the patch.
            pub(crate) fn apply_noting(&mut self, patch: &$patch, note: &mut dyn FnMut(&'static str)) {
                if let Some(value) = &patch.$first { self.$first = value.clone(); note(stringify!($first)); }
                $( if let Some(value) = &patch.$field { self.$field = value.clone(); note(stringify!($field)); } )*
            }

            /// The knob names, in document order.
            ///
            #[doc = concat!("```\nuse teistro_core::settings::{", stringify!($name), ", Settings};\n\nassert_eq!(", stringify!($name), "::KNOBS[0], \"", stringify!($first), "\");\nassert!(Settings::knob_paths().contains(&(\"", stringify!($path), "\", \"", stringify!($first), "\")));\n```")]
            pub const KNOBS: &'static [&'static str] = &[ stringify!($first), $( stringify!($field) ),* ];

            /// What a difference in each knob moves, in the order of
            /// [`Self::KNOBS`], declared beside each knob.
            pub const MOVES: &'static [Moves] = &[ Moves::$fmoves, $( Moves::$moves ),* ];
        }

        impl $patch {
            /// Whether the patch sets nothing.
            ///
            #[doc = concat!("```\nuse teistro_core::settings::{", stringify!($patch), ", root};\n\nlet mut patch = ", stringify!($patch), "::default();\nassert!(patch.is_empty());\npatch.", stringify!($first), " = Some(root().", stringify!($path), ".", stringify!($first), ");\nassert!(!patch.is_empty());\n```")]
            #[must_use]
            pub fn is_empty(&self) -> bool {
                self.$first.is_none() $( && self.$field.is_none() )*
            }
        }
    };
}

group!(
    /// The frame positions are expressed in.
    Frame,
    FramePatch,
    frame {
        /// Tropical or sidereal.
        zodiac: Zodiac => Sky,
        /// Which ayanamsha, when sidereal.
        ayanamsha: AyanamshaChoice => Sky,
        /// Mean or with nutation.
        ayanamsha_basis: AyanamshaBasis => Sky,
        /// Which node.
        node: Node => Sky,
        /// Geocentric or topocentric.
        centre: Centre => Sky,
        /// Apparent or true.
        positions: Positions => Sky,
        /// Modern or classical astronomy: which the chart asks for. The
        /// context holds it to the provider it opens, refusing a modern
        /// engine under the Surya Siddhanta, since only the text defines
        /// the text's chart (`03-design/classical-chart.md`).
        siddhanta: Siddhanta => Sky,
        /// Twenty-seven or twenty-eight nakshatras.
        /// lint: knob-has-a-reader — `panchanga` and `core::angle` both divide
        /// by twenty-seven today; the twenty-eighth (Abhijit) is unequal and
        /// wants its own measurement before either reads this.
        nakshatra_scheme: NakshatraScheme => Rule,
        /// Which precession carries positions and epoch-defined ayanamshas
        /// between epochs; asked of the engine with every position.
        precession: Precession => Sky,
        /// Which nutation series; asked of the engine with every position.
        nutation: Nutation => Sky,
        /// Which frame bias; asked of the engine with every position.
        frame_bias: FrameBias => Sky,
    }
);

group!(
    /// House systems and their policy.
    Houses, HousesPatch, houses {
        /// The system placements use.
        placement_system: HouseSystem => Rule,
        /// The Bhava-Chalit system.
        chalit_system: HouseSystem => Rule,
        /// Per-module overrides (`kp` to Placidus).
        module_overrides: BTreeMap<String, HouseSystem> => Rule,
        /// What to do where the system has no solution.
        polar_policy: PolarPolicy => Rule,
    }
);

group!(
    /// The local day.
    Day,
    DayPatch,
    day {
        /// The sunrise convention.
        sunrise: SunriseConvention => Sunrise,
        /// When the day begins.
        day_boundary: DayBoundary => Rule,
        /// A day without a sunrise.
        polar_day_policy: PolarDayPolicy => Rule,
        /// How ghatis are counted.
        ghati_reckoning: GhatiReckoning => Rule,
        /// How the planetary hours are counted.
        hora_reckoning: HoraReckoning => Rule,
        /// How every horizon is reckoned beyond the sunrise convention: the
        /// refraction formula, the disc's size and radii, the rising
        /// body's latitude, the dip and the horizon's altitude.
        horizon: HorizonModel => Sunrise,
    }
);

group!(
    /// The daily panchanga.
    ///
    /// Each knob is a difference a record measured: the conformance
    /// corpus (`03-design/panchanga-day.md` §2), a text, or Nepal's
    /// daily panchanga (`03-design/ritu-measured.md`). The window the day's
    /// limbs run over is `day.day_boundary`, and the arcs its periods
    /// divide are the `day.sunrise` ones, so neither is repeated here.
    Panchanga,
    PanchangaPatch,
    panchanga {
        /// Which centre the day's limbs are computed from. An almanac is
        /// geocentric wherever the chart is: the lunar parallax reaches a
        /// degree, which is two hours of tithi boundary.
        centre: Centre => Sky,
        /// Which window the Moon's rise and set are found in.
        moon_events: MoonEvents => Rule,
        /// The muhurta yoga tables' key, as `aspect.drishti_table` is:
        /// `CLASSICAL`, read at the star and tithi of sunrise, or
        /// `MUHURTA_CHINTAMANI`, the text's full tables over every star of
        /// the day (`03-design/day-markers.md` §2).
        muhurta_tables: String => Rule,
        /// Where panchaka begins: Dhanishtha's third quarter or the
        /// whole of it (crux C158).
        panchaka_start: PanchakaStart => Rule,
        /// What the panchaka is named: by the vara it begins on, a
        /// printed convention, or not at all (crux C821).
        panchaka_name: PanchakaName => Rule,
        /// Where Shiva dwells on each tithi: the almanacs' printed
        /// convention or none (crux C750).
        shiva_vasa: ShivaVasaReading => Rule,
        /// Which table places each nakshatra's varjyam (crux C165).
        varjyam: Varjyam => Rule,
        /// Which reading places each nakshatra's amrita kalam (crux C677).
        amrit_kalam: AmritKalam => Rule,
        /// Which ascendant the day's lagna table reads, when asked for
        /// (crux C680).
        lagna_table: LagnaTable => Rule,
        /// Which muhurtas of each weekday are censured, the durmuhurtas
        /// (crux C398).
        durmuhurta: DurmuhurtaTable => Rule,
        /// How many ghatis around each sankranti are its punya kala
        /// (crux C752).
        punya_kala: PunyaKalaTable => Rule,
        /// Which season a day falls in: the sidereal solar months', the
        /// lunar month's or the tropical Sun's (crux C178).
        ritu: RituReckoning => Rule,
        /// Which civil day a sidereal solar month begins on, which the
        /// solar season reads (crux C186).
        solar_month_start: SolarMonthStart => Rule,
        /// How much the Earth's shadow is enlarged for a lunar eclipse.
        eclipse_shadow: EclipseShadow => Rule,
        /// The horizon an eclipsed body is seen above. The eye sees the
        /// upper limb, lifted by refraction, which is *Dharmasindhu*'s test
        /// for an eclipse that counts and the rising Nepal's committee
        /// times a vedha from (`03-design/eclipses.md` §5).
        eclipse_horizon: SunriseConvention => Rule,
        /// How far before an eclipse its vedha opens (crux C192).
        eclipse_vedha: EclipseVedha => Rule,
    }
);

group!(
    /// Time resolution.
    Time,
    TimePatch,
    time {
        /// A civil time in a DST gap.
        dst_gap: DstGap => Time,
        /// A civil time in a DST overlap.
        dst_overlap: DstOverlap => Time,
        /// A birth without a time.
        unknown_time: UnknownTime => Time,
        /// Which Delta T.
        delta_t: DeltaT => Sky,
        /// UT1 less UTC: none, the IERS's, or a given value.
        dut1: Dut1 => Time,
        /// Which sidereal time turns the Earth.
        sidereal_time: SiderealTime => Sky,
    }
);

group!(
    /// Dasha computation.
    Dasha, DashaPatch, dasha {
        /// How the first period is balanced.
        balance: Balance => Rule,
        /// The year length per system.
        year_length: BTreeMap<DashaSystem, YearLength> => Rule,
        /// The default depth per system.
        depth: BTreeMap<DashaSystem, Depth> => Rule,
        /// A seed outside a conditional cycle.
        /// lint: knob-has-a-reader — `teistro_dasha::Rules::of`, which is handed the `dasha` group and reads it as `settings.seed_overflow`.
        seed_overflow: SeedOverflow => Rule,
        /// How Ashtottari's lords share the nakshatras (crux C5).
        /// lint: knob-has-a-reader — `teistro_dasha::Rules::of`, which is handed the `dasha` group and reads it as `settings.ashtottari_grouping`.
        ashtottari_grouping: AshtottariGrouping => Rule,
        /// How the birth period is divided among its sub-periods (crux C48).
        /// lint: knob-has-a-reader — `teistro_dasha::Rules::of`, which is handed the `dasha` group and reads it as `settings.birth_period`.
        birth_period: BirthPeriod => Rule,
        /// What a dasha answers past the end of its cycle (crux C48).
        after_cycle: AfterCycle => Rule,
        /// Which pada table a nakshatra takes in the Kalachakra dasha (crux C54).
        kalachakra_membership: KalachakraMembership => Rule,
        /// How the Kalachakra balance at birth is taken (crux C55).
        kalachakra_balance: KalachakraBalance => Rule,
        /// What follows the ninth Kalachakra mahadasha (crux C56).
        kalachakra_after_ninth: KalachakraAfterNinth => Rule,
        /// Which friendly signs make a dasha favourable (crux C79).
        shanta_sign: ShantaSign => Rule,
        /// How a rashi dasha finds the stronger lord of a dual-lorded sign
        /// (crux C51).
        dual_lord: DualLord => Rule,
        /// Where the rashi dashas that start from a stronger sign begin (crux
        /// C53).
        rashi_start: RashiStart => Rule,
        /// Where releasing from Daimon begins when Daimon shares Fortune's
        /// sign (crux C223).
        releasing_shared_sign: ReleasingSharedSign => Rule,
        /// Where the nodes take their years in a night birth's firdaria
        /// (crux C225).
        firdaria_nodes: FirdariaNodes => Rule,
        /// How the decennials divide a period below the second level (crux
        /// C228).
        decennial_division: DecennialDivision => Rule,
        /// How the Karaka dasha divides a period among its sub-periods
        /// (crux C737).
        karaka_division: GrahaDivision => Rule,
        /// What makes a graha the strongest where a dasha begins from the
        /// strongest and names no measure (crux C735).
        strongest_graha: StrongestGraha => Rule,
        /// Which of several bodies in one place takes the share in an
        /// ayurdaya dasha's antardashas (crux C741).
        ayurdaya_place: AntardashaPlace => Rule,
        /// How the Naisargika dasha divides a period among its sub-periods
        /// (crux C743).
        naisargika_division: NaisargikaDivision => Rule,
        /// Which span's years an ayurdaya dasha runs (crux C742).
        ayurdaya_span: AyurdayaSpan => Rule,
        /// Whose readings of the reductions give the ayurdaya dashas'
        /// years (crux C742).
        ayurdaya_readings: AyurdayaReadings => Rule,
    }
);

group!(
    /// Jaimini conventions.
    Jaimini,
    JaiminiPatch,
    jaimini {
        /// Seven or eight chara karakas: which ranks the Atmakaraka the
        /// karakamsha is read from.
        chara_karakas: CharaKarakas => Rule,
        /// Which karaka each rank carries when eight are counted (crux
        /// C101): the ranking is the sky's, the names are this.
        eight_karaka_order: EightKarakaOrder => Rule,
        /// The nodes' co-lordship of Scorpio and Aquarius: which lords of
        /// the 6th, 8th and 12th the Brahma graha is chosen among (C126).
        node_co_lordship: NodeCoLordship => Rule,
        /// Which rule finds the Brahma graha (crux C128).
        brahma: BrahmaRule => Rule,
        /// Whether a graha's arudha moves as a bhava's does (crux C132).
        graha_arudha_exception: GrahaArudhaException => Rule,
        /// Which karaka each rank carries when seven are counted (crux
        /// C2061): the ranking is the sky's, the names are this.
        seven_karaka_order: SevenKarakaOrder => Rule,
    }
);

group!(
    /// Transits read from the natal Moon (Phaladeepika ch. 26;
    /// `03-design/gochar.md`).
    Gochar,
    GocharPatch,
    gochar {
        /// The nodes' vedha (crux C136).
        node_vedha: NodeVedha => Rule,
        /// Whether the nodes obstruct another graha's transit (crux C137).
        node_obstruction: NodeObstruction => Rule,
        /// How many bindus in a graha's own Ashtakavarga make its transit
        /// good (crux C141; `03-design/gochar-ashtakavarga.md`).
        ashtakavarga_good_from: AshtakavargaGoodFrom => Rule,
        /// Where a transit is seen from (crux C778).
        transit_centre: TransitCentre => Sky,
    }
);

group!(
    /// A period read for each sign (`03-design/rashifal.md`).
    Rashifal,
    RashifalPatch,
    rashifal {
        /// The three descending thresholds a baseline score's overall is
        /// banded by: Tithipatra's 75, 50 and 30 (SR-124), since no text
        /// bands a score (`03-design/rashifal.md`, "The score band").
        score_bands: ScoreBands => Rule,
    }
);

group!(
    /// Krishnamurti Paddhati (`03-design/kp.md`).
    Kp,
    KpPatch,
    kp {
        /// How many ruling planets are counted (crux C150).
        ruling_count: RulingCount => Rule,
        /// The day the ruling day lord belongs to (crux C151).
        day_lord_day: DayLordDay => Rule,
        /// When a node joins the ruling planets as an agent (crux C152).
        node_rulers: NodeRulers => Rule,
        /// Which ruling planets a retrograde planet rejects (crux C153).
        retrograde_rejection: RetrogradeRejection => Rule,
    }
);

group!(
    /// Aspects.
    Aspect,
    AspectPatch,
    aspect {
        /// The nodes' aspects.
        node_aspects: NodeAspects => Rule,
        /// The sphuta drishti table's key.
        drishti_table: String => Rule,
    }
);

group!(
    /// Planetary state.
    State,
    StatePatch,
    state {
        /// The combustion orb table's key.
        combustion_orbs: String => Rule,
        /// Which count of the ghatis of birth the Sayanadi avasthas add (crux C78).
        sayanadi_ghatis: SayanadiGhatis => Rule,
        /// The numbers Rahu and Ketu multiply by in the Sayanadi avasthas (crux C78).
        sayanadi_nodes: SayanadiNodes => Rule,
        /// Which degrees of a sign are pushkara bhagas (crux C767).
        pushkara_bhaga: PushkaraBhaga => Rule,
        /// Where Saturn's marana karaka sthana is (crux C768).
        marana_saturn: MaranaSaturn => Rule,
        /// Where Venus's marana karaka sthana is (crux C768).
        marana_venus: MaranaVenus => Rule,
    }
);

group!(
    /// Strength.
    Strength,
    StrengthPatch,
    strength {
        /// Which components the Shadbala counts: the six of BPHS ch. 27.
        bala_scheme: BalaScheme => Rule,
        /// How the Ashtakavarga's Ekadhipatya reduction treats a co-ruled sign
        /// beside an occupied one (crux C60).
        ekadhipatya: Ekadhipatya => Rule,
        /// Where the Ashtakavarga's reductions and pindas are made (crux C59).
        shodhana: Shodhana => Rule,
        /// Where Jupiter gives the Moon a bindu in the Ashtakavarga (crux
        /// C144; `03-design/gochar-ashtakavarga.md`).
        moon_bindu_from_jupiter: MoonBinduFromJupiter => Rule,
        /// How the Vimshopaka scores a graha in a varga (crux C63).
        vimshopaka: Vimshopaka => Rule,
        /// How the Shadbala's Saptavargaja scores a varga (crux C64).
        saptavargaja: Saptavargaja => Rule,
        /// How the Shadbala's Nathonnatha measures the hour (crux C65).
        nathonnatha: Nathonnatha => Rule,
        /// Which night a birth before sunrise is measured in (crux C65).
        pre_dawn_night: PreDawnNight => Rule,
        /// Where the Sun's Ayana bala is counted (crux C66).
        sun_ayana: SunAyana => Rule,
        /// Whether the Moon's Paksha bala is doubled (crux C66).
        moon_paksha: MoonPaksha => Rule,
        /// The Sun's and the Moon's Cheshta balas (crux C66).
        luminary_cheshta: LuminaryCheshta => Rule,
        /// The declination the Ayana bala reads (crux C66).
        kranti: Kranti => Rule,
        /// Whose weekdays the Abda and Masa lords are (crux C67).
        kaala_lords: KaalaLords => Rule,
        /// The kendras the Dig bala measures from (crux C68).
        dig: DigKendras => Rule,
        /// How the Drik bala weighs the drishtis received (crux C69).
        drik: Drik => Rule,
        /// The natural strengths (crux C71).
        naisargika: Naisargika => Rule,
        /// The rupas a graha's Shadbala must reach (crux C71).
        required_rupas: RequiredRupas => Rule,
        /// Which decanate gives each gender its Drekkana bala (crux C72).
        drekkana: Drekkana => Rule,
        /// Which grahas are benefics for Paksha and Drik (crux C69).
        benefics: Benefics => Rule,
        /// The mean elements the Cheshta bala reads (crux C70).
        cheshta: Cheshta => Rule,
        /// Whether grahas at war gain and lose the Yuddha bala (crux C70).
        yuddha: Yuddha => Rule,
        /// How the Ishta and Kashta phalas are read (crux C76).
        ishta_kashta: IshtaKashta => Rule,
        /// Which division's bhavas the Bhava bala measures (crux C73).
        bhava_division: BhavaBalaDivision => Rule,
        /// How the Bhava bala's Dig bala reads a bhava's sign class (crux C73).
        bhava_dig: BhavaDig => Rule,
        /// How the Bhava bala weighs the drishtis a bhava receives (crux C74).
        bhava_drishti: BhavaDrishti => Rule,
        /// Whether the Bhava bala adds the special rules (crux C75).
        bhava_special_rules: BhavaSpecialRules => Rule,
    }
);

group!(
    /// Divisional charts.
    Vargas,
    VargasPatch,
    vargas {
        /// The convention for an unattested D-N.
        unattested_dn: UnattestedDn => Rule,
        /// The scheme each catalogued chart is computed by, where it is not
        /// `PARASHARA`: a named scheme of the texts or a consumer's own rows
        /// (`03-design/varga-schemes.md`). A chart left out is BPHS ch. 6's.
        /// Every reader of a chart under these settings asks
        /// `teistro_vargas::Scheme::chosen`.
        schemes: BTreeMap<Varga, VargaChoice> => Rule,
    }
);

group!(
    /// Calendars.
    Calendars, CalendarsPatch, calendars {
        /// The civil calendar of a request's dates.
        ///
        /// Read by `ts_chart_found`, which is what the deferral this knob
        /// carried was waiting for: "a binding builds a chart from a
        /// settings document alone".
        civil_calendar: Calendar => Rule,
        /// The lunar month system.
        lunar_month: LunarMonth => Rule,
        /// How the sixty-year cycle names a lunar year.
        samvatsara: SamvatsaraCount => Rule,
        /// The era numbers a date carries.
        /// lint: knob-has-a-reader — as `civil_calendar` above — `crates/calendar` renders the eras a
        /// caller asks for rather than the ones the settings name.
        eras: BTreeSet<Era> => Rule,
        /// How near a sankranti, in minutes of the Sun's motion, one of a
        /// lunar month's two new moons may fall before the month is
        /// called `marginal`: near enough that a reckoning with another
        /// Sun may count its sankrantis, and so its kind, otherwise
        /// (SR-92, C1902).
        ///
        /// Read by the almanac into every month's `margin`. Nought calls
        /// no month marginal. [`DEFAULT_MARGINAL_MONTH_MINUTES`] covers
        /// every month Nepal's committee and a drik almanac decided
        /// differently in AD 1900–2100, the widest by 6.3 hours.
        marginal_month_minutes: u32 => Rule,
        /// Where the Kirat Yele Sambat's year 1 stands, the year a
        /// Bikram Sambat date carries in `era.KIRAT_YELE`: `KIRATI`, the
        /// count printed today (5086 from Magh 1, 2082 BS; *Kantipur*,
        /// 15 January 2026; G. Rai, *Indigenous Voice*, 2021), or
        /// `YALAMBAR`, J. C. Regmi's count from Yalambar's accession
        /// (*Kirat Yele Samvat Abhilekh*, 2010), 1281 years fewer
        /// (`calendars/bikram-sambat.md` §Kirat Yele Sambat).
        kirat_yele: KiratYeleEpoch => Rule,
    }
);

group!(
    /// The provider.
    Provider,
    ProviderPatch,
    provider {
        /// The override policy (ADR-0013).
        overrides: OverridePolicy => Sky,
        /// The built-in ephemeris tier.
        /// lint: knob-has-a-reader — Phase 3's built-in ephemeris, which is what has tiers to choose
        /// between; a provider a caller supplies declares its own.
        tier: Tier => Sky,
        /// How many ephemeris cells a context remembers, so that a batch
        /// asks for each of them once.
        ///
        /// Nought is off. Any other number is the capacity, and past it
        /// the memo stops admitting and keeps serving what it holds, so
        /// the memory is a number chosen here rather than a function of
        /// how long a batch ran. One knob and not two, so that no pair of
        /// settings can disagree about whether the memo exists.
        ///
        /// It is honoured only over a provider that declares
        /// `deterministic`: a memo answers a repeat with the first
        /// answer, which is right exactly when the provider would have
        /// answered the same again.
        /// [`DEFAULT_CACHE_CELLS`] is what a fifty-day almanac range
        /// needs, measured (`03-design/batch-and-parallelism-measured.md`).
        cache_cells: u32 => Form,
    }
);

group!(
    /// Output.
    Output,
    OutputPatch,
    output {
        /// The rounding contract.
        precision: Precision => Form,
    }
);

/// Every knob, complete; built only by resolving a profile.
///
/// ```
/// use teistro_core::settings::root;
///
/// let settings = root();
/// // One document: it reads back from its own canonical form, to the hash.
/// let back: teistro_core::settings::Settings = serde_json::from_str(&settings.canonical_json()).unwrap();
/// assert_eq!(back.hash(), settings.hash());
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// The document's schema version.
    pub schema: u16,
    /// The frame.
    pub frame: Frame,
    /// House systems.
    pub houses: Houses,
    /// The local day.
    pub day: Day,
    /// The daily panchanga.
    pub panchanga: Panchanga,
    /// Time resolution.
    pub time: Time,
    /// Dashas.
    pub dasha: Dasha,
    /// Jaimini.
    pub jaimini: Jaimini,
    /// Transits.
    pub gochar: Gochar,
    /// A period read for each sign.
    pub rashifal: Rashifal,
    /// Krishnamurti Paddhati.
    pub kp: Kp,
    /// Aspects.
    pub aspect: Aspect,
    /// State.
    pub state: State,
    /// Strength.
    pub strength: Strength,
    /// Vargas.
    pub vargas: Vargas,
    /// Calendars.
    pub calendars: Calendars,
    /// The provider.
    pub provider: Provider,
    /// Output.
    pub output: Output,
}

impl Settings {
    /// Every knob of every group, as `("group", "knob")`, in document
    /// order.
    ///
    /// One list, so that a rule about the knobs has a single source of
    /// truth rather than a copy — `cargo xtask check-lints` uses it for
    /// the `knob-has-a-reader` rule. The group names are the field names
    /// a settings document uses, and
    /// `every_group_of_the_document_is_in_the_knob_paths` holds this
    /// list to the document itself, so a group added without a line here
    /// fails rather than going unwatched.
    ///
    /// ```
    /// use teistro_core::settings::Settings;
    ///
    /// let paths = Settings::knob_paths();
    /// assert!(paths.contains(&("frame", "ayanamsha")) && paths.contains(&("day", "sunrise")));
    /// assert_eq!(paths.first(), Some(&("frame", "zodiac")));
    /// ```
    #[must_use]
    pub fn knob_paths() -> Vec<(&'static str, &'static str)> {
        Settings::knobs()
            .into_iter()
            .map(|knob| (knob.group, knob.name))
            .collect()
    }
}

/// Every group of the document with its knobs and what each moves, in
/// document order: the one list [`Settings::knobs`] and
/// [`Settings::knob_paths`] read.
const fn groups() -> [(&'static str, &'static [&'static str], &'static [Moves]); 17] {
    [
        ("frame", Frame::KNOBS, Frame::MOVES),
        ("houses", Houses::KNOBS, Houses::MOVES),
        ("day", Day::KNOBS, Day::MOVES),
        ("panchanga", Panchanga::KNOBS, Panchanga::MOVES),
        ("time", Time::KNOBS, Time::MOVES),
        ("dasha", Dasha::KNOBS, Dasha::MOVES),
        ("jaimini", Jaimini::KNOBS, Jaimini::MOVES),
        ("gochar", Gochar::KNOBS, Gochar::MOVES),
        ("rashifal", Rashifal::KNOBS, Rashifal::MOVES),
        ("kp", Kp::KNOBS, Kp::MOVES),
        ("aspect", Aspect::KNOBS, Aspect::MOVES),
        ("state", State::KNOBS, State::MOVES),
        ("strength", Strength::KNOBS, Strength::MOVES),
        ("vargas", Vargas::KNOBS, Vargas::MOVES),
        ("calendars", Calendars::KNOBS, Calendars::MOVES),
        ("provider", Provider::KNOBS, Provider::MOVES),
        ("output", Output::KNOBS, Output::MOVES),
    ]
}

/// A patch: every group's knobs optional.
///
/// ```
/// use teistro_core::settings::{Node, SettingsPatch};
///
/// // What a request sends: only the knobs it changes.
/// let patch: SettingsPatch = serde_json::from_str(r#"{"frame": {"node": "TRUE"}}"#).unwrap();
/// assert_eq!(patch.frame.node, Some(Node::True));
/// assert!(serde_json::from_str::<SettingsPatch>(r#"{"frames": {}}"#).is_err());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct SettingsPatch {
    /// The frame.
    pub frame: FramePatch,
    /// House systems.
    pub houses: HousesPatch,
    /// The local day.
    pub day: DayPatch,
    /// The daily panchanga.
    pub panchanga: PanchangaPatch,
    /// Time resolution.
    pub time: TimePatch,
    /// Dashas.
    pub dasha: DashaPatch,
    /// Jaimini.
    pub jaimini: JaiminiPatch,
    /// Transits.
    pub gochar: GocharPatch,
    /// A period read for each sign.
    pub rashifal: RashifalPatch,
    /// Krishnamurti Paddhati.
    pub kp: KpPatch,
    /// Aspects.
    pub aspect: AspectPatch,
    /// State.
    pub state: StatePatch,
    /// Strength.
    pub strength: StrengthPatch,
    /// Vargas.
    pub vargas: VargasPatch,
    /// Calendars.
    pub calendars: CalendarsPatch,
    /// The provider.
    pub provider: ProviderPatch,
    /// Output.
    pub output: OutputPatch,
}

impl SettingsPatch {
    /// Whether the patch sets nothing.
    ///
    /// Compared with the empty patch rather than group by group: the list
    /// it replaced had left out `panchanga`, so a patch setting only a
    /// panchanga knob called itself empty, and a comparison cannot miss a
    /// group added later.
    ///
    /// ```
    /// use teistro_core::settings::{Node, SettingsPatch};
    ///
    /// let mut patch = SettingsPatch::default();
    /// assert!(patch.is_empty());
    /// patch.frame.node = Some(Node::Mean);
    /// assert!(!patch.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == SettingsPatch::default()
    }

    /// A patch read from its JSON, refused down to the knob under
    /// `settings` (`settings.houses.chalit_system`): what a context's
    /// options, a derived context and the agent server's `settings`
    /// argument are each read with, so the three refuse one patch alike.
    ///
    /// ```
    /// use teistro_core::settings::{Node, SettingsPatch};
    ///
    /// assert_eq!(SettingsPatch::from_json(r#"{"frame": {"node": "MEAN"}}"#)?.frame.node, Some(Node::Mean));
    /// let refused = SettingsPatch::from_json(r#"{"frame": {"nod": "MEAN"}}"#).unwrap_err();
    /// assert_eq!(refused.field(), Some("settings.frame.nod"));
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` at the field for text that is not a patch: an unknown
    /// group or knob, or a value outside the knob's type.
    pub fn from_json(json: &str) -> Result<SettingsPatch, Error> {
        crate::strict::deserialize_str(json, "settings")
    }
}

impl Settings {
    /// Applies a patch, knob by knob, and returns the result.
    ///
    /// ```
    /// use teistro_core::settings::{Node, SettingsPatch, root};
    ///
    /// let mut patch = SettingsPatch::default();
    /// patch.frame.node = Some(Node::True);
    /// let patched = root().patched(&patch);
    /// assert_eq!(patched.frame.node, Node::True);
    /// // Only what the patch names moves.
    /// assert_eq!(patched.houses, root().houses);
    /// ```
    #[must_use]
    pub fn patched(self, patch: &SettingsPatch) -> Settings {
        self.patched_noting(patch, &mut |_, _| {})
    }

    /// [`Settings::patched`], telling `note` each knob it sets as
    /// `("group", "knob")`: the merge and its accounting in one pass
    /// ([`Origins`]).
    fn patched_noting(
        mut self,
        patch: &SettingsPatch,
        note: &mut dyn FnMut(&'static str, &'static str),
    ) -> Settings {
        self.frame
            .apply_noting(&patch.frame, &mut |knob| note("frame", knob));
        self.houses
            .apply_noting(&patch.houses, &mut |knob| note("houses", knob));
        self.day
            .apply_noting(&patch.day, &mut |knob| note("day", knob));
        self.panchanga
            .apply_noting(&patch.panchanga, &mut |knob| note("panchanga", knob));
        self.time
            .apply_noting(&patch.time, &mut |knob| note("time", knob));
        self.dasha
            .apply_noting(&patch.dasha, &mut |knob| note("dasha", knob));
        self.jaimini
            .apply_noting(&patch.jaimini, &mut |knob| note("jaimini", knob));
        self.gochar
            .apply_noting(&patch.gochar, &mut |knob| note("gochar", knob));
        self.rashifal
            .apply_noting(&patch.rashifal, &mut |knob| note("rashifal", knob));
        self.kp
            .apply_noting(&patch.kp, &mut |knob| note("kp", knob));
        self.aspect
            .apply_noting(&patch.aspect, &mut |knob| note("aspect", knob));
        self.state
            .apply_noting(&patch.state, &mut |knob| note("state", knob));
        self.strength
            .apply_noting(&patch.strength, &mut |knob| note("strength", knob));
        self.vargas
            .apply_noting(&patch.vargas, &mut |knob| note("vargas", knob));
        self.calendars
            .apply_noting(&patch.calendars, &mut |knob| note("calendars", knob));
        self.provider
            .apply_noting(&patch.provider, &mut |knob| note("provider", knob));
        self.output
            .apply_noting(&patch.output, &mut |knob| note("output", knob));
        self
    }

    /// **The one merge**: a patch laid over these settings, knob by knob,
    /// and the result checked once by the coherence rules under
    /// `profile`: the merge [`Profile::resolve`] makes over a profile, and
    /// what a context derived with a patch and the agent server's
    /// per-call scope resolve with (`03-design/settings-and-profiles.md`,
    /// "Per call").
    ///
    /// ```
    /// use teistro_core::settings::{Node, ProfileId, SettingsPatch, SunriseConvention, root};
    ///
    /// let mut patch = SettingsPatch::default();
    /// patch.frame.node = Some(Node::Mean);
    /// let derived = root().derived(&patch, ProfileId::new("mine"))?;
    /// assert_eq!((derived.settings.frame.node, derived.profile.as_str()), (Node::Mean, "mine"));
    /// // A refusal names the knob under `settings`.
    /// patch.day.sunrise = Some(SunriseConvention::Custom { altitude_deg: 95.0 });
    /// assert_eq!(root().derived(&patch, ProfileId::new("mine")).unwrap_err().field(), Some("settings.day.sunrise"));
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the first knob a coherence error names, under
    /// `settings`, with every error in the message.
    pub fn derived(self, patch: &SettingsPatch, profile: ProfileId) -> Result<Resolved, Error> {
        // A value handed in whole: its knobs are a document's, and what the
        // patch sets the request's ([`Resolved::derived`] keeps a resolved
        // value's layers instead).
        let mut origins = Origins::under(Layer::Document);
        self.laid(patch, &mut origins, Layer::Request, None)
            .validated_with(profile, origins)
            .map_err(|diagnostics| Error::from(diagnostics).under("settings"))
    }

    /// The canonical document: keys in code-point order, no whitespace,
    /// every knob present ([`crate::envelope::canonical_json`]).
    ///
    /// ```
    /// use teistro_core::settings::root;
    ///
    /// let canonical = root().canonical_json();
    /// // Keys in code-point order and no line breaks: one text for one value.
    /// assert!(canonical.starts_with("{\"aspect\":{") && !canonical.contains('\n'));
    /// ```
    #[must_use]
    pub fn canonical_json(&self) -> String {
        crate::envelope::canonical_json(self)
    }

    /// The hash of the canonical document.
    ///
    /// ```
    /// use teistro_core::settings::{Node, SettingsPatch, root};
    ///
    /// let mut patch = SettingsPatch::default();
    /// patch.frame.node = Some(Node::True);
    /// // A knob that differs is a hash that differs; equal settings hash alike.
    /// assert_ne!(root().patched(&patch).hash(), root().hash());
    /// assert_eq!(root().hash(), root().hash());
    /// assert_eq!(root().hash().to_string().len(), 64);
    /// ```
    #[must_use]
    pub fn hash(&self) -> Hash {
        Hash::of(self.canonical_json().as_bytes())
    }

    /// A settings document parsed and checked for coherence.
    ///
    /// ```
    /// use teistro_core::settings::{Settings, root};
    ///
    /// let resolved = Settings::from_json(&root().canonical_json())?;
    /// assert_eq!(resolved.profile.as_str(), "document");
    /// // A document of another schema is refused by rule.
    /// let older = root().canonical_json().replace("\"schema\":1", "\"schema\":0");
    /// assert_eq!(Settings::from_json(&older).unwrap_err().items[0].rule, "schema");
    /// # Ok::<(), teistro_core::settings::Diagnostics>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A document that does not parse, or one with coherence errors.
    pub fn from_json(json: &str) -> Result<Resolved, Diagnostics> {
        // The message names the knob that failed, `houses.chalit_system`,
        // since a diagnostic's fields are the static knob names.
        let settings: Settings =
            crate::strict::deserialize_str(json, "").map_err(|e| Diagnostics {
                items: vec![Diagnostic::error("json", e.message, &[])],
            })?;
        if settings.schema != SCHEMA {
            return Err(Diagnostics {
                items: vec![Diagnostic {
                    severity: Severity::Error,
                    rule: "schema",
                    message: format!("settings schema {} is not {SCHEMA}", settings.schema),
                    fields: vec!["schema"],
                }],
            });
        }
        settings.validated(ProfileId::new("document"))
    }

    /// Runs the coherence rules and splits them into errors and warnings.
    ///
    /// ```
    /// use teistro_core::settings::{ProfileId, SunriseConvention, root};
    ///
    /// let mut settings = root();
    /// settings.day.sunrise = SunriseConvention::Custom { altitude_deg: 91.0 };
    /// let refused = settings.validated(ProfileId::new("mine")).unwrap_err();
    /// assert_eq!(refused.errors().map(|d| d.rule).collect::<Vec<_>>(), ["sunrise-altitude"]);
    /// ```
    ///
    /// # Errors
    ///
    /// The diagnostics when any rule is an error.
    pub fn validated(self, profile: ProfileId) -> Result<Resolved, Diagnostics> {
        self.validated_with(profile, Origins::under(Layer::Document))
    }

    /// [`Settings::validated`] with the accounting of where each knob came
    /// from, which resolution kept as it laid the patches.
    pub(crate) fn validated_with(
        self,
        profile: ProfileId,
        origins: Origins,
    ) -> Result<Resolved, Diagnostics> {
        let diagnostics = coherence(&self);
        if diagnostics.iter().any(|d| d.severity == Severity::Error) {
            return Err(Diagnostics { items: diagnostics });
        }
        Ok(Resolved {
            settings: self,
            warnings: diagnostics,
            profile,
            origins,
        })
    }
}

/// How bad a finding is.
///
/// ```
/// use teistro_core::settings::Severity;
///
/// assert_eq!(serde_json::to_string(&Severity::Warning).unwrap(), r#""WARNING""#);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    /// The settings cannot be used.
    Error,
    /// Worth a look; recorded in provenance.
    Warning,
}

/// One coherence finding.
///
/// ```
/// use teistro_core::settings::{Severity, SunriseConvention, root};
/// use teistro_core::settings::ProfileId;
///
/// let mut settings = root();
/// settings.day.sunrise = SunriseConvention::Custom { altitude_deg: f64::NAN };
/// let refused = settings.validated(ProfileId::new("mine")).unwrap_err();
/// let finding = &refused.items[0];
/// assert_eq!((finding.severity, finding.rule, finding.fields.as_slice()), (Severity::Error, "sunrise-altitude", ["day.sunrise"].as_slice()));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Diagnostic {
    /// Error or warning.
    pub severity: Severity,
    /// The rule's id.
    pub rule: &'static str,
    /// What is wrong.
    pub message: String,
    /// The knobs involved.
    pub fields: Vec<&'static str>,
}

impl Diagnostic {
    fn error(
        rule: &'static str,
        message: impl Into<String>,
        fields: &[&'static str],
    ) -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            rule,
            message: message.into(),
            fields: fields.to_vec(),
        }
    }

    fn warning(
        rule: &'static str,
        message: impl Into<String>,
        fields: &[&'static str],
    ) -> Diagnostic {
        Diagnostic {
            severity: Severity::Warning,
            rule,
            message: message.into(),
            fields: fields.to_vec(),
        }
    }
}

/// Every finding of a validation.
///
/// ```
/// use teistro_core::error::Error;
/// use teistro_core::settings::{ProfileId, SunriseConvention, root};
///
/// let mut settings = root();
/// settings.day.sunrise = SunriseConvention::Custom { altitude_deg: 91.0 };
/// let refused = settings.validated(ProfileId::new("mine")).unwrap_err();
/// assert!(refused.to_string().starts_with("sunrise-altitude [day.sunrise]: "));
/// // As an error, it names the first finding's knob.
/// assert_eq!(Error::from(refused).field(), Some("day.sunrise"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Diagnostics {
    /// The findings, in rule order.
    pub items: Vec<Diagnostic>,
}

impl Diagnostics {
    /// The errors.
    ///
    /// ```
    /// use teistro_core::settings::{Atmosphere, ProfileId, Sunrise, SunriseConvention, root};
    ///
    /// let mut settings = root();
    /// // Two findings: no refraction to take the air, and an air no surface has.
    /// settings.day.sunrise = SunriseConvention::Atmospheric { which: Sunrise::CentreNoRefraction, air: Atmosphere::given(50.0, 15.0) };
    /// let refused = settings.validated(ProfileId::new("mine")).unwrap_err();
    /// assert_eq!(refused.errors().count(), 2);
    /// ```
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.items.iter().filter(|d| d.severity == Severity::Error)
    }
}

impl core::fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (i, d) in self.items.iter().enumerate() {
            if i > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{} [{}]: {}", d.rule, d.fields.join(", "), d.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostics {}

impl From<Diagnostics> for Error {
    fn from(diagnostics: Diagnostics) -> Error {
        let mut error = Error::new(Status::InvalidArg, diagnostics.to_string());
        if let Some(first) = diagnostics.errors().next() {
            if let Some(field) = first.fields.first() {
                error = error.with_field(*field);
            }
        }
        error
    }
}

/// A resolved, validated settings value.
///
/// ```
/// use teistro_core::settings::{DEFAULT_PROFILE, Profile, SettingsPatch};
///
/// let resolved = Profile::shipped(DEFAULT_PROFILE).unwrap().resolve(&SettingsPatch::default()).unwrap();
/// assert_eq!(resolved.profile.as_str(), DEFAULT_PROFILE);
/// assert!(resolved.warnings.is_empty());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Resolved {
    /// The settings.
    pub settings: Settings,
    /// The warnings, which every result's provenance carries
    /// ([`Resolved::provenance`]).
    pub warnings: Vec<Diagnostic>,
    /// The profile it came from.
    pub profile: ProfileId,
    /// Where each knob came from: the layer that set it and its citation
    /// ([`Resolved::explained`]).
    pub origins: Origins,
}

impl Resolved {
    /// The provenance every result computed under these settings starts
    /// from: the versions, the profile, the settings' hash, the input's,
    /// and **the settings' warnings**.
    ///
    /// One constructor, because the warnings were computed at resolution
    /// and reached no result: each producer built its own stamp from the
    /// profile and the hash, and none carried them, so a warning such as
    /// `siddhanta-topocentric` was said to nobody. A warning crosses as
    /// its rule in the envelope's spelling (`SIDDHANTA_TOPOCENTRIC`), with
    /// its message and the knobs it names as slots.
    ///
    /// ```
    /// use teistro_core::envelope::{Hash, Version};
    /// use teistro_core::settings::{Profile, SettingsPatch};
    ///
    /// let mut patch = SettingsPatch::default();
    /// patch.frame.centre = Some(teistro_core::settings::Centre::Topocentric);
    /// patch.frame.siddhanta = Some(teistro_core::settings::Siddhanta::surya(
    ///     teistro_core::settings::SuryaBija::None,
    /// ));
    /// let resolved = Profile::shipped("parashari-classical").unwrap().resolve(&patch).unwrap();
    /// let stamp = resolved.provenance(Version::new(0, 1, 0), Hash::of(b"input"));
    /// assert_eq!(stamp.profile, "parashari-classical");
    /// assert!(stamp.warnings.iter().any(|w| w.code == "SIDDHANTA_TOPOCENTRIC"));
    /// ```
    #[must_use]
    pub fn provenance(&self, sdk_version: Version, input_hash: Hash) -> Provenance {
        let mut provenance = Provenance::new(
            sdk_version,
            CALCULATION_VERSION,
            crate::catalogue::SCHEMA_VERSION,
            self.profile.as_str(),
            self.settings.hash(),
            input_hash,
        );
        for warning in &self.warnings {
            provenance.warn(
                &warning.rule.to_uppercase().replace('-', "_"),
                None,
                vec![
                    (String::from("message"), warning.message.clone()),
                    (String::from("fields"), warning.fields.join(",")),
                ],
            );
        }
        provenance
    }
}

/// The coherence rules, every finding returned.
/// What the settings refuse in a sunrise convention: a custom altitude
/// that is not one, an air given to a convention that does not refract,
/// and an air outside what the Earth's surface has seen.
/// The largest fixed semidiameter a horizon may name, arcminutes: a degree,
/// twice the Moon's largest.
pub const LARGEST_FIXED_SEMIDIAMETER_ARCMIN: f64 = 60.0;

/// How far a horizon may be raised or lowered, degrees: Bennett's formula
/// has its pole at −4.4°, and no skyline stands higher than this above a
/// place's ideal horizon.
pub const HORIZON_ALTITUDE_DEG: core::ops::RangeInclusive<f64> = -4.0..=20.0;

fn horizon_coherence(model: &HorizonModel, out: &mut Vec<Diagnostic>) {
    if let DiscSize::Fixed {
        semidiameter_arcmin,
    } = model.disc_size
    {
        if !(semidiameter_arcmin.is_finite()
            && semidiameter_arcmin > 0.0
            && semidiameter_arcmin <= LARGEST_FIXED_SEMIDIAMETER_ARCMIN)
        {
            out.push(Diagnostic::error(
                "horizon-semidiameter",
                format!(
                    "a fixed semidiameter is a positive number of arcminutes up to {LARGEST_FIXED_SEMIDIAMETER_ARCMIN}"
                ),
                &["day.horizon"],
            ));
        }
    }
    if let Radii::Custom { sun_km, moon_km } = model.radii {
        if !(sun_km.is_finite() && sun_km > 0.0 && moon_km.is_finite() && moon_km > 0.0) {
            out.push(Diagnostic::error(
                "horizon-radii",
                "a custom radius is a positive, finite number of kilometres",
                &["day.horizon"],
            ));
        }
    }
    if !(model.altitude_deg.is_finite() && HORIZON_ALTITUDE_DEG.contains(&model.altitude_deg)) {
        out.push(Diagnostic::error(
            "horizon-altitude",
            format!(
                "the horizon's altitude is a finite number of degrees from {} to {}",
                HORIZON_ALTITUDE_DEG.start(),
                HORIZON_ALTITUDE_DEG.end()
            ),
            &["day.horizon"],
        ));
    }
}

fn sunrise_coherence(convention: SunriseConvention, out: &mut Vec<Diagnostic>) {
    if let SunriseConvention::Custom { altitude_deg } = convention {
        if !altitude_deg.is_finite() || altitude_deg.abs() > 90.0 {
            out.push(Diagnostic::error(
                "sunrise-altitude",
                "a custom sunrise altitude is a finite number of degrees within -90 to 90",
                &["day.sunrise"],
            ));
        }
    }
    if let SunriseConvention::Atmospheric { which, air } = convention {
        if !which.refracts() {
            out.push(Diagnostic::error(
                "sunrise-air-refracts",
                format!(
                    "{} does not refract, so it has no air to give; name UPPER_LIMB_REFRACTION, LOWER_LIMB_REFRACTION or CENTRE_REFRACTION",
                    which.key()
                ),
                &["day.sunrise"],
            ));
        }
        if !air.is_valid() {
            out.push(Diagnostic::error(
                "sunrise-air",
                format!(
                    "the air's pressure is {} to {} hPa and its temperature {} to {} °C, or left out for the standard at the place",
                    AIR_PRESSURE_HPA.start(),
                    AIR_PRESSURE_HPA.end(),
                    AIR_TEMPERATURE_C.start(),
                    AIR_TEMPERATURE_C.end()
                ),
                &["day.sunrise"],
            ));
        }
    }
}

/// The coherence rule a sidereal zodiac of date on the mean ayanamsha
/// breaks (SR-118): a warning, which a shipped profile may carry only with
/// a citation on `frame.ayanamsha_basis`.
const AYANAMSHA_MEAN_BASIS: &str = "ayanamsha-mean-basis";

/// What the settings warn of in the ayanamsha's basis (SR-118).
fn basis_coherence(s: &Settings, out: &mut Vec<Diagnostic>) {
    // Under modern astronomy every position is referred to the true
    // equinox of date, apparent or geometric alike: the nutation is
    // applied either way. A mean ayanamsha taken from it leaves the
    // nutation in longitude, up to 17.2 arcseconds, in every sidereal
    // longitude, which the IAE's "true ayanamsa" definition does not
    // (SR-100, SR-118, crux C1750). A classical sky has no nutation, so
    // its basis is moot. A profile that wants it anyway cites why.
    if s.frame.zodiac == Zodiac::Sidereal
        && s.frame.siddhanta == Siddhanta::Drik
        && s.frame.ayanamsha_basis == AyanamshaBasis::Mean
    {
        out.push(Diagnostic::warning(
            AYANAMSHA_MEAN_BASIS,
            "positions are referred to the true equinox of date, nutation applied; less the mean ayanamsha every sidereal longitude keeps the nutation in longitude, up to 17.2 arcseconds (set frame.ayanamsha_basis to TRUE, the IAE's nirayana longitude)",
            &["frame.ayanamsha_basis", "frame.positions"],
        ));
    }
}

/// A custom UT1 less UTC is finite and within the bound leap seconds keep.
fn dut1_coherence(dut1: Dut1, out: &mut Vec<Diagnostic>) {
    if let Dut1::Custom { seconds } = dut1 {
        if !(seconds.is_finite() && seconds.abs() <= DUT1_BOUND_SECONDS) {
            out.push(Diagnostic::error(
                "dut1-bound",
                format!(
                    "UT1 less UTC is a finite number of seconds within ±{DUT1_BOUND_SECONDS}, which leap seconds keep it within"
                ),
                &["time.dut1"],
            ));
        }
    }
}

/// Each threshold of `rashifal.score_bands`, as a refusal names it.
const SCORE_BAND_FIELDS: [&str; 3] = [
    "rashifal.score_bands[0]",
    "rashifal.score_bands[1]",
    "rashifal.score_bands[2]",
];

/// What the settings refuse of the score bands: a threshold past 100,
/// which no score reaches, and one not below the threshold before it,
/// which would leave a band empty.
fn score_bands_coherence(bands: ScoreBands, out: &mut Vec<Diagnostic>) {
    let mut above = None;
    for (threshold, field) in bands.0.into_iter().zip(SCORE_BAND_FIELDS) {
        if threshold > 100 {
            out.push(Diagnostic::error(
                "score-band-range",
                format!("a score band's threshold is 0 to 100, not {threshold}"),
                &[field],
            ));
        }
        if let Some(previous) = above
            && threshold >= previous
        {
            out.push(Diagnostic::error(
                "score-bands-descending",
                format!(
                    "the score bands' thresholds descend strictly, and {threshold} is not below {previous}"
                ),
                &[field],
            ));
        }
        above = Some(threshold);
    }
}

/// Every custom ayanamsha the settings name, the frame's and the lagna
/// table's own, has finite numbers (SR-112): the one thing that can be
/// checked without an ephemeris.
fn ayanamsha_coherence(s: &Settings, out: &mut Vec<Diagnostic>) {
    let lagna = match s.panchanga.lagna_table {
        LagnaTable::Sphere { ayanamsha } => Some(ayanamsha),
        LagnaTable::Chart => None,
    };
    for (choice, field) in [
        (Some(s.frame.ayanamsha), "frame.ayanamsha"),
        (lagna, "panchanga.lagna_table"),
    ] {
        if choice.is_some_and(|choice| !choice.is_finite()) {
            out.push(Diagnostic::error(
                "custom-ayanamsha-finite",
                "a custom ayanamsha needs a finite epoch, value, rate and longitude",
                &[field],
            ));
        }
    }
}

/// Every varga scheme the settings choose, checked as a chart would check
/// it, the first fault of each named by its field (`varga-schemes.md`).
fn scheme_coherence(s: &Settings, out: &mut Vec<Diagnostic>) {
    for (varga, choice) in &s.vargas.schemes {
        if let Err(refused) = choice.check(*varga) {
            out.push(Diagnostic::error(
                "varga-scheme",
                format!(
                    "{} (field `{}`)",
                    refused.message,
                    refused.field().unwrap_or("vargas.schemes")
                ),
                &["vargas.schemes"],
            ));
        }
    }
}

fn coherence(s: &Settings) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    ayanamsha_coherence(s, &mut out);
    sunrise_coherence(s.day.sunrise, &mut out);
    horizon_coherence(&s.day.horizon, &mut out);
    dut1_coherence(s.time.dut1, &mut out);
    score_bands_coherence(s.rashifal.score_bands, &mut out);
    if s.frame.nakshatra_scheme == NakshatraScheme::TwentyEight {
        let seeded: Vec<&DashaSystem> = s
            .dasha
            .depth
            .keys()
            .filter(|d| d.attributes().family == crate::catalogue::DashaFamily::Udu)
            .collect();
        if !seeded.is_empty() {
            out.push(Diagnostic::error(
                "nakshatra-scheme-dasha",
                format!(
                    "the twenty-eight nakshatra scheme has no seed map for {}",
                    seeded
                        .iter()
                        .map(|d| d.key())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                &["frame.nakshatra_scheme", "dasha.depth"],
            ));
        }
    }
    if s.houses.polar_policy == PolarPolicy::Clamp
        && s.houses.placement_system == HouseSystem::WholeSign
    {
        out.push(Diagnostic::warning(
            "polar-policy-unused",
            "the clamp policy never applies to whole-sign placements",
            &["houses.polar_policy", "houses.placement_system"],
        ));
    }
    if matches!(
        s.frame.siddhanta,
        Siddhanta::Surya { .. } | Siddhanta::Vakya
    ) && s.frame.centre == Centre::Topocentric
    {
        out.push(Diagnostic::warning(
            "siddhanta-topocentric",
            "the classical model is geocentric; the topocentric correction is applied on top and stamped",
            &["frame.siddhanta", "frame.centre"],
        ));
    }
    basis_coherence(s, &mut out);
    scheme_coherence(s, &mut out);
    // The text's sunrise is the centre on the geometric horizon, in local
    // mean time; asked for another convention, the provider declines and
    // the SDK solves it over the text's Sun, which is neither the text's
    // sunrise nor a modern one (`03-design/nepal-day-measured.md` §4).
    if matches!(
        s.frame.siddhanta,
        Siddhanta::Surya {
            sunrise: SuryaSunrise::Text,
            ..
        }
    ) && s.day.sunrise != SunriseConvention::from(Sunrise::CentreNoRefraction)
    {
        out.push(Diagnostic::warning(
            "siddhanta-text-sunrise-convention",
            "the text's sunrise is the centre on the geometric horizon; another convention is solved over the text's Sun, which is neither the text's sunrise nor a modern one (set frame.siddhanta.sunrise to MODERN for a modern one)",
            &["frame.siddhanta", "day.sunrise"],
        ));
    }
    let root = profiles::root();
    for (system, length) in &s.dasha.year_length {
        if root
            .dasha
            .year_length
            .get(system)
            .is_some_and(|classical| classical != length)
        {
            out.push(Diagnostic::warning(
                "year-length-convention",
                format!(
                    "{system} runs at {length}, not its classical year; recorded as a convention"
                ),
                &["dasha.year_length"],
            ));
        }
    }
    if s.frame.zodiac == Zodiac::Tropical && s.frame.ayanamsha != root.frame.ayanamsha {
        out.push(Diagnostic::warning(
            "ayanamsha-ignored",
            "the ayanamsha is ignored under the tropical zodiac",
            &["frame.zodiac", "frame.ayanamsha"],
        ));
    }
    out
}

/// A citation on a profile's defaults.
///
/// ```
/// use teistro_core::settings::{DEFAULT_PROFILE, Profile};
///
/// // Every default a shipped profile sets says why.
/// let profile = Profile::shipped(DEFAULT_PROFILE).unwrap();
/// assert!(profile.sources.iter().all(|cited| !cited.knob.is_empty() && !cited.source.text.is_empty()));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Citation {
    /// What the citation is for.
    pub knob: &'static str,
    /// The source.
    pub source: Source,
}

impl Citation {
    /// A citation.
    ///
    /// ```
    /// use teistro_core::catalogue::Source;
    /// use teistro_core::settings::Citation;
    ///
    /// const CENTRE: Citation = Citation::new("frame.centre", Source::new("BPHS", "ch. 3"));
    /// assert_eq!(CENTRE.source.to_string(), "BPHS ch. 3");
    /// ```
    #[must_use]
    pub const fn new(knob: &'static str, source: Source) -> Citation {
        Citation { knob, source }
    }
}

/// A profile's confidence, for the root and the shipped ones.
///
/// ```
/// use teistro_core::catalogue::Mark;
/// use teistro_core::settings::{DEFAULT_PROFILE, Profile, ProfileMark};
///
/// let mark: ProfileMark = Profile::shipped(DEFAULT_PROFILE).unwrap().mark;
/// assert_ne!(mark, Mark::Shape);
/// ```
pub type ProfileMark = Mark;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    fn shipped(id: &str) -> Profile {
        Profile::shipped(id).unwrap_or_else(|| panic!("{id}"))
    }

    /// The rules a settings document breaks with this patch.
    fn rules_broken(patch: &SettingsPatch) -> Vec<&'static str> {
        match shipped(DEFAULT_PROFILE).resolve(patch) {
            Ok(_) => Vec::new(),
            Err(diagnostics) => diagnostics.errors().map(|d| d.rule).collect(),
        }
    }

    #[test]
    fn a_horizon_model_is_refused_outside_what_it_can_mean() {
        let with = |model: HorizonModel| {
            let mut patch = SettingsPatch::default();
            patch.day.horizon = Some(model);
            rules_broken(&patch)
        };
        let standard = HorizonModel::STANDARD;
        assert!(with(standard).is_empty(), "the standard horizon");
        let fixed = |semidiameter_arcmin| HorizonModel {
            disc_size: DiscSize::Fixed {
                semidiameter_arcmin,
            },
            ..standard
        };
        assert!(with(fixed(16.0)).is_empty(), "a 16 arcminute disc");
        assert!(
            with(fixed(LARGEST_FIXED_SEMIDIAMETER_ARCMIN)).is_empty(),
            "the largest fixed disc"
        );
        for bad in [0.0, -1.0, 61.0, f64::NAN] {
            assert_eq!(with(fixed(bad)), ["horizon-semidiameter"], "{bad}");
        }
        let radii = |sun_km, moon_km| HorizonModel {
            radii: Radii::Custom { sun_km, moon_km },
            ..standard
        };
        assert!(with(radii(696_000.0, 1_738.0)).is_empty(), "radii in range");
        for (sun, moon) in [(0.0, 1_738.0), (696_000.0, -1.0), (f64::INFINITY, 1_738.0)] {
            assert_eq!(with(radii(sun, moon)), ["horizon-radii"], "{sun} {moon}");
        }
        let altitude = |altitude_deg| HorizonModel {
            altitude_deg,
            ..standard
        };
        for good in [-4.0, 0.0, 20.0] {
            assert!(with(altitude(good)).is_empty(), "{good}");
        }
        for bad in [-4.5, 20.5, f64::NAN] {
            assert_eq!(with(altitude(bad)), ["horizon-altitude"], "{bad}");
        }
        // Every rule a model breaks is named, not only the first.
        let all = HorizonModel {
            disc_size: DiscSize::Fixed {
                semidiameter_arcmin: 0.0,
            },
            radii: Radii::Custom {
                sun_km: 0.0,
                moon_km: 0.0,
            },
            altitude_deg: 90.0,
            ..standard
        };
        assert_eq!(
            with(all),
            ["horizon-semidiameter", "horizon-radii", "horizon-altitude"]
        );
    }

    #[test]
    fn a_horizon_models_key_names_every_field() {
        assert_eq!(
            HorizonModel::STANDARD.key(),
            "BENNETT/ACTUAL/IAU_2015/BODY/NONE/0"
        );
        let every = HorizonModel {
            refraction: RefractionFormula::Saemundsson,
            disc_size: DiscSize::MeanDistance,
            radii: Radii::Custom {
                sun_km: 696_000.0,
                moon_km: 1_738.0,
            },
            latitude: RisingLatitude::Zero,
            dip: Dip::Geometric,
            altitude_deg: 1.5,
        };
        assert_eq!(
            every.key(),
            "SAEMUNDSSON/MEAN_DISTANCE/CUSTOM[696000,1738]/ZERO/GEOMETRIC/1.5"
        );
        assert_eq!(HorizonModel::default(), HorizonModel::STANDARD);
    }

    #[test]
    fn a_custom_dut1_is_refused_past_the_bound_leap_seconds_keep() {
        let with = |dut1: Dut1| {
            let mut patch = SettingsPatch::default();
            patch.time.dut1 = Some(dut1);
            rules_broken(&patch)
        };
        for good in [
            Dut1::None,
            Dut1::Iers,
            Dut1::Custom { seconds: -0.9 },
            Dut1::Custom { seconds: 0.3 },
        ] {
            assert!(with(good).is_empty(), "{good:?}");
        }
        for seconds in [0.91, -2.0, f64::NAN] {
            assert_eq!(with(Dut1::Custom { seconds }), ["dut1-bound"], "{seconds}");
        }
    }

    /// The rules a settings document breaks with this sunrise convention.
    fn sunrise_rules(convention: SunriseConvention) -> Vec<&'static str> {
        let mut patch = SettingsPatch::default();
        patch.day.sunrise = Some(convention);
        match shipped(DEFAULT_PROFILE).resolve(&patch) {
            Ok(_) => Vec::new(),
            Err(diagnostics) => diagnostics.errors().map(|d| d.rule).collect(),
        }
    }

    #[test]
    fn an_air_is_named_in_json_with_either_part_left_to_the_standard() {
        let atmospheric = |json: &str| -> SunriseConvention {
            crate::strict::deserialize_str(json, "")
                .unwrap_or_else(|e| panic!("{json}: {}", e.message))
        };
        let standard =
            atmospheric(r#"{"kind":"ATMOSPHERIC","which":"UPPER_LIMB_REFRACTION","air":{}}"#);
        assert_eq!(
            standard,
            SunriseConvention::Atmospheric {
                which: Sunrise::UpperLimbRefraction,
                air: Atmosphere::STANDARD
            }
        );
        let partial = atmospheric(
            r#"{"kind":"ATMOSPHERIC","which":"LOWER_LIMB_REFRACTION","air":{"temperature_c":-4.5}}"#,
        );
        let SunriseConvention::Atmospheric { air, .. } = partial else {
            panic!("{partial:?}");
        };
        assert_eq!((air.pressure_hpa, air.temperature_c), (None, Some(-4.5)));
        // A part the document does not know is refused, not ignored.
        assert!(
            crate::strict::deserialize_str::<SunriseConvention>(
                r#"{"kind":"ATMOSPHERIC","which":"UPPER_LIMB_REFRACTION","air":{"humidity":0.4}}"#,
                ""
            )
            .is_err()
        );
        // And it writes back as it reads.
        let written = serde_json::to_string(&partial).unwrap();
        assert_eq!(atmospheric(&written), partial);
    }

    #[test]
    fn an_air_resolves_at_the_place_as_the_standard_atmosphere_does() {
        let at = |metres: f64| crate::quantity::Altitude::try_new(metres).unwrap();
        let sea = Atmosphere::STANDARD.at(at(0.0));
        assert_eq!((sea.pressure_hpa, sea.temperature_c), (1013.25, 15.0));
        // The ICAO atmosphere's tabulated pressures, 898.76 hPa at 1000 m,
        // 795.01 at 2000 m and 540.48 at 5000 m, are at geopotential
        // heights; an observer's height is geometric, which puts the
        // formula a few hundredths under the table at these heights (and
        // within 0.15 hPa of Teimeris's own to 8848 m, 0.006′ of
        // refraction).
        for (metres, hpa) in [(1000.0, 898.76), (2000.0, 795.01), (5000.0, 540.48)] {
            let air = Atmosphere::STANDARD.at(at(metres));
            assert!((air.pressure_hpa - hpa).abs() < 0.3, "{metres} m: {air:?}");
            assert_eq!(
                air.temperature_c.to_bits(),
                STANDARD_TEMPERATURE_C.to_bits()
            );
        }
        // What is given is kept at any height.
        let given = Atmosphere::given(987.0, 4.5).at(at(3000.0));
        assert_eq!((given.pressure_hpa, given.temperature_c), (987.0, 4.5));
        let half = Atmosphere {
            pressure_hpa: None,
            temperature_c: Some(30.0),
        }
        .at(at(0.0));
        assert_eq!((half.pressure_hpa, half.temperature_c), (1013.25, 30.0));
    }

    #[test]
    fn a_warning_reaches_the_stamp_and_a_refusal_names_its_field() {
        let root = shipped(DEFAULT_PROFILE);
        let base = root
            .resolve(&SettingsPatch::default())
            .expect("the default resolves");
        let other = [Ayanamsha::Raman, Ayanamsha::Lahiri]
            .into_iter()
            .map(Into::into)
            .find(|ayanamsha| *ayanamsha != base.settings.frame.ayanamsha)
            .expect("two ayanamshas differ");
        let mut patch = SettingsPatch::default();
        patch.frame.zodiac = Some(Zodiac::Tropical);
        patch.frame.ayanamsha = Some(other);
        let resolved = root.resolve(&patch).expect("a warning is not a refusal");
        assert!(
            resolved
                .warnings
                .iter()
                .any(|w| w.rule == "ayanamsha-ignored")
        );
        let stamp = resolved.provenance(Version::new(0, 1, 0), Hash::of(b"input"));
        let warned = stamp
            .warnings
            .iter()
            .find(|w| w.code == "AYANAMSHA_IGNORED")
            .expect("the warning is stamped");
        assert!(
            warned
                .slots
                .iter()
                .any(|(slot, value)| slot == "fields" && value == "frame.zodiac,frame.ayanamsha")
        );

        assert_eq!(
            sunrise_rules(SunriseConvention::Custom { altitude_deg: 91.0 }),
            ["sunrise-altitude"]
        );
        let mut patch = SettingsPatch::default();
        patch.day.sunrise = Some(SunriseConvention::Custom {
            altitude_deg: f64::NAN,
        });
        let refused = Error::from(root.resolve(&patch).expect_err("not an altitude"));
        assert_eq!(
            (refused.status, refused.field()),
            (Status::InvalidArg, Some("day.sunrise"))
        );
    }

    #[test]
    fn the_settings_refuse_an_air_that_cannot_be_applied() {
        let air = |which, air| SunriseConvention::Atmospheric { which, air };
        assert!(
            sunrise_rules(air(Sunrise::UpperLimbRefraction, Atmosphere::STANDARD)).is_empty(),
            "{:?}",
            sunrise_rules(air(Sunrise::UpperLimbRefraction, Atmosphere::STANDARD))
        );
        assert!(
            sunrise_rules(air(
                Sunrise::LowerLimbRefraction,
                Atmosphere::given(1100.0, -90.0)
            ))
            .is_empty(),
            "{:?}",
            sunrise_rules(air(
                Sunrise::LowerLimbRefraction,
                Atmosphere::given(1100.0, -90.0)
            ))
        );
        // The centre with refraction takes an air as the limbs do: the
        // horizon gives it one, so the settings must not refuse it.
        assert_eq!(
            sunrise_rules(air(
                Sunrise::CentreRefraction,
                Atmosphere::given(860.0, 12.0)
            )),
            Vec::<&str>::new()
        );
        assert_eq!(
            sunrise_rules(air(Sunrise::CentreNoRefraction, Atmosphere::STANDARD)),
            ["sunrise-air-refracts"]
        );
        for wrong in [
            Atmosphere::given(99.0, 15.0),
            Atmosphere::given(1101.0, 15.0),
            Atmosphere::given(1013.25, 61.0),
            Atmosphere::given(1013.25, -91.0),
            Atmosphere::given(f64::NAN, 15.0),
        ] {
            assert_eq!(
                sunrise_rules(air(Sunrise::UpperLimbRefraction, wrong)),
                ["sunrise-air"],
                "{wrong:?}"
            );
        }
    }

    #[test]
    fn every_shipped_profile_resolves_and_hashes_stably() {
        let mut hashes = BTreeMap::new();
        for id in SHIPPED_PROFILES {
            let resolved = shipped(id)
                .resolve(&SettingsPatch::default())
                .unwrap_or_else(|e| panic!("{id}: {e}"));
            assert!(
                resolved.warnings.is_empty(),
                "{id}: {:?}",
                resolved.warnings
            );
            let json = resolved.settings.canonical_json();
            assert!(!json.contains(' '), "{id}: canonical form has whitespace");
            assert!(
                json.starts_with("{\"aspect\":{\"drishti_table\""),
                "{id}: canonical form is not key-sorted: {}",
                &json[..40]
            );
            let back: Settings = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(back, resolved.settings);
            assert_eq!(
                back.canonical_json(),
                json,
                "{id}: canonical form is a fixed point"
            );
            hashes.insert(id, resolved.settings.hash().to_string());
        }
        assert_eq!(hashes.len(), SHIPPED_PROFILES.len());
        let distinct: BTreeSet<&String> = hashes.values().collect();
        assert_eq!(distinct.len(), SHIPPED_PROFILES.len(), "{hashes:?}");
    }

    #[test]
    fn an_alias_opens_its_profile_and_is_not_a_profile_of_its_own() {
        for (alias, id) in PROFILE_ALIASES {
            assert!(SHIPPED_PROFILES.contains(&id), "{alias} names {id}");
            assert!(!SHIPPED_PROFILES.contains(&alias), "{alias} is an alias");
            let resolved = shipped(alias)
                .resolve(&SettingsPatch::default())
                .unwrap_or_else(|e| panic!("{e}"));
            let named = shipped(id)
                .resolve(&SettingsPatch::default())
                .unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(resolved.settings.hash(), named.settings.hash());
            // The answer names the id, never the alias it was asked by.
            assert_eq!(resolved.profile.as_str(), id);
        }
    }

    #[test]
    fn patches_apply_in_order_and_change_the_hash() {
        let base = shipped("nepali-drik")
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{e}"));
        let mut patch = SettingsPatch::default();
        patch.frame.node = Some(Node::True);
        let patched = shipped("nepali-drik")
            .resolve(&patch)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_ne!(base.settings.hash(), patched.settings.hash());
        assert_eq!(patched.settings.frame.node, Node::True);
        assert_eq!(patched.settings.frame.zodiac, base.settings.frame.zodiac);
        let again = patched.settings.clone().patched(&SettingsPatch::default());
        assert_eq!(again, patched.settings);
        assert!(
            SettingsPatch::default().is_empty(),
            "{:?}",
            SettingsPatch::default()
        );
        assert!(!patch.is_empty(), "`patch` is empty");
    }

    /// A patch that sets one knob of any one group is not empty. The
    /// group-by-group check this replaced had left out `panchanga`.
    #[test]
    fn a_patch_setting_any_one_group_is_not_empty() {
        let root = serde_json::to_value(root()).expect("a settings document");
        for (group, knob) in Settings::knob_paths() {
            let value = root[group][knob].clone();
            let patch: SettingsPatch =
                serde_json::from_value(serde_json::json!({ group: { knob: value } }))
                    .unwrap_or_else(|e| panic!("{group}.{knob}: {e}"));
            assert!(!patch.is_empty(), "{group}.{knob}");
        }
    }

    /// Each number of a custom ayanamsha and a custom sunrise altitude is
    /// held to being finite (and the altitude to ±90°), one at a time, and
    /// finite ones pass.
    #[test]
    fn every_custom_number_is_checked_on_its_own() {
        let rules = |settings: Settings| -> Vec<&'static str> {
            coherence(&settings).iter().map(|d| d.rule).collect()
        };
        let ayanamsha = |epoch_jd_tt, value_deg, rate_deg_per_year| {
            let mut settings = root();
            settings.frame.ayanamsha = AyanamshaChoice::Custom {
                epoch_jd_tt,
                value_deg,
                rate_deg_per_year,
            };
            rules(settings)
        };
        for (epoch, value, rate) in [
            (f64::NAN, 23.0, 0.01),
            (2_451_545.0, f64::INFINITY, 0.01),
            (2_451_545.0, 23.0, f64::NEG_INFINITY),
        ] {
            assert!(ayanamsha(epoch, value, rate).contains(&"custom-ayanamsha-finite"));
        }
        assert!(!ayanamsha(2_451_545.0, 23.0, 0.01).contains(&"custom-ayanamsha-finite"));
        // The precessed and held-star forms (SR-112), in the frame and in
        // the lagna table's own zodiac.
        let choice = |choice: AyanamshaChoice| {
            let mut frame = root();
            frame.frame.ayanamsha = choice;
            let mut lagna = root();
            lagna.panchanga.lagna_table = LagnaTable::Sphere { ayanamsha: choice };
            let fields = |settings: Settings| -> Vec<Vec<&'static str>> {
                coherence(&settings)
                    .into_iter()
                    .filter(|d| d.rule == "custom-ayanamsha-finite")
                    .map(|d| d.fields.clone())
                    .collect()
            };
            (fields(frame), fields(lagna))
        };
        let precessed = |epoch_jd_tt, value_deg| AyanamshaChoice::Precessed {
            epoch_jd_tt,
            value_deg,
            basis: AyanamshaBasis::True,
            precession: Precession::Iau2006,
        };
        let held = |longitude_deg| AyanamshaChoice::Anchored {
            star: Star::Spica,
            longitude_deg,
        };
        for bad in [
            precessed(f64::NAN, 23.0),
            precessed(2_435_553.5, f64::INFINITY),
            held(f64::NAN),
        ] {
            assert_eq!(
                choice(bad),
                (
                    vec![vec!["frame.ayanamsha"]],
                    vec![vec!["panchanga.lagna_table"]]
                ),
                "{bad:?}"
            );
        }
        for good in [precessed(2_435_553.5, 23.25), held(180.0)] {
            assert_eq!(choice(good), (vec![], vec![]), "{good:?}");
        }
        let sunrise = |altitude_deg| {
            let mut settings = root();
            settings.day.sunrise = SunriseConvention::Custom { altitude_deg };
            rules(settings)
        };
        for altitude in [f64::NAN, 90.5, -91.0] {
            assert!(
                sunrise(altitude).contains(&"sunrise-altitude"),
                "{altitude}"
            );
        }
        assert!(!sunrise(-6.0).contains(&"sunrise-altitude"));
    }

    #[test]
    fn score_bands_are_refused_by_the_threshold_at_fault() {
        let refused = |bands: [u8; 3]| {
            let mut settings = root();
            settings.rashifal.score_bands = ScoreBands(bands);
            settings
                .validated(ProfileId::new("mine"))
                .map(drop)
                .map_err(|found| {
                    found
                        .errors()
                        .map(|d| (d.rule, d.fields.clone()))
                        .collect::<Vec<_>>()
                })
        };
        assert_eq!(refused([75, 50, 30]), Ok(()));
        assert_eq!(refused([100, 99, 0]), Ok(()));
        assert_eq!(
            refused([101, 50, 30]),
            Err(vec![("score-band-range", vec!["rashifal.score_bands[0]"])])
        );
        assert_eq!(
            refused([75, 75, 30]),
            Err(vec![(
                "score-bands-descending",
                vec!["rashifal.score_bands[1]"]
            )])
        );
        assert_eq!(
            refused([75, 50, 60]),
            Err(vec![(
                "score-bands-descending",
                vec!["rashifal.score_bands[2]"]
            )])
        );
        // Every fault is said, each by its own threshold.
        assert_eq!(
            refused([30, 50, 200]),
            Err(vec![
                ("score-bands-descending", vec!["rashifal.score_bands[1]"]),
                ("score-band-range", vec!["rashifal.score_bands[2]"]),
                ("score-bands-descending", vec!["rashifal.score_bands[2]"]),
            ])
        );
        // As a patch, under `settings`; and a list not of three bytes is
        // refused when it is read.
        let mut patch = SettingsPatch::default();
        patch.rashifal.score_bands = Some(ScoreBands([75, 80, 30]));
        assert_eq!(
            root()
                .derived(&patch, ProfileId::new("mine"))
                .unwrap_err()
                .field(),
            Some("settings.rashifal.score_bands[1]")
        );
        for bands in [
            "[75, 50]",
            "[75, 50, 30, 10]",
            "[256, 50, 30]",
            "[75.5, 50, 30]",
        ] {
            let json = format!(r#"{{"rashifal": {{"score_bands": {bands}}}}}"#);
            assert!(SettingsPatch::from_json(&json).is_err(), "{bands}");
        }
        assert_eq!(
            SettingsPatch::from_json(r#"{"rashifal": {"score_bands": [80, 60, 40]}}"#)
                .unwrap()
                .rashifal
                .score_bands,
            Some(ScoreBands([80, 60, 40]))
        );
    }

    #[test]
    fn coherence_rules_fire_with_fields() {
        let mut patch = SettingsPatch::default();
        patch.frame.ayanamsha = Some(AyanamshaChoice::Custom {
            epoch_jd_tt: f64::NAN,
            value_deg: 23.0,
            rate_deg_per_year: 0.0139,
        });
        patch.frame.nakshatra_scheme = Some(NakshatraScheme::TwentyEight);
        patch.houses.polar_policy = Some(PolarPolicy::Clamp);
        let error = shipped("nepali-drik").resolve(&patch).unwrap_err();
        let rules: Vec<&str> = error.items.iter().map(|d| d.rule).collect();
        assert!(rules.contains(&"custom-ayanamsha-finite"), "{rules:?}");
        assert!(rules.contains(&"nakshatra-scheme-dasha"), "{rules:?}");
        assert!(rules.contains(&"polar-policy-unused"), "{rules:?}");
        assert_eq!(error.errors().count(), 2);
        let as_error: Error = error.into();
        assert_eq!(as_error.status, Status::InvalidArg);
        assert_eq!(as_error.field(), Some("frame.ayanamsha"));

        let mut warned = SettingsPatch::default();
        warned.dasha.year_length = Some(
            [(DashaSystem::Vimshottari, YearLength::Savana360)]
                .into_iter()
                .collect(),
        );
        warned.frame.siddhanta = Some(Siddhanta::surya(SuryaBija::NepalCommittee));
        let resolved = shipped("nepali-drik")
            .resolve(&warned)
            .unwrap_or_else(|e| panic!("{e}"));
        let rules: Vec<&str> = resolved.warnings.iter().map(|d| d.rule).collect();
        // nepali-drik's refracted sunrise is not the text's either.
        assert_eq!(
            rules,
            [
                "siddhanta-topocentric",
                "siddhanta-text-sunrise-convention",
                "year-length-convention"
            ]
        );
    }

    /// Whether a profile, or the base it patches, cites a knob.
    fn cites(profile: &Profile, knob: &str) -> bool {
        profile.sources.iter().any(|cited| cited.knob == knob)
            || profile
                .base
                .as_ref()
                .is_some_and(|base| cites(&shipped(base.as_str()), knob))
    }

    /// A profile whose resolved settings take the mean ayanamsha under a
    /// modern sidereal zodiac of date, and say nowhere why.
    fn uncited_mean_basis(profile: &Profile) -> bool {
        let resolved = profile
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{e}"));
        resolved
            .warnings
            .iter()
            .any(|warning| warning.rule == AYANAMSHA_MEAN_BASIS)
            && !cites(profile, "frame.ayanamsha_basis")
    }

    #[test]
    fn a_mean_basis_under_a_sky_of_date_warns() {
        let rules = |patch: &SettingsPatch| -> Vec<&'static str> {
            shipped("nepali-drik")
                .resolve(patch)
                .unwrap_or_else(|e| panic!("{e}"))
                .warnings
                .iter()
                .map(|d| d.rule)
                .collect()
        };
        // Fires: apparent positions, and geometric ones, whose equinox is
        // still of date.
        let mut mean = SettingsPatch::default();
        mean.frame.ayanamsha_basis = Some(AyanamshaBasis::Mean);
        assert_eq!(rules(&mean), [AYANAMSHA_MEAN_BASIS]);
        let resolved = shipped("nepali-drik")
            .resolve(&mean)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            resolved.warnings[0].fields,
            ["frame.ayanamsha_basis", "frame.positions"]
        );
        mean.frame.positions = Some(Positions::True);
        assert_eq!(rules(&mean), [AYANAMSHA_MEAN_BASIS]);
        // Does not fire: the true basis, a tropical zodiac, a classical sky.
        assert_eq!(rules(&SettingsPatch::default()), Vec::<&str>::new());
        let mut tropical = SettingsPatch::default();
        tropical.frame.ayanamsha_basis = Some(AyanamshaBasis::Mean);
        tropical.frame.zodiac = Some(Zodiac::Tropical);
        tropical.frame.ayanamsha = Some(profiles::root().frame.ayanamsha);
        assert_eq!(rules(&tropical), Vec::<&str>::new());
        let mut classical = SettingsPatch::default();
        classical.frame.ayanamsha_basis = Some(AyanamshaBasis::Mean);
        classical.frame.centre = Some(Centre::Geocentric);
        classical.frame.siddhanta = Some(Siddhanta::surya(SuryaBija::None));
        classical.day.sunrise = Some(Sunrise::CentreNoRefraction.into());
        assert_eq!(rules(&classical), Vec::<&str>::new());
    }

    /// SR-118: a shipped profile pairing positions of date with the mean
    /// ayanamsha fails unless it cites why; and the check itself fails
    /// such a profile, so a citation dropped later is caught.
    #[test]
    fn no_shipped_profile_takes_a_mean_basis_uncited() {
        for id in SHIPPED_PROFILES {
            assert!(!uncited_mean_basis(&shipped(id)), "{id}");
        }
        let mut patch = SettingsPatch::default();
        patch.frame.ayanamsha_basis = Some(AyanamshaBasis::Mean);
        let mut mean = Profile {
            patch,
            ..shipped(DEFAULT_PROFILE)
        };
        assert!(uncited_mean_basis(&mean), "the check fails a mean basis");
        mean.sources.push(Citation::new(
            "frame.ayanamsha_basis",
            crate::catalogue::Source::new("a test", "a cited mean basis"),
        ));
        assert!(!uncited_mean_basis(&mean), "and passes a cited one");
    }

    #[test]
    fn a_bija_is_named_or_counted_and_never_a_bare_switch() {
        let read = |json: &str| serde_json::from_str::<Siddhanta>(json);
        let named = read(r#"{"kind": "SURYA", "bija": {"kind": "NEPAL_COMMITTEE"}}"#)
            .unwrap_or_else(|e| panic!("{e}"));
        let counted = read(
            r#"{"kind": "SURYA", "bija": {"kind": "CUSTOM", "revolutions": {"moon_apsis": -4}}}"#,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        let revolutions = |siddhanta: Siddhanta| match siddhanta {
            Siddhanta::Surya { bija, .. } => bija.revolutions(),
            Siddhanta::Drik | Siddhanta::Vakya => panic!("not the text"),
        };
        assert_eq!(revolutions(named), revolutions(counted));
        assert_eq!(revolutions(named), Bija::NEPAL_COMMITTEE);
        // The switch the knob replaced said "a bija" without saying which,
        // and nothing read it.
        assert!(read(r#"{"kind": "SURYA", "bija": true}"#).is_err());
        assert!(
            read(r#"{"kind": "SURYA", "bija": {"kind": "CUSTOM", "revolutions": {"sun": 1}}}"#)
                .is_err()
        );
        // The committee's profile names the set, and the text's zodiac.
        let settings = shipped("nepali-committee")
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{e}"))
            .settings;
        assert!(matches!(
            settings.frame.siddhanta,
            Siddhanta::Surya {
                bija: SuryaBija::NepalCommittee,
                ..
            }
        ));
        assert_eq!(
            settings.frame.ayanamsha,
            AyanamshaChoice::from(crate::catalogue::Ayanamsha::Suryasiddhanta)
        );
    }

    #[test]
    fn the_texts_day_begins_at_its_own_sunrise_unless_asked_for_a_modern_one() {
        let read =
            |json: &str| serde_json::from_str::<Siddhanta>(json).unwrap_or_else(|e| panic!("{e}"));
        // A document written before the knob reads as the text's sunrise.
        assert_eq!(
            read(r#"{"kind": "SURYA", "bija": {"kind": "NONE"}}"#),
            Siddhanta::surya(SuryaBija::None)
        );
        assert_eq!(
            read(r#"{"kind": "SURYA", "bija": {"kind": "NONE"}, "sunrise": "MODERN"}"#),
            Siddhanta::Surya {
                bija: SuryaBija::None,
                sunrise: SuryaSunrise::Modern,
                planets: SuryaPlanets::Text,
                trig: SuryaTrig::Table,
            }
        );
        // The committee's sky: the text's limbs at a modern sunrise, the
        // disc's centre with standard refraction, and no warning for it.
        let committee = shipped("nepali-committee")
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            committee.settings.frame.siddhanta,
            Siddhanta::Surya {
                bija: SuryaBija::NepalCommittee,
                sunrise: SuryaSunrise::Modern,
                planets: SuryaPlanets::Modern {
                    ayanamsha: crate::catalogue::Ayanamsha::Lahiri.into(),
                },
                trig: SuryaTrig::Exact,
            }
        );
        assert_eq!(
            committee.settings.day.sunrise,
            SunriseConvention::from(Sunrise::CentreRefraction)
        );
        assert!(committee.warnings.is_empty(), "{:?}", committee.warnings);
        // The same sky at the text's sunrise under that convention is
        // neither, and says so.
        let mut text = SettingsPatch::default();
        text.frame.siddhanta = Some(Siddhanta::surya(SuryaBija::NepalCommittee));
        let warned = shipped("nepali-committee")
            .resolve(&text)
            .unwrap_or_else(|e| panic!("{e}"));
        let rules: Vec<&str> = warned.warnings.iter().map(|d| d.rule).collect();
        assert_eq!(rules, ["siddhanta-text-sunrise-convention"]);
    }

    #[test]
    fn the_texts_equations_take_its_sine_table_unless_asked_for_the_true_sine() {
        let read =
            |json: &str| serde_json::from_str::<Siddhanta>(json).unwrap_or_else(|e| panic!("{e}"));
        // A document written before the knob reads as the text's table.
        assert_eq!(
            read(r#"{"kind": "SURYA", "bija": {"kind": "NONE"}}"#),
            read(r#"{"kind": "SURYA", "bija": {"kind": "NONE"}, "trig": "TABLE"}"#)
        );
        let exact = read(r#"{"kind": "SURYA", "bija": {"kind": "NONE"}, "trig": "EXACT"}"#);
        assert_eq!(
            exact,
            Siddhanta::Surya {
                bija: SuryaBija::None,
                sunrise: SuryaSunrise::Text,
                planets: SuryaPlanets::Text,
                trig: SuryaTrig::Exact,
            }
        );
        let json = serde_json::to_string(&exact).unwrap_or_else(|e| panic!("{e}"));
        assert!(json.contains(r#""trig":"EXACT""#), "{json}");
        assert_eq!(read(&json), exact);
        assert!(
            serde_json::from_str::<Siddhanta>(
                r#"{"kind": "SURYA", "bija": {"kind": "NONE"}, "trig": "CHORD"}"#
            )
            .is_err()
        );
        assert_eq!(SuryaTrig::from_key("EXACT"), Some(SuryaTrig::Exact));
        assert_eq!(SuryaTrig::ALL, &[SuryaTrig::Table, SuryaTrig::Exact]);
    }

    #[test]
    fn documents_round_trip_and_refuse_other_schemas() {
        let resolved = shipped("kp-default")
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{e}"));
        let json = resolved.settings.canonical_json();
        let back = Settings::from_json(&json).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(back.settings, resolved.settings);
        assert_eq!(back.settings.houses.placement_system, HouseSystem::Placidus);
        assert_eq!(
            back.settings.houses.module_overrides.get("kp"),
            Some(&HouseSystem::Placidus)
        );
        let other = json.replacen("\"schema\":1", "\"schema\":2", 1);
        assert!(Settings::from_json(&other).is_err());
        assert!(Settings::from_json("{\"schema\":1}").is_err());
        assert!(
            shipped("nepali-drik")
                .sources
                .iter()
                .any(|c| c.knob == "houses.chalit_system")
        );
    }

    /// T6 of `03-design/ketaki.md` §9: the Ketaki profile and its Tilak
    /// sibling resolve without a warning, round-trip, cite every knob the
    /// plan's §3 names, and hash to the values pinned here, which move only
    /// on purpose.
    #[test]
    fn the_ketaki_profiles_resolve_cite_and_hash_as_pinned() {
        let resolve = |id: &str| {
            shipped(id)
                .resolve(&SettingsPatch::default())
                .unwrap_or_else(|e| panic!("{id}: {e}"))
        };
        let ketaki = resolve("ketaki");
        assert!(ketaki.warnings.is_empty(), "{:?}", ketaki.warnings);
        let frame = &ketaki.settings.frame;
        assert_eq!(frame.ayanamsha, Ayanamsha::Ketaki.into());
        // The root's conventions stand: the texts' own.
        let root = root();
        assert_eq!(frame.ayanamsha_basis, root.frame.ayanamsha_basis);
        assert_eq!((frame.node, frame.centre), (Node::Mean, Centre::Geocentric));
        assert_eq!(ketaki.settings.calendars.lunar_month, LunarMonth::Amanta);
        assert!(ketaki.settings.calendars.eras.contains(&Era::Shaka));
        assert_eq!(
            ketaki.settings.calendars.samvatsara,
            SamvatsaraCount::Chandramana
        );
        assert_eq!(ketaki.settings.day.day_boundary, DayBoundary::Sunrise);
        for knob in [
            "frame.ayanamsha",
            "frame.ayanamsha_basis",
            "calendars.lunar_month",
            "calendars.eras",
            "calendars.samvatsara",
            "day.day_boundary",
        ] {
            assert!(cites(&shipped("ketaki"), knob), "ketaki cites {knob}");
        }
        assert_eq!(shipped("ketaki").mark, Mark::Traditional);
        let tilak = resolve("tilak-revati");
        assert!(tilak.warnings.is_empty(), "{:?}", tilak.warnings);
        assert_eq!(
            tilak.settings.frame.ayanamsha,
            AyanamshaChoice::Anchored {
                star: crate::catalogue::Star::Revati,
                longitude_deg: 0.0,
            }
        );
        // Everything else is Ketaki's.
        let mut as_ketaki = tilak.settings.clone();
        as_ketaki.frame.ayanamsha = Ayanamsha::Ketaki.into();
        assert_eq!(as_ketaki, ketaki.settings);
        assert!(cites(&shipped("tilak-revati"), "frame.ayanamsha"));
        for resolved in [&ketaki, &tilak] {
            let json = resolved.settings.canonical_json();
            let back = Settings::from_json(&json).unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(back.settings, resolved.settings);
        }
        assert_eq!(
            [
                ketaki.settings.hash().to_string(),
                tilak.settings.hash().to_string()
            ],
            [KETAKI_HASH, TILAK_REVATI_HASH]
        );
    }

    /// The pinned hashes of the two profiles (T6).
    const KETAKI_HASH: &str = "0ab6e4fd00655cb03ac0a5fdf1c88951fcaa55095a36037604ddcf35a6b94bda";
    const TILAK_REVATI_HASH: &str =
        "953264da3fdcc6966d753006c48d63e3117a698b0e8fc6b152e9ecd5df172e3f";

    #[test]
    fn profiles_inherit_and_are_listed() {
        for id in SHIPPED_PROFILES {
            let profile = shipped(id);
            assert_eq!(profile.id.as_str(), id);
            assert!(profile.version >= 1);
        }
        assert!(Profile::shipped("nowhere").is_none());
        // The default patches the root and nothing else, which is what
        // makes "the texts as read, with nothing of one country's
        // practice in it" true rather than stated (ADR-0024).
        let classical = shipped(DEFAULT_PROFILE);
        assert_eq!(classical.base, None);
        let resolved = classical
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(resolved.profile.as_str(), "parashari-classical");
        // What the texts give it.
        assert_eq!(resolved.settings.houses.chalit_system, HouseSystem::Sripati);
        assert_eq!(resolved.settings.jaimini.chara_karakas, CharaKarakas::Eight);
        assert_eq!(
            resolved.settings.jaimini.eight_karaka_order,
            EightKarakaOrder::Parashara
        );
        assert_eq!(resolved.settings.state.combustion_orbs, "SURYA_SIDDHANTA");
        // What it must not have taken from anywhere else. The centre is
        // the one that moves numbers: up to 39 arcminutes of Moon, which
        // changes a mahadasha lord in one of the six charts the corpus
        // records both ways.
        assert_eq!(resolved.settings.frame.zodiac, Zodiac::Sidereal);
        assert_eq!(resolved.settings.frame.centre, Centre::Geocentric);
        assert_eq!(
            resolved.settings.calendars.civil_calendar,
            Calendar::Gregorian
        );
        assert!(!resolved.settings.calendars.eras.contains(&Era::NepalSambat));
        assert_eq!(
            resolved.settings.day.polar_day_policy,
            PolarDayPolicy::Undefined
        );

        // The product's charts keep every one of them, cited to the
        // engine whose charts they reproduce.
        let nepali = shipped("nepali-drik");
        let product = nepali
            .resolve(&SettingsPatch::default())
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(product.settings.frame.centre, Centre::Topocentric);
        assert_eq!(
            product.settings.calendars.civil_calendar,
            Calendar::BikramSambat
        );
        assert!(product.settings.calendars.eras.contains(&Era::NepalSambat));

        // A change of defaults is a change of hash, which is what makes
        // it visible in every result computed under it.
        assert_ne!(resolved.settings.hash(), product.settings.hash());
    }

    #[test]
    fn every_group_of_the_document_is_in_the_knob_paths() {
        // The list `knob_paths` holds is checked against the document
        // itself, so a group added to `Settings` without a line there
        // fails here rather than going unwatched by the lint that reads
        // it.
        let document = serde_json::to_value(root()).expect("a settings document");
        let object = document.as_object().expect("an object");
        let paths = Settings::knob_paths();
        let mut groups: Vec<&str> = paths.iter().map(|(group, _)| *group).collect();
        groups.sort_unstable();
        groups.dedup();
        for key in object.keys() {
            // `schema` is the document's version and not a knob.
            if key == "schema" {
                continue;
            }
            assert!(groups.contains(&key.as_str()), "{key} is not in knob_paths");
        }
        assert_eq!(
            groups.len(),
            object.len() - 1,
            "knob_paths names a group the document does not have"
        );
        // And every knob of every group is a field of that group.
        for (group, knob) in &paths {
            let fields = object
                .get(*group)
                .and_then(serde_json::Value::as_object)
                .unwrap_or_else(|| panic!("{group} is a group"));
            assert!(fields.contains_key(*knob), "{group}.{knob}");
        }
        assert_eq!(
            paths.len(),
            object
                .iter()
                .filter(|(key, _)| *key != "schema")
                .filter_map(|(_, value)| value.as_object())
                .map(serde_json::Map::len)
                .sum::<usize>(),
            "every knob of the document is named once"
        );
    }
}
