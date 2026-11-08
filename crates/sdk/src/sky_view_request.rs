//! The sky view as a binding asks it (`03-design/sky-view.md`): a place,
//! an instant, and what to draw.

use serde::{Deserialize, Serialize};
use teistro_astro::sky_view::{Atmosphere, SkyView, SkyViewOptions, sky_view};
use teistro_core::catalogue::{Graha, Star};
use teistro_core::envelope::{Convention, Envelope, content_hash};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_port_ephemeris::Astronomy;

use crate::AlmanacArea;
use crate::asked::place_of;
use crate::ephemeris::no_ephemeris;

/// The record's name where a binding sends it, which a refusal is named
/// under.
const SKY_VIEW: &str = "skyView";

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
        let asked: SkyViewAsked = teistro_core::strict::read(text, SKY_VIEW)?;
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
            .map_err(|why| why.under(SKY_VIEW))?;
        Ok(SkyViewRequest {
            place,
            at: asked.at,
            options,
        })
    }
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
        Ok(Envelope::sealing(view, provenance))
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
