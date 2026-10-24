//! The Rust floor of `03-design/ffi-cost.md`: the scenario every binding
//! times, through the C ABI from Rust, printed as one JSON line a key.
//!
//! ```sh
//! cargo run --release -p teistro-ffi --example ffi_cost [GRID]
//! ```
//!
//! `call-*` is `ts_positions` over 1 instant and 3 bodies, `grid-*` over
//! 366 instants and 10 bodies. `*-native` is a context over the library's
//! own test provider. `*-hosted` is one over a provider written in Rust
//! behind a [`ProviderVtable`], answering fixed columns as every binding's
//! hosted provider does, so a binding's hosted figure less this one is what
//! its own callback machinery costs. `decode` opens the grid's blob with
//! the reference reader alone. `GRID`, when given, is a path the grid's
//! blob is written to, for a binding that cannot reach a call's raw bytes
//! to decode the same blob. `cargo xtask measure-ffi-cost` runs it.

// An example, not a library: it reports through stdout and stops on a
// failed call, and it calls the C ABI, so the library's lints are relaxed
// here as they are in the boundary's own tests.
#![allow(
    unsafe_code,
    clippy::print_stdout,
    clippy::expect_used,
    reason = "an example that drives the C boundary"
)]

use std::ffi::CString;
use std::hint::black_box;
use std::ptr;
use std::time::Instant;

use teistro_core::Status;
use teistro_ffi::TS_CONTEXT_TEST_PROVIDER;
use teistro_ffi::blob::{TsBlob, ts_blob_free};
use teistro_ffi::context::{
    TsContext, TsContextOptions, TsEphemeris, ts_context_free, ts_context_new,
};
use teistro_ffi::frame::{TsFrame, ts_frame_canonical, ts_frame_pack};
use teistro_ffi::positions::ts_positions;
use teistro_port_ephemeris::test_provider::TestProvider;
use teistro_port_ephemeris::vtable::{Exported, ObserverC, PositionRequestC, ProviderVtable};
use teistro_port_ephemeris::{
    Body, Capabilities, Cell, CellStatus, EphemerisProvider, PositionColumns, PositionRequest,
    ProviderError, TimeScale,
};

/// The shortest run a figure is timed over, in seconds.
const RUN_SECONDS: f64 = 0.05;
/// The runs a figure is the median of.
const RUNS: usize = 5;

/// A provider written in Rust that answers every cell with one fixed
/// longitude, as each binding's hosted provider does: its work is the
/// columns, so what it costs is the crossing.
struct Fixed;

impl EphemerisProvider for Fixed {
    fn capabilities(&self) -> Capabilities {
        let mut capabilities = TestProvider.capabilities();
        capabilities.identity.name = String::from("ffi-cost-in-rust");
        capabilities
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        let mut columns =
            PositionColumns::new(request.jds.len(), request.bodies.len(), request.frame);
        let cell = Cell {
            lon: 123.25,
            lon_speed: 1.0,
            status: CellStatus::Ok,
            ..Cell::EMPTY
        };
        for index in 0..columns.len() {
            columns.set(index, cell);
        }
        Ok(columns)
    }
}

/// A context over the test provider, either the library's own or the same
/// one behind a vtable.
fn context(hosted: Option<(&ProviderVtable, *mut core::ffi::c_void)>) -> *mut TsContext {
    let profile = CString::new("nepali-default").expect("a profile id");
    let options = TsContextOptions {
        struct_size: u32::try_from(size_of::<TsContextOptions>()).unwrap_or(0),
        flags: if hosted.is_some() {
            0
        } else {
            TS_CONTEXT_TEST_PROVIDER
        },
        profile: profile.as_ptr(),
        settings_json: ptr::null(),
        locale: ptr::null(),
        layouts_json: ptr::null(),
        dashas_json: ptr::null(),
        ephemeris: TsEphemeris::None as u8,
    };
    let (vtable, user_data) = hosted.map_or((ptr::null(), ptr::null_mut()), |(v, u)| {
        (ptr::from_ref(v), u)
    });
    let mut out: *mut TsContext = ptr::null_mut();
    // SAFETY: valid pointers for the call; the vtable and its user data
    // outlive the context, which `main` frees first.
    let status = unsafe {
        ts_context_new(
            &raw const options,
            vtable,
            user_data,
            &raw mut out,
            ptr::null_mut(),
        )
    };
    assert!(status == Status::Ok, "the context builds");
    out
}

/// The canonical frame's bits.
fn frame_bits() -> u32 {
    let mut frame = TsFrame {
        struct_size: u32::try_from(size_of::<TsFrame>()).unwrap_or(0),
        ayanamsha: 0,
        centre: 0,
        equinox: 0,
        coordinates: 0,
        sidereal: 0,
        light_time: 0,
        aberration: 0,
        deflection: 0,
        nutation: 0,
    };
    let mut bits = 0;
    // SAFETY: valid pointers for the calls.
    unsafe {
        assert_eq!(ts_frame_canonical(&raw mut frame), Status::Ok);
        assert_eq!(ts_frame_pack(&raw const frame, &raw mut bits), Status::Ok);
    }
    bits
}

/// One `ts_positions` call's blob, copied out and freed.
fn positions(context: *mut TsContext, request: &PositionRequestC) -> Vec<u8> {
    let mut blob = TsBlob::empty();
    // SAFETY: a live handle, a valid request and a valid slot.
    let status = unsafe { ts_positions(context, request, &raw mut blob) };
    assert!(status == Status::Ok, "positions compute");
    // SAFETY: the library wrote `len` bytes at `data`, freed once after.
    let bytes = unsafe { std::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote.
    unsafe { ts_blob_free(&raw mut blob) };
    bytes
}

/// Times `call`: warmed, then run in batches long enough to time, five of
/// them, and printed as microseconds a call.
fn time(key: &str, mut call: impl FnMut()) {
    for _ in 0..100 {
        call();
    }
    let mut calls = 1_u32;
    loop {
        let start = Instant::now();
        for _ in 0..calls {
            call();
        }
        if start.elapsed().as_secs_f64() >= RUN_SECONDS || calls >= 1 << 24 {
            break;
        }
        calls *= 2;
    }
    let mut runs: Vec<f64> = (0..RUNS)
        .map(|_| {
            let start = Instant::now();
            for _ in 0..calls {
                call();
            }
            start.elapsed().as_secs_f64() * 1e6 / f64::from(calls)
        })
        .collect();
    runs.sort_by(f64::total_cmp);
    let median = runs.get(RUNS / 2).copied().unwrap_or_default();
    let (min, max) = (
        runs.first().copied().unwrap_or_default(),
        runs.last().copied().unwrap_or_default(),
    );
    println!(
        r#"{{"binding":"rust","key":"{key}","runs":{RUNS},"calls":{calls},"median_us":{median},"min_us":{min},"max_us":{max}}}"#
    );
}

fn main() {
    let exported = Exported::new(Fixed);
    let vtable = Exported::<Fixed>::vtable();
    let native = context(None);
    let hosted = context(Some((&vtable, exported.user_data())));
    let bits = frame_bits();

    let request = |jds: &[f64], bodies: &[u16]| PositionRequestC {
        struct_size: u32::try_from(size_of::<PositionRequestC>()).unwrap_or(0),
        scale: TimeScale::Ut1.id(),
        frame_bits: bits,
        speeds: 1,
        has_observer: 0,
        reserved: [0; 2],
        observer: ObserverC::default(),
        jds: jds.as_ptr(),
        jd_count: jds.len(),
        bodies: bodies.as_ptr(),
        body_count: bodies.len(),
    };
    let one = [2_451_545.0_f64];
    let three = [Body::Sun.id(), Body::Moon.id(), Body::Mars.id()];
    let year: Vec<f64> = (0..366).map(|day| 2_451_545.0 + f64::from(day)).collect();
    let ten: Vec<u16> = TestProvider::BODIES
        .iter()
        .take(10)
        .map(|b| b.id())
        .collect();
    let call = request(&one, &three);
    let grid = request(&year, &ten);

    // The hosted provider's answer is the grid it was asked, every cell as
    // it sent it, so a figure is never taken over a call that failed.
    let schema = teistro_ffi::schemas::positions();
    let answered = positions(hosted, &grid);
    let reader = teistro_idl::blob::Reader::parse(&answered, &schema).expect("the grid reads");
    let lon = reader.column("cells", "lon").expect("the longitudes");
    assert_eq!(lon.len(), 3660, "the hosted provider answers the grid");
    assert!(
        lon.iter().all(|cell| (cell.as_f64() - 123.25).abs() < 1e-9),
        "every cell as the provider sent it"
    );

    time("call-native", || {
        black_box(positions(native, &call));
    });
    time("call-hosted", || {
        black_box(positions(hosted, &call));
    });
    time("grid-native", || {
        black_box(positions(native, &grid));
    });
    time("grid-hosted", || {
        black_box(positions(hosted, &grid));
    });
    let bytes = positions(native, &grid);
    if let Some(path) = std::env::args().nth(1) {
        std::fs::write(&path, &bytes).expect("the grid's path is writable");
    }
    // The reference reader checks the table and borrows every section, as
    // a binding's views do, and copies nothing.
    time("decode", || {
        black_box(teistro_idl::blob::Reader::parse(&bytes, &schema).expect("the grid reads"));
    });

    // SAFETY: the handles came from `ts_context_new` and are not used
    // again; the hosted one is freed before the provider it calls.
    unsafe {
        ts_context_free(native);
        ts_context_free(hosted);
    }
    drop(exported);
}
