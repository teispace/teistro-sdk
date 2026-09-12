//! `sdk.frame`: the canonical frame, and packing one to carry it.

use teistro_core::error::Error;
use teistro_port_ephemeris::Frame;

use crate::context::Context;

/// `sdk.frame`: the frame the SDK computes in unless a caller says
/// otherwise, and the twenty-two bits that carry one in a cache key or a
/// stored chart.
///
/// The operations do not read the context, and the area still takes one:
/// what a consumer holds is `sdk.frame()`, the same value in every
/// binding, and an area that took no context would be the one member of
/// this surface reached a different way.
#[derive(Clone, Copy, Debug)]
pub struct FrameArea<'a> {
    context: &'a Context,
}

impl<'a> FrameArea<'a> {
    pub(crate) fn of(context: &'a Context) -> FrameArea<'a> {
        FrameArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(&self) -> &'a Context {
        self.context
    }

    /// The canonical frame: geocentric, of date, ecliptic, tropical,
    /// apparent — what an ephemeris answers in before the SDK completes
    /// it.
    #[must_use]
    pub fn canonical(&self) -> Frame {
        Frame::CANONICAL
    }

    /// A frame as the bits that carry it.
    #[must_use]
    pub fn pack(&self, frame: Frame) -> u32 {
        frame.to_bits()
    }

    /// A frame from the bits that carried it.
    ///
    /// # Errors
    ///
    /// Bits that name no frame — a reserved field set, or a member no
    /// catalogue has.
    pub fn unpack(&self, bits: u32) -> Result<Frame, Error> {
        Frame::try_from_bits(bits).map_err(Error::from)
    }
}
