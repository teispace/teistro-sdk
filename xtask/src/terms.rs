//! The falsification pass over the terms of the signs (ὅρια, *fines*), the
//! first enumerable doctrine of Phase 7's `hellenistic` module (crux C46).
//!
//! A term is a run of degrees within a sign given to one of the five
//! planets; the luminaries take none. Ptolemy's *Tetrabiblos* (I.XXIII and
//! I.XXIV, Ashmand's 1822 translation, public domain) gives three systems:
//! the Egyptian, printed as a table with its totals stated; the Chaldean,
//! given as a rule with its totals stated and no table; and his own,
//! printed as a table with alternate cells and argued from a rule that
//! leaves ties open. Lilly's *Christian Astrology* (1647, p. 104) prints
//! "the Essentiall Dignities of the Planets according to Ptolomy", which is
//! a second witness to the third.
//!
//! Nothing here is a corpus answer: every number is a source's, so a claim
//! is a source checked against itself (a printed width against its printed
//! end, a table against its stated totals), against a rule the same
//! chapter states, or against a second source. Each table is transcribed
//! off the page cell by cell; what the transcription read and what it
//! could not are in `03-design/terms-measured.md`, which this pass writes.
//!
//! `cargo xtask terms` writes the page; `check-terms` regenerates it in
//! memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::tajika::hudda_lord;
use teistro_core::catalogue::{Graha, Rashi};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, table};

use Graha::{Jupiter as Ju, Mars as Ma, Mercury as Me, Moon, Saturn as Sa, Sun, Venus as Ve};

const PAGE: &str = "docs/03-design/terms-measured.md";

const SIGNS: usize = 12;
const TERMS: usize = 5;
const SIGN_DEGREES: u8 = 30;

/// The five planets that take terms, in the order the Egyptian totals are
/// stated.
const FIVE: [Graha; TERMS] = [Sa, Ju, Ma, Ve, Me];

/// A term as a table prints it: its lord, its width in degrees and the
/// degree it ends at. Both numbers are printed, so each checks the other.
type Printed = (Graha, u8, u8);

/// A sign's five terms as their lords and the degrees they end at.
type Terms = [(Graha, u8); TERMS];

/// *Tetrabiblos* I.XXIII, "The Terms according to the Ægyptians", read off
/// Ashmand's table cell by cell.
#[rustfmt::skip]
const EGYPTIAN: [[Printed; TERMS]; SIGNS] = [
    [(Ju, 6, 6), (Ve, 6, 12), (Me, 8, 20), (Ma, 5, 25), (Sa, 5, 30)],
    [(Ve, 8, 8), (Me, 6, 14), (Ju, 8, 22), (Sa, 5, 27), (Ma, 3, 30)],
    [(Me, 6, 6), (Ju, 6, 12), (Ve, 5, 17), (Ma, 7, 24), (Sa, 6, 30)],
    [(Ma, 7, 7), (Ve, 6, 13), (Me, 6, 19), (Ju, 7, 26), (Sa, 4, 30)],
    [(Ju, 6, 6), (Ve, 5, 11), (Sa, 7, 18), (Me, 6, 24), (Ma, 6, 30)],
    [(Me, 7, 7), (Ve, 10, 17), (Ju, 4, 21), (Ma, 7, 28), (Sa, 2, 30)],
    [(Sa, 6, 6), (Me, 8, 14), (Ju, 7, 21), (Ve, 7, 28), (Ma, 2, 30)],
    [(Ma, 7, 7), (Ve, 4, 11), (Me, 8, 19), (Ju, 5, 24), (Sa, 6, 30)],
    [(Ju, 12, 12), (Ve, 5, 17), (Me, 4, 21), (Sa, 5, 26), (Ma, 4, 30)],
    [(Me, 7, 7), (Ju, 7, 14), (Ve, 8, 22), (Sa, 4, 26), (Ma, 4, 30)],
    [(Me, 7, 7), (Ve, 6, 13), (Ju, 7, 20), (Ma, 5, 25), (Sa, 5, 30)],
    [(Ve, 12, 12), (Ju, 4, 16), (Me, 3, 19), (Ma, 9, 28), (Sa, 2, 30)],
];

/// The Egyptian totals I.XXIII states after its table: "Saturn 57, Jupiter
/// 79, Mars 66, Venus 82, and Mercury 76".
const EGYPTIAN_TOTALS: [(Graha, u16); TERMS] = [(Sa, 57), (Ju, 79), (Ma, 66), (Ve, 82), (Me, 76)];

/// The Chaldean totals I.XXIII states, Saturn's and Mercury's by day and by
/// night: "Saturn ... by day to 78, and by night to 66; ... Jupiter to 72,
/// ... Mars to 69, ... Venus to 75, and ... Mercury by day to 66, and by
/// night to 78".
const CHALDEAN_TOTALS: [(Graha, u16, u16); TERMS] = [
    (Sa, 78, 66),
    (Ju, 72, 72),
    (Ma, 69, 69),
    (Ve, 75, 75),
    (Me, 66, 78),
];

/// The two Chaldean orders I.XXIII spells out — the fire triplicity's and
/// the earth's, by day — which the rule below must reproduce; it says only
/// that "in the other two triplicities a similar order of succession is
/// closely followed".
const CHALDEAN_STATED: [(usize, [Graha; TERMS]); 2] =
    [(0, [Ju, Ve, Sa, Me, Ma]), (1, [Ve, Sa, Me, Ma, Ju])];

/// *Tetrabiblos* I.XXIV, "The Terms according to Ptolemy", the first line
/// of each cell of Ashmand's table.
#[rustfmt::skip]
const PTOLEMY: [[Printed; TERMS]; SIGNS] = [
    [(Ju, 6, 6), (Ve, 8, 14), (Me, 7, 21), (Ma, 5, 26), (Sa, 4, 30)],
    [(Ve, 8, 8), (Me, 7, 15), (Ju, 7, 22), (Sa, 2, 24), (Ma, 6, 30)],
    [(Me, 7, 7), (Ju, 6, 13), (Ve, 7, 20), (Ma, 6, 26), (Sa, 4, 30)],
    [(Ma, 6, 6), (Me, 7, 13), (Ju, 7, 20), (Ve, 7, 27), (Sa, 3, 30)],
    [(Ju, 6, 6), (Me, 7, 13), (Sa, 6, 19), (Ju, 6, 25), (Ma, 5, 30)],
    [(Me, 7, 7), (Ve, 6, 13), (Ju, 5, 18), (Sa, 6, 24), (Ma, 6, 30)],
    [(Sa, 6, 6), (Ve, 5, 11), (Me, 8, 19), (Ju, 5, 24), (Ma, 6, 30)],
    [(Ma, 6, 6), (Ve, 8, 14), (Ju, 7, 21), (Me, 6, 27), (Sa, 3, 30)],
    [(Ju, 8, 8), (Ve, 6, 14), (Me, 5, 19), (Sa, 6, 25), (Ma, 5, 30)],
    [(Ve, 6, 6), (Me, 6, 12), (Ju, 7, 19), (Sa, 6, 25), (Ma, 5, 30)],
    [(Sa, 6, 6), (Me, 6, 12), (Ve, 8, 20), (Ju, 5, 25), (Ma, 5, 30)],
    [(Ve, 8, 8), (Ju, 6, 14), (Me, 6, 20), (Ma, 6, 26), (Sa, 4, 30)],
];

/// A second line Ashmand prints under a cell of I.XXIV's table: another
/// lord, another width, another end, or some of the three.
struct Alternate {
    sign: usize,
    cell: usize,
    lord: Option<Graha>,
    width: Option<u8>,
    end: Option<u8>,
}

impl Alternate {
    const fn new(sign: usize, cell: usize) -> Alternate {
        Alternate {
            sign,
            cell,
            lord: None,
            width: None,
            end: None,
        }
    }

    const fn lord(mut self, lord: Graha) -> Alternate {
        self.lord = Some(lord);
        self
    }

    const fn width(mut self, width: u8) -> Alternate {
        self.width = Some(width);
        self
    }

    const fn end(mut self, end: u8) -> Alternate {
        self.end = Some(end);
        self
    }
}

/// Every second line of I.XXIV's table, all thirteen.
const ALTERNATES: [Alternate; 13] = [
    Alternate::new(1, 3).width(4).end(26),
    Alternate::new(1, 4).width(4),
    Alternate::new(3, 1).lord(Ju),
    Alternate::new(3, 2).lord(Me),
    Alternate::new(4, 0).lord(Sa),
    Alternate::new(4, 2).lord(Ve),
    Alternate::new(6, 2).lord(Ju).width(5).end(16),
    Alternate::new(6, 3).lord(Me).width(8),
    Alternate::new(7, 1).lord(Ju).width(7).end(13),
    Alternate::new(7, 2).lord(Ve).width(8),
    Alternate::new(9, 3).lord(Ma).width(5).end(25),
    Alternate::new(9, 4).lord(Sa),
    Alternate::new(11, 4).width(3),
];

/// Lilly, *Christian Astrology* (1647), p. 104, the terms column: each cell
/// a lord and the degree its term ends at (Wellcome Collection scan,
/// archive.org `b30338724`, leaf n137).
#[rustfmt::skip]
const LILLY: [Terms; SIGNS] = [
    [(Ju, 6), (Ve, 14), (Me, 21), (Ma, 26), (Sa, 30)],
    [(Ve, 8), (Me, 15), (Ju, 22), (Sa, 26), (Ma, 30)],
    [(Me, 7), (Ju, 14), (Ve, 21), (Sa, 25), (Ma, 30)],
    [(Ma, 6), (Ju, 13), (Me, 20), (Ve, 27), (Sa, 30)],
    [(Sa, 6), (Me, 13), (Ve, 19), (Ju, 25), (Ma, 30)],
    [(Me, 7), (Ve, 13), (Ju, 18), (Sa, 24), (Ma, 30)],
    [(Sa, 6), (Ve, 11), (Ju, 19), (Me, 24), (Ma, 30)],
    [(Ma, 6), (Ju, 14), (Ve, 21), (Me, 27), (Sa, 30)],
    [(Ju, 8), (Ve, 14), (Me, 19), (Sa, 25), (Ma, 30)],
    [(Ve, 6), (Me, 12), (Ju, 19), (Ma, 25), (Sa, 30)],
    [(Sa, 6), (Me, 12), (Ve, 20), (Ju, 25), (Ma, 30)],
    [(Ve, 8), (Ju, 14), (Me, 20), (Ma, 26), (Sa, 30)],
];

/// Lilly's exaltations, p. 104, with their degrees; I.XXII gives the signs
/// alone.
const LILLY_EXALTATIONS: [(Graha, usize, u8); 7] = [
    (Sun, 0, 19),
    (Moon, 1, 3),
    (Ju, 3, 15),
    (Me, 5, 15),
    (Sa, 6, 21),
    (Ma, 9, 28),
    (Ve, 11, 27),
];

/// Lilly's faces, p. 104: each sign's three ten-degree faces by lord.
#[rustfmt::skip]
const LILLY_FACES: [[Graha; 3]; SIGNS] = [
    [Ma, Sun, Ve], [Me, Moon, Sa], [Ju, Ma, Sun], [Ve, Me, Moon],
    [Sa, Ju, Ma], [Sun, Ve, Me], [Moon, Sa, Ju], [Ma, Sun, Ve],
    [Me, Moon, Sa], [Ju, Ma, Sun], [Ve, Me, Moon], [Sa, Ju, Ma],
];

/// Lilly's detriments, p. 104, a sign to a planet.
const LILLY_DETRIMENTS: [Graha; SIGNS] = [Ve, Ma, Ju, Sa, Sa, Ju, Ma, Ve, Me, Moon, Sun, Me];

/// Lilly's falls, p. 104: the planet each sign prints, or none. Gemini and
/// Sagittarius print one of the Moon's nodes, whose glyph this scan does
/// not settle, so they are read as no planet.
const LILLY_FALLS: [Option<Graha>; SIGNS] = [
    Some(Sa),
    None,
    None,
    Some(Ma),
    None,
    Some(Ve),
    Some(Sun),
    Some(Moon),
    None,
    Some(Ju),
    None,
    Some(Me),
];

/// The Chaldean order of the seven, slowest first, which the faces follow.
const CHALDEAN_ORDER: [Graha; 7] = [Sa, Ju, Ma, Sun, Ve, Me, Moon];

/// The house of each sign, I.XX: the Moon's Cancer and the Sun's Leo, then
/// each planet's two on either side of them.
const HOUSES: [Graha; SIGNS] = [Ma, Ve, Me, Moon, Sun, Me, Ve, Ma, Ju, Sa, Sa, Ju];

/// The exaltations, I.XXII, as each planet's sign.
const EXALTATIONS: [(Graha, usize); 7] = [
    (Sun, 0),
    (Moon, 1),
    (Sa, 6),
    (Ju, 3),
    (Ma, 9),
    (Ve, 11),
    (Me, 5),
];

/// The triplicity lords, I.XXI, by day, by night and any third: fire the
/// Sun and Jupiter; earth Venus and the Moon; air Saturn and Mercury; water
/// Venus and the Moon "together with Mars". A sign's triplicity is its
/// index modulo four, Aries the first.
const TRIPLICITIES: [[Option<Graha>; 3]; 4] = [
    [Some(Sun), Some(Ju), None],
    [Some(Ve), Some(Moon), None],
    [Some(Sa), Some(Me), None],
    [Some(Ve), Some(Moon), Some(Ma)],
];

/// Lilly's triplicity lords by day and by night, p. 104.
const LILLY_TRIPLICITIES: [(Graha, Graha); 4] = [(Sun, Ju), (Ve, Moon), (Sa, Me), (Ma, Ma)];

/// The kinds of right a planet holds in a sign, in the order I.XXIV says
/// their lords take the first degrees: "the lord of the exaltation is placed
/// first; then the lord of the triplicity; and then the lord of the house".
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Right {
    Exaltation,
    Triplicity,
    House,
}

/// The rights each of the five holds in a sign, its first and how many.
fn rights(sign: usize, planet: Graha) -> (Option<Right>, usize) {
    let held = [
        (Right::Exaltation, EXALTATIONS.contains(&(planet, sign))),
        (
            Right::Triplicity,
            TRIPLICITIES[sign % 4].contains(&Some(planet)),
        ),
        (Right::House, HOUSES[sign] == planet),
    ];
    let first = held
        .iter()
        .find(|(_, holds)| *holds)
        .map(|(right, _)| *right);
    (first, held.iter().filter(|(_, holds)| *holds).count())
}

/// The one planet with two rights in a sign, if there is one.
fn double(sign: usize) -> Option<Graha> {
    FIVE.into_iter().find(|planet| rights(sign, *planet).1 >= 2)
}

/// I.XXIV's exception: "Mars ... receives the first degrees in Cancer, and
/// Saturn in Leo".
fn excepted(sign: usize) -> Option<Graha> {
    match sign {
        3 => Some(Ma),
        4 => Some(Sa),
        _ => None,
    }
}

const fn malefic(planet: Graha) -> bool {
    matches!(planet, Graha::Mars | Graha::Saturn)
}

/// I.XXIV's quantities "when there is no planet found to be lord by two
/// rights": Jupiter and Venus seven degrees, Saturn and Mars five, Mercury
/// six.
const fn base(planet: Graha) -> u8 {
    match planet {
        Graha::Jupiter | Graha::Venus => 7,
        Graha::Mercury => 6,
        _ => 5,
    }
}

fn of_printed(printed: &[Printed; TERMS]) -> Terms {
    printed.map(|(lord, _, end)| (lord, end))
}

/// The lord of the degree that starts at `degree` within a sign.
fn lord_at(terms: &Terms, degree: u8) -> Option<Graha> {
    terms
        .iter()
        .find(|(_, end)| degree < *end)
        .map(|(lord, _)| *lord)
}

fn widths(terms: &Terms) -> [u8; TERMS] {
    let mut start = 0;
    terms.map(|(_, end)| {
        let width = end.saturating_sub(start);
        start = end;
        width
    })
}

/// The Chaldean terms of a sign (I.XXIII): the cycle Jupiter, Venus, the
/// pair Saturn and Mercury, Mars, started at the triplicity's own lord —
/// fire Jupiter, earth Venus, air the pair, water Mars, which is to say at
/// the triplicity's index — with Saturn first of the pair by day and
/// Mercury by night, and widths 8, 7, 6, 5 and 4.
fn chaldean(sign: usize, by_day: bool) -> Terms {
    let pair = if by_day { [Sa, Me] } else { [Me, Sa] };
    let cycle = [[Ju, Ju], [Ve, Ve], pair, [Ma, Ma]];
    let mut lords = Vec::with_capacity(TERMS);
    for step in 0..4 {
        let slot = cycle[(sign % 4 + step) % 4];
        lords.push(slot[0]);
        if slot[0] != slot[1] {
            lords.push(slot[1]);
        }
    }
    let mut end = 0;
    let mut terms = [(Ju, 0); TERMS];
    for ((term, lord), width) in terms.iter_mut().zip(lords).zip([8, 7, 6, 5, 4]) {
        end += width;
        *term = (lord, end);
    }
    terms
}

/// A planet's degrees summed over the twelve signs.
fn total(table: &[Terms; SIGNS], planet: Graha) -> u16 {
    table
        .iter()
        .flat_map(|terms| terms.iter().zip(widths(terms)))
        .filter(|((lord, _), _)| *lord == planet)
        .map(|(_, width)| u16::from(width))
        .sum()
}

/// One reading of Ptolemy's terms, by whoever printed it.
struct Witness {
    name: &'static str,
    terms: [Terms; SIGNS],
}

/// Ashmand's table with its alternates applied: their lords only, onto the
/// first line's ends, or the lords and widths whole, the ends then summed.
fn alternated(whole: bool) -> [Terms; SIGNS] {
    let mut table = PTOLEMY.map(|sign| sign.map(|(lord, width, _)| (lord, width)));
    for alternate in &ALTERNATES {
        let cell = &mut table[alternate.sign][alternate.cell];
        if let Some(lord) = alternate.lord {
            cell.0 = lord;
        }
        if let (true, Some(width)) = (whole, alternate.width) {
            cell.1 = width;
        }
    }
    let mut out = [[(Ju, 0); TERMS]; SIGNS];
    for (sign, cells) in table.iter().enumerate() {
        let mut end = 0;
        for (cell, (lord, width)) in cells.iter().enumerate() {
            end += width;
            out[sign][cell] = (*lord, if whole { end } else { PTOLEMY[sign][cell].2 });
        }
    }
    out
}

fn witnesses() -> [Witness; 4] {
    [
        Witness {
            name: "Ashmand, first lines",
            terms: PTOLEMY.map(|sign| of_printed(&sign)),
        },
        Witness {
            name: "Ashmand, alternate lords",
            terms: alternated(false),
        },
        Witness {
            name: "Ashmand, alternates whole",
            terms: alternated(true),
        },
        Witness {
            name: "Lilly",
            terms: LILLY,
        },
    ]
}

/// A clause judged over a whole table: how many of its comparisons
/// disagree, of how many, and in which signs.
struct Tally {
    wrong: usize,
    of: usize,
    signs: Vec<usize>,
}

impl Tally {
    /// Sums a judgement of one sign — its disagreements and its
    /// comparisons — over the twelve.
    fn over(table: &[Terms; SIGNS], judge: fn(usize, &Terms) -> (usize, usize)) -> Tally {
        let mut tally = Tally {
            wrong: 0,
            of: 0,
            signs: Vec::new(),
        };
        for (sign, terms) in table.iter().enumerate() {
            let (wrong, of) = judge(sign, terms);
            tally.wrong += wrong;
            tally.of += of;
            if wrong > 0 {
                tally.signs.push(sign);
            }
        }
        tally
    }

    /// The tally as a table cell, naming the signs that disagree.
    fn cell(&self) -> String {
        if self.of == 0 {
            String::from("untested")
        } else if self.wrong == 0 {
            format!("**holds** ({})", self.of)
        } else {
            let signs: Vec<_> = self.signs.iter().map(|sign| sign_name(*sign)).collect();
            format!(
                "{} of {} disagree: {}",
                self.wrong,
                self.of,
                signs.join(", ")
            )
        }
    }
}

/// One of I.XXIV's clauses, judged a sign at a time over every witness.
struct Clause {
    rule: &'static str,
    judge: fn(usize, &Terms) -> (usize, usize),
}

/// One comparison, and whether it disagrees.
fn one(wrong: bool) -> (usize, usize) {
    (usize::from(wrong), 1)
}

fn position(terms: &Terms, planet: Graha) -> Option<usize> {
    terms.iter().position(|(lord, _)| *lord == planet)
}

/// The sign's five lords are the five planets once each, and its last term
/// ends at thirty degrees.
fn well_formed(_: usize, terms: &Terms) -> (usize, usize) {
    one(terms[TERMS - 1].1 != SIGN_DEGREES
        || !FIVE
            .iter()
            .all(|planet| terms.iter().filter(|(lord, _)| lord == planet).count() == 1))
}

fn double_first(sign: usize, terms: &Terms) -> (usize, usize) {
    double(sign).map_or((0, 0), |planet| one(terms[0].0 != planet))
}

fn exception_first(sign: usize, terms: &Terms) -> (usize, usize) {
    excepted(sign).map_or((0, 0), |planet| one(terms[0].0 != planet))
}

/// Every malefic without two rights, and not the exception's, in one of the
/// last two terms.
fn malefics_last(sign: usize, terms: &Terms) -> (usize, usize) {
    [Ma, Sa]
        .into_iter()
        .filter(|planet| rights(sign, *planet).1 < 2 && excepted(sign) != Some(*planet))
        .map(|planet| one(position(terms, planet).is_none_or(|at| at < TERMS - 2)))
        .fold((0, 0), |(wrong, of), (w, o)| (wrong + w, of + o))
}

/// Every ordered pair of `planets` that `before` says must stand in that
/// order, and how many of them do not.
fn ordered(
    terms: &Terms,
    planets: &[Graha],
    before: impl Fn(Graha, Graha) -> bool,
) -> (usize, usize) {
    let (mut wrong, mut of) = (0, 0);
    for first in planets {
        for second in planets.iter().filter(|second| before(*first, **second)) {
            of += 1;
            if position(terms, *first) > position(terms, *second) {
                wrong += 1;
            }
        }
    }
    (wrong, of)
}

/// Among the planets the clauses above do not place — not the double, not
/// the exception's malefic, not a malefic sent last — every pair holding
/// rights of different kinds in the order exaltation, triplicity, house.
fn rights_in_order(sign: usize, terms: &Terms) -> (usize, usize) {
    let free: Vec<_> = FIVE
        .into_iter()
        .filter(|planet| {
            double(sign) != Some(*planet) && excepted(sign) != Some(*planet) && !malefic(*planet)
        })
        .collect();
    ordered(terms, &free, |first, second| {
        match (rights(sign, first).0, rights(sign, second).0) {
            (Some(right), Some(later)) => right < later,
            _ => false,
        }
    })
}

/// A benefic or Mercury holding a right before one holding none.
fn holders_before_others(sign: usize, terms: &Terms) -> (usize, usize) {
    let benign: Vec<_> = FIVE
        .into_iter()
        .filter(|planet| !malefic(*planet))
        .collect();
    ordered(terms, &benign, |first, second| {
        rights(sign, first).1 > 0 && rights(sign, second).1 == 0
    })
}

/// The double right's extra degree: its holder's width above its base.
fn double_widened(sign: usize, terms: &Terms) -> (usize, usize) {
    double(sign).map_or((0, 0), |planet| {
        one(position(terms, planet).is_none_or(|at| widths(terms)[at] <= base(planet)))
    })
}

fn within_one(_: usize, terms: &Terms) -> (usize, usize) {
    let wrong = terms
        .iter()
        .zip(widths(terms))
        .filter(|((lord, _), width)| width.abs_diff(base(*lord)) > 1)
        .count();
    (wrong, TERMS)
}

fn lilly_agrees(sign: usize, terms: &Terms) -> (usize, usize) {
    let wrong = terms
        .iter()
        .zip(LILLY[sign])
        .filter(|(term, lilly)| **term != *lilly)
        .count();
    (wrong, TERMS)
}

const CLAUSES: [Clause; 9] = [
    Clause {
        rule: "each sign's five lords are the five planets once each, the last ending at 30°",
        judge: well_formed,
    },
    Clause {
        rule: "\"whatever planet ... may possess two rights of dominion in one and the same sign ... is universally placed first\"",
        judge: double_first,
    },
    Clause {
        rule: "\"Mars ... receives the first degrees in Cancer, and Saturn in Leo\"",
        judge: exception_first,
    },
    Clause {
        rule: "a malefic without two rights \"is always placed last\" — read as one of the last two terms",
        judge: malefics_last,
    },
    Clause {
        rule: "the other lords in the order exaltation, triplicity, house — each pair holding rights of different kinds",
        judge: rights_in_order,
    },
    Clause {
        rule: "a benefic or Mercury holding a right before one holding none — not a clause the chapter states; measured because its order implies it",
        judge: holders_before_others,
    },
    Clause {
        rule: "the planet with two rights \"receives in addition one degree\" over its base (Jupiter and Venus 7, Mercury 6, Saturn and Mars 5)",
        judge: double_widened,
    },
    Clause {
        rule: "every width within one degree of its planet's base",
        judge: within_one,
    },
    Clause {
        rule: "term by term, Lilly's lord and end",
        judge: lilly_agrees,
    },
];

fn sign_name(sign: usize) -> String {
    format!("{:?}", Rashi::ALL[sign % SIGNS])
}

/// The printed cells whose width does not reach their printed end, by
/// table: a check the printing makes on itself.
fn misprinted() -> Vec<String> {
    let mut out = Vec::new();
    for (name, table) in [("Egyptian", &EGYPTIAN), ("Ptolemaic", &PTOLEMY)] {
        for (sign, cells) in table.iter().enumerate() {
            let mut start = 0;
            for (cell, (lord, width, end)) in cells.iter().enumerate() {
                if start + width != *end {
                    out.push(format!(
                        "{name} {} term {} ({lord:?} {width} to {end})",
                        sign_name(sign),
                        cell + 1
                    ));
                }
                start = *end;
            }
        }
    }
    for alternate in &ALTERNATES {
        if let (Some(width), Some(end)) = (alternate.width, alternate.end) {
            let start = alternate
                .cell
                .checked_sub(1)
                .map_or(0, |before| PTOLEMY[alternate.sign][before].2);
            if start + width != end {
                out.push(format!(
                    "Ptolemaic {} term {}'s alternate ({} to {end})",
                    sign_name(alternate.sign),
                    alternate.cell + 1,
                    width
                ));
            }
        }
    }
    out
}

/// The degrees whose Hudda lord (Charak, crux C109) is not their Egyptian
/// lord, by sign.
fn hudda_differences() -> Vec<(usize, usize)> {
    let mut differ = [0; SIGNS];
    for degree in 0..u16::from(SIGN_DEGREES) * 12 {
        let sign = usize::from(degree / u16::from(SIGN_DEGREES));
        let within = u8::try_from(degree % u16::from(SIGN_DEGREES)).unwrap_or(SIGN_DEGREES);
        let lord = lord_at(&of_printed(&EGYPTIAN[sign]), within);
        if lord != Some(hudda_lord(f64::from(degree) + 0.5)) {
            differ[sign] += 1;
        }
    }
    differ
        .into_iter()
        .enumerate()
        .filter(|(_, differ)| *differ > 0)
        .collect()
}

/// What the terms tables say of themselves, of their stated totals and of
/// the rules the same chapters give.
fn term_claims() -> Vec<Claim> {
    let egyptian = EGYPTIAN.map(|sign| of_printed(&sign));
    let misprints = misprinted();
    let egyptian_totals = EGYPTIAN_TOTALS
        .iter()
        .filter(|(planet, stated)| total(&egyptian, *planet) != *stated)
        .count();
    let day = std::array::from_fn::<Terms, SIGNS, _>(|sign| chaldean(sign, true));
    let night = std::array::from_fn::<Terms, SIGNS, _>(|sign| chaldean(sign, false));
    let chaldean_totals = CHALDEAN_TOTALS
        .iter()
        .flat_map(|(planet, by_day, by_night)| {
            [
                total(&day, *planet) != *by_day,
                total(&night, *planet) != *by_night,
            ]
        })
        .filter(|wrong| *wrong)
        .count();
    let chaldean_orders = CHALDEAN_STATED
        .iter()
        .filter(|(sign, lords)| chaldean(*sign, true).map(|(lord, _)| lord) != *lords)
        .count();
    let hudda = hudda_differences();
    let hudda_degrees = hudda.iter().map(|(_, differ)| differ).sum();
    let doubles = (0..SIGNS).filter(|sign| double(*sign).is_some()).count();
    let base_applies = (0..SIGNS)
        .filter(|sign| (0..3).all(|next| double((sign + next) % SIGNS).is_none()))
        .count();
    vec![
        Claim::counted(
            "every printed width reaches its printed end, in both tables and in each alternate that prints both",
            misprints.len(),
            SIGNS * TERMS * 2 + ALTERNATES.iter().filter(|alt| alt.width.is_some() && alt.end.is_some()).count(),
        )
        .with_note(if misprints.is_empty() {
            String::from("none")
        } else {
            misprints.join("; ")
        }),
        Claim::counted(
            "the Egyptian table is the five planets once each in every sign, ending at 30°",
            Tally::over(&egyptian, well_formed).wrong,
            SIGNS,
        ),
        Claim::counted(
            "the Egyptian table's totals are the ones I.XXIII states",
            egyptian_totals,
            EGYPTIAN_TOTALS.len(),
        ),
        Claim::counted(
            "the Chaldean rule — the cycle Jupiter, Venus, Saturn and Mercury, Mars from the triplicity's own lord, widths 8 to 4 — reproduces the two orders I.XXIII spells out",
            chaldean_orders,
            CHALDEAN_STATED.len(),
        ),
        Claim::counted(
            "the same rule, with the pair Saturn first by day and Mercury by night, gives the seven totals I.XXIII states",
            chaldean_totals,
            CHALDEAN_TOTALS.len() + 2,
        ),
        Claim::counted(
            "Charak's Hudda is the Egyptian terms, degree by degree",
            hudda_degrees,
            usize::from(SIGN_DEGREES) * SIGNS,
        )
        .with_note(
            hudda
                .iter()
                .map(|(sign, differ)| format!("{} {differ}", sign_name(*sign)))
                .collect::<Vec<_>>()
                .join(", "),
        ),
        Claim::stated(
            "I.XXIV's base widths apply where \"no planet [is] found to be lord by two rights in the same sign, or in the two signs next following\"",
            if base_applies == 0 {
                Verdict::Untested
            } else {
                Verdict::Holds
            },
            format!(
                "it applies in {} of 12 signs; {} hold a double right",
                count(base_applies),
                count(doubles)
            ),
        ),
    ]
}

/// What the dignities beneath the terms share with the catalogue, and
/// where Lilly parts from Ptolemy.
fn dignity_claims() -> Vec<Claim> {
    let houses = (0..SIGNS)
        .filter(|sign| Rashi::ALL[*sign].attributes().lord != HOUSES[*sign])
        .count();
    let exaltations = EXALTATIONS
        .iter()
        .filter(|(planet, sign)| {
            planet.attributes().exaltation.map(|at| at.sign) != Some(Rashi::ALL[*sign])
        })
        .count();
    let lilly_degrees: Vec<_> = LILLY_EXALTATIONS
        .iter()
        .filter(|(planet, sign, degree)| {
            planet
                .attributes()
                .exaltation
                .map(|at| (at.sign, at.degree))
                != Some((Rashi::ALL[*sign], *degree))
        })
        .map(|(planet, _, degree)| {
            let ours = planet
                .attributes()
                .exaltation
                .map_or(String::from("none"), |at| at.degree.to_string());
            format!("{planet:?} {degree}° against {ours}°")
        })
        .collect();
    let lilly_triplicities = LILLY_TRIPLICITIES
        .iter()
        .zip(TRIPLICITIES)
        .filter(|((day, night), ptolemy)| [Some(*day), Some(*night)] != ptolemy[..2])
        .count();
    let faces = LILLY_FACES
        .iter()
        .flatten()
        .enumerate()
        .filter(|(decan, lord)| CHALDEAN_ORDER[(decan + 2) % CHALDEAN_ORDER.len()] != **lord)
        .count();
    let detriments = (0..SIGNS)
        .filter(|sign| HOUSES[(sign + SIGNS / 2) % SIGNS] != LILLY_DETRIMENTS[*sign])
        .count();
    let falls = (0..SIGNS)
        .filter(|sign| {
            let exalted = LILLY_EXALTATIONS
                .iter()
                .find(|(_, at, _)| *at == (sign + SIGNS / 2) % SIGNS)
                .map(|(planet, _, _)| *planet);
            exalted != LILLY_FALLS[*sign]
        })
        .count();
    vec![
        Claim::counted(
            "Ptolemy's houses (I.XX) are the catalogue's sign lords",
            houses,
            SIGNS,
        ),
        Claim::counted(
            "Ptolemy's exaltations (I.XXII) are the catalogue's exaltation signs",
            exaltations,
            EXALTATIONS.len(),
        ),
        Claim::counted(
            "Lilly's exaltation degrees are the catalogue's",
            lilly_degrees.len(),
            LILLY_EXALTATIONS.len(),
        )
        .with_note(lilly_degrees.join(", ")),
        Claim::counted(
            "Lilly's triplicity lords by day and night are Ptolemy's",
            lilly_triplicities,
            TRIPLICITIES.len(),
        ),
        Claim::counted(
            "Lilly's faces are ten-degree decans in the Chaldean order of the seven (Saturn, Jupiter, Mars, the Sun, Venus, Mercury, the Moon), from Mars in Aries",
            faces,
            SIGNS * 3,
        ),
        Claim::counted(
            "Lilly's detriment of a sign is the lord of the sign opposite",
            detriments,
            SIGNS,
        ),
        Claim::counted(
            "Lilly's fall of a sign is the planet exalted in the sign opposite, by his own exaltations",
            falls,
            SIGNS,
        ),
    ]
}

/// I.XXIV's clauses over every witness, one column each.
fn clause_table(witnesses: &[Witness]) -> String {
    let mut out = String::from("| clause |");
    for witness in witnesses {
        let _ = write!(out, " {} |", witness.name);
    }
    out.push_str("\n|---|");
    for _ in witnesses {
        out.push_str("---|");
    }
    out.push('\n');
    for clause in &CLAUSES {
        let _ = write!(out, "| {} |", clause.rule);
        for witness in witnesses {
            let _ = write!(
                out,
                " {} |",
                Tally::over(&witness.terms, clause.judge).cell()
            );
        }
        out.push('\n');
    }
    out
}

/// A system of terms as a table, a sign to a row, each term its lord and
/// the degree it ends at.
fn terms_table(table: &[Terms; SIGNS]) -> String {
    let mut out = String::from("| sign | 1 | 2 | 3 | 4 | 5 |\n|---|---|---|---|---|---|\n");
    for (sign, terms) in table.iter().enumerate() {
        let _ = write!(out, "| {} |", sign_name(sign));
        for (lord, end) in terms {
            let _ = write!(out, " {lord:?} {end} |");
        }
        out.push('\n');
    }
    out
}

fn page() -> String {
    let witnesses = witnesses();
    let mut out = String::new();
    let _ = write!(
        out,
        "# The terms of the signs, measured\n\n\
         Status: `generated` by `cargo xtask terms` from the *Tetrabiblos* (I.XX to I.XXIV, \
         Ashmand's 1822 translation) and Lilly's *Christian Astrology* (1647, p. 104), \
         2026-10-01. Do not edit: `check-terms` regenerates this page and fails on any \
         difference.\n\n\
         A term is a run of degrees within a sign given to one of the five planets; the \
         Sun and the Moon take none. Ptolemy gives three systems and says which he rejects. \
         Every number on this page is a source's, transcribed off the page cell by cell, so \
         a claim is a source held to itself, to a rule the same chapter states, or to a \
         second source: there is no corpus answer for the doctrine (crux C46).\n\n\
         ## What the sources decide\n\n{claims}\n\
         ## Ptolemy's own terms, clause by clause\n\n\
         I.XXIV argues its table from a rule and prints alternates under thirteen of its \
         sixty cells. Four readings are measured: the first line of every cell; the \
         alternates' lords onto the first lines' ends; the alternates whole, lords and \
         widths, with the ends summed again; and Lilly's table, which calls itself \
         Ptolemy's. A clause holds for a reading when no sign contradicts it.\n\n{clauses}\n",
        claims = table(
            &term_claims()
                .into_iter()
                .chain(dignity_claims())
                .collect::<Vec<_>>()
        ),
        clauses = clause_table(&witnesses),
    );
    out.push_str(
        "## What it means for the module\n\n\
         **The Egyptian terms are a table, and the one the text certifies**: it is \
         well-formed and its totals are the ones Ptolemy states, so `hellenistic` ships it as \
         printed. Charak's Hudda is not a second witness to it but a variant of it, its \
         differing degrees confined to the three signs crux C109 names; the Tajika crate keeps \
         its own row and the two are never read for each other.\n\n\
         **The Chaldean terms are a rule**, and the rule reproduces both orders the chapter \
         spells out and all seven totals it states, by day and by night. So it ships as the \
         rule, with the sect of the chart choosing which of Saturn and Mercury leads the air \
         triplicity's pair, and the table above is its output rather than a transcription.\n\n\
         **Ptolemy's own terms are not determined by his rule**, and no printing of them is \
         simply the text (crux C208). The clauses that hold in every reading fix who is first \
         and who is last and that the double right gains a degree; they leave the middle order \
         and most widths open, and the chapter itself says the degree is taken \"most \
         generally\" from Saturn and Jupiter. Ashmand's first lines are not a table at all \
         where the row above says they break; his alternate lords make every sign well-formed, \
         and Lilly's table is that reading but for the cells the last row names. So the \
         Ptolemaic terms ship as cited tables, Lilly's and Ashmand's alternate-lords reading, \
         each a named row a consumer chooses, and neither is derived.\n\n\
         **The dignities beneath the terms are the catalogue's where the signs are concerned \
         and not where the degrees are.** Ptolemy's houses and exaltation signs are the \
         catalogue's sign lords and exaltation signs, so a sign-level dignity reads them from \
         there. Lilly's exaltation degrees are not the catalogue's, which are the Vedic deep \
         exaltations, so a Western score that weighs a degree reads its own table. Lilly's \
         triplicity lords part from Ptolemy's only in the water signs, where Ptolemy gives Venus \
         and the Moon with Mars and Lilly gives Mars alone; that is a named choice of \
         triplicity scheme, not a correction.\n\n\
         **Lilly's faces, detriments and falls are rules**, each reproducing every cell he \
         prints: the faces are the 36 decans in the Chaldean order, the detriment is the lord \
         of the opposite sign, the fall the planet exalted opposite. So they ship as rules. \
         The word \"face\" names two doctrines, and the catalogue must keep them apart: \
         Ptolemy's \"proper face\" (I.XXVI) is no division of a sign but a planet's aspect \
         to the Sun or the Moon matching the distance between their houses, the text giving \
         Venus's case and Ashmand's note 60 the rest.\n\n",
    );
    out.push_str("## The tables\n\n### Egyptian (I.XXIII)\n\n");
    out.push_str(&terms_table(&EGYPTIAN.map(|sign| of_printed(&sign))));
    out.push_str(
        "\n### Chaldean (I.XXIII), derived\n\nBy day; by night the pair Saturn and Mercury \
         change places wherever they stand.\n\n",
    );
    out.push_str(&terms_table(&std::array::from_fn(|sign| {
        chaldean(sign, true)
    })));
    for witness in &witnesses {
        let _ = write!(out, "\n### Ptolemaic: {}\n\n", witness.name);
        out.push_str(&terms_table(&witness.terms));
    }
    fill(&out)
}

pub(crate) fn generate(root: &Path) -> i32 {
    write(root, &[Output::new(PAGE, page())])
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    i32::from(check(root, &[Output::new(PAGE, page())], "cargo xtask terms") != 0)
}
