//! The **Harsha bala**: four places a planet of the annual chart is
//! "happy" in, five units each (`03-design/tajika-harsha.md`).
//!
//! The source is K. S. Charak, *A Textbook of Varshaphala*, ch. VI, and
//! behind it the *Tajika Nilakanthi*, Saṃjñā Tantra v. 76. The four parts:
//!
//! - **Sthana**, the place: each planet's house of joy, from the Sun the
//!   9th, 3rd, 6th, 1st, 11th, 5th and 12th;
//! - **Uchcha-Swakshetra**: its exaltation sign or a sign it owns;
//! - **Stri-Purusha**: a female planet in a feminine house (1, 2, 3, 7, 8,
//!   9), a male one in a masculine house;
//! - **Dina-Ratri**: a male planet in a year opening by day, a female one
//!   by night.
//!
//! The genders are Tajika's own and not the catalogue's: the Sun, Mars and
//! Jupiter are male, and the Moon, Mercury, Venus and Saturn female, for
//! "there is no neuter here". Houses are whole signs from the annual
//! lagna, "three signs at a time from the lagna's sign".
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_tajika::{AnnualSky, HarshaGrade, HarshaRules, harsha};
//!
//! // The source's Example Chart: its forty-first year, by day, Scorpio
//! // rising (Table VI-1).
//! let at = |sign: f64, deg: f64| sign * 30.0 + deg;
//! let sky = AnnualSky {
//!     sun_deg: at(4.0, 3.8), moon_deg: at(1.0, 9.7), mars_deg: at(7.0, 7.7),
//!     mercury_deg: at(4.0, 18.3), jupiter_deg: at(8.0, 9.6), venus_deg: at(4.0, 21.8),
//!     saturn_deg: at(6.0, 17.2),
//! };
//! let seven = harsha(&sky, at(7.0, 9.4), true, HarshaRules::default())?;
//! let sun = seven[0];
//! assert_eq!(sun.graha, Graha::Sun);
//! assert_eq!(sun.total.units(), 15);
//! assert_eq!(sun.grade, HarshaGrade::PoornaBali);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::house::House;

use crate::SEVEN;
use crate::bala::{AnnualSky, Bala, finite_longitude, sign_of_longitude};

/// What each of the four parts is worth, in whole units.
pub const HARSHA_PART_UNITS: i64 = 5;

/// Where the sources differ on the Harsha bala, each a named reading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase")]
pub struct HarshaRules {
    /// Venus's house of joy.
    pub venus: VenusPlace,
}

/// Venus's house of joy, which the Sthana part is read from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VenusPlace {
    /// The 5th: the *Tajika Nilakanthi*'s verse (*putra*, the house of
    /// children) and the source's own.
    #[default]
    Fifth,
    /// The 12th, as a widely used program reads it: measured over 600
    /// charts by its values alone, where it and this differ in nothing
    /// else (`03-design/tajika-harsha.md`).
    Twelfth,
}

/// What the source calls a planet by its Harsha bala.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarshaGrade {
    /// No part: without strength.
    Nirbala,
    /// One part, five units: weak.
    Alpabali,
    /// Two parts, ten units: of medium strength.
    MadhyaBali,
    /// Three parts, fifteen units: fully strong, and as much as a planet
    /// usually reaches.
    PoornaBali,
    /// All four, twenty units: "extraordinarily strong", which the source
    /// calls rare — a night year with an exalted Moon in the third, say.
    Extraordinary,
}

impl HarshaGrade {
    /// The grade of so many parts held, 0 to 4; anything above is the
    /// highest, since no planet holds more than four.
    #[must_use]
    pub const fn of_parts(parts: u8) -> HarshaGrade {
        match parts {
            0 => HarshaGrade::Nirbala,
            1 => HarshaGrade::Alpabali,
            2 => HarshaGrade::MadhyaBali,
            3 => HarshaGrade::PoornaBali,
            _ => HarshaGrade::Extraordinary,
        }
    }
}

/// One planet's Harsha bala: which of the four parts it holds, what they
/// add to, and its grade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[expect(
    clippy::struct_excessive_bools,
    reason = "four independent parts of one printed definition, any of which may hold at once; \
              collapsing them into a count would lose the part a reader asks for. \
              `Harsha::parts` is how a consumer walks them without naming fields."
)]
pub struct Harsha {
    /// Whose strength this is.
    pub graha: Graha,
    /// The house it stands in, whole signs from the annual lagna: what the
    /// first and third parts are read from.
    pub house: House,
    /// In its house of joy (the Prathama bala).
    pub sthana: bool,
    /// In its exaltation sign or a sign it owns (the Dwitiya bala).
    pub uchcha_swakshetra: bool,
    /// In a house of its own gender (the Tritiya bala).
    pub stri_purusha: bool,
    /// In a year opening at its own part of the day (the Chaturtha bala).
    pub dina_ratri: bool,
    /// The parts held, five units each: 0 to 20.
    pub total: Bala,
    /// What the source calls that total.
    pub grade: HarshaGrade,
}

impl Harsha {
    /// The four parts in the source's order, each named and whether it is
    /// held: the order the source numbers them, first to fourth, so a part
    /// cannot be relabelled by being moved.
    #[must_use]
    pub const fn parts(&self) -> [(&'static str, bool); 4] {
        [
            ("in its house of joy", self.sthana),
            ("in its exaltation or own sign", self.uchcha_swakshetra),
            ("in a house of its own gender", self.stri_purusha),
            (
                "in a year opening at its own part of the day",
                self.dina_ratri,
            ),
        ]
    }

    /// How many of the four parts it holds, 0 to 4.
    #[must_use]
    pub const fn held(&self) -> u8 {
        self.sthana as u8
            + self.uchcha_swakshetra as u8
            + self.stri_purusha as u8
            + self.dina_ratri as u8
    }
}

/// The Harsha bala of all seven, in the catalogue's order.
///
/// `lagna_deg` is the annual chart's lagna, whose sign the houses are
/// counted from, and `by_day` whether the year opened between sunrise and
/// sunset.
///
/// # Errors
///
/// `INVALID_ARG` for a longitude that is not a number, named by its field.
pub fn harsha(
    sky: &AnnualSky,
    lagna_deg: f64,
    by_day: bool,
    rules: HarshaRules,
) -> Result<[Harsha; 7], Error> {
    sky.check()?;
    finite_longitude("lagna_deg", lagna_deg)?;
    let lagna = sign_of_longitude(lagna_deg);
    Ok(SEVEN.map(|graha| one(graha, sky.sign_of(graha), lagna, by_day, rules)))
}

/// One planet's four parts.
fn one(graha: Graha, sign: Rashi, lagna: Rashi, by_day: bool, rules: HarshaRules) -> Harsha {
    let house = House::between(lagna, sign);
    let female = is_female(graha);
    let attributes = graha.attributes();
    let mut held = Harsha {
        graha,
        house,
        sthana: house == joy(graha, rules),
        uchcha_swakshetra: attributes.exaltation.is_some_and(|at| at.sign == sign)
            || attributes.own.contains(&sign),
        stri_purusha: is_feminine(house) == female,
        dina_ratri: by_day != female,
        total: Bala::default(),
        grade: HarshaGrade::Nirbala,
    };
    let parts = held.held();
    held.total = Bala::new(i64::from(parts) * HARSHA_PART_UNITS, 0, 0);
    held.grade = HarshaGrade::of_parts(parts);
    held
}

/// Each planet's house of joy, from the Sun: *nanda tri ṣaṭ* [lagna]
/// *bhava putra vyayā* — 9, 3, 6, 1, 11, 5, 12.
fn joy(graha: Graha, rules: HarshaRules) -> House {
    let [
        first,
        _,
        third,
        _,
        fifth,
        sixth,
        _,
        _,
        ninth,
        _,
        eleventh,
        twelfth,
    ] = House::ALL;
    match graha {
        Graha::Moon => third,
        Graha::Mars => sixth,
        Graha::Mercury => first,
        Graha::Jupiter => eleventh,
        Graha::Venus => match rules.venus {
            VenusPlace::Fifth => fifth,
            VenusPlace::Twelfth => twelfth,
        },
        Graha::Saturn => twelfth,
        // The Sun, and anything the seven do not include, which is never
        // asked.
        _ => ninth,
    }
}

/// Tajika's genders: the Moon, Mercury, Venus and Saturn female, the Sun,
/// Mars and Jupiter male, and no neuter.
const fn is_female(graha: Graha) -> bool {
    matches!(
        graha,
        Graha::Moon | Graha::Mercury | Graha::Venus | Graha::Saturn
    )
}

/// The feminine houses, three at a time from the lagna: 1–3 and 7–9.
const fn is_feminine(house: House) -> bool {
    matches!(house.get(), 1..=3 | 7..=9)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they asked for"
    )]

    use super::{AnnualSky, Harsha, HarshaGrade, HarshaRules, VenusPlace, harsha};
    use teistro_core::catalogue::Graha;

    fn sky_of(deg: [f64; 7]) -> AnnualSky {
        AnnualSky {
            sun_deg: deg[0],
            moon_deg: deg[1],
            mars_deg: deg[2],
            mercury_deg: deg[3],
            jupiter_deg: deg[4],
            venus_deg: deg[5],
            saturn_deg: deg[6],
        }
    }

    fn row(found: &Harsha) -> [bool; 4] {
        [
            found.sthana,
            found.uchcha_swakshetra,
            found.stri_purusha,
            found.dina_ratri,
        ]
    }

    /// Table VI-1, all twenty-eight cells: the source's Example Chart, the
    /// forty-first year by day, lagna Scorpio 9°26′.
    #[test]
    fn the_sources_worked_chart_is_reproduced_cell_for_cell() {
        let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
        let sky = sky_of([
            at(4.0, 3.0, 50.0),
            at(1.0, 9.0, 40.0),
            at(7.0, 7.0, 42.0),
            at(4.0, 18.0, 20.0),
            at(8.0, 9.0, 38.0),
            at(4.0, 21.0, 45.0),
            at(6.0, 17.0, 13.0),
        ]);
        let found = harsha(&sky, at(7.0, 9.0, 26.0), true, HarshaRules::default()).unwrap();
        // Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn: the four
        // parts as the table prints them, a five as `true`.
        let printed = [
            [false, true, true, true],
            [false, true, true, false],
            [false, true, false, true],
            [false, false, false, false],
            [false, true, false, true],
            [false, false, false, false],
            [true, true, false, false],
        ];
        let totals = [15, 10, 10, 0, 10, 0, 10];
        for (k, one) in found.iter().enumerate() {
            assert_eq!(row(one), printed[k], "{:?}", one.graha);
            assert_eq!(one.total.units(), totals[k], "{:?}", one.graha);
        }
        assert_eq!(found[0].grade, HarshaGrade::PoornaBali);
        assert_eq!(found[3].grade, HarshaGrade::Nirbala);
        assert_eq!(found[1].grade, HarshaGrade::MadhyaBali);
    }

    /// The source's own example of the rare twenty: a year opening at
    /// night with an exalted Moon in the third.
    #[test]
    fn an_exalted_moon_in_the_third_by_night_holds_all_four() {
        // Lagna Pisces, so the Moon's Taurus, its exaltation, is the third.
        let sky = sky_of([100.0, 40.0, 200.0, 220.0, 250.0, 280.0, 310.0]);
        let moon = harsha(&sky, 345.0, false, HarshaRules::default()).unwrap()[1];
        assert_eq!(moon.house.get(), 3);
        assert_eq!(moon.held(), 4);
        assert!(moon.parts().iter().all(|(_, holds)| *holds));
        assert_eq!(moon.total.units(), 20);
        assert_eq!(moon.grade, HarshaGrade::Extraordinary);
    }

    /// A black-box probe of the program: the lagna, the seven, whether the
    /// year opened by day, and its seven totals.
    type Probe = (f64, [f64; 7], bool, [i64; 7]);

    /// Twelve of the 600 probes, the first six where Venus's place matters
    /// and the rest where it does not.
    const PROGRAM_PROBES: [Probe; 12] = [
        (
            289.103_861,
            [
                76.430_281,
                222.273_433,
                169.437_440,
                55.705_541,
                185.989_550,
                44.594_586,
                171.188_354,
            ],
            false,
            [5, 5, 0, 5, 5, 10, 10],
        ),
        (
            285.710_180,
            [
                56.331_926,
                120.492_680,
                6.914_276,
                52.844_249,
                38.863_989,
                58.303_397,
                324.082_664,
            ],
            false,
            [5, 10, 10, 5, 5, 10, 15],
        ),
        (
            26.429_663,
            [
                142.508_764,
                204.055_393,
                50.694_154,
                168.927_081,
                359.162_188,
                122.248_228,
                94.919_942,
            ],
            false,
            [10, 10, 0, 10, 10, 5, 5],
        ),
        (
            131.057_458,
            [
                206.178_242,
                43.289_246,
                138.624_728,
                190.819_134,
                182.692_691,
                253.243_908,
                172.852_009,
            ],
            false,
            [0, 10, 0, 10, 0, 5, 10],
        ),
        (
            57.477_148,
            [
                10.830_382,
                154.970_749,
                150.398_725,
                29.836_742,
                175.655_636,
                19.326_936,
                18.234_092,
            ],
            true,
            [15, 0, 10, 0, 10, 5, 5],
        ),
        (
            152.203_742,
            [
                186.690_459,
                14.580_800,
                224.277_466,
                205.332_572,
                154.944_692,
                148.329_893,
                159.537_576,
            ],
            false,
            [0, 10, 5, 10, 0, 10, 10],
        ),
        (
            59.353_257,
            [
                258.427_792,
                273.062_145,
                206.734_432,
                273.535_503,
                111.838_801,
                211.755_551,
                59.433_091,
            ],
            true,
            [5, 5, 15, 5, 10, 5, 5],
        ),
        (
            292.208_718,
            [
                348.388_579,
                359.034_957,
                213.775_323,
                7.298_203,
                257.302_811,
                328.866_713,
                200.705_944,
            ],
            false,
            [0, 15, 10, 5, 10, 10, 10],
        ),
        (
            259.303_202,
            [
                76.198_572,
                199.945_661,
                358.789_972,
                76.515_691,
                85.139_337,
                44.848_565,
                268.265_511,
            ],
            false,
            [0, 5, 5, 15, 0, 10, 10],
        ),
        (
            119.290_877,
            [
                274.699_247,
                2.656_839,
                268.751_615,
                284.908_145,
                197.617_036,
                275.322_299,
                304.284_685,
            ],
            false,
            [0, 5, 10, 10, 5, 10, 15],
        ),
        (
            47.211_659,
            [
                325.070_640,
                260.409_373,
                53.607_673,
                333.120_574,
                99.547_911,
                355.506_342,
                278.784_872,
            ],
            true,
            [10, 5, 5, 0, 10, 5, 10],
        ),
        (
            163.315_968,
            [
                198.534_611,
                194.037_506,
                199.253_545,
                217.678_541,
                135.605_291,
                152.066_490,
                276.410_279,
            ],
            false,
            [0, 10, 0, 10, 5, 10, 10],
        ),
    ];

    /// A widely used program's values, read as a black box (`CLEAN_ROOM`
    /// rule 3): the lagna, the seven and whether the year opened by day,
    /// with its seven totals. It reads Venus's joy as the 12th; the first
    /// six rows are where that matters and the rest where it does not.
    #[test]
    fn venus_in_the_twelfth_reproduces_the_program() {
        let program = HarshaRules {
            venus: VenusPlace::Twelfth,
        };
        for (k, (lagna, deg, by_day, want)) in PROGRAM_PROBES.iter().enumerate() {
            let totals = |rules| {
                harsha(&sky_of(*deg), *lagna, *by_day, rules)
                    .unwrap()
                    .map(|one| one.total.units())
            };
            assert_eq!(totals(program), *want, "probe {k}");
            // The verse's reading differs from the program's in Venus
            // alone, and only in the first six.
            let verse = totals(HarshaRules::default());
            assert_eq!(verse[..5], want[..5], "probe {k}");
            assert_eq!(verse[6], want[6], "probe {k}");
            assert_eq!(verse[5] != want[5], k < 6, "probe {k}");
        }
    }

    #[test]
    fn the_genders_are_tajikas_and_every_part_answers_by_day_and_night() {
        let sky = sky_of([0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0]);
        for by_day in [true, false] {
            let found = harsha(&sky, 0.0, by_day, HarshaRules::default()).unwrap();
            for one in &found {
                let female = matches!(
                    one.graha,
                    Graha::Moon | Graha::Mercury | Graha::Venus | Graha::Saturn
                );
                // Mercury and Saturn are female here, not the catalogue's
                // neuter: by night they take the fourth part.
                assert_eq!(one.dina_ratri, by_day != female, "{:?}", one.graha);
                assert_eq!(one.total.units(), 5 * i64::from(one.held()));
            }
        }
    }

    /// Twenty is out of reach for the Sun, Venus and Saturn under the
    /// verse, whatever the sky: each one's house of joy is of the other
    /// gender. Walked over every sign, every lagna and both halves of the
    /// day, for all seven, so the four that can reach it are shown to.
    #[test]
    fn only_a_joy_of_ones_own_gender_can_reach_twenty() {
        for (at, graha) in super::SEVEN.iter().enumerate() {
            let mut reaches = false;
            for sign in 0..12 {
                for lagna in 0..12 {
                    for by_day in [true, false] {
                        let mut deg = [0.0; 7];
                        deg[at] = f64::from(sign) * 30.0 + 15.0;
                        let found = harsha(
                            &sky_of(deg),
                            f64::from(lagna) * 30.0,
                            by_day,
                            HarshaRules::default(),
                        )
                        .unwrap();
                        reaches |= found[at].grade == HarshaGrade::Extraordinary;
                    }
                }
            }
            let never = matches!(graha, Graha::Sun | Graha::Venus | Graha::Saturn);
            assert_eq!(reaches, !never, "{graha:?}");
        }
    }

    #[test]
    fn a_longitude_that_is_not_a_number_is_refused_by_name() {
        let sky = sky_of([0.0; 7]);
        let refused = harsha(&sky, f64::NAN, true, HarshaRules::default()).unwrap_err();
        assert_eq!(refused.field(), Some("lagna_deg"));
        let mut bad = sky;
        bad.venus_deg = f64::INFINITY;
        let refused = harsha(&bad, 0.0, true, HarshaRules::default()).unwrap_err();
        assert_eq!(refused.field(), Some("venus_deg"));
    }

    #[test]
    fn the_rules_read_camel_case_and_fill_the_rest() {
        let rules: HarshaRules = serde_json::from_str(r#"{"venus":"TWELFTH"}"#).unwrap();
        assert_eq!(rules.venus, VenusPlace::Twelfth);
        let empty: HarshaRules = serde_json::from_str("{}").unwrap();
        assert_eq!(empty, HarshaRules::default());
        assert_eq!(
            serde_json::to_string(&HarshaRules::default()).unwrap(),
            r#"{"venus":"FIFTH"}"#
        );
    }
}
