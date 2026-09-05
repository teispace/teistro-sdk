//! The IAU routines the astronomy layer needs, ported from ERFA
//! (Essential Routines for Fundamental Astronomy, the `NumFOCUS`
//! Foundation, BSD-3-Clause, derived with permission from SOFA), as
//! ADR-0021 requires: structure and operation order preserved so the C
//! and the Rust read side by side, coefficient tables as constants, no
//! allocation, every function tested against the reference values of
//! ERFA's own test program. The notice is in the repository's `NOTICE`.
//! Dates are two-part Julian dates as in ERFA, so an instant keeps its
//! sub-millisecond resolution.
//!
//! The provenance table (ported from ERFA 2.0.1, `github.com/liberfa/erfa`,
//! main branch of 2026-09-03, commit `1a8044c`; the test is the value in
//! `t_erfa_c.c` unless a sweep is named):
//!
//! | here | ERFA | what |
//! |---|---|---|
//! | [`anp`] | `eraAnp` | an angle into `[0, 2π)` |
//! | [`era00`] | `eraEra00` | the Earth rotation angle (IAU 2000) |
//! | [`gmst00`] | `eraGmst00` | Greenwich mean sidereal time (IAU 2000) |
//! | [`gmst06`] | `eraGmst06` | Greenwich mean sidereal time (IAU 2006) |
//! | [`obl80`] | `eraObl80` | mean obliquity (IAU 1980) |
//! | [`obl06`] | `eraObl06` | mean obliquity (IAU 2006) |
//! | [`pr00`] | `eraPr00` | the IAU 2000 precession-rate adjustments |
//! | [`nut00b`] | `eraNut00b` | nutation (IAU 2000B), the 77-term table in [`nut00b`] |
//! | [`fal03`], [`falp03`], [`faf03`], [`fad03`], [`faom03`], [`fave03`], [`fae03`], [`fapa03`] | `eraFal03` and kin | the fundamental arguments (IERS Conventions 2003) |
//! | [`eect00`] | `eraEect00` | the equation of the equinoxes' complementary terms (IAU 2000) |
//! | [`ee00`] | `eraEe00` | the equation of the equinoxes from an obliquity and a nutation |
//! | [`ee00b`] | `eraEe00b` | the equation of the equinoxes (IAU 2000B) |
//! | [`gst00b`] | `eraGst00b` | Greenwich apparent sidereal time (IAU 2000B) |
//! | [`ee06b`] | none: `eraEe06a` with the IAU 2000B nutation | the equation of the equinoxes (IAU 2006 precession, 2000B nutation) |
//! | [`gst06b`] | none: `eraGst06a` with the IAU 2000B nutation | Greenwich apparent sidereal time (IAU 2006 precession, 2000B nutation) |
//! | [`refco`] | `eraRefco` | the refraction constants of a standard model |
//! | [`vector::ir`], [`vector::rx`], [`vector::ry`], [`vector::rz`], [`vector::rxr`], [`vector::tr`], [`vector::rxp`], [`vector::trxp`], [`vector::pxp`], [`vector::pdp`], [`vector::pm`], [`vector::pn`], [`vector::sxp`], [`vector::c2s`], [`vector::s2c`], [`vector::anpm`] | `eraIr` and kin | the vector and matrix primitives |
//! | [`p06::p06e`] | `eraP06e` | the sixteen IAU 2006 precession angles |
//! | [`p06::pfw06`] | `eraPfw06` | the IAU 2006 bias-precession Fukushima-Williams angles |
//! | [`p06::fw2m`] | `eraFw2m` | a rotation matrix from Fukushima-Williams angles |
//! | [`p06::pmat06`] | `eraPmat06` | the IAU 2006 bias-precession matrix |
//! | [`p06::bp06`] | `eraBp06` | the frame bias, precession and bias-precession matrices (IAU 2006) |
//! | [`p06::bi00`] | `eraBi00` | the frame bias constants (IAU 2000) |
//! | [`ltp::ltpecl`] | `eraLtpecl` | the long-term ecliptic pole (Vondrák 2011) |
//! | [`ltp::ltpequ`] | `eraLtpequ` | the long-term equator pole (Vondrák 2011) |
//! | [`ltp::ltp`] | `eraLtp` | the long-term precession matrix (Vondrák 2011) |
//! | [`ltp::ltpb`] | `eraLtpb` | the long-term precession matrix with the frame bias |
//! | [`epv00::epv00`] | `eraEpv00` | the Earth's heliocentric and barycentric position and velocity (a simplified VSOP2000), the tables in [`epv00`] |
//! | [`apparent::pmpx`] | `eraPmpx` | proper motion and parallax |
//! | [`apparent::ld`], [`apparent::ldsun`] | `eraLd`, `eraLdsun` | the light deflection by a body and by the Sun |
//! | [`apparent::ab`] | `eraAb` | the stellar aberration |
//! | [`apparent::numat`] | `eraNumat` | the nutation matrix |
//! | [`ltp::ltpeps`] | none | the long-term general precession and obliquity series of Vondrák, Capitaine and Wallace (2011), equations 10 and Table 5, which ERFA does not carry; checked at J2000.0 against the IAU 2006 obliquity and over a millennium either side |
//!
//! Nothing here reads a clock or allocates; every function is a pure
//! computation on its arguments.

pub mod apparent;
pub mod epv00;
pub mod ltp;
pub mod nut00b;
pub mod p06;
pub mod vector;

/// J2000.0 as a Julian day (`ERFA_DJ00`).
pub const DJ00: f64 = 2_451_545.0;
/// Days per Julian century (`ERFA_DJC`).
pub const DJC: f64 = 36_525.0;

/// The Modified Julian Date zero point, JD 2400000.5.
pub const DJM0: f64 = 2_400_000.5;

/// J2000.0 as a Modified Julian Date.
pub const DJM00: f64 = 51_544.5;

/// Days in a Julian year.
pub const DJY: f64 = 365.25;
/// Days in a Julian millennium (`ERFA_DJM`).
pub const DJM: f64 = 365_250.0;
/// Seconds in a day (`ERFA_DAYSEC`).
pub const DAYSEC: f64 = 86_400.0;
/// The astronomical unit, metres (`ERFA_DAU`, IAU 2012).
pub const DAU: f64 = 149_597_870.7e3;
/// The speed of light, metres a second (`ERFA_CMPS`).
pub const CMPS: f64 = 299_792_458.0;
/// The light time for one au, seconds (`ERFA_AULT`).
pub const AULT: f64 = DAU / CMPS;
/// The speed of light, au a day (`ERFA_DC`).
pub const DC: f64 = DAYSEC / AULT;
/// The Schwarzschild radius of the Sun, au (`ERFA_SRS`: 2 G M☉ / c²).
pub const SRS: f64 = 1.974_125_743_36e-8;
/// Arcseconds to radians (`ERFA_DAS2R`, 4.848136811095359935899141e-6,
/// written to the double's own precision).
pub const DAS2R: f64 = 4.848_136_811_095_36e-6;
/// Milliarcseconds to radians (`ERFA_DMAS2R`).
pub const DMAS2R: f64 = DAS2R / 1e3;
/// Arcseconds in a full circle (`ERFA_TURNAS`).
pub const TURNAS: f64 = 1_296_000.0;
/// Two pi (`ERFA_D2PI`; the same double as the standard library's).
pub const D2PI: f64 = core::f64::consts::TAU;
/// Degrees to radians.
pub const DEG2RAD: f64 = core::f64::consts::PI / 180.0;
/// Radians to degrees.
pub const RAD2DEG: f64 = 180.0 / core::f64::consts::PI;

/// Julian centuries since J2000.0 of a two-part date.
#[must_use]
pub fn centuries(date1: f64, date2: f64) -> f64 {
    ((date1 - DJ00) + date2) / DJC
}

/// Normalises an angle into the range `0 <= a < 2π`. Port of `eraAnp`.
#[must_use]
pub fn anp(a: f64) -> f64 {
    let mut w = a % D2PI;
    if w < 0.0 {
        w += D2PI;
    }
    w
}

/// Earth rotation angle (IAU 2000 model) at a UT1 two-part date, radians.
/// Port of `eraEra00`.
#[must_use]
pub fn era00(dj1: f64, dj2: f64) -> f64 {
    // Days since fundamental epoch.
    let (d1, d2) = if dj1 < dj2 { (dj1, dj2) } else { (dj2, dj1) };
    let t = d1 + (d2 - DJ00);
    // Fractional part of T (days).
    let f = d1 % 1.0 + d2 % 1.0;
    // Earth rotation angle at this UT1.
    anp(D2PI * (f + 0.779_057_273_264_0 + 0.002_737_811_911_354_48 * t))
}

/// Greenwich mean sidereal time (model consistent with IAU 2000
/// resolutions), radians, from a UT1 date and a TT date. Port of
/// `eraGmst00`.
#[must_use]
pub fn gmst00(uta: f64, utb: f64, tta: f64, ttb: f64) -> f64 {
    // TT Julian centuries since J2000.0.
    let t = centuries(tta, ttb);
    // Greenwich Mean Sidereal Time, IAU 2000.
    anp(era00(uta, utb)
        + (0.014_506
            + (4_612.157_399_66 + (1.396_677_21 + (-0.000_093_44 + (0.000_018_82) * t) * t) * t)
                * t)
            * DAS2R)
}

/// Greenwich mean sidereal time (consistent with IAU 2006 precession),
/// radians, from a UT1 date and a TT date. Port of `eraGmst06`.
#[must_use]
pub fn gmst06(uta: f64, utb: f64, tta: f64, ttb: f64) -> f64 {
    // TT Julian centuries since J2000.0.
    let t = centuries(tta, ttb);
    // Greenwich mean sidereal time, IAU 2006.
    anp(era00(uta, utb)
        + (0.014_506
            + (4_612.156_534
                + (1.391_581_7
                    + (-0.000_000_44 + (-0.000_029_956 + (-0.000_000_036_8) * t) * t) * t)
                    * t)
                * t)
            * DAS2R)
}

/// Mean obliquity of the ecliptic, IAU 1980 model, radians, at a TT date.
/// Port of `eraObl80`.
#[must_use]
pub fn obl80(date1: f64, date2: f64) -> f64 {
    // Interval between fundamental epoch J2000.0 and given date (JC).
    let t = centuries(date1, date2);
    // Mean obliquity of date.
    DAS2R * (84_381.448 + (-46.8150 + (-0.000_59 + (0.001_813) * t) * t) * t)
}

/// Mean obliquity of the ecliptic, IAU 2006 precession model, radians, at
/// a TT date. Port of `eraObl06`.
#[must_use]
pub fn obl06(date1: f64, date2: f64) -> f64 {
    // Interval between fundamental date J2000.0 and given date (JC).
    let t = centuries(date1, date2);
    // Mean obliquity.
    (84_381.406
        + (-46.836_769
            + (-0.000_183_1 + (0.002_003_40 + (-0.000_000_576 + (-0.000_000_043_4) * t) * t) * t)
                * t)
            * t)
        * DAS2R
}

/// The precession-rate part of the IAU 2000 precession-nutation models
/// (part of MHB2000): the adjustments to the IAU 1976 precession in
/// longitude and to the obliquity, radians, at a TT date. Port of
/// `eraPr00`.
#[must_use]
pub fn pr00(date1: f64, date2: f64) -> (f64, f64) {
    // Precession and obliquity corrections (radians per century).
    const PRECOR: f64 = -0.299_65 * DAS2R;
    const OBLCOR: f64 = -0.025_24 * DAS2R;
    // Interval between fundamental epoch J2000.0 and given date (JC).
    let t = centuries(date1, date2);
    // Precession rate contributions with respect to IAU 1976/80.
    (PRECOR * t, OBLCOR * t)
}

/// Nutation in longitude and obliquity, radians.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nutation {
    /// Nutation in longitude, radians.
    pub dpsi: f64,
    /// Nutation in obliquity, radians.
    pub deps: f64,
}

/// Nutation, IAU 2000B model, at a TT date: the 77 luni-solar terms plus
/// fixed offsets in lieu of the planetary terms, about a milliarcsecond
/// over 1995 to 2050. Port of `eraNut00b`.
#[must_use]
pub fn nut00b(date1: f64, date2: f64) -> Nutation {
    // Units of 0.1 microarcsecond to radians.
    const U2R: f64 = DAS2R / 1e7;
    // Fixed offsets in lieu of planetary terms.
    const DPPLAN: f64 = -0.135 * DMAS2R;
    const DEPLAN: f64 = 0.388 * DMAS2R;

    // Interval between fundamental epoch J2000.0 and given date (JC).
    let t = centuries(date1, date2);

    // Fundamental arguments from Simon et al. (1994), in the truncated
    // form the model uses.
    // Mean anomaly of the Moon.
    let el = (485_868.249_036 + (1_717_915_923.217_8) * t) % TURNAS * DAS2R;
    // Mean anomaly of the Sun.
    let elp = (1_287_104.793_05 + (129_596_581.048_1) * t) % TURNAS * DAS2R;
    // Mean argument of the latitude of the Moon.
    let f = (335_779.526_232 + (1_739_527_262.847_8) * t) % TURNAS * DAS2R;
    // Mean elongation of the Moon from the Sun.
    let d = (1_072_260.703_69 + (1_602_961_601.209_0) * t) % TURNAS * DAS2R;
    // Mean longitude of the ascending node of the Moon.
    let om = (450_160.398_036 + (-6_962_890.543_1) * t) % TURNAS * DAS2R;

    // Summation of luni-solar nutation series (smallest terms first).
    let mut dp = 0.0;
    let mut de = 0.0;
    for term in nut00b::TERMS.iter().rev() {
        // Argument and functions.
        let arg = (f64::from(term.nl) * el
            + f64::from(term.nlp) * elp
            + f64::from(term.nf) * f
            + f64::from(term.nd) * d
            + f64::from(term.nom) * om)
            % D2PI;
        let (sarg, carg) = arg.sin_cos();
        // Term.
        dp += (term.ps + term.pst * t) * sarg + term.pc * carg;
        de += (term.ec + term.ect * t) * carg + term.es * sarg;
    }
    // Convert from 0.1 microarcsec units to radians.
    let dpsils = dp * U2R;
    let depsls = de * U2R;
    // Fixed offset to correct for missing terms in truncated series.
    Nutation {
        dpsi: dpsils + DPPLAN,
        deps: depsls + DEPLAN,
    }
}

/// Mean anomaly of the Moon (IERS Conventions 2003), radians, at `t`
/// Julian centuries since J2000.0. Port of `eraFal03`.
#[must_use]
pub fn fal03(t: f64) -> f64 {
    (485_868.249_036
        + t * (1_717_915_923.217_8 + t * (31.879_2 + t * (0.051_635 + t * (-0.000_244_70)))))
        % TURNAS
        * DAS2R
}

/// Mean anomaly of the Sun (IERS Conventions 2003). Port of `eraFalp03`.
#[must_use]
pub fn falp03(t: f64) -> f64 {
    (1_287_104.793_048
        + t * (129_596_581.048_1 + t * (-0.553_2 + t * (0.000_136 + t * (-0.000_011_49)))))
        % TURNAS
        * DAS2R
}

/// Mean longitude of the Moon minus that of the ascending node (IERS
/// Conventions 2003). Port of `eraFaf03`.
#[must_use]
pub fn faf03(t: f64) -> f64 {
    (335_779.526_232
        + t * (1_739_527_262.847_8 + t * (-12.751_2 + t * (-0.001_037 + t * (0.000_004_17)))))
        % TURNAS
        * DAS2R
}

/// Mean elongation of the Moon from the Sun (IERS Conventions 2003). Port
/// of `eraFad03`.
#[must_use]
pub fn fad03(t: f64) -> f64 {
    (1_072_260.703_692
        + t * (1_602_961_601.209_0 + t * (-6.370_6 + t * (0.006_593 + t * (-0.000_031_69)))))
        % TURNAS
        * DAS2R
}

/// Mean longitude of the Moon's ascending node (IERS Conventions 2003).
/// Port of `eraFaom03`.
#[must_use]
pub fn faom03(t: f64) -> f64 {
    (450_160.398_036
        + t * (-6_962_890.543_1 + t * (7.472_2 + t * (0.007_702 + t * (-0.000_059_39)))))
        % TURNAS
        * DAS2R
}

/// Mean longitude of Venus (IERS Conventions 2003). Port of `eraFave03`.
#[must_use]
pub fn fave03(t: f64) -> f64 {
    (3.176_146_697 + 1_021.328_554_621_1 * t) % D2PI
}

/// Mean longitude of Earth (IERS Conventions 2003). Port of `eraFae03`.
#[must_use]
pub fn fae03(t: f64) -> f64 {
    (1.753_470_314 + 628.307_584_999_1 * t) % D2PI
}

/// General accumulated precession in longitude (IERS Conventions 2003).
/// Port of `eraFapa03`.
#[must_use]
pub fn fapa03(t: f64) -> f64 {
    (0.024_381_750 + 0.000_005_386_91 * t) * t
}

/// One term of the complementary series: the multipliers of the eight
/// fundamental arguments (l, l′, F, D, Ω, `L_Ve`, `L_E`, `p_A`) and the sine and
/// cosine coefficients.
struct EectTerm {
    nfa: [i8; 8],
    s: f64,
    c: f64,
}

const fn e(nfa: [i8; 8], s: f64, c: f64) -> EectTerm {
    EectTerm { nfa, s, c }
}

/// Terms of order t^0 of the complementary series.
const EECT_E0: [EectTerm; 33] = [
    e([0, 0, 0, 0, 1, 0, 0, 0], 2640.96e-6, -0.39e-6),
    e([0, 0, 0, 0, 2, 0, 0, 0], 63.52e-6, -0.02e-6),
    e([0, 0, 2, -2, 3, 0, 0, 0], 11.75e-6, 0.01e-6),
    e([0, 0, 2, -2, 1, 0, 0, 0], 11.21e-6, 0.01e-6),
    e([0, 0, 2, -2, 2, 0, 0, 0], -4.55e-6, 0.00e-6),
    e([0, 0, 2, 0, 3, 0, 0, 0], 2.02e-6, 0.00e-6),
    e([0, 0, 2, 0, 1, 0, 0, 0], 1.98e-6, 0.00e-6),
    e([0, 0, 0, 0, 3, 0, 0, 0], -1.72e-6, 0.00e-6),
    e([0, 1, 0, 0, 1, 0, 0, 0], -1.41e-6, -0.01e-6),
    e([0, 1, 0, 0, -1, 0, 0, 0], -1.26e-6, -0.01e-6),
    e([1, 0, 0, 0, -1, 0, 0, 0], -0.63e-6, 0.00e-6),
    e([1, 0, 0, 0, 1, 0, 0, 0], -0.63e-6, 0.00e-6),
    e([0, 1, 2, -2, 3, 0, 0, 0], 0.46e-6, 0.00e-6),
    e([0, 1, 2, -2, 1, 0, 0, 0], 0.45e-6, 0.00e-6),
    e([0, 0, 4, -4, 4, 0, 0, 0], 0.36e-6, 0.00e-6),
    e([0, 0, 1, -1, 1, -8, 12, 0], -0.24e-6, -0.12e-6),
    e([0, 0, 2, 0, 0, 0, 0, 0], 0.32e-6, 0.00e-6),
    e([0, 0, 2, 0, 2, 0, 0, 0], 0.28e-6, 0.00e-6),
    e([1, 0, 2, 0, 3, 0, 0, 0], 0.27e-6, 0.00e-6),
    e([1, 0, 2, 0, 1, 0, 0, 0], 0.26e-6, 0.00e-6),
    e([0, 0, 2, -2, 0, 0, 0, 0], -0.21e-6, 0.00e-6),
    e([0, 1, -2, 2, -3, 0, 0, 0], 0.19e-6, 0.00e-6),
    e([0, 1, -2, 2, -1, 0, 0, 0], 0.18e-6, 0.00e-6),
    e([0, 0, 0, 0, 0, 8, -13, -1], -0.10e-6, 0.05e-6),
    e([0, 0, 0, 2, 0, 0, 0, 0], 0.15e-6, 0.00e-6),
    e([2, 0, -2, 0, -1, 0, 0, 0], -0.14e-6, 0.00e-6),
    e([1, 0, 0, -2, 1, 0, 0, 0], 0.14e-6, 0.00e-6),
    e([0, 1, 2, -2, 2, 0, 0, 0], -0.14e-6, 0.00e-6),
    e([1, 0, 0, -2, -1, 0, 0, 0], 0.14e-6, 0.00e-6),
    e([0, 0, 4, -2, 4, 0, 0, 0], 0.13e-6, 0.00e-6),
    e([0, 0, 2, -2, 4, 0, 0, 0], -0.11e-6, 0.00e-6),
    e([1, 0, -2, 0, -3, 0, 0, 0], 0.11e-6, 0.00e-6),
    e([1, 0, -2, 0, -1, 0, 0, 0], 0.11e-6, 0.00e-6),
];

/// Terms of order t^1 of the complementary series.
const EECT_E1: [EectTerm; 1] = [e([0, 0, 0, 0, 1, 0, 0, 0], -0.87e-6, 0.00e-6)];

/// The sum of a series over the fundamental arguments.
fn eect_sum(terms: &[EectTerm], fa: &[f64; 8]) -> f64 {
    let mut sum = 0.0;
    for term in terms.iter().rev() {
        let a = term
            .nfa
            .iter()
            .zip(fa)
            .fold(0.0, |acc, (n, value)| acc + f64::from(*n) * value);
        sum += term.s * a.sin() + term.c * a.cos();
    }
    sum
}

/// The equation of the equinoxes' complementary terms, consistent with
/// IAU 2000 resolutions, radians, at a TT date. Port of `eraEect00`.
#[must_use]
pub fn eect00(date1: f64, date2: f64) -> f64 {
    // Interval between fundamental epoch J2000.0 and current date (JC).
    let t = centuries(date1, date2);
    // Fundamental Arguments (from IERS Conventions 2003).
    let fa = [
        fal03(t),
        falp03(t),
        faf03(t),
        fad03(t),
        faom03(t),
        fave03(t),
        fae03(t),
        fapa03(t),
    ];
    // Evaluate the EE complementary terms.
    let s0 = eect_sum(&EECT_E0, &fa);
    let s1 = eect_sum(&EECT_E1, &fa);
    (s0 + s1 * t) * DAS2R
}

/// The equation of the equinoxes, compatible with IAU 2000 resolutions,
/// given the nutation in longitude and the mean obliquity, radians. Port
/// of `eraEe00`.
#[must_use]
pub fn ee00(date1: f64, date2: f64, epsa: f64, dpsi: f64) -> f64 {
    dpsi * epsa.cos() + eect00(date1, date2)
}

/// The equation of the equinoxes, compatible with IAU 2000 resolutions
/// but using the truncated nutation model IAU 2000B, radians, at a TT
/// date. Port of `eraEe00b`.
#[must_use]
pub fn ee00b(date1: f64, date2: f64) -> f64 {
    // IAU 2000 precession-rate adjustments.
    let (_dpsipr, depspr) = pr00(date1, date2);
    // Mean obliquity, consistent with IAU 2000 precession-nutation.
    let epsa = obl80(date1, date2) + depspr;
    // Nutation in longitude.
    let nutation = nut00b(date1, date2);
    // Equation of the equinoxes.
    ee00(date1, date2, epsa, nutation.dpsi)
}

/// Greenwich apparent sidereal time (consistent with IAU 2000 resolutions
/// but using the truncated nutation model IAU 2000B), radians, at a UT1
/// date taken as TT too. Port of `eraGst00b`.
#[must_use]
pub fn gst00b(uta: f64, utb: f64) -> f64 {
    let gmst = gmst00(uta, utb, uta, utb);
    let ee = ee00b(uta, utb);
    anp(gmst + ee)
}

/// The equation of the equinoxes consistent with IAU 2006 precession and
/// the IAU 2000B nutation, radians, at a TT date: the nutation in
/// longitude times the cosine of the IAU 2006 mean obliquity plus the
/// complementary terms, the equinox-based expression of IERS Conventions
/// 2010 (§5.5.7) with the truncated nutation. ERFA's `ee06a` is the same
/// with the IAU 2000A nutation; the two differ by that nutation's
/// truncation, under a milliarcsecond in the modern era.
#[must_use]
pub fn ee06b(date1: f64, date2: f64) -> f64 {
    let epsa = obl06(date1, date2);
    let nutation = nut00b(date1, date2);
    ee00(date1, date2, epsa, nutation.dpsi)
}

/// Greenwich apparent sidereal time consistent with IAU 2006 precession
/// and the IAU 2000B nutation, radians, from a UT1 date (the Earth's
/// rotation) and a TT date (the precession and nutation): [`gmst06`] plus
/// [`ee06b`]. ERFA's `gst06a` is the same with the IAU 2000A nutation.
#[must_use]
pub fn gst06b(uta: f64, utb: f64, tta: f64, ttb: f64) -> f64 {
    anp(gmst06(uta, utb, tta, ttb) + ee06b(tta, ttb))
}

/// `ERFA_GMAX` then `ERFA_GMIN`: the larger of the value and the lower
/// bound, then the smaller of that and the upper bound, by the C macros'
/// comparisons.
fn restrict(value: f64, lower: f64, upper: f64) -> f64 {
    let floored = if value > lower { value } else { lower };
    if floored < upper { floored } else { upper }
}

/// The refraction constants A and B of the model `dZ = A tan Z + B tan³ Z`
/// for a standard atmosphere at a pressure in hectopascals, a temperature
/// in Celsius, a relative humidity in `0..=1` and a wavelength in
/// micrometres. Port of `eraRefco`.
#[must_use]
pub fn refco(phpa: f64, tc: f64, rh: f64, wl: f64) -> (f64, f64) {
    // Decide whether optical/IR or radio case: switch at 100 microns.
    let optic = wl <= 100.0;
    // Restrict parameters to safe values, with the reference's own
    // comparisons (a NaN lands on the lower bound, as the C macros do).
    let t = restrict(tc, -150.0, 200.0);
    let p = restrict(phpa, 0.0, 10_000.0);
    let r = restrict(rh, 0.0, 1.0);
    let w = restrict(wl, 0.1, 1e6);
    // Water vapour pressure at the observer.
    let pw = if p > 0.0 {
        let ps = 10f64.powf((0.7859 + 0.03477 * t) / (1.0 + 0.00412 * t))
            * (1.0 + p * (4.5e-6 + 6e-10 * t * t));
        r * ps / (1.0 - (1.0 - r) * ps / p)
    } else {
        0.0
    };
    // Refractive index minus 1 at the observer.
    let tk = t + 273.15;
    let gamma = if optic {
        let wlsq = w * w;
        ((77.534_84e-6 + (4.391_08e-7 + 3.666e-9 / wlsq) / wlsq) * p - 11.2684e-6 * pw) / tk
    } else {
        (77.6890e-6 * p - (6.3938e-6 - 0.375_463 / tk) * pw) / tk
    };
    // Formula for beta from Stone, with empirical adjustments.
    let mut beta = 4.4474e-6 * tk;
    if !optic {
        beta -= 0.0074 * pw * beta;
    }
    // Refraction constants from Green.
    (gamma * (1.0 - beta), -gamma * (beta - gamma / 2.0))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::excessive_precision,
        reason = "tests fail by panicking, and the reference values are quoted as ERFA prints them"
    )]

    use super::*;

    /// The reference program's check: `|value - expected| <= tolerance`.
    fn vvd(value: f64, expected: f64, tolerance: f64, name: &str) {
        assert!(
            (value - expected).abs() <= tolerance,
            "{name}: {value} against {expected}"
        );
    }

    #[test]
    fn the_ports_reproduce_erfa_reference_values() {
        vvd(anp(-0.1), 6.183_185_307_179_586_477, 1e-12, "anp");
        vvd(
            era00(2_400_000.5, 54_388.0),
            0.402_283_724_002_815_810_2,
            1e-12,
            "era00",
        );
        vvd(
            gmst00(2_400_000.5, 53_736.0, 2_400_000.5, 53_736.0),
            1.754_174_972_210_740_592,
            1e-12,
            "gmst00",
        );
        vvd(
            gmst06(2_400_000.5, 53_736.0, 2_400_000.5, 53_736.0),
            1.754_174_971_870_091_203,
            1e-12,
            "gmst06",
        );
        vvd(
            obl80(2_400_000.5, 54_388.0),
            0.409_075_134_764_381_621_8,
            1e-14,
            "obl80",
        );
        vvd(
            obl06(2_400_000.5, 54_388.0),
            0.409_074_922_938_725_820_4,
            1e-14,
            "obl06",
        );
        let (dpsipr, depspr) = pr00(2_400_000.5, 53_736.0);
        vvd(
            dpsipr,
            -0.871_646_517_266_834_762_9e-7,
            1e-22,
            "pr00 dpsipr",
        );
        vvd(
            depspr,
            -0.734_201_838_672_281_308_7e-8,
            1e-22,
            "pr00 depspr",
        );
        let n = nut00b(2_400_000.5, 53_736.0);
        vvd(
            n.dpsi,
            -0.963_255_229_114_836_278_3e-5,
            1e-13,
            "nut00b dpsi",
        );
        vvd(n.deps, 0.406_319_710_662_115_936_7e-4, 1e-13, "nut00b deps");
        vvd(fal03(0.80), 5.132_369_751_108_684_150, 1e-12, "fal03");
        vvd(falp03(0.80), 6.226_797_973_505_507_345, 1e-12, "falp03");
        vvd(faf03(0.80), 0.259_771_136_674_549_951_8, 1e-12, "faf03");
        vvd(fad03(0.80), 1.946_709_205_396_925_672, 1e-12, "fad03");
        vvd(faom03(0.80), -5.973_618_440_951_302_183, 1e-12, "faom03");
        vvd(fave03(0.80), 3.424_900_460_533_758_000, 1e-12, "fave03");
        vvd(fae03(0.80), 1.744_713_738_913_081_846, 1e-12, "fae03");
        vvd(
            fapa03(0.80),
            0.195_088_476_224_000_000_0e-1,
            1e-12,
            "fapa03",
        );
        vvd(
            eect00(2_400_000.5, 53_736.0),
            0.204_608_500_488_512_526_4e-8,
            1e-20,
            "eect00",
        );
        vvd(
            ee00(
                2_400_000.5,
                53_736.0,
                0.409_078_976_335_650_990_0,
                -0.963_090_910_711_558_239_3e-5,
            ),
            -0.883_419_323_536_796_547_9e-5,
            1e-18,
            "ee00",
        );
        vvd(
            ee00b(2_400_000.5, 53_736.0),
            -0.883_570_006_000_303_283_1e-5,
            1e-18,
            "ee00b",
        );
        vvd(
            gst00b(2_400_000.5, 53_736.0),
            1.754_166_136_510_680_589,
            1e-12,
            "gst00b",
        );
        let (a, b) = refco(800.0, 10.0, 0.9, 0.4);
        vvd(a, 0.226_494_995_624_141_500_9e-3, 1e-15, "refco refa");
        vvd(b, -0.259_865_826_172_934_397_0e-6, 1e-18, "refco refb");
    }

    #[test]
    fn the_iau_2006_sidereal_time_agrees_with_erfa_within_the_2000b_truncation() {
        // The IAU 2006 expressions with the 2000B nutation against ERFA's
        // `ee06a` and `gst06a` with the 2000A: the truncation, 0.3 mas at
        // this date, inside a milliarcsecond (4.85e-9 rad); and the
        // composition is exact.
        vvd(
            ee06b(2_400_000.5, 53_736.0),
            -0.883_419_507_204_379_015_6e-5,
            4.85e-9,
            "ee06b against ee06a",
        );
        vvd(
            gst06b(2_400_000.5, 53_736.0, 2_400_000.5, 53_736.0),
            1.754_166_137_675_019_159,
            4.85e-9,
            "gst06b against gst06a",
        );
        vvd(
            gst06b(2_400_000.5, 53_736.0, 2_400_000.5, 53_736.0),
            anp(gmst06(2_400_000.5, 53_736.0, 2_400_000.5, 53_736.0) + ee06b(2_400_000.5, 53_736.0)),
            0.0,
            "gst06b is gmst06 plus ee06b",
        );
    }

    #[test]
    fn two_part_dates_agree_with_one_part_dates_to_the_resolution_of_a_double() {
        // The same instant split two ways gives the same nutation to the
        // double's resolution at the argument.
        let a = nut00b(2_451_545.0, 0.25);
        let b = nut00b(2_400_000.5, 51_544.75);
        assert!((a.dpsi - b.dpsi).abs() < 1e-18 && (a.deps - b.deps).abs() < 1e-18);
        // The radio branch of the refraction constants answers too.
        let (radio_a, radio_b) = refco(1013.25, 15.0, 0.5, 500.0);
        assert!(radio_a > 0.0 && radio_b < 0.0);
        // Zero pressure gives no water vapour and finite constants.
        let (dry_a, dry_b) = refco(0.0, 15.0, 0.5, 0.55);
        assert!(dry_a.is_finite() && dry_b.is_finite());
    }
}
