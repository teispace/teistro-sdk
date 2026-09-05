//! The Swiss Ephemeris adapter: the ephemeris port over the C library,
//! compiled from sources named at build time and linked under the
//! library's own terms, outside the SDK workspace (ADR-0019). This crate's
//! own source is Apache-2.0; a binary that links the library is
//! distributed under the library's licence, which is the reason for the
//! containment.
//!
//! The library keeps its state in process-wide globals: the ephemeris
//! path, the topocentric observer, the sidereal mode. The adapter is the
//! one gateway to them. Every request takes the process-wide lock, sets
//! the state it needs, computes and releases, so two threads interleaving
//! sidereal modes and observers get the same bits as one thread in
//! sequence (tested). The path is set once; a second adapter over a
//! different directory is refused rather than silently repointing the
//! first.
//!
//! The library falls back to its analytic model when a file is missing and
//! says so only in the flags it returns; the adapter reads them and reports
//! the cell as missing data rather than passing the fallback off as a file
//! position. An instant outside the files' coverage is reported as out of
//! range without a call.

mod ffi;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::Place;
use teistro_port_ephemeris::{
    Astronomy, Body, Capabilities, Cell, CellStatus, Centre, Coordinates, DistanceUnit,
    EphemerisKind, EphemerisProvider, Equinox, Frame, Identity, Obliquity, Overrides,
    PositionColumns, PositionRequest, ProviderError, Source, SpeedModel, TimeScale, Zodiac, sefile,
    validate,
};

/// The environment variable naming the data directory.
pub const DATA_DIR_ENV: &str = "SWEPH_DATA_DIR";

/// The cell status and error code for a call the library answered with its
/// generic failure (`ERR`), whose message the cell cannot carry.
pub const CALC_FAILED: i32 = -101;

/// The library's process-wide state the adapter has set.
struct Globals {
    path: Option<PathBuf>,
}

/// The one lock over the library's globals.
static GATE: Mutex<Globals> = Mutex::new(Globals { path: None });

/// One turn at the library. Holding it is holding the process-wide lock,
/// and every call that reads or writes the library's globals is a method
/// on it, so the receiver is the proof the lock is held.
struct Session<'a> {
    globals: MutexGuard<'a, Globals>,
}

/// A `swe_calc` answer: the six values, the flags actually used (or a
/// negative code), and the message.
struct Answer {
    xx: [f64; 6],
    retflag: i32,
    message: String,
}

fn message(buffer: &[u8; ffi::MESSAGE_LEN]) -> String {
    CStr::from_bytes_until_nul(buffer)
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[allow(
    clippy::unused_self,
    reason = "the receiver is the proof that the process-wide lock is held"
)]
impl Session<'_> {
    fn take() -> Session<'static> {
        Session {
            globals: GATE.lock().unwrap_or_else(PoisonError::into_inner),
        }
    }

    /// Points the library at `dir`, once per process.
    #[allow(unsafe_code, reason = "a C entry point; see the safety comment")]
    fn set_path(&mut self, dir: &Path) -> Result<(), ProviderError> {
        if let Some(current) = &self.globals.path {
            return if current == dir {
                Ok(())
            } else {
                Err(ProviderError::Refused {
                    detail: format!(
                        "the library's ephemeris path is process-wide and already {}",
                        current.display()
                    ),
                })
            };
        }
        let path =
            CString::new(dir.to_string_lossy().as_bytes()).map_err(|_| ProviderError::Invalid {
                detail: format!("{} holds a NUL byte", dir.display()),
            })?;
        // SAFETY: `path` is a valid NUL-terminated string that outlives the
        // call, and the library copies it into its own state; the lock is
        // held, so no other call reads that state meanwhile.
        unsafe { ffi::swe_set_ephe_path(path.as_ptr()) };
        self.globals.path = Some(dir.to_path_buf());
        Ok(())
    }

    #[allow(unsafe_code, reason = "a C entry point; see the safety comment")]
    fn version(&self) -> String {
        let mut buffer = [0u8; ffi::MESSAGE_LEN];
        // SAFETY: the library writes a short NUL-terminated string into a
        // buffer of the length it documents.
        unsafe { ffi::swe_version(buffer.as_mut_ptr().cast::<c_char>()) };
        message(&buffer)
    }

    #[allow(unsafe_code, reason = "a C entry point; see the safety comment")]
    fn calc(&self, jd: f64, scale: TimeScale, body: c_int, flags: i32) -> Answer {
        let mut xx = [0.0f64; 6];
        let mut buffer = [0u8; ffi::MESSAGE_LEN];
        let serr = buffer.as_mut_ptr().cast::<c_char>();
        // SAFETY: `xx` has the six slots the library writes and `serr` the
        // documented message length; both outlive the call; the lock is
        // held, so the globals the call reads are the ones this request set.
        let retflag = unsafe {
            match scale {
                TimeScale::Ut1 => ffi::swe_calc_ut(jd, body, flags, xx.as_mut_ptr(), serr),
                TimeScale::Tt => ffi::swe_calc(jd, body, flags, xx.as_mut_ptr(), serr),
            }
        };
        Answer {
            xx,
            retflag,
            message: message(&buffer),
        }
    }

    #[allow(unsafe_code, reason = "a C entry point; see the safety comment")]
    fn set_topo(&self, observer: Place) {
        // SAFETY: three scalars into process-wide state, under the lock.
        unsafe {
            ffi::swe_set_topo(
                observer.longitude.get(),
                observer.latitude.get(),
                observer.altitude.get(),
            );
        }
    }

    #[allow(unsafe_code, reason = "a C entry point; see the safety comment")]
    fn set_sid_mode(&self, ayanamsha: Ayanamsha) {
        // SAFETY: three scalars into process-wide state, under the lock. The
        // library's mode number is the catalogue's attribute.
        unsafe { ffi::swe_set_sid_mode(i32::from(ayanamsha.attributes().swiss_mode), 0.0, 0.0) };
    }

    #[allow(unsafe_code, reason = "a C entry point; see the safety comment")]
    fn ayanamsha(&self, jd: f64, scale: TimeScale, flags: i32) -> Result<f64, String> {
        let mut value = 0.0f64;
        let mut buffer = [0u8; ffi::MESSAGE_LEN];
        let serr = buffer.as_mut_ptr().cast::<c_char>();
        // SAFETY: one output slot and a message buffer of the documented
        // length, both outliving the call, under the lock.
        let retflag = unsafe {
            match scale {
                TimeScale::Ut1 => ffi::swe_get_ayanamsa_ex_ut(jd, flags, &raw mut value, serr),
                TimeScale::Tt => ffi::swe_get_ayanamsa_ex(jd, flags, &raw mut value, serr),
            }
        };
        if retflag < 0 {
            Err(message(&buffer))
        } else {
            Ok(value)
        }
    }

    /// Delta T in seconds; the library answers in days.
    #[allow(unsafe_code, reason = "a C entry point; see the safety comment")]
    fn delta_t_seconds(&self, jd_ut1: f64) -> f64 {
        let mut buffer = [0u8; ffi::MESSAGE_LEN];
        // SAFETY: a message buffer of the documented length, under the lock.
        let days =
            unsafe { ffi::swe_deltat_ex(jd_ut1, ffi::flag::SWIEPH, buffer.as_mut_ptr().cast()) };
        days * 86_400.0
    }
}

/// The library's body number for one of the port's; `None` for a body added
/// to the port after this adapter, which the capabilities then do not list.
const fn map_body(body: Body) -> Option<c_int> {
    Some(match body {
        Body::Sun => 0,
        Body::Moon => 1,
        Body::Mercury => 2,
        Body::Venus => 3,
        Body::Mars => 4,
        Body::Jupiter => 5,
        Body::Saturn => 6,
        Body::Uranus => 7,
        Body::Neptune => 8,
        Body::Pluto => 9,
        Body::MeanNode => 10,
        Body::TrueNode => 11,
        Body::MeanApogee => 12,
        Body::OsculatingApogee => 13,
        _ => return None,
    })
}

/// The library's body numbers for a request's bodies, in the same order.
fn library_bodies(bodies: &[Body]) -> Result<Vec<c_int>, ProviderError> {
    bodies
        .iter()
        .map(|body| map_body(*body).ok_or_else(|| ProviderError::unsupported(body.key())))
        .collect()
}

/// The library's flags for a frame: every switch explicit, the file
/// ephemeris always named so the library never picks one for us.
fn map_flags(frame: Frame, speeds: bool) -> i32 {
    let switches = [
        (speeds, ffi::flag::SPEED),
        (
            frame.coordinates == Coordinates::Equatorial,
            ffi::flag::EQUATORIAL,
        ),
        (frame.equinox == Equinox::J2000, ffi::flag::J2000),
        (!frame.corrections.nutation, ffi::flag::NONUT),
        (!frame.corrections.light_time, ffi::flag::TRUEPOS),
        (!frame.corrections.aberration, ffi::flag::NOABERR),
        (!frame.corrections.deflection, ffi::flag::NOGDEFL),
        (frame.centre == Centre::Topocentric, ffi::flag::TOPOCTR),
        (frame.centre == Centre::Heliocentric, ffi::flag::HELCTR),
        (frame.centre == Centre::Barycentric, ffi::flag::BARYCTR),
        (
            matches!(frame.zodiac, Zodiac::Sidereal { .. }),
            ffi::flag::SIDEREAL,
        ),
    ];
    switches
        .into_iter()
        .filter(|(on, _)| *on)
        .fold(ffi::flag::SWIEPH, |flags, (_, flag)| flags | flag)
}

/// Which ephemeris answered, from the flags the library returns.
const fn map_source(retflag: i32) -> Source {
    let kind = if retflag & ffi::flag::JPLEPH != 0 {
        EphemerisKind::Jpl
    } else if retflag & ffi::flag::MOSEPH != 0 {
        EphemerisKind::Analytic
    } else if retflag & ffi::flag::SWIEPH != 0 {
        EphemerisKind::Files
    } else {
        EphemerisKind::Unknown
    };
    Source { kind, tier: None }
}

/// A position answer as a cell: a failure is the generic code, an analytic
/// fallback is missing data, anything else is the values.
fn map_cell(answer: &Answer) -> Cell {
    if answer.retflag < 0 {
        return Cell::failed(CellStatus::Provider { code: CALC_FAILED });
    }
    let source = map_source(answer.retflag);
    if source.kind == EphemerisKind::Analytic {
        return Cell::failed(CellStatus::DataMissing);
    }
    let [lon, lat, dist, lon_speed, lat_speed, dist_speed] = answer.xx;
    Cell {
        lon,
        lat,
        dist,
        lon_speed,
        lat_speed,
        dist_speed,
        status: CellStatus::Ok,
        source,
    }
}

fn failed(answer: &Answer) -> ProviderError {
    ProviderError::Provider {
        code: CALC_FAILED,
        detail: answer.message.clone(),
    }
}

/// The data directory: [`DATA_DIR_ENV`], else the Teimeris checkout's data
/// directory beside the SDK checkout, which holds the same file family.
#[must_use]
pub fn data_dir_from_env() -> PathBuf {
    std::env::var_os(DATA_DIR_ENV).map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../teimeris/data"),
        PathBuf::from,
    )
}

/// The port over the process-wide library.
#[derive(Debug)]
pub struct SwephProvider {
    capabilities: Capabilities,
}

impl SwephProvider {
    /// Points the library at a data directory of `.se1` files and hashes
    /// the files it will read.
    ///
    /// # Errors
    ///
    /// When the directory holds no planet block, or the library is already
    /// pointed at a different directory in this process.
    pub fn open(data_dir: &Path) -> Result<SwephProvider, ProviderError> {
        let files = sefile::scan(data_dir).map_err(|error| ProviderError::DataMissing {
            detail: format!("{}: {error}", data_dir.display()),
        })?;
        let Some(jd_range) = files.jd_range else {
            return Err(ProviderError::DataMissing {
                detail: format!("no planet block (sepl_*.se1) in {}", data_dir.display()),
            });
        };
        let ayanamshas = files.ayanamshas();
        let mut session = Session::take();
        session.set_path(data_dir)?;
        let version = session.version();
        drop(session);
        Ok(SwephProvider {
            capabilities: Capabilities {
                identity: Identity {
                    name: String::from("sweph"),
                    version,
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
                    .with(Overrides::TOPOCENTRIC),
                ayanamshas,
                deterministic: true,
            },
        })
    }

    /// The library's own call per cell in the canonical frame, under one
    /// hold of the lock: the benchmark's direct-binding row, the cost the
    /// port is measured against.
    ///
    /// # Errors
    ///
    /// A body the library does not number.
    pub fn direct_grid(
        &self,
        jds: &[f64],
        bodies: &[Body],
    ) -> Result<Vec<[f64; 6]>, ProviderError> {
        let numbers = library_bodies(bodies)?;
        let session = Session::take();
        let flags = map_flags(Frame::CANONICAL, true);
        Ok(jds
            .iter()
            .flat_map(|jd| numbers.iter().map(move |body| (*jd, *body)))
            .map(|(jd, body)| session.calc(jd, TimeScale::Ut1, body, flags).xx)
            .collect())
    }
}

impl EphemerisProvider for SwephProvider {
    fn capabilities(&self) -> Capabilities {
        self.capabilities.clone()
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        validate(&self.capabilities, request)?;
        let numbers = library_bodies(request.bodies)?;
        let flags = map_flags(request.frame, request.speeds);
        let mut columns =
            PositionColumns::new(request.jds.len(), request.bodies.len(), request.frame);
        let session = Session::take();
        if let Some(observer) = request.observer {
            session.set_topo(observer);
        }
        if let Zodiac::Sidereal { ayanamsha } = request.frame.zodiac {
            session.set_sid_mode(ayanamsha);
        }
        for (jd_index, jd) in request.jds.iter().enumerate() {
            for (body_index, body) in numbers.iter().enumerate() {
                let Some(index) = columns.index(jd_index, body_index) else {
                    continue;
                };
                let cell = if self.capabilities.covers(*jd) {
                    map_cell(&session.calc(*jd, request.scale, *body, flags))
                } else {
                    Cell::failed(CellStatus::OutOfRange)
                };
                columns.set(index, cell);
            }
        }
        Ok(columns)
    }

    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        let answer = Session::take().calc(jd, scale, ffi::ECL_NUT, ffi::flag::SWIEPH);
        if answer.retflag < 0 {
            return Err(failed(&answer));
        }
        let [true_deg, mean_deg, nutation_lon_deg, nutation_obl_deg, ..] = answer.xx;
        Ok(Obliquity {
            mean_deg,
            true_deg,
            nutation_lon_deg,
            nutation_obl_deg,
        })
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        Ok(Session::take().delta_t_seconds(jd_ut1))
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
        let session = Session::take();
        session.set_sid_mode(ayanamsha);
        // The mean ayanamsha, the value sidereal longitudes subtract; without
        // `NONUT` the library adds the nutation in longitude.
        session
            .ayanamsha(jd, scale, ffi::flag::SWIEPH | ffi::flag::NONUT)
            .map_err(|detail| ProviderError::Provider {
                code: CALC_FAILED,
                detail,
            })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "a test fails by panicking")]

    use super::*;

    fn provider() -> SwephProvider {
        SwephProvider::open(&data_dir_from_env()).unwrap_or_else(|e| panic!("{e}"))
    }

    fn request<'a>(
        jds: &'a [f64],
        bodies: &'a [Body],
        frame: Frame,
        observer: Option<Place>,
    ) -> PositionRequest<'a> {
        PositionRequest {
            jds,
            scale: TimeScale::Ut1,
            bodies,
            frame,
            observer,
            speeds: true,
        }
    }

    #[test]
    fn the_sun_is_at_the_vernal_point_at_the_march_equinox_of_2000() {
        let p = provider();
        let columns = p
            .positions(&request(
                &[2_451_623.815],
                &[Body::Sun],
                Frame::CANONICAL,
                None,
            ))
            .unwrap_or_else(|e| panic!("{e}"));
        let sun = columns.cell(0).unwrap_or_else(|| panic!("cell"));
        assert_eq!(sun.status, CellStatus::Ok);
        assert!(sun.lon < 0.02 || sun.lon > 359.98, "{}", sun.lon);
        assert_eq!(sun.source.kind, EphemerisKind::Files);
    }

    #[test]
    fn an_instant_outside_the_files_is_out_of_range_not_an_analytic_answer() {
        let p = provider();
        let (lo, _) = p.capabilities.jd_range;
        let columns = p
            .positions(&request(
                &[lo - 1000.0],
                &[Body::Sun],
                Frame::CANONICAL,
                None,
            ))
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            columns.cell(0).map(|c| c.status),
            Some(CellStatus::OutOfRange)
        );
    }

    #[test]
    fn a_second_directory_is_refused_rather_than_repointing_the_process() {
        let _first = provider();
        let other = std::env::temp_dir();
        let second = SwephProvider::open(&other);
        assert!(
            matches!(
                second,
                Err(ProviderError::DataMissing { .. } | ProviderError::Refused { .. })
            ),
            "{second:?}"
        );
    }

    #[test]
    fn threads_interleaving_sidereal_modes_and_observers_get_the_serial_bits() {
        let p = provider();
        let jds: Vec<f64> = (0..12)
            .map(|i| 2_451_545.0 + f64::from(i) * 100.0)
            .collect();
        let bodies = [Body::Sun, Body::Moon, Body::Mars];
        let kathmandu =
            Place::try_from_degrees(27.7172, 85.324, 1400.0).unwrap_or_else(|e| panic!("{e}"));
        let zurich = Place::try_from_degrees(47.37, 8.55, 400.0).unwrap_or_else(|e| panic!("{e}"));
        let topocentric = Frame::CANONICAL.with_centre(Centre::Topocentric);
        let cases = [
            (Frame::CANONICAL, None),
            (
                Frame::CANONICAL.with_zodiac(Zodiac::sidereal(Ayanamsha::Lahiri)),
                None,
            ),
            (
                Frame::CANONICAL.with_zodiac(Zodiac::sidereal(Ayanamsha::Raman)),
                None,
            ),
            (topocentric, Some(kathmandu)),
            (topocentric, Some(zurich)),
        ];
        let compute = |(frame, observer): (Frame, Option<Place>)| {
            p.positions(&request(&jds, &bodies, frame, observer))
                .unwrap_or_else(|e| panic!("{e}"))
        };
        let serial: Vec<PositionColumns> = cases.iter().copied().map(compute).collect();
        assert!(serial.iter().all(PositionColumns::all_ok));
        assert!(
            !serial
                .first()
                .is_some_and(|a| serial.get(1).is_some_and(|b| a.bit_identical(b))),
            "the sidereal grid must differ from the tropical one"
        );
        std::thread::scope(|scope| {
            for thread in 0..8usize {
                let compute = &compute;
                let serial = &serial;
                let cases = &cases;
                scope.spawn(move || {
                    for round in 0..4usize {
                        for (case, expected) in cases
                            .iter()
                            .zip(serial)
                            .cycle()
                            .skip(thread + round)
                            .take(cases.len())
                        {
                            assert!(
                                compute(*case).bit_identical(expected),
                                "thread {thread} round {round}: {case:?} differs from the serial run"
                            );
                        }
                    }
                });
            }
        });
    }
}
