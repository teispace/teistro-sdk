//! Writes one blob of every schema into a directory, through the C ABI, so
//! a binding's decoder can be tested against what the library really
//! produces rather than against a hand-built buffer.
//!
//! ```sh
//! cargo run -p teistro-ffi --example blob_fixtures -- target/tsrb
//! ```
//!
//! `cargo xtask check-node` runs it before the Node test.

// An example, not a library: it reports through stdout and stops on a bad
// argument, and it calls the C ABI, so the library's lints are relaxed
// here as they are in the boundary's own tests.
#![allow(
    unsafe_code,
    clippy::print_stdout,
    clippy::expect_used,
    reason = "an example that drives the C boundary"
)]

use std::ffi::CString;
use std::path::Path;
use std::ptr;

use teistro_core::Status;
use teistro_ffi::TS_CONTEXT_TEST_PROVIDER;
use teistro_ffi::blob::{TsBlob, ts_blob_free};
use teistro_ffi::context::{TsContext, TsContextOptions, ts_context_free, ts_context_new};
use teistro_ffi::frame::{TsFrame, ts_frame_canonical, ts_frame_pack};
use teistro_ffi::intl::ts_intl_render;
use teistro_ffi::positions::ts_positions;
use teistro_port_ephemeris::vtable::{ObserverC, PositionRequestC};
use teistro_port_ephemeris::{Body, TimeScale};

/// The bytes of a blob the library filled, copied out before it is freed.
fn take(blob: &mut TsBlob) -> Vec<u8> {
    if blob.data.is_null() {
        return Vec::new();
    }
    // SAFETY: the library wrote `len` bytes at `data`.
    let bytes = unsafe { std::slice::from_raw_parts(blob.data, blob.len) }.to_vec();
    // SAFETY: a descriptor the library wrote, freed once.
    unsafe { ts_blob_free(&raw mut *blob) };
    bytes
}

fn write(dir: &Path, name: &str, bytes: &[u8]) {
    let path = dir.join(name);
    std::fs::write(&path, bytes).expect("the fixture directory is writable");
    println!("{} ({} bytes)", path.display(), bytes.len());
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("target/tsrb"));
    let dir = Path::new(&dir);
    std::fs::create_dir_all(dir).expect("the fixture directory is creatable");

    let profile = CString::new("nepali-default").expect("a profile id");
    let locale = CString::new("ne-Deva-NP").expect("a locale tag");
    let options = TsContextOptions {
        struct_size: u32::try_from(size_of::<TsContextOptions>()).unwrap_or(0),
        flags: TS_CONTEXT_TEST_PROVIDER,
        profile: profile.as_ptr(),
        settings_json: ptr::null(),
        locale: locale.as_ptr(),
    };
    let mut context: *mut TsContext = ptr::null_mut();
    // SAFETY: valid pointers for the call.
    let status = unsafe {
        ts_context_new(
            &raw const options,
            ptr::null(),
            ptr::null_mut(),
            &raw mut context,
            ptr::null_mut(),
        )
    };
    assert!(status == Status::Ok, "the context builds");

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
    let mut bits = 0u32;
    // SAFETY: valid pointers for the calls.
    unsafe {
        assert!(
            ts_frame_canonical(&raw mut frame) == Status::Ok,
            "the canonical frame"
        );
        assert!(
            ts_frame_pack(&raw const frame, &raw mut bits) == Status::Ok,
            "the frame packs"
        );
    }

    let jds = [2_451_545.0_f64, 2_451_546.0];
    let bodies = [Body::Sun.id(), Body::Moon.id(), Body::Mars.id()];
    let request = PositionRequestC {
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
    let mut blob = TsBlob::empty();
    // SAFETY: a live handle, a valid request and a valid slot.
    let status = unsafe { ts_positions(context, &raw const request, &raw mut blob) };
    assert!(status == Status::Ok, "positions compute");
    write(dir, "positions.tsrb", &take(&mut blob));

    let key = CString::new("sdk.reason.grahaInBhava").expect("a key");
    let params = CString::new(r#"{"graha": {"$entity": "graha.JUPITER"}, "bhava": 7}"#)
        .expect("the parameters");
    // SAFETY: a live handle and valid pointers.
    let status = unsafe { ts_intl_render(context, key.as_ptr(), params.as_ptr(), &raw mut blob) };
    assert!(status == Status::Ok, "the message renders");
    write(dir, "intl_render.tsrb", &take(&mut blob));

    // SAFETY: the handle came from `ts_context_new` and is not used again.
    unsafe { ts_context_free(context) };
}
