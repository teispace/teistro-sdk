//! The terms of the signs: a run of degrees within a sign given to one of
//! the five planets (`03-design/essential-dignities.md`).
//!
//! Every table is transcribed off its source's page and held to it in
//! `03-design/terms-measured.md`, which reads these very constants.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;

use crate::Sect;

/// The terms in one sign.
pub const TERMS_PER_SIGN: usize = 5;

/// The five planets that take terms; the Sun and the Moon take none.
pub const TERM_LORDS: [Graha; TERMS_PER_SIGN] = [
    Graha::Saturn,
    Graha::Jupiter,
    Graha::Mars,
    Graha::Venus,
    Graha::Mercury,
];

const SIGN_DEGREES: u8 = 30;

/// One term: its lord, and the degree within the sign it ends at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Term {
    /// The planet the term belongs to.
    pub lord: Graha,
    /// The degree it ends at, exclusive: a term ending at 6 holds 0° up
    /// to, and not including, 6°.
    pub end: u8,
}

const fn term(lord: Graha, end: u8) -> Term {
    Term { lord, end }
}

use Graha::{Jupiter as Ju, Mars as Ma, Mercury as Me, Saturn as Sa, Venus as Ve};

/// A whole system of terms: five a sign, Aries first, each sign holding
/// the five planets once and ending at 30°.
///
/// The shipped systems are constants; a consumer's own is
/// [`TermsTable::new`], which refuses a malformed sign by name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(
    try_from = "[[Term; TERMS_PER_SIGN]; 12]",
    into = "[[Term; TERMS_PER_SIGN]; 12]"
)]
pub struct TermsTable([[Term; TERMS_PER_SIGN]; 12]);

impl TryFrom<[[Term; TERMS_PER_SIGN]; 12]> for TermsTable {
    type Error = Error;

    fn try_from(signs: [[Term; TERMS_PER_SIGN]; 12]) -> Result<TermsTable, Error> {
        TermsTable::new(signs)
    }
}

impl From<TermsTable> for [[Term; TERMS_PER_SIGN]; 12] {
    fn from(table: TermsTable) -> [[Term; TERMS_PER_SIGN]; 12] {
        table.0
    }
}

impl TermsTable {
    /// The Egyptian terms, *Tetrabiblos* I.XXIII (Ashmand's 1822
    /// translation): the table the chapter prints, whose totals it states.
    #[rustfmt::skip]
    pub const EGYPTIAN: TermsTable = TermsTable([
        [term(Ju, 6), term(Ve, 12), term(Me, 20), term(Ma, 25), term(Sa, 30)],
        [term(Ve, 8), term(Me, 14), term(Ju, 22), term(Sa, 27), term(Ma, 30)],
        [term(Me, 6), term(Ju, 12), term(Ve, 17), term(Ma, 24), term(Sa, 30)],
        [term(Ma, 7), term(Ve, 13), term(Me, 19), term(Ju, 26), term(Sa, 30)],
        [term(Ju, 6), term(Ve, 11), term(Sa, 18), term(Me, 24), term(Ma, 30)],
        [term(Me, 7), term(Ve, 17), term(Ju, 21), term(Ma, 28), term(Sa, 30)],
        [term(Sa, 6), term(Me, 14), term(Ju, 21), term(Ve, 28), term(Ma, 30)],
        [term(Ma, 7), term(Ve, 11), term(Me, 19), term(Ju, 24), term(Sa, 30)],
        [term(Ju, 12), term(Ve, 17), term(Me, 21), term(Sa, 26), term(Ma, 30)],
        [term(Me, 7), term(Ju, 14), term(Ve, 22), term(Sa, 26), term(Ma, 30)],
        [term(Me, 7), term(Ve, 13), term(Ju, 20), term(Ma, 25), term(Sa, 30)],
        [term(Ve, 12), term(Ju, 16), term(Me, 19), term(Ma, 28), term(Sa, 30)],
    ]);

    /// Ptolemy's own terms as Lilly prints them, "according to Ptolomy"
    /// (*Christian Astrology*, 1647, p. 104).
    #[rustfmt::skip]
    pub const PTOLEMAIC_LILLY: TermsTable = TermsTable([
        [term(Ju, 6), term(Ve, 14), term(Me, 21), term(Ma, 26), term(Sa, 30)],
        [term(Ve, 8), term(Me, 15), term(Ju, 22), term(Sa, 26), term(Ma, 30)],
        [term(Me, 7), term(Ju, 14), term(Ve, 21), term(Sa, 25), term(Ma, 30)],
        [term(Ma, 6), term(Ju, 13), term(Me, 20), term(Ve, 27), term(Sa, 30)],
        [term(Sa, 6), term(Me, 13), term(Ve, 19), term(Ju, 25), term(Ma, 30)],
        [term(Me, 7), term(Ve, 13), term(Ju, 18), term(Sa, 24), term(Ma, 30)],
        [term(Sa, 6), term(Ve, 11), term(Ju, 19), term(Me, 24), term(Ma, 30)],
        [term(Ma, 6), term(Ju, 14), term(Ve, 21), term(Me, 27), term(Sa, 30)],
        [term(Ju, 8), term(Ve, 14), term(Me, 19), term(Sa, 25), term(Ma, 30)],
        [term(Ve, 6), term(Me, 12), term(Ju, 19), term(Ma, 25), term(Sa, 30)],
        [term(Sa, 6), term(Me, 12), term(Ve, 20), term(Ju, 25), term(Ma, 30)],
        [term(Ve, 8), term(Ju, 14), term(Me, 20), term(Ma, 26), term(Sa, 30)],
    ]);

    /// Ptolemy's own terms as *Tetrabiblos* I.XXIV prints them in
    /// Ashmand's edition, the alternates' lords taken onto the first
    /// lines' ends: the one reading of that table in which every sign
    /// holds the five planets once (crux C208).
    #[rustfmt::skip]
    pub const PTOLEMAIC_ASHMAND: TermsTable = TermsTable([
        [term(Ju, 6), term(Ve, 14), term(Me, 21), term(Ma, 26), term(Sa, 30)],
        [term(Ve, 8), term(Me, 15), term(Ju, 22), term(Sa, 24), term(Ma, 30)],
        [term(Me, 7), term(Ju, 13), term(Ve, 20), term(Ma, 26), term(Sa, 30)],
        [term(Ma, 6), term(Ju, 13), term(Me, 20), term(Ve, 27), term(Sa, 30)],
        [term(Sa, 6), term(Me, 13), term(Ve, 19), term(Ju, 25), term(Ma, 30)],
        [term(Me, 7), term(Ve, 13), term(Ju, 18), term(Sa, 24), term(Ma, 30)],
        [term(Sa, 6), term(Ve, 11), term(Ju, 19), term(Me, 24), term(Ma, 30)],
        [term(Ma, 6), term(Ju, 14), term(Ve, 21), term(Me, 27), term(Sa, 30)],
        [term(Ju, 8), term(Ve, 14), term(Me, 19), term(Sa, 25), term(Ma, 30)],
        [term(Ve, 6), term(Me, 12), term(Ju, 19), term(Ma, 25), term(Sa, 30)],
        [term(Sa, 6), term(Me, 12), term(Ve, 20), term(Ju, 25), term(Ma, 30)],
        [term(Ve, 8), term(Ju, 14), term(Me, 20), term(Ma, 26), term(Sa, 30)],
    ]);

    /// A system of terms of the caller's own, refused unless every sign
    /// holds each of the five planets once, in terms that end in rising
    /// order at 30°.
    ///
    /// # Errors
    ///
    /// An invalid-argument error naming the sign (`signs[3]` for Cancer)
    /// and what is wrong with it.
    ///
    /// ```
    /// use teistro_hellenistic::{Term, TermsTable};
    /// use teistro_core::catalogue::Graha;
    ///
    /// let mut signs = TermsTable::EGYPTIAN.signs();
    /// assert!(TermsTable::new(signs).is_ok());
    /// signs[3][4] = Term { lord: Graha::Jupiter, end: 30 };
    /// let refused = TermsTable::new(signs).unwrap_err();
    /// assert_eq!(refused.field(), Some("signs[3]"));
    /// ```
    pub fn new(signs: [[Term; TERMS_PER_SIGN]; 12]) -> Result<TermsTable, Error> {
        for (index, (sign, terms)) in Rashi::ALL.iter().zip(&signs).enumerate() {
            let refuse = |what: String| {
                Error::invalid_arg(format!("the terms of {sign:?} {what}"))
                    .with_field(format!("signs[{index}]"))
            };
            for lord in TERM_LORDS {
                let held = terms.iter().filter(|term| term.lord == lord).count();
                if held != 1 {
                    return Err(
                        refuse(format!("give {lord:?} {held} terms, not one")).with_hint(
                            "each sign holds Saturn, Jupiter, Mars, Venus and Mercury once",
                        ),
                    );
                }
            }
            let mut start = 0;
            for term in terms {
                if term.end <= start {
                    return Err(refuse(format!(
                        "end at {}° after a term ending at {start}°",
                        term.end
                    )));
                }
                start = term.end;
            }
            if start != SIGN_DEGREES {
                return Err(refuse(format!("end at {start}°, not 30°")));
            }
        }
        Ok(TermsTable(signs))
    }

    /// The Chaldean terms, *Tetrabiblos* I.XXIII's rule: the lords in the
    /// cycle Jupiter, Venus, the pair Saturn and Mercury, Mars, begun at the
    /// triplicity's own lord (fire Jupiter, earth Venus, air the pair, water
    /// Mars), Saturn first of the pair by day and Mercury by night, widths
    /// 8, 7, 6, 5 and 4. The same in all three signs of a triplicity.
    ///
    /// ```
    /// use teistro_hellenistic::{Sect, TermsTable};
    /// use teistro_core::catalogue::Graha;
    ///
    /// // Gemini, an air sign, opens with the pair: Saturn by day.
    /// assert_eq!(TermsTable::chaldean(Sect::Day).lord_at(60.0), Some(Graha::Saturn));
    /// assert_eq!(TermsTable::chaldean(Sect::Night).lord_at(60.0), Some(Graha::Mercury));
    /// ```
    #[must_use]
    pub fn chaldean(sect: Sect) -> TermsTable {
        let pair = match sect {
            Sect::Day => [Sa, Me],
            Sect::Night => [Me, Sa],
        };
        let cycle = [[Ju, Ju], [Ve, Ve], pair, [Ma, Ma]];
        let mut signs = [[term(Ju, 0); TERMS_PER_SIGN]; 12];
        for (index, terms) in signs.iter_mut().enumerate() {
            let lords = cycle
                .iter()
                .cycle()
                .skip(index % cycle.len())
                .take(cycle.len())
                .flat_map(|[first, second]| {
                    std::iter::once(*first).chain((first != second).then_some(*second))
                });
            let mut end = 0;
            for ((cell, lord), width) in terms.iter_mut().zip(lords).zip([8, 7, 6, 5, 4]) {
                end += width;
                *cell = term(lord, end);
            }
        }
        TermsTable(signs)
    }

    /// The twelve signs' terms, Aries first.
    #[must_use]
    pub const fn signs(&self) -> [[Term; TERMS_PER_SIGN]; 12] {
        self.0
    }

    /// The terms of one sign.
    #[must_use]
    pub fn of(&self, sign: Rashi) -> [Term; TERMS_PER_SIGN] {
        let [aries, ..] = self.0;
        self.0
            .iter()
            .zip(Rashi::ALL)
            .find(|(_, each)| *each == sign)
            .map_or(aries, |(terms, _)| *terms)
    }

    /// The lord of the term a longitude falls in, taken in any turn; none
    /// for a longitude that is not a number.
    #[must_use]
    pub fn lord_at(&self, longitude_deg: f64) -> Option<Graha> {
        if !longitude_deg.is_finite() {
            return None;
        }
        let within = longitude_deg.rem_euclid(360.0) % 30.0;
        let terms = self.of(Rashi::of_longitude(longitude_deg));
        terms
            .iter()
            .find(|term| within < f64::from(term.end))
            .or(terms.last())
            .map(|term| term.lord)
    }
}

/// Which system of terms a reading uses.
///
/// No system is a default: a reading names its terms, because the
/// sources disagree and choosing one is choosing a school (crux C208).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
#[expect(
    clippy::large_enum_variant,
    reason = "a table is 240 bytes and a rules value is built once a request; boxing it would cost `Copy` and the `const` rule rows"
)]
pub enum Terms {
    /// [`TermsTable::EGYPTIAN`].
    Egyptian,
    /// [`TermsTable::PTOLEMAIC_LILLY`].
    PtolemaicLilly,
    /// [`TermsTable::PTOLEMAIC_ASHMAND`].
    PtolemaicAshmand,
    /// [`TermsTable::chaldean`], by the chart's sect.
    Chaldean,
    /// A table of the caller's own.
    Table(TermsTable),
}

impl Terms {
    /// The table this system gives a chart of the sect.
    #[must_use]
    pub fn table(&self, sect: Sect) -> TermsTable {
        match self {
            Terms::Egyptian => TermsTable::EGYPTIAN,
            Terms::PtolemaicLilly => TermsTable::PTOLEMAIC_LILLY,
            Terms::PtolemaicAshmand => TermsTable::PTOLEMAIC_ASHMAND,
            Terms::Chaldean => TermsTable::chaldean(sect),
            Terms::Table(table) => *table,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own results"
    )]

    use super::{Graha, Sect, TERM_LORDS, Term, Terms, TermsTable};

    const SHIPPED: [TermsTable; 3] = [
        TermsTable::EGYPTIAN,
        TermsTable::PTOLEMAIC_LILLY,
        TermsTable::PTOLEMAIC_ASHMAND,
    ];

    #[test]
    fn every_shipped_table_passes_its_own_constructor() {
        for table in SHIPPED
            .into_iter()
            .chain([Sect::Day, Sect::Night].map(TermsTable::chaldean))
        {
            assert_eq!(TermsTable::new(table.signs()), Ok(table));
        }
    }

    /// I.XXIII states each system's totals, so the tables are held to the
    /// numbers printed beside them and not only to their own shape.
    #[test]
    fn the_totals_are_the_ones_the_tetrabiblos_states() {
        let total = |table: &TermsTable, lord: Graha| -> u16 {
            table
                .signs()
                .iter()
                .flat_map(|terms| {
                    let mut start = 0;
                    terms.map(|term| {
                        let width = term.end - start;
                        start = term.end;
                        (term.lord, width)
                    })
                })
                .filter(|(each, _)| *each == lord)
                .map(|(_, width)| u16::from(width))
                .sum()
        };
        let egyptian = [57, 79, 66, 82, 76];
        let day = [78, 72, 69, 75, 66];
        let night = [66, 72, 69, 75, 78];
        for (index, lord) in TERM_LORDS.into_iter().enumerate() {
            assert_eq!(
                total(&TermsTable::EGYPTIAN, lord),
                egyptian[index],
                "{lord:?}"
            );
            assert_eq!(
                total(&TermsTable::chaldean(Sect::Day), lord),
                day[index],
                "{lord:?} by day"
            );
            assert_eq!(
                total(&TermsTable::chaldean(Sect::Night), lord),
                night[index],
                "{lord:?} by night"
            );
        }
    }

    /// Lilly, p. 102: "if Jupiter be in one, two, three ... degrees of
    /// Aries, he is then in his owne Termes".
    #[test]
    fn lillys_worked_term_holds_and_ends_where_his_table_says() {
        let lilly = Terms::PtolemaicLilly.table(Sect::Day);
        assert_eq!(lilly.lord_at(0.0), Some(Graha::Jupiter));
        assert_eq!(lilly.lord_at(5.999), Some(Graha::Jupiter));
        assert_eq!(lilly.lord_at(6.0), Some(Graha::Venus));
        assert_eq!(lilly.lord_at(359.999), Some(Graha::Saturn));
        assert_eq!(lilly.lord_at(-0.5), Some(Graha::Saturn));
        assert_eq!(lilly.lord_at(f64::NAN), None);
    }

    #[test]
    fn a_malformed_table_is_refused_by_its_sign_both_ways() {
        let mut twice = TermsTable::EGYPTIAN.signs();
        twice[7][0] = Term {
            lord: Graha::Venus,
            end: 7,
        };
        let refused = TermsTable::new(twice).unwrap_err();
        assert_eq!(refused.field(), Some("signs[7]"));

        let mut short = TermsTable::EGYPTIAN.signs();
        short[11][4].end = 29;
        assert_eq!(
            TermsTable::new(short).unwrap_err().field(),
            Some("signs[11]")
        );

        let mut backwards = TermsTable::EGYPTIAN.signs();
        backwards[0][1].end = 5;
        assert_eq!(
            TermsTable::new(backwards).unwrap_err().field(),
            Some("signs[0]")
        );

        let mut sun = TermsTable::EGYPTIAN.signs();
        sun[2][0].lord = Graha::Sun;
        assert_eq!(TermsTable::new(sun).unwrap_err().field(), Some("signs[2]"));
    }

    #[test]
    fn a_table_reads_and_writes_as_its_signs_and_refuses_on_reading() {
        let json = serde_json::to_string(&Terms::Table(TermsTable::EGYPTIAN)).unwrap();
        let back: Terms = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Terms::Table(TermsTable::EGYPTIAN));
        assert_eq!(
            serde_json::to_string(&Terms::Chaldean).unwrap(),
            "\"CHALDEAN\""
        );
        let broken = json.replacen("\"end\":6", "\"end\":40", 1);
        assert!(serde_json::from_str::<Terms>(&broken).is_err());
    }
}
