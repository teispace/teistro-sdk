//! The **Panchavargiya bala**: the five-fold strength the lord of the year
//! is chosen by (`03-design/panchavargiya.md`).
//!
//! Five parts, each a maximum reduced by how a planet stands to the lord of
//! the division it is in — the whole in its own, three quarters in a
//! friend's, a half in a neutral's, a quarter in an enemy's:
//!
//! | part | at most |
//! |---|---|
//! | Griha (Kshetra), the sign | 30 |
//! | Uchcha, the arc from debilitation | 20 |
//! | Hudda, the Tajika term | 15 |
//! | Drekkana, the Tajika decanate | 10 |
//! | Navamsha | 5 |
//!
//! Eighty at most, divided by four for a **Vishwa bala** out of twenty.
//!
//! Two things here are Tajika's own and not Parashari. Friendship is
//! **positional**, read off the annual chart — friends at houses 3, 5, 9
//! and 11 from each other, enemies at 1, 4, 7 and 10, neutrals at the rest
//! — so a planet sharing a sign with its own lord is an *enemy* of it. And
//! the decanate lords are not the Parashari ones (crux C108's sibling,
//! C109's neighbour): they run in one cycle from Mars.
//!
//! The arithmetic is **exact**, in sub-sub units of which a unit holds
//! 3600, because the source works in units, sub-units and sub-sub units
//! and says the Vishwa bala is "preferable to do so up to sub-sub units".
//! Floating point would lose its worked chart's last digit.

use core::fmt;

use serde::{Deserialize, Serialize};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Graha, Rashi, Varga};
use teistro_core::error::Error;
use teistro_core::quantity::Degrees;
use teistro_vargas::{Scheme, place};

/// A strength, exact, in sub-sub units.
///
/// The source writes one as `14:20:15` — units, sub-units, sub-sub units,
/// sixty of each in the next — and its worked chart's last figure is a
/// sub-sub unit, so nothing here rounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Bala(i64);

impl Bala {
    /// The sub-sub units in one unit.
    pub const UNIT: i64 = 60 * 60;

    /// The same, as a float: the one place a strength meets floating
    /// point, where an arc in degrees becomes units.
    const UNIT_F64: f64 = 3600.0;

    /// A strength of that many sub-sub units.
    #[must_use]
    pub const fn of_sub_sub(sub_sub: i64) -> Bala {
        Bala(sub_sub)
    }

    /// A strength written as the source writes one.
    #[must_use]
    pub const fn new(units: i64, sub_units: i64, sub_sub: i64) -> Bala {
        Bala(units * Bala::UNIT + sub_units * 60 + sub_sub)
    }

    /// Whole units, the figure a reader compares.
    #[must_use]
    pub const fn units(self) -> i64 {
        self.0 / Bala::UNIT
    }

    /// The sub-units after the units.
    #[must_use]
    pub const fn sub_units(self) -> i64 {
        (self.0 % Bala::UNIT) / 60
    }

    /// The sub-sub units after those.
    #[must_use]
    pub const fn sub_sub(self) -> i64 {
        self.0 % 60
    }

    /// The whole of it in sub-sub units, which is what to compare and sum.
    #[must_use]
    pub const fn as_sub_sub(self) -> i64 {
        self.0
    }

    /// As a decimal, for a caller that wants one; the exact value is
    /// [`Self::as_sub_sub`].
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "a strength is at most twenty units, 72 000 sub-sub units, exact in f64"
    )]
    pub fn as_f64(self) -> f64 {
        self.0 as f64 / Bala::UNIT as f64
    }

    /// This strength cut to whole sub-units, the remainder dropped.
    ///
    /// The Uchcha bala is the one part the source truncates — "ignore the
    /// remainder from the second division" — and it says so in the verse
    /// of its own arithmetic rather than as a rounding convenience, so it
    /// is done here and named.
    #[must_use]
    const fn to_sub_units(self) -> Bala {
        Bala(self.0 - self.0 % 60)
    }

    /// This strength under a relation: whole, three quarters, a half or a
    /// quarter, exactly.
    #[must_use]
    const fn under(self, relation: Relation) -> Bala {
        Bala(self.0 * relation.quarters() / 4)
    }
}

impl fmt::Display for Bala {
    /// `14:20:15`, as the source writes one.
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            out,
            "{:02}:{:02}:{:02}",
            self.units(),
            self.sub_units(),
            self.sub_sub()
        )
    }
}

/// How one planet stands to another, **by their places in the annual
/// chart** and not by nature.
///
/// Tajika's own: friends at houses 3, 5, 9 and 11 from each other, enemies
/// at 1, 4, 7 and 10, neutral at 2, 6, 8 and 12. A planet in the same sign
/// as another is at house 1 from it, so **a planet sharing a sign with the
/// lord of that sign is its enemy** — which surprises a reader who expects
/// the Parashari answer and is what the source's worked chart does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    /// The planet is the lord of the division: the whole of it.
    Own,
    /// Three quarters.
    Friend,
    /// A half.
    Neutral,
    /// A quarter.
    Enemy,
}

impl Relation {
    /// How many quarters of a division's maximum it gives.
    #[must_use]
    pub const fn quarters(self) -> i64 {
        match self {
            Relation::Own => 4,
            Relation::Friend => 3,
            Relation::Neutral => 2,
            Relation::Enemy => 1,
        }
    }

    /// How `graha` stands to `lord`, both placed in the annual chart.
    #[must_use]
    pub fn between(graha: Graha, lord: Graha, sky: &AnnualSky) -> Relation {
        if graha == lord {
            return Relation::Own;
        }
        let house = houses_from(sky.sign_of(graha), sky.sign_of(lord));
        match house {
            3 | 5 | 9 | 11 => Relation::Friend,
            1 | 4 | 7 | 10 => Relation::Enemy,
            _ => Relation::Neutral,
        }
    }
}

/// The house `to` stands in, counted from `from` inclusively: 1 to 12.
fn houses_from(from: Rashi, to: Rashi) -> u16 {
    (to.id() + 12 - from.id()) % 12 + 1
}

/// The seven the Panchavargiya bala is computed for; the nodes hold no
/// division and take no strength here.
pub const SEVEN: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// Where the seven stand in an annual chart, sidereal degrees.
///
/// Named fields and not an array: every one of them is a longitude, and a
/// caller filling an array in the wrong order would get an answer.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AnnualSky {
    /// The Sun.
    pub sun_deg: f64,
    /// The Moon.
    pub moon_deg: f64,
    /// Mars.
    pub mars_deg: f64,
    /// Mercury.
    pub mercury_deg: f64,
    /// Jupiter.
    pub jupiter_deg: f64,
    /// Venus.
    pub venus_deg: f64,
    /// Saturn.
    pub saturn_deg: f64,
}

impl AnnualSky {
    /// Where one of the seven stands, degrees folded into a circle.
    #[must_use]
    pub fn longitude_of(&self, graha: Graha) -> f64 {
        let raw = match graha {
            Graha::Moon => self.moon_deg,
            Graha::Mars => self.mars_deg,
            Graha::Mercury => self.mercury_deg,
            Graha::Jupiter => self.jupiter_deg,
            Graha::Venus => self.venus_deg,
            Graha::Saturn => self.saturn_deg,
            // The Sun, and anything the catalogue adds that this does not
            // place: the nodes hold no division and are never asked.
            _ => self.sun_deg,
        };
        raw.rem_euclid(360.0)
    }

    /// The sign one of the seven stands in.
    #[must_use]
    pub fn sign_of(&self, graha: Graha) -> Rashi {
        sign_of_longitude(self.longitude_of(graha))
    }

    /// Every longitude is a number, refused by the field that is not.
    fn check(&self) -> Result<(), Error> {
        for (field, value) in [
            ("sun_deg", self.sun_deg),
            ("moon_deg", self.moon_deg),
            ("mars_deg", self.mars_deg),
            ("mercury_deg", self.mercury_deg),
            ("jupiter_deg", self.jupiter_deg),
            ("venus_deg", self.venus_deg),
            ("saturn_deg", self.saturn_deg),
        ] {
            if !value.is_finite() {
                return Err(Error::invalid_arg(format!(
                    "{field} is a longitude that is not a number"
                ))
                .with_field(field.to_owned()));
            }
        }
        Ok(())
    }
}

/// The sign a longitude stands in.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a longitude folded into 0..360 divides by thirty into 0..12"
)]
pub(crate) fn sign_of_longitude(longitude_deg: f64) -> Rashi {
    let index = (longitude_deg.rem_euclid(360.0) / 30.0) as u16 % 12;
    Rashi::from_id(index).unwrap_or(Rashi::Aries)
}

/// One planet's five parts, what they add to, and the Vishwa bala.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Panchavargiya {
    /// Whose strength this is.
    pub graha: Graha,
    /// The sign it stands in, carried so a reader of the strength need not
    /// found the chart again to know where it was read from.
    pub sign: Rashi,
    /// The sign, at most 30.
    pub griha: Bala,
    /// The arc from debilitation, at most 20.
    pub uchcha: Bala,
    /// The Tajika term, at most 15.
    pub hudda: Bala,
    /// The Tajika decanate, at most 10.
    pub drekkana: Bala,
    /// The navamsha, at most 5.
    pub navamsha: Bala,
    /// The five added, at most 80.
    pub total: Bala,
    /// The total quartered: the figure the year lord is chosen by, at most
    /// 20.
    pub vishwa: Bala,
}

/// The Panchavargiya bala of all seven, in the catalogue's order.
///
/// # Errors
///
/// A longitude that is not a number, named by its field.
pub fn panchavargiya(sky: &AnnualSky) -> Result<[Panchavargiya; 7], Error> {
    sky.check()?;
    Ok(SEVEN.map(|graha| one(graha, sky)))
}

/// One planet's strength.
fn one(graha: Graha, sky: &AnnualSky) -> Panchavargiya {
    let longitude = sky.longitude_of(graha);
    let sign = sign_of_longitude(longitude);
    let kshetra = Bala::new(30, 0, 0).under(Relation::between(graha, sign.attributes().lord, sky));
    let hudda = Bala::new(15, 0, 0).under(Relation::between(graha, hudda_lord(longitude), sky));
    let drekkana =
        Bala::new(10, 0, 0).under(Relation::between(graha, drekkana_lord(longitude), sky));
    let navamsha =
        Bala::new(5, 0, 0).under(Relation::between(graha, navamsha_lord(longitude), sky));
    let uchcha = uchcha(graha, longitude);
    let total = Bala::of_sub_sub(
        kshetra.as_sub_sub()
            + uchcha.as_sub_sub()
            + hudda.as_sub_sub()
            + drekkana.as_sub_sub()
            + navamsha.as_sub_sub(),
    );
    Panchavargiya {
        graha,
        sign,
        griha: kshetra,
        uchcha,
        hudda,
        drekkana,
        navamsha,
        total,
        vishwa: Bala::of_sub_sub(total.as_sub_sub() / 4),
    }
}

/// Each planet's **debilitation** point, degrees: where its Uchcha bala is
/// nothing, six signs from its exaltation.
const DEBILITATION_DEG: [(Graha, f64); 7] = [
    (Graha::Sun, 190.0),
    (Graha::Moon, 213.0),
    (Graha::Mars, 118.0),
    (Graha::Mercury, 345.0),
    (Graha::Jupiter, 275.0),
    (Graha::Venus, 177.0),
    (Graha::Saturn, 20.0),
];

/// The Uchcha bala: the arc from the debilitation point, a unit for every
/// nine degrees, cut to whole sub-units.
///
/// Twenty units at deep exaltation, nothing at deep debilitation, and the
/// 180 degrees between them at a unit per nine. An arc past 180 is
/// measured the short way round, so the strength rises to the exaltation
/// point from either side.
fn uchcha(graha: Graha, longitude_deg: f64) -> Bala {
    let Some((_, debilitation)) = DEBILITATION_DEG.into_iter().find(|(who, _)| *who == graha)
    else {
        return Bala::default();
    };
    let mut arc = (longitude_deg - debilitation).rem_euclid(360.0);
    if arc > 180.0 {
        arc = 360.0 - arc;
    }
    // A unit for every nine degrees, exactly, then cut to sub-units as the
    // source's own arithmetic does.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "an arc of at most 180 degrees is at most 72 000 sub-sub units"
    )]
    let sub_sub = (arc / 9.0 * Bala::UNIT_F64) as i64;
    Bala::of_sub_sub(sub_sub).to_sub_units()
}

/// The **Hudda** table: five terms in each sign, in degrees, with their
/// lords (K.S. Charak, *A Textbook of Varshaphala*, Table VI-4).
///
/// Read off the rendered page and not the OCR. Its degrees are the
/// Egyptian terms exactly, and 57 of its 60 lords are; Gemini,
/// Sagittarius and Aquarius each transpose two adjacent lords, which the
/// source's own worked chart never touches (crux C109). Two invariants a
/// test holds over the whole of it: every sign's widths sum to thirty, and
/// every sign carries Mars, Mercury, Jupiter, Venus and Saturn once each.
const HUDDA: [[(u8, Graha); 5]; 12] = [
    // Aries
    [
        (6, Graha::Jupiter),
        (6, Graha::Venus),
        (8, Graha::Mercury),
        (5, Graha::Mars),
        (5, Graha::Saturn),
    ],
    // Taurus
    [
        (8, Graha::Venus),
        (6, Graha::Mercury),
        (8, Graha::Jupiter),
        (5, Graha::Saturn),
        (3, Graha::Mars),
    ],
    // Gemini — Venus and Jupiter the other way round from the Egyptian.
    [
        (6, Graha::Mercury),
        (6, Graha::Venus),
        (5, Graha::Jupiter),
        (7, Graha::Mars),
        (6, Graha::Saturn),
    ],
    // Cancer
    [
        (7, Graha::Mars),
        (6, Graha::Venus),
        (6, Graha::Mercury),
        (7, Graha::Jupiter),
        (4, Graha::Saturn),
    ],
    // Leo
    [
        (6, Graha::Jupiter),
        (5, Graha::Venus),
        (7, Graha::Saturn),
        (6, Graha::Mercury),
        (6, Graha::Mars),
    ],
    // Virgo
    [
        (7, Graha::Mercury),
        (10, Graha::Venus),
        (4, Graha::Jupiter),
        (7, Graha::Mars),
        (2, Graha::Saturn),
    ],
    // Libra
    [
        (6, Graha::Saturn),
        (8, Graha::Mercury),
        (7, Graha::Jupiter),
        (7, Graha::Venus),
        (2, Graha::Mars),
    ],
    // Scorpio
    [
        (7, Graha::Mars),
        (4, Graha::Venus),
        (8, Graha::Mercury),
        (5, Graha::Jupiter),
        (6, Graha::Saturn),
    ],
    // Sagittarius — Mars and Saturn the other way round.
    [
        (12, Graha::Jupiter),
        (5, Graha::Venus),
        (4, Graha::Mercury),
        (5, Graha::Mars),
        (4, Graha::Saturn),
    ],
    // Capricorn
    [
        (7, Graha::Mercury),
        (7, Graha::Jupiter),
        (8, Graha::Venus),
        (4, Graha::Saturn),
        (4, Graha::Mars),
    ],
    // Aquarius — Venus and Mercury the other way round.
    [
        (7, Graha::Venus),
        (6, Graha::Mercury),
        (7, Graha::Jupiter),
        (5, Graha::Mars),
        (5, Graha::Saturn),
    ],
    // Pisces
    [
        (12, Graha::Venus),
        (4, Graha::Jupiter),
        (3, Graha::Mercury),
        (9, Graha::Mars),
        (2, Graha::Saturn),
    ],
];

/// The lord of the Hudda a longitude falls in.
#[must_use]
pub fn hudda_lord(longitude_deg: f64) -> Graha {
    let longitude = longitude_deg.rem_euclid(360.0);
    let sign = sign_of_longitude(longitude);
    let within = longitude % 30.0;
    // Twelve signs, twelve rows, which the table's own type fixes, so the
    // sign always has one; answering Saturn rather than panicking in a
    // consumer's process is the only thing to do if that ever changes.
    let Some(terms) = HUDDA.get(usize::from(sign.id() % 12)) else {
        return Graha::Saturn;
    };
    let mut edge = 0.0;
    for (width, lord) in terms {
        edge += f64::from(*width);
        if within < edge {
            return *lord;
        }
    }
    // The widths fill a sign, which a test holds; the last term answers a
    // longitude on the very edge rather than nothing answering.
    terms.last().map_or(Graha::Saturn, |(_, lord)| *lord)
}

/// The Tajika decanate lords, in one cycle from Mars — the weekday order
/// begun there.
const DREKKANA_CYCLE: [Graha; 7] = [
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Sun,
    Graha::Moon,
];

/// The lord of the **Tajika** decanate a longitude falls in, which is not
/// the Parashari one.
///
/// The source gives the rule in prose beside its table: Mars takes the
/// first decanate of Aries, the first decanates run through the cycle sign
/// by sign, and each next decanate of Aries starts five along — which is
/// `(sign + 5 × decanate) mod 7` and reproduces all 36 printed cells.
#[must_use]
pub fn drekkana_lord(longitude_deg: f64) -> Graha {
    let longitude = longitude_deg.rem_euclid(360.0);
    let sign = usize::from(sign_of_longitude(longitude).id() % 12);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude within a sign divides by ten into 0..3"
    )]
    let decanate = ((longitude % 30.0) / 10.0) as usize % 3;
    // The cycle has seven members and the index is taken modulo seven, so
    // the fallback cannot be reached.
    DREKKANA_CYCLE
        .get((sign + 5 * decanate) % 7)
        .copied()
        .unwrap_or(Graha::Mars)
}

/// The lord of the navamsha a longitude falls in.
///
/// The navamsha here is the Parashari one, which the source says outright,
/// so it is taken from `teistro_vargas` rather than written again.
#[must_use]
pub fn navamsha_lord(longitude_deg: f64) -> Graha {
    let Ok(degrees) = Degrees::try_new(longitude_deg.rem_euclid(360.0)) else {
        // Checked before anything is computed; a caller reaching here has
        // a longitude that is not a number, and Aries' lord is as good an
        // answer as a panic in their process.
        return Rashi::Aries.attributes().lord;
    };
    place(&Scheme::of(Varga::D9), Nas::from_degrees(degrees))
        .sign
        .attributes()
        .lord
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index what they asked for"
    )]

    use super::{
        AnnualSky, Bala, DREKKANA_CYCLE, HUDDA, Panchavargiya, Relation, SEVEN, drekkana_lord,
        hudda_lord, panchavargiya,
    };
    use teistro_core::catalogue::Graha;

    /// The source's Example Chart: the annual chart of its forty-first
    /// year (Table VI-10's subject).
    fn worked() -> AnnualSky {
        let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
        AnnualSky {
            sun_deg: at(4.0, 3.0, 50.0),
            moon_deg: at(1.0, 9.0, 40.0),
            mars_deg: at(7.0, 7.0, 42.0),
            mercury_deg: at(4.0, 18.0, 20.0),
            jupiter_deg: at(8.0, 9.0, 38.0),
            venus_deg: at(4.0, 21.0, 45.0),
            saturn_deg: at(6.0, 17.0, 13.0),
        }
    }

    /// Table VI-10, every cell of it: the five parts, the total and the
    /// Vishwa bala of all seven planets, as the source prints them.
    ///
    /// Thirty-five component figures and fourteen more that depend on all
    /// of them, so a wrong relation, a wrong table cell, a wrong
    /// truncation or a wrong division lands here.
    #[test]
    fn the_sources_worked_chart_is_reproduced_cell_for_cell() {
        // graha: griha, uchcha, hudda, drekkana, navamsha, total, vishwa
        /// One printed row: the graha, its five parts, its total and its
        /// Vishwa bala, each written as the source writes one.
        type Row = (Graha, [(i64, i64); 5], (i64, i64), (i64, i64, i64));
        let printed: [Row; 7] = [
            (
                Graha::Sun,
                [(30, 0), (7, 21), (11, 15), (7, 30), (1, 15)],
                (57, 21),
                (14, 20, 15),
            ),
            (
                Graha::Moon,
                [(7, 30), (19, 15), (3, 45), (2, 30), (2, 30)],
                (35, 30),
                (8, 52, 30),
            ),
            (
                Graha::Mars,
                [(30, 0), (11, 4), (3, 45), (10, 0), (1, 15)],
                (56, 4),
                (14, 1, 0),
            ),
            (
                Graha::Mercury,
                [(7, 30), (17, 2), (15, 0), (7, 30), (5, 0)],
                (52, 2),
                (13, 0, 30),
            ),
            (
                Graha::Jupiter,
                [(30, 0), (2, 49), (15, 0), (7, 30), (3, 45)],
                (59, 4),
                (14, 46, 0),
            ),
            (
                Graha::Venus,
                [(7, 30), (3, 55), (3, 45), (2, 30), (5, 0)],
                (22, 40),
                (5, 40, 0),
            ),
            (
                Graha::Saturn,
                [(22, 30), (19, 41), (11, 15), (10, 0), (3, 45)],
                (67, 11),
                (16, 47, 45),
            ),
        ];
        let found = panchavargiya(&worked()).unwrap();
        for (row, (graha, parts, total, vishwa)) in printed.into_iter().enumerate() {
            let one: Panchavargiya = found[row];
            assert_eq!(one.graha, graha);
            let want = |(units, sub): (i64, i64)| Bala::new(units, sub, 0);
            assert_eq!(one.griha, want(parts[0]), "{graha:?} griha");
            assert_eq!(one.uchcha, want(parts[1]), "{graha:?} uchcha");
            assert_eq!(one.hudda, want(parts[2]), "{graha:?} hudda");
            assert_eq!(one.drekkana, want(parts[3]), "{graha:?} drekkana");
            assert_eq!(one.navamsha, want(parts[4]), "{graha:?} navamsha");
            assert_eq!(one.total, want(total), "{graha:?} total");
            assert_eq!(
                one.vishwa,
                Bala::new(vishwa.0, vishwa.1, vishwa.2),
                "{graha:?} vishwa"
            );
        }
    }

    /// The strongest of the source's worked chart is Saturn, and the
    /// strongest of its **office-bearers** is Jupiter, which is the figure
    /// the year lord is chosen by.
    #[test]
    fn the_worked_chart_ranks_as_the_source_says() {
        let found = panchavargiya(&worked()).unwrap();
        let strongest = found.iter().max_by_key(|one| one.vishwa).unwrap();
        assert_eq!(strongest.graha, Graha::Saturn);
        let bearers = [Graha::Jupiter, Graha::Sun, Graha::Mars];
        let best = found
            .iter()
            .filter(|one| bearers.contains(&one.graha))
            .max_by_key(|one| one.vishwa)
            .unwrap();
        assert_eq!(best.graha, Graha::Jupiter);
        assert_eq!(best.vishwa.to_string(), "14:46:00");
    }

    /// The whole of the Hudda table, held two ways at once: a sign's five
    /// widths fill it, and its five lords are the five that own one each.
    #[test]
    fn every_sign_of_the_hudda_fills_and_is_owned_once_each() {
        for (index, terms) in HUDDA.into_iter().enumerate() {
            let width: u16 = terms.iter().map(|(width, _)| u16::from(*width)).sum();
            assert_eq!(width, 30, "sign {index}");
            let mut lords: Vec<Graha> = terms.iter().map(|(_, lord)| *lord).collect();
            lords.sort_by_key(|lord| lord.id());
            lords.dedup();
            assert_eq!(lords.len(), 5, "sign {index}");
            assert!(!lords.contains(&Graha::Sun) && !lords.contains(&Graha::Moon));
        }
    }

    /// The source's own seven worked Hudda lords, which are the only cells
    /// of the sixty it checks.
    #[test]
    fn the_sources_worked_hudda_lords_are_reproduced() {
        let sky = worked();
        let printed = [
            (Graha::Sun, Graha::Jupiter),
            (Graha::Moon, Graha::Mercury),
            (Graha::Mars, Graha::Venus),
            (Graha::Mercury, Graha::Mercury),
            (Graha::Jupiter, Graha::Jupiter),
            (Graha::Venus, Graha::Mercury),
            (Graha::Saturn, Graha::Jupiter),
        ];
        for (graha, lord) in printed {
            assert_eq!(hudda_lord(sky.longitude_of(graha)), lord, "{graha:?}");
        }
    }

    /// The decanate rule against the source's printed 36 cells.
    #[test]
    fn the_rule_reproduces_the_printed_drekkana_table() {
        let printed: [[Graha; 12]; 3] = [
            [
                Graha::Mars,
                Graha::Mercury,
                Graha::Jupiter,
                Graha::Venus,
                Graha::Saturn,
                Graha::Sun,
                Graha::Moon,
                Graha::Mars,
                Graha::Mercury,
                Graha::Jupiter,
                Graha::Venus,
                Graha::Saturn,
            ],
            [
                Graha::Sun,
                Graha::Moon,
                Graha::Mars,
                Graha::Mercury,
                Graha::Jupiter,
                Graha::Venus,
                Graha::Saturn,
                Graha::Sun,
                Graha::Moon,
                Graha::Mars,
                Graha::Mercury,
                Graha::Jupiter,
            ],
            [
                Graha::Venus,
                Graha::Saturn,
                Graha::Sun,
                Graha::Moon,
                Graha::Mars,
                Graha::Mercury,
                Graha::Jupiter,
                Graha::Venus,
                Graha::Saturn,
                Graha::Sun,
                Graha::Moon,
                Graha::Mars,
            ],
        ];
        for (decanate, row) in printed.into_iter().enumerate() {
            for (sign, lord) in row.into_iter().enumerate() {
                #[expect(clippy::cast_precision_loss, reason = "twelve signs")]
                let longitude = sign as f64 * 30.0 + decanate as f64 * 10.0 + 5.0;
                assert_eq!(drekkana_lord(longitude), lord, "sign {sign}, {decanate}");
            }
        }
        // Seven lords, each taking every seventh cell: the cycle is the
        // whole of the rule and nothing else is in the table.
        assert_eq!(DREKKANA_CYCLE.len(), 7);
    }

    /// A planet sharing a sign with the lord of that sign is its **enemy**,
    /// not its friend: house one is an enemy's house in Tajika, and the
    /// source's worked Mercury and Venus in the Sun's Leo are why this is
    /// asserted rather than assumed.
    #[test]
    fn a_planet_with_its_own_lord_is_an_enemy_of_it() {
        let sky = worked();
        assert_eq!(
            Relation::between(Graha::Mercury, Graha::Sun, &sky),
            Relation::Enemy
        );
        assert_eq!(
            Relation::between(Graha::Sun, Graha::Sun, &sky),
            Relation::Own
        );
        // And Saturn in Venus's Libra, eleven houses from Venus: a friend.
        assert_eq!(
            Relation::between(Graha::Saturn, Graha::Venus, &sky),
            Relation::Friend
        );
    }

    /// The parts add to the total and the total quarters to the Vishwa
    /// bala, exactly, for every planet — no rounding anywhere.
    #[test]
    fn the_parts_add_and_the_total_quarters_exactly() {
        let found = panchavargiya(&worked()).unwrap();
        for one in found {
            let parts = one.griha.as_sub_sub()
                + one.uchcha.as_sub_sub()
                + one.hudda.as_sub_sub()
                + one.drekkana.as_sub_sub()
                + one.navamsha.as_sub_sub();
            assert_eq!(parts, one.total.as_sub_sub(), "{:?}", one.graha);
            assert_eq!(one.vishwa.as_sub_sub() * 4, one.total.as_sub_sub());
            assert!(one.total <= Bala::new(80, 0, 0));
            assert!(one.vishwa <= Bala::new(20, 0, 0));
        }
        assert_eq!(found.len(), SEVEN.len());
    }

    #[test]
    fn a_longitude_that_is_not_a_number_is_refused_by_its_field() {
        let why = panchavargiya(&AnnualSky {
            venus_deg: f64::NAN,
            ..worked()
        })
        .expect_err("refused");
        assert_eq!(why.field(), Some("venus_deg"));
    }

    /// A strength reads back as the source writes one.
    #[test]
    fn a_strength_is_written_in_units_and_sixtieths() {
        let bala = Bala::new(14, 20, 15);
        assert_eq!(bala.to_string(), "14:20:15");
        assert_eq!(
            (bala.units(), bala.sub_units(), bala.sub_sub()),
            (14, 20, 15)
        );
        assert!((bala.as_f64() - 14.3375).abs() < 1e-12);
        assert_eq!(Bala::new(0, 0, 0).to_string(), "00:00:00");
    }

    /// Every sign has a Hudda and every Hudda a lord, over the whole
    /// circle at a tenth of a degree: nothing falls between two terms.
    #[test]
    fn every_longitude_falls_in_a_hudda() {
        for step in 0..3600 {
            let longitude = f64::from(step) / 10.0;
            let lord = hudda_lord(longitude);
            assert!(lord != Graha::Sun && lord != Graha::Moon, "{longitude}");
        }
    }
}
