//! B.V. Raman's Standard Horoscope, the chart *Graha and Bhava Balas*
//! (thirteenth edition) works every Shadbala and Bhava bala component on:
//! 16 October 1918, 2h 6m 16s p.m. local mean time at 77° 35′ E, Makara
//! rising.
//!
//! The inputs are his Chapter I's and Example 58's, so a reading of them
//! can be held to his printed answers, as the crate's own test holds every
//! component (`tests/sripati.rs`) and the examples here quote a few:
//! Mercury's Shadbala is 537.02 virupas under
//! [`ShadbalaRules::SRIPATI`](crate::ShadbalaRules::SRIPATI) (Example 56).
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_strength::{ShadbalaReading, ShadbalaRules, sample};
//!
//! let reading = ShadbalaReading::of(&sample::standard_horoscope(), ShadbalaRules::SRIPATI);
//! let mercury = &reading.grahas[Graha::Mercury as usize];
//! assert!((mercury.virupas - 537.02).abs() < 0.5);
//! ```

use teistro_core::angle::Nas;
use teistro_core::catalogue::{Graha, Rashi, Varga};
use teistro_vargas::{Scheme, sign};

use crate::bhava_bala::{BhavaBalaChart, BhavaGraha};
use crate::shadbala::{SAPTAVARGAJA_VARGAS, ShadbalaChart, ShadbalaGraha};

/// Degrees, minutes and seconds.
fn dms(degrees: f64, minutes: f64, seconds: f64) -> f64 {
    degrees + minutes / 60.0 + seconds / 3600.0
}

/// Art. 6: the nirayana longitudes, Sun to Saturn.
fn longitudes() -> [f64; 7] {
    [
        dms(180.0, 53.0, 55.0),
        dms(311.0, 17.0, 19.0),
        dms(229.0, 30.0, 34.0),
        dms(181.0, 31.0, 34.0),
        dms(84.0, 0.0, 49.0),
        dms(171.0, 9.0, 56.0),
        dms(124.0, 22.0, 41.0),
    ]
}

/// The day the birth falls on, 16 October 1918, at 0h UT.
const DAY: f64 = 2_421_882.5;

/// An instant of the day, hours, minutes and seconds of UT.
fn ut(hours: f64, minutes: f64, seconds: f64) -> f64 {
    DAY + dms(hours, minutes, seconds) / 24.0
}

/// The house a longitude falls in, whole signs from Makara rising (Art. 7).
fn house(longitude: f64) -> u8 {
    (varga_sign(Varga::D1, longitude) as u8 + 12 - Rashi::Capricorn as u8) % 12 + 1
}

/// The sign a longitude falls in, in a divisional chart.
///
/// # Panics
///
/// Never: every longitude here is a finite constant.
fn varga_sign(varga: Varga, longitude: f64) -> Rashi {
    #[expect(
        clippy::expect_used,
        reason = "the horoscope's longitudes are finite constants"
    )]
    let at = Nas::try_from_degrees(longitude).expect("a finite longitude");
    sign(&Scheme::of(varga), at)
}

/// The Standard Horoscope as the Shadbala reads it, from Chapter I and the
/// examples that use it: the ayanamsha 21° 16′ (Example 32), a Wednesday in
/// the Moon's hora (Examples 22 and 31), a day of 29 ghatis 20 vighatis
/// from a sunrise at 5h 54m LMT (Art. 5), and the first and tenth bhava
/// madhyas as the angles (Art. 29). Raman gives no latitudes, so they are
/// nought, and no sankranti lord.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_strength::{ShadbalaReading, ShadbalaRules, sample};
///
/// // Example 57: every graha reaches Sripati's requirement but Mars.
/// let reading = ShadbalaReading::of(&sample::standard_horoscope(), ShadbalaRules::SRIPATI);
/// for graha in &reading.grahas {
///     assert_eq!(graha.strong, graha.graha != Graha::Mars);
/// }
/// ```
///
/// # Panics
///
/// Never; see [`varga_sign`].
#[must_use]
pub fn standard_horoscope() -> ShadbalaChart {
    let longitudes = longitudes();
    let ayanamsha = dms(21.0, 16.0, 0.0);
    let sunrise = ut(0.0, 43.0, 40.0);
    ShadbalaChart {
        grahas: longitudes.map(|longitude| ShadbalaGraha {
            longitude,
            tropical: longitude + ayanamsha,
            latitude: 0.0,
            house: house(longitude),
        }),
        rahu: Some(dms(234.0, 23.0, 47.0)),
        vargas: SAPTAVARGAJA_VARGAS.map(|varga| longitudes.map(|l| varga_sign(varga, l))),
        // 2h 6m 16s p.m. LMT at 77° 35′ E is 8h 55m 56s UT.
        instant: ut(8.0, 55.0, 56.0),
        sunrise,
        sunset: sunrise + 29.333_333 / 60.0,
        next_sunrise: sunrise + 1.0,
        after_midnight: false,
        civil_day: 2_421_883,
        weekday_lord: Graha::Mercury,
        hora_lord: Graha::Moon,
        sankranti_lord: None,
        ascendant: dms(298.0, 27.0, 0.0),
        midheaven: dms(216.0, 36.0, 0.0),
        ayanamsha,
        obliquity: 24.0,
    }
}

/// The Standard Horoscope as the Bhava bala reads it: the twelve bhava
/// madhyas of Art. 29, the grahas of [`standard_horoscope`], and the lords'
/// Shadbalas and Drik balas as Example 59 takes them from Example 58, with
/// Rahu where Art. 6 places it and Ketu opposite.
///
/// ```
/// use teistro_strength::{BhavaBalaReading, BhavaBalaRules, sample};
///
/// // Example 59: the first bhava's Digbala is 30 and its total 473.39.
/// let reading = BhavaBalaReading::of(&sample::standard_bhavas(), BhavaBalaRules::SRIPATI);
/// assert_eq!(reading.bhavas[0].dig, 30.0);
/// assert!((reading.bhavas[0].virupas - 473.39).abs() < 0.5);
/// ```
#[must_use]
pub fn standard_bhavas() -> BhavaBalaChart {
    let horoscope = standard_horoscope();
    let madhya = [
        (298.0, 27.0),
        (331.0, 10.0),
        (3.0, 53.0),
        (36.0, 36.0),
        (63.0, 53.0),
        (91.0, 10.0),
        (118.0, 27.0),
        (151.0, 10.0),
        (183.0, 53.0),
        (216.0, 36.0),
        (243.0, 53.0),
        (271.0, 10.0),
    ]
    .map(|(degrees, minutes)| dms(degrees, minutes, 0.0));
    let rahu = horoscope.rahu.unwrap_or_default();
    let node = |longitude: f64| BhavaGraha {
        longitude,
        house: house(longitude),
    };
    let [sun, moon, mars, mercury, jupiter, venus, saturn] =
        horoscope.grahas.map(|at| BhavaGraha {
            longitude: at.longitude,
            house: at.house,
        });
    let grahas = [
        sun,
        moon,
        mars,
        mercury,
        jupiter,
        venus,
        saturn,
        node(rahu),
        node((rahu + 180.0).rem_euclid(360.0)),
    ];
    BhavaBalaChart {
        madhya,
        grahas,
        shadbala: [
            (424.94, 15.86),
            (389.80, -21.73),
            (298.14, 0.95),
            (537.02, 15.64),
            (433.77, -16.04),
            (376.15, 18.47),
            (389.21, 7.21),
        ],
        instant: horoscope.instant,
        day: (horoscope.sunrise, horoscope.sunset, horoscope.next_sunrise),
    }
}
