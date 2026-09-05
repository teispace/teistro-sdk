//! Long-term precession (Vondrák, Capitaine and Wallace 2011, A&A 534, A22),
//! valid over two hundred millennia either side of J2000.0: the ecliptic
//! and equator poles as periodic and polynomial series, the precession
//! matrix assembled from them, and the same with the frame bias, ported
//! from ERFA; and the paper's own series for the general precession in
//! longitude and the obliquity of the ecliptic (its equations 10 and
//! Table 5), which ERFA does not carry and the ayanamsha catalogue needs
//! for the obliquity consistent with this precession.

use super::vector::{Matrix3, Vector3, pn, pxp};
use super::{D2PI, DAS2R};

/// Julian centuries since J2000.0 from a Julian epoch.
fn centuries_from_epoch(epj: f64) -> f64 {
    (epj - 2000.0) / 100.0
}

/// Sums a Vondrák series: periodic terms `(period, c1, c2, s1, s2)` and
/// polynomials in `t` for two components, arcseconds.
fn series(t: f64, periodic: &[[f64; 5]], polynomial: [&[f64]; 2]) -> (f64, f64) {
    let turn = D2PI * t;
    let mut first = 0.0;
    let mut second = 0.0;
    for term in periodic {
        let (sine, cosine) = (turn / term[0]).sin_cos();
        first += cosine * term[1] + sine * term[3];
        second += cosine * term[2] + sine * term[4];
    }
    let mut power = 1.0;
    for i in 0..polynomial[0].len().max(polynomial[1].len()) {
        first += polynomial[0].get(i).copied().unwrap_or(0.0) * power;
        second += polynomial[1].get(i).copied().unwrap_or(0.0) * power;
        power *= t;
    }
    (first * DAS2R, second * DAS2R)
}

/// The ecliptic pole as a unit vector in the J2000.0 mean equatorial triad
/// at a Julian epoch (`eraLtpecl`).
#[must_use]
pub fn ltpecl(epj: f64) -> Vector3 {
    const EPS0: f64 = 84_381.406 * DAS2R;
    const PERIODIC: [[f64; 5]; 8] = [
        [
            708.15,
            -5_486.751_211,
            -684.661_560,
            667.666_730,
            -5_523.863_691,
        ],
        [
            2309.00,
            -17.127_623,
            2_446.283_880,
            -2_354.886_252,
            -549.747_450,
        ],
        [
            1620.00,
            -617.517_403,
            399.671_049,
            -428.152_441,
            -310.998_056,
        ],
        [492.20, 413.442_940, -356.652_376, 376.202_861, 421.535_876],
        [1183.00, 78.614_193, -186.387_003, 184.778_874, -36.776_172],
        [
            622.00,
            -180.732_815,
            -316.800_070,
            335.321_713,
            -145.278_396,
        ],
        [882.00, -87.676_083, 198.296_701, -185.138_669, -34.744_450],
        [547.00, 46.140_315, 101.135_679, -120.972_830, 22.885_731],
    ];
    let t = centuries_from_epoch(epj);
    let (p, q) = series(
        t,
        &PERIODIC,
        [
            &[5_851.607_687, -0.118_900_0, -0.000_289_13, 0.000_000_101],
            &[-1_600.886_300, 1.168_981_8, -0.000_000_20, -0.000_000_437],
        ],
    );
    let third = 1.0 - p * p - q * q;
    let third = if third < 0.0 { 0.0 } else { third.sqrt() };
    let (sine, cosine) = EPS0.sin_cos();
    [p, -q * cosine - third * sine, -q * sine + third * cosine]
}

/// The equator pole as a unit vector in the J2000.0 mean equatorial triad
/// at a Julian epoch (`eraLtpequ`).
#[must_use]
pub fn ltpequ(epj: f64) -> Vector3 {
    const PERIODIC: [[f64; 5]; 14] = [
        [
            256.75,
            -819.940_624,
            75_004.344_875,
            81_491.287_984,
            1_558.515_853,
        ],
        [
            708.15,
            -8_444.676_815,
            624.033_993,
            787.163_481,
            7_774.939_698,
        ],
        [
            274.20,
            2_600.009_459,
            1_251.136_893,
            1_251.296_102,
            -2_219.534_038,
        ],
        [
            241.45,
            2_755.175_630,
            -1_102.212_834,
            -1_257.950_837,
            -2_523.969_396,
        ],
        [
            2309.00,
            -167.659_835,
            -2_660.664_980,
            -2_966.799_730,
            247.850_422,
        ],
        [492.20, 871.855_056, 699.291_817, 639.744_522, -846.485_643],
        [396.10, 44.769_698, 153.167_220, 131.600_209, -1_393.124_055],
        [
            288.90,
            -512.313_065,
            -950.865_637,
            -445.040_117,
            368.526_116,
        ],
        [231.10, -819.415_595, 499.754_645, 584.522_874, 749.045_012],
        [
            1610.00,
            -538.071_099,
            -145.188_210,
            -89.756_563,
            444.704_518,
        ],
        [620.00, -189.793_622, 558.116_553, 524.429_630, 235.934_465],
        [157.87, -402.922_932, -23.923_029, -13.549_067, 374.049_623],
        [
            220.30,
            179.516_345,
            -165.405_086,
            -210.157_124,
            -171.330_180,
        ],
        [1200.00, -9.814_756, 9.344_131, -44.919_798, -22.899_655],
    ];
    let t = centuries_from_epoch(epj);
    let (x, y) = series(
        t,
        &PERIODIC,
        [
            &[5_453.282_155, 0.425_284_1, -0.000_371_73, -0.000_000_152],
            &[-73_750.930_350, -0.767_545_2, -0.000_187_25, 0.000_000_231],
        ],
    );
    let third = 1.0 - x * x - y * y;
    [x, y, if third < 0.0 { 0.0 } else { third.sqrt() }]
}

/// The long-term precession matrix, J2000.0 mean equator and equinox to
/// the mean of a Julian epoch (`eraLtp`).
#[must_use]
pub fn ltp(epj: f64) -> Matrix3 {
    let peqr = ltpequ(epj);
    let pecl = ltpecl(epj);
    let (_, eqx) = pn(&pxp(&peqr, &pecl));
    let v = pxp(&peqr, &eqx);
    [eqx, v, peqr]
}

/// The long-term precession matrix with the frame bias, GCRS to the mean
/// of a Julian epoch (`eraLtpb`).
#[must_use]
pub fn ltpb(epj: f64) -> Matrix3 {
    const DX: f64 = -0.016_617 * DAS2R;
    const DE: f64 = -0.006_819_2 * DAS2R;
    const DR: f64 = -0.0146 * DAS2R;
    let rp = ltp(epj);
    let mut rpb = [[0.0; 3]; 3];
    for (out, row) in rpb.iter_mut().zip(rp.iter()) {
        *out = [
            row[0] - row[1] * DR + row[2] * DX,
            row[0] * DR + row[1] + row[2] * DE,
            -row[0] * DX - row[1] * DE + row[2],
        ];
    }
    rpb
}

/// The long-term general precession in longitude and the mean obliquity of
/// the ecliptic at a Julian epoch, radians: Vondrák, Capitaine and Wallace
/// (2011), equations 10 with Table 5 (a cubic and ten periodic terms each).
/// At J2000.0 the obliquity is the IAU 2006 value, 84381.406″.
#[must_use]
pub fn ltpeps(epj: f64) -> (f64, f64) {
    const PERIODIC: [[f64; 5]; 10] = [
        [
            409.90,
            -6_908.287_473,
            753.872_780,
            -2_845.175_469,
            -1_704.720_302,
        ],
        [
            396.15,
            -3_198.706_291,
            -247.805_823,
            449.844_989,
            -862.308_358,
        ],
        [
            537.22,
            1_453.674_527,
            379.471_484,
            -1_255.915_323,
            447.832_178,
        ],
        [402.90, -857.748_557, -53.880_558, 886.736_783, -889.571_909],
        [417.15, 1_173.231_614, -90.109_153, 418.887_514, 190.402_846],
        [288.92, -156.981_465, -353.600_190, 997.912_441, -56.564_991],
        [
            4043.00,
            371.836_550,
            -63.115_353,
            -240.979_710,
            -296.222_622,
        ],
        [306.00, -216.619_040, -28.248_187, 76.541_307, -75.859_952],
        [277.00, 193.691_479, 17.703_387, -36.788_069, 67.473_503],
        [203.00, 11.891_524, 38.911_307, -170.964_086, 3.014_055],
    ];
    let t = centuries_from_epoch(epj);
    series(
        t,
        &PERIODIC,
        [
            &[8_134.017_132, 5_043.052_003_5, -0.007_107_33, 0.000_000_271],
            &[84_028.206_305, 0.362_444_5, -0.000_040_39, -0.000_000_11],
        ],
    )
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
    fn the_long_term_precession_reproduces_erfa() {
        // ERFA's test epochs: -1500.0 for the ecliptic pole, -2500.0 for
        // the equator pole, 1666.666 for the matrices.
        let vec = ltpecl(-1500.0);
        vvd(vec[0], 0.476_862_567_647_709_652_5e-3, 1e-14, "ltpecl 1");
        vvd(vec[1], -0.405_225_953_309_187_511_2, 1e-14, "ltpecl 2");
        vvd(vec[2], 0.914_216_440_109_644_801_2, 1e-14, "ltpecl 3");
        let veq = ltpequ(-2500.0);
        vvd(veq[0], -0.358_665_256_023_732_665_9, 1e-14, "ltpequ 1");
        vvd(veq[1], -0.199_697_891_077_112_847_5, 1e-14, "ltpequ 2");
        vvd(veq[2], 0.911_855_244_225_081_962_4, 1e-14, "ltpequ 3");
        let epj = 1666.666;
        let rp = ltp(epj);
        vvd(rp[0][0], 0.996_704_414_115_921_381_9, 1e-14, "ltp 11");
        vvd(rp[0][1], 0.743_780_189_319_321_084_0e-1, 1e-14, "ltp 12");
        vvd(rp[0][2], 0.323_762_440_934_560_340_1e-1, 1e-14, "ltp 13");
        vvd(rp[1][0], -0.743_780_273_181_961_816_7e-1, 1e-14, "ltp 21");
        vvd(rp[1][1], 0.997_229_389_445_453_307_0, 1e-14, "ltp 22");
        vvd(rp[1][2], -0.120_576_884_272_359_334_6e-2, 1e-14, "ltp 23");
        vvd(rp[2][0], -0.323_762_248_276_657_539_9e-1, 1e-14, "ltp 31");
        vvd(rp[2][1], -0.120_628_603_969_760_900_8e-2, 1e-14, "ltp 32");
        vvd(rp[2][2], 0.999_475_024_670_401_091_4, 1e-14, "ltp 33");
        let rpb = ltpb(epj);
        vvd(rpb[0][0], 0.996_704_416_772_327_185_1, 1e-14, "ltpb 11");
        vvd(rpb[0][1], 0.743_779_473_120_334_034_5e-1, 1e-14, "ltpb 12");
        vvd(rpb[0][2], 0.323_763_268_484_162_554_7e-1, 1e-14, "ltpb 13");
        vvd(rpb[1][0], -0.743_779_566_343_717_715_2e-1, 1e-14, "ltpb 21");
        vvd(rpb[1][1], 0.997_229_394_750_001_366_6, 1e-14, "ltpb 22");
        vvd(rpb[1][2], -0.120_574_186_591_124_323_5e-2, 1e-14, "ltpb 23");
        vvd(rpb[2][0], -0.323_763_054_322_466_499_2e-1, 1e-14, "ltpb 31");
        vvd(rpb[2][1], -0.120_631_679_107_648_529_5e-2, 1e-14, "ltpb 32");
        vvd(rpb[2][2], 0.999_475_022_022_243_881_9, 1e-14, "ltpb 33");
    }

    #[test]
    fn the_obliquity_series_gives_the_iau_2006_value_at_j2000_and_tracks_it_nearby() {
        // The periodic terms sum to minus the constants at J2000.0, so the
        // general precession is zero there and the obliquity the IAU 2006 value.
        // the published coefficients carry six decimals, so to a microarcsecond.
        let (pa, eps) = ltpeps(2000.0);
        vvd(eps, 84_381.406 * DAS2R, 1e-5 * DAS2R, "eps at J2000");
        vvd(pa, 0.0, 1e-5 * DAS2R, "pA at J2000");
        // Against IAU 2006 over a millennium either side: within 0.05″.
        for years in [-1000.0, -500.0, 0.0, 500.0, 1000.0] {
            let epj = 2000.0 + years;
            let (_, eps) = ltpeps(epj);
            let iau = super::super::obl06(super::super::DJ00, years * 365.25);
            vvd(eps / DAS2R, iau / DAS2R, 0.05, "eps against IAU 2006");
        }
        // At J2000 the matrix is the identity to the bias-free rounding.
        let rp = ltp(2000.0);
        for (i, row) in rp.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                let expected = if i == j { 1.0 } else { 0.0 };
                vvd(*cell, expected, 3e-8, "ltp at J2000");
            }
        }
    }
}
