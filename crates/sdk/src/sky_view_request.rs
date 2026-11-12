//! The sky view as a binding asks it (`03-design/sky-view.md`): a place,
//! an instant, and what to draw; and the sky pose (`03-design/sky-pose.md`),
//! the same with a device's pose beside it.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_astro::sky_pose::{Pose, SkyPose, sky_pose};
use teistro_astro::sky_view::{Atmosphere, SkyView, SkyViewOptions, sky_view};
use teistro_core::catalogue::{Graha, Star};
use teistro_core::envelope::{Convention, Envelope, Provenance, content_hash};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_port_ephemeris::Astronomy;

use crate::AlmanacArea;
use crate::asked::place_of;
use crate::ephemeris::no_ephemeris;

/// The record's name where a binding sends it, which a refusal is named
/// under.
const SKY_VIEW: &str = "skyView";

/// The sky pose's name where a binding sends it.
const SKY_POSE: &str = "skyPose";

/// The pose's key beside the sky view's fields.
const POSE: &str = "pose";

/// A place, an instant and what to draw there: the sky view a binding
/// asks for.
#[derive(Clone, Debug, PartialEq)]
pub struct SkyViewRequest {
    /// The place the sky is seen from.
    pub place: Place,
    /// The instant, UTC, read as UT1.
    pub at: JulianDay<Utc>,
    /// The grahas, the stars, the steps and the air.
    pub options: SkyViewOptions,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "SkyViewRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SkyViewAsked {
    /// The place's latitude, degrees north positive.
    latitude_deg: f64,
    /// The place's longitude, degrees east positive.
    longitude_deg: f64,
    /// The place's height above sea level in metres, 0 when left out.
    #[serde(default)]
    altitude_m: f64,
    /// The instant, a UTC Julian day.
    at: JulianDay<Utc>,
    /// The grahas placed, in this order; the nine, Sun to Ketu, when left
    /// out.
    #[serde(default)]
    grahas: Option<Vec<Graha>>,
    /// The stars placed, in this order; the twenty-seven nakshatras'
    /// yogataras when left out.
    #[serde(default)]
    stars: Option<Vec<Star>>,
    /// The ecliptic's step in longitude, degrees, 0.1 to 90; 5 when left
    /// out.
    #[serde(default)]
    ecliptic_step_deg: Option<f64>,
    /// The horizon's step in azimuth, degrees, 0.1 to 90; 5 when left out.
    #[serde(default)]
    horizon_step_deg: Option<f64>,
    /// The pressure the refraction is reckoned under, hectopascals, 0 to
    /// 1100; 1010 when left out, and 0 for no refraction.
    #[serde(default)]
    pressure_hpa: Option<f64>,
    /// The temperature, degrees Celsius, −90 to 60; 10 when left out.
    #[serde(default)]
    temperature_c: Option<f64>,
}

impl SkyViewRequest {
    /// The record a binding sends, as JSON.
    ///
    /// ```
    /// use teistro::SkyViewRequest;
    /// use teistro::catalogue::Graha;
    ///
    /// let asked = SkyViewRequest::from_json(
    ///     r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400, "at": 2461345.25,
    ///         "grahas": ["MOON", "graha.SUN"], "eclipticStepDeg": 1}"#,
    /// )?;
    /// assert_eq!(asked.options.grahas, [Graha::Moon, Graha::Sun]);
    /// assert_eq!(asked.options.stars.len(), 27);
    /// let coarse = SkyViewRequest::from_json(r#"{"latitudeDeg": 0, "longitudeDeg": 0, "at": 2461345.25, "horizonStepDeg": 100}"#);
    /// assert_eq!(coarse.unwrap_err().field(), Some("skyView.horizonStepDeg"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a place out of range, a step outside 0.1 to 90 degrees, or an
    /// atmosphere no planet has, named under `skyView`.
    pub fn from_json(text: &str) -> Result<SkyViewRequest, Error> {
        SkyViewRequest::read(&teistro_core::strict::parse(text, SKY_VIEW)?, SKY_VIEW)
    }

    /// The request read from JSON already parsed, each refusal named
    /// under `root`: the sky view's own, or the sky pose's.
    fn read(given: &Value, root: &str) -> Result<SkyViewRequest, Error> {
        let asked: SkyViewAsked = teistro_core::strict::read_value(given, root)?;
        let defaults = SkyViewOptions::default();
        let options = SkyViewOptions {
            ecliptic_step_deg: asked
                .ecliptic_step_deg
                .unwrap_or(defaults.ecliptic_step_deg),
            horizon_step_deg: asked.horizon_step_deg.unwrap_or(defaults.horizon_step_deg),
            atmosphere: Atmosphere {
                pressure_hpa: asked
                    .pressure_hpa
                    .unwrap_or(Atmosphere::STANDARD.pressure_hpa),
                temperature_c: asked
                    .temperature_c
                    .unwrap_or(Atmosphere::STANDARD.temperature_c),
            },
            grahas: asked.grahas.unwrap_or(defaults.grahas),
            stars: asked.stars.unwrap_or(defaults.stars),
        };
        let place = place_of(asked.latitude_deg, asked.longitude_deg, asked.altitude_m)
            .and_then(|place| options.check().map(|()| place))
            .map_err(|why| why.under(root))?;
        Ok(SkyViewRequest {
            place,
            at: asked.at,
            options,
        })
    }
}

/// A sky view and a device's pose: what `almanac.skyPose` reads
/// (`03-design/sky-pose.md`).
#[derive(Clone, Debug, PartialEq)]
pub struct SkyPoseRequest {
    /// The sky view, as `almanac.skyView` reads it.
    pub view: SkyViewRequest,
    /// Where the camera points and what it frames, checked.
    pub pose: Pose,
}

impl SkyPoseRequest {
    /// The record a binding sends, as JSON: the sky view's fields, and
    /// `pose` beside them.
    ///
    /// ```
    /// use teistro::SkyPoseRequest;
    ///
    /// let asked = SkyPoseRequest::from_json(
    ///     r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "at": 2461345.25,
    ///         "pose": {"azimuthDeg": -90, "altitudeDeg": 30, "rollDeg": 0, "horizontalFovDeg": 60, "aspectRatio": 0.75}}"#,
    /// )?;
    /// assert_eq!(asked.pose.azimuth_deg, 270.0);
    /// assert_eq!(asked.view.options.stars.len(), 27);
    /// let wide = SkyPoseRequest::from_json(
    ///     r#"{"latitudeDeg": 0, "longitudeDeg": 0, "at": 2461345.25,
    ///         "pose": {"azimuthDeg": 0, "altitudeDeg": 0, "rollDeg": 0, "horizontalFovDeg": 180, "aspectRatio": 1}}"#,
    /// );
    /// assert_eq!(wide.unwrap_err().field(), Some("skyPose.pose.horizontalFovDeg"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a missing `pose`, a
    /// key either part does not read, and either part's own refusals,
    /// named under `skyPose` and `skyPose.pose`.
    pub fn from_json(text: &str) -> Result<SkyPoseRequest, Error> {
        let mut given = teistro_core::strict::parse(text, SKY_POSE)?;
        let pose = match given.as_object_mut().map(|fields| fields.remove(POSE)) {
            Some(Some(pose)) => pose,
            Some(None) => {
                return Err(Error::invalid_arg("`pose` is missing")
                    .with_field(format!("{SKY_POSE}.{POSE}"))
                    .with_hint("the record needs the device's pose"));
            }
            // Not an object: the sky view's reader names it.
            None => Value::Null,
        };
        let view = SkyViewRequest::read(&given, SKY_POSE)?;
        let under = format!("{SKY_POSE}.{POSE}");
        let pose = teistro_core::strict::read_value::<Pose>(&pose, &under)?
            .checked()
            .map_err(|why| why.under(&under))?;
        Ok(SkyPoseRequest { view, pose })
    }
}

/// The sky pose's request schema: the sky view's, with the pose beside
/// it and required.
#[cfg(feature = "schema")]
pub(crate) fn pose_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    let pose = generator.subschema_for::<Pose>();
    let mut schema = crate::records::beside::<SkyViewAsked>(generator, vec![(POSE, pose)]);
    if let Some(required) = schema
        .get_mut("required")
        .and_then(serde_json::Value::as_array_mut)
    {
        required.push(Value::from(POSE));
    }
    schema
}

/// What a sky view is a function of, beside the settings.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
struct SkyViewInput<'a> {
    place: Place,
    at: f64,
    grahas: &'a [Graha],
    stars: &'a [Star],
    ecliptic_step_deg: f64,
    horizon_step_deg: f64,
    atmosphere: Atmosphere,
    /// The device's pose, for a sky pose; left out of a sky view's input,
    /// whose hash stays what it was.
    #[serde(skip_serializing_if = "Option::is_none")]
    pose: Option<Pose>,
}

impl AlmanacArea<'_> {
    /// The sky at a place and an instant (`03-design/sky-view.md`): where
    /// each graha and each junction star stands, topocentric, apparent and
    /// of date, its altitude airless and refracted and its azimuth from
    /// north through east; and the ecliptic, with the ascendant and the
    /// descendant, and the horizon as point lists at the steps asked.
    /// Rahu and Ketu are the node the settings name.
    ///
    /// ```no_run
    /// # #[cfg(feature = "builtin-ephemeris")] {
    /// use teistro::{Context, Ephemeris, SkyViewRequest};
    /// use teistro::catalogue::Graha;
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let asked = SkyViewRequest::from_json(r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400, "at": 2461345.25}"#)?;
    /// let view = sdk.almanac().sky_view(&asked)?;
    /// let moon = view.value.graha(Graha::Moon).expect("asked by default");
    /// println!("the Moon {:.1}° up at {:.1}°", moon.refracted_altitude_deg, moon.azimuth_deg);
    /// # }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `CAPABILITY` for a context with no ephemeris; `UNSUPPORTED` for a
    /// classical one, whose places are a text's and not where to look
    /// (C1185); the ephemeris's refusal for the instant.
    pub fn sky_view(self, asked: &SkyViewRequest) -> Result<Envelope<SkyView>, Error> {
        let (view, provenance) = self.sky_stamped(asked, None)?;
        Ok(Envelope::sealing(view, provenance))
    }

    /// The sky a request asks for, and its provenance: stamped with the
    /// pose too when one is asked, and naming the projection then.
    fn sky_stamped(
        self,
        asked: &SkyViewRequest,
        pose: Option<Pose>,
    ) -> Result<(SkyView, Provenance), Error> {
        let context = self.context();
        let provider = context.ephemeris().ok_or_else(no_ephemeris)?;
        if provider.capabilities().astronomy == Astronomy::Classical {
            return Err(Error::new(
                Status::Unsupported,
                "a classical text's places are its own reckoning, not where a graha is seen (C1185)",
            )
            .with_hint("ask the sky view of a context over a modern ephemeris"));
        }
        let settings = context.settings();
        let delta_t = context.delta_t();
        let source = teistro_astro::Completion::new(provider, settings.provider.overrides, delta_t);
        let node = settings.frame.node;
        let view = sky_view(
            &source,
            asked.place,
            asked.at.relabel(),
            delta_t,
            node,
            &asked.options,
        )?;
        let options = &asked.options;
        let mut provenance = context.stamped(
            content_hash(&SkyViewInput {
                place: asked.place,
                at: asked.at.get(),
                grahas: &options.grahas,
                stars: &options.stars,
                ecliptic_step_deg: options.ecliptic_step_deg,
                horizon_step_deg: options.horizon_step_deg,
                atmosphere: options.atmosphere,
                pose,
            }),
            source.apparent_frame(),
            Vec::new(),
        );
        provenance.applied_conventions.push(Convention {
            knob: String::from("sky.refraction"),
            value: format!(
                "Saemundsson at {} hPa and {} C",
                options.atmosphere.pressure_hpa, options.atmosphere.temperature_c
            ),
            reason: String::from(
                "a refracted altitude is the airless one lifted by Saemundsson's standard refraction scaled to the air, and left airless below the horizon (C973); UT1 read as UTC",
            ),
        });
        if pose.is_some() {
            provenance.applied_conventions.push(Convention {
                knob: String::from("sky.pose"),
                value: String::from("rectilinear (gnomonic), true north, refracted altitudes"),
                reason: String::from(
                    "a camera's picture is the gnomonic projection about its line of sight; the azimuth asked is from true north, the declination the consumer's; a point is drawn at its refracted altitude and a horizon point at nought (03-design/sky-pose.md)",
                ),
            });
        }
        Ok((view, provenance))
    }

    /// A device's camera pointed at the sky (`03-design/sky-pose.md`):
    /// the sky view the request asks for, unchanged, and where each of its
    /// grahas, stars, ecliptic points and horizon points falls in the
    /// camera's picture, normalised with the origin at the top left. The
    /// rising lagna point is the ecliptic's `ASCENDANT`
    /// ([`SkyPose::ascendant`]).
    ///
    /// ```no_run
    /// # #[cfg(feature = "builtin-ephemeris")] {
    /// use teistro::{Context, Ephemeris, SkyPoseRequest};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let asked = SkyPoseRequest::from_json(
    ///     r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "at": 2461345.25,
    ///         "pose": {"azimuthDeg": 90, "altitudeDeg": 20, "rollDeg": 0, "horizontalFovDeg": 60, "aspectRatio": 0.75}}"#,
    /// )?;
    /// let seen = sdk.almanac().sky_pose(&asked)?.value;
    /// for (star, placed) in seen.sky.stars.iter().zip(&seen.screen.stars) {
    ///     if placed.in_view {
    ///         println!("{:?} at {:?}, {:?}", star.star, placed.x, placed.y);
    ///     }
    /// }
    /// # }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::sky_view`].
    pub fn sky_pose(self, asked: &SkyPoseRequest) -> Result<Envelope<SkyPose>, Error> {
        let (view, provenance) = self.sky_stamped(&asked.view, Some(asked.pose))?;
        Ok(Envelope::sealing(sky_pose(view, asked.pose), provenance))
    }

    /// As [`AlmanacArea::sky_pose`] for a query a binding sent, every
    /// catalogue member of its sky written as its full key.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::sky_view`].
    pub fn sky_pose_query(
        self,
        asked: &SkyPoseRequest,
    ) -> Result<Envelope<serde_json::Value>, Error> {
        let Envelope { value, provenance } = self.sky_pose(asked)?;
        Ok(Envelope::sealing(value.in_full()?, provenance))
    }

    /// As [`AlmanacArea::sky_view`] for a query a binding sent, every
    /// catalogue member written as its full key.
    ///
    /// ```no_run
    /// # #[cfg(feature = "builtin-ephemeris")] {
    /// use teistro::{Context, Ephemeris, SkyViewRequest};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let asked = SkyViewRequest::from_json(r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "at": 2461345.25}"#)?;
    /// let written = sdk.almanac().sky_view_query(&asked)?;
    /// assert_eq!(written.value["grahas"][0]["graha"], "graha.SUN");
    /// # }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::sky_view`].
    pub fn sky_view_query(
        self,
        asked: &SkyViewRequest,
    ) -> Result<Envelope<serde_json::Value>, Error> {
        let Envelope { value, provenance } = self.sky_view(asked)?;
        Ok(Envelope::sealing(value.in_full()?, provenance))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

    use super::*;

    /// A pose beside the sky view's fields reads, and each refusal is
    /// named under the part it stands in.
    #[test]
    fn a_pose_reads_beside_the_sky_and_each_refusal_is_named() {
        let pose = r#""pose": {"azimuthDeg": 10, "altitudeDeg": 90, "rollDeg": -180, "horizontalFovDeg": 179.9, "aspectRatio": 10}"#;
        let asked = SkyPoseRequest::from_json(&format!(
            r#"{{"latitudeDeg": 0, "longitudeDeg": 0, "at": 2461345.25, "stars": [], {pose}}}"#
        ))
        .unwrap();
        assert_eq!(asked.pose.altitude_deg, 90.0);
        assert_eq!(asked.view.options.stars, Vec::new());
        let good = r#""azimuthDeg": 0, "altitudeDeg": 0, "rollDeg": 0, "aspectRatio": 1"#;
        for (text, field) in [
            (
                String::from(r#"{"latitudeDeg": 0, "longitudeDeg": 0, "at": 2461345.25}"#),
                "skyPose.pose",
            ),
            (
                format!(
                    r#"{{"latitudeDeg": 0, "longitudeDeg": 0, "at": 1, "pose": {{{good}, "horizontalFovDeg": 180}}}}"#
                ),
                "skyPose.pose.horizontalFovDeg",
            ),
            (
                format!(r#"{{"latitudeDeg": 0, "longitudeDeg": 0, "at": 1, "pose": {{{good}}}}}"#),
                "skyPose.pose",
            ),
            (
                format!(
                    r#"{{"latitudeDeg": 0, "longitudeDeg": 0, "at": 1, "pose": {{{good}, "horizontalFovDeg": 60, "yawDeg": 1}}}}"#
                ),
                "skyPose.pose.yawDeg",
            ),
            (
                format!(
                    r#"{{"latitudeDeg": 0, "longitudeDeg": 0, "at": 1, "pose": {{{good}, "horizontalFovDeg": 60}}, "eclipticStepDeg": 0}}"#
                ),
                "skyPose.eclipticStepDeg",
            ),
            (
                format!(
                    r#"{{"latitudeDeg": 0, "longitudeDeg": 0, "at": 1, "pose": {{{good}, "horizontalFovDeg": 60}}, "grahas": ["ABHIJIT"]}}"#
                ),
                "skyPose.grahas",
            ),
            (
                format!(
                    r#"{{"latitudeDeg": 0, "longitudeDeg": 0, "at": 1, "pose": {{{good}, "horizontalFovDeg": 60}}, "posture": 1}}"#
                ),
                "skyPose.posture",
            ),
            (
                r#"{"latitudeDeg": 0, "longitudeDeg": 0, "at": 1, "pose": [1, 2]}"#.to_string(),
                "skyPose.pose",
            ),
        ] {
            let refused = SkyPoseRequest::from_json(&text).unwrap_err();
            assert!(
                refused
                    .field()
                    .is_some_and(|named| named.starts_with(field)),
                "{text}: {refused:?}"
            );
        }
        assert!(SkyPoseRequest::from_json("[]").is_err());
    }

    #[test]
    fn every_option_reads_and_each_refusal_is_named() {
        let asked = SkyViewRequest::from_json(
            r#"{"latitudeDeg": -33.8688, "longitudeDeg": 151.2093, "at": 2461345.25,
                "stars": ["star.VEGA"], "eclipticStepDeg": 2.5, "horizonStepDeg": 10,
                "pressureHpa": 0, "temperatureC": -20}"#,
        )
        .unwrap();
        assert_eq!(asked.options.stars, [Star::Vega]);
        assert_eq!(asked.options.grahas.len(), 9);
        assert_eq!(
            (
                asked.options.ecliptic_step_deg,
                asked.options.horizon_step_deg
            ),
            (2.5, 10.0)
        );
        assert_eq!(asked.options.atmosphere.factor(), 0.0);
        for (text, field) in [
            (
                r#"{"latitudeDeg": 91, "longitudeDeg": 0, "at": 2461345.25}"#,
                "skyView.latitudeDeg",
            ),
            (
                r#"{"latitudeDeg": 0, "longitudeDeg": 0, "at": 2461345.25, "pressureHpa": -1}"#,
                "skyView.pressureHpa",
            ),
            (
                r#"{"latitudeDeg": 0, "longitudeDeg": 0, "at": 2461345.25, "temperatureC": 80}"#,
                "skyView.temperatureC",
            ),
            (
                r#"{"latitudeDeg": 0, "longitudeDeg": 0, "at": 2461345.25, "eclipticStepDeg": 0}"#,
                "skyView.eclipticStepDeg",
            ),
            (
                r#"{"latitudeDeg": 0, "longitudeDeg": 0, "at": 2461345.25, "pose": {}}"#,
                "skyView.pose",
            ),
            (
                r#"{"latitudeDeg": 0, "longitudeDeg": 0, "at": 2461345.25, "grahas": ["ABHIJIT"]}"#,
                "skyView.grahas",
            ),
        ] {
            let refused = SkyViewRequest::from_json(text).unwrap_err();
            assert!(
                refused
                    .field()
                    .is_some_and(|named| named.starts_with(field)),
                "{text}: {refused:?}"
            );
        }
    }
}
