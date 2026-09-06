//! Positions over the ephemeris port at the boundary: the port's own C
//! request, completed into the requested frame by the astronomy layer,
//! returned as a result blob with the steps that produced it and the
//! provenance envelope (`docs/03-design/ephemeris-port-and-adapters.md`).

#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use serde::Serialize;
use teistro_astro::completion::{Completed, Completion, CompletionError};
use teistro_core::Status;
use teistro_core::envelope::{
    CALCULATION_VERSION, Provenance, Version, canonical_json, content_hash,
};
use teistro_core::error::Error;
use teistro_idl::blob::{ColumnData, Writer};
use teistro_port_ephemeris::{DecodedRequest, PositionRequestC, ProviderError};
use teistro_time::EmbeddedTzdb;

use crate::blob::TsBlob;
use crate::context::TsContext;
use crate::schemas;
use crate::support::{with_context, write_plain};

/// The request as the input hash sees it.
#[derive(Serialize)]
struct RequestRecord<'a> {
    scale: &'static str,
    frame: String,
    bodies: Vec<&'static str>,
    jds: &'a [f64],
    observer: Option<[f64; 3]>,
    speeds: bool,
}

impl<'a> RequestRecord<'a> {
    fn of(decoded: &'a DecodedRequest<'_>) -> RequestRecord<'a> {
        RequestRecord {
            scale: decoded.scale.name(),
            frame: decoded.frame.key(),
            bodies: decoded.bodies.iter().map(|b| b.key()).collect(),
            jds: decoded.jds,
            observer: decoded
                .observer
                .map(|p| [p.latitude.get(), p.longitude.get(), p.altitude.get()]),
            speeds: decoded.speeds,
        }
    }
}

/// Positions over a grid of instants and bodies in the requested frame:
/// the provider answers in the frame it can, the SDK completes the rest
/// under the profile's override policy, and every step is stamped. A
/// context without an ephemeris is `CAPABILITY`; a provider failure is
/// `PROVIDER` with the provider's own code in the last error.
///
/// `api: blob=positions`
///
/// # Safety
///
/// `context` must be a live handle; `request` valid for a read, with its
/// instants and body ids valid for the counts it states; `out_blob` valid
/// for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_positions(
    context: *const TsContext,
    request: *const PositionRequestC,
    out_blob: *mut TsBlob,
) -> Status {
    with_context(context, |ctx| {
        if request.is_null() {
            return Err(crate::support::null("request"));
        }
        // SAFETY: non-null; the caller promises a readable request whose
        // arrays hold the counts it states.
        let decoded = unsafe { (*request).decode() }?;
        let provider = ctx.provider().ok_or_else(|| {
            Error::new(
                Status::Capability,
                "the context has no ephemeris: pass a provider vtable to ts_context_new, or the TS_CONTEXT_TEST_PROVIDER flag for tests",
            )
            .with_field("provider")
        })?;
        let completion =
            Completion::new(provider, ctx.settings().provider.overrides, ctx.delta_t());
        let completed = match completion.positions(&decoded.request()) {
            Ok(completed) => completed,
            Err(error) => {
                if let CompletionError::Provider {
                    error: ProviderError::Provider { code, .. },
                } = &error
                {
                    ctx.set_provider_code(*code);
                }
                return Err(error.into());
            }
        };
        let provenance = provenance(ctx, &completion, &decoded, &completed);
        let encoded = encode(&decoded, &completed, &provenance)?;
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_blob, "out_blob", TsBlob::from_vec(encoded)) }
    })
}

fn provenance<P: teistro_port_ephemeris::EphemerisProvider + ?Sized>(
    ctx: &TsContext,
    completion: &Completion<'_, P>,
    decoded: &DecodedRequest<'_>,
    completed: &Completed,
) -> Provenance {
    let settings = ctx.settings();
    let mut provenance = Provenance::new(
        Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Version::new(0, 0, 0)),
        CALCULATION_VERSION,
        teistro_core::catalogue::SCHEMA_VERSION,
        ctx.profile(),
        settings.hash(),
        content_hash(&RequestRecord::of(decoded)),
    );
    provenance.provider = completion
        .capabilities()
        .identity
        .stamp(completed.columns.frame, completed.step_keys());
    provenance.time.delta_t_model = ctx.delta_t().key().to_string();
    provenance.time.leap_table = teistro_time::leap::version().to_string();
    provenance.time.tzdb_version = EmbeddedTzdb::bundled_version().to_string();
    provenance.content_hash = content_hash(&completed.columns);
    provenance
}

fn encode(
    decoded: &DecodedRequest<'_>,
    completed: &Completed,
    provenance: &Provenance,
) -> Result<Vec<u8>, Error> {
    let columns = &completed.columns;
    let schema = schemas::positions();
    let mut writer = Writer::new(&schema);
    let count = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    let bodies: Vec<u16> = decoded.bodies.iter().map(|b| b.id()).collect();
    let status: Vec<i32> = columns.status.iter().map(|s| s.code()).collect();
    let source: Vec<u32> = columns.source.iter().map(|s| s.to_bits()).collect();
    let steps = serde_json::to_string(&completed.steps).unwrap_or_else(|_| String::from("[]"));
    (|| {
        writer.fixed(
            "summary",
            &[
                columns.frame.to_bits().into(),
                count(columns.jd_count).into(),
                count(columns.body_count).into(),
                decoded.scale.id().into(),
            ],
        )?;
        writer.columns(
            "instants",
            decoded.jds.len(),
            &[ColumnData::F64(decoded.jds)],
        )?;
        writer.columns("bodies", bodies.len(), &[ColumnData::U16(&bodies)])?;
        writer.columns(
            "cells",
            columns.len(),
            &[
                ColumnData::F64(&columns.lon),
                ColumnData::F64(&columns.lat),
                ColumnData::F64(&columns.dist),
                ColumnData::F64(&columns.lon_speed),
                ColumnData::F64(&columns.lat_speed),
                ColumnData::F64(&columns.dist_speed),
                ColumnData::I32(&status),
                ColumnData::U32(&source),
            ],
        )?;
        writer.bytes("steps", steps.as_bytes())?;
        writer.bytes("provenance", canonical_json(provenance).as_bytes())?;
        writer.finish()
    })()
    .map_err(|e| Error::internal(format!("the positions blob did not encode: {e}")))
}
