//! The model behind the ephemeris port: the text's places, latitudes and
//! daily motions as positions in the text's own sidereal frame, with the
//! text's obliquity, its precession as the ayanamsha, and its sunrise as
//! the rise and set override for the Sun, so the chart layer reads the
//! Surya Siddhanta as it reads an engine
//! (`docs/03-design/siddhanta.md`, §5; `ephemeris-port-and-adapters.md`).
//!
//! Distances are the text's: the hypotenuse over the radius for the star
//! planets, the mean motion over the true motion for the Sun and the
//! Moon (IV.2 to 3, where the apparent diameter grows with the true
//! motion), so 1 is the mean distance and the capabilities say so.
//! Speeds are the text's daily motions, a rule rather than the
//! derivative of its places, and the capabilities say that too.

use teistro_astro::{DeltaTModel, ut1_from_tt};
use teistro_core::catalogue::{Ayanamsha, Graha};
use teistro_core::quantity::{JulianDay, Latitude, Ut1};
use teistro_port_ephemeris::{
    Astronomy, Body, Capabilities, Cell, CellStatus, Centre, Coordinates, Corrections, DiscPoint,
    DistanceUnit, EphemerisKind, EphemerisProvider, Equinox, Frame, Horizon, HorizonEventKind,
    HorizonRequest, Identity, Obliquity, Overrides, PositionColumns, PositionRequest,
    ProviderError, Refraction, Source, SpeedModel, TimeScale, Zodiac, validate,
};

use crate::model::{SuryaSiddhanta, Trace};
use crate::params::Planet;
use crate::trig::RADIUS;

/// The last Julian day the provider answers for: a few thousand years
/// on, well inside the text's own age.
const LAST_JD: f64 = 3_000_000.0;

/// The Surya Siddhanta as an ephemeris provider.
///
/// ```
/// use teistro_port_ephemeris::{Body, EphemerisProvider, PositionRequest, TimeScale};
/// use teistro_siddhanta::SiddhantaProvider;
///
/// let provider = SiddhantaProvider::text();
/// let frame = provider.capabilities().native_frame;
/// let jds = [2_460_413.5];
/// let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun, Body::Moon], frame);
/// let columns = provider.positions(&request).expect("the text's frame");
/// let sun = columns.at(0, 0).expect("a cell");
/// assert!(sun.is_ok() && (sun.lon < 1.0 || sun.lon > 359.0));
/// ```
#[derive(Clone, Debug)]
pub struct SiddhantaProvider {
    model: SuryaSiddhanta,
}

impl SiddhantaProvider {
    /// The text as the provider.
    #[must_use]
    pub const fn text() -> SiddhantaProvider {
        SiddhantaProvider {
            model: SuryaSiddhanta::text(),
        }
    }

    /// Any model (a bija applied, exact trigonometry) as the provider.
    #[must_use]
    pub const fn new(model: SuryaSiddhanta) -> SiddhantaProvider {
        SiddhantaProvider { model }
    }

    /// The model.
    #[must_use]
    pub const fn model(&self) -> &SuryaSiddhanta {
        &self.model
    }

    /// The bodies the text models, in the port's order: the seven, the
    /// Moon's node and its apogee.
    pub const BODIES: [Body; 9] = [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::MeanNode,
        Body::MeanApogee,
    ];

    /// The frame the text answers in: its own sidereal ecliptic of date,
    /// geocentric, without the modern corrections.
    pub const FRAME: Frame = Frame {
        centre: Centre::Geocentric,
        equinox: Equinox::OfDate,
        coordinates: Coordinates::Ecliptic,
        zodiac: Zodiac::Sidereal {
            ayanamsha: Ayanamsha::Suryasiddhanta,
        },
        corrections: Corrections::GEOMETRIC,
    };

    /// The UT1 instant of a request's instant: a TT instant through the
    /// SDK's Delta T, since the text reckons in mean solar time.
    fn ut1_of(jd: f64, scale: TimeScale) -> Result<JulianDay<Ut1>, ProviderError> {
        let invalid = |error: &dyn core::fmt::Display| ProviderError::invalid(error.to_string());
        match scale {
            TimeScale::Ut1 => JulianDay::try_new(jd).map_err(|e| invalid(&e)),
            TimeScale::Tt => {
                let tt = JulianDay::try_new(jd).map_err(|e| invalid(&e))?;
                ut1_from_tt(tt, DeltaTModel::TableThenModel)
                    .map(|(ut1, _)| ut1)
                    .map_err(|e| invalid(&e))
            }
        }
    }

    /// The trace of a body at a UT1 instant.
    fn trace(&self, body: Body, at: JulianDay<Ut1>) -> Option<Trace> {
        let graha = match body {
            Body::Sun => Graha::Sun,
            Body::Moon => Graha::Moon,
            Body::Mercury => Graha::Mercury,
            Body::Venus => Graha::Venus,
            Body::Mars => Graha::Mars,
            Body::Jupiter => Graha::Jupiter,
            Body::Saturn => Graha::Saturn,
            Body::MeanNode => Graha::Rahu,
            Body::MeanApogee => return Some(self.model.moon_apogee_trace(at)),
            _ => return None,
        };
        self.model.trace(graha, at).ok()
    }

    /// A body's cell at a UT1 instant.
    fn cell(&self, body: Body, at: JulianDay<Ut1>, speeds: bool) -> Cell {
        let Some(trace) = self.trace(body, at) else {
            return Cell::failed(CellStatus::UnsupportedBody);
        };
        // The distance relative to the mean: the hypotenuse over the
        // radius for a star planet; the mean over the true motion for the
        // Sun and the Moon (IV.2 to 3); none for the points.
        let dist = match body {
            Body::Sun | Body::Moon => {
                let planet = if body == Body::Sun {
                    Planet::Sun
                } else {
                    Planet::Moon
                };
                let mean = self
                    .model
                    .parameters()
                    .motion(planet)
                    .degrees_per_day(self.model.parameters());
                if trace.speed_deg_per_day > 0.0 {
                    mean / trace.speed_deg_per_day
                } else {
                    1.0
                }
            }
            Body::MeanNode | Body::MeanApogee => 0.0,
            _ => trace.karna.map_or(1.0, |karna| karna / RADIUS),
        };
        Cell {
            lon: trace.longitude_deg,
            lat: trace.latitude_deg,
            dist,
            lon_speed: if speeds { trace.speed_deg_per_day } else { 0.0 },
            lat_speed: 0.0,
            dist_speed: 0.0,
            status: CellStatus::Ok,
            source: Source {
                kind: EphemerisKind::Analytic,
                tier: None,
            },
        }
    }

    /// The identity: the text and the model's stamp.
    fn identity(&self) -> Identity {
        Identity {
            name: String::from("surya-siddhanta"),
            version: String::from(env!("CARGO_PKG_VERSION")),
            data_version: self.model.describe(),
            tier: None,
            data_hashes: Vec::new(),
        }
    }
}

impl EphemerisProvider for SiddhantaProvider {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            identity: self.identity(),
            jd_range: (self.model.parameters().epoch_jd_ut, LAST_JD),
            bodies: SiddhantaProvider::BODIES.to_vec(),
            native_frame: SiddhantaProvider::FRAME,
            astronomy: Astronomy::Classical,
            speeds: true,
            speed_model: SpeedModel::Rule,
            distance_unit: DistanceUnit::MeanDistances,
            overrides: Overrides::OBLIQUITY
                .with(Overrides::AYANAMSHA)
                .with(Overrides::RISE_SET),
            ayanamshas: vec![Ayanamsha::Suryasiddhanta],
            deterministic: true,
        }
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        let capabilities = self.capabilities();
        validate(&capabilities, request)?;
        if request.frame != SiddhantaProvider::FRAME {
            return Err(ProviderError::unsupported(format!(
                "a frame other than the text's own ({})",
                SiddhantaProvider::FRAME
            )));
        }
        let mut columns =
            PositionColumns::new(request.jds.len(), request.bodies.len(), request.frame);
        for (jd_index, jd) in request.jds.iter().enumerate() {
            let at = SiddhantaProvider::ut1_of(*jd, request.scale)?;
            for (body_index, body) in request.bodies.iter().enumerate() {
                let cell = if capabilities.covers(at.get()) {
                    self.cell(*body, at, request.speeds)
                } else {
                    Cell::failed(CellStatus::OutOfRange)
                };
                columns.set_at(jd_index, body_index, cell);
            }
        }
        Ok(columns)
    }

    fn obliquity(&self, _jd: f64, _scale: TimeScale) -> Result<Obliquity, ProviderError> {
        // The text's greatest declination (II.28), the arc of its sine on
        // the radius; the text has no nutation.
        let obliquity = self
            .model
            .trig()
            .arc(f64::from(self.model.parameters().obliquity_sine));
        Ok(Obliquity {
            mean_deg: obliquity,
            true_deg: obliquity,
            nutation_lon_deg: 0.0,
            nutation_obl_deg: 0.0,
        })
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: Ayanamsha,
    ) -> Result<f64, ProviderError> {
        if ayanamsha != Ayanamsha::Suryasiddhanta {
            return Err(ProviderError::unsupported(format!(
                "the {} ayanamsha; the text knows its own (SURYASIDDHANTA)",
                ayanamsha.key()
            )));
        }
        let at = SiddhantaProvider::ut1_of(jd, scale)?;
        Ok(self.model.ayanamsha_deg(at))
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        if request.body != Body::Sun {
            return Err(ProviderError::unsupported(format!(
                "the rise and set of {} in the text; it gives the Sun's",
                request.body.key()
            )));
        }
        let geometric = Horizon {
            disc: DiscPoint::Centre,
            refraction: Refraction::None,
            altitude_deg: 0.0,
        };
        if request.horizon != geometric {
            return Err(ProviderError::unsupported(format!(
                "the {} convention; the text gives the centre on the geometric horizon",
                request.horizon
            )));
        }
        let kind = match request.kind {
            HorizonEventKind::Rise | HorizonEventKind::Set => request.kind,
            other => {
                return Err(ProviderError::unsupported(format!(
                    "a {other} in the text's day arc"
                )));
            }
        };
        let latitude: Latitude = request.place.latitude;
        let start = request.from.get();
        let end = start + request.window_days;
        // The local mean day the search begins in, then the following
        // days, until the event falls inside the window or the window ends.
        let mut midnight =
            SuryaSiddhanta::local_mean_midnight(request.from, request.place.longitude);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a small day count"
        )]
        let days = (request.window_days.ceil() as u32).saturating_add(1);
        for _ in 0..days {
            if let Some(arc) = self.model.day_arc(midnight, latitude) {
                let instant = match kind {
                    HorizonEventKind::Rise => arc.sunrise,
                    _ => arc.sunset,
                };
                if instant.get() >= start && instant.get() < end {
                    return Ok(Some(instant));
                }
                if instant.get() >= end {
                    return Ok(None);
                }
            }
            midnight = midnight
                .plus_days(1.0)
                .map_err(|error| ProviderError::invalid(error.to_string()))?;
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::quantity::{Altitude, Longitude, Place};

    use super::*;

    #[test]
    fn the_text_answers_the_port_in_its_own_frame() {
        let provider = SiddhantaProvider::text();
        let capabilities = provider.capabilities();
        assert_eq!(capabilities.native_frame, SiddhantaProvider::FRAME);
        assert_eq!(capabilities.distance_unit, DistanceUnit::MeanDistances);
        assert_eq!(capabilities.speed_model, SpeedModel::Rule);
        assert_eq!(capabilities.astronomy, Astronomy::Classical);
        assert!(capabilities.has_ayanamsha(Ayanamsha::Suryasiddhanta));
        assert!(capabilities.describe().contains("MEAN_DISTANCES"));
        let jds = [2_460_413.5, 2_400_410.714];
        let request = PositionRequest::new(
            &jds,
            TimeScale::Ut1,
            &SiddhantaProvider::BODIES,
            SiddhantaProvider::FRAME,
        );
        let columns = provider.positions(&request).unwrap();
        assert!(columns.all_ok(), "{:?}", columns.status);
        for cell in columns.cells() {
            assert!((0.0..360.0).contains(&cell.lon) && cell.lat.abs() < 10.0);
        }
        // The Moon's distance rises and falls about the mean; the points
        // have none; a star planet's is its hypotenuse on the radius.
        let moon = columns.at(0, 1).unwrap();
        assert!((0.9..1.1).contains(&moon.dist), "{}", moon.dist);
        assert!(columns.at(0, 7).unwrap().dist.abs() < f64::EPSILON);
        let mars = columns.at(0, 4).unwrap();
        assert!(mars.dist > 0.3 && mars.dist < 2.0, "{}", mars.dist);
        // A frame the text does not answer in is refused; TT is accepted.
        let tropical = request.in_frame(Frame::CANONICAL);
        assert!(matches!(
            provider.positions(&tropical),
            Err(ProviderError::Unsupported { .. })
        ));
        let tt = PositionRequest::new(&jds, TimeScale::Tt, &[Body::Sun], SiddhantaProvider::FRAME);
        let terrestrial = provider.positions(&tt).unwrap().at(0, 0).unwrap();
        let universal = columns.at(0, 0).unwrap();
        let apart = (terrestrial.lon - universal.lon).abs();
        assert!(apart < 0.01 && apart > 0.0, "{apart}");
        // An instant before the epoch is out of range.
        let early = PositionRequest::new(
            &[500_000.0],
            TimeScale::Ut1,
            &[Body::Sun],
            SiddhantaProvider::FRAME,
        );
        assert_eq!(
            provider.positions(&early).unwrap().at(0, 0).unwrap().status,
            CellStatus::OutOfRange
        );
    }

    #[test]
    fn the_overrides_are_the_texts_obliquity_ayanamsha_and_sunrise() {
        let provider = SiddhantaProvider::text();
        let from = JulianDay::literal(2_460_413.5);
        let obliquity = provider.obliquity(2_460_413.5, TimeScale::Ut1).unwrap();
        assert!(
            (obliquity.true_deg - 24.0).abs() < 0.05,
            "{}",
            obliquity.true_deg
        );
        assert!(
            (provider
                .ayanamsha_deg(2_460_413.5, TimeScale::Ut1, Ayanamsha::Suryasiddhanta)
                .unwrap()
                - 22.87)
                .abs()
                < 0.05
        );
        assert!(
            provider
                .ayanamsha_deg(2_460_413.5, TimeScale::Ut1, Ayanamsha::Lahiri)
                .is_err()
        );
        let kathmandu = Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        );
        let sunrise = provider
            .horizon_event(&HorizonRequest {
                body: Body::Sun,
                kind: HorizonEventKind::Rise,
                place: kathmandu,
                from,
                window_days: 1.0,
                horizon: Horizon::CENTRE_NO_REFRACTION,
            })
            .unwrap()
            .unwrap();
        assert!(sunrise.get() >= from.get() && sunrise.get() < from.get() + 1.0);
        let sunset = provider
            .horizon_event(&HorizonRequest {
                body: Body::Sun,
                kind: HorizonEventKind::Set,
                place: kathmandu,
                from: sunrise,
                window_days: 1.0,
                horizon: Horizon::CENTRE_NO_REFRACTION,
            })
            .unwrap()
            .unwrap();
        assert!(sunset.get() - sunrise.get() > 0.5);
        let refracted = provider.horizon_event(&HorizonRequest {
            body: Body::Sun,
            kind: HorizonEventKind::Rise,
            place: kathmandu,
            from,
            window_days: 1.0,
            horizon: Horizon::UPPER_LIMB_REFRACTION,
        });
        assert!(matches!(refracted, Err(ProviderError::Unsupported { .. })));
        let moon = provider.horizon_event(&HorizonRequest {
            body: Body::Moon,
            kind: HorizonEventKind::Rise,
            place: kathmandu,
            from,
            window_days: 1.0,
            horizon: Horizon::CENTRE_NO_REFRACTION,
        });
        assert!(matches!(moon, Err(ProviderError::Unsupported { .. })));
        assert!(
            SiddhantaProvider::new(SuryaSiddhanta::text())
                .model()
                .describe()
                .starts_with("Surya")
        );
    }
}
