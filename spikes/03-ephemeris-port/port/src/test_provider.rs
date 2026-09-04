//! The spike-2 analytic provider behind this port: fixed tables, no
//! astronomy, the zero-setup provider unit tests and the kit itself run
//! against. It reuses `teistro-spike-slice`'s `TestProvider` rather than
//! copying it.

use teistro_spike_slice::{Body as SliceBody, EphemerisPort, TestProvider};

use crate::model::{
    Body, Capabilities, Cell, CellStatus, EphemerisKind, Frame, Identity, Overrides,
    PositionColumns, PositionRequest, ProviderError, Source,
};
use crate::provider::{EphemerisProvider, validate};

/// The analytic test provider of spike 2, declared honestly: apparent
/// geocentric ecliptic of date, tropical, deterministic, no overrides, a
/// distance of one AU for every body because the slice carries none.
#[derive(Clone, Copy, Debug, Default)]
pub struct SliceTestProvider {
    inner: TestProvider,
}

impl SliceTestProvider {
    /// The provider.
    #[must_use]
    pub fn new() -> SliceTestProvider {
        SliceTestProvider {
            inner: TestProvider,
        }
    }

    /// Coverage: year 0 to year 3000, generous because the model is a
    /// polynomial.
    pub const JD_RANGE: (f64, f64) = (1_721_057.5, 2_816_787.5);

    /// The bodies the slice knows, in port terms.
    pub const BODIES: [Body; 8] = [
        Body::Sun,
        Body::Moon,
        Body::Mars,
        Body::Mercury,
        Body::Jupiter,
        Body::Venus,
        Body::Saturn,
        Body::MeanNode,
    ];

    fn slice_body(body: Body) -> Option<SliceBody> {
        match body {
            Body::Sun => Some(SliceBody::Sun),
            Body::Moon => Some(SliceBody::Moon),
            Body::Mars => Some(SliceBody::Mars),
            Body::Mercury => Some(SliceBody::Mercury),
            Body::Jupiter => Some(SliceBody::Jupiter),
            Body::Venus => Some(SliceBody::Venus),
            Body::Saturn => Some(SliceBody::Saturn),
            Body::MeanNode => Some(SliceBody::Rahu),
            _ => None,
        }
    }
}

impl EphemerisProvider for SliceTestProvider {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            identity: Identity {
                name: String::from("test-provider"),
                version: String::from("spike-2"),
                data_version: String::from("analytic"),
                tier: None,
                data_hashes: Vec::new(),
            },
            jd_range: SliceTestProvider::JD_RANGE,
            bodies: SliceTestProvider::BODIES.to_vec(),
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
                let Some(index) = columns.index(jd_index, body_index) else {
                    continue;
                };
                let cell = if capabilities.covers(*jd) {
                    match SliceTestProvider::slice_body(*body).map(|b| self.inner.position(*jd, b))
                    {
                        Some(Ok(p)) => Cell {
                            lon: p.longitude_deg,
                            lat: p.latitude_deg,
                            dist: 1.0,
                            lon_speed: if request.speeds {
                                p.speed_deg_per_day
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
                        },
                        Some(Err(code)) => Cell {
                            status: CellStatus::Provider(code),
                            ..Cell::EMPTY
                        },
                        None => Cell {
                            status: CellStatus::UnsupportedBody,
                            ..Cell::EMPTY
                        },
                    }
                } else {
                    Cell {
                        status: CellStatus::OutOfRange,
                        ..Cell::EMPTY
                    }
                };
                columns.set(index, cell);
            }
        }
        Ok(columns)
    }
}
