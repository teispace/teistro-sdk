//! The vector and matrix primitives the IAU routines are written in, ported
//! from ERFA one for one so the precession code reads beside the C: a
//! position is `[f64; 3]`, a rotation `[[f64; 3]; 3]` in row-major order,
//! angles in radians. Nothing here allocates.

use core::f64::consts::PI;

use super::D2PI;

/// A Cartesian vector.
pub type Vector3 = [f64; 3];

/// A rotation matrix, row-major.
pub type Matrix3 = [[f64; 3]; 3];

/// The identity matrix (`eraIr`).
#[must_use]
pub const fn ir() -> Matrix3 {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

/// Rotates the matrix about the x axis by `phi` (`eraRx`): the rotation
/// is applied after the matrix's own, as ERFA composes them.
pub fn rx(phi: f64, r: &mut Matrix3) {
    let (s, c) = phi.sin_cos();
    let a10 = c * r[1][0] + s * r[2][0];
    let a11 = c * r[1][1] + s * r[2][1];
    let a12 = c * r[1][2] + s * r[2][2];
    let a20 = -s * r[1][0] + c * r[2][0];
    let a21 = -s * r[1][1] + c * r[2][1];
    let a22 = -s * r[1][2] + c * r[2][2];
    r[1] = [a10, a11, a12];
    r[2] = [a20, a21, a22];
}

/// Rotates the matrix about the y axis by `theta` (`eraRy`).
pub fn ry(theta: f64, r: &mut Matrix3) {
    let (s, c) = theta.sin_cos();
    let a00 = c * r[0][0] - s * r[2][0];
    let a01 = c * r[0][1] - s * r[2][1];
    let a02 = c * r[0][2] - s * r[2][2];
    let a20 = s * r[0][0] + c * r[2][0];
    let a21 = s * r[0][1] + c * r[2][1];
    let a22 = s * r[0][2] + c * r[2][2];
    r[0] = [a00, a01, a02];
    r[2] = [a20, a21, a22];
}

/// Rotates the matrix about the z axis by `psi` (`eraRz`).
pub fn rz(psi: f64, r: &mut Matrix3) {
    let (s, c) = psi.sin_cos();
    let a00 = c * r[0][0] + s * r[1][0];
    let a01 = c * r[0][1] + s * r[1][1];
    let a02 = c * r[0][2] + s * r[1][2];
    let a10 = -s * r[0][0] + c * r[1][0];
    let a11 = -s * r[0][1] + c * r[1][1];
    let a12 = -s * r[0][2] + c * r[1][2];
    r[0] = [a00, a01, a02];
    r[1] = [a10, a11, a12];
}

/// The matrix product `a b` (`eraRxr`): each entry the scalar product of a
/// row of `a` with a column of `b`.
#[must_use]
pub fn rxr(a: &Matrix3, b: &Matrix3) -> Matrix3 {
    let columns = tr(b);
    let row = |r: &Vector3| {
        [
            pdp(r, &columns[0]),
            pdp(r, &columns[1]),
            pdp(r, &columns[2]),
        ]
    };
    [row(&a[0]), row(&a[1]), row(&a[2])]
}

/// The transpose (`eraTr`).
#[must_use]
pub const fn tr(r: &Matrix3) -> Matrix3 {
    [
        [r[0][0], r[1][0], r[2][0]],
        [r[0][1], r[1][1], r[2][1]],
        [r[0][2], r[1][2], r[2][2]],
    ]
}

/// The matrix times a vector (`eraRxp`).
#[must_use]
pub fn rxp(r: &Matrix3, p: &Vector3) -> Vector3 {
    [pdp(&r[0], p), pdp(&r[1], p), pdp(&r[2], p)]
}

/// The transposed matrix times a vector (`eraTrxp`).
#[must_use]
pub fn trxp(r: &Matrix3, p: &Vector3) -> Vector3 {
    rxp(&tr(r), p)
}

/// The vector product `a × b` (`eraPxp`).
#[must_use]
pub fn pxp(a: &Vector3, b: &Vector3) -> Vector3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// The scalar product (`eraPdp`).
#[must_use]
pub fn pdp(a: &Vector3, b: &Vector3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The modulus (`eraPm`).
#[must_use]
pub fn pm(p: &Vector3) -> f64 {
    (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()
}

/// A vector scaled (`eraSxp`).
#[must_use]
pub fn sxp(s: f64, p: &Vector3) -> Vector3 {
    [s * p[0], s * p[1], s * p[2]]
}

/// The unit vector and the modulus (`eraPn`); the zero vector stays zero.
#[must_use]
pub fn pn(p: &Vector3) -> (f64, Vector3) {
    let w = pm(p);
    if w == 0.0 {
        (0.0, [0.0; 3])
    } else {
        (w, sxp(1.0 / w, p))
    }
}

/// Cartesian to spherical (`eraC2s`): longitude and latitude in radians,
/// the longitude in `(-π, π]`.
#[must_use]
pub fn c2s(p: &Vector3) -> (f64, f64) {
    let d2 = p[0] * p[0] + p[1] * p[1];
    let theta = if d2 == 0.0 { 0.0 } else { p[1].atan2(p[0]) };
    let phi = if p[2] == 0.0 {
        0.0
    } else {
        p[2].atan2(d2.sqrt())
    };
    (theta, phi)
}

/// Spherical to a unit Cartesian vector (`eraS2c`).
#[must_use]
pub fn s2c(theta: f64, phi: f64) -> Vector3 {
    let cp = phi.cos();
    [theta.cos() * cp, theta.sin() * cp, phi.sin()]
}

/// An angle into `(-π, π]` (`eraAnpm`).
#[must_use]
pub fn anpm(a: f64) -> f64 {
    let mut w = a % D2PI;
    if w.abs() >= PI {
        w -= D2PI.copysign(a);
    }
    w
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

    const R: Matrix3 = [[2.0, 3.0, 2.0], [3.0, 2.0, 3.0], [3.0, 4.0, 5.0]];

    #[test]
    fn the_vector_helpers_reproduce_erfa_reference_values() {
        let (theta, phi) = c2s(&[100.0, -50.0, 25.0]);
        vvd(theta, -0.463_647_609_000_806_116_2, 1e-14, "c2s theta");
        vvd(phi, 0.219_987_977_395_459_446_3, 1e-14, "c2s phi");
        let c = s2c(3.0123, -0.999);
        vvd(c[0], -0.536_626_766_726_052_390_6, 1e-12, "s2c 1");
        vvd(c[1], 0.069_771_110_976_514_536_5, 1e-12, "s2c 2");
        vvd(c[2], -0.840_930_261_856_621_404_1, 1e-12, "s2c 3");
        vvd(anpm(-4.0), 2.283_185_307_179_586_477, 1e-12, "anpm");
        vvd(pdp(&[2.0, 2.0, 3.0], &[1.0, 3.0, 4.0]), 20.0, 1e-12, "pdp");
        // eraRxp and eraTrxp on ERFA's test matrix and p = (0.2, 1.5, 0.1).
        let p = [0.2, 1.5, 0.1];
        let rp = rxp(&R, &p);
        vvd(rp[0], 5.1, 1e-12, "rxp 1");
        vvd(rp[1], 3.9, 1e-12, "rxp 2");
        vvd(rp[2], 7.1, 1e-12, "rxp 3");
        let trp = trxp(&R, &p);
        vvd(trp[0], 5.2, 1e-12, "trxp 1");
        vvd(trp[1], 4.0, 1e-12, "trxp 2");
        vvd(trp[2], 5.4, 1e-12, "trxp 3");
        // eraPxp on a = (2, 2, 3), b = (1, 3, 4).
        let axb = pxp(&[2.0, 2.0, 3.0], &[1.0, 3.0, 4.0]);
        vvd(axb[0], -1.0, 1e-12, "pxp 1");
        vvd(axb[1], -5.0, 1e-12, "pxp 2");
        vvd(axb[2], 4.0, 1e-12, "pxp 3");
        // eraPn on (0.3, 1.2, -2.5).
        let (r, u) = pn(&[0.3, 1.2, -2.5]);
        vvd(r, 2.789_265_136_196_270_604, 1e-12, "pn r");
        vvd(u[0], 0.107_555_210_907_311_205_8, 1e-12, "pn u1");
        vvd(u[1], 0.430_220_843_629_244_823_2, 1e-12, "pn u2");
        vvd(u[2], -0.896_293_424_227_593_381_6, 1e-12, "pn u3");
        vvd(
            pm(&[0.3, 1.2, -2.5]),
            2.789_265_136_196_270_604,
            1e-12,
            "pm",
        );
    }

    #[test]
    fn the_rotations_reproduce_erfa_reference_values() {
        // eraRx by phi = 0.3456789 on the test matrix.
        let mut r = R;
        rx(0.345_678_9, &mut r);
        vvd(r[0][0], 2.0, 0.0, "rx 11");
        vvd(r[1][0], 3.839_043_388_235_612_460, 1e-12, "rx 21");
        vvd(r[1][1], 3.237_033_249_594_111_899, 1e-12, "rx 22");
        vvd(r[1][2], 4.516_714_379_005_982_719, 1e-12, "rx 23");
        vvd(r[2][0], 1.806_030_415_924_501_684, 1e-12, "rx 31");
        vvd(r[2][1], 3.085_711_545_336_372_503, 1e-12, "rx 32");
        vvd(r[2][2], 3.687_721_683_977_873_065, 1e-12, "rx 33");
        // eraRy by theta = 0.3456789.
        let mut r = R;
        ry(0.345_678_9, &mut r);
        vvd(r[0][0], 0.865_184_781_897_815_993_0, 1e-12, "ry 11");
        vvd(r[0][1], 1.467_194_920_539_316_554, 1e-12, "ry 12");
        vvd(r[0][2], 0.187_513_791_127_445_734_2, 1e-12, "ry 13");
        vvd(r[2][0], 3.500_207_892_850_427_330, 1e-12, "ry 31");
        vvd(r[2][1], 4.779_889_022_262_298_150, 1e-12, "ry 32");
        vvd(r[2][2], 5.381_899_160_903_798_712, 1e-12, "ry 33");
        // eraRz by psi = 0.3456789.
        let mut r = R;
        rz(0.345_678_9, &mut r);
        vvd(r[0][0], 2.898_197_754_208_926_769, 1e-12, "rz 11");
        vvd(r[0][1], 3.500_207_892_850_427_330, 1e-12, "rz 12");
        vvd(r[0][2], 2.898_197_754_208_926_769, 1e-12, "rz 13");
        vvd(r[1][0], 2.144_865_911_309_686_813, 1e-12, "rz 21");
        vvd(r[1][1], 0.865_184_781_897_815_993, 1e-12, "rz 22");
        vvd(r[1][2], 2.144_865_911_309_686_813, 1e-12, "rz 23");
        // eraRxr and eraTr.
        let b: Matrix3 = [[1.0, 2.0, 2.0], [4.0, 1.0, 1.0], [3.0, 0.0, 1.0]];
        let atb = rxr(&R, &b);
        vvd(atb[0][0], 20.0, 1e-12, "rxr 11");
        vvd(atb[0][1], 7.0, 1e-12, "rxr 12");
        vvd(atb[0][2], 9.0, 1e-12, "rxr 13");
        vvd(atb[2][2], 15.0, 1e-12, "rxr 33");
        let rt = tr(&R);
        assert_eq!(rt, [[2.0, 3.0, 3.0], [3.0, 2.0, 4.0], [2.0, 3.0, 5.0]]);
        assert_eq!(ir(), [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    }
}
