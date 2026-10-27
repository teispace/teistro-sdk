//! A frame's bits at the C ABI (`ts_frame_unpack`, `ts_frame_pack`): every
//! low half and a spread of high halves, so each bit pattern a caller
//! could hand over is either refused as `INVALID_ARG` or read as a frame
//! that packs back to exactly the bits it came from. A pattern read and
//! written differently would be two callers' frames compared equal that
//! were not.

#![allow(
    unsafe_code,
    clippy::unwrap_used,
    reason = "tests cross the boundary and fail by panicking"
)]

use teistro_core::Status;
use teistro_ffi::frame::{TsFrame, ts_frame_canonical, ts_frame_pack, ts_frame_unpack};
use teistro_port_ephemeris::Frame;

/// A frame record the library will fill.
fn empty() -> TsFrame {
    TsFrame {
        struct_size: u32::try_from(size_of::<TsFrame>()).unwrap(),
        ..TsFrame::of(Frame::CANONICAL)
    }
}

/// What the boundary reads `bits` as, and the bits it writes back.
fn round_trip(bits: u32) -> Result<u32, Status> {
    let mut frame = empty();
    // SAFETY: a sized record to write.
    let status = unsafe { ts_frame_unpack(bits, &raw mut frame) };
    if status != Status::Ok {
        return Err(status);
    }
    let mut packed = 0_u32;
    // SAFETY: a record the library wrote and a slot to write.
    let status = unsafe { ts_frame_pack(&raw const frame, &raw mut packed) };
    assert_eq!(status, Status::Ok, "{bits:#x} unpacked and did not pack");
    Ok(packed)
}

#[test]
fn every_frame_read_packs_back_to_its_bits_and_every_other_is_refused() {
    let highs = (0..=u16::MAX)
        .filter(|high| high.count_ones() <= 1 || *high < 64 || *high == u16::MAX)
        .map(u32::from);
    let mut read = 0_usize;
    for high in highs {
        for low in 0..=u32::from(u16::MAX) {
            let bits = high << 16 | low;
            match round_trip(bits) {
                Ok(packed) => {
                    read += 1;
                    assert_eq!(
                        packed, bits,
                        "{bits:#010x} read and written as {packed:#010x}"
                    );
                }
                Err(status) => assert_eq!(status, Status::InvalidArg, "{bits:#010x}"),
            }
        }
    }
    assert!(read > 0, "no pattern read at all");
}

#[test]
fn the_canonical_frame_packs_to_its_bits_and_a_record_out_of_range_is_refused() {
    let mut frame = empty();
    // SAFETY: a sized record to write.
    assert_eq!(unsafe { ts_frame_canonical(&raw mut frame) }, Status::Ok);
    let mut packed = u32::MAX;
    // SAFETY: a record the library wrote and a slot to write.
    assert_eq!(
        unsafe { ts_frame_pack(&raw const frame, &raw mut packed) },
        Status::Ok
    );
    assert_eq!(round_trip(packed), Ok(packed));
    // A centre past the catalogue's.
    let wrong = TsFrame {
        centre: 200,
        ..frame
    };
    // SAFETY: a sized record and a slot to write.
    assert_eq!(
        unsafe { ts_frame_pack(&raw const wrong, &raw mut packed) },
        Status::InvalidArg
    );
    // No slot to write into.
    // SAFETY: a null slot, which the entry point refuses.
    assert_eq!(
        unsafe { ts_frame_pack(&raw const frame, core::ptr::null_mut()) },
        Status::InvalidArg
    );
}
