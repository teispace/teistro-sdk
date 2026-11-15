//! An ephemeris provider written in JavaScript, bound into the port's
//! vtable (`docs/02-architecture/07-binding-architecture.md`, "Ports
//! across the boundary").
//!
//! HAND-WRITTEN, and one of only two files in this crate that is. The
//! architecture puts the port adapter in the ergonomic layer because
//! every binding wraps its own callback mechanism: napi here, an isolate
//! callback in Dart, the GIL in Python. What the adapter does is small,
//! because the port carries the rest: `teistro_port_ephemeris::host` turns
//! the description into capabilities, reads the answer and keeps what the
//! callback threw, for this binding and the wasm one alike, so this file
//! is only napi's way of making the call.
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

use std::cell::Cell;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use teistro_port_ephemeris::host::{self, Answer, Bound, Declared, HostCallback, Unanswered};
use teistro_port_ephemeris::vtable::ConventionsC;
use teistro_port_ephemeris::{PositionRequest, ProviderError, ProviderVtable};

use crate::generated::{Observer, PositionColumns as JsColumns, PositionRequest as JsRequest};

/// What a JavaScript provider says about itself. Everything but the name
/// and the bodies has a default, because a provider that answers the
/// canonical frame with apparent geocentric positions is the common case
/// and should not have to say so (`teistro_port_ephemeris::host::Declared`
/// holds the defaults, for every binding).
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

impl From<ProviderInfo> for Declared {
    fn from(info: ProviderInfo) -> Declared {
        Declared {
            name: info.name,
            bodies: info.bodies,
            version: info.version,
            data_version: info.data_version,
            jd_min: info.jd_min,
            jd_max: info.jd_max,
            native_frame_bits: info.native_frame_bits,
            speeds: info.speeds,
            deterministic: info.deterministic,
        }
    }
}

/// The callback the SDK reaches the provider through, and the environment
/// it is lent for the length of one call.
struct Callback {
    positions: FunctionRef<FnArgs<(JsRequest,)>, Option<JsColumns>>,
    env: Cell<Option<Env>>,
}

// SAFETY: a context is used by one thread at a time, which is the
// boundary's documented contract, and the callback is only ever reached
// from inside a call this addon made on that thread. The port's trait
// requires `Send + Sync` because a context may be moved between threads;
// nothing here is touched while it is in flight.
unsafe impl Send for Callback {}
// SAFETY: as above.
unsafe impl Sync for Callback {}

impl HostCallback for Callback {
    fn positions(
        &self,
        request: &PositionRequest<'_>,
    ) -> core::result::Result<Option<Answer>, Unanswered> {
        // Not lent means the callback outlived the call that borrowed it.
        let Some(env) = self.env.get() else {
            return Err(Unanswered::Refused(ProviderError::Refused {
                detail: String::from(
                    "the provider was called outside a call into the SDK; a context and its provider belong to one thread",
                ),
            }));
        };
        let answered = self
            .positions
            .borrow_back(&env)
            .and_then(|callback| callback.call(FnArgs::from((asked(request),))));
        match answered {
            Ok(columns) => Ok(columns.map(answer)),
            Err(error) => Err(Unanswered::Threw(
                error.reason.trim_start_matches("Error: ").to_string(),
            )),
        }
    }
}

/// The request as the provider's callback receives it.
fn asked(request: &PositionRequest<'_>) -> JsRequest {
    JsRequest {
        scale: crate::generated::time_scale_to_str(request.scale.id()),
        frame_bits: request.frame.to_bits(),
        speeds: request.speeds,
        observer: request.observer.map(|place| Observer {
            longitude_deg: place.longitude.get(),
            latitude_deg: place.latitude.get(),
            altitude_m: place.altitude.get(),
        }),
        jds: request.jds.to_vec(),
        // SAFETY: `ConventionsC` is plain data with no pointers, so the
        // generated reader reads nothing beyond the value it is lent.
        conventions: unsafe {
            crate::generated::Conventions::write(&ConventionsC::of(request.conventions))
        },
        bodies: request
            .bodies
            .iter()
            .map(|body| crate::generated::body_to_str(body.id()))
            .collect(),
    }
}

/// The columns the callback returned, moved into the port's answer.
fn answer(columns: JsColumns) -> Answer {
    Answer {
        frame_bits: columns.frame_bits,
        lon: columns.lon,
        lat: columns.lat,
        dist: columns.dist,
        lon_speed: columns.lon_speed,
        lat_speed: columns.lat_speed,
        dist_speed: columns.dist_speed,
        status: columns.status,
        source: columns.source,
    }
}

/// A bound JavaScript provider, kept alive by the context that owns it.
#[derive(Debug)]
pub(crate) struct Host {
    bound: Bound<Callback>,
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
        let callback = Callback {
            positions: positions.create_ref()?,
            env: Cell::new(None),
        };
        Bound::new(info.into(), callback)
            .map(|bound| Host { bound })
            .map_err(Error::from_reason)
    }

    /// Lends the environment for one call into the SDK.
    pub(crate) fn enter(&self, env: Env) {
        let provider = self.bound.provider();
        provider.callback().env.set(Some(env));
        provider.forget_thrown();
    }

    /// Takes the environment back, and reports what the callback threw.
    ///
    /// # Errors
    ///
    /// The message the callback threw, so a failure inside a provider
    /// reaches the caller as its own error rather than as a status code.
    pub(crate) fn leave(&self) -> Result<()> {
        self.bound.provider().callback().env.set(None);
        self.bound.leave().map_err(Error::from_reason)
    }
}

/// The vtable pointer and the user data of an optional host, as the
/// generated constructor passes them.
#[must_use]
pub(crate) fn parts(host: Option<&Host>) -> (Option<ProviderVtable>, *mut core::ffi::c_void) {
    host::parts(host.map(|host| &host.bound))
}
