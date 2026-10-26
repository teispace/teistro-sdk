//! The rites beyond marriage: Raman's naming, first feeding, ear boring,
//! tonsure, thread ceremony, commencing education, house entry and
//! journey, and his elections for buying for business, selling, new
//! clothes and a vehicle (*Muhurtha*, chs. IV, VIII, X, XI, XII and XIV),
//! each a value of [`ActivityRules`] like the marriage.
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

use crate::activity::{ActivityRules, Bar, Hours, MonthRule, Pada, Shool, Unjudged, Unwanted};
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
            "the chapter says it should, and a should weighs (crux C205); `hours` bars, so a caller who holds the forenoon sets it to `FORENOON`",
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

/// The ghatis past sunrise from which a journey towards the day's disha
/// shool may begin, weekday by weekday (Raman, *Muhurtha*, ch. XIV,
/// p. 157).
const YATRA_SHOOL: [(Vara, u8); 7] = [
    (Vara::Guruvara, 22),
    (Vara::Mangalavara, 12),
    (Vara::Budhavara, 12),
    (Vara::Shukravara, 15),
    (Vara::Ravivara, 15),
    (Vara::Shanivara, 8),
    (Vara::Somavara, 8),
];

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
    ///
    /// ```
    /// use teistro_muhurta::ActivityRules;
    ///
    /// let naming = ActivityRules::raman_namakarana();
    /// naming.check()?;
    /// // The child's age, which a request does not carry, is said rather than dropped.
    /// assert!(!naming.unjudged.is_empty());
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
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
            hours: Hours::Any,
            shool: Vec::new(),
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
    ///
    /// ```
    /// use teistro_core::catalogue::Vara;
    /// use teistro_muhurta::{ActivityRules, Grade};
    ///
    /// let feeding = ActivityRules::raman_annaprasana();
    /// feeding.check()?;
    /// assert_eq!(feeding.day.varas.grade(&Vara::Guruvara), Grade::Best);
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
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
            hours: Hours::Any,
            shool: Vec::new(),
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
    ///
    /// ```
    /// use teistro_muhurta::ActivityRules;
    ///
    /// let thread = ActivityRules::raman_upanayana();
    /// thread.check()?;
    /// assert_ne!(thread, ActivityRules::raman_marriage());
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
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
            hours: Hours::Any,
            shool: Vec::new(),
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
    ///
    /// ```
    /// use teistro_muhurta::ActivityRules;
    ///
    /// let entry = ActivityRules::raman_griha_pravesha();
    /// entry.check()?;
    /// assert!(entry.baseline.is_none());
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
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
            hours: Hours::Any,
            shool: Vec::new(),
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

    /// Raman's ear boring, karnavedha (*Muhurtha*, ch. VIII, p. 60), over
    /// his general shuddhi, in daylight: "in the forenoon or in the
    /// afternoon but never during night".
    ///
    /// - **Tithis**: "the other lunar days usually declared as
    ///   inauspicious", the general shuddhi's.
    /// - **Varas**: Monday, Wednesday, Thursday and Friday good.
    /// - **Lagnas**: Aquarius, Leo and Scorpio "should be rejected".
    /// - **Against**: the 8th occupied, which "should be unoccupied".
    ///
    /// ```
    /// use teistro_muhurta::{ActivityRules, Hours};
    ///
    /// let boring = ActivityRules::raman_karnavedha();
    /// boring.check()?;
    /// assert_eq!(boring.hours, Hours::Daylight);
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn raman_karnavedha() -> ActivityRules {
        ActivityRules {
            day: over_shuddhi(
                Graded::none(),
                Graded::none(),
                good(benefic_varas(), Grade::Middling),
            ),
            months: MonthRule::Any,
            lagnas: Graded::rejecting(vec![Rashi::Aquarius, Rashi::Leo, Rashi::Scorpio]),
            hours: Hours::Daylight,
            shool: Vec::new(),
            padas: Vec::new(),
            heeds: Vec::new(),
            unwanted: vec![Unwanted::vacant(vec![8])],
            bars: Vec::new(),
            unjudged: vec![
                age("the 12th or 16th day of the child, or its 6th, 7th or 8th month"),
                unjudged(
                    "a day ruled by two asterisms or two lunar days",
                    "a day's count of stars or tithis is not graded; each window is judged by the star and tithi it holds",
                ),
            ],
            baseline: None,
        }
    }

    /// Raman's tonsure, chudakarana, his *chowlam* (*Muhurtha*, ch. VIII,
    /// pp. 60–61), over his general shuddhi, in the forenoon, where it
    /// "should always be done". The scan lacks pp. 62–63, where the
    /// chapter's lagnas end.
    ///
    /// - **Tithis**: the bright 2nd, 3rd, 5th, 7th, 10th, 11th and 13th
    ///   good, the dark half's same days ordinary, since "the dark
    ///   fortnight is supposed to affect the health"; the 1st, 4th, 6th,
    ///   8th, 9th and 14th, the Purnima and the Amavasya rejected.
    /// - **Stars**: Punarvasu, Mrigashira, Dhanishtha, Shravana, Revati,
    ///   Pushya, Chitra, Ashwini and Hasta favourable; Swati, Rohini,
    ///   Shatabhisha and the three Uttaras ordinary. Naming an ordinary
    ///   class below the favourable one, he leaves the rest below both:
    ///   rejected.
    /// - **Varas**: Monday, Wednesday, Thursday and Friday good.
    /// - **Lagnas**: Cancer, Virgo, Gemini, Pisces, Libra, Taurus and
    ///   Capricorn good; the rest "not beneficial" but usable with a
    ///   benefic in the lagna, which is not judged, so middling.
    /// - **Heeds**: Guru and Shukra asta, "when Jupiter and Venus are free
    ///   from combustion".
    ///
    /// ```
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_muhurta::{ActivityRules, Grade, Hours};
    ///
    /// let tonsure = ActivityRules::raman_chudakarana();
    /// tonsure.check()?;
    /// assert_eq!(tonsure.hours, Hours::Forenoon);
    /// assert_eq!(tonsure.day.nakshatras.grade(&Nakshatra::Swati), Grade::Middling);
    /// assert_eq!(tonsure.day.nakshatras.grade(&Nakshatra::Bharani), Grade::Rejected);
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn raman_chudakarana() -> ActivityRules {
        let good_days = [2, 3, 5, 7, 10, 11, 13];
        let mut rejected = both(&[1, 4, 6, 8, 9, 14]);
        rejected.extend(syzygies());
        let day = over_shuddhi(
            Graded {
                best: good_days.into_iter().filter_map(bright).collect(),
                middling: good_days.into_iter().filter_map(dark).collect(),
                rejected,
                otherwise: Grade::Middling,
            },
            Graded {
                best: vec![
                    Nakshatra::Punarvasu,
                    Nakshatra::Mrigashira,
                    Nakshatra::Dhanishtha,
                    Nakshatra::Shravana,
                    Nakshatra::Revati,
                    Nakshatra::Pushya,
                    Nakshatra::Chitra,
                    Nakshatra::Ashwini,
                    Nakshatra::Hasta,
                ],
                middling: vec![
                    Nakshatra::Swati,
                    Nakshatra::Rohini,
                    Nakshatra::Shatabhisha,
                    Nakshatra::UttaraPhalguni,
                    Nakshatra::UttaraBhadrapada,
                    Nakshatra::UttaraAshadha,
                ],
                rejected: Vec::new(),
                otherwise: Grade::Rejected,
            },
            good(benefic_varas(), Grade::Middling),
        );
        ActivityRules {
            day,
            months: MonthRule::Any,
            lagnas: good(
                vec![
                    Rashi::Cancer,
                    Rashi::Virgo,
                    Rashi::Gemini,
                    Rashi::Pisces,
                    Rashi::Libra,
                    Rashi::Taurus,
                    Rashi::Capricorn,
                ],
                Grade::Middling,
            ),
            hours: Hours::Forenoon,
            shool: Vec::new(),
            padas: Vec::new(),
            heeds: vec![BlackoutKind::GuruAsta, BlackoutKind::ShukraAsta],
            unwanted: Vec::new(),
            bars: Vec::new(),
            unjudged: vec![
                age("the 3rd or 5th year of the child"),
                unjudged(
                    "not while the child's mother is pregnant",
                    "the mother's state, which a muhurta request does not carry",
                ),
                unjudged(
                    "the Sun in the Tropic of Cancer",
                    "read as either the Sun's northern course or its solstice; neither is applied, and a caller who keeps one sets `months`",
                ),
                unjudged(
                    "a sign other than the seven rising with a benefic in the lagna",
                    "the lagna's occupant is not graded; the other signs are left middling",
                ),
                unjudged(
                    "the rest of the chapter",
                    "pp. 62–63 are missing from the Internet Archive's scan",
                ),
            ],
            baseline: None,
        }
    }

    /// Raman's commencing of education, vidyarambha (*Muhurtha*, ch. XI,
    /// pp. 114–117), over his general shuddhi.
    ///
    /// - **Stars**: Mrigashira, Ardra, Punarvasu, Pushya, Hasta, Chitra,
    ///   Swati, Shravana, Dhanishtha and Shatabhisha the most beneficial;
    ///   Ashwini, "held by some writers to be one of the best", and
    ///   Rohini, Uttara Phalguni, Uttara Ashadha and Revati neutral; "the
    ///   remaining ones are to be rejected".
    /// - **Tithis**: the dark 1st, and the 2nd, 3rd, 5th, 6th, 10th and
    ///   11th, highly auspicious; the 4th, 8th, 9th and 14th, the Purnima
    ///   and the Amavasya avoided.
    /// - **Varas**: Tuesday, Saturday and Sunday "invariably avoided";
    ///   Wednesday the best.
    /// - **Lagnas**: common signs the best, movable ordinary, fixed not.
    /// - **Against**: the 8th occupied, which "should be clean and
    ///   unoccupied".
    ///
    /// ```
    /// use teistro_core::catalogue::{Rashi, Vara};
    /// use teistro_muhurta::{ActivityRules, Grade};
    ///
    /// let learning = ActivityRules::raman_vidyarambha();
    /// learning.check()?;
    /// assert_eq!(learning.day.varas.grade(&Vara::Budhavara), Grade::Best);
    /// assert_eq!(learning.lagnas.grade(&Rashi::Leo), Grade::Rejected);
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn raman_vidyarambha() -> ActivityRules {
        let mut best = both(&[2, 3, 5, 6, 10, 11]);
        best.extend(dark(1));
        let mut rejected = both(&[4, 8, 9, 14]);
        rejected.extend(syzygies());
        let day = over_shuddhi(
            Graded {
                best,
                middling: Vec::new(),
                rejected,
                otherwise: Grade::Middling,
            },
            Graded {
                best: vec![
                    Nakshatra::Mrigashira,
                    Nakshatra::Ardra,
                    Nakshatra::Punarvasu,
                    Nakshatra::Pushya,
                    Nakshatra::Hasta,
                    Nakshatra::Chitra,
                    Nakshatra::Swati,
                    Nakshatra::Shravana,
                    Nakshatra::Dhanishtha,
                    Nakshatra::Shatabhisha,
                ],
                middling: vec![
                    Nakshatra::Ashwini,
                    Nakshatra::Rohini,
                    Nakshatra::UttaraPhalguni,
                    Nakshatra::UttaraAshadha,
                    Nakshatra::Revati,
                ],
                rejected: Vec::new(),
                otherwise: Grade::Rejected,
            },
            Graded {
                best: vec![Vara::Budhavara],
                middling: Vec::new(),
                rejected: vec![Vara::Mangalavara, Vara::Shanivara, Vara::Ravivara],
                otherwise: Grade::Middling,
            },
        );
        ActivityRules {
            day,
            months: MonthRule::Any,
            lagnas: Graded {
                best: COMMON.into(),
                middling: MOVABLE.into(),
                rejected: FIXED.into(),
                otherwise: Grade::Middling,
            },
            hours: Hours::Any,
            shool: Vec::new(),
            padas: Vec::new(),
            heeds: Vec::new(),
            unwanted: vec![Unwanted::vacant(vec![8])],
            bars: Vec::new(),
            unjudged: vec![
                unjudged(
                    "the forenoon and the noon the best, and Wednesday morning",
                    "the hours are a should, and a should weighs (crux C205); `hours` bars, so a caller who holds the forenoon sets it to `FORENOON`",
                ),
                unjudged(
                    "malefics in the 3rd, 6th and 11th",
                    "a house wanted occupied is not yet expressed",
                ),
                unjudged(
                    "the Saraswati and Vidya yogas",
                    "combinations that favour the time are not graded",
                ),
                unjudged(
                    "the stars and grahas for each subject: the Vedas, astrology, grammar, medicine, music",
                    "one rule set is the general commencement; a caller who elects for a subject grades its stars in `day.nakshatras`",
                ),
            ],
            baseline: None,
        }
    }

    /// Raman's journey, yatra (*Muhurtha*, ch. XIV, pp. 156–159), over his
    /// general shuddhi: what he calls the most essential, a good lunar
    /// day, a favourable star and a fortified lagna.
    ///
    /// - **Tithis**: the 2nd, 3rd, 5th, 7th, 10th, 11th and 13th the best;
    ///   the 14th, the Purnima and the Amavasya "avoided at any cost".
    /// - **Stars**: Mrigashira, Ashwini, Pushya, Punarvasu, Hasta,
    ///   Anuradha, Shravana, Mula, Dhanishtha and Revati, after which the
    ///   traveller returns early with the work done; no journey in
    ///   Krittika, Bharani, Ashlesha, Vishakha, Purva Phalguni, Purva
    ///   Bhadrapada or Ardra. The good stars' first quarter "may be
    ///   avoided as far as possible".
    /// - **Varas**: Tuesday, which "must be avoided" in his view.
    /// - **Lagnas**: Aries, Taurus, Cancer, Leo, Libra and Sagittarius
    ///   favourable.
    /// - **Heeds**: the sankranti, since the days the Sun enters a sign,
    ///   the equinoxes among them, are avoided.
    /// - **Direction** (`shool`): asked one, a day whose disha shool it is
    ///   opens past 22 ghatis on Thursday, 12 on Tuesday and Wednesday, 15
    ///   on Friday and Sunday and 8 on Saturday and Monday.
    ///
    /// ```
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_muhurta::{ActivityRules, Grade};
    ///
    /// let journey = ActivityRules::raman_yatra();
    /// journey.check()?;
    /// assert_eq!(journey.day.nakshatras.grade(&Nakshatra::Krittika), Grade::Rejected);
    /// assert!(journey.padas.iter().all(|p| p.pada == 1));
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn raman_yatra() -> ActivityRules {
        let mut rejected = both(&[14]);
        rejected.extend(syzygies());
        let returning = vec![
            Nakshatra::Mrigashira,
            Nakshatra::Ashwini,
            Nakshatra::Pushya,
            Nakshatra::Punarvasu,
            Nakshatra::Hasta,
            Nakshatra::Anuradha,
            Nakshatra::Shravana,
            Nakshatra::Mula,
            Nakshatra::Dhanishtha,
            Nakshatra::Revati,
        ];
        let padas = returning
            .iter()
            .map(|nakshatra| Pada {
                nakshatra: *nakshatra,
                pada: 1,
            })
            .collect();
        let day = over_shuddhi(
            Graded {
                best: both(&[2, 3, 5, 7, 10, 11, 13]),
                middling: Vec::new(),
                rejected,
                otherwise: Grade::Middling,
            },
            Graded {
                best: returning,
                middling: Vec::new(),
                rejected: vec![
                    Nakshatra::Krittika,
                    Nakshatra::Bharani,
                    Nakshatra::Ashlesha,
                    Nakshatra::Vishakha,
                    Nakshatra::PurvaPhalguni,
                    Nakshatra::PurvaBhadrapada,
                    Nakshatra::Ardra,
                ],
                otherwise: Grade::Middling,
            },
            Graded::rejecting(vec![Vara::Mangalavara]),
        );
        ActivityRules {
            day,
            months: MonthRule::Any,
            lagnas: good(
                vec![
                    Rashi::Aries,
                    Rashi::Taurus,
                    Rashi::Cancer,
                    Rashi::Leo,
                    Rashi::Libra,
                    Rashi::Sagittarius,
                ],
                Grade::Middling,
            ),
            hours: Hours::Any,
            shool: YATRA_SHOOL
                .map(|(vara, after_ghatis)| Shool { vara, after_ghatis })
                .into(),
            padas,
            heeds: vec![BlackoutKind::Sankranti],
            unwanted: Vec::new(),
            bars: Vec::new(),
            unjudged: vec![
                unjudged(
                    "the direction of travel",
                    "judged only when the request names one (`direction`); unasked, every day is open to every direction",
                ),
                unjudged(
                    "no journey south while the nakshatra panchaka runs (ch. IV, p. 26)",
                    "the panchaka's own direction is not yet read against the request's",
                ),
                unjudged(
                    "the forbidden stars past their ghatis when the journey cannot wait: Krittika 13, Bharani 7, Magha 14, the three Purvas 16, Swati, Ashlesha and Vishakha 14",
                    "Raman takes it only in urgency and rejects Bharani and Krittika invariably; the stars stay rejected",
                ),
                unjudged(
                    "the lagna the native's janma rashi, never the janma lagna, nor the 5th, 7th or 9th from it",
                    "a lagna's grade that turns on the native's chart is not yet expressed",
                ),
                unjudged(
                    "the whole day the Sun enters a sign",
                    "the sankranti closes the sixteen ghatis either side of the ingress, not the civil day",
                ),
                unjudged(
                    "Jupiter or Venus in the lagna, the Moon in the 3rd, 6th, 9th or 12th with Jupiter in a kendra, and the chapter's other combinations",
                    "combinations that favour the time are not graded",
                ),
                unjudged(
                    "the rules for short, long, air, sea, business journeys and pilgrimage",
                    "one rule set is the general journey; a caller adds a purpose's houses to `unwanted`",
                ),
            ],
            baseline: None,
        }
    }

    /// Raman's buying for business (*Muhurtha*, ch. X, p. 112), over his
    /// general shuddhi.
    ///
    /// - **Tithis**: the 10th the best.
    /// - **Stars**: Pushya the best.
    /// - **Varas**: Thursday the best; Tuesday "completely rejected";
    ///   Saturday passable.
    ///
    /// ```
    /// use teistro_core::catalogue::Vara;
    /// use teistro_muhurta::{ActivityRules, Grade};
    ///
    /// let buying = ActivityRules::raman_trade_purchase();
    /// buying.check()?;
    /// assert_eq!(buying.day.varas.grade(&Vara::Shanivara), Grade::Middling);
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn raman_trade_purchase() -> ActivityRules {
        let day = over_shuddhi(
            good(both(&[10]), Grade::Middling),
            good(vec![Nakshatra::Pushya], Grade::Middling),
            Graded {
                best: vec![Vara::Guruvara],
                middling: vec![Vara::Shanivara],
                rejected: vec![Vara::Mangalavara],
                otherwise: Grade::Middling,
            },
        );
        ActivityRules {
            unjudged: vec![unjudged(
                "Mercury, the 2nd lord and the 2nd house fortified, Mercury free of Mars, and Mercury with Jupiter in the lagna or in mutual aspect",
                "a graha's strength and aspects are not graded",
            )],
            ..ActivityRules::general(day)
        }
    }

    /// Raman's selling for profit (*Muhurtha*, ch. X, p. 112), over his
    /// general shuddhi.
    ///
    /// - **Varas**: Monday, Wednesday and Thursday the best; Saturday
    ///   middling; Tuesday avoided and Friday unpropitious.
    ///
    /// ```
    /// use teistro_core::catalogue::Vara;
    /// use teistro_muhurta::{ActivityRules, Grade};
    ///
    /// let selling = ActivityRules::raman_selling();
    /// selling.check()?;
    /// assert_eq!(selling.day.varas.grade(&Vara::Shukravara), Grade::Rejected);
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn raman_selling() -> ActivityRules {
        let day = over_shuddhi(
            Graded::none(),
            Graded::none(),
            Graded {
                best: vec![Vara::Somavara, Vara::Budhavara, Vara::Guruvara],
                middling: vec![Vara::Shanivara],
                rejected: vec![Vara::Mangalavara, Vara::Shukravara],
                otherwise: Grade::Middling,
            },
        );
        ActivityRules {
            unjudged: vec![unjudged(
                "the Moon and Mercury free of Mars, the Moon in Taurus, Cancer or Pisces, and Mercury in a kendra or aspected by Jupiter",
                "a graha's sign and aspects are not graded",
            )],
            ..ActivityRules::general(day)
        }
    }

    /// Raman's wearing of new clothes (*Muhurtha*, ch. X, p. 105), over his
    /// general shuddhi.
    ///
    /// - **Stars**: Ashwini, Rohini, Punarvasu, Pushya, Uttara Phalguni,
    ///   Hasta, Chitra, Swati, Vishakha, Dhanishtha and Revati the best.
    /// - **Tithis**: the 4th, 9th and 14th and the Amavasya avoided.
    /// - **Varas**: Monday, Wednesday, Thursday and Friday good; Sunday
    ///   middling; Tuesday and Saturday inauspicious.
    ///
    /// ```
    /// use teistro_core::catalogue::Tithi;
    /// use teistro_muhurta::{ActivityRules, Grade};
    ///
    /// let clothes = ActivityRules::raman_new_clothes();
    /// clothes.check()?;
    /// assert_eq!(clothes.day.tithis.grade(&Tithi::Amavasya), Grade::Rejected);
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn raman_new_clothes() -> ActivityRules {
        let mut rejected = both(&[4, 9, 14]);
        rejected.push(Tithi::Amavasya);
        let day = over_shuddhi(
            Graded {
                best: Vec::new(),
                middling: Vec::new(),
                rejected,
                otherwise: Grade::Middling,
            },
            good(
                vec![
                    Nakshatra::Ashwini,
                    Nakshatra::Rohini,
                    Nakshatra::Punarvasu,
                    Nakshatra::Pushya,
                    Nakshatra::UttaraPhalguni,
                    Nakshatra::Hasta,
                    Nakshatra::Chitra,
                    Nakshatra::Swati,
                    Nakshatra::Vishakha,
                    Nakshatra::Dhanishtha,
                    Nakshatra::Revati,
                ],
                Grade::Middling,
            ),
            Graded {
                best: benefic_varas(),
                middling: vec![Vara::Ravivara],
                rejected: vec![Vara::Mangalavara, Vara::Shanivara],
                otherwise: Grade::Middling,
            },
        );
        ActivityRules {
            unjudged: vec![unjudged(
                "the Moon in good aspect to the Sun",
                "aspects are not graded",
            )],
            ..ActivityRules::general(day)
        }
    }

    /// Acquiring a vehicle in Raman's movable stars (*Muhurtha*, ch. IV,
    /// p. 26, after the Brihat Samhita): Shravana, Dhanishtha,
    /// Shatabhisha, Punarvasu and Swati, "auspicious for acquiring
    /// vehicles", over his general shuddhi. He gives the stars alone.
    ///
    /// ```
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_muhurta::{ActivityRules, Grade};
    ///
    /// let vehicle = ActivityRules::raman_vehicle();
    /// vehicle.check()?;
    /// assert_eq!(vehicle.day.nakshatras.grade(&Nakshatra::Swati), Grade::Best);
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn raman_vehicle() -> ActivityRules {
        let day = over_shuddhi(
            Graded::none(),
            good(
                vec![
                    Nakshatra::Shravana,
                    Nakshatra::Dhanishtha,
                    Nakshatra::Shatabhisha,
                    Nakshatra::Punarvasu,
                    Nakshatra::Swati,
                ],
                Grade::Middling,
            ),
            Graded::none(),
        );
        ActivityRules {
            unjudged: vec![unjudged(
                "the rest of a vehicle's election",
                "Raman gives the stars alone; the rest is his general shuddhi",
            )],
            ..ActivityRules::general(day)
        }
    }

    /// A day's rules alone, for an election whose source grades nothing
    /// else: any month and lagna, any hour, nothing heeded, unwanted or
    /// barred.
    fn general(day: DayRules) -> ActivityRules {
        ActivityRules {
            day,
            months: MonthRule::Any,
            lagnas: Graded::none(),
            hours: Hours::Any,
            shool: Vec::new(),
            padas: Vec::new(),
            heeds: Vec::new(),
            unwanted: Vec::new(),
            bars: Vec::new(),
            unjudged: Vec::new(),
            baseline: None,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::expect_used, reason = "tests fail by panicking")]
mod tests {
    use super::{ActivityRules, Hours, Unwanted};
    use crate::clause::ClauseKind;
    use crate::grade::Grade;
    use crate::instant::Sky;
    use crate::season::BlackoutKind;
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
        assert!(
            unwanted(&rules, &sky(&[])).is_empty(),
            "{:?}",
            unwanted(&rules, &sky(&[]))
        );
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
        assert!(
            unwanted(&rules, &sky(&[(Graha::Venus, 185.0)])).is_empty(),
            "{:?}",
            unwanted(&rules, &sky(&[(Graha::Venus, 185.0)]))
        );
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
            ActivityRules::raman_karnavedha(),
            ActivityRules::raman_chudakarana(),
            ActivityRules::raman_vidyarambha(),
            ActivityRules::raman_yatra(),
            ActivityRules::raman_trade_purchase(),
            ActivityRules::raman_selling(),
            ActivityRules::raman_new_clothes(),
            ActivityRules::raman_vehicle(),
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
        assert!(back.unwanted.is_empty(), "{:?}", back.unwanted);
    }

    #[test]
    fn rules_spelt_out_without_hours_keep_any_hour() {
        let mut json = serde_json::to_value(ActivityRules::raman_chudakarana()).expect("json");
        json.as_object_mut().expect("an object").remove("hours");
        let back: ActivityRules = serde_json::from_value(json).expect("reads");
        assert_eq!(back.hours, Hours::Any);
    }

    #[test]
    fn the_chapters_grade_what_they_name_over_the_shuddhi() {
        // The tonsure's dark half is ordinary where its bright half is good.
        let tonsure = ActivityRules::raman_chudakarana();
        assert_eq!(tonsure.day.tithis.grade(&Tithi::ShuklaDvitiya), Grade::Best);
        assert_eq!(
            tonsure.day.tithis.grade(&Tithi::KrishnaDvitiya),
            Grade::Middling
        );
        assert_eq!(
            tonsure.day.tithis.grade(&Tithi::ShuklaPratipada),
            Grade::Rejected
        );
        // A journey rejects its forbidden stars and Tuesday, and grades the
        // returning stars' first quarter apart.
        let journey = ActivityRules::raman_yatra();
        assert_eq!(journey.day.varas.grade(&Vara::Mangalavara), Grade::Rejected);
        assert_eq!(journey.day.nakshatras.grade(&Nakshatra::Mula), Grade::Best);
        assert_eq!(journey.padas.len(), 10);
        assert!(journey.heeds.contains(&BlackoutKind::Sankranti));
        // Education keeps the fixed signs out and Wednesday first.
        let learning = ActivityRules::raman_vidyarambha();
        assert_eq!(learning.lagnas.grade(&Rashi::Aquarius), Grade::Rejected);
        assert_eq!(learning.lagnas.grade(&Rashi::Pisces), Grade::Best);
        assert_eq!(
            learning.day.nakshatras.grade(&Nakshatra::Magha),
            Grade::Rejected
        );
        // Selling reads Saturday middling over the shuddhi's rejection.
        let selling = ActivityRules::raman_selling();
        assert_eq!(selling.day.varas.grade(&Vara::Shanivara), Grade::Middling);
        // Buying for business rejects Tuesday and keeps every lagna.
        let buying = ActivityRules::raman_trade_purchase();
        assert_eq!(buying.day.varas.grade(&Vara::Mangalavara), Grade::Rejected);
        assert!(
            Rashi::ALL
                .iter()
                .all(|r| buying.lagnas.grade(r) == Grade::Middling)
        );
        assert_eq!(buying.hours, Hours::Any);
    }
}
