//! Relocation and astrocartography (`03-design/astrocartography.md`): a
//! chart's angles and houses recast at other places for the same instant,
//! and the lines on the Earth where each body stands on an angle.

use serde::{Deserialize, Serialize};
use teistro_chart::foundation::{ChartFoundation, angles_of, cusps_of};
use teistro_core::catalogue::{ChartKind, Graha, HouseSystem};
use teistro_core::envelope::{Envelope, content_hash};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;
use teistro_houses::system::override_of;
use teistro_western::{
    AstroLine, LEO_HOUSE_SYSTEM, LINE_TOLERANCE_DEG, MODULE, MapSky, MappedBody, Projection,
    Sampling, lines,
};

use crate::area::ChartArea;
use crate::asked::{instant, place_of};

/// The relocation record's name where a binding sends it.
const RELOCATED: &str = "relocated";

/// The astrocartography record's name where a binding sends it.
const ASTROCARTOGRAPHY: &str = "astrocartography";

/// The most places one relocation recasts.
pub const RELOCATION_PLACES_MAX: usize = 256;

/// The bodies a map draws when the request names none: the seven, then
/// whichever outer planets the chart places.
const SEVEN: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mercury,
    Graha::Venus,
    Graha::Mars,
    Graha::Jupiter,
    Graha::Saturn,
];

/// A place a chart is recast at.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelocationPlace {
    /// Latitude, degrees, north positive.
    pub latitude_deg: f64,
    /// Longitude, degrees, east positive.
    pub longitude_deg: f64,
    /// Altitude, metres.
    #[serde(default)]
    pub altitude_m: f64,
}

/// A chart's instant recast at other places.
///
/// ```
/// use teistro::RelocationRequest;
///
/// let asked = RelocationRequest::from_json(r#"{"instant": 2447000.25, "places": [{"latitudeDeg": 51.5, "longitudeDeg": -0.12}]}"#)?;
/// assert_eq!(asked.places.len(), 1);
/// let none = RelocationRequest::from_json(r#"{"instant": 2447000.25, "places": []}"#).unwrap_err();
/// assert_eq!(none.field(), Some("relocated.places"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelocationRequest {
    /// The chart's instant, a Julian day (UTC).
    pub instant: f64,
    /// The places, one to 256.
    pub places: Vec<RelocationPlace>,
    /// The house division; the profile's Western override, else Placidus,
    /// when left out (C1157).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub house_system: Option<HouseSystem>,
}

/// One place's angles and houses.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RelocatedChart {
    /// Latitude, degrees.
    pub latitude_deg: f64,
    /// Longitude, degrees.
    pub longitude_deg: f64,
    /// Altitude, metres.
    pub altitude_m: f64,
    /// The ascendant, degrees, in the chart's zodiac.
    pub ascendant_deg: f64,
    /// The midheaven, degrees, in the chart's zodiac.
    pub midheaven_deg: f64,
    /// The twelve cusps, degrees, in the chart's zodiac.
    pub cusps_deg: [f64; 12],
    /// The division the cusps are of: the one asked, or the one its
    /// polar policy fell back to.
    pub house_system: HouseSystem,
}

/// A relocation's answer: each place's angles and houses, in the order
/// asked.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Relocation {
    /// The instant, a Julian day (UTC).
    pub instant: JulianDay<Utc>,
    /// The true obliquity of the ecliptic, degrees.
    pub obliquity_deg: f64,
    /// Each place.
    pub places: Vec<RelocatedChart>,
}

impl RelocationRequest {
    /// The request a binding writes as `relocated`, read and checked.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, an instant out of range, no place or more than 256, or a
    /// place out of range, each named under `relocated`.
    pub fn from_json(text: &str) -> Result<RelocationRequest, Error> {
        let asked: RelocationRequest = teistro_core::strict::read(text, RELOCATED)?;
        asked.check().map_err(|why| why.under(RELOCATED))?;
        Ok(asked)
    }

    /// Whether the request can be answered, before any sky is read.
    ///
    /// # Errors
    ///
    /// As [`RelocationRequest::from_json`], unnamed by the record.
    pub fn check(&self) -> Result<(), Error> {
        instant(self.instant, "instant")?;
        if self.places.is_empty() || self.places.len() > RELOCATION_PLACES_MAX {
            return Err(Error::invalid_arg(format!(
                "a relocation recasts one to {RELOCATION_PLACES_MAX} places, not {}",
                self.places.len()
            ))
            .with_field("places"));
        }
        for (at, place) in self.places.iter().enumerate() {
            place_of(place.latitude_deg, place.longitude_deg, place.altitude_m)
                .map_err(|why| why.under(&format!("places[{at}]")))?;
        }
        Ok(())
    }
}

/// The lines of a map at one instant.
///
/// ```
/// use teistro::AstrocartographyRequest;
/// use teistro::western::Projection;
///
/// let asked = AstrocartographyRequest::from_json(r#"{"instant": 2447000.25}"#)?;
/// assert_eq!((asked.step_deg, asked.latitude_limit_deg, asked.projection), (1.0, 80.0, Projection::InMundo));
/// let coarse = AstrocartographyRequest::from_json(r#"{"instant": 2447000.25, "stepDeg": 20}"#).unwrap_err();
/// assert_eq!(coarse.field(), Some("astrocartography.stepDeg"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AstrocartographyRequest {
    /// The instant, a Julian day (UTC).
    pub instant: f64,
    /// The bodies, in the order their lines are answered; none for the
    /// seven and whichever outer planets the profile places.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bodies: Vec<Graha>,
    /// In mundo, the default, or zodiacal (C1155).
    #[serde(default)]
    pub projection: Projection,
    /// The step of latitude between a line's points, degrees, above zero
    /// and at most ten; one by default.
    #[serde(default = "default_step")]
    pub step_deg: f64,
    /// The furthest latitude sampled north and south, degrees, above zero
    /// and below ninety; eighty by default.
    #[serde(default = "default_limit")]
    pub latitude_limit_deg: f64,
}

fn default_step() -> f64 {
    1.0
}

fn default_limit() -> f64 {
    80.0
}

/// A map's lines and what they were drawn under.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Astrocartography {
    /// The instant, a Julian day (UTC).
    pub instant: JulianDay<Utc>,
    /// The projection.
    pub projection: Projection,
    /// The step of latitude, degrees.
    pub step_deg: f64,
    /// The furthest latitude sampled, degrees.
    pub latitude_limit_deg: f64,
    /// How far a point may stand from its angle, read back through a
    /// chart cast there, degrees.
    pub tolerance_deg: f64,
    /// The right ascension of the Greenwich meridian, degrees.
    pub greenwich_ramc_deg: f64,
    /// The true obliquity of the ecliptic, degrees.
    pub obliquity_deg: f64,
    /// Each body's MC, IC, ASC and DSC lines.
    pub lines: Vec<AstroLine>,
}

impl AstrocartographyRequest {
    /// The lines at `instant` under the defaults.
    #[must_use]
    pub fn new(instant: f64) -> AstrocartographyRequest {
        AstrocartographyRequest {
            instant,
            bodies: Vec::new(),
            projection: Projection::InMundo,
            step_deg: default_step(),
            latitude_limit_deg: default_limit(),
        }
    }

    /// The request a binding writes as `astrocartography`, read and
    /// checked.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, an instant out of range, or a step or a limit out of range,
    /// each named under `astrocartography`.
    pub fn from_json(text: &str) -> Result<AstrocartographyRequest, Error> {
        let asked: AstrocartographyRequest = teistro_core::strict::read(text, ASTROCARTOGRAPHY)?;
        asked.check().map_err(|why| why.under(ASTROCARTOGRAPHY))?;
        Ok(asked)
    }

    /// Whether the request can be answered, before any sky is read.
    ///
    /// # Errors
    ///
    /// As [`AstrocartographyRequest::from_json`], unnamed by the record.
    pub fn check(&self) -> Result<(), Error> {
        instant(self.instant, "instant")?;
        // The sampling's own checks, on no body.
        lines(
            MapSky {
                greenwich_ramc_deg: 0.0,
                obliquity_deg: 23.4,
            },
            &[],
            self.sampling(),
        )
        .map(|_| ())
    }

    fn sampling(&self) -> Sampling {
        Sampling {
            projection: self.projection,
            step_deg: self.step_deg,
            latitude_limit_deg: self.latitude_limit_deg,
        }
    }
}

impl ChartArea<'_> {
    /// The chart every relocation and map is read from: founded for `at`
    /// at 0° N 0° E, the kind `RELOCATED`. A place's angles and cusps read
    /// only the instant, the zodiac and the place, so they are read from
    /// this chart with its place moved, and a polar place whose day has no
    /// sunrise still has its angles.
    fn relocated_origin(self, at: JulianDay<Utc>) -> Result<Envelope<ChartFoundation>, Error> {
        let origin = place_of(0.0, 0.0, 0.0)?;
        self.found(at, &origin, UtcOffset::UTC, ChartKind::Relocated)
    }

    /// A chart's instant **recast at other places**
    /// (`03-design/astrocartography.md`): each place's ascendant,
    /// midheaven and twelve cusps, in the context's zodiac, for the same
    /// instant, so the grahas stand where they stood and only the angles
    /// and the houses move, a polar place's too. Sealed by the chart founded
    /// at the instant, the request its input.
    ///
    /// ```
    /// # #[cfg(feature = "builtin-ephemeris")] {
    /// use teistro::{Context, Ephemeris, RelocationRequest};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let asked = RelocationRequest::from_json(r#"{"instant": 2447000.25, "places": [{"latitudeDeg": 27.7172, "longitudeDeg": 85.324}, {"latitudeDeg": 27.7172, "longitudeDeg": 100.324}]}"#)?;
    /// let moved = sdk.chart().relocated(&asked)?.value;
    /// // Fifteen degrees east, the meridian has turned about fifteen degrees.
    /// let turned = (moved.places[1].midheaven_deg - moved.places[0].midheaven_deg).rem_euclid(360.0);
    /// assert!((turned - 15.0).abs() < 3.0, "{turned}");
    /// # }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// What [`RelocationRequest::check`] refuses; a context with no
    /// ephemeris; a chart whose angles are its provider's (a classical
    /// astronomy's); a place the polar policy refuses.
    pub fn relocated(self, asked: &RelocationRequest) -> Result<Envelope<Relocation>, Error> {
        asked.check()?;
        let at = instant(asked.instant, "instant")?;
        let settings = self.context().settings();
        let delta_t = self.context().delta_t();
        let policy = settings.houses.polar_policy;
        let system = asked
            .house_system
            .or_else(|| override_of(settings, MODULE))
            .unwrap_or(LEO_HOUSE_SYSTEM);
        let Envelope {
            value: mut moved,
            mut provenance,
        } = self.relocated_origin(at)?;
        let mut places = Vec::with_capacity(asked.places.len());
        let mut obliquity_deg = 0.0;
        for (index, asked_place) in asked.places.iter().enumerate() {
            moved.place = place_of(
                asked_place.latitude_deg,
                asked_place.longitude_deg,
                asked_place.altitude_m,
            )?;
            let named = |why: Error| why.under(&format!("places[{index}]"));
            let angles = angles_of(&moved, delta_t, policy).map_err(named)?;
            let (cusps_deg, house_system) =
                cusps_of(&moved, system, delta_t, policy).map_err(named)?;
            obliquity_deg = angles.obliquity_deg;
            places.push(RelocatedChart {
                latitude_deg: asked_place.latitude_deg,
                longitude_deg: asked_place.longitude_deg,
                altitude_m: asked_place.altitude_m,
                ascendant_deg: angles.ascendant_deg,
                midheaven_deg: angles.midheaven_deg,
                cusps_deg,
                house_system,
            });
        }
        provenance.input_hash = content_hash(asked);
        Ok(Envelope::sealing(
            Relocation {
                instant: at,
                obliquity_deg,
                places,
            },
            provenance,
        ))
    }

    /// The **astrocartography lines** at an instant
    /// (`03-design/astrocartography.md`): each body's MC, IC, ASC and DSC
    /// lines as polylines of `[longitude, latitude]` sampled at the
    /// request's step, a rising or setting line stopping at its polar
    /// limit and saying so (C1156). The bodies and the sidereal time are
    /// read from a chart founded at the instant at 0° N 0° E, so a point's
    /// angle read back through a chart cast there agrees to
    /// [`teistro_western::LINE_TOLERANCE_DEG`]. Geometric: no refraction,
    /// the geocentric sky.
    ///
    /// ```
    /// # #[cfg(feature = "builtin-ephemeris")] {
    /// use teistro::{AstrocartographyRequest, Context, Ephemeris};
    /// use teistro::catalogue::Graha;
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let mut asked = AstrocartographyRequest::new(2447000.25);
    /// asked.bodies = vec![Graha::Sun];
    /// let map = sdk.chart().astrocartography(&asked)?.value;
    /// assert_eq!(map.lines.len(), 4);
    /// // The Sun's MC line is a meridian.
    /// let mc = &map.lines[0].points;
    /// assert!(mc.iter().all(|point| point[0] == mc[0][0]));
    /// # }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// What [`AstrocartographyRequest::check`] refuses; a context with no
    /// ephemeris; a body the chart does not place (`INVALID_ARG` naming
    /// `bodies`); a chart whose angles are its provider's.
    pub fn astrocartography(
        self,
        asked: &AstrocartographyRequest,
    ) -> Result<Envelope<Astrocartography>, Error> {
        asked.check()?;
        let at = instant(asked.instant, "instant")?;
        let Envelope {
            value: chart,
            mut provenance,
        } = self.relocated_origin(at)?;
        let settings = self.context().settings();
        let angles = angles_of(
            &chart,
            self.context().delta_t(),
            settings.houses.polar_policy,
        )?;
        // The midheaven's right ascension is the sidereal time there.
        let meridian = teistro_astro::sky::ecliptic_to_equatorial(
            teistro_astro::sky::Spherical {
                lon_deg: chart.zodiac.to_tropical(angles.midheaven_deg),
                lat_deg: 0.0,
            },
            angles.obliquity_deg,
        );
        let asked_bodies: Vec<Graha> = if asked.bodies.is_empty() {
            SEVEN
                .into_iter()
                .chain(chart.outer.iter().map(|placed| placed.graha))
                .collect()
        } else {
            asked.bodies.clone()
        };
        let mut bodies = Vec::with_capacity(asked_bodies.len());
        for graha in asked_bodies {
            let placed = chart.graha(graha).ok_or_else(|| {
                Error::invalid_arg(format!(
                    "this context's chart does not place {}",
                    graha.key()
                ))
                .with_field("bodies")
            })?;
            bodies.push(MappedBody {
                graha,
                longitude_deg: placed.tropical_deg,
                latitude_deg: placed.latitude_deg,
            });
        }
        let sky = MapSky {
            greenwich_ramc_deg: meridian.lon_deg,
            obliquity_deg: angles.obliquity_deg,
        };
        let drawn = lines(sky, &bodies, asked.sampling())?;
        provenance.input_hash = content_hash(asked);
        Ok(Envelope::sealing(
            Astrocartography {
                instant: at,
                projection: asked.projection,
                step_deg: asked.step_deg,
                latitude_limit_deg: asked.latitude_limit_deg,
                tolerance_deg: LINE_TOLERANCE_DEG,
                greenwich_ramc_deg: sky.greenwich_ramc_deg,
                obliquity_deg: sky.obliquity_deg,
                lines: drawn,
            },
            provenance,
        ))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::float_cmp,
        reason = "a test fails by panicking, and a start is an exact day"
    )]

    use super::*;

    #[test]
    fn a_relocation_refuses_too_many_places_and_a_bad_one_by_its_index() {
        let place = r#"{"latitudeDeg": 0, "longitudeDeg": 0}"#;
        let many = vec![place; RELOCATION_PLACES_MAX + 1].join(",");
        let refused = RelocationRequest::from_json(&format!(
            r#"{{"instant": 2447000.25, "places": [{many}]}}"#
        ))
        .unwrap_err();
        assert_eq!(refused.field(), Some("relocated.places"));
        let bad = RelocationRequest::from_json(
            r#"{"instant": 2447000.25, "places": [{"latitudeDeg": 0, "longitudeDeg": 0}, {"latitudeDeg": 91, "longitudeDeg": 0}]}"#,
        )
        .unwrap_err();
        assert_eq!(bad.field(), Some("relocated.places[1].latitudeDeg"));
    }

    #[test]
    fn a_map_refuses_its_limit_and_an_unplaced_body_by_name() {
        for (text, field) in [
            (
                r#"{"instant": 2447000.25, "latitudeLimitDeg": 90}"#,
                "astrocartography.latitudeLimitDeg",
            ),
            (
                r#"{"instant": 2447000.25, "bodies": ["ABHIJIT"]}"#,
                "astrocartography.bodies",
            ),
        ] {
            let refused = AstrocartographyRequest::from_json(text).unwrap_err();
            assert!(
                refused
                    .field()
                    .is_some_and(|named| named.starts_with(field)),
                "{text}: {refused:?}"
            );
        }
        assert_eq!(
            AstrocartographyRequest::new(2_447_000.25)
                .sampling()
                .step_deg,
            1.0
        );
    }
}
