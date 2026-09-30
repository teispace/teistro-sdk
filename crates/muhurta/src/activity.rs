//! An activity's rules: what one rite asks of the time, as data.
//!
//! The texts give each rite its own chapter — the months it may be held
//! in, the tithis, varas, stars and lagnas it grades, the placements it
//! cannot bear — over a general shuddhi every rite shares. So the rules
//! are a value: the day's grades already laid over the general ones
//! ([`Graded::over`]), the month rule, the lagnas, the padas, which of
//! the season's blackouts the rite heeds, which instant clauses bar it
//! outright, and what the source asks that the SDK does not yet judge,
//! said rather than dropped.
//!
//! [`ActivityRules::raman_marriage`] is the first such value. A consumer's
//! own tradition, or a rite nobody ships, is another value of this type,
//! not a fork.

use serde::{Deserialize, Serialize};
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Graha, Kaala, Karana, Masa, Nakshatra, Rashi, Tithi, Vara, Yoga};
use teistro_core::interval::Interval;
use teistro_panchanga::Panchanga;

use crate::baseline::BaselineEvent;
use crate::clause::{Clause, ClauseKey, ClauseKind};
use crate::day::{DayRules, reported};
use crate::grade::{Grade, Graded};
use crate::instant::Sky;
use crate::season::BlackoutKind;
use crate::tara::ChandraBala;
use crate::window::navamsa_of;

/// Which month a rite is keyed on (crux C161).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "reckoning", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MonthRule {
    /// No month is graded.
    Any,
    /// The amanta lunar month, graded; an adhika month is left to the
    /// season, which blacks it out.
    Lunar {
        /// The months.
        months: Graded<Masa>,
        /// Months allowed as middling while the Sun is in a sign, which
        /// the rule would otherwise reject: "some sages" allow Pausha with
        /// the Sun in Capricorn and Chaitra with it in Aries for a
        /// marriage (Raman, ch. IX).
        with_sun: Vec<MonthWithSun>,
    },
    /// The Sun's sign, graded: the solar month.
    Solar {
        /// The signs.
        signs: Graded<Rashi>,
    },
}

/// A month allowed while the Sun stands in a sign.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MonthWithSun {
    /// The month.
    pub masa: Masa,
    /// The Sun's sign.
    pub sun: Rashi,
}

/// A nakshatra's quarter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Pada {
    /// The star.
    pub nakshatra: Nakshatra,
    /// The quarter, 1 to 4.
    pub pada: u8,
}

/// A clause that bars a rite outright: every clause of a kind, or one
/// clause exactly.
///
/// In JSON a bar is either a clause's key, `"KAALA"`, which bars all
/// three kaalas, or a clause, `{"clause": "KAALA", "kaala": "RAHU_KAALA"}`,
/// which bars Rahu kaala alone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Bar {
    /// Every clause of a kind.
    Key(ClauseKey),
    /// One clause, matched whole.
    Clause(ClauseKind),
}

impl Bar {
    /// Whether a clause is one this bar names.
    #[must_use]
    pub fn names(&self, kind: &ClauseKind) -> bool {
        match self {
            Bar::Key(key) => kind.key() == *key,
            Bar::Clause(clause) => clause == kind,
        }
    }
}

impl From<ClauseKey> for Bar {
    fn from(key: ClauseKey) -> Bar {
        Bar::Key(key)
    }
}

impl From<ClauseKind> for Bar {
    fn from(kind: ClauseKind) -> Bar {
        Bar::Clause(kind)
    }
}

/// Something the source asks of the time that the SDK does not judge yet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Unjudged {
    /// What is asked.
    pub what: String,
    /// Why it is not judged, and where it is recorded.
    pub why: String,
}

/// What one activity asks of the time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ActivityRules {
    /// The day's limbs and vara, the rite's grades over the general ones.
    pub day: DayRules,
    /// The months.
    pub months: MonthRule,
    /// The lagnas.
    pub lagnas: Graded<Rashi>,
    /// The quarters of a star rejected though the star is good.
    pub padas: Vec<Pada>,
    /// The season's blackouts the rite is not held in.
    pub heeds: Vec<BlackoutKind>,
    /// The clauses that bar the rite outright rather than weigh against
    /// it.
    pub bars: Vec<Bar>,
    /// What the source asks that is not judged, said rather than dropped.
    pub unjudged: Vec<Unjudged>,
    /// The inputs the baseline engine's weights take for the rite, which
    /// [`crate::Ranking::Baseline`] needs and no other ranking reads.
    pub baseline: Option<BaselineEvent>,
}

impl ActivityRules {
    /// Raman's marriage (*Muhurtha*, ch. IX), over his general shuddhi
    /// ([`DayRules::raman`]), heeding what Muhurta Chintamani forbids a
    /// marriage in (ch. I, vv. 46–47, with its commentary): Jupiter or
    /// Venus set, and an adhika month; and the sankranti, the second of
    /// the twenty-one Mahadoshas.
    ///
    /// - **Months** (lunar, crux C161): Magha, Phalguna, Vaishakha and
    ///   Jyeshtha good; Kartika and Margashirsha ordinary; the rest
    ///   rejected. The sages' Pausha and Chaitra are not taken; a caller
    ///   adds them to `with_sun`.
    /// - **Tithis**: the bright 2nd, 3rd, 5th, 7th, 10th, 11th and 13th
    ///   good; the dark 11th to the new Moon, the riktas (4th, 9th,
    ///   14th), the 6th, 8th and 12th rejected.
    /// - **Varas**: Monday, Wednesday, Thursday and Friday good; Sunday and
    ///   Saturday middling; Tuesday rejected.
    /// - **Stars**: Rohini, Mrigashira, Magha, Uttara Phalguni, Hasta,
    ///   Swati, Anuradha, Mula, Uttara Ashadha, Uttara Bhadrapada and Revati
    ///   good, "constellations not mentioned here are unsuitable"; and the
    ///   first quarter of Magha and Mula and the last of Revati rejected.
    /// - **Yogas**: as printed (crux C163) — Vyatipata, Dhruva, Ganda,
    ///   Vajra, Shoola, Vishkambha, Atiganda, Vyaghata, Parigha — with
    ///   Mrityu unjudged.
    /// - **Lagnas**: Gemini, Virgo and Libra good; Taurus, Cancer, Leo,
    ///   Sagittarius and Aquarius middling; the rest rejected.
    /// - **Bars**, "the most important considerations": the 7th occupied,
    ///   Mars in the 8th, Venus in the 6th, kartari, a malefic in the
    ///   lagna, and the Moon joined.
    #[must_use]
    pub fn raman_marriage() -> ActivityRules {
        let day = raman_marriage_day();
        ActivityRules {
            day,
            months: MonthRule::Lunar {
                months: Graded {
                    best: vec![Masa::Magha, Masa::Phalguna, Masa::Vaishakha, Masa::Jyeshtha],
                    middling: vec![Masa::Kartika, Masa::Margashirsha],
                    rejected: Vec::new(),
                    otherwise: Grade::Rejected,
                },
                with_sun: Vec::new(),
            },
            lagnas: Graded {
                best: vec![Rashi::Gemini, Rashi::Virgo, Rashi::Libra],
                middling: vec![
                    Rashi::Taurus,
                    Rashi::Cancer,
                    Rashi::Leo,
                    Rashi::Sagittarius,
                    Rashi::Aquarius,
                ],
                rejected: Vec::new(),
                otherwise: Grade::Rejected,
            },
            padas: vec![
                Pada { nakshatra: Nakshatra::Magha, pada: 1 },
                Pada { nakshatra: Nakshatra::Mula, pada: 1 },
                Pada { nakshatra: Nakshatra::Revati, pada: 4 },
            ],
            heeds: vec![
                BlackoutKind::AdhikaMasa,
                BlackoutKind::Sankranti,
                BlackoutKind::GuruAsta,
                BlackoutKind::ShukraAsta,
            ],
            bars: [
                ClauseKey::SeventhOccupied,
                ClauseKey::MarsInEighth,
                ClauseKey::VenusInSixth,
                ClauseKey::Kartari,
                ClauseKey::MaleficInLagna,
                ClauseKey::MoonJoined,
            ]
            .map(Bar::from)
            .into(),
            unjudged: vec![
                Unjudged {
                    what: "the Mrityu yoga".into(),
                    why: "an Anandadi yoga, which the SDK does not compute (crux C163)".into(),
                },
                Unjudged {
                    what: "Jupiter's and Venus's infancy and old age (bala, vriddha)".into(),
                    why: "the days either side of an asta the texts count differ by body and side, and are not yet sourced (Muhurta Chintamani ch. I, vv. 46–47)".into(),
                },
                Unjudged {
                    what: "the kshaya month".into(),
                    why: "the season does not yet mark it (Muhurta Chintamani ch. I, vv. 46–47)".into(),
                },
            ],
            baseline: None,
        }
    }

    /// The baseline engine's marriage: its gates as rules, and the inputs
    /// its weights take (`muhurta.md` §4.5). Meant for
    /// [`crate::Ranking::Baseline`], under which the roadmap's regression
    /// is stated.
    ///
    /// - **Gates**, as bars: the Sun in Aries, Taurus, Gemini, Scorpio,
    ///   Capricorn or Aquarius (the solar month, crux C161); the Moon in
    ///   one of the eleven stars; Rahu kaala, the one kaala the engine
    ///   avoids by default.
    /// - **Heeds**: Chaturmas, the adhika month, Kharmas, Pitru paksha and
    ///   Guru and Shukra asta.
    /// - Nothing else is graded: the engine weighs the rest, and a
    ///   weight is the ranking's.
    #[must_use]
    pub fn baseline_marriage() -> ActivityRules {
        let event = BaselineEvent::marriage();
        ActivityRules {
            day: DayRules {
                tithis: Graded::none(),
                nakshatras: Graded::admitting(event.stars.clone()),
                yogas: Graded::none(),
                karanas: Graded::none(),
                varas: Graded::none(),
                chandrabala: ChandraBala::raman(),
            },
            months: MonthRule::Solar {
                signs: Graded::admitting(vec![
                    Rashi::Aries,
                    Rashi::Taurus,
                    Rashi::Gemini,
                    Rashi::Scorpio,
                    Rashi::Capricorn,
                    Rashi::Aquarius,
                ]),
            },
            lagnas: Graded::none(),
            padas: Vec::new(),
            heeds: vec![
                BlackoutKind::Chaturmas,
                BlackoutKind::AdhikaMasa,
                BlackoutKind::Kharmas,
                BlackoutKind::PitruPaksha,
                BlackoutKind::GuruAsta,
                BlackoutKind::ShukraAsta,
            ],
            bars: vec![
                Bar::Key(ClauseKey::SolarMonth),
                Bar::Key(ClauseKey::Nakshatra),
                Bar::Clause(ClauseKind::Kaala {
                    kaala: Kaala::RahuKaala,
                }),
            ],
            unjudged: Vec::new(),
            baseline: Some(event),
        }
    }

    /// The day's month clauses: the lunar month's grade over the day, or
    /// the Sun's sign's over each span of it, best or rejected.
    ///
    /// An adhika month is left alone: the season blacks it out for a rite
    /// that heeds it, and its name is not the month the rule grades.
    #[must_use]
    pub fn month_clauses(&self, day: &Panchanga) -> Vec<Clause> {
        let spans = day.sun.signs.iter().filter_map(|span| {
            span.inside
                .clipped_to(day.window)
                .map(|at| (span.member, at))
        });
        match &self.months {
            MonthRule::Any => Vec::new(),
            MonthRule::Lunar { months, with_sun } => {
                if day.month.kind == MonthKind::Adhika {
                    return Vec::new();
                }
                let masa = day.month.amanta;
                spans
                    .filter_map(|(sun, at)| {
                        let allowed = with_sun.iter().any(|w| w.masa == masa && w.sun == sun);
                        let grade = if allowed {
                            Grade::Middling
                        } else {
                            months.grade(&masa)
                        };
                        reported(grade).map(|grade| Clause {
                            kind: ClauseKind::Month { masa, grade },
                            at,
                        })
                    })
                    .collect()
            }
            MonthRule::Solar { signs } => spans
                .filter_map(|(sign, at)| {
                    reported(signs.grade(&sign)).map(|grade| Clause {
                        kind: ClauseKind::SolarMonth { sign, grade },
                        at,
                    })
                })
                .collect(),
        }
    }

    /// The instant's clauses the rite's own grades give: the lagna's grade
    /// and a rejected pada of the Moon's star.
    #[must_use]
    pub fn instant_clauses(&self, sky: &Sky, at: Interval) -> Vec<Clause> {
        let mut found = Vec::new();
        let sign = sky.lagna();
        if let Some(grade) = reported(self.lagnas.grade(&sign)) {
            found.push(Clause {
                kind: ClauseKind::Lagna { sign, grade },
                at,
            });
        }
        let moon = navamsa_of(sky.longitude(Graha::Moon));
        let pada = Pada {
            nakshatra: Nakshatra::from_id(u16::from(moon / 4)).unwrap_or(Nakshatra::Ashwini),
            pada: moon % 4 + 1,
        };
        if self.padas.contains(&pada) {
            found.push(Clause {
                kind: ClauseKind::Pada { pada },
                at,
            });
        }
        found
    }

    /// Whether a clause bars the rite outright.
    #[must_use]
    pub fn bars(&self, clause: &Clause) -> bool {
        self.bars.iter().any(|bar| bar.names(&clause.kind))
    }
}

/// Raman's marriage grades of the day's limbs and vara (ch. IX), laid
/// over his general shuddhi.
fn raman_marriage_day() -> DayRules {
    let general = DayRules::raman();
    let bright = |n: u16| Tithi::from_id(n - 1);
    let dark = |n: u16| Tithi::from_id(n + 14);
    let per_paksha = |n: u16| [bright(n), dark(n)];
    let tithis = Graded {
        best: [2, 3, 5, 7, 10, 11, 13]
            .into_iter()
            .filter_map(bright)
            .collect(),
        middling: Vec::new(),
        rejected: (11..=15)
            .filter_map(dark)
            .chain(
                [4, 9, 14, 6, 8, 12]
                    .into_iter()
                    .flat_map(per_paksha)
                    .flatten(),
            )
            .collect(),
        otherwise: Grade::Middling,
    };
    DayRules {
        tithis: tithis.over(&general.tithis),
        nakshatras: Graded {
            best: vec![
                Nakshatra::Rohini,
                Nakshatra::Mrigashira,
                Nakshatra::Magha,
                Nakshatra::UttaraPhalguni,
                Nakshatra::Hasta,
                Nakshatra::Swati,
                Nakshatra::Anuradha,
                Nakshatra::Mula,
                Nakshatra::UttaraAshadha,
                Nakshatra::UttaraBhadrapada,
                Nakshatra::Revati,
            ],
            middling: Vec::new(),
            rejected: Vec::new(),
            otherwise: Grade::Rejected,
        }
        .over(&general.nakshatras),
        yogas: Graded::rejecting(vec![
            Yoga::Vyatipata,
            Yoga::Dhruva,
            Yoga::Ganda,
            Yoga::Vajra,
            Yoga::Shoola,
            Yoga::Vishkambha,
            Yoga::Atiganda,
            Yoga::Vyaghata,
            Yoga::Parigha,
        ])
        .over(&general.yogas),
        karanas: Graded::rejecting(vec![Karana::Vishti]).over(&general.karanas),
        varas: Graded {
            best: vec![
                Vara::Somavara,
                Vara::Budhavara,
                Vara::Guruvara,
                Vara::Shukravara,
            ],
            middling: vec![Vara::Ravivara, Vara::Shanivara],
            rejected: vec![Vara::Mangalavara],
            otherwise: Grade::Middling,
        }
        .over(&general.varas),
        chandrabala: ChandraBala::raman(),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "tests fail by panicking")]
mod tests {
    use super::{ActivityRules, Bar, Pada};
    use crate::clause::{ClauseKey, ClauseKind};
    use crate::grade::Grade;
    use crate::instant::Sky;
    use teistro_core::catalogue::{Nakshatra, Rashi, Tithi, Vara, Yoga};
    use teistro_core::interval::Interval;

    #[test]
    fn the_marriage_chapter_wins_where_it_speaks_and_the_shuddhi_where_it_does_not() {
        let rules = ActivityRules::raman_marriage();
        let day = &rules.day;
        // Saturday: the chapter's middling over the shuddhi's rejection.
        assert_eq!(day.varas.grade(&Vara::Shanivara), Grade::Middling);
        assert_eq!(day.varas.grade(&Vara::Mangalavara), Grade::Rejected);
        assert_eq!(day.varas.grade(&Vara::Guruvara), Grade::Best);
        // Vaidhriti: the chapter is silent and the shuddhi rejects it.
        assert_eq!(day.yogas.grade(&Yoga::Vaidhriti), Grade::Rejected);
        assert_eq!(day.yogas.grade(&Yoga::Dhruva), Grade::Rejected);
        assert_eq!(day.yogas.grade(&Yoga::Siddhi), Grade::Middling);
        // The bright 13th good; the Purnima rejected by the shuddhi; the
        // dark 11th by the chapter; the bright 1st neither.
        assert_eq!(day.tithis.grade(&Tithi::ShuklaTrayodashi), Grade::Best);
        assert_eq!(day.tithis.grade(&Tithi::Purnima), Grade::Rejected);
        assert_eq!(day.tithis.grade(&Tithi::KrishnaEkadashi), Grade::Rejected);
        assert_eq!(day.tithis.grade(&Tithi::ShuklaPratipada), Grade::Middling);
        // A star not named is unsuitable.
        assert_eq!(day.nakshatras.grade(&Nakshatra::Rohini), Grade::Best);
        assert_eq!(day.nakshatras.grade(&Nakshatra::Ashwini), Grade::Rejected);
        let best = (0..30)
            .filter_map(Tithi::from_id)
            .filter(|t| day.tithis.grade(t) == Grade::Best)
            .count();
        assert_eq!(best, 7);
    }

    #[test]
    fn the_lagna_and_the_moons_quarter_are_judged_at_the_instant() {
        let rules = ActivityRules::raman_marriage();
        let at = Interval::literal(2_460_000.5, 2_460_000.6);
        // Lagna 65° (Gemini), the Moon at 241° (Mula's first quarter).
        let mut grahas = [0.0; 9];
        grahas[1] = 241.0;
        let sky = Sky {
            lagna_deg: 65.0,
            grahas,
            speeds: [1.0; 9],
        };
        let found = rules.instant_clauses(&sky, at);
        let kinds: Vec<&ClauseKind> = found.iter().map(|c| &c.kind).collect();
        assert_eq!(
            kinds,
            [
                &ClauseKind::Lagna {
                    sign: Rashi::Gemini,
                    grade: Grade::Best
                },
                &ClauseKind::Pada {
                    pada: Pada {
                        nakshatra: Nakshatra::Mula,
                        pada: 1
                    }
                },
            ]
        );
        // Aries rising is rejected, Cancer middling and not reported; the
        // Moon in Mula's second quarter is not.
        grahas[1] = 244.0;
        let aries = rules.instant_clauses(
            &Sky {
                lagna_deg: 5.0,
                grahas,
                speeds: [1.0; 9],
            },
            at,
        );
        assert_eq!(aries.len(), 1);
        assert!(aries.iter().all(|c| !c.favourable()));
        assert!(
            rules
                .instant_clauses(
                    &Sky {
                        lagna_deg: 95.0,
                        grahas,
                        speeds: [1.0; 9],
                    },
                    at
                )
                .is_empty()
        );
    }

    #[test]
    fn a_bar_names_a_kind_or_one_clause_and_reads_either_spelling() {
        use teistro_core::catalogue::Kaala;
        let rahu = ClauseKind::Kaala {
            kaala: Kaala::RahuKaala,
        };
        let gulika = ClauseKind::Kaala {
            kaala: Kaala::GulikaKaala,
        };
        let every: Bar = serde_json::from_str(r#""KAALA""#).expect("a key");
        let one: Bar =
            serde_json::from_str(r#"{"clause":"KAALA","kaala":"RAHU_KAALA"}"#).expect("a clause");
        assert_eq!(every, Bar::Key(ClauseKey::Kaala));
        assert_eq!(one, Bar::Clause(rahu.clone()));
        assert!(every.names(&rahu) && every.names(&gulika));
        assert!(one.names(&rahu) && !one.names(&gulika));
        // Each writes back as it was read.
        assert_eq!(serde_json::to_string(&every).expect("json"), r#""KAALA""#);
        assert_eq!(
            serde_json::to_string(&one).expect("json"),
            r#"{"clause":"KAALA","kaala":"RAHU_KAALA"}"#
        );
    }

    #[test]
    fn the_baseline_marriage_gates_by_bars_and_grades_nothing_else() {
        use teistro_core::catalogue::Kaala;
        let rules = ActivityRules::baseline_marriage();
        let at = Interval::literal(2_460_000.5, 2_460_000.6);
        let clause = |kind| crate::clause::Clause { kind, at };
        // Rahu kaala bars, Gulika does not; the solar month and the star
        // bar wherever the rules reject them.
        assert!(rules.bars(&clause(ClauseKind::Kaala {
            kaala: Kaala::RahuKaala
        })));
        assert!(!rules.bars(&clause(ClauseKind::Kaala {
            kaala: Kaala::GulikaKaala
        })));
        assert_eq!(
            rules.day.nakshatras.grade(&Nakshatra::Ashwini),
            Grade::Rejected
        );
        assert_eq!(
            rules.day.nakshatras.grade(&Nakshatra::Rohini),
            Grade::Middling
        );
        assert_eq!(rules.day.varas.grade(&Vara::Mangalavara), Grade::Middling);
        assert_eq!(rules.heeds.len(), 6);
        assert!(rules.baseline.is_some());
        assert!(ActivityRules::raman_marriage().baseline.is_none());
    }

    #[test]
    fn the_six_considerations_bar_and_the_rest_weigh() {
        let rules = ActivityRules::raman_marriage();
        let at = Interval::literal(2_460_000.5, 2_460_000.6);
        let clause = |kind| crate::clause::Clause { kind, at };
        assert!(rules.bars(&clause(ClauseKind::MarsInEighth {})));
        assert!(!rules.bars(&clause(ClauseKind::AshtamaLagna {})));
        assert_eq!(rules.bars.len(), 6);
        assert!(rules.bars.contains(&Bar::Key(ClauseKey::MoonJoined)));
    }

    #[test]
    fn the_rules_are_data_that_round_trip() {
        let rules = ActivityRules::raman_marriage();
        let json = serde_json::to_string(&rules).expect("the rules serialise");
        let back: ActivityRules = serde_json::from_str(&json).expect("and read back");
        assert_eq!(back, rules);
        assert!(json.contains(r#""reckoning":"LUNAR""#), "{json}");
        assert!(json.contains(r#""bars":["SEVENTH_OCCUPIED""#), "{json}");
    }
}
