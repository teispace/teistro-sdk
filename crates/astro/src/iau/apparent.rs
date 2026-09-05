//! The corrections that take a catalogue direction to an apparent one,
//! ported from ERFA: proper motion and parallax (`eraPmpx`), the light
//! deflection by the Sun (`eraLd`, `eraLdsun`), the aberration (`eraAb`)
//! and the nutation matrix (`eraNumat`). Directions are unit vectors in
//! the BCRS until the bias-precession-nutation rotation takes them to the
//! equator of date.

use super::vector::{Matrix3, Vector3, ir, pdp, pn, pxp, rx, rz};
use super::{AULT, DAS2R, DAU, DAYSEC, DJM, DJY, SRS};

/// Km/s to au/year.
const VF: f64 = DAYSEC * DJM / DAU;

/// The light time for one au, Julian years.
const AULTY: f64 = AULT / DAYSEC / DJY;

/// Proper motion and parallax (`eraPmpx`): the catalogue direction
/// `rc, dc` (ICRS right ascension and declination, radians) with its
/// proper motions `pr` (dRA/dt, not multiplied by cos δ, radians a year)
/// and `pd` (radians a year), parallax `px` (arcseconds), radial velocity
/// `rv` (km/s, positive receding), carried `pmt` Julian years from the
/// catalogue epoch and seen from the observer's barycentric position `pob`
/// (au); returns the coordinate direction as a BCRS unit vector. A star
/// with no parallax is placed at infinity, its radial velocity idle.
#[must_use]
#[allow(
    clippy::many_single_char_names,
    clippy::too_many_arguments,
    reason = "ERFA's own names and signature, kept so the C and the Rust read side by side"
)]
pub fn pmpx(
    rc: f64,
    dc: f64,
    pr: f64,
    pd: f64,
    px: f64,
    rv: f64,
    pmt: f64,
    pob: &Vector3,
) -> Vector3 {
    let (sr, cr) = rc.sin_cos();
    let (sd, cd) = dc.sin_cos();
    let x = cr * cd;
    let y = sr * cd;
    let z = sd;
    let p = [x, y, z];

    // Proper motion time interval, years, including the Roemer effect.
    let dt = pmt + pdp(&p, pob) * AULTY;

    // Space motion, radians a year.
    let pxr = px * DAS2R;
    let w = VF * rv * pxr;
    let pdz = pd * z;
    let pm = [
        -pr * y - pdz * cr + w * x,
        pr * x - pdz * sr + w * y,
        pd * cd + w * z,
    ];

    // Coordinate direction of the star, with the proper motion applied and
    // the parallax taken off.
    let [pob_x, pob_y, pob_z] = *pob;
    let [pm_x, pm_y, pm_z] = pm;
    let moved = [
        x + dt * pm_x - pxr * pob_x,
        y + dt * pm_y - pxr * pob_y,
        z + dt * pm_z - pxr * pob_z,
    ];
    pn(&moved).1
}

/// The gravitational deflection of light by a solar-system body
/// (`eraLd`): `bm` the body's mass in solar masses, `p` the direction from
/// the observer to the source (unit), `q` the direction from the body to
/// the source (unit), `e` the direction from the body to the observer
/// (unit), `em` the body-to-observer distance (au) and `dlim` the
/// deflection limiter; returns the deflected direction.
#[must_use]
pub fn ld(bm: f64, p: &Vector3, q: &Vector3, e: &Vector3, em: f64, dlim: f64) -> Vector3 {
    // q . (q + e).
    let [q_x, q_y, q_z] = *q;
    let [e_x, e_y, e_z] = *e;
    let qpe = [q_x + e_x, q_y + e_y, q_z + e_z];
    let qdqpe = pdp(q, &qpe);

    // 2 x G x bm / ( em x c^2 x ( q . (q + e) ) ).
    let w = bm * SRS / em / qdqpe.max(dlim);

    // p x (e x q).
    let eq = pxp(e, q);
    let peq = pxp(p, &eq);

    // Apply the deflection.
    let [p_x, p_y, p_z] = *p;
    let [peq_x, peq_y, peq_z] = peq;
    [p_x + w * peq_x, p_y + w * peq_y, p_z + w * peq_z]
}

/// The light deflection by the Sun (`eraLdsun`) for a source at infinity:
/// `p` the direction from the observer to the source (unit), `e` the
/// direction from the Sun to the observer (unit), `em` the Sun-to-observer
/// distance (au); the limiter keeps a source behind the Sun finite.
#[must_use]
pub fn ldsun(p: &Vector3, e: &Vector3, em: f64) -> Vector3 {
    let em2 = (em * em).max(1.0);
    let dlim = 1e-6 / em2;
    ld(1.0, p, p, e, em, dlim)
}

/// The stellar aberration (`eraAb`): `pnat` the natural direction to the
/// source (unit), `v` the observer's barycentric velocity in units of c,
/// `s` the observer's distance from the Sun (au) and `bm1` the reciprocal
/// Lorenz factor `√(1 − |v|²)`; returns the proper direction, with the
/// relativistic terms and the Sun's gravitational potential.
#[must_use]
pub fn ab(pnat: &Vector3, v: &Vector3, s: f64, bm1: f64) -> Vector3 {
    let pdv = pdp(pnat, v);
    let w1 = 1.0 + pdv / (1.0 + bm1);
    let w2 = SRS / s;
    let term = |p: f64, vi: f64| p * bm1 + w1 * vi + w2 * (vi - pdv * p);
    let [p_x, p_y, p_z] = *pnat;
    let [v_x, v_y, v_z] = *v;
    let p = [term(p_x, v_x), term(p_y, v_y), term(p_z, v_z)];
    pn(&p).1
}

/// The nutation matrix (`eraNumat`) from the mean obliquity `epsa` and the
/// nutation in longitude and obliquity `dpsi`, `deps`, radians: mean
/// equator and equinox of date to true.
#[must_use]
pub fn numat(epsa: f64, dpsi: f64, deps: f64) -> Matrix3 {
    let mut rmatn = ir();
    rx(epsa, &mut rmatn);
    rz(-dpsi, &mut rmatn);
    rx(-(epsa + deps), &mut rmatn);
    rmatn
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::excessive_precision,
        clippy::unreadable_literal,
        reason = "tests fail by panicking, and the reference values are quoted as ERFA prints them"
    )]

    use super::*;

    fn vvd(value: f64, expected: f64, tolerance: f64, name: &str) {
        assert!(
            (value - expected).abs() <= tolerance,
            "{name}: {value} against {expected}"
        );
    }

    /// The reference values of `t_pmpx`.
    #[test]
    fn pmpx_matches_erfa() {
        let pco = pmpx(
            1.234,
            0.789,
            1e-5,
            -2e-5,
            1e-2,
            10.0,
            8.75,
            &[0.9, 0.4, 0.1],
        );
        vvd(pco[0], 0.2328137623960308438, 1e-12, "1");
        vvd(pco[1], 0.6651097085397855328, 1e-12, "2");
        vvd(pco[2], 0.7095257765896359837, 1e-12, "3");
    }

    /// The reference values of `t_ab`.
    #[test]
    fn ab_matches_erfa() {
        let pnat = [
            -0.76321968546737951,
            -0.60869453983060384,
            -0.21676408580639883,
        ];
        let v = [
            2.1044018893653786e-5,
            -8.9108923304429319e-5,
            -3.8633714797716569e-5,
        ];
        let ppr = ab(&pnat, &v, 0.99980921395708788, 0.99999999506209258);
        vvd(ppr[0], -0.7631631094219556269, 1e-12, "1");
        vvd(ppr[1], -0.6087553082505590832, 1e-12, "2");
        vvd(ppr[2], -0.2167926269368471279, 1e-12, "3");
    }

    /// The reference values of `t_ldsun` and `t_ld`.
    #[test]
    fn deflection_matches_erfa() {
        let p = [-0.763276255, -0.608633767, -0.216735543];
        let e = [-0.973644023, -0.20925523, -0.0907169552];
        let p1 = ldsun(&p, &e, 0.999809214);
        vvd(p1[0], -0.7632762580731413169, 1e-12, "ldsun 1");
        vvd(p1[1], -0.6086337635262647900, 1e-12, "ldsun 2");
        vvd(p1[2], -0.2167355419322321302, 1e-12, "ldsun 3");

        let e = [0.76700421, 0.605629598, 0.211937094];
        let p1 = ld(0.00028574, &p, &p, &e, 8.91276983, 3e-10);
        vvd(p1[0], -0.7632762548968159627, 1e-12, "ld 1");
        vvd(p1[1], -0.6086337670823762701, 1e-12, "ld 2");
        vvd(p1[2], -0.2167355431320546947, 1e-12, "ld 3");
    }

    /// The reference values of `t_numat`.
    #[test]
    fn numat_matches_erfa() {
        let r = numat(
            0.4090789763356509900,
            -0.9630909107115582393e-5,
            0.4063239174001678826e-4,
        );
        vvd(r[0][0], 0.9999999999536227949, 1e-12, "11");
        vvd(r[0][1], 0.8836239320236250577e-5, 1e-12, "12");
        vvd(r[0][2], 0.3830833447458251908e-5, 1e-12, "13");
        vvd(r[1][0], -0.8836083657016688588e-5, 1e-12, "21");
        vvd(r[1][1], 0.9999999991354654959, 1e-12, "22");
        vvd(r[1][2], -0.4063240865361857698e-4, 1e-12, "23");
        vvd(r[2][0], -0.3831192481833385226e-5, 1e-12, "31");
        vvd(r[2][1], 0.4063237480216934159e-4, 1e-12, "32");
        vvd(r[2][2], 0.9999999991671660407, 1e-12, "33");
    }
}
