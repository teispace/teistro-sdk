//! An ephemeris provider written in JavaScript, bound into the port's
//! vtable (`docs/02-architecture/07-binding-architecture.md`, "Ports
//! across the boundary").
//!
//! HAND-WRITTEN, and one of only two files in this crate that is. The
//! architecture puts the port adapter in the ergonomic layer because
//! every binding wraps its own callback mechanism: napi here, an isolate
//! callback in Dart, the GIL in Python. What the adapter does is small,
//! because the port already carries the machinery: a Rust
//! [`EphemerisProvider`] becomes a vtable through
//! [`teistro_port_ephemeris::Exported`], which is tested to round-trip
//! bit for bit, so this file only has to be that provider.
//!
//! The call is synchronous and on the thread that entered the SDK, which
//! is the boundary's documented contract (one context, one thread at a
//! time). The environment is lent to the provider for the length of a
//! call and taken back after it, so a callback that escaped its call
//! finds nothing to call into rather than a stale handle.

#![allow(
    unsafe_code,
    reason = "the port's trait is `Send + Sync`; the contract is argued at each impl"
)]

use std::cell::{Cell, RefCell};
use std::ffi::c_void;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use teistro_core::catalogue::Ayanamsha;
use teistro_port_ephemeris::{
    Astronomy, Body, Capabilities, Cell as PortCell, CellStatus, DistanceUnit, EphemerisKind,
    EphemerisProvider, Exported, Frame, Identity, Overrides, PositionColumns, PositionRequest,
    ProviderError, ProviderVtable, Source, SpeedModel, validate,
};

use crate::generated::{Observer, PositionColumns as JsColumns, PositionRequest as JsRequest};

/// What a JavaScript provider says about itself. Everything but the name
/// and the bodies has a default, because a provider that answers the
/// canonical frame with apparent geocentric positions is the common case
/// and should not have to say so.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct ProviderInfo {
    /// What the provider is, stamped in every result's provenance.
    pub name: String,
    /// The bodies it answers, by their catalogue keys.
    pub bodies: Vec<String>,
    /// Its version; empty by default.
    pub version: Option<String>,
    /// What identifies its data (an ephemeris file's edition); empty by
    /// default.
    pub data_version: Option<String>,
    /// The first Julian day it covers; year 0 by default.
    pub jd_min: Option<f64>,
    /// The last Julian day it covers; year 3000 by default.
    pub jd_max: Option<f64>,
    /// The frame it returns natively, packed; the canonical frame by
    /// default (`packFrame` builds one).
    pub native_frame_bits: Option<u32>,
    /// Whether it computes speeds; `true` by default.
    pub speeds: Option<bool>,
    /// Whether identical requests give identical bits; `true` by default,
    /// and a provider that is not deterministic must say so, because the
    /// conformance contract rests on it (ADR-0022).
    pub deterministic: Option<bool>,
}

/// The provider as the SDK sees it: the description read once, and the
/// callback the SDK reaches through.
struct JsProvider {
    capabilities: Capabilities,
    positions: FunctionRef<FnArgs<(JsRequest,)>, Option<JsColumns>>,
    /// The environment, lent for the length of one call.
    env: Cell<Option<Env>>,
    /// What the callback threw, kept for the layer above to rethrow.
    thrown: RefCell<Option<String>>,
}

// SAFETY: a context is used by one thread at a time, which is the
// boundary's documented contract, and the callback is only ever reached
// from inside a call this addon made on that thread. The port's trait
// requires `Send + Sync` because a context may be moved between threads;
// nothing here is touched while it is in flight.
unsafe impl Send for JsProvider {}
// SAFETY: as above.
unsafe impl Sync for JsProvider {}

impl JsProvider {
    /// The error a call reports when the environment is not lent, which
    /// means the callback outlived the call that borrowed it.
    fn no_environment() -> ProviderError {
        ProviderError::Refused {
            detail: String::from(
                "the provider was called outside a call into the SDK; a context and its provider belong to one thread",
            ),
        }
    }
}

impl EphemerisProvider for JsProvider {
    fn capabilities(&self) -> Capabilities {
        self.capabilities.clone()
    }

    fn positions(
        &self,
        request: &PositionRequest<'_>,
    ) -> core::result::Result<PositionColumns, ProviderError> {
        // The port's own check runs first, so a provider is never asked
        // for a body, an instant or a frame it did not declare, and the
        // refusal names what is missing rather than leaving a callback to
        // discover it.
        validate(&self.capabilities, request)?;
        let Some(env) = self.env.get() else {
            return Err(JsProvider::no_environment());
        };
        let asked = JsRequest {
            scale: crate::generated::time_scale_to_str(request.scale.id()),
            frame_bits: request.frame.to_bits(),
            speeds: request.speeds,
            observer: request.observer.map(|place| Observer {
                longitude_deg: place.longitude.get(),
                latitude_deg: place.latitude.get(),
                altitude_m: place.altitude.get(),
            }),
            jds: request.jds.to_vec(),
            bodies: request
                .bodies
                .iter()
                .map(|body| crate::generated::body_to_str(body.id()))
                .collect(),
        };
        let answered = self
            .positions
            .borrow_back(&env)
            .and_then(|callback| callback.call(FnArgs::from((asked,))));
        match answered {
            // A provider that cannot produce the frame asked for says so
            // by answering with nothing; the SDK then asks for its native
            // frame and completes the rest itself.
            Ok(None) => Err(ProviderError::unsupported(format!(
                "the {} frame",
                request.frame.key()
            ))),
            Ok(Some(columns)) => match columns_of(&columns, request) {
                Ok(columns) => Ok(columns),
                Err(sentence) => Err(self.refused(sentence)),
            },
            Err(error) => Err(self.refused(format!(
                "threw: {}",
                error.reason.trim_start_matches("Error: ")
            ))),
        }
    }
}

impl JsProvider {
    /// Records what went wrong and refuses the call. Only a code crosses
    /// the C boundary, and a provider written in JavaScript has more to
    /// say than a code, so the sentence is kept for the layer above.
    fn refused(&self, sentence: String) -> ProviderError {
        *self.thrown.borrow_mut() = Some(sentence.clone());
        ProviderError::Refused { detail: sentence }
    }
}

/// The columns a JavaScript provider returned, read into the port's own
/// shape. A column of the wrong length is refused by name rather than
/// read past its end.
fn columns_of(
    answered: &JsColumns,
    request: &PositionRequest<'_>,
) -> core::result::Result<PositionColumns, String> {
    let cells = request.cell_count();
    let frame = Frame::try_from_bits(answered.frame_bits)
        .map_err(|error| format!("answered in a frame that is not one: {error}"))?;
    let check = |name: &str, len: usize| -> core::result::Result<(), String> {
        if len == cells {
            Ok(())
        } else {
            Err(format!(
                "returned {len} values in `{name}` for {cells} cells"
            ))
        }
    };
    for (name, len) in [
        ("lon", answered.lon.len()),
        ("lat", answered.lat.len()),
        ("dist", answered.dist.len()),
        ("status", answered.status.len()),
    ] {
        check(name, len)?;
    }
    let mut columns = PositionColumns::new(request.jds.len(), request.bodies.len(), frame);
    let speed = |column: &Vec<f64>, index: usize| column.get(index).copied().unwrap_or(0.0);
    for index in 0..cells {
        let status = answered
            .status
            .get(index)
            .map_or(CellStatus::Ok, |code| CellStatus::from_code(*code));
        let source = answered.source.get(index).map_or(
            Source {
                kind: EphemerisKind::Unknown,
                tier: None,
            },
            |bits| Source::from_bits(*bits),
        );
        columns.set(
            index,
            PortCell {
                lon: speed(&answered.lon, index),
                lat: speed(&answered.lat, index),
                dist: speed(&answered.dist, index),
                lon_speed: speed(&answered.lon_speed, index),
                lat_speed: speed(&answered.lat_speed, index),
                dist_speed: speed(&answered.dist_speed, index),
                status,
                source,
            },
        );
    }
    Ok(columns)
}

/// A bound JavaScript provider: the boxed provider the vtable points at,
/// kept alive by the context that owns it.
pub(crate) struct Host {
    exported: Box<Exported<JsProvider>>,
}

impl core::fmt::Debug for Host {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Host").finish_non_exhaustive()
    }
}

impl Host {
    /// Binds a provider's description and its callback.
    ///
    /// # Errors
    ///
    /// A body key the catalogue does not have, frame bits no frame sets,
    /// or a callback the environment cannot hold a reference to.
    pub(crate) fn bind(
        info: ProviderInfo,
        positions: &Function<'_, FnArgs<(JsRequest,)>, Option<JsColumns>>,
    ) -> Result<Host> {
        let bodies = info
            .bodies
            .iter()
            .map(|key| {
                Body::from_key(key.rsplit('.').next().unwrap_or(key)).ok_or_else(|| {
                    Error::from_reason(format!("`{key}` is not a body the port knows"))
                })
            })
            .collect::<Result<Vec<Body>>>()?;
        if bodies.is_empty() {
            return Err(Error::from_reason(
                "a provider must answer at least one body",
            ));
        }
        let native_frame = match info.native_frame_bits {
            Some(bits) => {
                Frame::try_from_bits(bits).map_err(|error| Error::from_reason(error.to_string()))?
            }
            None => Frame::CANONICAL,
        };
        let capabilities = Capabilities {
            identity: Identity {
                name: info.name,
                version: info.version.unwrap_or_default(),
                data_version: info.data_version.unwrap_or_default(),
                tier: None,
                data_hashes: Vec::new(),
            },
            jd_range: (
                info.jd_min.unwrap_or(1_721_057.5),
                info.jd_max.unwrap_or(2_816_787.5),
            ),
            bodies,
            native_frame,
            astronomy: Astronomy::Modern,
            speeds: info.speeds.unwrap_or(true),
            speed_model: SpeedModel::Derivative,
            distance_unit: DistanceUnit::AstronomicalUnits,
            overrides: Overrides::NONE,
            ayanamshas: Vec::<Ayanamsha>::new(),
            deterministic: info.deterministic.unwrap_or(true),
        };
        Ok(Host {
            exported: Exported::new(JsProvider {
                capabilities,
                positions: positions.create_ref()?,
                env: Cell::new(None),
                thrown: RefCell::new(None),
            }),
        })
    }

    /// The vtable the SDK drives the provider through.
    #[must_use]
    pub(crate) fn vtable() -> ProviderVtable {
        Exported::<JsProvider>::vtable()
    }

    /// The pointer the SDK hands back to every callback.
    #[must_use]
    pub(crate) fn user_data(&self) -> *mut c_void {
        self.exported.user_data()
    }

    /// Lends the environment for one call into the SDK.
    pub(crate) fn enter(&self, env: Env) {
        let provider = self.exported.provider();
        provider.env.set(Some(env));
        provider.thrown.borrow_mut().take();
    }

    /// Takes the environment back, and reports what the callback threw.
    ///
    /// # Errors
    ///
    /// The message the callback threw, so a failure inside a provider
    /// reaches the caller as its own error rather than as a status code.
    pub(crate) fn leave(&self) -> Result<()> {
        let provider = self.exported.provider();
        provider.env.set(None);
        match provider.thrown.borrow_mut().take() {
            Some(sentence) => Err(Error::from_reason(format!(
                "the ephemeris provider {sentence}"
            ))),
            None => Ok(()),
        }
    }
}

/// The vtable pointer and the user data of an optional host, as the
/// generated constructor passes them.
#[must_use]
pub(crate) fn parts(host: Option<&Host>) -> (Option<ProviderVtable>, *mut c_void) {
    match host {
        Some(host) => (Some(Host::vtable()), host.user_data()),
        None => (None, core::ptr::null_mut()),
    }
}
