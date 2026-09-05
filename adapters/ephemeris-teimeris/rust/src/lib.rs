//! The Teimeris adapter: the ephemeris port over Teimeris's Rust binding.
//!
//! Teimeris returns the SDK's canonical frame natively and offers the
//! obliquity, Delta T and ayanamsha overrides, so the adapter is a
//! translation of vocabularies plus the three rules of the port the engine
//! does not enforce itself:
//!
//! - a cell the engine answered from its analytic fallback is reported as
//!   missing data rather than passed off as a file position;
//! - an instant outside the declared coverage is reported as out of range,
//!   whatever the engine did with it;
//! - the engine's own status codes, which collide with the port's reserved
//!   range, are carried with an offset.
//!
//! One context per adapter, behind a mutex: the sidereal mode is context
//! state, and the port's requests carry frames rather than state
//! (ADR-0002), so a request sets the state it needs and computes under
//! the same hold of the lock. The engine's own rise and set search is
//! offered as the rise and set override under the port's horizon
//! conventions.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use teimeris::{
    Body as TmBody, Columns, Context, ErrorKind, EventKind, EventOption, EventOptions, Flags,
    Observer as TmObserver, Profile, TimeScale as TmScale,
};
use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_port_ephemeris::{
    Astronomy, Body, Capabilities, Cell, CellStatus, Centre, Coordinates, DiscPoint, DistanceUnit,
    EphemerisKind, EphemerisProvider, Equinox, Frame, HorizonEventKind, HorizonRequest, Identity,
    Obliquity, Overrides, PositionColumns, PositionRequest, ProviderError, Refraction, Source,
    SpeedModel, TimeScale, Zodiac, sefile, validate,
};

/// The environment variable naming the data directory.
pub const DATA_DIR_ENV: &str = "TEIMERIS_DATA_DIR";

/// The offset added to the engine's status codes so they stay clear of the
/// port's reserved range (-1 to -6): the engine's -2 arrives as -102.
pub const CODE_BASE: i32 = -100;

/// The port over one Teimeris context.
pub struct TeimerisProvider {
    context: Mutex<Context>,
    capabilities: Capabilities,
}

impl core::fmt::Debug for TeimerisProvider {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TeimerisProvider")
            .field("version", &self.capabilities.identity.version)
            .field("jd_range", &self.capabilities.jd_range)
            .finish_non_exhaustive()
    }
}

/// The data directory: [`DATA_DIR_ENV`], else the Teimeris checkout
/// beside the SDK checkout.
#[must_use]
pub fn data_dir_from_env() -> PathBuf {
    std::env::var_os(DATA_DIR_ENV).map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../teimeris/data"),
        PathBuf::from,
    )
}

/// The engine's error in the port's vocabulary. `jd` is the instant the
/// call was about, for the out-of-range case.
fn map_error(error: &teimeris::Error, jd: f64) -> ProviderError {
    let detail = error.to_string();
    match error.kind() {
        ErrorKind::OutOfRange => ProviderError::OutOfRange { jd },
        ErrorKind::NoEphemeris | ErrorKind::DataCorrupt | ErrorKind::Io => {
            ProviderError::DataMissing { detail }
        }
        ErrorKind::Unsupported => ProviderError::Unsupported { what: detail },
        ErrorKind::InvalidArg => ProviderError::Invalid { detail },
        _ => ProviderError::Provider {
            code: CODE_BASE + error.status.raw(),
            detail,
        },
    }
}

/// The engine's per-cell status in the port's vocabulary.
fn map_status(code: i32) -> CellStatus {
    match teimeris::Status(code) {
        teimeris::Status::OK => CellStatus::Ok,
        teimeris::Status::ERR_OUT_OF_RANGE => CellStatus::OutOfRange,
        teimeris::Status::ERR_NO_EPHEMERIS
        | teimeris::Status::ERR_DATA_CORRUPT
        | teimeris::Status::ERR_IO => CellStatus::DataMissing,
        teimeris::Status::ERR_UNSUPPORTED => CellStatus::UnsupportedBody,
        _ => CellStatus::Provider {
            code: CODE_BASE + code,
        },
    }
}

/// A flag as the bit it occupies in the engine's `flags_used`.
#[allow(
    clippy::cast_sign_loss,
    reason = "flag bits are single set bits of a non-negative value"
)]
const fn bit(flag: Flags) -> u32 {
    flag.raw() as u32
}

/// Which ephemeris answered, from the engine's `flags_used`.
const fn map_source(flags_used: u32) -> Source {
    let kind = if flags_used & bit(Flags::EPH_JPL) != 0 {
        EphemerisKind::Jpl
    } else if flags_used & bit(Flags::EPH_MOSHIER) != 0 {
        EphemerisKind::Analytic
    } else if flags_used & bit(Flags::EPH_SWISS) != 0 {
        EphemerisKind::Files
    } else {
        EphemerisKind::Unknown
    };
    Source { kind, tier: None }
}

/// The engine's body for one of the port's; `None` for a body added to the
/// port after this adapter, which the capabilities then do not list.
const fn map_body(body: Body) -> Option<TmBody> {
    Some(match body {
        Body::Sun => TmBody::SUN,
        Body::Moon => TmBody::MOON,
        Body::Mercury => TmBody::MERCURY,
        Body::Venus => TmBody::VENUS,
        Body::Mars => TmBody::MARS,
        Body::Jupiter => TmBody::JUPITER,
        Body::Saturn => TmBody::SATURN,
        Body::Uranus => TmBody::URANUS,
        Body::Neptune => TmBody::NEPTUNE,
        Body::Pluto => TmBody::PLUTO,
        Body::MeanNode => TmBody::MEAN_NODE,
        Body::TrueNode => TmBody::TRUE_NODE,
        Body::MeanApogee => TmBody::MEAN_APOGEE,
        Body::OsculatingApogee => TmBody::OSCU_APOGEE,
        _ => return None,
    })
}

/// The engine's bodies for a request's, in the same order.
fn engine_bodies(bodies: &[Body]) -> Result<Vec<TmBody>, ProviderError> {
    bodies
        .iter()
        .map(|body| map_body(*body).ok_or_else(|| ProviderError::unsupported(body.key())))
        .collect()
}

const fn map_scale(scale: TimeScale) -> TmScale {
    match scale {
        TimeScale::Ut1 => TmScale::UT1,
        TimeScale::Tt => TmScale::TT,
    }
}

/// The engine's flags for a frame: every switch explicit, the file
/// ephemeris always named so the engine never picks one for us.
fn map_flags(frame: Frame, speeds: bool) -> Flags {
    let mut flags = Flags::EPH_SWISS;
    let switches = [
        (!speeds, Flags::NO_SPEED),
        (
            frame.coordinates == Coordinates::Equatorial,
            Flags::EQUATORIAL,
        ),
        (frame.equinox == Equinox::J2000, Flags::J2000),
        (!frame.corrections.nutation, Flags::NO_NUTATION),
        (!frame.corrections.light_time, Flags::NO_LIGHT_TIME),
        (!frame.corrections.aberration, Flags::NO_ABERRATION),
        (!frame.corrections.deflection, Flags::NO_DEFLECTION),
        (frame.centre == Centre::Topocentric, Flags::TOPOCENTRIC),
        (frame.centre == Centre::Heliocentric, Flags::HELIOCENTRIC),
        (frame.centre == Centre::Barycentric, Flags::BARYCENTRIC),
        (
            matches!(frame.zodiac, Zodiac::Sidereal { .. }),
            Flags::SIDEREAL,
        ),
    ];
    for (on, flag) in switches {
        if on {
            flags |= flag;
        }
    }
    flags
}

impl TeimerisProvider {
    /// Opens a context over a data directory of `.se1` files, in the
    /// engine's compatible profile, and hashes the files it will read.
    ///
    /// # Errors
    ///
    /// When the directory holds no planet block, or the engine cannot open.
    pub fn open(data_dir: &Path) -> Result<TeimerisProvider, ProviderError> {
        let files = sefile::scan(data_dir).map_err(|error| ProviderError::DataMissing {
            detail: format!("{}: {error}", data_dir.display()),
        })?;
        let Some(jd_range) = files.jd_range else {
            return Err(ProviderError::DataMissing {
                detail: format!("no planet block (sepl_*.se1) in {}", data_dir.display()),
            });
        };
        let context = Context::builder()
            .ephemeris_path(data_dir.to_string_lossy())
            .profile(Profile::COMPATIBLE)
            .open()
            .map_err(|error| map_error(&error, f64::NAN))?;
        let ayanamshas = files.ayanamshas();
        let capabilities = Capabilities {
            identity: Identity {
                name: String::from("teimeris"),
                version: teimeris::version().to_string(),
                data_version: files.names.join(" "),
                tier: None,
                data_hashes: files.hashes,
            },
            jd_range,
            bodies: Body::ALL
                .iter()
                .copied()
                .filter(|body| map_body(*body).is_some())
                .collect(),
            native_frame: Frame::CANONICAL,
            astronomy: Astronomy::Modern,
            speeds: true,
            speed_model: SpeedModel::Derivative,
            distance_unit: DistanceUnit::AstronomicalUnits,
            overrides: Overrides::OBLIQUITY
                .with(Overrides::DELTA_T)
                .with(Overrides::AYANAMSHA)
                .with(Overrides::TOPOCENTRIC)
                .with(Overrides::RISE_SET),
            ayanamshas,
            deterministic: true,
        };
        Ok(TeimerisProvider {
            context: Mutex::new(context),
            capabilities,
        })
    }

    /// Runs `f` with the context, serialised.
    fn with_context<T>(&self, f: impl FnOnce(&Context) -> T) -> T {
        let guard = self.context.lock().unwrap_or_else(PoisonError::into_inner);
        f(&guard)
    }

    /// The engine's own batch call in the canonical frame: the benchmark's
    /// direct-binding row, the cost the port is measured against.
    ///
    /// # Errors
    ///
    /// The engine's error.
    pub fn direct_columns(&self, jds: &[f64], bodies: &[Body]) -> Result<Columns, ProviderError> {
        let tm_bodies = engine_bodies(bodies)?;
        self.with_context(|ctx| {
            ctx.positions_columns_at(
                jds,
                &tm_bodies,
                map_flags(Frame::CANONICAL, true),
                TmScale::UT1,
                None,
            )
            .map_err(|partial| map_error(&partial.error, f64::NAN))
        })
    }
}

impl EphemerisProvider for TeimerisProvider {
    fn capabilities(&self) -> Capabilities {
        self.capabilities.clone()
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        validate(&self.capabilities, request)?;
        let tm_bodies = engine_bodies(request.bodies)?;
        let observer = request.observer.map(engine_observer);
        let flags = map_flags(request.frame, request.speeds);
        let raw = self.with_context(|ctx| {
            if let Zodiac::Sidereal { ayanamsha } = request.frame.zodiac {
                ctx.set_ayanamsha(engine_mode(ayanamsha), 0.0, 0.0)
                    .map_err(|error| map_error(&error, f64::NAN))?;
            }
            // A batch that partly failed still carries every cell's own status.
            Ok::<Columns, ProviderError>(
                ctx.positions_columns_at(
                    request.jds,
                    &tm_bodies,
                    flags,
                    map_scale(request.scale),
                    observer,
                )
                .unwrap_or_else(|partial| partial.results),
            )
        })?;
        // The engine's grid is body-major (`body × instants + instant`); the
        // port's is instants-outermost (`instant × bodies + body`).
        let jd_count = request.jds.len();
        let mut columns = PositionColumns::new(jd_count, request.bodies.len(), request.frame);
        for (jd_index, jd) in request.jds.iter().enumerate() {
            for body_index in 0..request.bodies.len() {
                let (Some(index), Some(engine)) = (
                    columns.index(jd_index, body_index),
                    body_index.checked_mul(jd_count).map(|b| b + jd_index),
                ) else {
                    continue;
                };
                let at = |v: &[f64]| v.get(engine).copied().unwrap_or(f64::NAN);
                let source = map_source(raw.flags_used.get(engine).copied().unwrap_or(0));
                let engine_status = map_status(raw.status.get(engine).copied().unwrap_or(-6));
                let status = if !self.capabilities.covers(*jd) {
                    CellStatus::OutOfRange
                } else if engine_status == CellStatus::Ok && source.kind == EphemerisKind::Analytic
                {
                    // The engine fell back to its analytic model for this cell;
                    // the port reports missing data rather than a position from
                    // a model the caller did not ask for.
                    CellStatus::DataMissing
                } else {
                    engine_status
                };
                let cell = if status == CellStatus::Ok {
                    Cell {
                        lon: at(&raw.lon),
                        lat: at(&raw.lat),
                        dist: at(&raw.dist),
                        lon_speed: at(&raw.lon_speed),
                        lat_speed: at(&raw.lat_speed),
                        dist_speed: at(&raw.dist_speed),
                        status,
                        source,
                    }
                } else {
                    Cell {
                        status,
                        ..Cell::EMPTY
                    }
                };
                columns.set(index, cell);
            }
        }
        Ok(columns)
    }

    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        self.with_context(|ctx| ctx.obliquity(jd, map_scale(scale)))
            .map(|o| Obliquity {
                mean_deg: o.mean_obliquity,
                true_deg: o.true_obliquity,
                nutation_lon_deg: o.nutation_lon,
                nutation_obl_deg: o.nutation_obl,
            })
            .map_err(|error| map_error(&error, jd))
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        self.with_context(|ctx| ctx.delta_t(jd_ut1))
            .map_err(|error| map_error(&error, jd_ut1))
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: Ayanamsha,
    ) -> Result<f64, ProviderError> {
        if !self.capabilities.has_ayanamsha(ayanamsha) {
            return Err(ProviderError::unsupported(format!(
                "the {} ayanamsha",
                ayanamsha.key()
            )));
        }
        // The mean ayanamsha, the value sidereal longitudes subtract; the
        // engine's default adds the nutation in longitude, and the call takes
        // no other flag.
        self.with_context(|ctx| {
            ctx.set_ayanamsha(engine_mode(ayanamsha), 0.0, 0.0)?;
            ctx.ayanamsha(jd, map_scale(scale), Flags::NO_NUTATION)
        })
        .map_err(|error| map_error(&error, jd))
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        let Some(body) = map_body(request.body) else {
            return Err(ProviderError::unsupported(request.body.key()));
        };
        let options = EventOptions {
            kind: match request.kind {
                HorizonEventKind::Rise => EventKind::RISE,
                HorizonEventKind::Set => EventKind::SET,
                HorizonEventKind::Transit => EventKind::TRANSIT,
                HorizonEventKind::Antitransit => EventKind::ANTITRANSIT,
            },
            options: EventOption(engine_horizon_options(request)?),
            flags: Flags::EPH_SWISS,
            atmosphere: None,
            horizon_height: 0.0,
            jd_end: request.from.get() + request.window_days,
        };
        let from = request.from.get();
        let found = self.with_context(|ctx| {
            match ctx
                .events(from, body, engine_observer(request.place), &options)
                .next()
            {
                Some(Ok(event)) => Ok(event.found.then_some(event.jd)),
                Some(Err(error)) => Err(map_error(&error, from)),
                None => Ok(None),
            }
        })?;
        match found {
            Some(jd) => JulianDay::try_new(jd)
                .map(Some)
                .map_err(|error| ProviderError::invalid(error.to_string())),
            None => Ok(None),
        }
    }
}

/// The engine's observer for a place.
fn engine_observer(place: Place) -> TmObserver {
    TmObserver::new(
        place.longitude.get(),
        place.latitude.get(),
        place.altitude.get(),
    )
}

/// The engine's sidereal mode for a catalogued ayanamsha: the numbering
/// both engines share, carried by the catalogue.
fn engine_mode(ayanamsha: Ayanamsha) -> i32 {
    i32::from(ayanamsha.attributes().swiss_mode)
}

/// The engine's event options for a horizon convention: the disc point,
/// the refraction, and a twilight depression of the Sun where the
/// convention's altitude names one; any other altitude is refused, since
/// the engine's horizon height means a raised horizon, not a depression.
fn engine_horizon_options(request: &HorizonRequest) -> Result<i32, ProviderError> {
    let horizon = request.horizon;
    let mut options = match horizon.disc {
        DiscPoint::Centre => EventOption::DISC_CENTER.raw(),
        DiscPoint::UpperLimb => 0,
        DiscPoint::LowerLimb => EventOption::DISC_BOTTOM.raw(),
    };
    if horizon.refraction == Refraction::None {
        options |= EventOption::NO_REFRACTION.raw();
    }
    if horizon.altitude_deg != 0.0 {
        let twilight = match (request.body, horizon.altitude_deg) {
            (Body::Sun, a) if (a + 6.0).abs() < 1e-9 => EventOption::CIVIL_TWILIGHT,
            (Body::Sun, a) if (a + 12.0).abs() < 1e-9 => EventOption::NAUTICAL_TWILIGHT,
            (Body::Sun, a) if (a + 18.0).abs() < 1e-9 => EventOption::ASTRONOMICAL_TWILIGHT,
            _ => {
                return Err(ProviderError::unsupported(format!(
                    "a horizon altitude of {} degrees in a native search",
                    horizon.altitude_deg
                )));
            }
        };
        options |= twilight.raw();
    }
    Ok(options)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "a test fails by panicking")]

    use super::*;

    fn provider() -> TeimerisProvider {
        TeimerisProvider::open(&data_dir_from_env()).unwrap_or_else(|e| panic!("{e}"))
    }

    fn request<'a>(jds: &'a [f64], bodies: &'a [Body]) -> PositionRequest<'a> {
        PositionRequest {
            jds,
            scale: TimeScale::Ut1,
            bodies,
            frame: Frame::CANONICAL,
            observer: None,
            speeds: true,
        }
    }

    #[test]
    fn the_grid_is_transposed_to_instants_outermost() {
        let p = provider();
        let jds = [2_451_545.0, 2_460_000.5, 2_440_000.5];
        let bodies = [Body::Sun, Body::Moon, Body::Mars];
        let grid = p
            .positions(&request(&jds, &bodies))
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(
            grid.all_ok(),
            "{:?}",
            grid.cells().map(|c| c.status).collect::<Vec<_>>()
        );
        for (jd_index, jd) in jds.iter().enumerate() {
            for (body_index, body) in bodies.iter().enumerate() {
                let single = p
                    .positions(&request(
                        core::slice::from_ref(jd),
                        core::slice::from_ref(body),
                    ))
                    .unwrap_or_else(|e| panic!("{e}"));
                let a = grid
                    .at(jd_index, body_index)
                    .unwrap_or_else(|| panic!("cell"));
                let b = single.cell(0).unwrap_or_else(|| panic!("cell"));
                assert_eq!(a.lon.to_bits(), b.lon.to_bits(), "{} at {jd}", body.key());
                assert_eq!(
                    a.lon_speed.to_bits(),
                    b.lon_speed.to_bits(),
                    "{} speed at {jd}",
                    body.key()
                );
            }
        }
    }

    #[test]
    fn the_sun_is_at_the_vernal_point_at_the_march_equinox_of_2000() {
        let p = provider();
        let columns = p
            .positions(&request(&[2_451_623.815], &[Body::Sun]))
            .unwrap_or_else(|e| panic!("{e}"));
        let sun = columns.cell(0).unwrap_or_else(|| panic!("cell"));
        assert!(sun.lon < 0.02 || sun.lon > 359.98, "{}", sun.lon);
        assert_eq!(sun.source.kind, EphemerisKind::Files);
    }

    #[test]
    fn an_instant_outside_the_files_is_out_of_range_not_an_analytic_answer() {
        let p = provider();
        let (lo, _) = p.capabilities.jd_range;
        let columns = p
            .positions(&request(&[lo - 1000.0], &[Body::Sun]))
            .unwrap_or_else(|e| panic!("{e}"));
        let sun = columns.cell(0).unwrap_or_else(|| panic!("cell"));
        assert_eq!(sun.status, CellStatus::OutOfRange);
    }
}
