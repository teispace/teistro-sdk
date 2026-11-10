//! The vakya almanac's sky behind the ephemeris port: the
//! *Vakyakarana*'s Sun and the almanac's Moon at each sunrise, between
//! sunrises by proportion, so the panchanga's tithi and nakshatra
//! searches find exactly the ends a vakya almanac prints
//! ([`super::tithi_ends`], [`super::nakshatra_ends`]).
//!
//! A vakya almanac gives its Sun and Moon for the moment of the Kali
//! day's sunrise and reads a day's fraction as the time after the
//! reader's sunrise. This sky counts the day at the place it is reckoned
//! for, Chennai by default, where the Manonmani Vilasam almanac is
//! printed and the vakya calendar is reckoned, from sunrise to sunrise by
//! the modern ephemeris beside it: a reader elsewhere in Tamil Nadu
//! counts from a sunrise a few minutes away (crux C1294). Every other
//! part of the sky, the star planets, the angles, the zodiac and the
//! obliquity, is the Surya Siddhanta's, and the rise and set are the
//! modern ephemeris's ([`ModernParts::with_sunrise`]).

use std::collections::BTreeMap;
use std::sync::Mutex;

use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro_port_ephemeris::{
    Angles, AnglesRequest, Body, Capabilities, Cell, CellStatus, EphemerisProvider, Horizon,
    HorizonEventKind, HorizonRequest, Obliquity, PositionColumns, PositionRequest, ProviderError,
    TimeScale,
};

use super::moon::{Arc, almanac_moon, true_sun};
use super::{KaliMoment, PARTS_PER_DAY};
use crate::modern::ModernParts;
use crate::provider::SiddhantaProvider;

/// The Julian day number of the civil date before Kali day 1: Kali day
/// `d` is the civil date whose noon is Julian day `d + 588,465`, so Kali
/// day 1,843,824 is 13 April 1947 (the edition's Example 1).
///
/// ```
/// use teistro_siddhanta::vakya::KALI_DAY_JD;
///
/// // 13 April 1947 at noon is JD 2,432,289.
/// assert_eq!(1_843_824 + KALI_DAY_JD, 2_432_289);
/// ```
pub const KALI_DAY_JD: i64 = 588_465;

/// Chennai, where the vakya sky's days are counted by default: the place
/// the Manonmani Vilasam almanac is printed and `TAMIL_VAKYA` is
/// reckoned.
pub const CHENNAI: Place = Place::new(
    Latitude::literal(13.0827),
    Longitude::literal(80.2707),
    Altitude::literal(0.0),
);

/// The most sunrises the sky keeps before it forgets them all: a year's
/// and a half, more than one almanac request spans.
const KEPT_SUNRISES: usize = 600;

/// The vakya almanac's sky, as an ephemeris.
///
/// ```
/// use teistro_port_ephemeris::{Body, EphemerisProvider, PositionRequest, TestProvider, TimeScale};
/// use teistro_siddhanta::vakya::VakyaSky;
///
/// let sky = VakyaSky::new(Box::new(TestProvider::new()));
/// let frame = sky.capabilities().native_frame;
/// let jds = [2_460_206.0];
/// let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun, Body::Moon], frame);
/// let columns = sky.positions(&request)?;
/// let (sun, moon) = (columns.at(0, 0).ok_or("a cell")?, columns.at(0, 1).ok_or("a cell")?);
/// // 18 September 2023: the Sun in Simha, the Moon a little over two
/// // tithis ahead of it.
/// assert!((120.0..180.0).contains(&sun.lon));
/// assert!((24.0..40.0).contains(&(moon.lon - sun.lon).rem_euclid(360.0)));
/// assert!(sky.capabilities().identity.name.contains("vakya"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct VakyaSky {
    parts: ModernParts,
    place: Place,
    horizon: Horizon,
    sunrises: Mutex<BTreeMap<i64, f64>>,
}

impl core::fmt::Debug for VakyaSky {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VakyaSky")
            .field("parts", &self.parts)
            .field("place", &self.place)
            .field("horizon", &self.horizon)
            .finish_non_exhaustive()
    }
}

impl VakyaSky {
    /// The vakya sky beside a modern ephemeris, its days counted at
    /// Chennai from the Sun's upper limb rising with standard refraction.
    ///
    /// ```
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    /// use teistro_siddhanta::vakya::{CHENNAI, VakyaSky};
    ///
    /// let sky = VakyaSky::new(Box::new(TestProvider::new()));
    /// assert_eq!(sky.place(), CHENNAI);
    /// ```
    #[must_use]
    pub fn new(modern: Box<dyn EphemerisProvider>) -> VakyaSky {
        VakyaSky::with_parts(ModernParts::new(SiddhantaProvider::text(), modern).with_sunrise())
    }

    /// The vakya sky over a text sky that already has its modern sunrise
    /// (and Delta T): what the SDK opens.
    ///
    /// ```
    /// use teistro_port_ephemeris::TestProvider;
    /// use teistro_siddhanta::vakya::VakyaSky;
    /// use teistro_siddhanta::{ModernParts, SiddhantaProvider};
    ///
    /// let parts = ModernParts::new(SiddhantaProvider::text(), Box::new(TestProvider::new())).with_sunrise();
    /// assert!(format!("{:?}", VakyaSky::with_parts(parts)).contains("VakyaSky"));
    /// ```
    #[must_use]
    pub fn with_parts(parts: ModernParts) -> VakyaSky {
        VakyaSky {
            parts,
            place: CHENNAI,
            horizon: Horizon::UPPER_LIMB_REFRACTION,
            sunrises: Mutex::new(BTreeMap::new()),
        }
    }

    /// The same sky, its days counted at another place.
    ///
    /// ```
    /// use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
    /// use teistro_port_ephemeris::TestProvider;
    /// use teistro_siddhanta::vakya::VakyaSky;
    ///
    /// let madurai = Place::new(Latitude::literal(9.9252), Longitude::literal(78.1198), Altitude::literal(0.0));
    /// assert_eq!(VakyaSky::new(Box::new(TestProvider::new())).at(madurai).place(), madurai);
    /// ```
    #[must_use]
    pub fn at(self, place: Place) -> VakyaSky {
        VakyaSky {
            place,
            sunrises: Mutex::new(BTreeMap::new()),
            ..self
        }
    }

    /// The place the sky's days are counted at.
    ///
    /// ```
    /// use teistro_port_ephemeris::TestProvider;
    /// use teistro_siddhanta::vakya::{CHENNAI, VakyaSky};
    ///
    /// assert_eq!(VakyaSky::new(Box::new(TestProvider::new())).place(), CHENNAI);
    /// ```
    #[must_use]
    pub const fn place(&self) -> Place {
        self.place
    }

    /// The sunrise that begins the civil date whose noon is Julian day
    /// `noon`, at the sky's place.
    fn sunrise(&self, noon: i64) -> Result<f64, ProviderError> {
        if let Ok(kept) = self.sunrises.lock() {
            if let Some(&at) = kept.get(&noon) {
                return Ok(at);
            }
        }
        let midnight = jd_of(noon) - 0.5 - self.place.longitude.get() / 360.0;
        let from = JulianDay::<Ut1>::try_new(midnight)
            .map_err(|error| ProviderError::invalid(error.to_string()))?;
        let at = self
            .parts
            .horizon_event(&HorizonRequest {
                body: Body::Sun,
                kind: HorizonEventKind::Rise,
                place: self.place,
                from,
                window_days: 1.0,
                horizon: self.horizon,
            })?
            .ok_or_else(|| ProviderError::Refused {
                detail: format!("no sunrise at {} to count a vakya day from", self.place),
            })?
            .get();
        if let Ok(mut kept) = self.sunrises.lock() {
            if kept.len() >= KEPT_SUNRISES {
                kept.clear();
            }
            kept.insert(noon, at);
        }
        Ok(at)
    }

    /// The Kali day an instant falls in at the sky's place, with that
    /// day's sunrise and the next.
    fn day_of(&self, ut1: f64) -> Result<(i64, f64, f64), ProviderError> {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a Julian day number, inside the provider's range"
        )]
        let mut noon = (ut1 + self.place.longitude.get() / 360.0).round() as i64;
        let mut rise = self.sunrise(noon)?;
        if ut1 < rise {
            noon -= 1;
            rise = self.sunrise(noon)?;
        }
        Ok((noon - KALI_DAY_JD, rise, self.sunrise(noon + 1)?))
    }

    /// A body's cell at an instant: its places at the two sunrises about
    /// it, joined by proportion.
    fn cell(&self, body: Body, ut1: f64, speeds: bool) -> Result<Cell, ProviderError> {
        let place: fn(i64) -> Arc = match body {
            Body::Sun => |days| true_sun(KaliMoment::from_parts(days * PARTS_PER_DAY)),
            _ => almanac_moon,
        };
        let (day, rise, next) = self.day_of(ut1)?;
        let first = place(day - 1);
        let travel = place(day).since(first).degrees();
        let length = next - rise;
        let fraction = (ut1 - rise) / length;
        Ok(Cell {
            lon: (first.degrees() + fraction * travel).rem_euclid(360.0),
            lat: 0.0,
            dist: 1.0,
            lon_speed: if speeds { travel / length } else { 0.0 },
            lat_speed: 0.0,
            dist_speed: 0.0,
            status: CellStatus::Ok,
            source: teistro_port_ephemeris::Source {
                kind: teistro_port_ephemeris::EphemerisKind::Analytic,
                tier: None,
            },
        })
    }
}

/// A Julian day number as an instant.
fn jd_of(number: i64) -> f64 {
    // Julian day numbers of any almanac date fit an i32 exactly.
    f64::from(i32::try_from(number).unwrap_or(i32::MAX))
}

impl EphemerisProvider for VakyaSky {
    fn capabilities(&self) -> Capabilities {
        let mut capabilities = self.parts.capabilities();
        capabilities.identity.name = String::from("vakyakarana");
        capabilities.identity.data_version = format!(
            "Vakyakarana (Kuppanna Sastri and Sarma, 1962): the Sun of I.4-5, the Moon of I.9-14 with the almanacs' manyadi, joined between sunrises at {} ({}); beside {}",
            self.place, self.horizon, capabilities.identity.data_version
        );
        capabilities
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        let mut columns = self.parts.positions(request)?;
        for (jd_index, jd) in request.jds.iter().enumerate() {
            let ut1 = SiddhantaProvider::ut1_of(*jd, request.scale)?.get();
            for (body_index, body) in request.bodies.iter().enumerate() {
                if !matches!(body, Body::Sun | Body::Moon) {
                    continue;
                }
                let covered = columns
                    .at(jd_index, body_index)
                    .is_some_and(|cell| cell.is_ok());
                let cell = if covered {
                    self.cell(*body, ut1, request.speeds)?
                } else {
                    Cell::failed(CellStatus::OutOfRange)
                };
                columns.set_at(jd_index, body_index, cell);
            }
        }
        Ok(columns)
    }

    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        self.parts.obliquity(jd, scale)
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        self.parts.delta_t_seconds(jd_ut1)
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: teistro_core::catalogue::Ayanamsha,
    ) -> Result<f64, ProviderError> {
        self.parts.ayanamsha_deg(jd, scale, ayanamsha)
    }

    fn angles(&self, request: &AnglesRequest) -> Result<Angles, ProviderError> {
        self.parts.angles(request)
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        self.parts.horizon_event(request)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use teistro_core::catalogue::Ayanamsha;
    use teistro_port_ephemeris::TestProvider;

    use super::*;

    fn sky() -> VakyaSky {
        VakyaSky::new(Box::new(TestProvider::new()))
    }

    /// 18 September 2023, noon at Chennai, in UT1.
    const NOON: f64 = 2_460_206.0;

    #[test]
    fn the_sky_names_itself_and_moves_to_another_place() {
        let madurai = Place::new(
            Latitude::literal(9.9252),
            Longitude::literal(78.1198),
            Altitude::literal(0.0),
        );
        let sky = sky().at(madurai);
        assert_eq!(sky.place(), madurai);
        assert!(format!("{sky:?}").contains("VakyaSky"));
        let identity = sky.capabilities().identity;
        assert_eq!(identity.name, "vakyakarana");
        assert!(identity.data_version.contains("Vakyakarana"));
    }

    #[test]
    fn the_sun_and_moon_are_the_almanacs_and_the_rest_the_texts() {
        let sky = sky();
        let frame = sky.capabilities().native_frame;
        let jds = [NOON];
        let bodies = [Body::Sun, Body::Moon, Body::Mars];
        let mut request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, frame);
        let columns = sky.positions(&request).unwrap();
        let (sun, moon) = (columns.at(0, 0).unwrap(), columns.at(0, 1).unwrap());
        // The Sun moves about a degree a day and the Moon about thirteen;
        // the analytic latitude and distance are the vakya's flat ones.
        assert!((0.9..1.1).contains(&sun.lon_speed), "{}", sun.lon_speed);
        assert!((11.0..16.0).contains(&moon.lon_speed), "{}", moon.lon_speed);
        assert_eq!((moon.lat, moon.dist), (0.0, 1.0));
        // Mars is the text's, untouched: the ephemeris beside it answers.
        let text = sky.parts.positions(&request).unwrap();
        assert_eq!(columns.at(0, 2), text.at(0, 2));
        // Asked without speeds, the places stand and the speeds are none.
        request.speeds = false;
        let still = sky.positions(&request).unwrap();
        assert_eq!(still.at(0, 0).unwrap().lon, sun.lon);
        assert_eq!(still.at(0, 0).unwrap().lon_speed, 0.0);
    }

    #[test]
    fn an_instant_before_sunrise_belongs_to_the_day_before() {
        let sky = sky();
        let (day, rise, next) = sky.day_of(NOON).unwrap();
        assert!(rise < NOON && NOON < next);
        // Chennai's midnight, before the day's sunrise, is yesterday's.
        let (before, _, ends) = sky.day_of(rise - 0.25).unwrap();
        assert_eq!(before, day - 1);
        assert!((ends - rise).abs() < 1e-9);
    }

    #[test]
    fn the_kept_sunrises_are_forgotten_whole_past_their_limit() {
        let sky = sky();
        let jds: Vec<f64> = (0..KEPT_SUNRISES + 10)
            .map(|day| NOON + f64::from(u32::try_from(day).unwrap()))
            .collect();
        let frame = sky.capabilities().native_frame;
        let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], frame);
        let columns = sky.positions(&request).unwrap();
        assert!(columns.at(jds.len() - 1, 0).unwrap().is_ok());
        let kept = sky.sunrises.lock().unwrap().len();
        assert!(kept < KEPT_SUNRISES, "{kept}");
        // A day asked again reads the kept sunrise, the same to the bit.
        let last = jds[jds.len() - 1];
        assert_eq!(sky.day_of(last).unwrap(), sky.day_of(last).unwrap());
    }

    #[test]
    fn every_other_part_of_the_sky_is_the_texts() {
        let sky = sky();
        let parts = &sky.parts;
        let same = |ours: String, text: String| assert_eq!(ours, text);
        same(
            format!("{:?}", sky.obliquity(NOON, TimeScale::Ut1)),
            format!("{:?}", parts.obliquity(NOON, TimeScale::Ut1)),
        );
        // Delta T is the text sky's answer, a refusal included.
        same(
            format!("{:?}", sky.delta_t_seconds(NOON)),
            format!("{:?}", parts.delta_t_seconds(NOON)),
        );
        same(
            format!(
                "{:?}",
                sky.ayanamsha_deg(NOON, TimeScale::Ut1, Ayanamsha::Suryasiddhanta)
            ),
            format!(
                "{:?}",
                parts.ayanamsha_deg(NOON, TimeScale::Ut1, Ayanamsha::Suryasiddhanta)
            ),
        );
        let angles = AnglesRequest {
            jd: NOON,
            scale: TimeScale::Ut1,
            place: CHENNAI,
        };
        same(
            format!("{:?}", sky.angles(&angles)),
            format!("{:?}", parts.angles(&angles)),
        );
        let rise = HorizonRequest {
            body: Body::Sun,
            kind: HorizonEventKind::Rise,
            place: CHENNAI,
            from: JulianDay::<Ut1>::try_new(NOON - 0.5).unwrap(),
            window_days: 1.0,
            horizon: Horizon::UPPER_LIMB_REFRACTION,
        };
        same(
            format!("{:?}", sky.horizon_event(&rise)),
            format!("{:?}", parts.horizon_event(&rise)),
        );
    }
}
