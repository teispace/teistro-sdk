//! The analytic test provider: fixed elements, no astronomy, the
//! zero-setup provider the SDK's unit tests and the conformance kit run
//! against with no engine present. Declared honestly: the canonical frame,
//! deterministic, no overrides, a distance of one astronomical unit for
//! every body because the elements carry none.

use teistro_core::angle::normalise_deg;

use crate::body::Body;
use crate::capabilities::{Capabilities, Identity, Overrides};
use crate::columns::{Cell, CellStatus, EphemerisKind, PositionColumns, Source};
use crate::error::ProviderError;
use crate::frame::Frame;
use crate::provider::{EphemerisProvider, PositionRequest, validate};

/// Degrees to radians.
const DEG2RAD: f64 = core::f64::consts::PI / 180.0;
/// J2000.0.
const J2000: f64 = 2_451_545.0;

/// One body's elements: the longitude at J2000, the mean motion in
/// degrees per day, and the amplitude of one periodic term in degrees.
struct Elements {
    body: Body,
    longitude_at_j2000: f64,
    rate: f64,
    amplitude: f64,
}

/// The analytic test provider.
///
/// ```
/// use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, PositionRequest, TestProvider, TimeScale};
///
/// let provider = TestProvider::new();
/// let request = PositionRequest::new(&[2_451_545.0], TimeScale::Ut1, &[Body::Moon], Frame::CANONICAL);
/// let moon = provider.positions(&request).expect("the canonical frame").at(0, 0).expect("a cell");
/// assert!(moon.is_ok() && moon.lon_speed > 10.0);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct TestProvider;

impl TestProvider {
    /// Coverage: year 0 to year 3000, generous because the model is a
    /// polynomial.
    pub const JD_RANGE: (f64, f64) = (1_721_057.5, 2_816_787.5);

    /// The bodies the elements cover, in port order.
    pub const BODIES: [Body; 8] = [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::MeanNode,
    ];

    /// The elements: mean longitudes and motions of the order of the real
    /// ones, with one periodic term each, so the grid looks like a sky
    /// without being one.
    const ELEMENTS: [Elements; 8] = [
        Elements {
            body: Body::Sun,
            longitude_at_j2000: 280.46,
            rate: 0.985_647_4,
            amplitude: 1.915,
        },
        Elements {
            body: Body::Moon,
            longitude_at_j2000: 218.32,
            rate: 13.176_396,
            amplitude: 6.289,
        },
        Elements {
            body: Body::Mercury,
            longitude_at_j2000: 252.25,
            rate: 4.092_339,
            amplitude: 23.44,
        },
        Elements {
            body: Body::Venus,
            longitude_at_j2000: 181.98,
            rate: 1.602_130,
            amplitude: 0.77,
        },
        Elements {
            body: Body::Mars,
            longitude_at_j2000: 355.45,
            rate: 0.524_033,
            amplitude: 10.69,
        },
        Elements {
            body: Body::Jupiter,
            longitude_at_j2000: 34.35,
            rate: 0.083_129,
            amplitude: 5.55,
        },
        Elements {
            body: Body::Saturn,
            longitude_at_j2000: 50.08,
            rate: 0.033_444,
            amplitude: 6.41,
        },
        Elements {
            body: Body::MeanNode,
            longitude_at_j2000: 125.04,
            rate: -0.052_954,
            amplitude: 0.0,
        },
    ];

    /// The provider.
    #[must_use]
    pub const fn new() -> TestProvider {
        TestProvider
    }

    /// The position of a body: the mean longitude plus one sine term, a
    /// small latitude, the analytic speed.
    fn cell(elements: &Elements, jd: f64, speeds: bool) -> Cell {
        let t = jd - J2000;
        let mean = elements.longitude_at_j2000 + elements.rate * t;
        let (sin_mean, cos_mean) = (mean * DEG2RAD).sin_cos();
        Cell {
            lon: normalise_deg(mean + elements.amplitude * sin_mean),
            lat: 0.05 * elements.amplitude * (mean * DEG2RAD * 0.5).cos(),
            dist: 1.0,
            lon_speed: if speeds {
                elements.rate + elements.amplitude * elements.rate * DEG2RAD * cos_mean
            } else {
                0.0
            },
            lat_speed: 0.0,
            dist_speed: 0.0,
            status: CellStatus::Ok,
            source: Source {
                kind: EphemerisKind::Test,
                tier: None,
            },
        }
    }
}

impl EphemerisProvider for TestProvider {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            identity: Identity {
                name: String::from("test-provider"),
                version: String::from("1"),
                data_version: String::from("analytic"),
                tier: None,
                data_hashes: Vec::new(),
            },
            jd_range: TestProvider::JD_RANGE,
            bodies: TestProvider::BODIES.to_vec(),
            native_frame: Frame::CANONICAL,
            speeds: true,
            overrides: Overrides::NONE,
            ayanamshas: Vec::new(),
            deterministic: true,
        }
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        let capabilities = self.capabilities();
        validate(&capabilities, request)?;
        if request.frame != Frame::CANONICAL {
            return Err(ProviderError::unsupported(
                "a frame other than the canonical one",
            ));
        }
        let mut columns =
            PositionColumns::new(request.jds.len(), request.bodies.len(), Frame::CANONICAL);
        for (jd_index, jd) in request.jds.iter().enumerate() {
            for (body_index, body) in request.bodies.iter().enumerate() {
                let cell = if capabilities.covers(*jd) {
                    match TestProvider::ELEMENTS.iter().find(|e| e.body == *body) {
                        Some(elements) => TestProvider::cell(elements, *jd, request.speeds),
                        None => Cell::failed(CellStatus::UnsupportedBody),
                    }
                } else {
                    Cell::failed(CellStatus::OutOfRange)
                };
                columns.set_at(jd_index, body_index, cell);
            }
        }
        Ok(columns)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use crate::body::TimeScale;

    #[test]
    fn the_test_provider_is_deterministic_finite_and_honest() {
        let provider = TestProvider::new();
        let jds = [2_460_000.5, 2_460_000.5 + 1e-3];
        let request = PositionRequest::new(
            &jds,
            TimeScale::Ut1,
            &TestProvider::BODIES,
            Frame::CANONICAL,
        );
        let a = provider.positions(&request).unwrap();
        let b = provider.positions(&request).unwrap();
        assert!(a.all_ok() && a.bit_identical(&b));
        for cell in a.cells() {
            assert!(cell.lon.is_finite() && (0.0..360.0).contains(&cell.lon));
        }
        // The speed agrees with a difference over a thousandth of a day.
        let moon_now = a.at(0, 1).unwrap();
        let moon_then = a.at(1, 1).unwrap();
        let rate = teistro_core::angle::difference_deg(moon_then.lon, moon_now.lon) / 1e-3;
        assert!(
            (rate - moon_now.lon_speed).abs() < 1e-3,
            "{rate} {}",
            moon_now.lon_speed
        );
        let outside = [TestProvider::JD_RANGE.0 - 1.0];
        let request =
            PositionRequest::new(&outside, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
        assert_eq!(
            provider
                .positions(&request)
                .unwrap()
                .at(0, 0)
                .map(|c| c.status),
            Some(CellStatus::OutOfRange)
        );
        let equatorial = PositionRequest::new(
            &jds,
            TimeScale::Ut1,
            &[Body::Sun],
            Frame::CANONICAL.with_coordinates(crate::frame::Coordinates::Equatorial),
        );
        assert!(matches!(
            provider.positions(&equatorial),
            Err(ProviderError::Unsupported { .. })
        ));
        assert!(provider.positions(&request.without_speeds()).is_ok());
    }
}
