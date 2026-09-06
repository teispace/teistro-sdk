//! The frame at the boundary: a caller names a centre, an equinox, a
//! coordinate system, a zodiac and the corrections by field, and the
//! library packs them into the 32 bits every position request carries
//! (`docs/03-design/ephemeris-port-and-adapters.md`, §3). Nothing outside
//! this module needs to know the packing.

#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use teistro_core::Status;
use teistro_core::catalogue::{Ayanamsha, Catalogued};
use teistro_core::error::{Detail, Error};
use teistro_core::key::KeyId;
use teistro_port_ephemeris::{Centre, Coordinates, Corrections, Equinox, Frame, Zodiac};

use crate::support::{c_struct, read_in, write_out};

/// A frame by name: what a position is seen from, in which equinox and
/// coordinates, in which zodiac, with which corrections applied.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TsFrame {
    /// `sizeof(ts_frame)` as the caller compiled it.
    pub struct_size: u32,
    /// The ayanamsha, read only when `sidereal` is set; `0xFFFF` names
    /// none, which is what a tropical frame carries.
    /// `api: enum=Ayanamsha nullable example=0`
    pub ayanamsha: u16,
    /// Where the position is seen from; a topocentric frame needs the
    /// request's observer.
    /// `api: enum=Centre example=0`
    pub centre: u8,
    /// Which equinox the coordinates refer to.
    /// `api: enum=Equinox example=0`
    pub equinox: u8,
    /// Ecliptic or equatorial.
    /// `api: enum=Coordinates example=0`
    pub coordinates: u8,
    /// The sidereal zodiac, which uses `ayanamsha`; tropical otherwise.
    /// `api: flag example=0`
    pub sidereal: u8,
    /// Whether light time is applied.
    /// `api: flag example=1`
    pub light_time: u8,
    /// Whether annual aberration is applied.
    /// `api: flag example=1`
    pub aberration: u8,
    /// Whether relativistic deflection is applied.
    /// `api: flag example=0`
    pub deflection: u8,
    /// Whether nutation is applied.
    /// `api: flag example=1`
    pub nutation: u8,
}

c_struct!(TsFrame);

impl TsFrame {
    /// The boundary form of a frame.
    #[must_use]
    pub fn of(frame: Frame) -> TsFrame {
        TsFrame {
            struct_size: 0,
            ayanamsha: frame
                .zodiac
                .ayanamsha()
                .map_or(KeyId::NONE_ID, Catalogued::id),
            centre: frame.centre as u8,
            equinox: frame.equinox as u8,
            coordinates: frame.coordinates as u8,
            sidereal: u8::from(frame.zodiac != Zodiac::Tropical),
            light_time: u8::from(frame.corrections.light_time),
            aberration: u8::from(frame.corrections.aberration),
            deflection: u8::from(frame.corrections.deflection),
            nutation: u8::from(frame.corrections.nutation),
        }
    }

    /// The frame this struct names.
    ///
    /// # Errors
    ///
    /// A centre, equinox, coordinate or ayanamsha value the catalogue does
    /// not have.
    pub fn to_frame(&self) -> Result<Frame, Error> {
        let zodiac = if self.sidereal == 0 {
            Zodiac::Tropical
        } else {
            let ayanamsha = Ayanamsha::from_id(self.ayanamsha).ok_or_else(|| {
                Error::unsupported(format!("no ayanamsha has id {}", self.ayanamsha))
                    .with_detail(Detail::UnknownKey)
                    .with_field("frame.ayanamsha")
            })?;
            Zodiac::sidereal(ayanamsha)
        };
        Ok(Frame {
            centre: centre(self.centre)?,
            equinox: equinox(self.equinox)?,
            coordinates: coordinates(self.coordinates)?,
            zodiac,
            corrections: Corrections {
                light_time: self.light_time != 0,
                aberration: self.aberration != 0,
                deflection: self.deflection != 0,
                nutation: self.nutation != 0,
            },
        })
    }
}

/// `INVALID_ARG` naming the field and what it accepts.
fn refuse(field: &str, value: u8, accepted: &str) -> Error {
    Error::invalid_arg(format!("`{field}` is {value}; the values are {accepted}")).with_field(field)
}

fn centre(value: u8) -> Result<Centre, Error> {
    Ok(match value {
        0 => Centre::Geocentric,
        1 => Centre::Topocentric,
        2 => Centre::Heliocentric,
        3 => Centre::Barycentric,
        other => {
            return Err(refuse(
                "frame.centre",
                other,
                "GEOCENTRIC=0, TOPOCENTRIC=1, HELIOCENTRIC=2, BARYCENTRIC=3",
            ));
        }
    })
}

fn equinox(value: u8) -> Result<Equinox, Error> {
    Ok(match value {
        0 => Equinox::OfDate,
        1 => Equinox::J2000,
        other => return Err(refuse("frame.equinox", other, "OF_DATE=0, J2000=1")),
    })
}

fn coordinates(value: u8) -> Result<Coordinates, Error> {
    Ok(match value {
        0 => Coordinates::Ecliptic,
        1 => Coordinates::Equatorial,
        other => {
            return Err(refuse(
                "frame.coordinates",
                other,
                "ECLIPTIC=0, EQUATORIAL=1",
            ));
        }
    })
}

/// Writes the SDK's canonical frame: apparent geocentric ecliptic of date,
/// tropical, which both licensed engines return by default and every chart
/// module consumes. Set `struct_size` before the call.
///
/// # Safety
///
/// `out_frame` must be valid for a read of its `struct_size` and a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_frame_canonical(out_frame: *mut TsFrame) -> Status {
    // SAFETY: the entry point's contract.
    match unsafe { write_out(out_frame, "out_frame", TsFrame::of(Frame::CANONICAL)) } {
        Ok(()) => Status::Ok,
        Err(error) => error.status,
    }
}

/// Packs a frame into the bits a position request carries.
///
/// # Safety
///
/// `frame` must be valid for a read of its `struct_size`; `out_bits` for a
/// write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_frame_pack(frame: *const TsFrame, out_bits: *mut u32) -> Status {
    // SAFETY: the entry point's contract.
    let packed = unsafe { read_in(frame, "frame") }.and_then(TsFrame::to_frame);
    match packed {
        Ok(frame) => {
            if out_bits.is_null() {
                return Status::InvalidArg;
            }
            // SAFETY: non-null; the caller promises a writable slot.
            unsafe { out_bits.write(frame.to_bits()) };
            Status::Ok
        }
        Err(error) => error.status,
    }
}

/// Reads packed frame bits back into their fields; bits no frame sets are
/// `INVALID_ARG`.
///
/// # Safety
///
/// `out_frame` must be valid for a read of its `struct_size` and a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_frame_unpack(bits: u32, out_frame: *mut TsFrame) -> Status {
    match Frame::try_from_bits(bits) {
        // SAFETY: the entry point's contract.
        Ok(frame) => match unsafe { write_out(out_frame, "out_frame", TsFrame::of(frame)) } {
            Ok(()) => Status::Ok,
            Err(error) => error.status,
        },
        Err(error) => Error::from(error).status,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn every_frame_round_trips_through_its_fields_and_its_bits() {
        let frames = [
            Frame::CANONICAL,
            Frame::CANONICAL
                .with_centre(Centre::Topocentric)
                .with_coordinates(Coordinates::Equatorial)
                .with_zodiac(Zodiac::sidereal(Ayanamsha::Lahiri)),
            Frame {
                centre: Centre::Barycentric,
                equinox: Equinox::J2000,
                coordinates: Coordinates::Equatorial,
                zodiac: Zodiac::Tropical,
                corrections: Corrections::GEOMETRIC,
            },
        ];
        for frame in frames {
            let c = TsFrame::of(frame);
            assert_eq!(c.to_frame().unwrap(), frame);
            assert_eq!(Frame::try_from_bits(frame.to_bits()).unwrap(), frame);
        }
        let mut wrong = TsFrame::of(Frame::CANONICAL);
        wrong.centre = 9;
        let error = wrong.to_frame().unwrap_err();
        assert_eq!(error.field(), Some("frame.centre"));
        assert!(error.message.contains("BARYCENTRIC=3"));
        wrong = TsFrame::of(Frame::CANONICAL);
        wrong.equinox = 4;
        assert_eq!(wrong.to_frame().unwrap_err().field(), Some("frame.equinox"));
        wrong = TsFrame::of(Frame::CANONICAL);
        wrong.coordinates = 4;
        assert_eq!(
            wrong.to_frame().unwrap_err().field(),
            Some("frame.coordinates")
        );
        wrong = TsFrame::of(Frame::CANONICAL);
        wrong.sidereal = 1;
        wrong.ayanamsha = 9999;
        let error = wrong.to_frame().unwrap_err();
        assert_eq!(error.status, Status::Unsupported);
        assert_eq!(error.field(), Some("frame.ayanamsha"));
    }
}
