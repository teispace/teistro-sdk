//! The time scales, as a value, and what a conversion between two of
//! them applied.
//!
//! **The surface owns these two types, and that is the rule rather than
//! an exception.** The crates keep a scale in the *type system* —
//! `JulianDay<Ut1>`, `<Tt>`, `<Utc>` — which is right for code that
//! knows its scales when it is written and gives a caller who names them
//! at run time nothing to name them with. `crates/ffi` invented a
//! `TsScale` for that reason and a private `Applied` beside it; the
//! surface owns them so the boundary can convert *from* them once
//! `03-design/rust-consumer-surface.md`'s inversion lands, rather than
//! there being a third copy.

use teistro_astro::delta_t::DeltaT;

/// A time scale, as a value a caller can choose at run time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scale {
    /// Universal time: the Earth's rotation, which is what a sunrise is
    /// reckoned against.
    Ut1,
    /// Terrestrial time: the uniform scale the ephemerides are in.
    Tt,
    /// Coordinated universal time: the civil scale, with leap seconds.
    Utc,
}

impl Scale {
    /// The scale's key, as every catalogue and fixture spells it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Scale::Ut1 => "UT1",
            Scale::Tt => "TT",
            Scale::Utc => "UTC",
        }
    }
}

/// A converted instant, and **what was applied to get it**.
///
/// Never only the number: the "no dead ends" brief asks that what was
/// applied be reported, and a ΔT of 63.8 seconds from one model is not
/// the same answer as 63.8 from another. A caching consumer needs the
/// model; an auditing one needs all of it.
#[derive(Clone, Copy, Debug)]
pub struct Conversion {
    /// The instant, in the scale it was converted to.
    pub jd: f64,
    /// The ΔT applied, when the conversion needed one. `None` where
    /// UT1 and UTC differ only by DUT1.
    pub delta_t: Option<DeltaT>,
    /// Whether UTC was extended before leap seconds began, which is a
    /// convention rather than a measurement.
    pub proleptic_utc: bool,
    /// The DUT1 applied, in seconds.
    pub dut1_seconds: f64,
}

impl Conversion {
    /// A conversion that applied nothing, which is a scale to itself.
    pub(crate) const NONE: Conversion = Conversion {
        jd: 0.0,
        delta_t: None,
        proleptic_utc: false,
        dut1_seconds: 0.0,
    };
}
