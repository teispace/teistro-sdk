//! Panchaka, the muhurta yogas and the disha shool: what a day is said to
//! be, as opposed to what it measures.
//!
//! Each of these is an **interval** and not a flag. The corpus records
//! them as flags, and a flag is what an interval reduces to — "some
//! interval contains sunrise" — so reporting the interval is strictly
//! more information at no cost, since the limb spans it is cut from are
//! already computed.
//!
//! The muhurta yogas are the one limb the corpus cannot settle. Five of
//! them fire thirteen times on twelve of the fifty-five recorded days,
//! which cannot derive a table of seven varas by twenty-seven
//! nakshatras, and the recording engine's positives match no published
//! table under any rotation of either index
//! (`03-design/panchanga-day-conventions.md` §8). Two things the corpus
//! does settle and this module keeps:
//!
//! 1. **A vara-and-nakshatra yoga is read at sunrise.** Reading any
//!    nakshatra of the day into the Amrit Siddhi pairs fires it on four
//!    days the engine leaves clear; reading only the sunrise one agrees
//!    on all fifty-five.
//! 2. **Tripushkar's classical rule holds** on the corpus's one instance
//!    with no false positives.
//!
//! So the tables ship as data with their own confidence marks, chosen by
//! `panchanga.muhurta_tables`, and a yoga that fires says what made it.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{
    Direction, MuhurtaYoga as Kind, Nakshatra, Panchaka, Rashi, Tithi, TithiClass, Vara,
};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::settings::PanchakaStart;

use crate::span::Span;

/// The tables the SDK ships, by the key `panchanga.muhurta_tables` names.
pub const CLASSICAL: &str = "CLASSICAL";

/// What made a muhurta yoga hold.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "cause", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum YogaCause {
    /// The vara and the nakshatra the Moon was in.
    VaraNakshatra {
        /// The day's vara.
        vara: Vara,
        /// The nakshatra that makes it.
        nakshatra: Nakshatra,
    },
    /// The vara, the tithi's class and the nakshatra's number of feet.
    VaraTithiNakshatra {
        /// The day's vara.
        vara: Vara,
        /// The tithi that makes it.
        tithi: Tithi,
        /// The nakshatra that makes it.
        nakshatra: Nakshatra,
    },
}

/// A muhurta yoga that held, and for how long.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PanchangaMuhurtaYoga"))]
pub struct MuhurtaYoga {
    /// Which yoga.
    pub yoga: Kind,
    /// While it held, clipped to the day.
    pub at: Interval,
    /// What made it, so a reader can see why.
    pub because: YogaCause,
}

/// What a day is said to be.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Omens {
    /// Panchaka, while the Moon is in the last five nakshatras.
    pub panchaka: Vec<Span<Panchaka>>,
    /// The muhurta yogas that held.
    pub yogas: Vec<MuhurtaYoga>,
    /// The direction not to travel in, which is the vara's.
    pub disha_shool: Direction,
}

/// The direction a vara's travel is blocked in.
///
/// Monday and Saturday east, Tuesday and Wednesday north, Thursday
/// south, Friday and Sunday west — the corpus's own table, on all 55
/// days.
#[must_use]
pub const fn disha_shool(vara: Vara) -> Direction {
    match vara {
        Vara::Somavara | Vara::Shanivara => Direction::East,
        Vara::Mangalavara | Vara::Budhavara => Direction::North,
        Vara::Guruvara => Direction::South,
        _ => Direction::West,
    }
}

/// The panchaka a nakshatra carries, if it is one of the last five.
#[must_use]
pub fn panchaka_of(nakshatra: Nakshatra) -> Option<Panchaka> {
    Panchaka::ALL
        .iter()
        .copied()
        .find(|panchaka| panchaka.attributes().nakshatra == nakshatra)
}

/// The panchaka spans of a day, from its nakshatra spans.
///
/// The window a panchaka runs in is the nakshatra's own, cut at the
/// Moon's entry into Aquarius where `start` says it begins there (crux
/// C158): only Dhanishtha straddles that entry, so `moon_signs` is asked
/// for the signs over a Dhanishtha span and nothing else. The corpus
/// cannot separate the two readings — they differ only in the first half
/// of Dhanishtha, and no recorded day has the Moon there — so the SDK
/// follows Raman's *Muhurtha* (ch. IV), the one text in hand, and names
/// the recording engine's whole nakshatra as the other.
///
/// # Errors
///
/// Whatever `moon_signs` returns.
pub fn panchaka<F>(
    nakshatras: &[Span<Nakshatra>],
    start: PanchakaStart,
    mut moon_signs: F,
) -> Result<Vec<Span<Panchaka>>, Error>
where
    F: FnMut(Interval) -> Result<Vec<Span<Rashi>>, Error>,
{
    let mut found = Vec::new();
    for span in nakshatras {
        let Some(kind) = panchaka_of(span.member) else {
            continue;
        };
        if span.member != Nakshatra::Dhanishtha || start == PanchakaStart::Dhanishtha {
            found.push(span.map(|_| kind));
            continue;
        }
        for sign in moon_signs(span.inside)? {
            if sign.member != Rashi::Aquarius {
                continue;
            }
            let Some(whole) = span.whole.clipped_to(sign.whole) else {
                continue;
            };
            found.extend(Span::new(kind, whole, span.inside));
        }
    }
    Ok(found)
}

/// The panchaka an instant's four numbers leave, or `None` when the
/// remainder is a good one (crux C159).
///
/// Raman's *Muhurtha* (ch. III, p. 20) sums the tithi's number (from
/// Shukla Pratipada, 1 to 30), the vara's (Sunday 1), the nakshatra's
/// (from Ashwini) and the lagna's (from Aries) and divides by nine: 1
/// leaves mrityu, 2 agni, 4 raja, 6 chora and 8 roga, each member's
/// `remainder`; 3, 5, 7 and 0 are good. It is a clause of an **instant**,
/// since the lagna is one, and not the almanac's panchaka, which is the
/// Moon's run through the last nakshatras and carries the recording
/// engine's kind for each.
#[must_use]
pub fn panchaka_remainder(
    tithi: Tithi,
    vara: Vara,
    nakshatra: Nakshatra,
    lagna: Rashi,
) -> Option<Panchaka> {
    let sum = u32::from(tithi.id())
        + u32::from(vara.attributes().weekday)
        + u32::from(nakshatra.id())
        + u32::from(lagna.id())
        + 4;
    let remainder = sum % 9;
    Panchaka::ALL
        .iter()
        .copied()
        .find(|panchaka| u32::from(panchaka.attributes().remainder) == remainder)
}

/// One row of a muhurta yoga table.
struct Row {
    /// Which yoga the row makes.
    yoga: Kind,
    /// The vara it needs.
    vara: Vara,
    /// The nakshatra it needs.
    nakshatra: Nakshatra,
}

/// The vara-and-nakshatra rows of the classical tables.
///
/// Marked traditional: the corpus cannot derive them and the recording
/// engine's own do not match any published set
/// (`03-design/panchanga-day-conventions.md` §8). They are here so that a
/// caller gets an answer with a stated provenance rather than none, and
/// `panchanga.muhurta_tables` is where another tradition's rows go.
const VARA_NAKSHATRA: [Row; 12] = [
    // Amrit Siddhi: one pair per vara.
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Ravivara,
        nakshatra: Nakshatra::Hasta,
    },
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Somavara,
        nakshatra: Nakshatra::Mrigashira,
    },
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Mangalavara,
        nakshatra: Nakshatra::Ashwini,
    },
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Budhavara,
        nakshatra: Nakshatra::Anuradha,
    },
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Guruvara,
        nakshatra: Nakshatra::Pushya,
    },
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Shukravara,
        nakshatra: Nakshatra::Revati,
    },
    Row {
        yoga: Kind::AmritSiddhi,
        vara: Vara::Shanivara,
        nakshatra: Nakshatra::Rohini,
    },
    // Sarvartha Siddhi: a few nakshatras per vara. Only the rows the
    // published tables agree on are here; the rest await a citation.
    Row {
        yoga: Kind::SarvarthaSiddhi,
        vara: Vara::Ravivara,
        nakshatra: Nakshatra::Pushya,
    },
    Row {
        yoga: Kind::SarvarthaSiddhi,
        vara: Vara::Ravivara,
        nakshatra: Nakshatra::Hasta,
    },
    Row {
        yoga: Kind::SarvarthaSiddhi,
        vara: Vara::Somavara,
        nakshatra: Nakshatra::Shravana,
    },
    Row {
        yoga: Kind::SarvarthaSiddhi,
        vara: Vara::Budhavara,
        nakshatra: Nakshatra::Rohini,
    },
    Row {
        yoga: Kind::SarvarthaSiddhi,
        vara: Vara::Guruvara,
        nakshatra: Nakshatra::Revati,
    },
];

/// The nakshatras of three feet, which a Bhadra tithi on a Sunday,
/// Tuesday or Saturday makes Tripushkar in.
const THREE_FOOTED: [Nakshatra; 6] = [
    Nakshatra::Krittika,
    Nakshatra::Punarvasu,
    Nakshatra::UttaraPhalguni,
    Nakshatra::Vishakha,
    Nakshatra::UttaraAshadha,
    Nakshatra::PurvaBhadrapada,
];

/// The nakshatras of two feet, which the same combination makes
/// Dwipushkar in. No recorded day carries one, so the rule ships
/// untested and says so.
const TWO_FOOTED: [Nakshatra; 4] = [
    Nakshatra::Mrigashira,
    Nakshatra::Chitra,
    Nakshatra::Dhanishtha,
    Nakshatra::Anuradha,
];

/// The varas a pushkara yoga needs.
const PUSHKARA_VARAS: [Vara; 3] = [Vara::Ravivara, Vara::Mangalavara, Vara::Shanivara];

/// The muhurta yogas of a day.
///
/// # Errors
///
/// `UNSUPPORTED` for a table key the SDK does not ship, naming the ones
/// it does.
pub fn yogas(
    tables: &str,
    vara: Vara,
    nakshatras: &[Span<Nakshatra>],
    tithis: &[Span<Tithi>],
) -> Result<Vec<MuhurtaYoga>, Error> {
    if !tables.eq_ignore_ascii_case(CLASSICAL) {
        return Err(Error::unsupported(format!(
            "no muhurta yoga table `{tables}`; the SDK ships `{CLASSICAL}`"
        ))
        .with_field("panchanga.muhurta_tables"));
    }
    // Measured: a vara-and-nakshatra yoga is read at the nakshatra the
    // day opens in, not at any nakshatra of the day.
    let Some(at_sunrise) = nakshatras.first() else {
        return Ok(Vec::new());
    };
    let mut found: Vec<MuhurtaYoga> = VARA_NAKSHATRA
        .iter()
        .filter(|row| row.vara == vara && row.nakshatra == at_sunrise.member)
        .map(|row| MuhurtaYoga {
            yoga: row.yoga,
            at: at_sunrise.inside,
            because: YogaCause::VaraNakshatra {
                vara,
                nakshatra: at_sunrise.member,
            },
        })
        .collect();
    found.extend(pushkara(vara, at_sunrise, tithis));
    Ok(found)
}

/// Dwipushkar and Tripushkar: a Bhadra tithi on a Sunday, Tuesday or
/// Saturday in a two- or three-footed nakshatra.
fn pushkara(vara: Vara, at_sunrise: &Span<Nakshatra>, tithis: &[Span<Tithi>]) -> Vec<MuhurtaYoga> {
    if !PUSHKARA_VARAS.contains(&vara) {
        return Vec::new();
    }
    let Some(tithi) = tithis.first() else {
        return Vec::new();
    };
    if tithi.member.attributes().class != TithiClass::Bhadra {
        return Vec::new();
    }
    let yoga = if THREE_FOOTED.contains(&at_sunrise.member) {
        Kind::Tripushkar
    } else if TWO_FOOTED.contains(&at_sunrise.member) {
        Kind::Dwipushkar
    } else {
        return Vec::new();
    };
    // It holds while both the tithi and the nakshatra do.
    let Some(at) = tithi.inside.clipped_to(at_sunrise.inside) else {
        return Vec::new();
    };
    vec![MuhurtaYoga {
        yoga,
        at,
        because: YogaCause::VaraTithiNakshatra {
            vara,
            tithi: tithi.member,
            nakshatra: at_sunrise.member,
        },
    }]
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use super::{CLASSICAL, disha_shool, panchaka, panchaka_of, panchaka_remainder, yogas};
    use teistro_core::catalogue::{
        Direction, MuhurtaYoga as Kind, Nakshatra, Panchaka, Rashi, Tithi, Vara,
    };
    use teistro_core::error::Error;
    use teistro_core::interval::Interval;
    use teistro_core::settings::PanchakaStart;

    use crate::span::Span;

    fn window() -> Interval {
        Interval::literal(100.0, 101.0)
    }

    fn span<T>(member: T) -> Span<T> {
        Span::new(member, window(), window()).expect("it fills the day")
    }

    #[test]
    fn the_last_five_nakshatras_carry_a_panchaka_and_no_others() {
        assert_eq!(panchaka_of(Nakshatra::Dhanishtha), Some(Panchaka::Mrityu));
        assert_eq!(panchaka_of(Nakshatra::Shatabhisha), Some(Panchaka::Agni));
        assert_eq!(
            panchaka_of(Nakshatra::PurvaBhadrapada),
            Some(Panchaka::Raja)
        );
        assert_eq!(
            panchaka_of(Nakshatra::UttaraBhadrapada),
            Some(Panchaka::Chora)
        );
        assert_eq!(panchaka_of(Nakshatra::Revati), Some(Panchaka::Roga));
        assert_eq!(panchaka_of(Nakshatra::Shravana), None, "the one before");
        assert_eq!(panchaka_of(Nakshatra::Ashwini), None);
        assert_eq!(
            Nakshatra::ALL
                .iter()
                .filter(|n| panchaka_of(**n).is_some())
                .count(),
            5
        );
    }

    /// A Moon that is never asked about: every span it is given is not
    /// Dhanishtha, or the start is the whole nakshatra.
    fn unasked(_: Interval) -> Result<Vec<Span<Rashi>>, Error> {
        Err(Error::internal("the Moon's signs were asked for"))
    }

    #[test]
    fn a_panchaka_span_is_the_nakshatras_own() {
        let spans = [span(Nakshatra::Revati), span(Nakshatra::Ashwini)];
        let found = panchaka(&spans, PanchakaStart::Aquarius, unasked).unwrap();
        assert_eq!(found.len(), 1, "only Revati of the two");
        assert_eq!(found[0].member, Panchaka::Roga);
        assert_eq!(found[0].inside, spans[0].inside);
        assert!(
            panchaka(
                &[span(Nakshatra::Ashwini)],
                PanchakaStart::Aquarius,
                unasked
            )
            .unwrap()
            .is_empty()
        );
    }

    /// Dhanishtha over the whole test day, the Moon entering Aquarius
    /// half way through it.
    fn dhanishtha_across_the_entry() -> (Span<Nakshatra>, Vec<Span<Rashi>>) {
        let entry = 100.5;
        let signs = vec![
            Span::new(Rashi::Capricorn, Interval::literal(98.0, entry), window()).unwrap(),
            Span::new(Rashi::Aquarius, Interval::literal(entry, 102.5), window()).unwrap(),
        ];
        (span(Nakshatra::Dhanishtha), signs)
    }

    #[test]
    fn under_raman_panchaka_begins_where_the_moon_enters_aquarius() {
        let (dhanishtha, signs) = dhanishtha_across_the_entry();
        let found = panchaka(&[dhanishtha], PanchakaStart::Aquarius, |within| {
            assert_eq!(within, dhanishtha.inside, "asked over the nakshatra's day");
            Ok(signs.clone())
        })
        .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].member, Panchaka::Mrityu);
        assert_eq!(found[0].inside, Interval::literal(100.5, 101.0));
        assert_eq!(found[0].whole, Interval::literal(100.5, 101.0));
        assert!(!found[0].began_before(), "it began inside the day");
    }

    #[test]
    fn under_the_recording_engine_panchaka_is_the_whole_of_dhanishtha() {
        let (dhanishtha, _) = dhanishtha_across_the_entry();
        let found = panchaka(&[dhanishtha], PanchakaStart::Dhanishtha, unasked).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].inside, dhanishtha.inside);
    }

    #[test]
    fn a_dhanishtha_wholly_in_capricorn_carries_no_panchaka_under_raman() {
        let (dhanishtha, signs) = dhanishtha_across_the_entry();
        let found = panchaka(&[dhanishtha], PanchakaStart::Aquarius, |_| {
            Ok(signs[..1].to_vec())
        })
        .unwrap();
        assert!(found.is_empty());
    }

    #[test]
    fn the_remainder_reproduces_ramans_worked_example() {
        // Ch. III, p. 20: Aslesha (9), the 13th tithi, Virgo rising (6),
        // a Sunday (1) make 29, which leaves 2: agni, "not favourable".
        assert_eq!(
            panchaka_remainder(
                Tithi::ShuklaTrayodashi,
                Vara::Ravivara,
                Nakshatra::Ashlesha,
                Rashi::Virgo
            ),
            Some(Panchaka::Agni)
        );
    }

    #[test]
    fn every_remainder_names_its_kind_or_none() {
        // Walk the lagna through all twelve signs from a sum of 1+1+1+1 = 4:
        // the remainders run 4, 5, ... 15 by nine.
        let expected = [
            Some(Panchaka::Raja),   // 4
            None,                   // 5
            Some(Panchaka::Chora),  // 6
            None,                   // 7
            Some(Panchaka::Roga),   // 8
            None,                   // 0
            Some(Panchaka::Mrityu), // 1
            Some(Panchaka::Agni),   // 2
            None,                   // 3
            Some(Panchaka::Raja),   // 4
            None,                   // 5
            Some(Panchaka::Chora),  // 6
        ];
        for (lagna, kind) in Rashi::ALL.iter().zip(expected) {
            assert_eq!(
                panchaka_remainder(
                    Tithi::ShuklaPratipada,
                    Vara::Ravivara,
                    Nakshatra::Ashwini,
                    *lagna
                ),
                kind,
                "{lagna:?}"
            );
        }
    }

    #[test]
    fn the_thirtieth_tithi_counts_as_thirty() {
        // Amavasya is 30, not 15 of the dark half: 30 + 1 + 1 + 1 = 33,
        // which leaves 6, chora; counted as 15 it would be 18, which
        // leaves 0 and is good.
        assert_eq!(
            panchaka_remainder(
                Tithi::Amavasya,
                Vara::Ravivara,
                Nakshatra::Ashwini,
                Rashi::Aries
            ),
            Some(Panchaka::Chora)
        );
    }

    #[test]
    fn the_disha_shool_is_the_varas_own() {
        assert_eq!(disha_shool(Vara::Somavara), Direction::East);
        assert_eq!(disha_shool(Vara::Shanivara), Direction::East);
        assert_eq!(disha_shool(Vara::Mangalavara), Direction::North);
        assert_eq!(disha_shool(Vara::Budhavara), Direction::North);
        assert_eq!(disha_shool(Vara::Guruvara), Direction::South);
        assert_eq!(disha_shool(Vara::Shukravara), Direction::West);
        assert_eq!(disha_shool(Vara::Ravivara), Direction::West);
    }

    #[test]
    fn a_yoga_is_read_at_the_nakshatra_the_day_opens_in() {
        // Wednesday and Anuradha make Amrit Siddhi.
        let found = yogas(
            CLASSICAL,
            Vara::Budhavara,
            &[span(Nakshatra::Anuradha), span(Nakshatra::Jyeshtha)],
            &[],
        )
        .expect("a table the SDK ships");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].yoga, Kind::AmritSiddhi);

        // The same nakshatra second in the day makes nothing, which is
        // what the corpus falsified the other reading with.
        let later = yogas(
            CLASSICAL,
            Vara::Budhavara,
            &[span(Nakshatra::Vishakha), span(Nakshatra::Anuradha)],
            &[],
        )
        .expect("a table the SDK ships");
        assert!(later.is_empty());
    }

    #[test]
    fn tripushkar_needs_all_three_of_its_conditions() {
        let nakshatra = [span(Nakshatra::PurvaBhadrapada)];
        let bhadra = [span(Tithi::ShuklaSaptami)];
        let found =
            yogas(CLASSICAL, Vara::Shanivara, &nakshatra, &bhadra).expect("a table the SDK ships");
        assert!(found.iter().any(|yoga| yoga.yoga == Kind::Tripushkar));

        // A vara that is not one of the three.
        assert!(
            !yogas(CLASSICAL, Vara::Somavara, &nakshatra, &bhadra)
                .expect("a table")
                .iter()
                .any(|yoga| yoga.yoga == Kind::Tripushkar)
        );
        // A tithi that is not Bhadra.
        let nanda = [span(Tithi::ShuklaPratipada)];
        assert!(
            !yogas(CLASSICAL, Vara::Shanivara, &nakshatra, &nanda)
                .expect("a table")
                .iter()
                .any(|yoga| yoga.yoga == Kind::Tripushkar)
        );
        // A nakshatra of neither two nor three feet.
        assert!(
            !yogas(
                CLASSICAL,
                Vara::Shanivara,
                &[span(Nakshatra::Magha)],
                &bhadra
            )
            .expect("a table")
            .iter()
            .any(|yoga| yoga.yoga == Kind::Tripushkar)
        );
        // Two feet make the other one.
        assert!(
            yogas(
                CLASSICAL,
                Vara::Shanivara,
                &[span(Nakshatra::Chitra)],
                &bhadra
            )
            .expect("a table")
            .iter()
            .any(|yoga| yoga.yoga == Kind::Dwipushkar)
        );
    }

    #[test]
    fn a_table_the_sdk_does_not_ship_is_refused_by_name() {
        let error = yogas("SOMEONE_ELSES", Vara::Somavara, &[], &[]).expect_err("no such table");
        assert!(error.message.contains("SOMEONE_ELSES"), "{error}");
        assert!(error.message.contains(CLASSICAL), "it names what it has");
    }

    #[test]
    fn a_day_with_no_nakshatra_span_has_no_yogas() {
        assert!(
            yogas(CLASSICAL, Vara::Somavara, &[], &[])
                .expect("a table")
                .is_empty()
        );
    }
}
