//! The rites beyond marriage: Raman's naming, first feeding, thread
//! ceremony and house entry (*Muhurtha*, chs. VIII, IX and XII), each a
//! value of [`ActivityRules`] like the marriage.
//!
//! Each chapter is laid over Raman's general shuddhi ([`DayRules::raman`])
//! as the marriage is: where a chapter names a member its grade wins,
//! and where it is silent the shuddhi's does. A placement the chapter
//! says **must** not be bars the rite; one it says **should** not be
//! weighs against the time (crux C205), as Raman keeps bars for what he
//! ranks first. A chapter that lists the
//! good and says nothing of the rest leaves the rest middling; one that
//! says "the other signs should be avoided" rejects them. What a chapter
//! asks that the SDK does not judge — most often the child's age, which
//! a muhurta request does not carry — is listed in `unjudged`, so a
//! consumer sees it rather than a silence.
//!
//! Nepal knows these rites as nwaran, pasni, bratabandha and griha
//! pravesh; `saait-measured.md` holds them to the days the national
//! panchanga committee printed for them.

use teistro_core::catalogue::{Graha, Masa, Nakshatra, Rashi, Tithi, Vara};

use crate::activity::{ActivityRules, Bar, MonthRule, Unjudged, Unwanted};
use crate::clause::ClauseKey;
use crate::day::DayRules;
use crate::grade::{Grade, Graded};
use crate::instant::MALEFICS;
use crate::season::BlackoutKind;
use crate::tara::ChandraBala;

/// The bright half's `n`th tithi, 1 to 15 (the 15th the Purnima).
fn bright(n: u16) -> Option<Tithi> {
    Tithi::from_id(n - 1)
}

/// The dark half's `n`th tithi, 1 to 15 (the 15th the Amavasya).
fn dark(n: u16) -> Option<Tithi> {
    Tithi::from_id(n + 14)
}

/// The `n`th tithi of both halves.
fn both(numbers: &[u16]) -> Vec<Tithi> {
    numbers
        .iter()
        .flat_map(|n| [bright(*n), dark(*n)])
        .flatten()
        .collect()
}

/// The two days the Moon is full and new.
fn syzygies() -> Vec<Tithi> {
    vec![Tithi::Purnima, Tithi::Amavasya]
}

/// Monday, Wednesday, Thursday and Friday, which every chapter here
/// calls good.
fn benefic_varas() -> Vec<Vara> {
    vec![
        Vara::Somavara,
        Vara::Budhavara,
        Vara::Guruvara,
        Vara::Shukravara,
    ]
}

/// The fixed signs.
const FIXED: [Rashi; 4] = [Rashi::Taurus, Rashi::Leo, Rashi::Scorpio, Rashi::Aquarius];

/// The common signs.
const COMMON: [Rashi; 4] = [
    Rashi::Gemini,
    Rashi::Virgo,
    Rashi::Sagittarius,
    Rashi::Pisces,
];

/// The movable signs.
const MOVABLE: [Rashi; 4] = [Rashi::Aries, Rashi::Cancer, Rashi::Libra, Rashi::Capricorn];

/// The natural benefics a house can be "devoid of": Jupiter, Venus and
/// Mercury, the set the neutralisations read (`instant.rs`).
const BENEFICS: [Graha; 3] = [Graha::Jupiter, Graha::Venus, Graha::Mercury];

/// A chapter's grades laid over the general shuddhi.
fn over_shuddhi(
    tithis: Graded<Tithi>,
    nakshatras: Graded<Nakshatra>,
    varas: Graded<Vara>,
) -> DayRules {
    let general = DayRules::raman();
    DayRules {
        tithis: tithis.over(&general.tithis),
        nakshatras: nakshatras.over(&general.nakshatras),
        yogas: general.yogas,
        karanas: general.karanas,
        varas: varas.over(&general.varas),
        chandrabala: ChandraBala::raman(),
    }
}

/// A list named good, the rest left as `otherwise`.
fn good<T>(best: Vec<T>, otherwise: Grade) -> Graded<T> {
    Graded {
        best,
        middling: Vec::new(),
        rejected: Vec::new(),
        otherwise,
    }
}

/// Something asked and not judged.
fn unjudged(what: &str, why: &str) -> Unjudged {
    Unjudged {
        what: what.into(),
        why: why.into(),
    }
}

/// What the thread ceremony's chapter asks and the SDK does not judge.
fn upanayana_unjudged() -> Vec<Unjudged> {
    vec![
        unjudged(
            "the ceremony before noon",
            "a window is not yet judged by the hour of the day",
        ),
        unjudged(
            "Wednesday rejected while Mercury is combust",
            "a vara's grade that turns on a graha's asta is not yet expressed",
        ),
        unjudged(
            "the 14th good for a boy above the prescribed age",
            "the boy's age, which a muhurta request does not carry",
        ),
        unjudged(
            "the Moon in a Cancer lagna with Jupiter, malefics in the 3rd, 6th and 11th, lifting the Moon in the lagna",
            "a neutralisation of one bar by name is not yet expressed; the bar holds",
        ),
        unjudged(
            "the 3rd fortified by malefics or benefics",
            "a house wanted occupied is not yet expressed",
        ),
        unjudged(
            "the malefic yogas the chapter lists",
            "they are on p. 66, which the Internet Archive's scan lacks",
        ),
        unjudged(
            "the season's blackouts",
            "the chapter's opening, pp. 62–63, is missing from the scan; a caller adds them to `heeds`",
        ),
    ]
}

/// The child's age, which no muhurta request carries.
fn age(what: &str) -> Unjudged {
    unjudged(
        what,
        "the child's age, which a muhurta request does not carry: a consumer keeps the days that fall in it",
    )
}

impl ActivityRules {
    /// Raman's naming of the child, namakarana — Nepal's nwaran (*Muhurtha*,
    /// ch. VIII, pp. 58–59, read off the page images), over his general
    /// shuddhi.
    ///
    /// - **Stars**: Anuradha, Punarvasu, Magha, Uttara Phalguni, Uttara
    ///   Ashadha, Uttara Bhadrapada, Shatabhisha, Swati, Dhanishtha,
    ///   Shravana, Rohini, Ashwini, Mrigashira, Revati, Hasta and Pushya
    ///   good.
    /// - **Tithis**: the 4th, 6th, 8th, 9th, 12th and 14th, the Purnima
    ///   and the Amavasya avoided.
    /// - **Varas**: Monday, Wednesday, Thursday and Friday good; "other
    ///   week days are not good".
    /// - **Lagnas**: the fixed signs preferable; the common good when a
    ///   benefic occupies them, which is not judged, so they are left
    ///   middling.
    /// - **Against**: the 8th occupied, which "should be unoccupied" as
    ///   far as possible. Nothing bars.
    ///
    /// The day itself is the 10th, 12th or 16th from the birth; an
    /// elected day is for when that cannot be.
    #[must_use]
    pub fn raman_namakarana() -> ActivityRules {
        let mut rejected = both(&[4, 6, 8, 9, 12, 14]);
        rejected.extend(syzygies());
        let day = over_shuddhi(
            Graded::rejecting(rejected),
            good(
                vec![
                    Nakshatra::Anuradha,
                    Nakshatra::Punarvasu,
                    Nakshatra::Magha,
                    Nakshatra::UttaraPhalguni,
                    Nakshatra::UttaraAshadha,
                    Nakshatra::UttaraBhadrapada,
                    Nakshatra::Shatabhisha,
                    Nakshatra::Swati,
                    Nakshatra::Dhanishtha,
                    Nakshatra::Shravana,
                    Nakshatra::Rohini,
                    Nakshatra::Ashwini,
                    Nakshatra::Mrigashira,
                    Nakshatra::Revati,
                    Nakshatra::Hasta,
                    Nakshatra::Pushya,
                ],
                Grade::Middling,
            ),
            good(benefic_varas(), Grade::Rejected),
        );
        ActivityRules {
            day,
            months: MonthRule::Any,
            lagnas: Graded {
                best: FIXED.into(),
                middling: COMMON.into(),
                rejected: Vec::new(),
                otherwise: Grade::Middling,
            },
            padas: Vec::new(),
            heeds: Vec::new(),
            unwanted: vec![Unwanted::vacant(vec![8])],
            bars: Vec::new(),
            unjudged: vec![
                age("the 10th, 12th or 16th day from the birth"),
                unjudged(
                    "a common sign rising only when a benefic occupies it, and the lagna rendered strong",
                    "the lagna's strength and its occupant are not graded; a common sign is left middling",
                ),
            ],
            baseline: None,
        }
    }

    /// Raman's first feeding on rice, annaprasana — Nepal's pasni
    /// (*Muhurtha*, ch. VIII, pp. 59–60, read off the page images), over
    /// his general shuddhi.
    ///
    /// - **Stars**: Ashwini, Mrigashira, Punarvasu, Dhanishtha, Pushya,
    ///   Hasta, Swati, Anuradha, Shravana, Shatabhisha, "Uttara" (Uttara
    ///   Phalguni, as Raman names it beside Uttara Ashadha and Uttara
    ///   Bhadrapada elsewhere; crux C203) and Chitra good; Ardra,
    ///   Krittika, Jyeshtha, Bharani, Ashlesha, Purva Ashadha and Purva
    ///   Bhadrapada rejected.
    /// - **Tithis**: "the usual unfavourable lunar days", the shuddhi's.
    /// - **Varas**: Monday, Wednesday, Thursday and Friday good.
    /// - **Lagnas**: Aries, Scorpio and Pisces inauspicious.
    /// - **Bars**: the 10th occupied, which "must be unoccupied".
    /// - **Against**: Mercury in the 7th, Mars in the 8th and Venus in the
    ///   9th, which "should not occupy" them; a malefic in the lagna, the
    ///   general clause.
    /// - **Heeds** nothing: "the most important factor in this function
    ///   is the month", the child's, so Jupiter and Venus combust are not
    ///   minded.
    #[must_use]
    pub fn raman_annaprasana() -> ActivityRules {
        let day = over_shuddhi(
            Graded::none(),
            Graded {
                best: vec![
                    Nakshatra::Ashwini,
                    Nakshatra::Mrigashira,
                    Nakshatra::Punarvasu,
                    Nakshatra::Dhanishtha,
                    Nakshatra::Pushya,
                    Nakshatra::Hasta,
                    Nakshatra::Swati,
                    Nakshatra::Anuradha,
                    Nakshatra::Shravana,
                    Nakshatra::Shatabhisha,
                    Nakshatra::UttaraPhalguni,
                    Nakshatra::Chitra,
                ],
                middling: Vec::new(),
                rejected: vec![
                    Nakshatra::Ardra,
                    Nakshatra::Krittika,
                    Nakshatra::Jyeshtha,
                    Nakshatra::Bharani,
                    Nakshatra::Ashlesha,
                    Nakshatra::PurvaAshadha,
                    Nakshatra::PurvaBhadrapada,
                ],
                otherwise: Grade::Middling,
            },
            good(benefic_varas(), Grade::Middling),
        );
        ActivityRules {
            day,
            months: MonthRule::Any,
            lagnas: Graded::rejecting(vec![Rashi::Aries, Rashi::Scorpio, Rashi::Pisces]),
            padas: Vec::new(),
            heeds: Vec::new(),
            unwanted: vec![
                Unwanted::vacant(vec![10]).barring(),
                Unwanted::of(vec![Graha::Mercury], vec![7]),
                Unwanted::of(vec![Graha::Mars], vec![8]),
                Unwanted::of(vec![Graha::Venus], vec![9]),
            ],
            bars: Vec::new(),
            unjudged: vec![age("the 6th, 8th, 9th or 12th month of the child")],
            baseline: None,
        }
    }

    /// Raman's thread ceremony, upanayana — Nepal's bratabandha
    /// (*Muhurtha*, ch. VIII, pp. 64–65, read off the page images), over
    /// his general shuddhi. The scan lacks pp. 62–63, where the chapter
    /// begins, and p. 66, where its malefic yogas are listed.
    ///
    /// - **Months** (lunar): Magha, Phalguna, Chaitra and Vaishakha good,
    ///   all four in the Sun's northern course the chapter calls the best
    ///   season; the rest left middling.
    /// - **Tithis**: the bright 2nd, 3rd, 5th, 7th, 10th and 13th and the
    ///   dark 1st, 2nd and 3rd good; the 4th, 8th, 9th, 11th, 12th and
    ///   14th, the Purnima and the Amavasya avoided.
    /// - **Varas**: Monday, Wednesday, Thursday and Friday good; Sunday
    ///   ordinary; Tuesday "invariably rejected".
    /// - **Stars**: Anuradha, Hasta, Chitra, Swati, Shravana, Dhanishtha,
    ///   Shatabhisha, Uttara Phalguni, Uttara Ashadha, Uttara Bhadrapada,
    ///   Revati, Rohini, Mrigashira, Ashwini, Punarvasu and Pushya good.
    /// - **Lagnas**: Aries, Taurus, Gemini, Cancer, Virgo, Libra and
    ///   Aquarius good; "the other signs should be avoided".
    /// - **Bars**: the Moon in the 6th, 8th or 12th, and the 8th
    ///   occupied, which each "must" not be.
    /// - **Against**: a malefic in a kendra, a benefic in the 6th, Mars or
    ///   Saturn in the 2nd, 5th or 12th, and the Moon in the lagna, which
    ///   each "should" not be.
    #[must_use]
    pub fn raman_upanayana() -> ActivityRules {
        let mut best: Vec<Tithi> = [2, 3, 5, 7, 10, 13]
            .into_iter()
            .filter_map(bright)
            .collect();
        best.extend([1, 2, 3].into_iter().filter_map(dark));
        let mut rejected = both(&[4, 8, 9, 11, 12, 14]);
        rejected.extend(syzygies());
        let day = over_shuddhi(
            Graded {
                best,
                middling: Vec::new(),
                rejected,
                otherwise: Grade::Middling,
            },
            good(
                vec![
                    Nakshatra::Anuradha,
                    Nakshatra::Hasta,
                    Nakshatra::Chitra,
                    Nakshatra::Swati,
                    Nakshatra::Shravana,
                    Nakshatra::Dhanishtha,
                    Nakshatra::Shatabhisha,
                    Nakshatra::UttaraPhalguni,
                    Nakshatra::UttaraAshadha,
                    Nakshatra::UttaraBhadrapada,
                    Nakshatra::Revati,
                    Nakshatra::Rohini,
                    Nakshatra::Mrigashira,
                    Nakshatra::Ashwini,
                    Nakshatra::Punarvasu,
                    Nakshatra::Pushya,
                ],
                Grade::Middling,
            ),
            Graded {
                best: benefic_varas(),
                middling: vec![Vara::Ravivara],
                rejected: vec![Vara::Mangalavara],
                otherwise: Grade::Middling,
            },
        );
        ActivityRules {
            day,
            months: MonthRule::Lunar {
                months: good(
                    vec![Masa::Magha, Masa::Phalguna, Masa::Chaitra, Masa::Vaishakha],
                    Grade::Middling,
                ),
                with_sun: Vec::new(),
            },
            lagnas: good(
                vec![
                    Rashi::Aries,
                    Rashi::Taurus,
                    Rashi::Gemini,
                    Rashi::Cancer,
                    Rashi::Virgo,
                    Rashi::Libra,
                    Rashi::Aquarius,
                ],
                Grade::Rejected,
            ),
            padas: Vec::new(),
            heeds: Vec::new(),
            unwanted: vec![
                Unwanted::of(MALEFICS.into(), vec![1, 4, 7, 10]),
                Unwanted::vacant(vec![8]).barring(),
                Unwanted::of(BENEFICS.into(), vec![6]),
                Unwanted::of(vec![Graha::Mars, Graha::Saturn], vec![2, 5, 12]),
                Unwanted::of(vec![Graha::Moon], vec![1]),
            ],
            bars: vec![Bar::Key(ClauseKey::MoonInDusthana)],
            unjudged: upanayana_unjudged(),
            baseline: None,
        }
    }

    /// Raman's entry into a new house, griha pravesha — Nepal's griha
    /// pravesh (*Muhurtha*, ch. XII, pp. 134–136, read off the page
    /// images), over his general shuddhi.
    ///
    /// - **Months** (lunar): Vaishakha, Jyeshtha, Magha and Phalguna best;
    ///   Kartika and Margashirsha "neutral or middling"; the rest, below
    ///   middling, rejected. The chapter also asks the Sun's northern
    ///   course, which the two middling months are not in; the months
    ///   are graded as printed (crux C204).
    /// - **Tithis**: the dark 1st and the bright 2nd, 3rd, 5th, 7th,
    ///   10th, 11th and 13th the most auspicious.
    /// - **Stars**: Rohini, Mrigashira, Uttara Phalguni, Uttara Ashadha,
    ///   Chitra and Uttara Bhadrapada best; Anuradha and Revati
    ///   permissible; "the other constellations should be rejected".
    /// - **Varas**: Monday, Wednesday, Thursday and Friday good; Saturday,
    ///   which "some" allow at a risk of thefts, left to the shuddhi,
    ///   which rejects it.
    /// - **Lagnas**: fixed best, common ordinary, movable avoided.
    /// - **Against**: the 8th occupied, which "should be vacant". Nothing
    ///   bars.
    /// - **Heeds**: Guru and Shukra asta, since the chapter asks Jupiter
    ///   and Venus "strongly disposed" and a set graha is not.
    #[must_use]
    pub fn raman_griha_pravesha() -> ActivityRules {
        let mut best: Vec<Tithi> = dark(1).into_iter().collect();
        best.extend([2, 3, 5, 7, 10, 11, 13].into_iter().filter_map(bright));
        let day = over_shuddhi(
            good(best, Grade::Middling),
            Graded {
                best: vec![
                    Nakshatra::Rohini,
                    Nakshatra::Mrigashira,
                    Nakshatra::UttaraPhalguni,
                    Nakshatra::UttaraAshadha,
                    Nakshatra::Chitra,
                    Nakshatra::UttaraBhadrapada,
                ],
                middling: vec![Nakshatra::Anuradha, Nakshatra::Revati],
                rejected: Vec::new(),
                otherwise: Grade::Rejected,
            },
            good(benefic_varas(), Grade::Middling),
        );
        ActivityRules {
            day,
            months: MonthRule::Lunar {
                months: Graded {
                    best: vec![Masa::Vaishakha, Masa::Jyeshtha, Masa::Magha, Masa::Phalguna],
                    middling: vec![Masa::Kartika, Masa::Margashirsha],
                    rejected: Vec::new(),
                    otherwise: Grade::Rejected,
                },
                with_sun: Vec::new(),
            },
            lagnas: Graded {
                best: FIXED.into(),
                middling: COMMON.into(),
                rejected: MOVABLE.into(),
                otherwise: Grade::Middling,
            },
            padas: Vec::new(),
            heeds: vec![BlackoutKind::GuruAsta, BlackoutKind::ShukraAsta],
            unwanted: vec![Unwanted::vacant(vec![8])],
            bars: Vec::new(),
            unjudged: vec![
                unjudged(
                    "a movable sign rising when the navamsa lagna is Taurus",
                    "a lagna's grade that turns on its navamsa is not yet expressed; the movable signs stay rejected",
                ),
                unjudged(
                    "malefics in the upachayas, benefics in the kendras, the Moon strong, the lagna Jupiter's or Venus's",
                    "these favour the time and are not graded",
                ),
                unjudged(
                    "the Sun in its northern course",
                    "the months are graded as printed, two of them in the southern course (crux C204)",
                ),
            ],
            baseline: None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "tests fail by panicking")]
mod tests {
    use super::{ActivityRules, Unwanted};
    use crate::clause::ClauseKind;
    use crate::grade::Grade;
    use crate::instant::Sky;
    use teistro_core::catalogue::{Graha, Masa, Nakshatra, Rashi, Tithi, Vara};
    use teistro_core::interval::Interval;

    fn at() -> Interval {
        Interval::literal(2_460_000.5, 2_460_000.6)
    }

    /// Aries rising, every graha in Sagittarius (the 9th) but those moved.
    fn sky(moved: &[(Graha, f64)]) -> Sky {
        let mut grahas = [245.0; 9];
        for (graha, deg) in moved {
            if let Some(slot) = grahas.get_mut(usize::from(graha.id())) {
                *slot = *deg;
            }
        }
        Sky {
            lagna_deg: 5.0,
            grahas,
            speeds: [1.0; 9],
        }
    }

    fn unwanted(rules: &ActivityRules, sky: &Sky) -> Vec<ClauseKind> {
        rules
            .instant_clauses(sky, at())
            .into_iter()
            .map(|c| c.kind)
            .filter(|k| matches!(k, ClauseKind::UnwantedPlacement { .. }))
            .collect()
    }

    #[test]
    fn a_house_wanted_vacant_reports_whoever_stands_in_it_once() {
        // Scorpio is the 8th from Aries; Mars and Jupiter stand there.
        let rules = ActivityRules::raman_griha_pravesha();
        let found = unwanted(
            &rules,
            &sky(&[(Graha::Mars, 215.0), (Graha::Jupiter, 220.0)]),
        );
        assert_eq!(
            found,
            [ClauseKind::UnwantedPlacement {
                house: 8,
                by: vec![Graha::Mars, Graha::Jupiter],
            }]
        );
        assert!(unwanted(&rules, &sky(&[])).is_empty());
    }

    #[test]
    fn the_first_feeding_wants_each_graha_out_of_its_own_house() {
        let rules = ActivityRules::raman_annaprasana();
        // Mercury in Libra (the 7th) is unwanted; Venus there is not.
        // Venus is moved to the 7th in both, out of the 9th.
        let mercury = unwanted(
            &rules,
            &sky(&[(Graha::Mercury, 185.0), (Graha::Venus, 190.0)]),
        );
        assert_eq!(
            mercury,
            [ClauseKind::UnwantedPlacement {
                house: 7,
                by: vec![Graha::Mercury],
            }]
        );
        assert!(unwanted(&rules, &sky(&[(Graha::Venus, 185.0)])).is_empty());
        // Venus in Sagittarius (the 9th) is; so the default sky, with
        // every graha there, names Venus alone.
        assert_eq!(
            unwanted(&rules, &sky(&[])),
            [ClauseKind::UnwantedPlacement {
                house: 9,
                by: vec![Graha::Venus],
            }]
        );
    }

    #[test]
    fn the_thread_ceremony_reads_its_chapter_over_the_shuddhi() {
        let rules = ActivityRules::raman_upanayana();
        let day = &rules.day;
        assert_eq!(day.tithis.grade(&Tithi::KrishnaPratipada), Grade::Best);
        assert_eq!(day.tithis.grade(&Tithi::ShuklaEkadashi), Grade::Rejected);
        // The bright 6th: the chapter is silent, the shuddhi rejects it.
        assert_eq!(day.tithis.grade(&Tithi::ShuklaShashthi), Grade::Rejected);
        assert_eq!(day.varas.grade(&Vara::Ravivara), Grade::Middling);
        assert_eq!(day.varas.grade(&Vara::Mangalavara), Grade::Rejected);
        assert_eq!(day.nakshatras.grade(&Nakshatra::Bharani), Grade::Rejected);
        assert_eq!(rules.lagnas.grade(&Rashi::Leo), Grade::Rejected);
        // A benefic in the 6th and the Moon in the lagna are unwanted.
        let found = unwanted(&rules, &sky(&[(Graha::Moon, 5.0), (Graha::Venus, 155.0)]));
        assert!(found.contains(&ClauseKind::UnwantedPlacement {
            house: 6,
            by: vec![Graha::Venus],
        }));
        assert!(found.contains(&ClauseKind::UnwantedPlacement {
            house: 1,
            by: vec![Graha::Moon],
        }));
    }

    #[test]
    fn a_must_bars_and_a_should_weighs() {
        let rules = ActivityRules::raman_upanayana();
        let clause = |kind| crate::clause::Clause { kind, at: at() };
        let eighth = ClauseKind::UnwantedPlacement {
            house: 8,
            by: vec![Graha::Jupiter],
        };
        let kendra = ClauseKind::UnwantedPlacement {
            house: 10,
            by: vec![Graha::Saturn],
        };
        assert!(rules.bars(&clause(eighth.clone())));
        assert!(!rules.bars(&clause(kendra.clone())));
        // Judged, the 8th bars as its own clause and the kendra weighs.
        let held = [clause(eighth.clone()), clause(kendra)];
        let judged = crate::judge::Judgement::of(at(), &held, &rules);
        assert_eq!(judged.barred_by, [crate::activity::Bar::Clause(eighth)]);
        assert_eq!(judged.against_count(), 2);
        // A consumer who bars every placement still can, by its key, which
        // then names the 8th too, once.
        let every = crate::activity::Bar::Key(crate::clause::ClauseKey::UnwantedPlacement);
        let mut strict = rules;
        strict.bars.push(every.clone());
        let judged = crate::judge::Judgement::of(at(), &held, &strict);
        assert_eq!(judged.barred_by, [every]);
    }

    #[test]
    fn the_house_entry_grades_its_months_as_printed() {
        let rules = ActivityRules::raman_griha_pravesha();
        let months = match &rules.months {
            crate::activity::MonthRule::Lunar { months, .. } => Some(months),
            _ => None,
        }
        .expect("a lunar month rule");
        assert_eq!(months.grade(&Masa::Magha), Grade::Best);
        assert_eq!(months.grade(&Masa::Kartika), Grade::Middling);
        assert_eq!(months.grade(&Masa::Ashadha), Grade::Rejected);
        assert_eq!(
            rules.day.nakshatras.grade(&Nakshatra::Revati),
            Grade::Middling
        );
        assert_eq!(
            rules.day.nakshatras.grade(&Nakshatra::Pushya),
            Grade::Rejected
        );
        assert_eq!(rules.lagnas.grade(&Rashi::Libra), Grade::Rejected);
    }

    #[test]
    fn the_naming_rejects_the_weekdays_it_does_not_call_good() {
        let rules = ActivityRules::raman_namakarana();
        assert_eq!(rules.day.varas.grade(&Vara::Ravivara), Grade::Rejected);
        assert_eq!(
            rules.day.tithis.grade(&Tithi::ShuklaNavami),
            Grade::Rejected
        );
        assert_eq!(rules.day.nakshatras.grade(&Nakshatra::Magha), Grade::Best);
    }

    #[test]
    fn every_rite_checks_and_round_trips() {
        for rules in [
            ActivityRules::raman_namakarana(),
            ActivityRules::raman_annaprasana(),
            ActivityRules::raman_upanayana(),
            ActivityRules::raman_griha_pravesha(),
        ] {
            rules.check().expect("the shipped rules check");
            let json = serde_json::to_string(&rules).expect("the rules serialise");
            let back: ActivityRules = serde_json::from_str(&json).expect("and read back");
            assert_eq!(back, rules);
        }
    }

    #[test]
    fn a_house_or_a_quarter_that_does_not_exist_is_refused_by_field() {
        let mut rules = ActivityRules::raman_namakarana();
        rules.unwanted.push(Unwanted::vacant(vec![13]));
        let refused = rules.check().expect_err("house 13");
        assert_eq!(refused.field(), Some("unwanted[1].houses"));
        let mut rules = ActivityRules::raman_marriage();
        rules.padas.first_mut().expect("a quarter").pada = 5;
        let refused = rules.check().expect_err("quarter 5");
        assert_eq!(refused.field(), Some("padas[0].pada"));
    }

    #[test]
    fn rules_spelt_out_without_unwanted_still_read() {
        let mut json = serde_json::to_value(ActivityRules::raman_marriage()).expect("json");
        json.as_object_mut().expect("an object").remove("unwanted");
        let back: ActivityRules = serde_json::from_value(json).expect("reads");
        assert!(back.unwanted.is_empty());
    }
}
