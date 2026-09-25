//! An ephemeris provider written in JavaScript, bound into the port's
//! vtable (`docs/02-architecture/07-binding-architecture.md`, "Ports
//! across the boundary").
//!
//! HAND-WRITTEN, like the Node addon's, because each binding wraps its
//! own callback mechanism: a `js_sys::Function` here. Everything else —
//! the description's defaults, reading the answer, keeping what the
//! callback threw — is `teistro_port_ephemeris::host`, the same for this
//! binding and Node's, so a provider behaves identically in both.
//!
//! The call is synchronous and on the thread that entered the SDK, which
//! is the boundary's contract; wasm has one thread, and a worker builds
//! its own context. Unlike napi's, a wasm-bindgen function needs no
//! environment lent to it, so the call can never find itself without one.

#![allow(
    unsafe_code,
    reason = "the port's trait is `Send + Sync`; the contract is argued at each impl"
)]

use serde::Deserialize;
use teistro_port_ephemeris::host::{self, Answer, Bound, Declared, HostCallback, Unanswered};
use teistro_port_ephemeris::{PositionRequest, ProviderVtable};
use wasm_bindgen::{JsCast, JsValue};

use crate::generated::{
    Error, Observer, PositionColumns as JsColumns, PositionRequest as JsRequest, Result,
};

/// What a JavaScript provider says about itself, as Node's `ProviderInfo`
/// is: everything but the name and the bodies has a default
/// (`teistro_port_ephemeris::host::Declared` holds them).
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    /// Whether identical requests give identical bits; `true` by default
    /// (ADR-0022).
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

/// The function the SDK reaches the provider through.
struct Callback {
    positions: js_sys::Function,
}

// SAFETY: wasm32 has one thread, and a context is used by one thread at a
// time, which is the boundary's documented contract; the callback is only
// reached from inside a call this module made. The port's trait requires
// `Send + Sync` because a native context may move between threads.
unsafe impl Send for Callback {}
// SAFETY: as above.
unsafe impl Sync for Callback {}

impl HostCallback for Callback {
    fn positions(
        &self,
        request: &PositionRequest<'_>,
    ) -> core::result::Result<Option<Answer>, Unanswered> {
        let asked = serde_wasm_bindgen::to_value(&asked(request))
            .map_err(|error| Unanswered::Threw(error.to_string()))?;
        let answered = self
            .positions
            .call1(&JsValue::NULL, &asked)
            .map_err(|thrown| Unanswered::Threw(sentence(&thrown)))?;
        // A provider that cannot produce the frame asked for says so by
        // answering with nothing, as in Node.
        if answered.is_undefined() || answered.is_null() {
            return Ok(None);
        }
        serde_wasm_bindgen::from_value::<JsColumns>(answered)
            .map(|columns| Some(answer(columns)))
            .map_err(|error| Unanswered::Threw(error.to_string()))
    }
}

/// What a callback threw, as a sentence: an `Error`'s message, a thrown
/// string itself, or what the value is.
fn sentence(thrown: &JsValue) -> String {
    thrown
        .dyn_ref::<js_sys::Error>()
        .map(|error| String::from(error.message()))
        .or_else(|| thrown.as_string())
        .unwrap_or_else(|| format!("{thrown:?}"))
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
    /// A body key the catalogue does not have, or frame bits no frame
    /// sets.
    pub(crate) fn bind(info: ProviderInfo, positions: &js_sys::Function) -> Result<Host> {
        let callback = Callback {
            positions: positions.clone(),
        };
        Bound::new(info.into(), callback)
            .map(|bound| Host { bound })
            .map_err(Error::from_reason)
    }

    /// Starts a call into the SDK: what an earlier one threw is forgotten.
    pub(crate) fn enter(&self) {
        self.bound.provider().forget_thrown();
    }

    /// Ends it, and reports what the callback threw.
    ///
    /// # Errors
    ///
    /// The message the callback threw, so a failure inside a provider
    /// reaches the caller as its own error rather than as a status code.
    pub(crate) fn leave(&self) -> Result<()> {
        self.bound.leave().map_err(Error::from_reason)
    }
}

/// The vtable pointer and the user data of an optional host, as the
/// generated constructor passes them.
#[must_use]
pub(crate) fn parts(host: Option<&Host>) -> (Option<ProviderVtable>, *mut core::ffi::c_void) {
    host::parts(host.map(|host| &host.bound))
}
