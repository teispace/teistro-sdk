//! The epicycles and the equations: the corrected epicycle (II.38), the
//! manda equation (II.39 to 40, 45), the sighra equation through its
//! hypotenuse (II.39 to 42), the four-step procedure for the star planets
//! (II.43 to 44) and the true daily motion of a manda-only body (II.47 to
//! 49). Every equation is signed by the text's rule (II.45): additive
//! when the anomaly is in the half of the circle that begins with Mesha,
//! subtractive in the half that begins with Tula.

use crate::trig::{Bhuja, RADIUS, Trig};

/// An epicycle: its circumference in minutes of arc at the end of the
/// even quadrants and at the end of the odd ones (II.34 to 37).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Epicycle {
    /// The circumference at the end of an even quadrant (the anomaly at
    /// 180° or 360°), minutes.
    pub even_arcmin: u32,
    /// The circumference at the end of an odd quadrant (the anomaly at
    /// 90° or 270°), minutes.
    pub odd_arcmin: u32,
}

impl Epicycle {
    /// An epicycle from its two circumferences in minutes.
    #[must_use]
    pub const fn new(even_arcmin: u32, odd_arcmin: u32) -> Epicycle {
        Epicycle {
            even_arcmin,
            odd_arcmin,
        }
    }

    /// The corrected circumference in degrees at a reduced anomaly
    /// (II.38): the even-quadrant value, moved toward the odd-quadrant
    /// value by the sine of the reduced arc over the radius.
    #[must_use]
    pub fn corrected_deg(self, trig: Trig, bhuja: Bhuja) -> f64 {
        let even = f64::from(self.even_arcmin) / 60.0;
        let odd = f64::from(self.odd_arcmin) / 60.0;
        even + (odd - even) * trig.sine(bhuja.arc_deg) / RADIUS
    }
}

/// Applies the text's sign rule (II.45) to an equation's magnitude.
fn signed(magnitude_deg: f64, bhuja: Bhuja) -> f64 {
    if bhuja.sine_positive {
        magnitude_deg
    } else {
        -magnitude_deg
    }
}

/// The manda equation in degrees, signed, for an anomaly
/// (apsis less mean place; II.29): the sine of the reduced anomaly times
/// the corrected circumference over 360 is the "sine of the equation"; its
/// arc is the equation (II.39 to 40).
#[must_use]
pub fn manda_equation_deg(trig: Trig, epicycle: Epicycle, kendra_deg: f64) -> f64 {
    let bhuja = Bhuja::of(kendra_deg);
    let circumference = epicycle.corrected_deg(trig, bhuja);
    let bhujaphala = circumference / 360.0 * trig.sine(bhuja.arc_deg);
    signed(trig.arc(bhujaphala), bhuja)
}

/// The sighra equation and the hypotenuse it went through.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SighraEquation {
    /// The equation in degrees, signed.
    pub equation_deg: f64,
    /// The sighra hypotenuse (karna) on the radius: the distance of the
    /// planet from the observer in the epicycle's construction.
    pub karna: f64,
}

/// The sighra equation for an anomaly (conjunction less planet; II.29):
/// the sine and cosine of the reduced anomaly scaled by the corrected
/// circumference over 360 are the bhujaphala and kotiphala; the kotiphala
/// applied to the radius (added in the half from Makara through Mithuna,
/// taken away in the other) and the bhujaphala give the hypotenuse; the
/// bhujaphala times the radius over the hypotenuse is the sine of the
/// equation, whose arc is the equation (II.39 to 42).
#[must_use]
pub fn sighra_equation(trig: Trig, epicycle: Epicycle, kendra_deg: f64) -> SighraEquation {
    let bhuja = Bhuja::of(kendra_deg);
    let scale = epicycle.corrected_deg(trig, bhuja) / 360.0;
    let bhujaphala = scale * trig.sine(bhuja.arc_deg);
    let kotiphala = scale * trig.sine(bhuja.complement_deg());
    let radial = if bhuja.cosine_positive {
        RADIUS + kotiphala
    } else {
        RADIUS - kotiphala
    };
    let karna = (radial * radial + bhujaphala * bhujaphala).sqrt();
    let sine = if karna > 0.0 {
        bhujaphala * RADIUS / karna
    } else {
        0.0
    };
    SighraEquation {
        equation_deg: signed(trig.arc(sine), bhuja),
        karna,
    }
}

/// The four-step procedure of a star planet (II.43 to 44), from its mean
/// place, its apsis and its conjunction: half the sighra equation to the
/// mean place; half the manda equation of the result to the result; the
/// whole manda equation of that, to the original mean place; the whole
/// sighra equation of that, to it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FourStep {
    /// The mean place corrected by the whole manda equation (step three),
    /// degrees in `[0, 360)`.
    pub manda_corrected_deg: f64,
    /// The manda equation applied in step three, degrees, signed.
    pub manda_equation_deg: f64,
    /// The sighra equation applied in step four.
    pub sighra: SighraEquation,
    /// The true place, degrees in `[0, 360)`.
    pub true_deg: f64,
}

/// Runs the four steps.
#[must_use]
pub fn four_step(
    trig: Trig,
    manda: Epicycle,
    sighra: Epicycle,
    mean_deg: f64,
    apsis_deg: f64,
    conjunction_deg: f64,
) -> FourStep {
    let half_sighra = sighra_equation(trig, sighra, conjunction_deg - mean_deg).equation_deg / 2.0;
    let first = (mean_deg + half_sighra).rem_euclid(360.0);
    let half_manda = manda_equation_deg(trig, manda, apsis_deg - first) / 2.0;
    let second = (first + half_manda).rem_euclid(360.0);
    let manda_equation = manda_equation_deg(trig, manda, apsis_deg - second);
    let manda_corrected = (mean_deg + manda_equation).rem_euclid(360.0);
    let sighra_result = sighra_equation(trig, sighra, conjunction_deg - manda_corrected);
    FourStep {
        manda_corrected_deg: manda_corrected,
        manda_equation_deg: manda_equation,
        sighra: sighra_result,
        true_deg: (manda_corrected + sighra_result.equation_deg).rem_euclid(360.0),
    }
}

/// The true daily motion of a manda-only body (II.47 to 49): the
/// anomaly's daily motion (the mean motion less the apsis's) times the
/// cosine of the reduced anomaly (the table's difference of sines over
/// the step) times the corrected circumference over 360, taken from the
/// mean motion in the half from Makara through Mithuna and added in the
/// other. Degrees per day.
#[must_use]
pub fn manda_motion_deg_per_day(
    trig: Trig,
    epicycle: Epicycle,
    kendra_deg: f64,
    mean_motion_deg: f64,
    apsis_motion_deg: f64,
) -> f64 {
    let bhuja = Bhuja::of(kendra_deg);
    let circumference = epicycle.corrected_deg(trig, bhuja);
    let anomaly_motion = mean_motion_deg - apsis_motion_deg;
    let correction = anomaly_motion * trig.cosine_ratio(bhuja.arc_deg) * circumference / 360.0;
    if bhuja.cosine_positive {
        mean_motion_deg - correction
    } else {
        mean_motion_deg + correction
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use crate::params::{Parameters, Planet};

    const SUN: Epicycle = Epicycle::new(14 * 60, 13 * 60 + 40);

    #[test]
    fn the_corrected_epicycle_moves_from_even_to_odd() {
        // At the end of an even quadrant the even value; at the end of an
        // odd quadrant the odd value; between, by the sine (II.38).
        assert!((SUN.corrected_deg(Trig::Table, Bhuja::of(0.0)) - 14.0).abs() < 1e-12);
        assert!((SUN.corrected_deg(Trig::Table, Bhuja::of(180.0)) - 14.0).abs() < 1e-12);
        let odd = 13.0 + 40.0 / 60.0;
        assert!((SUN.corrected_deg(Trig::Table, Bhuja::of(90.0)) - odd).abs() < 1e-12);
        assert!((SUN.corrected_deg(Trig::Table, Bhuja::of(270.0)) - odd).abs() < 1e-12);
        let mid = SUN.corrected_deg(Trig::Table, Bhuja::of(30.0));
        assert!((mid - (14.0 - (1.0 / 3.0) * 0.5)).abs() < 1e-3, "{mid}");
        let jupiter_sighra = Epicycle::new(70 * 60, 72 * 60);
        assert!(
            jupiter_sighra.corrected_deg(Trig::Exact, Bhuja::of(90.0)) > 70.0,
            "an odd value above the even one moves upward"
        );
    }

    #[test]
    fn the_manda_equation_peaks_at_the_odd_quadrant_and_carries_its_sign() {
        // The greatest equation of the Sun: arc of 13°40′/360 × R, 2.175°
        // (with the odd-quadrant circumference, not the even one).
        let peak = manda_equation_deg(Trig::Table, SUN, 90.0);
        let expected = ((13.0 + 40.0 / 60.0) / 360.0_f64).asin().to_degrees();
        assert!((peak - expected).abs() < 1e-3, "{peak} vs {expected}");
        assert!(manda_equation_deg(Trig::Table, SUN, 270.0) < 0.0);
        assert!(manda_equation_deg(Trig::Table, SUN, 0.0).abs() < 1e-12);
        assert!(manda_equation_deg(Trig::Table, SUN, 180.0).abs() < 1e-12);
        // The tradition's worked figure: a circumference of 13°43′ at a
        // sine of 3020.93 gives 115.1′ of equation, 1°55′.
        let bhuja = Bhuja::of(61.0 + 31.0 / 60.0 + 9.0 / 3600.0);
        let circumference = SUN.corrected_deg(Trig::Table, bhuja);
        assert!(
            (circumference - (13.0 + 43.0 / 60.0)).abs() < 0.01,
            "{circumference}"
        );
        let equation = manda_equation_deg(Trig::Table, SUN, bhuja.arc_deg);
        assert!((equation * 60.0 - 115.1).abs() < 0.2, "{}", equation * 60.0);
        // Exact and tabular trigonometry agree to the table's precision.
        for k in [10.0, 45.0, 100.0, 200.0, 300.0] {
            let a = manda_equation_deg(Trig::Table, SUN, k);
            let b = manda_equation_deg(Trig::Exact, SUN, k);
            assert!((a - b).abs() < 0.002, "{k}: {a} {b}");
        }
    }

    #[test]
    fn the_sighra_equation_goes_through_the_hypotenuse() {
        let mars = Parameters::TEXT.sighra_epicycle(Planet::Mars).unwrap();
        // At conjunction and opposition the equation vanishes; the
        // hypotenuse is the radius plus or less the circumference's share.
        let at_zero = sighra_equation(Trig::Table, mars, 0.0);
        assert!(at_zero.equation_deg.abs() < 1e-12);
        assert!((at_zero.karna - (RADIUS + 235.0 / 360.0 * RADIUS)).abs() < 1e-9);
        let at_opposition = sighra_equation(Trig::Table, mars, 180.0);
        assert!(at_opposition.equation_deg.abs() < 1e-12);
        assert!((at_opposition.karna - (RADIUS - 235.0 / 360.0 * RADIUS)).abs() < 1e-9);
        // Near opposition the equation is large and steep: Mars's greatest
        // sighra equation is over 40°.
        let mut greatest = 0.0f64;
        let mut k = 0.0;
        while k < 360.0 {
            greatest = greatest.max(sighra_equation(Trig::Exact, mars, k).equation_deg.abs());
            k += 0.5;
        }
        assert!(greatest > 40.0 && greatest < 46.0, "{greatest}");
        assert!(sighra_equation(Trig::Table, mars, 250.0).equation_deg < 0.0);
        // Mercury's greatest elongation from the Sun is about 22° in the
        // text's construction.
        let mercury = Parameters::TEXT.sighra_epicycle(Planet::Mercury).unwrap();
        let mut elongation = 0.0f64;
        let mut k = 0.0;
        while k < 360.0 {
            elongation =
                elongation.max(sighra_equation(Trig::Table, mercury, k).equation_deg.abs());
            k += 0.5;
        }
        assert!(elongation > 20.0 && elongation < 24.0, "{elongation}");
    }

    #[test]
    fn the_four_steps_compose_as_the_text_says() {
        let manda = Parameters::TEXT.manda_epicycle(Planet::Mars);
        let sighra = Parameters::TEXT.sighra_epicycle(Planet::Mars).unwrap();
        // Step three applies the whole manda equation to the original mean
        // place; step four applies the whole sighra equation of that.
        let result = four_step(Trig::Table, manda, sighra, 100.0, 130.0, 100.0);
        assert!(result.manda_equation_deg > 0.0);
        assert!((result.manda_corrected_deg - (100.0 + result.manda_equation_deg)).abs() < 1e-9);
        let expected_sighra =
            sighra_equation(Trig::Table, sighra, 100.0 - result.manda_corrected_deg);
        assert!((result.sighra.equation_deg - expected_sighra.equation_deg).abs() < 1e-9);
        assert!(
            ((result.manda_corrected_deg + result.sighra.equation_deg).rem_euclid(360.0)
                - result.true_deg)
                .abs()
                < 1e-9
        );
        // With the conjunction at the manda-corrected place the sighra
        // equation vanishes and the true place is the manda-corrected one.
        // (Not exactly: the half sighra equation of step one moves the
        // place the manda equation is taken at, so the two differ a little.)
        let aligned = four_step(
            Trig::Table,
            manda,
            sighra,
            100.0,
            130.0,
            result.manda_corrected_deg,
        );
        assert!(
            aligned.sighra.equation_deg.abs() < 0.2,
            "{}",
            aligned.sighra.equation_deg
        );
        // A general case stays in range and is continuous in the mean place.
        let a = four_step(Trig::Table, manda, sighra, 200.0, 130.0, 20.0).true_deg;
        let b = four_step(Trig::Table, manda, sighra, 200.1, 130.0, 20.1).true_deg;
        assert!((0.0..360.0).contains(&a) && (a - b).abs() < 1.0);
    }

    #[test]
    fn the_true_motion_is_slowest_near_the_apsis() {
        let mean = 0.985_602_6;
        let apsis = 0.000_002;
        // Anomaly 0: at the apsis (kendra 0 means the mean place is at the
        // apsis), the planet is farthest and slowest.
        let at_apsis = manda_motion_deg_per_day(Trig::Table, SUN, 0.0, mean, apsis);
        let at_perigee = manda_motion_deg_per_day(Trig::Table, SUN, 180.0, mean, apsis);
        assert!(at_apsis < mean && at_perigee > mean);
        assert!(
            (at_apsis - mean * (1.0 - 14.0 / 360.0)).abs() < 1e-6,
            "{at_apsis}"
        );
        let at_quadrature = manda_motion_deg_per_day(Trig::Exact, SUN, 90.0, mean, apsis);
        assert!((at_quadrature - mean).abs() < 1e-6);
        assert!(
            (manda_motion_deg_per_day(Trig::Table, SUN, 90.0, mean, apsis) - mean).abs() < 2e-3
        );
    }
}
