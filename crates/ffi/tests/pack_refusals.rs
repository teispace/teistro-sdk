//! A pack file damaged anywhere is refused at the C ABI, and a refused
//! file leaves the context as it found it: every prefix of a real pack
//! and every one-bit change to it comes back `TS_PACK` naming `bytes`,
//! and the whole file then loads as it does into a context that never saw
//! them, so nothing of a refused file stayed behind.

#![allow(
    unsafe_code,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests cross the boundary and fail by panicking"
)]

use core::ptr;

use teistro_core::Status;
use teistro_ffi::context::TsEphemeris;
use teistro_ffi::intl::{TsIntlLoaded, ts_intl_load_pack};

mod common;

use common::{Ctx, aspect_pack, sized};

/// What loading `bytes` into `ctx` answered, and the record it wrote.
fn load(ctx: &Ctx, bytes: &[u8]) -> (Status, TsIntlLoaded) {
    let mut loaded = sized(
        TsIntlLoaded {
            struct_size: 0,
            entries: 0,
            replaced: 0,
            merged: 0,
            locale: ptr::null(),
            sha256: ptr::null(),
        },
        |r, s| r.struct_size = s,
    );
    // SAFETY: a live context, readable bytes and a sized record.
    let status =
        unsafe { ts_intl_load_pack(ctx.handle, bytes.as_ptr(), bytes.len(), &raw mut loaded) };
    (status, loaded)
}

/// Asserts `bytes` is refused as a pack, naming the bytes.
fn refused(ctx: &Ctx, bytes: &[u8], what: &str) {
    let (status, _) = load(ctx, bytes);
    assert_eq!(status, Status::Pack, "{what} was not refused as a pack");
    let (_, _, field, _, _) = ctx.last_error();
    assert_eq!(field.as_deref(), Some("bytes"), "{what}");
}

#[test]
fn every_prefix_and_every_flipped_bit_is_refused_and_leaves_nothing() {
    let pack = aspect_pack();
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::None, None, None, None).unwrap();
    for len in 0..pack.len() {
        refused(&ctx, &pack[..len], &format!("the first {len} bytes"));
    }
    let mut damaged = pack.clone();
    for at in 0..pack.len() {
        for bit in 0..8 {
            damaged[at] ^= 1 << bit;
            refused(&ctx, &damaged, &format!("bit {bit} of byte {at} flipped"));
            damaged[at] ^= 1 << bit;
        }
    }
    // Loaded after them all, the file counts what it counts in a context
    // that never saw them.
    let fresh = Ctx::with_ephemeris(0, TsEphemeris::None, None, None, None).unwrap();
    let (status, clean) = load(&fresh, &pack);
    assert_eq!(status, Status::Ok);
    let (status, loaded) = load(&ctx, &pack);
    assert_eq!(status, Status::Ok);
    assert_eq!(
        (loaded.entries, loaded.replaced, loaded.merged),
        (clean.entries, clean.replaced, clean.merged),
        "a refused file left entries behind"
    );
}

#[test]
fn a_pack_with_bytes_after_its_end_is_refused() {
    let ctx = Ctx::with_ephemeris(0, TsEphemeris::None, None, None, None).unwrap();
    let mut longer = aspect_pack();
    longer.push(0);
    refused(&ctx, &longer, "a byte past the end");
}
