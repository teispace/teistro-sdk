//! The IAU 2006 precession (Capitaine, Wallace and Chapront 2003, the P03
//! solution adopted by IAU 2006) ported from ERFA: the sixteen precession
//! angles, the Fukushima-Williams bias-precession angles and the matrices
//! built from them, and the frame bias constants. Two-part Julian dates
//! throughout, as in ERFA, and ERFA's own names for the angles, kept so
//! the port reads beside the C.

#![allow(
    clippy::similar_names,
    reason = "ERFA's names for the precession angles (psia, pia, bpa, bpia, epsa, eps0, gamb, phib, psib)"
)]

use super::vector::{Matrix3, ir, rx, rxr, rz, tr};
use super::{DAS2R, DJM00, DJM0, centuries, obl06};

/// The IAU 2006 precession angles at a TT date (`eraP06e`), radians, in
/// ERFA's order and names: the J2000.0 obliquity, the luni-solar
/// precession, the inclination of the equator on the J2000.0 ecliptic,
/// the ecliptic pole components, the planetary precession angles, the
/// mean obliquity of date, the planetary precession and the equatorial
/// precession angles, the general precession in longitude, and the
/// Fukushima-Williams angles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrecessionAngles {
    /// The obliquity at J2000.0.
    pub eps0: f64,
    /// The luni-solar precession.
    pub psia: f64,
    /// The inclination of the mean equator of date on the J2000.0 ecliptic.
    pub oma: f64,
    /// The ecliptic pole x, J2000.0 ecliptic triad.
    pub bpa: f64,
    /// The ecliptic pole −y, J2000.0 ecliptic triad.
    pub bqa: f64,
    /// The angle between the moving and J2000.0 ecliptics.
    pub pia: f64,
    /// The longitude of the ascending node of the ecliptic.
    pub bpia: f64,
    /// The mean obliquity of the ecliptic of date.
    pub epsa: f64,
    /// The planetary precession.
    pub chia: f64,
    /// The equatorial precession: −3rd 323 Euler angle.
    pub za: f64,
    /// The equatorial precession: −1st 323 Euler angle.
    pub zetaa: f64,
    /// The equatorial precession: 2nd 323 Euler angle.
    pub thetaa: f64,
    /// The general precession in longitude.
    pub pa: f64,
    /// The Fukushima-Williams angle γ.
    pub gam: f64,
    /// The Fukushima-Williams angle φ.
    pub phi: f64,
    /// The Fukushima-Williams angle ψ.
    pub psi: f64,
}

/// A polynomial in `t` with ERFA's grouping: `(c0 + (c1 + (c2 + ...) t) t) t`
/// when `leading_t` is set (the angle vanishes at J2000.0), or the plain
/// Horner form when it is not, in arcseconds.
fn poly(t: f64, coefficients: &[f64], leading_t: bool) -> f64 {
    let mut value = 0.0;
    for c in coefficients.iter().rev() {
        value = c + value * t;
    }
    if leading_t { value * t } else { value }
}

/// The IAU 2006 precession angles (`eraP06e`).
#[must_use]
#[allow(
    clippy::too_many_lines,
    reason = "the sixteen angles of ERFA's eraP06e in one function, as the C"
)]
pub fn p06e(date1: f64, date2: f64) -> PrecessionAngles {
    let t = centuries(date1, date2);
    let eps0 = 84_381.406 * DAS2R;
    let psia = poly(
        t,
        &[
            5_038.481_507,
            -1.079_006_9,
            -0.001_140_45,
            0.000_132_851,
            -0.000_000_095_1,
        ],
        true,
    ) * DAS2R;
    let oma = eps0
        + poly(
            t,
            &[
                -0.025_754,
                0.051_262_3,
                -0.007_725_03,
                -0.000_000_467,
                0.000_000_333_7,
            ],
            true,
        ) * DAS2R;
    let bpa = poly(
        t,
        &[
            4.199_094,
            0.193_987_3,
            -0.000_224_66,
            -0.000_000_912,
            0.000_000_012_0,
        ],
        true,
    ) * DAS2R;
    let bqa = poly(
        t,
        &[
            -46.811_015,
            0.051_028_3,
            0.000_524_13,
            -0.000_000_646,
            -0.000_000_017_2,
        ],
        true,
    ) * DAS2R;
    let pia = poly(
        t,
        &[
            46.998_973,
            -0.033_492_6,
            -0.000_125_59,
            0.000_000_113,
            -0.000_000_002_2,
        ],
        true,
    ) * DAS2R;
    let bpia = poly(
        t,
        &[
            629_546.793_6,
            -867.957_58,
            0.157_992,
            -0.000_537_1,
            -0.000_047_97,
            0.000_000_072,
        ],
        false,
    ) * DAS2R;
    let epsa = obl06(date1, date2);
    let chia = poly(
        t,
        &[
            10.556_403,
            -2.381_429_2,
            -0.001_211_97,
            0.000_170_663,
            -0.000_000_056_0,
        ],
        true,
    ) * DAS2R;
    let za = poly(
        t,
        &[
            -2.650_545,
            2_306.077_181,
            1.092_734_8,
            0.018_268_37,
            -0.000_028_596,
            -0.000_000_290_4,
        ],
        false,
    ) * DAS2R;
    let zetaa = poly(
        t,
        &[
            2.650_545,
            2_306.083_227,
            0.298_849_9,
            0.018_018_28,
            -0.000_005_971,
            -0.000_000_317_3,
        ],
        false,
    ) * DAS2R;
    let thetaa = poly(
        t,
        &[
            2_004.191_903,
            -0.429_493_4,
            -0.041_822_64,
            -0.000_007_089,
            -0.000_000_127_4,
        ],
        true,
    ) * DAS2R;
    let pa = poly(
        t,
        &[
            5_028.796_195,
            1.105_434_8,
            0.000_079_64,
            -0.000_023_857,
            -0.000_000_038_3,
        ],
        true,
    ) * DAS2R;
    let gam = poly(
        t,
        &[
            10.556_403,
            0.493_204_4,
            -0.000_312_38,
            -0.000_002_788,
            0.000_000_026_0,
        ],
        true,
    ) * DAS2R;
    let phi = eps0
        + poly(
            t,
            &[
                -46.811_015,
                0.051_126_9,
                0.000_532_89,
                -0.000_000_440,
                -0.000_000_017_6,
            ],
            true,
        ) * DAS2R;
    let psi = poly(
        t,
        &[
            5_038.481_507,
            1.558_417_6,
            -0.000_185_22,
            -0.000_026_452,
            -0.000_000_014_8,
        ],
        true,
    ) * DAS2R;
    PrecessionAngles {
        eps0,
        psia,
        oma,
        bpa,
        bqa,
        pia,
        bpia,
        epsa,
        chia,
        za,
        zetaa,
        thetaa,
        pa,
        gam,
        phi,
        psi,
    }
}

/// The IAU 2006 bias-precession Fukushima-Williams angles at a TT date
/// (`eraPfw06`): γ̄, φ̄, ψ̄ and the mean obliquity, radians.
#[must_use]
pub fn pfw06(date1: f64, date2: f64) -> (f64, f64, f64, f64) {
    let t = centuries(date1, date2);
    let gamb = poly(
        t,
        &[
            -0.052_928,
            10.556_378,
            0.493_204_4,
            -0.000_312_38,
            -0.000_002_788,
            0.000_000_026_0,
        ],
        false,
    ) * DAS2R;
    let phib = poly(
        t,
        &[
            84_381.412_819,
            -46.811_016,
            0.051_126_8,
            0.000_532_89,
            -0.000_000_440,
            -0.000_000_017_6,
        ],
        false,
    ) * DAS2R;
    let psib = poly(
        t,
        &[
            -0.041_775,
            5_038.481_484,
            1.558_417_5,
            -0.000_185_22,
            -0.000_026_452,
            -0.000_000_014_8,
        ],
        false,
    ) * DAS2R;
    (gamb, phib, psib, obl06(date1, date2))
}

/// A rotation matrix from Fukushima-Williams angles (`eraFw2m`): the
/// four rotations in ERFA's order.
#[must_use]
pub fn fw2m(gamb: f64, phib: f64, psi: f64, eps: f64) -> Matrix3 {
    let mut r = ir();
    rz(gamb, &mut r);
    rx(phib, &mut r);
    rz(-psi, &mut r);
    rx(-eps, &mut r);
    r
}

/// The IAU 2006 bias-precession matrix, GCRS to mean equator and equinox
/// of date (`eraPmat06`).
#[must_use]
pub fn pmat06(date1: f64, date2: f64) -> Matrix3 {
    let (gamb, phib, psib, epsa) = pfw06(date1, date2);
    fw2m(gamb, phib, psib, epsa)
}

/// The frame bias matrix, the precession matrix and their product
/// (`eraBp06`): `rb` GCRS to J2000.0, `rp` J2000.0 to mean of date, `rbp`
/// GCRS to mean of date.
#[must_use]
pub fn bp06(date1: f64, date2: f64) -> (Matrix3, Matrix3, Matrix3) {
    // The bias matrix is the bias-precession matrix at J2000.0 itself.
    let (gamb, phib, psib, epsa) = pfw06(DJM0, DJM00);
    let rb = fw2m(gamb, phib, psib, epsa);
    let rbpw = pmat06(date1, date2);
    let rp = rxr(&rbpw, &tr(&rb));
    (rb, rp, rbpw)
}

/// The frame bias components of the IAU 2000 precession-nutation models
/// (`eraBi00`): the corrections in longitude and obliquity and the ICRS
/// right ascension of the J2000.0 equinox, radians.
#[must_use]
pub const fn bi00() -> (f64, f64, f64) {
    (-0.041_775 * DAS2R, -0.006_819_2 * DAS2R, -0.0146 * DAS2R)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::excessive_precision,
        reason = "tests fail by panicking, and the reference values are quoted as ERFA prints them"
    )]

    use super::*;

    fn vvd(value: f64, expected: f64, tolerance: f64, name: &str) {
        assert!(
            (value - expected).abs() <= tolerance,
            "{name}: {value} against {expected}"
        );
    }

    #[test]
    fn p06e_reproduces_erfa() {
        let a = p06e(2_400_000.5, 52_541.0);
        vvd(a.eps0, 0.409_092_600_600_582_871_5, 1e-14, "eps0");
        vvd(a.psia, 0.666_436_963_019_161_343_1e-3, 1e-14, "psia");
        vvd(a.oma, 0.409_092_597_378_325_598_2, 1e-14, "oma");
        vvd(a.bpa, 0.556_114_937_126_520_944_5e-6, 1e-14, "bpa");
        vvd(a.bqa, -0.619_151_719_329_062_127_0e-5, 1e-14, "bqa");
        vvd(a.pia, 0.621_644_175_188_438_292_3e-5, 1e-14, "pia");
        vvd(a.bpia, 3.052_014_180_023_779_882, 1e-14, "bpia");
        vvd(a.epsa, 0.409_086_405_492_243_168_8, 1e-14, "epsa");
        vvd(a.chia, 0.138_770_337_953_091_536_4e-5, 1e-14, "chia");
        vvd(a.za, 0.292_178_984_665_179_054_6e-3, 1e-14, "za");
        vvd(a.zetaa, 0.317_877_329_033_200_931_0e-3, 1e-14, "zetaa");
        vvd(a.thetaa, 0.265_093_270_165_749_718_1e-3, 1e-14, "thetaa");
        vvd(a.pa, 0.665_163_768_138_101_628_8e-3, 1e-14, "pa");
        vvd(a.gam, 0.139_807_711_596_375_498_7e-5, 1e-14, "gam");
        vvd(a.phi, 0.409_086_409_083_746_260_2, 1e-14, "phi");
        vvd(a.psi, 0.666_446_480_748_092_032_5e-3, 1e-14, "psi");
    }

    #[test]
    fn pfw06_fw2m_pmat06_and_bp06_reproduce_erfa() {
        let (gamb, phib, psib, epsa) = pfw06(2_400_000.5, 50_123.999_9);
        vvd(gamb, -0.224_338_767_099_799_569_0e-5, 1e-16, "gamb");
        vvd(phib, 0.409_101_460_239_131_280_8, 1e-12, "phib");
        vvd(psib, -0.950_195_417_801_303_189_5e-3, 1e-14, "psib");
        vvd(epsa, 0.409_101_431_658_736_749_1, 1e-12, "epsa");
        // eraFw2m with gamb = -0.2243387670997992368e-5, phib = 0.4091014602391312982,
        // psi = -0.9501954178013015092e-3, eps = 0.4091014316587367472.
        let r = fw2m(
            -0.224_338_767_099_799_236_8e-5,
            0.409_101_460_239_131_298_2,
            -0.950_195_417_801_301_509_2e-3,
            0.409_101_431_658_736_747_2,
        );
        vvd(r[0][0], 0.999_999_550_517_600_704_7, 1e-12, "fw2m 11");
        vvd(r[0][1], 0.869_540_461_734_819_295_7e-3, 1e-12, "fw2m 12");
        vvd(r[0][2], 0.377_973_520_186_558_257_1e-3, 1e-12, "fw2m 13");
        vvd(r[1][0], -0.869_540_472_377_201_603_8e-3, 1e-12, "fw2m 21");
        vvd(r[1][1], 0.999_999_621_949_602_716_1, 1e-12, "fw2m 22");
        vvd(r[1][2], -0.136_175_249_688_710_002_6e-6, 1e-12, "fw2m 23");
        vvd(r[2][0], -0.377_973_495_703_408_279_0e-3, 1e-12, "fw2m 31");
        vvd(r[2][1], -0.192_488_084_808_761_565_1e-6, 1e-12, "fw2m 32");
        vvd(r[2][2], 0.999_999_928_567_997_195_8, 1e-12, "fw2m 33");
        let rbp = pmat06(2_400_000.5, 50_123.999_9);
        vvd(rbp[0][0], 0.999_999_550_517_600_704_7, 1e-12, "pmat06 11");
        vvd(
            rbp[0][1],
            0.869_540_461_734_820_840_6e-3,
            1e-14,
            "pmat06 12",
        );
        vvd(
            rbp[0][2],
            0.377_973_520_186_558_910_4e-3,
            1e-14,
            "pmat06 13",
        );
        vvd(
            rbp[1][0],
            -0.869_540_472_377_203_141_4e-3,
            1e-14,
            "pmat06 21",
        );
        vvd(rbp[1][1], 0.999_999_621_949_602_716_1, 1e-12, "pmat06 22");
        vvd(
            rbp[1][2],
            -0.136_175_249_708_027_014_3e-6,
            1e-14,
            "pmat06 23",
        );
        vvd(
            rbp[2][0],
            -0.377_973_495_703_408_949_0e-3,
            1e-14,
            "pmat06 31",
        );
        vvd(
            rbp[2][1],
            -0.192_488_084_789_445_711_3e-6,
            1e-14,
            "pmat06 32",
        );
        vvd(rbp[2][2], 0.999_999_928_567_997_195_8, 1e-12, "pmat06 33");
    }

    #[test]
    fn bp06_and_bi00_reproduce_erfa() {
        let rbp = pmat06(2_400_000.5, 50_123.999_9);
        let (rb, rp, rbp2) = bp06(2_400_000.5, 50_123.999_9);
        vvd(rb[0][0], 0.999_999_999_999_994_249_7, 1e-12, "bp06 rb11");
        vvd(
            rb[0][1],
            -0.707_836_896_097_155_714_5e-7,
            1e-14,
            "bp06 rb12",
        );
        vvd(rb[0][2], 0.805_621_397_761_318_560_6e-7, 1e-14, "bp06 rb13");
        vvd(rb[1][0], 0.707_836_869_463_767_433_3e-7, 1e-14, "bp06 rb21");
        vvd(rb[1][1], 0.999_999_999_999_996_948_4, 1e-12, "bp06 rb22");
        vvd(rb[1][2], 0.330_594_374_298_913_412_4e-7, 1e-14, "bp06 rb23");
        vvd(
            rb[2][0],
            -0.805_621_421_162_005_679_2e-7,
            1e-14,
            "bp06 rb31",
        );
        vvd(
            rb[2][1],
            -0.330_594_317_274_058_695_0e-7,
            1e-14,
            "bp06 rb32",
        );
        vvd(rb[2][2], 0.999_999_999_999_996_208_4, 1e-12, "bp06 rb33");
        vvd(rp[0][0], 0.999_999_550_486_496_027_8, 1e-12, "bp06 rp11");
        vvd(rp[0][1], 0.869_611_257_885_540_483_2e-3, 1e-14, "bp06 rp12");
        vvd(rp[0][2], 0.377_892_929_334_139_012_7e-3, 1e-14, "bp06 rp13");
        vvd(
            rp[1][0],
            -0.869_611_256_051_018_624_4e-3,
            1e-14,
            "bp06 rp21",
        );
        vvd(rp[1][1], 0.999_999_621_888_045_882_0, 1e-12, "bp06 rp22");
        vvd(
            rp[1][2],
            -0.169_164_616_894_189_628_5e-6,
            1e-14,
            "bp06 rp23",
        );
        vvd(
            rp[2][0],
            -0.377_892_933_555_760_341_8e-3,
            1e-14,
            "bp06 rp31",
        );
        vvd(
            rp[2][1],
            -0.159_455_404_078_649_507_6e-6,
            1e-14,
            "bp06 rp32",
        );
        vvd(rp[2][2], 0.999_999_928_598_450_122_2, 1e-12, "bp06 rp33");
        assert_eq!(rbp2, rbp);
        let (dpsibi, depsbi, dra) = bi00();
        vvd(
            dpsibi,
            -0.202_530_912_070_404_669_7e-6,
            1e-12,
            "bi00 dpsibi",
        );
        vvd(
            depsbi,
            -0.330_602_244_063_034_636_2e-7,
            1e-12,
            "bi00 depsbi",
        );
        vvd(dra, -0.707_827_432_878_226_069_6e-7, 1e-12, "bi00 dra");
    }
}
