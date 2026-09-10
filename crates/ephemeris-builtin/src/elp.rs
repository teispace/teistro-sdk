//! ELP2000-82B: the Moon.
//!
//! A faithful port of `elp82b.f`, the reader Chapront-Touzé and Chapront
//! publish with the series (CDS catalogue VI/79). Structure and order of
//! operations are preserved so the Fortran and the Rust read side by
//! side, which is the discipline ADR-0021 sets for the ERFA port and for
//! the same reason: a theory transcribed loosely is a theory whose
//! disagreements cannot be attributed.
//!
//! The solution is a sum of sines over arguments built from five
//! polynomial terms in Delaunay's variables, in three groups that differ
//! in how their argument is assembled:
//!
//! | files | group | argument |
//! |---|---|---|
//! | 1 to 3 | the main problem | the four Delaunay arguments alone |
//! | 4 to 9, 22 to 36 | Earth figure, tides, Moon figure, relativity, solar eccentricity | a phase, `zeta`, and the Delaunay arguments |
//! | 10 to 21 | planetary perturbations | a phase and the eight planetary longitudes, with the Delaunay arguments entering two different ways either side of file 16 |
//!
//! The output is geocentric rectangular, in kilometres, referred to the
//! **mean dynamical ecliptic and inertial equinox of J2000** — the same
//! frame family VSOP87 is stated in, and so the same small rotation away
//! from the ICRS that the floor measurement has to separate out.
//!
//! # What the theory says about itself
//!
//! `elp82b.f`'s own header: "Constants fitted to JPL's ephemerides
//! DE200/LE200". That is a 1981-generation fit, and the planetary floor
//! measurement found VSOP87 — fitted to the same DE200 — drifting by
//! arcseconds against a modern ephemeris. Whether the Moon inherits it
//! is what the floor measurement for this theory is for.

#![allow(
    clippy::unreadable_literal,
    clippy::inconsistent_digit_grouping,
    clippy::unusual_byte_groupings,
    reason = "the constants are transcribed from the published reader and are written \
              exactly as it prints them; regrouping their digits would make the port \
              harder to check against its source, which is the one thing a port has to \
              stay easy to do"
)]

use crate::series::J2000;

/// Arcseconds in a radian, as the Fortran computes it: `648000/π`.
const RAD: f64 = 648000.0 / std::f64::consts::PI;

/// Degrees to radians.
const DEG: f64 = std::f64::consts::PI / 180.0;

/// Days in a Julian century, the unit ELP's time argument counts.
const DAYS_PER_CENTURY: f64 = 36525.0;

/// The theory's two distance constants; their ratio scales the radius.
const ATH: f64 = 384747.980674_316_5;
const A0: f64 = 384747.980644_895_4;

/// Constants of the lunar problem the corrections are expressed against.
const AM: f64 = 0.074801_329_518;
const ALFA: f64 = 0.002571_881_335;

/// Which coordinate a file carries: the files cycle longitude, latitude,
/// distance, so the coordinate is the file number modulo three.
///
/// ```
/// use teistro_ephemeris_builtin::elp::coordinate_of;
///
/// assert_eq!(coordinate_of(1), 0, "ELP1 is longitude");
/// assert_eq!(coordinate_of(2), 1, "ELP2 is latitude");
/// assert_eq!(coordinate_of(3), 2, "ELP3 is distance");
/// assert_eq!(coordinate_of(10), 0, "and it cycles");
/// ```
#[must_use]
pub const fn coordinate_of(file: u8) -> usize {
    (file as usize - 1) % 3
}

/// The polynomial arguments, evaluated once and shared by every term.
#[derive(Clone, Copy, Debug)]
pub struct Arguments {
    /// `t^0` to `t^4`, where `t` is Julian centuries from J2000.
    pub t: [f64; 5],
    /// Delaunay's four arguments — D, l', l and F — **by power of
    /// time**, so `del[k]` is the four coefficients of `t^k`.
    ///
    /// Power-major rather than argument-major because the published
    /// reader's loops run the power outside and the argument inside, and
    /// in floating point a sum is its order. Storing it this way lets
    /// the port keep that order without indexing.
    pub del: [[f64; 4]; 5],
    /// The two `zeta` coefficients, the mean longitude with precession.
    pub zeta: [f64; 2],
    /// The eight planetary mean longitudes, by power of time.
    pub p: [[f64; 8]; 2],
    /// The Moon's own mean longitude, as five polynomial coefficients.
    pub w1: [f64; 5],
}

/// The constants of the theory, in the order and to the digits the
/// published reader states them.
#[derive(Clone, Copy, Debug)]
struct Constants {
    /// The Moon's mean longitude, its perigee and its node, each as five
    /// polynomial coefficients.
    w: [[f64; 5]; 3],
    /// The Earth's mean longitude.
    eart: [f64; 5],
    /// The Earth's perihelion.
    peri: [f64; 5],
    /// The eight planetary mean longitudes, constant and rate.
    p: [[f64; 2]; 8],
    /// General precession in longitude.
    preces: f64,
}

fn constants() -> Constants {
    let c1 = 60.0;
    let c2 = 3600.0;
    let w = [
        [
            (218.0 + 18.0 / c1 + 59.95571 / c2) * DEG,
            1732559343.73604 / RAD,
            -5.8883 / RAD,
            0.6604e-2 / RAD,
            -0.3169e-4 / RAD,
        ],
        [
            (83.0 + 21.0 / c1 + 11.67475 / c2) * DEG,
            14643420.2632 / RAD,
            -38.2776 / RAD,
            -0.45047e-1 / RAD,
            0.21301e-3 / RAD,
        ],
        [
            (125.0 + 2.0 / c1 + 40.39816 / c2) * DEG,
            -6967919.3622 / RAD,
            6.3622 / RAD,
            0.7625e-2 / RAD,
            -0.3586e-4 / RAD,
        ],
    ];
    let eart = [
        (100.0 + 27.0 / c1 + 59.22059 / c2) * DEG,
        129597742.2758 / RAD,
        -0.0202 / RAD,
        0.9e-5 / RAD,
        0.15e-6 / RAD,
    ];
    let peri = [
        (102.0 + 56.0 / c1 + 14.42753 / c2) * DEG,
        1161.2283 / RAD,
        0.5327 / RAD,
        -0.138e-3 / RAD,
        0.0,
    ];
    let p = [
        [
            (252.0 + 15.0 / c1 + 3.25986 / c2) * DEG,
            538101628.68898 / RAD,
        ],
        [
            (181.0 + 58.0 / c1 + 47.28305 / c2) * DEG,
            210664136.43355 / RAD,
        ],
        [eart[0], eart[1]],
        [
            (355.0 + 25.0 / c1 + 59.78866 / c2) * DEG,
            68905077.59284 / RAD,
        ],
        [
            (34.0 + 21.0 / c1 + 5.34212 / c2) * DEG,
            10925660.42861 / RAD,
        ],
        [(50.0 + 4.0 / c1 + 38.89694 / c2) * DEG, 4399609.65932 / RAD],
        [
            (314.0 + 3.0 / c1 + 18.01841 / c2) * DEG,
            1542481.19393 / RAD,
        ],
        [
            (304.0 + 20.0 / c1 + 55.19575 / c2) * DEG,
            786550.32074 / RAD,
        ],
    ];
    Constants {
        w,
        eart,
        peri,
        p,
        preces: 5029.0966 / RAD,
    }
}

/// The corrections that fit the theory's constants to DE200 and LE200.
///
/// They are not cosmetic: the main problem's amplitudes are rebuilt from
/// them term by term, so a port that dropped them would be a different
/// theory that still parsed the same files.
#[derive(Clone, Copy, Debug)]
struct Fit {
    delnu: f64,
    dele: f64,
    delg: f64,
    delnp: f64,
    delep: f64,
    dtasm: f64,
}

fn fit(w1_rate: f64) -> Fit {
    Fit {
        delnu: 0.55604 / RAD / w1_rate,
        dele: 0.01789 / RAD,
        delg: -0.08066 / RAD,
        delnp: -0.06424 / RAD / w1_rate,
        delep: -0.12879 / RAD,
        dtasm: 2.0 * ALFA / (3.0 * AM),
    }
}

impl Arguments {
    /// The arguments at a Julian day in Barycentric Dynamical Time.
    #[must_use]
    pub fn at(jd: f64) -> Arguments {
        let Constants {
            w,
            eart,
            peri,
            p,
            preces,
        } = constants();
        let century = (jd - J2000) / DAYS_PER_CENTURY;
        let t = [
            1.0,
            century,
            century * century,
            century * century * century,
            century * century * century * century,
        ];
        // Delaunay's four arguments, each as five polynomial
        // coefficients: D, l', l and F, in the reader's own order.
        let [w1, w2, w3] = w;
        let mut d = [0.0; 5];
        let mut l_prime = [0.0; 5];
        let mut l = [0.0; 5];
        let mut f = [0.0; 5];
        for (((((d, l_prime), l), f), (w1, w2, w3)), (eart, peri)) in d
            .iter_mut()
            .zip(l_prime.iter_mut())
            .zip(l.iter_mut())
            .zip(f.iter_mut())
            .zip(w1.into_iter().zip(w2).zip(w3).map(|((a, b), c)| (a, b, c)))
            .zip(eart.into_iter().zip(peri))
        {
            *d = w1 - eart;
            *l_prime = eart - peri;
            *l = w1 - w2;
            *f = w1 - w3;
        }
        // The reader adds half a turn to D's constant term.
        if let Some(first) = d.first_mut() {
            *first += std::f64::consts::PI;
        }
        // Transposed to power-major as the field documents.
        let mut del = [[0.0; 4]; 5];
        for (k, row) in del.iter_mut().enumerate() {
            for (slot, argument) in row.iter_mut().zip([&d, &l_prime, &l, &f]) {
                *slot = argument.get(k).copied().unwrap_or(0.0);
            }
        }
        let [w1_constant, w1_rate, ..] = w1;
        Arguments {
            t,
            del,
            zeta: [w1_constant, w1_rate + preces],
            p: {
                let mut by_power = [[0.0; 8]; 2];
                for (k, row) in by_power.iter_mut().enumerate() {
                    for (slot, longitude) in row.iter_mut().zip(&p) {
                        *slot = longitude.get(k).copied().unwrap_or(0.0);
                    }
                }
                by_power
            },
            w1,
        }
    }
}

/// One term of the main problem: four Delaunay multipliers and the seven
/// coefficients its amplitude is rebuilt from.
#[derive(Clone, Copy, Debug)]
pub struct MainTerm {
    /// The four Delaunay multipliers.
    pub multipliers: [i8; 4],
    /// The seven coefficients of the published record.
    pub coefficients: [f64; 7],
}

/// One term of every other group: a phase in degrees, an amplitude, and
/// the multipliers its argument is built from.
#[derive(Clone, Copy, Debug)]
pub struct PerturbationTerm {
    /// The multipliers, in the order the file gives them.
    pub multipliers: [i8; 11],
    /// The phase, in degrees.
    pub phase_deg: f64,
    /// The amplitude, in the coordinate's own unit.
    pub amplitude: f64,
}

/// The three groups a file belongs to, which decide how its argument is
/// built and whether its amplitude carries a power of time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    /// Files 1 to 3.
    Main,
    /// Files 4 to 9 and 22 to 36.
    Figure,
    /// Files 10 to 21.
    Planetary,
}

impl Group {
    /// Which group a file number belongs to.
    ///
    /// ```
    /// use teistro_ephemeris_builtin::elp::Group;
    ///
    /// assert_eq!(Group::of(1), Some(Group::Main));
    /// assert_eq!(Group::of(9), Some(Group::Figure));
    /// assert_eq!(Group::of(10), Some(Group::Planetary));
    /// assert_eq!(Group::of(22), Some(Group::Figure));
    /// assert_eq!(Group::of(37), None);
    /// ```
    #[must_use]
    pub const fn of(file: u8) -> Option<Group> {
        match file {
            1..=3 => Some(Group::Main),
            4..=9 | 22..=36 => Some(Group::Figure),
            10..=21 => Some(Group::Planetary),
            _ => None,
        }
    }
}

/// The power of time a file's amplitudes carry.
///
/// The published reader scales three ranges by `t` and one by `t²`;
/// every other file's amplitudes stand as they are.
///
/// ```
/// use teistro_ephemeris_builtin::elp::time_power;
///
/// assert_eq!(time_power(1), 0);
/// assert_eq!(time_power(7), 1, "Earth figure, per time");
/// assert_eq!(time_power(35), 2, "solar eccentricity, per time squared");
/// ```
#[must_use]
pub const fn time_power(file: u8) -> u8 {
    match file {
        7..=9 | 13..=15 | 19..=21 | 25..=27 => 1,
        34..=36 => 2,
        _ => 0,
    }
}

/// The main problem's contribution to a coordinate.
#[must_use]
pub fn main_contribution(term: &MainTerm, file: u8, arguments: &Arguments) -> f64 {
    let f = fit(arguments.w1[1]);
    let mut coefficients = term.coefficients;
    // The distance file's leading coefficient carries a correction of
    // its own before the others are applied.
    if file == 3 {
        coefficients[0] -= 2.0 * coefficients[0] * f.delnu / 3.0;
    }
    let tgv = coefficients[1] + f.dtasm * coefficients[5];
    let amplitude = coefficients[0]
        + tgv * (f.delnp - AM * f.delnu)
        + coefficients[2] * f.delg
        + coefficients[3] * f.dele
        + coefficients[4] * f.delep;
    // `do k=1,5 / do i=1,4`, in that nesting: the power outside, the
    // argument inside.
    let mut angle = 0.0;
    for (row, t) in arguments.del.iter().zip(arguments.t) {
        for (multiplier, coefficient) in term.multipliers.iter().zip(row) {
            angle += f64::from(*multiplier) * coefficient * t;
        }
    }
    // The distance series is a cosine, expressed as a shifted sine.
    if coordinate_of(file) == 2 {
        angle += std::f64::consts::FRAC_PI_2;
    }
    amplitude * angle.sin()
}

/// A perturbation term's contribution to a coordinate.
#[must_use]
pub fn perturbation_contribution(term: &PerturbationTerm, file: u8, arguments: &Arguments) -> f64 {
    let amplitude = term.amplitude
        * arguments
            .t
            .get(time_power(file) as usize)
            .copied()
            .unwrap_or(1.0);
    let mut angle = term.phase_deg * DEG;
    match Group::of(file) {
        Some(Group::Figure) => {
            // Per power: the zeta term first, then the four Delaunay
            // terms — which is the order inside the reader's `do k=1,2`.
            let (zeta_multiplier, rest) = term.multipliers.split_first().unwrap_or((&0, &[]));
            let zeta_multiplier = f64::from(*zeta_multiplier);
            for ((row, zeta), t) in arguments
                .del
                .iter()
                .zip(arguments.zeta)
                .zip(arguments.t)
                .take(2)
            {
                angle += zeta_multiplier * zeta * t;
                for (multiplier, coefficient) in rest.iter().zip(row) {
                    angle += f64::from(*multiplier) * coefficient * t;
                }
            }
        }
        Some(Group::Planetary) if file <= 15 => {
            // Table 1, per power: three Delaunay terms summed **inside
            // one bracket** and then multiplied by the power, and after
            // them the eight planetary longitudes. The bracket is the
            // reader's own and is kept, because it is a different sum
            // from three separate additions.
            let (planets, lunar) = term.multipliers.split_at(8);
            let [d_multiplier, l_multiplier, f_multiplier] =
                <[i8; 3]>::try_from(lunar).unwrap_or([0; 3]);
            for ((row, planet_row), t) in arguments
                .del
                .iter()
                .zip(&arguments.p)
                .zip(arguments.t)
                .take(2)
            {
                let [d, _, l, f] = *row;
                angle += (f64::from(d_multiplier) * d
                    + f64::from(l_multiplier) * l
                    + f64::from(f_multiplier) * f)
                    * t;
                for (multiplier, coefficient) in planets.iter().zip(planet_row) {
                    angle += f64::from(*multiplier) * coefficient * t;
                }
            }
        }
        Some(Group::Planetary) => {
            // Table 2, per power: the four Delaunay terms, then seven
            // planetary longitudes.
            let (planets, lunar) = term.multipliers.split_at(7);
            for ((row, planet_row), t) in arguments
                .del
                .iter()
                .zip(&arguments.p)
                .zip(arguments.t)
                .take(2)
            {
                for (multiplier, coefficient) in lunar.iter().zip(row) {
                    angle += f64::from(*multiplier) * coefficient * t;
                }
                for (multiplier, coefficient) in planets.iter().zip(planet_row) {
                    angle += f64::from(*multiplier) * coefficient * t;
                }
            }
        }
        Some(Group::Main) | None => return 0.0,
    }
    amplitude * angle.sin()
}

/// The three summed coordinates, turned into a geocentric rectangular
/// position in kilometres referred to the mean dynamical ecliptic and
/// inertial equinox of J2000.
///
/// `sums` are the raw series totals: longitude and latitude in
/// arcseconds, distance in kilometres before its scale.
#[must_use]
pub fn to_rectangular(sums: [f64; 3], jd: f64) -> [f64; 3] {
    let arguments = Arguments::at(jd);
    let [_, t1, t2, t3, t4] = arguments.t;
    let [sum_longitude, sum_latitude, sum_distance] = sums;
    let [w0, w1, w2, w3, w4] = arguments.w1;
    // The Moon's mean longitude is added back to the longitude series,
    // accumulated left to right exactly as the published reader does it.
    // An iterator sum here is the same arithmetic and not the same
    // floating-point result: it associates differently, and the
    // measurement moved in its ninth digit when this was written that
    // way. A port's operation order is part of what is being ported
    // (ADR-0021).
    let longitude = sum_longitude / RAD + w0 + w1 * t1 + w2 * t2 + w3 * t3 + w4 * t4;
    let latitude = sum_latitude / RAD;
    let distance = sum_distance * A0 / ATH;

    let in_plane = distance * latitude.cos();
    let x1 = in_plane * longitude.cos();
    let x2 = in_plane * longitude.sin();
    let x3 = distance * latitude.sin();

    // The rotation to J2000, as the published reader writes it.
    let pw = (0.10180391e-4 + 0.47020439e-6 * t1 - 0.5417367e-9 * t2 - 0.2507948e-11 * t3
        + 0.463486e-14 * t4)
        * t1;
    let qw = (-0.113469002e-3 + 0.12372674e-6 * t1 + 0.1265417e-8 * t2
        - 0.1371808e-11 * t3
        - 0.320334e-14 * t4)
        * t1;
    let ra = 2.0 * (1.0 - pw * pw - qw * qw).sqrt();
    let pwqw = 2.0 * pw * qw;
    let pw2 = 1.0 - 2.0 * pw * pw;
    let qw2 = 1.0 - 2.0 * qw * qw;
    let pw = pw * ra;
    let qw = qw * ra;
    [
        pw2 * x1 + pwqw * x2 + pw * x3,
        pwqw * x1 + qw2 * x2 - qw * x3,
        -pw * x1 + qw * x2 + (pw2 + qw2 - 1.0) * x3,
    ]
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "a test fails by panicking and indexes its own fixtures"
    )]

    use super::*;

    #[test]
    fn the_files_divide_into_the_three_groups_the_reader_switches_on() {
        let mut counts = [0usize; 3];
        for file in 1..=36u8 {
            match Group::of(file).expect("every file of the theory") {
                Group::Main => counts[0] += 1,
                Group::Figure => counts[1] += 1,
                Group::Planetary => counts[2] += 1,
            }
        }
        assert_eq!(counts, [3, 21, 12], "3 main, 21 figure-class, 12 planetary");
    }

    #[test]
    fn a_coordinate_cycles_with_the_file_number() {
        // Every group starts on longitude, which is what makes the
        // modulo work across all thirty-six.
        for (file, expected) in [(1, 0), (2, 1), (3, 2), (4, 0), (22, 0), (34, 0), (36, 2)] {
            assert_eq!(coordinate_of(file), expected, "ELP{file}");
        }
    }

    #[test]
    fn the_arguments_are_the_constants_at_the_epoch() {
        let arguments = Arguments::at(J2000);
        assert!((arguments.t[1]).abs() < 1e-15, "t is nothing at J2000");
        assert!((arguments.t[0] - 1.0).abs() < 1e-15, "t^0 is one");
        // Delaunay's D at J2000 is the Moon's mean longitude less the
        // Earth's, plus half a turn.
        let expected = (218.0 + 18.0 / 60.0 + 59.95571 / 3600.0) * DEG
            - (100.0 + 27.0 / 60.0 + 59.22059 / 3600.0) * DEG
            + std::f64::consts::PI;
        assert!((arguments.del[0][0] - expected).abs() < 1e-12, "D at t^0");
    }

    /// The rotation must be the identity at the epoch it rotates to,
    /// which is the one point where it can be checked without the series.
    #[test]
    fn the_rotation_is_the_identity_at_j2000() {
        let sums = [0.0, 0.0, 1.0];
        let out = to_rectangular(sums, J2000);
        let arguments = Arguments::at(J2000);
        let longitude = arguments.w1[0];
        let expected = [A0 / ATH * longitude.cos(), A0 / ATH * longitude.sin(), 0.0];
        for (got, want) in out.iter().zip(expected) {
            assert!((got - want).abs() < 1e-12, "{got} against {want}");
        }
    }

    #[test]
    fn the_time_powers_are_the_four_ranges_the_reader_scales() {
        let scaled: Vec<u8> = (1..=36u8).filter(|f| time_power(*f) > 0).collect();
        assert_eq!(
            scaled,
            vec![7, 8, 9, 13, 14, 15, 19, 20, 21, 25, 26, 27, 34, 35, 36]
        );
        assert!(
            (34..=36).all(|f| time_power(f) == 2),
            "only the solar-eccentricity files are quadratic"
        );
    }
}

/// Reading the published ELP2000-82B files.
///
/// The generator's side, behind the `ingest` feature for the same reason
/// the VSOP87 reader is: a consumer receives truncated tables and has no
/// use for a parser of 2.4 MB of text.
#[cfg(feature = "ingest")]
pub mod ingest {
    use std::path::Path;

    use super::{Group, MainTerm, PerturbationTerm};

    /// Every term the theory holds, kept by the file it came from so the
    /// group, the coordinate and the power of time stay attached to it.
    #[derive(Clone, Debug, Default)]
    pub struct Theory {
        /// The main problem, files 1 to 3.
        pub main: Vec<(u8, MainTerm)>,
        /// Every other file's terms.
        pub perturbations: Vec<(u8, PerturbationTerm)>,
    }

    impl Theory {
        /// How many terms the whole theory holds.
        #[must_use]
        pub fn terms(&self) -> usize {
            self.main.len() + self.perturbations.len()
        }
    }

    /// What went wrong reading a file.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ElpError {
        /// The file number.
        pub file: u8,
        /// The one-based line.
        pub line: usize,
        /// What could not be read.
        pub detail: String,
    }

    impl core::fmt::Display for ElpError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "ELP{}: line {}: {}", self.file, self.line, self.detail)
        }
    }

    impl std::error::Error for ElpError {}

    /// Reads `N` fixed-width integers from the front of a record, into a
    /// slot of the eleven a term carries.
    ///
    /// Fixed-width rather than split on whitespace because the
    /// multipliers are `i3` fields, which touch as soon as one of them
    /// reaches `-10`.
    fn integers<const N: usize>(line: &str) -> Option<[i8; 11]> {
        let mut out = [0i8; 11];
        for (index, slot) in out.iter_mut().take(N).enumerate() {
            *slot = line
                .get(index * 3..index * 3 + 3)?
                .trim()
                .parse::<i8>()
                .ok()?;
        }
        Some(out)
    }

    /// Reads exactly `want` whitespace-separated floats from after the
    /// integer prefix, refusing a record that holds fewer.
    ///
    /// The count is fixed rather than checked afterwards so the caller
    /// receives an array and never an index that could be out of range.
    fn floats<const N: usize>(line: &str, skip: usize) -> Option<[f64; N]> {
        let mut out = [0.0; N];
        let mut fields = line.get(skip * 3..)?.split_whitespace();
        for slot in &mut out {
            *slot = fields.next()?.parse::<f64>().ok()?;
        }
        Some(out)
    }

    /// Parses one file's records into the theory.
    ///
    /// # Errors
    ///
    /// [`ElpError`] naming the file, the line and what would not read.
    pub fn parse_file(file: u8, text: &str, into: &mut Theory) -> Result<(), ElpError> {
        let group = Group::of(file).ok_or_else(|| ElpError {
            file,
            line: 0,
            detail: format!("{file} is not one of the theory's thirty-six files"),
        })?;
        let fail = |line: usize, detail: &str| ElpError {
            file,
            line,
            detail: detail.to_string(),
        };
        // The first line of every file is its title.
        for (index, line) in text.lines().enumerate().skip(1) {
            let number = index + 1;
            if line.trim().is_empty() {
                continue;
            }
            match group {
                Group::Main => {
                    let eleven =
                        integers::<4>(line).ok_or_else(|| fail(number, "four multipliers"))?;
                    let coefficients =
                        floats::<7>(line, 4).ok_or_else(|| fail(number, "seven coefficients"))?;
                    let [a, b, c, d, ..] = eleven;
                    into.main.push((
                        file,
                        MainTerm {
                            multipliers: [a, b, c, d],
                            coefficients,
                        },
                    ));
                }
                Group::Figure => {
                    let multipliers =
                        integers::<5>(line).ok_or_else(|| fail(number, "five multipliers"))?;
                    let [phase_deg, amplitude] = floats::<2>(line, 5)
                        .ok_or_else(|| fail(number, "a phase and an amplitude"))?;
                    into.perturbations.push((
                        file,
                        PerturbationTerm {
                            multipliers,
                            phase_deg,
                            amplitude,
                        },
                    ));
                }
                Group::Planetary => {
                    let multipliers =
                        integers::<11>(line).ok_or_else(|| fail(number, "eleven multipliers"))?;
                    let [phase_deg, amplitude] = floats::<2>(line, 11)
                        .ok_or_else(|| fail(number, "a phase and an amplitude"))?;
                    into.perturbations.push((
                        file,
                        PerturbationTerm {
                            multipliers,
                            phase_deg,
                            amplitude,
                        },
                    ));
                }
            }
        }
        Ok(())
    }

    /// Reads all thirty-six files from a directory.
    ///
    /// # Errors
    ///
    /// [`ElpError`] for a file that cannot be read or parsed.
    pub fn load(dir: &Path) -> Result<Theory, ElpError> {
        let mut theory = Theory::default();
        for file in 1..=36u8 {
            let path = dir.join(format!("ELP{file}"));
            let text = std::fs::read_to_string(&path).map_err(|error| ElpError {
                file,
                line: 0,
                detail: format!("cannot read {}: {error}", path.display()),
            })?;
            parse_file(file, &text, &mut theory)?;
        }
        Ok(theory)
    }

    /// The Moon's geocentric rectangular position in kilometres, in the
    /// mean dynamical ecliptic and inertial equinox of J2000, keeping
    /// only terms at or above `threshold` in the coordinate's own unit.
    ///
    /// `threshold` is the published reader's own truncation knob, which
    /// is what makes a tier a length rather than a different table.
    #[must_use]
    pub fn position(theory: &Theory, jd: f64, threshold: f64) -> [f64; 3] {
        let arguments = super::Arguments::at(jd);
        let mut sums = [0.0; 3];
        // `get_mut` rather than an index: the coordinate is always one of
        // three by construction, and saying so with the access rather
        // than in a comment is what keeps it true after an edit.
        let mut add = |file: u8, value: f64| {
            if let Some(sum) = sums.get_mut(super::coordinate_of(file)) {
                *sum += value;
            }
        };
        for (file, term) in &theory.main {
            let Some(leading) = term.coefficients.first() else {
                continue;
            };
            if leading.abs() < threshold {
                continue;
            }
            add(*file, super::main_contribution(term, *file, &arguments));
        }
        for (file, term) in &theory.perturbations {
            if term.amplitude < threshold {
                continue;
            }
            add(
                *file,
                super::perturbation_contribution(term, *file, &arguments),
            );
        }
        super::to_rectangular(sums, jd)
    }

    #[cfg(test)]
    mod tests {
        #![allow(
            clippy::unwrap_used,
            clippy::expect_used,
            clippy::indexing_slicing,
            reason = "a test fails by panicking and indexes its own fixtures"
        )]

        use super::*;

        #[test]
        fn a_main_record_reads_its_multipliers_and_seven_coefficients() {
            let text = "MAIN PROBLEM. LONGITUDE(SINE)\n  0  0  0  2     -411.60287      168.48   -18433.81     -121.62        0.40       -0.18        0.00";
            let mut theory = Theory::default();
            parse_file(1, text, &mut theory).expect("a well-formed record");
            assert_eq!(theory.main.len(), 1);
            let (file, term) = theory.main[0];
            assert_eq!(file, 1);
            assert_eq!(term.multipliers, [0, 0, 0, 2]);
            assert!((term.coefficients[0] - -411.60287).abs() < 1e-9);
            assert!((term.coefficients[6] - 0.0).abs() < 1e-9);
        }

        #[test]
        fn a_figure_record_reads_five_multipliers() {
            let text = "EARTH FIGURE PERTURBATIONS. LONGITUDE\n  0  0  0  0  1 270.00000   0.00003     0.075";
            let mut theory = Theory::default();
            parse_file(4, text, &mut theory).expect("a well-formed record");
            let (_, term) = theory.perturbations[0];
            assert_eq!(&term.multipliers[..5], &[0, 0, 0, 0, 1]);
            assert!((term.phase_deg - 270.0).abs() < 1e-9);
            assert!((term.amplitude - 0.00003).abs() < 1e-9);
        }

        #[test]
        fn a_planetary_record_reads_eleven_multipliers() {
            let text = "PLANETARY PERTURBATIONS. TABLE 1. LONGITUDE\n  0  0  0  0  0  0  0  0  0  1 -2 359.98254   0.00007     0.074";
            let mut theory = Theory::default();
            parse_file(10, text, &mut theory).expect("a well-formed record");
            let (_, term) = theory.perturbations[0];
            assert_eq!(term.multipliers, [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, -2]);
            assert!((term.phase_deg - 359.98254).abs() < 1e-9);
        }

        #[test]
        fn a_file_outside_the_theory_is_refused() {
            let mut theory = Theory::default();
            assert!(parse_file(37, "x\n", &mut theory).is_err());
        }
    }
}
