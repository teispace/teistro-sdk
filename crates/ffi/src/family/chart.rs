//! The chart area's entry points in a build without the `chart` family,
//! each refusing as `CAPABILITY` (`03-design/wasm-profiles.md`).
//!
//! The symbols are the ones `chart.rs` and `naam.rs` export, with the same
//! arguments, so the generated glue binds every build alike; the request
//! struct is compiled into every build for that reason, and is never read
//! here. They are described once, from those modules, and not again from
//! here.
//!
//! lint: boundary-is-described
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

use teistro_core::error::Status;

use crate::blob::TsBlob;
pub use crate::chart_request::TsChartRequest;
use crate::context::TsContext;
use crate::string::TsString;
use crate::support::with_context;

/// `ts_chart_found`, refused: this build has no chart area.
///
/// # Safety
///
/// `context` must be a live handle; nothing else is read.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_chart_found(
    context: *const TsContext,
    _request: *const TsChartRequest,
    _out_blob: *mut TsBlob,
) -> Status {
    with_context(context, |_| {
        Err(teistro_core::error::Error::left_out("chart"))
    })
}

/// `ts_chart_layout_row`, refused: this build has no chart area.
///
/// # Safety
///
/// `context` must be a live handle; nothing else is read.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_chart_layout_row(
    context: *const TsContext,
    _key: *const c_char,
    _out_json: *mut TsString,
) -> Status {
    with_context(context, |_| {
        Err(teistro_core::error::Error::left_out("chart"))
    })
}

/// `ts_naam_milan`, refused: naam milan is matching, which the chart area
/// holds.
///
/// # Safety
///
/// `context` must be a live handle; nothing else is read.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_naam_milan(
    context: *const TsContext,
    _request_json: *const c_char,
    _out_blob: *mut TsBlob,
) -> Status {
    with_context(context, |_| {
        Err(teistro_core::error::Error::left_out("chart"))
    })
}
