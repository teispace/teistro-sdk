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
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_panchanga::Panchanga;

use crate::baseline::BaselineEvent;
use crate::clause::{Clause, ClauseKey, ClauseKind};
use crate::day::{DayRules, reported};
use crate::grade::{Grade, Graded};
use crate::instant::{GRAHAS, Sky};
use crate::season::BlackoutKind;
use crate::tara::ChandraBala;
use crate::window::navamsa_of;

/// Which month a rite is keyed on (crux C161).
///
/// ```
/// use teistro_muhurta::{ActivityRules, MonthRule};
///
/// // Raman keys a marriage on the lunar month, the baseline on the Sun's sign.
/// assert!(matches!(ActivityRules::raman_marriage().months, MonthRule::Lunar { .. }));
/// assert!(matches!(ActivityRules::baseline_marriage().months, MonthRule::Solar { .. }));
/// assert_eq!(serde_json::to_value(MonthRule::Any)?, serde_json::json!({"reckoning": "ANY"}));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(
    tag = "reckoning",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
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
///
/// ```
/// use teistro_core::catalogue::{Masa, Rashi};
/// use teistro_muhurta::{ActivityRules, MonthRule};
/// use teistro_muhurta::activity::MonthWithSun;
///
/// // The sages' Pausha with the Sun in Capricorn, added to Raman's marriage.
/// let mut rules = ActivityRules::raman_marriage();
/// if let MonthRule::Lunar { with_sun, .. } = &mut rules.months {
///     with_sun.push(MonthWithSun { masa: Masa::Pausha, sun: Rashi::Capricorn });
/// }
/// assert!(matches!(&rules.months, MonthRule::Lunar { with_sun, .. } if with_sun.len() == 1));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MonthWithSun {
    /// The month.
    pub masa: Masa,
    /// The Sun's sign.
    pub sun: Rashi,
}

/// A nakshatra's quarter.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_muhurta::ActivityRules;
/// use teistro_muhurta::activity::Pada;
///
/// // Raman rejects the first quarter of Magha though Magha is good.
/// assert!(ActivityRules::raman_marriage().padas.contains(&Pada { nakshatra: Nakshatra::Magha, pada: 1 }));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "NakshatraPada"))]
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
///
/// ```
/// use teistro_core::catalogue::Kaala;
/// use teistro_muhurta::{Bar, ClauseKind};
/// use teistro_muhurta::clause::ClauseKey;
///
/// // A key bars every kaala; a clause bars Rahu kaala alone.
/// assert_eq!(serde_json::from_str::<Bar>(r#""KAALA""#)?, Bar::Key(ClauseKey::Kaala));
/// let rahu: Bar = serde_json::from_str(r#"{"clause": "KAALA", "kaala": "RAHU_KAALA"}"#)?;
/// assert_eq!(rahu, Bar::Clause(ClauseKind::Kaala { kaala: Kaala::RahuKaala }));
/// # Ok::<(), serde_json::Error>(())
/// ```
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
    ///
    /// ```
    /// use teistro_core::catalogue::Kaala;
    /// use teistro_muhurta::{Bar, ClauseKind};
    /// use teistro_muhurta::clause::ClauseKey;
    ///
    /// let gulika = ClauseKind::Kaala { kaala: Kaala::GulikaKaala };
    /// assert!(Bar::Key(ClauseKey::Kaala).names(&gulika));
    /// assert!(!Bar::Clause(ClauseKind::Kaala { kaala: Kaala::RahuKaala }).names(&gulika));
    /// ```
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

/// The part of the day a rite is held in: Raman's "never during night"
/// for the ear boring, "always in the forenoon" for the tonsure.
///
/// Noon is the middle of the daylight, halfway from sunrise to sunset. A
/// polar day's daylight is its policy's, and a day with none holds no
/// window under `DAYLIGHT` or `FORENOON`.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_core::quantity::JulianDay;
/// use teistro_muhurta::Hours;
///
/// // Sunrise a quarter into the day, sunset three quarters in.
/// let (day, sunrise, sunset) = (Interval::literal(0.0, 1.0), JulianDay::literal(0.25), JulianDay::literal(0.75));
/// assert_eq!(Hours::Any.within(day, sunrise, sunset), Some(day));
/// assert_eq!(Hours::Daylight.within(day, sunrise, sunset), Some(Interval::literal(0.25, 0.75)));
/// assert_eq!(Hours::Forenoon.within(day, sunrise, sunset), Some(Interval::literal(0.25, 0.5)));
/// assert_eq!(serde_json::to_value(Hours::Forenoon)?, serde_json::json!("FORENOON"));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Hours {
    /// Day and night alike.
    #[default]
    Any,
    /// Sunrise to sunset.
    Daylight,
    /// Sunrise to noon.
    Forenoon,
}

impl Hours {
    /// The part of `window` these hours keep, given the day's sunrise
    /// and sunset, or `None` where they keep nothing of it.
    ///
    /// ```
    /// use teistro_core::interval::Interval;
    /// use teistro_core::quantity::{JulianDay, Utc};
    /// use teistro_muhurta::Hours;
    ///
    /// // A day from sunrise at 0.25 to the next at 1.25, the Sun setting at 0.75.
    /// let day = Interval::literal(0.25, 1.25);
    /// let (rise, set) = (JulianDay::<Utc>::literal(0.25), JulianDay::<Utc>::literal(0.75));
    /// assert_eq!(Hours::Any.within(day, rise, set), Some(day));
    /// assert_eq!(Hours::Daylight.within(day, rise, set), Some(Interval::literal(0.25, 0.75)));
    /// assert_eq!(Hours::Forenoon.within(day, rise, set), Some(Interval::literal(0.25, 0.5)));
    /// // A window wholly after noon keeps no forenoon.
    /// assert_eq!(Hours::Forenoon.within(Interval::literal(0.6, 0.7), rise, set), None);
    /// ```
    #[must_use]
    pub fn within(
        self,
        window: Interval,
        sunrise: JulianDay<Utc>,
        sunset: JulianDay<Utc>,
    ) -> Option<Interval> {
        let to = match self {
            Hours::Any => return Some(window),
            Hours::Daylight => sunset.get(),
            Hours::Forenoon => f64::midpoint(sunrise.get(), sunset.get()),
        };
        (sunrise.get() < to)
            .then(|| Interval::literal(sunrise.get(), to))
            .and_then(|hours| hours.clipped_to(window))
            .filter(|kept| !kept.is_empty())
    }
}

/// The ghati of a weekday past which a journey towards its disha shool
/// may begin: Raman's "provided the journey is timed to begin beyond 22
/// ghaties on Thursday" (*Muhurtha*, ch. XIV, p. 157), counted from
/// sunrise under the day's own ghati reckoning.
///
/// ```
/// use teistro_core::catalogue::Vara;
/// use teistro_muhurta::{ActivityRules, Shool};
///
/// let journey = ActivityRules::raman_yatra();
/// assert!(journey.shool.contains(&Shool { vara: Vara::Guruvara, after_ghatis: 22 }));
/// assert_eq!(serde_json::to_value(Shool { vara: Vara::Guruvara, after_ghatis: 22 })?,
///     serde_json::json!({"vara": "GURUVARA", "afterGhatis": 22}));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Shool {
    /// The weekday.
    pub vara: Vara,
    /// The ghatis from sunrise, 0 to 59, before which the journey does not
    /// begin.
    pub after_ghatis: u8,
}

/// Grahas a rite wants out of houses: "the 8th house should be
/// unoccupied" is every graha out of the 8th, "Mars and Saturn should be
/// avoided in the 5th" is two grahas out of one house.
///
/// Houses are counted by sign from the lagna's, 1 to 12, as the texts
/// count them. A graha standing in a listed house is reported as
/// [`ClauseKind::UnwantedPlacement`], one clause a house, and counts
/// against the time; an entry that `bars` bars the rite outright (crux
/// C205: the texts' "must be unoccupied" against their "should").
///
/// ```
/// # use teistro_core::interval::Interval;
/// # use teistro_muhurta::instant::Sky;
/// #
/// # // Libra rising; Mars in Taurus (the 8th), Venus in Pisces, Jupiter in
/// # // Libra, the Sun in Aquarius with Mercury, the Moon and Saturn in
/// # // Capricorn, the nodes opposite.
/// # let grahas = [320.0, 285.0, 45.0, 300.0, 190.0, 345.0, 285.0, 285.0, 105.0];
/// # let sky = Sky { lagna_deg: 185.0, grahas, speeds: [1.0; 9] };
/// # let at = Interval::literal(2_460_000.5, 2_460_000.51);
/// use teistro_core::catalogue::Graha;
/// use teistro_muhurta::{ActivityRules, ClauseKind, Unwanted};
///
/// // "Mars and Saturn should be avoided in the 8th": Mars is there.
/// let mut rules = ActivityRules::raman_marriage();
/// rules.unwanted.push(Unwanted::of(vec![Graha::Mars, Graha::Saturn], vec![8]));
/// let found = rules.instant_clauses(&sky, at);
/// assert!(found.iter().any(|c| c.kind == ClauseKind::UnwantedPlacement { house: 8, by: vec![Graha::Mars] }));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "MuhurtaUnwanted"))]
pub struct Unwanted {
    /// The grahas.
    pub grahas: Vec<Graha>,
    /// The houses they are wanted out of, 1 to 12.
    pub houses: Vec<u8>,
    /// Whether a graha standing there bars the rite rather than weighs
    /// against it. Unbarring where the rules spelt out leave it out.
    #[serde(default)]
    pub bars: bool,
}

impl Unwanted {
    /// Grahas out of houses, weighing against the time.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_muhurta::Unwanted;
    ///
    /// let unwanted = Unwanted::of(vec![Graha::Saturn], vec![5]);
    /// assert!(!unwanted.bars);
    /// ```
    #[must_use]
    pub fn of(grahas: Vec<Graha>, houses: Vec<u8>) -> Unwanted {
        Unwanted {
            grahas,
            houses,
            bars: false,
        }
    }

    /// Every graha out of the houses: the houses should be unoccupied.
    ///
    /// ```
    /// use teistro_muhurta::Unwanted;
    ///
    /// // The eighth unoccupied: all nine grahas out of it.
    /// assert_eq!(Unwanted::vacant(vec![8]).grahas.len(), 9);
    /// ```
    #[must_use]
    pub fn vacant(houses: Vec<u8>) -> Unwanted {
        Unwanted::of(GRAHAS.into(), houses)
    }

    /// The same, barring the rite rather than weighing against it.
    ///
    /// ```
    /// # use teistro_core::interval::Interval;
    /// # use teistro_muhurta::instant::Sky;
    /// #
    /// # // Libra rising; Mars in Taurus (the 8th), Venus in Pisces, Jupiter in
    /// # // Libra, the Sun in Aquarius with Mercury, the Moon and Saturn in
    /// # // Capricorn, the nodes opposite.
    /// # let grahas = [320.0, 285.0, 45.0, 300.0, 190.0, 345.0, 285.0, 285.0, 105.0];
    /// # let sky = Sky { lagna_deg: 185.0, grahas, speeds: [1.0; 9] };
    /// # let at = Interval::literal(2_460_000.5, 2_460_000.51);
    /// use teistro_muhurta::{ActivityRules, Unwanted};
    ///
    /// // The eighth "must be unoccupied": Mars there bars the rite.
    /// let mut rules = ActivityRules::raman_marriage();
    /// rules.unwanted.push(Unwanted::vacant(vec![8]).barring());
    /// let found = rules.instant_clauses(&sky, at);
    /// assert!(found.iter().any(|c| rules.bars(c)));
    /// ```
    #[must_use]
    pub fn barring(self) -> Unwanted {
        Unwanted { bars: true, ..self }
    }

    /// Whether the entry names a graha of `by` in `house`.
    fn names(&self, house: u8, by: &[Graha]) -> bool {
        self.houses.contains(&house) && by.iter().any(|g| self.grahas.contains(g))
    }
}

/// Something the source asks of the time that the SDK does not judge yet.
///
/// ```
/// use teistro_muhurta::ActivityRules;
///
/// // What Raman asks and the SDK does not yet judge is said, not dropped.
/// let rules = ActivityRules::raman_marriage();
/// assert!(rules.unjudged.iter().any(|u| u.what.contains("Mrityu")));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "MuhurtaUnjudged"))]
pub struct Unjudged {
    /// What is asked.
    pub what: String,
    /// Why it is not judged, and where it is recorded.
    pub why: String,
}

/// What one activity asks of the time.
///
/// ```
/// use teistro_muhurta::ActivityRules;
/// use teistro_muhurta::season::BlackoutKind;
///
/// let rules = ActivityRules::raman_marriage();
/// rules.check()?;
/// assert!(rules.heeds.contains(&BlackoutKind::AdhikaMasa));
/// assert!(rules.baseline.is_none());
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ActivityRules {
    /// The day's limbs and vara, the rite's grades over the general ones.
    pub day: DayRules,
    /// The months.
    pub months: MonthRule,
    /// The lagnas.
    pub lagnas: Graded<Rashi>,
    /// The part of the day the rite is held in.
    #[serde(default)]
    pub hours: Hours,
    /// For a journey asked a direction, the ghati each weekday's disha
    /// shool is lifted at. Empty, the direction is not judged; a weekday
    /// missing from a list that is not shuts its days to the direction.
    #[serde(default)]
    pub shool: Vec<Shool>,
    /// The quarters of a star rejected though the star is good.
    pub padas: Vec<Pada>,
    /// The season's blackouts the rite is not held in.
    pub heeds: Vec<BlackoutKind>,
    /// The grahas the rite wants out of houses. Empty where the rules
    /// spelt out leave it out.
    #[serde(default)]
    pub unwanted: Vec<Unwanted>,
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
    /// Venus set, and an adhika month; the sankranti, the second of the
    /// twenty-one Mahadoshas; and the sixteenth, grahanotpatha, the star of
    /// an eclipse the place saw, which a marriage avoids for six months.
    /// The months a kshaya year closes are *Dharmasindhu*'s (p. 3, crux
    /// C179): the kshaya month and the adhika after it, while the samsarpa
    /// adhika before it is fit. A caller who closes that one too adds
    /// [`BlackoutKind::Samsarpa`] to `heeds`.
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
    ///
    /// ```
    /// use teistro_core::catalogue::{Rashi, Vara};
    /// use teistro_muhurta::{ActivityRules, Grade};
    ///
    /// let rules = ActivityRules::raman_marriage();
    /// assert_eq!(rules.day.varas.grade(&Vara::Mangalavara), Grade::Rejected);
    /// assert_eq!(rules.lagnas.grade(&Rashi::Gemini), Grade::Best);
    /// assert_eq!(rules.lagnas.grade(&Rashi::Aries), Grade::Rejected);
    /// ```
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
            hours: Hours::Any,
            shool: Vec::new(),
            padas: vec![
                Pada { nakshatra: Nakshatra::Magha, pada: 1 },
                Pada { nakshatra: Nakshatra::Mula, pada: 1 },
                Pada { nakshatra: Nakshatra::Revati, pada: 4 },
            ],
            heeds: vec![
                BlackoutKind::AdhikaMasa,
                BlackoutKind::KshayaMasa,
                BlackoutKind::Sankranti,
                BlackoutKind::GuruAsta,
                BlackoutKind::ShukraAsta,
                BlackoutKind::EclipseStar,
            ],
            unwanted: Vec::new(),
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
    /// - **Heeds**: Chaturmas, every adhika month (the samsarpa too),
    ///   Kharmas, Pitru paksha and Guru and Shukra asta.
    /// - Nothing else is graded: the engine weighs the rest, and a
    ///   weight is the ranking's.
    ///
    /// ```
    /// use teistro_muhurta::ActivityRules;
    /// use teistro_muhurta::season::BlackoutKind;
    ///
    /// let rules = ActivityRules::baseline_marriage();
    /// assert!(rules.baseline.is_some());
    /// assert!(rules.heeds.contains(&BlackoutKind::Chaturmas));
    /// ```
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
            hours: Hours::Any,
            shool: Vec::new(),
            padas: Vec::new(),
            heeds: vec![
                BlackoutKind::Chaturmas,
                BlackoutKind::AdhikaMasa,
                BlackoutKind::Samsarpa,
                BlackoutKind::Kharmas,
                BlackoutKind::PitruPaksha,
                BlackoutKind::GuruAsta,
                BlackoutKind::ShukraAsta,
            ],
            unwanted: Vec::new(),
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
    ///
    /// ```
    /// # use teistro_astro::Completion;
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_astro::visibility::{Criterion, Heliacal};
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_chart::foundation::Founder;
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_ephemeris_builtin::provider::Builtin;
    /// # use teistro_muhurta::sources::Over;
    /// # use teistro_muhurta::{ProviderSources, Sources};
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::Horizon;
    /// #
    /// # // Kathmandu, on the Nepal clock, over the built-in ephemeris.
    /// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
    /// # let provider = Builtin::new();
    /// # let resolved = Profile::shipped(DEFAULT_PROFILE)
    /// #     .ok_or("a shipped profile")?
    /// #     .resolve(&SettingsPatch::default())
    /// #     .map_err(|d| format!("{d:?}"))?;
    /// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
    /// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
    /// # let clock = UtcOffset::literal(5, 45, 0);
    /// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
    /// # let founder = Founder::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
    /// # let completion = Completion::new(&provider, resolved.settings.provider.overrides, delta_t);
    /// # let heliacal = Heliacal::new(&completion, kathmandu, Criterion::SURYA_SIDDHANTA, Horizon::CENTRE_NO_REFRACTION, delta_t);
    /// # let over = Over {
    /// #     almanac: &almanac,
    /// #     founder: &founder,
    /// #     completion: &completion,
    /// #     heliacal: &heliacal,
    /// #     calendar: &Gregorian,
    /// #     clock: &clock,
    /// #     settings: &resolved.settings,
    /// #     precession,
    /// #     delta_t,
    /// # };
    /// # // The chart zodiac taken at 2026-09-01 00:00 UTC.
    /// # let sources = ProviderSources::new(&over, kathmandu, JulianDay::literal(2_461_284.5))?;
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// // 25 November 2026, after Devuthani.
    /// let day = sources.day(&date(11, 25))?;
    /// use teistro_muhurta::{ActivityRules, ClauseKind};
    ///
    /// // Raman's marriage grades the lunar month; the baseline's the Sun's sign.
    /// let lunar = ActivityRules::raman_marriage().month_clauses(&day);
    /// assert!(lunar.iter().all(|c| matches!(c.kind, ClauseKind::Month { .. })));
    /// let solar = ActivityRules::baseline_marriage().month_clauses(&day);
    /// assert!(solar.iter().all(|c| matches!(c.kind, ClauseKind::SolarMonth { .. })));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

    /// The instant's clauses the rite's own rules give: the lagna's
    /// grade, a rejected pada of the Moon's star, and the grahas standing
    /// where the rite does not want them.
    ///
    /// ```
    /// # use teistro_core::interval::Interval;
    /// # use teistro_muhurta::instant::Sky;
    /// #
    /// # // Libra rising; Mars in Taurus (the 8th), Venus in Pisces, Jupiter in
    /// # // Libra, the Sun in Aquarius with Mercury, the Moon and Saturn in
    /// # // Capricorn, the nodes opposite.
    /// # let grahas = [320.0, 285.0, 45.0, 300.0, 190.0, 345.0, 285.0, 285.0, 105.0];
    /// # let sky = Sky { lagna_deg: 185.0, grahas, speeds: [1.0; 9] };
    /// # let at = Interval::literal(2_460_000.5, 2_460_000.51);
    /// use teistro_core::catalogue::Rashi;
    /// use teistro_muhurta::{ActivityRules, ClauseKind, Grade};
    ///
    /// // Libra rising is a good lagna for Raman's marriage; the Moon's quarter is not barred.
    /// let found = ActivityRules::raman_marriage().instant_clauses(&sky, at);
    /// assert_eq!(found.len(), 1);
    /// assert_eq!(found[0].kind, ClauseKind::Lagna { sign: Rashi::Libra, grade: Grade::Best });
    /// ```
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
        for house in 1..=12 {
            let named: Vec<Graha> = GRAHAS
                .into_iter()
                .filter(|graha| {
                    self.unwanted
                        .iter()
                        .any(|u| u.houses.contains(&house) && u.grahas.contains(graha))
                })
                .collect();
            let by = sky.in_house(house, &named);
            if !by.is_empty() {
                found.push(Clause {
                    kind: ClauseKind::UnwantedPlacement { house, by },
                    at,
                });
            }
        }
        found
    }

    /// Refuses rules that name a house or a quarter that does not exist,
    /// which would otherwise never match and so judge nothing, silently.
    ///
    /// ```
    /// use teistro_muhurta::{ActivityRules, Unwanted};
    ///
    /// let mut rules = ActivityRules::raman_marriage();
    /// rules.unwanted.push(Unwanted::vacant(vec![13]));
    /// assert_eq!(rules.check().unwrap_err().field(), Some("unwanted[0].houses"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `unwanted[i].houses` (a house outside 1 to 12)
    /// or `padas[i].pada` (a quarter outside 1 to 4).
    pub fn check(&self) -> Result<(), Error> {
        for (i, unwanted) in self.unwanted.iter().enumerate() {
            if let Some(house) = unwanted.houses.iter().find(|h| !(1..=12).contains(*h)) {
                return Err(Error::invalid_arg(format!(
                    "house {house} does not exist; houses are counted 1 to 12 from the lagna"
                ))
                .with_field(format!("unwanted[{i}].houses")));
            }
        }
        for (i, shool) in self.shool.iter().enumerate() {
            if shool.after_ghatis >= 60 {
                return Err(Error::invalid_arg(format!(
                    "ghati {} does not exist; a day's ghatis are 0 to 59 from sunrise",
                    shool.after_ghatis
                ))
                .with_field(format!("shool[{i}].afterGhatis")));
            }
        }
        for (i, pada) in self.padas.iter().enumerate() {
            if !(1..=4).contains(&pada.pada) {
                return Err(Error::invalid_arg(format!(
                    "quarter {} does not exist; a star's quarters are 1 to 4",
                    pada.pada
                ))
                .with_field(format!("padas[{i}].pada")));
            }
        }
        Ok(())
    }

    /// Whether a clause bars the rite outright.
    ///
    /// ```
    /// use teistro_core::interval::Interval;
    /// use teistro_muhurta::{ActivityRules, Clause, ClauseKind};
    ///
    /// let at = Interval::literal(2_460_000.5, 2_460_000.6);
    /// let rules = ActivityRules::raman_marriage();
    /// assert!(rules.bars(&Clause { kind: ClauseKind::VenusInSixth {}, at }));
    /// assert!(!rules.bars(&Clause { kind: ClauseKind::Abhijit {}, at }));
    /// ```
    #[must_use]
    pub fn bars(&self, clause: &Clause) -> bool {
        self.bars.iter().any(|bar| bar.names(&clause.kind)) || self.placement_bars(&clause.kind)
    }

    /// Whether a clause is an unwanted placement an entry that `bars`
    /// names: a graha standing where the rules say it must not.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_muhurta::{ActivityRules, ClauseKind, Unwanted};
    ///
    /// let mars = ClauseKind::UnwantedPlacement { house: 8, by: vec![Graha::Mars] };
    /// let mut rules = ActivityRules::raman_marriage();
    /// rules.unwanted.push(Unwanted::vacant(vec![8]));
    /// assert!(!rules.placement_bars(&mars), "a should weighs");
    /// rules.unwanted[0].bars = true;
    /// assert!(rules.placement_bars(&mars), "a must bars");
    /// ```
    #[must_use]
    pub fn placement_bars(&self, kind: &ClauseKind) -> bool {
        match kind {
            ClauseKind::UnwantedPlacement { house, by } => {
                self.unwanted.iter().any(|u| u.bars && u.names(*house, by))
            }
            _ => false,
        }
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
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::expect_used, reason = "tests fail by panicking")]
mod tests {
    use super::{ActivityRules, Bar, BlackoutKind, Pada};
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
                .is_empty(),
            "{:?}",
            rules.instant_clauses(
                &Sky {
                    lagna_deg: 95.0,
                    grahas,
                    speeds: [1.0; 9],
                },
                at
            )
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
        assert_eq!(rules.heeds.len(), 7);
        // The engine closes every adhika month, the samsarpa too.
        assert!(rules.heeds.contains(&BlackoutKind::AdhikaMasa));
        assert!(rules.heeds.contains(&BlackoutKind::Samsarpa));
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
