//! The sky view (`docs/03-design/sky-view.md`): at a place and an
//! instant, where every graha and every nakshatra's junction star stands
//! in the local sky, with the ecliptic and the horizon as point lists a
//! drawing joins.
//!
//! Every direction is topocentric, apparent and of date: the source's
//! apparent geocentric place less the station's position on the WGS 84
//! ellipsoid, turned by the local apparent sidereal time into altitude
//! and azimuth. Each altitude is given airless and refracted, the
//! refraction Sæmundsson's under a stated atmosphere.
//!
//! ```
//! use teistro_astro::sky_view::{EclipticPointKind, SkyViewOptions, sky_view};
//! use teistro_astro::{Completion, DeltaTModel};
//! use teistro_core::catalogue::{Graha, Nakshatra};
//! use teistro_core::quantity::{JulianDay, Place, Ut1};
//! use teistro_core::settings::{Node, OverridePolicy};
//! use teistro_port_ephemeris::TestProvider;
//!
//! let provider = TestProvider::new();
//! let delta_t = DeltaTModel::TableThenModel;
//! let source = Completion::new(&provider, OverridePolicy::SdkOnly, delta_t);
//! let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
//! // 06:00 UTC at Kathmandu on 2024-06-21.
//! let at = JulianDay::<Ut1>::literal(2_460_483.25);
//! let view = sky_view(&source, kathmandu, at, delta_t, Node::Mean, &SkyViewOptions::default())?;
//! // The nine grahas and the twenty-seven junction stars.
//! assert_eq!(view.grahas.len(), 9);
//! assert_eq!(view.stars.len(), 27);
//! // Refraction only lifts.
//! assert!(view.grahas.iter().all(|placed| placed.refracted_altitude_deg >= placed.altitude_deg));
//! // Rahu and Ketu face each other through the Earth.
//! let (rahu, ketu) = (view.graha(Graha::Rahu).ok_or("Rahu")?, view.graha(Graha::Ketu).ok_or("Ketu")?);
//! assert!((rahu.altitude_deg + ketu.altitude_deg).abs() < 1e-9);
//! // The ecliptic meets the horizon at the ascendant, in the east.
//! let rising = view.ecliptic.iter().find(|point| point.kind == EclipticPointKind::Ascendant).ok_or("a rising point")?;
//! assert!(rising.altitude_deg.abs() < 1e-9 && rising.azimuth_deg < 180.0);
//! assert_eq!(view.stars[13].nakshatra, Some(Nakshatra::Chitra));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use serde::Serialize;
use teistro_core::angle::{difference_deg, normalise_deg};
use teistro_core::catalogue::{Graha, Kind, Nakshatra, Star, write_in_full};
use teistro_core::error::Error;
use teistro_core::math;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_core::settings::Node;
use teistro_port_ephemeris::Body;

use crate::day_sky::horizontal;
use crate::delta_t::DeltaTModel;
use crate::iau;
use crate::iau::vector::{Vector3, c2s, s2c, sxp};
use crate::iau::{DEG2RAD, RAD2DEG};
use crate::scale::tt_of;
use crate::sky::{ApparentPositions, obliquity_parts, observer_at};
use crate::stars::{Options, place_of, yogatara};

/// The atmosphere the refraction is reckoned under: its pressure and
/// temperature, which scale Sæmundsson's standard refraction by
/// `(P / 1010) · (283 / (273 + T))` (Meeus, ch. 16). A pressure of
/// nothing is no air: the refracted altitude is the airless one.
///
/// ```
/// use teistro_astro::sky_view::Atmosphere;
///
/// assert_eq!(Atmosphere::STANDARD.factor(), 1.0);
/// let cold = Atmosphere::new(1_010.0, -17.0)?;
/// assert!(cold.factor() > 1.0);
/// assert_eq!(Atmosphere::new(0.0, 10.0)?.factor(), 0.0);
/// // An atmosphere no planet has is refused by its field.
/// assert_eq!(Atmosphere::new(-1.0, 10.0).unwrap_err().field(), Some("pressureHpa"));
/// assert_eq!(Atmosphere::new(1_010.0, 99.0).unwrap_err().field(), Some("temperatureC"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "SkyAtmosphere"))]
#[serde(rename_all = "camelCase")]
pub struct Atmosphere {
    /// The pressure at the place, hectopascals, 0 to 1100.
    pub pressure_hpa: f64,
    /// The temperature at the place, degrees Celsius, −90 to 60.
    pub temperature_c: f64,
}

impl Atmosphere {
    /// The standard atmosphere Sæmundsson's formula is stated for:
    /// 1010 hPa and 10 °C.
    pub const STANDARD: Atmosphere = Atmosphere {
        pressure_hpa: 1_010.0,
        temperature_c: 10.0,
    };

    /// The highest pressure taken, hectopascals.
    pub const MOST_PRESSURE_HPA: f64 = 1_100.0;

    /// The temperatures taken, degrees Celsius.
    pub const TEMPERATURES_C: (f64, f64) = (-90.0, 60.0);

    /// An atmosphere of a pressure and a temperature.
    ///
    /// ```
    /// use teistro_astro::sky_view::Atmosphere;
    ///
    /// assert_eq!(Atmosphere::new(1_010.0, 10.0)?, Atmosphere::STANDARD);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `pressureHpa` outside 0 to 1100, or
    /// `temperatureC` outside −90 to 60.
    pub fn new(pressure_hpa: f64, temperature_c: f64) -> Result<Atmosphere, Error> {
        if !(0.0..=Self::MOST_PRESSURE_HPA).contains(&pressure_hpa) {
            return Err(Error::invalid_arg(format!(
                "a pressure is 0 to {} hPa, not {pressure_hpa}",
                Self::MOST_PRESSURE_HPA
            ))
            .with_field("pressureHpa"));
        }
        let (coldest, warmest) = Self::TEMPERATURES_C;
        if !(coldest..=warmest).contains(&temperature_c) {
            return Err(Error::invalid_arg(format!(
                "a temperature is {coldest} to {warmest} °C, not {temperature_c}"
            ))
            .with_field("temperatureC"));
        }
        Ok(Atmosphere {
            pressure_hpa,
            temperature_c,
        })
    }

    /// What the standard refraction is multiplied by.
    ///
    /// ```
    /// use teistro_astro::sky_view::Atmosphere;
    ///
    /// // Half the air, half the bending.
    /// assert!((Atmosphere::new(505.0, 10.0)?.factor() - 0.5).abs() < 1e-15);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn factor(&self) -> f64 {
        (self.pressure_hpa / 1_010.0) * (283.0 / (273.0 + self.temperature_c))
    }
}

impl Default for Atmosphere {
    fn default() -> Atmosphere {
        Atmosphere::STANDARD
    }
}

/// The airless altitude refracted under an atmosphere, degrees
/// (Sæmundsson 1986, Meeus eq. 16.4, scaled by [`Atmosphere::factor`]),
/// or the airless altitude where the body stays below the horizon
/// (crux C973).
pub(crate) fn refracted_deg(altitude_deg: f64, atmosphere: Atmosphere) -> f64 {
    // Sæmundsson's fit turns over below about −3°, and anything that low
    // is below the horizon however it is refracted.
    const FLOOR_DEG: f64 = -2.0;
    if altitude_deg < FLOOR_DEG {
        return altitude_deg;
    }
    let lifted = altitude_deg + 10.3 / (altitude_deg + 5.11);
    let refraction_deg = 1.02 / math::tan(lifted * DEG2RAD) / 60.0 * atmosphere.factor();
    let refracted = altitude_deg + refraction_deg;
    if refracted < 0.0 {
        altitude_deg
    } else {
        refracted
    }
}

/// What a sky view draws besides the bodies, and the air it is seen
/// through: the steps of the ecliptic's and the horizon's point lists,
/// and the atmosphere.
///
/// ```
/// use teistro_astro::sky_view::{Atmosphere, SkyViewOptions};
///
/// let options = SkyViewOptions::default();
/// assert_eq!((options.ecliptic_step_deg, options.horizon_step_deg), (5.0, 5.0));
/// assert_eq!(options.atmosphere, Atmosphere::STANDARD);
/// assert!(options.check().is_ok());
/// let fine = SkyViewOptions { ecliptic_step_deg: 0.05, ..options };
/// assert_eq!(fine.check().unwrap_err().field(), Some("eclipticStepDeg"));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SkyViewOptions {
    /// The ecliptic's step in longitude, degrees, 0.1 to 90.
    pub ecliptic_step_deg: f64,
    /// The horizon's step in azimuth, degrees, 0.1 to 90.
    pub horizon_step_deg: f64,
    /// The air the refracted altitudes are reckoned through.
    pub atmosphere: Atmosphere,
    /// The grahas placed, in the order given; the nine when left as the
    /// default.
    pub grahas: Vec<Graha>,
    /// The stars placed, in the order given; the twenty-seven yogataras
    /// in the nakshatras' order by default.
    pub stars: Vec<Star>,
}

impl SkyViewOptions {
    /// The least step, degrees: 3,600 points a circle.
    pub const LEAST_STEP_DEG: f64 = 0.1;

    /// The greatest step, degrees: four points a circle.
    pub const MOST_STEP_DEG: f64 = 90.0;

    /// The nine grahas, Sun to Ketu.
    pub const NINE: [Graha; 9] = [
        Graha::Sun,
        Graha::Moon,
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
        Graha::Rahu,
        Graha::Ketu,
    ];

    /// The yogataras of the twenty-seven nakshatras, in the nakshatras'
    /// order, as the star catalogue identifies them
    /// (`03-design/sky-view.md` §3).
    ///
    /// ```
    /// use teistro_astro::sky_view::SkyViewOptions;
    /// use teistro_core::catalogue::Star;
    ///
    /// let stars = SkyViewOptions::yogataras();
    /// assert_eq!(stars.len(), 27);
    /// assert_eq!(stars[13], Star::Spica);
    /// ```
    #[must_use]
    pub fn yogataras() -> Vec<Star> {
        Nakshatra::ALL.into_iter().filter_map(yogatara).collect()
    }

    /// The options checked: each step in range.
    ///
    /// ```
    /// use teistro_astro::sky_view::SkyViewOptions;
    ///
    /// let coarse = SkyViewOptions { horizon_step_deg: 91.0, ..SkyViewOptions::default() };
    /// assert_eq!(coarse.check().unwrap_err().field(), Some("horizonStepDeg"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `eclipticStepDeg` or `horizonStepDeg` for a
    /// step outside 0.1 to 90 degrees, and the atmosphere's own refusal.
    pub fn check(&self) -> Result<(), Error> {
        for (step, field) in [
            (self.ecliptic_step_deg, "eclipticStepDeg"),
            (self.horizon_step_deg, "horizonStepDeg"),
        ] {
            if !(Self::LEAST_STEP_DEG..=Self::MOST_STEP_DEG).contains(&step) {
                return Err(Error::invalid_arg(format!(
                    "a step is {} to {} degrees, not {step}",
                    Self::LEAST_STEP_DEG,
                    Self::MOST_STEP_DEG
                ))
                .with_field(field));
            }
        }
        Atmosphere::new(self.atmosphere.pressure_hpa, self.atmosphere.temperature_c)?;
        Ok(())
    }
}

impl Default for SkyViewOptions {
    fn default() -> SkyViewOptions {
        SkyViewOptions {
            ecliptic_step_deg: 5.0,
            horizon_step_deg: 5.0,
            atmosphere: Atmosphere::STANDARD,
            grahas: Self::NINE.to_vec(),
            stars: Self::yogataras(),
        }
    }
}

/// Where a body stands in the local sky.
///
/// ```
/// use teistro_astro::sky_view::Seen;
///
/// let seen = Seen { ra_deg: 10.0, dec_deg: -5.0, altitude_deg: 30.0, refracted_altitude_deg: 30.03, azimuth_deg: 120.0 };
/// assert!(seen.refracted_altitude_deg > seen.altitude_deg);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "SkySeen"))]
#[serde(rename_all = "camelCase")]
pub struct Seen {
    /// The topocentric apparent right ascension, degrees in `[0, 360)`,
    /// on the true equator and equinox of date.
    pub ra_deg: f64,
    /// The topocentric apparent declination, degrees.
    pub dec_deg: f64,
    /// The airless altitude of the centre, degrees.
    pub altitude_deg: f64,
    /// The altitude under the atmosphere's refraction, degrees; the
    /// airless altitude where the body stays below the horizon (C973).
    pub refracted_altitude_deg: f64,
    /// The azimuth, degrees in `[0, 360)`, from north through east.
    pub azimuth_deg: f64,
}

/// A graha in the local sky.
///
/// ```
/// use teistro_astro::sky_view::{GrahaSky, Seen};
/// use teistro_core::catalogue::Graha;
///
/// let seen = Seen { ra_deg: 0.0, dec_deg: 0.0, altitude_deg: -10.0, refracted_altitude_deg: -10.0, azimuth_deg: 90.0 };
/// let mars = GrahaSky { graha: Graha::Mars, seen };
/// assert!(!mars.is_up());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GrahaSky {
    /// The graha.
    pub graha: Graha,
    /// Where it stands.
    #[serde(flatten)]
    pub seen: Seen,
}

impl GrahaSky {
    /// Whether its refracted centre is above the horizon.
    ///
    /// ```
    /// # use teistro_astro::sky_view::{GrahaSky, Seen};
    /// # use teistro_core::catalogue::Graha;
    /// let seen = Seen { ra_deg: 0.0, dec_deg: 0.0, altitude_deg: -0.3, refracted_altitude_deg: 0.2, azimuth_deg: 90.0 };
    /// assert!(GrahaSky { graha: Graha::Sun, seen }.is_up());
    /// ```
    #[must_use]
    pub fn is_up(&self) -> bool {
        self.seen.refracted_altitude_deg > 0.0
    }
}

impl core::ops::Deref for GrahaSky {
    type Target = Seen;

    fn deref(&self) -> &Seen {
        &self.seen
    }
}

/// A star in the local sky.
///
/// ```
/// use teistro_astro::sky_view::{Seen, StarSky};
/// use teistro_core::catalogue::{Nakshatra, Star};
///
/// let seen = Seen { ra_deg: 201.3, dec_deg: -11.2, altitude_deg: 40.0, refracted_altitude_deg: 40.02, azimuth_deg: 180.0 };
/// let spica = StarSky { star: Star::Spica, nakshatra: Some(Nakshatra::Chitra), magnitude: Some(0.97), seen };
/// assert_eq!(spica.altitude_deg, 40.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct StarSky {
    /// The star.
    pub star: Star,
    /// The nakshatra it is the yogatara of, if any.
    pub nakshatra: Option<Nakshatra>,
    /// Its visual magnitude, where the catalogue gives one.
    pub magnitude: Option<f64>,
    /// Where it stands.
    #[serde(flatten)]
    pub seen: Seen,
}

impl core::ops::Deref for StarSky {
    type Target = Seen;

    fn deref(&self) -> &Seen {
        &self.seen
    }
}

/// What a point of the ecliptic's list is.
///
/// ```
/// use teistro_astro::sky_view::EclipticPointKind;
///
/// assert_eq!(serde_json::to_string(&EclipticPointKind::Ascendant)?, r#""ASCENDANT""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EclipticPointKind {
    /// A longitude at the step.
    Sample,
    /// Where the ecliptic meets the eastern horizon: the ascendant.
    Ascendant,
    /// Where it meets the western horizon: the descendant.
    Descendant,
}

/// A point of the ecliptic in the local sky.
///
/// ```
/// use teistro_astro::sky_view::{EclipticPoint, EclipticPointKind};
///
/// let point = EclipticPoint { kind: EclipticPointKind::Sample, longitude_deg: 90.0, altitude_deg: 10.0, refracted_altitude_deg: 10.09, azimuth_deg: 60.0 };
/// assert_eq!(point.longitude_deg, 90.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct EclipticPoint {
    /// A step's point or a horizon crossing.
    pub kind: EclipticPointKind,
    /// Its tropical longitude on the true ecliptic of date, degrees in
    /// `[0, 360)`.
    pub longitude_deg: f64,
    /// Its airless altitude, degrees.
    pub altitude_deg: f64,
    /// Its altitude under the atmosphere's refraction (C973).
    pub refracted_altitude_deg: f64,
    /// Its azimuth, degrees in `[0, 360)`, from north through east.
    pub azimuth_deg: f64,
}

/// A point of the airless horizon, and where it lies on the sphere of
/// date.
///
/// ```
/// use teistro_astro::sky_view::HorizonPoint;
///
/// let north = HorizonPoint { azimuth_deg: 0.0, ra_deg: 0.0, dec_deg: 62.3, longitude_deg: 70.0, latitude_deg: 40.0 };
/// assert_eq!(north.azimuth_deg, 0.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct HorizonPoint {
    /// Its azimuth, degrees in `[0, 360)`, from north through east.
    pub azimuth_deg: f64,
    /// Its right ascension on the true equator of date, degrees in
    /// `[0, 360)`.
    pub ra_deg: f64,
    /// Its declination, degrees.
    pub dec_deg: f64,
    /// Its tropical longitude on the true ecliptic of date, degrees in
    /// `[0, 360)`.
    pub longitude_deg: f64,
    /// Its latitude from that ecliptic, degrees.
    pub latitude_deg: f64,
}

/// The sky at one place and instant.
///
/// ```
/// # use teistro_astro::sky_view::{SkyViewOptions, sky_view};
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::catalogue::Graha;
/// # use teistro_core::quantity::{JulianDay, Place, Ut1};
/// # use teistro_core::settings::{Node, OverridePolicy};
/// # use teistro_port_ephemeris::TestProvider;
/// # let provider = TestProvider::new();
/// # let source = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// let place = Place::try_from_degrees(-33.8688, 151.2093, 50.0)?;
/// let options = SkyViewOptions { horizon_step_deg: 90.0, ecliptic_step_deg: 90.0, ..SkyViewOptions::default() };
/// let view = sky_view(&source, place, JulianDay::<Ut1>::J2000, DeltaTModel::TableThenModel, Node::Mean, &options)?;
/// // North, east, south and west; four longitudes and the two crossings.
/// assert_eq!(view.horizon.len(), 4);
/// assert_eq!(view.ecliptic.len(), 6);
/// assert!(view.graha(Graha::Uranus).is_none());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SkyView {
    /// The instant, UT1 read as UTC.
    pub at: JulianDay<Ut1>,
    /// The local apparent sidereal time, degrees in `[0, 360)`.
    pub sidereal_time_deg: f64,
    /// The true obliquity of the ecliptic, degrees.
    pub obliquity_deg: f64,
    /// The atmosphere the refracted altitudes are reckoned under.
    pub atmosphere: Atmosphere,
    /// The ecliptic's step, degrees.
    pub ecliptic_step_deg: f64,
    /// The horizon's step, degrees.
    pub horizon_step_deg: f64,
    /// Each graha asked, in the order asked.
    pub grahas: Vec<GrahaSky>,
    /// Each star asked, in the order asked.
    pub stars: Vec<StarSky>,
    /// The ecliptic: a point at every step of longitude from 0°, and the
    /// ascendant and the descendant, in the order of longitude.
    pub ecliptic: Vec<EclipticPoint>,
    /// The horizon: a point at every step of azimuth from north.
    pub horizon: Vec<HorizonPoint>,
}

impl SkyView {
    /// A graha's place, if it was asked.
    ///
    /// ```
    /// # use teistro_astro::sky_view::{SkyViewOptions, sky_view};
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::catalogue::Graha;
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::{Node, OverridePolicy};
    /// # use teistro_port_ephemeris::TestProvider;
    /// # let provider = TestProvider::new();
    /// # let source = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(0.0, 0.0, 0.0)?;
    /// let options = SkyViewOptions { grahas: vec![Graha::Moon], ..SkyViewOptions::default() };
    /// let view = sky_view(&source, place, JulianDay::<Ut1>::J2000, DeltaTModel::TableThenModel, Node::Mean, &options)?;
    /// assert!(view.graha(Graha::Moon).is_some() && view.graha(Graha::Sun).is_none());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn graha(&self, graha: Graha) -> Option<&GrahaSky> {
        self.grahas.iter().find(|placed| placed.graha == graha)
    }

    /// Where the sky names catalogue members, and of which kind: a dotted
    /// path through the value, a list reaching each element. What a
    /// boundary section writes in full; `crates/sdk/tests/sky_view.rs`
    /// holds it to serde.
    ///
    /// ```
    /// use teistro_astro::sky_view::SkyView;
    /// use teistro_core::catalogue::Kind;
    ///
    /// assert!(SkyView::MEMBERS.contains(&("stars.star", Kind::Star)));
    /// ```
    pub const MEMBERS: [(&'static str, Kind); 3] = [
        ("grahas.graha", Kind::Graha),
        ("stars.star", Kind::Star),
        ("stars.nakshatra", Kind::Nakshatra),
    ];

    /// The sky as JSON, every catalogue member written as its full key
    /// where [`SkyView::MEMBERS`] says: what a boundary section carries
    /// and seals.
    ///
    /// ```
    /// # use teistro_astro::sky_view::{SkyViewOptions, sky_view};
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::{Node, OverridePolicy};
    /// # use teistro_port_ephemeris::TestProvider;
    /// # let provider = TestProvider::new();
    /// # let source = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(0.0, 0.0, 0.0)?;
    /// let view = sky_view(&source, place, JulianDay::<Ut1>::J2000, DeltaTModel::TableThenModel, Node::Mean, &SkyViewOptions::default())?;
    /// let written = view.in_full()?;
    /// assert_eq!(written["grahas"][1]["graha"], "graha.MOON");
    /// assert_eq!(written["stars"][13]["nakshatra"], "nakshatra.CHITRA");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INTERNAL` if the sky does not serialise, which a value this crate
    /// built cannot do.
    pub fn in_full(&self) -> Result<serde_json::Value, Error> {
        let mut value = serde_json::to_value(self)
            .map_err(|error| Error::internal(format!("a sky view did not serialise: {error}")))?;
        for (path, kind) in Self::MEMBERS {
            write_in_full(&mut value, path, kind);
        }
        Ok(value)
    }
}

/// The body a graha is read from, and whether it is the node's opposite;
/// `None` for a member a later catalogue adds without a body.
fn body_of(graha: Graha, node: Node) -> Option<(Body, bool)> {
    let node_body = if node == Node::True {
        Body::TrueNode
    } else {
        Body::MeanNode
    };
    Some(match graha {
        Graha::Sun => (Body::Sun, false),
        Graha::Moon => (Body::Moon, false),
        Graha::Mars => (Body::Mars, false),
        Graha::Mercury => (Body::Mercury, false),
        Graha::Jupiter => (Body::Jupiter, false),
        Graha::Venus => (Body::Venus, false),
        Graha::Saturn => (Body::Saturn, false),
        Graha::Rahu => (node_body, false),
        Graha::Ketu => (node_body, true),
        Graha::Uranus => (Body::Uranus, false),
        Graha::Neptune => (Body::Neptune, false),
        Graha::Pluto => (Body::Pluto, false),
        _ => return None,
    })
}

/// The frame of the instant: the station, the sidereal time and the
/// obliquity, which every direction read at it shares.
struct Frame {
    place: Place,
    atmosphere: Atmosphere,
    /// The local apparent sidereal time, degrees.
    sidereal_deg: f64,
    /// The true obliquity, radians.
    obliquity: f64,
    /// The station's geocentric position on the true equator of date,
    /// astronomical units.
    station: Vector3,
}

impl Frame {
    /// A direction of date, as a vector, seen from the place.
    fn seen(&self, direction: &Vector3) -> Seen {
        let (ra, dec) = c2s(direction);
        let ra_deg = normalise_deg(ra * RAD2DEG);
        let dec_deg = dec * RAD2DEG;
        let hour_angle_deg = difference_deg(self.sidereal_deg, ra_deg);
        let (altitude_deg, azimuth_deg) =
            horizontal(hour_angle_deg, dec_deg, self.place.latitude.get());
        Seen {
            ra_deg,
            dec_deg,
            altitude_deg,
            refracted_altitude_deg: refracted_deg(altitude_deg, self.atmosphere),
            azimuth_deg,
        }
    }

    /// The point of the true ecliptic at a longitude, degrees, as a unit
    /// vector on the true equator of date.
    fn ecliptic_at(&self, longitude_deg: f64) -> Vector3 {
        let (sin_l, cos_l) = math::sin_cos(longitude_deg * DEG2RAD);
        let (sin_e, cos_e) = math::sin_cos(self.obliquity);
        [cos_l, sin_l * cos_e, sin_l * sin_e]
    }

    /// The ecliptic longitude and latitude of a vector on the equator of
    /// date, degrees.
    fn ecliptic_of(&self, direction: &Vector3) -> (f64, f64) {
        let (sin_e, cos_e) = math::sin_cos(self.obliquity);
        let turned = [
            direction[0],
            direction[1] * cos_e + direction[2] * sin_e,
            -direction[1] * sin_e + direction[2] * cos_e,
        ];
        let (lon, lat) = c2s(&turned);
        (normalise_deg(lon * RAD2DEG), lat * RAD2DEG)
    }
}

/// Each step of a circle, degrees, from 0 and short of 360.
fn steps(step_deg: f64) -> impl Iterator<Item = f64> {
    (0_u32..)
        .map(move |count| f64::from(count) * step_deg)
        .take_while(|angle| *angle < 360.0)
}

/// The graha's direction seen from the station: the geocentric place
/// less the station, or the node's direction, which has no distance.
fn graha_seen(
    source: &dyn ApparentPositions,
    frame: &Frame,
    graha: Graha,
    node: Node,
    at: JulianDay<Ut1>,
) -> Result<GrahaSky, Error> {
    let (body, opposite) = body_of(graha, node).ok_or_else(|| {
        Error::invalid_arg(format!("{} has no body to place", graha.key())).with_field("grahas")
    })?;
    let apparent = source.apparent(body, at)?;
    let unit = s2c(apparent.ra_deg * DEG2RAD, apparent.dec_deg * DEG2RAD);
    // A node is a direction on the ecliptic, not a body: whatever distance
    // a provider gives it, the station is not taken from it, and Ketu is
    // the opposite direction.
    let is_node = matches!(body, Body::MeanNode | Body::TrueNode);
    let direction = if is_node {
        if opposite {
            unit.map(|component| -component)
        } else {
            unit
        }
    } else if apparent.distance_au > 0.0 {
        let geocentric = sxp(apparent.distance_au, &unit);
        [
            geocentric[0] - frame.station[0],
            geocentric[1] - frame.station[1],
            geocentric[2] - frame.station[2],
        ]
    } else {
        unit
    };
    Ok(GrahaSky {
        graha,
        seen: frame.seen(&direction),
    })
}

/// The ecliptic's points at the step and its two horizon crossings, in
/// the order of longitude.
fn ecliptic(frame: &Frame, step_deg: f64) -> Vec<EclipticPoint> {
    let point = |kind, longitude_deg: f64| {
        let seen = frame.seen(&frame.ecliptic_at(longitude_deg));
        EclipticPoint {
            kind,
            longitude_deg,
            altitude_deg: seen.altitude_deg,
            refracted_altitude_deg: seen.refracted_altitude_deg,
            azimuth_deg: seen.azimuth_deg,
        }
    };
    let mut points: Vec<EclipticPoint> = steps(step_deg)
        .map(|longitude| point(EclipticPointKind::Sample, longitude))
        .collect();
    // The crossings are perpendicular to the zenith and to the ecliptic's
    // pole; the ascendant is the one on the eastern side.
    let latitude = frame.place.latitude.get() * DEG2RAD;
    let (sin_t, cos_t) = math::sin_cos(frame.sidereal_deg * DEG2RAD);
    let (sin_p, cos_p) = math::sin_cos(latitude);
    let zenith = [cos_p * cos_t, cos_p * sin_t, sin_p];
    let (sin_e, cos_e) = math::sin_cos(frame.obliquity);
    let pole = [0.0, -sin_e, cos_e];
    let crossing = [
        pole[1] * zenith[2] - pole[2] * zenith[1],
        pole[2] * zenith[0] - pole[0] * zenith[2],
        pole[0] * zenith[1] - pole[1] * zenith[0],
    ];
    let east = [-sin_t, cos_t, 0.0];
    let eastward = crossing[0] * east[0] + crossing[1] * east[1];
    // The ecliptic lies in the horizon only at an instant at a latitude
    // of 90° less the obliquity; then there is no crossing to name.
    if math::hypot(math::hypot(crossing[0], crossing[1]), crossing[2]) > 1e-12 {
        let rising = if eastward >= 0.0 {
            crossing
        } else {
            crossing.map(|component| -component)
        };
        let (ascendant, _) = frame.ecliptic_of(&rising);
        points.push(point(EclipticPointKind::Ascendant, ascendant));
        points.push(point(
            EclipticPointKind::Descendant,
            normalise_deg(ascendant + 180.0),
        ));
        points.sort_by(|left, right| left.longitude_deg.total_cmp(&right.longitude_deg));
    }
    points
}

/// The airless horizon's points at the step of azimuth.
fn horizon(frame: &Frame, step_deg: f64) -> Vec<HorizonPoint> {
    let latitude = frame.place.latitude.get() * DEG2RAD;
    let (sin_p, cos_p) = math::sin_cos(latitude);
    steps(step_deg)
        .map(|azimuth_deg| {
            let (sin_a, cos_a) = math::sin_cos(azimuth_deg * DEG2RAD);
            // At altitude nought (Meeus eq. 13.5 and 13.6 turned round).
            let dec = math::asin((cos_p * cos_a).clamp(-1.0, 1.0));
            let hour_angle = math::atan2(-sin_a, -sin_p * cos_a);
            let ra_deg = normalise_deg(frame.sidereal_deg - hour_angle * RAD2DEG);
            let direction = s2c(ra_deg * DEG2RAD, dec);
            let (longitude_deg, latitude_deg) = frame.ecliptic_of(&direction);
            HorizonPoint {
                azimuth_deg,
                ra_deg,
                dec_deg: dec * RAD2DEG,
                longitude_deg,
                latitude_deg,
            }
        })
        .collect()
}

/// The sky at a place and an instant: each graha asked from `source`,
/// Rahu and Ketu from the `node` asked, each star asked from the star
/// table's apparent place, and the ecliptic and the horizon at the steps
/// of `options`.
///
/// ```
/// # use teistro_astro::sky_view::{SkyViewOptions, sky_view};
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Place, Ut1};
/// # use teistro_core::settings::{Node, OverridePolicy};
/// # use teistro_port_ephemeris::TestProvider;
/// # let provider = TestProvider::new();
/// # let source = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let place = Place::try_from_degrees(0.0, 0.0, 0.0)?;
/// let coarse = SkyViewOptions { ecliptic_step_deg: 120.0, ..SkyViewOptions::default() };
/// let refused = sky_view(&source, place, JulianDay::<Ut1>::J2000, DeltaTModel::TableThenModel, Node::Mean, &coarse);
/// assert_eq!(refused.unwrap_err().field(), Some("eclipticStepDeg"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for options [`SkyViewOptions::check`] refuses; the
/// source's refusal for a graha, a Delta T model that cannot answer, or
/// a star the star table refuses.
pub fn sky_view(
    source: &dyn ApparentPositions,
    place: Place,
    at: JulianDay<Ut1>,
    delta_t: DeltaTModel,
    node: Node,
    options: &SkyViewOptions,
) -> Result<SkyView, Error> {
    options.check()?;
    let (tt, _) = tt_of(at, delta_t)?;
    let (obliquity, mean_obliquity, longitude_nutation) = obliquity_parts(tt);
    let (uta, utb) = at.split();
    let (tta, ttb) = tt.split();
    let greenwich = iau::gst06b_with(uta, utb, tta, ttb, mean_obliquity, longitude_nutation);
    let frame = Frame {
        place,
        atmosphere: options.atmosphere,
        sidereal_deg: normalise_deg(greenwich * RAD2DEG + place.longitude.get()),
        obliquity: obliquity.true_deg * DEG2RAD,
        station: observer_at(place, greenwich).position_au,
    };
    let grahas = options
        .grahas
        .iter()
        .map(|graha| graha_seen(source, &frame, *graha, node, at))
        .collect::<Result<Vec<_>, Error>>()?;
    let stars = options
        .stars
        .iter()
        .map(|star| {
            let place = place_of(*star, tt, &Options::APPARENT)?;
            let attributes = star.attributes();
            Ok(StarSky {
                star: *star,
                nakshatra: attributes.yogatara_of,
                magnitude: attributes.magnitude,
                seen: frame.seen(&s2c(place.ra_deg * DEG2RAD, place.dec_deg * DEG2RAD)),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(SkyView {
        at,
        sidereal_time_deg: frame.sidereal_deg,
        obliquity_deg: obliquity.true_deg,
        atmosphere: options.atmosphere,
        ecliptic_step_deg: options.ecliptic_step_deg,
        horizon_step_deg: options.horizon_step_deg,
        grahas,
        stars,
        ecliptic: ecliptic(&frame, options.ecliptic_step_deg),
        horizon: horizon(&frame, options.horizon_step_deg),
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests fail by panicking, index what they asked for and compare what is exact"
    )]

    use teistro_core::settings::OverridePolicy;
    use teistro_port_ephemeris::TestProvider;

    use super::*;
    use crate::Completion;
    use crate::houses::{Input, houses};
    use teistro_core::catalogue::HouseSystem;
    use teistro_core::settings::PolarPolicy;

    const DELTA_T: DeltaTModel = DeltaTModel::TableThenModel;

    fn view_at(place: Place, at: f64, node: Node, options: &SkyViewOptions) -> SkyView {
        let provider = TestProvider::new();
        let source = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
        sky_view(
            &source,
            place,
            JulianDay::literal(at),
            DELTA_T,
            node,
            options,
        )
        .unwrap()
    }

    #[test]
    fn refraction_scales_with_the_air_and_none_is_airless() {
        let thin = Atmosphere::new(505.0, 10.0).unwrap();
        let standard = refracted_deg(10.0, Atmosphere::STANDARD) - 10.0;
        assert!((refracted_deg(10.0, thin) - 10.0 - standard / 2.0).abs() < 1e-12);
        let none = Atmosphere::new(0.0, 10.0).unwrap();
        assert_eq!(refracted_deg(0.3, none), 0.3);
        // Below the floor, and refracted still below the horizon.
        assert_eq!(refracted_deg(-5.0, Atmosphere::STANDARD), -5.0);
        assert_eq!(refracted_deg(-1.0, Atmosphere::STANDARD), -1.0);
        assert_eq!(Atmosphere::default(), Atmosphere::STANDARD);
        assert!(Atmosphere::new(f64::NAN, 10.0).is_err());
        assert_eq!(
            Atmosphere::new(1_010.0, -91.0).unwrap_err().field(),
            Some("temperatureC")
        );
    }

    #[test]
    fn a_step_out_of_range_and_bad_air_are_refused_by_field() {
        let options = SkyViewOptions::default();
        for (bad, field) in [
            (
                SkyViewOptions {
                    ecliptic_step_deg: f64::NAN,
                    ..options.clone()
                },
                "eclipticStepDeg",
            ),
            (
                SkyViewOptions {
                    horizon_step_deg: 0.0,
                    ..options.clone()
                },
                "horizonStepDeg",
            ),
            (
                SkyViewOptions {
                    atmosphere: Atmosphere {
                        pressure_hpa: 2_000.0,
                        temperature_c: 10.0,
                    },
                    ..options.clone()
                },
                "pressureHpa",
            ),
        ] {
            assert_eq!(bad.check().unwrap_err().field(), Some(field));
        }
    }

    /// The ascendant is the houses module's, from the same sidereal time
    /// and obliquity, at a northern, a southern and an equatorial place.
    #[test]
    fn the_ascendant_is_the_houses_ascendant() {
        for (latitude, longitude) in [(27.7172, 85.324), (-33.8688, 151.2093), (-0.18, -78.47)] {
            let place = Place::try_from_degrees(latitude, longitude, 0.0).unwrap();
            for hours in 0..8 {
                let at = 2_460_483.0 + f64::from(hours) * 0.13;
                let view = view_at(place, at, Node::Mean, &SkyViewOptions::default());
                let input = Input {
                    armc_deg: view.sidereal_time_deg,
                    latitude_deg: latitude,
                    obliquity_deg: view.obliquity_deg,
                    sun_declination_deg: None,
                    sidereal_offset_deg: 0.0,
                };
                let want = houses(HouseSystem::Equal, &input, PolarPolicy::Error)
                    .unwrap()
                    .angles
                    .ascendant_deg;
                let rising = view
                    .ecliptic
                    .iter()
                    .find(|point| point.kind == EclipticPointKind::Ascendant)
                    .unwrap();
                assert!(
                    difference_deg(rising.longitude_deg, want).abs() < 1e-9,
                    "{} against {want}",
                    rising.longitude_deg
                );
                assert!(rising.altitude_deg.abs() < 1e-9);
                let setting = view
                    .ecliptic
                    .iter()
                    .find(|point| point.kind == EclipticPointKind::Descendant)
                    .unwrap();
                assert!(
                    difference_deg(setting.azimuth_deg, rising.azimuth_deg + 180.0).abs() < 1e-6
                );
            }
        }
    }

    #[test]
    fn the_lists_are_ordered_and_the_horizon_sits_at_nought() {
        let place = Place::try_from_degrees(69.65, 18.96, 0.0).unwrap();
        let options = SkyViewOptions {
            ecliptic_step_deg: 7.5,
            horizon_step_deg: 10.0,
            atmosphere: Atmosphere::new(990.0, -5.0).unwrap(),
            grahas: vec![Graha::Ketu, Graha::Pluto, Graha::Sun],
            stars: vec![Star::Vega],
        };
        let view = view_at(place, 2_460_665.3, Node::Mean, &options);
        assert_eq!(view.ecliptic.len(), 48 + 2);
        assert!(
            view.ecliptic
                .windows(2)
                .all(|pair| pair[0].longitude_deg <= pair[1].longitude_deg)
        );
        assert_eq!(view.horizon.len(), 36);
        let sun = view.graha(Graha::Sun).unwrap();
        for point in &view.horizon {
            // The horizon point read back through the frame is at nought.
            let frame_altitude = horizontal(
                difference_deg(view.sidereal_time_deg, point.ra_deg),
                point.dec_deg,
                69.65,
            );
            assert!(frame_altitude.0.abs() < 1e-9, "{frame_altitude:?}");
            assert!(difference_deg(frame_altitude.1, point.azimuth_deg).abs() < 1e-9);
        }
        assert_eq!(
            view.grahas.iter().map(|g| g.graha).collect::<Vec<_>>(),
            options.grahas
        );
        assert_eq!(view.stars[0].nakshatra, None);
        assert_eq!(view.atmosphere, options.atmosphere);
        assert_eq!(sun.is_up(), sun.refracted_altitude_deg > 0.0);
        // The settings' node picks the body; Ketu is its opposite.
        assert_eq!(
            body_of(Graha::Ketu, Node::True),
            Some((Body::TrueNode, true))
        );
        assert_eq!(
            body_of(Graha::Rahu, Node::Mean),
            Some((Body::MeanNode, false))
        );
    }

    #[test]
    fn a_star_on_the_meridian_stands_at_the_colatitude_less_its_declination() {
        let place = Place::try_from_degrees(27.7172, 85.324, 1_400.0).unwrap();
        let options = SkyViewOptions {
            stars: vec![Star::Spica],
            ..SkyViewOptions::default()
        };
        let view = view_at(place, 2_460_483.25, Node::Mean, &options);
        let spica = view.stars[0];
        let hour_angle = difference_deg(view.sidereal_time_deg, spica.ra_deg);
        let (altitude, _) = horizontal(0.0, spica.dec_deg, 27.7172);
        // The altitude at the meridian bounds every other.
        assert!(spica.altitude_deg <= altitude + 1e-9, "{hour_angle}");
        assert!((altitude - (90.0 - 27.7172 + spica.dec_deg)).abs() < 1e-9);
        assert_eq!(spica.nakshatra, Some(Nakshatra::Chitra));
        assert_eq!(*spica, spica.seen);
    }
}
