//! A festival day's sait, the time its rite is done
//! (`03-design/festival-sait.md`): the text's windows tried in order, each
//! with what it avoids, the rising sign a tradition prefers inside them,
//! and the instant chosen, with every candidate beside it.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Kaala, Karana, Modality, Nakshatra, Rashi, Yoga};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};

use super::DayPart;
use crate::almanac::Panchanga;
use crate::span::Span;

/// A ghati, and a nadi, in days: the sixtieth of a civil day.
const GHATI_DAYS: f64 = 1.0 / 60.0;

/// How a rule elects its sait (§3).
///
/// ```
/// use teistro_panchanga::festival::{SaitRule, SaitWindow};
///
/// // Ghatasthapana (Dharmasindhu p. 65): within ten ghatis of sunrise,
/// // failing that at midday in Abhijit.
/// let json = r#"{"rule": "GHATASTHAPANA", "source": "DS p. 65",
///   "windows": [{"within": {"window": "GHATIS", "ghatis": 10},
///                "avoid": [{"avoid": "TITHI_START", "nadis": 16}]},
///               {"within": {"window": "DAY_MUHURTA", "from": 8, "to": 8}}],
///   "rising": {"of": "NAVAMSA", "modalities": ["DWISWABHAVA"]}}"#;
/// let rule: SaitRule = serde_json::from_str(json)?;
/// assert_eq!(rule.windows[0].within, SaitWindow::Ghatis { ghatis: 10 });
/// assert!(rule.windows[1].avoid.is_empty());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaitRule {
    /// The key of the karmakala rule whose day it is on.
    pub rule: String,
    /// Where the rule is stated.
    pub source: String,
    /// The windows, in the order the text falls back through them: the
    /// first holding a candidate gives the sait.
    pub windows: Vec<SaitWindowRule>,
    /// The rising sign preferred inside the window; the window's free
    /// stretches themselves are the candidates when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rising: Option<SaitRising>,
    /// Which candidate is the sait.
    #[serde(default)]
    pub choose: SaitChoice,
}

/// One window a sait may fall in, with what it avoids there: a text keeps
/// its exclusions only where they fit (Dharmasindhu p. 65), so a fallback
/// window lists its own.
///
/// ```
/// use teistro_core::catalogue::Kaala;
/// use teistro_panchanga::festival::{SaitAvoid, SaitWindow, SaitWindowRule};
///
/// let forenoon = SaitWindowRule {
///     within: SaitWindow::DayMuhurta { from: 1, to: 6 },
///     avoid: vec![SaitAvoid::Kaala { kaala: Kaala::RahuKaala }],
/// };
/// let json = serde_json::to_string(&forenoon)?;
/// assert!(json.contains(r#""avoid":"KAALA""#));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaitWindowRule {
    /// The window.
    pub within: SaitWindow,
    /// What it avoids; nothing when left out.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub avoid: Vec<SaitAvoid>,
}

/// A stretch of the festival day a sait may fall in.
///
/// ```
/// use teistro_panchanga::festival::SaitWindow;
///
/// let json = serde_json::to_string(&SaitWindow::Ghatis { ghatis: 10 })?;
/// assert_eq!(json, r#"{"window":"GHATIS","ghatis":10}"#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "window", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum SaitWindow {
    /// The first ghatis after sunrise, a ghati a sixtieth of the civil
    /// day (24 minutes): Ghatasthapana's ten (DS p. 65).
    Ghatis {
        /// How many, 1 to 60.
        ghatis: u8,
    },
    /// Muhurtas of the daylight, a fifteenth of it from sunrise, `from` to
    /// `to` inclusive, 1 to 15: Abhijit is the 8th, the forenoon the first
    /// six.
    DayMuhurta {
        /// The first, 1 for the muhurta after sunrise.
        from: u8,
        /// The last, at least `from`.
        to: u8,
    },
    /// A fifth of the daylight.
    Part {
        /// Which fifth.
        part: DayPart,
    },
    /// Three muhurtas after sunset, a muhurta a fifteenth of the night.
    Pradosha,
    /// A muhurta of the night, 1 to 15 from sunset.
    NightMuhurta {
        /// Which muhurta.
        muhurta: u8,
    },
}

/// What a window avoids.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_panchanga::festival::SaitAvoid;
///
/// // Katyayana's reading drops only Chitra's first quarter (NS p. 123).
/// let first = SaitAvoid::Nakshatra { nakshatra: Nakshatra::Chitra, quarters: 1 };
/// let json = serde_json::to_string(&first)?;
/// assert_eq!(json, r#"{"avoid":"NAKSHATRA","nakshatra":"CHITRA","quarters":1}"#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "avoid", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum SaitAvoid {
    /// The occurrence's first nadis (Ghatasthapana's sixteen, NS p. 120).
    TithiStart {
        /// How many, 1 to 60.
        nadis: u8,
    },
    /// The Moon in a nakshatra: its first `quarters` quarters, 4 for the
    /// whole of it.
    Nakshatra {
        /// Which.
        nakshatra: Nakshatra,
        /// How many of its quarters from its start, 1 to 4.
        #[serde(default = "whole")]
        quarters: u8,
    },
    /// A nitya yoga: its first `quarters` quarters, 4 for the whole.
    Yoga {
        /// Which.
        yoga: Yoga,
        /// How many of its quarters from its start, 1 to 4.
        #[serde(default = "whole")]
        quarters: u8,
    },
    /// An eighth of the daylight: Rahu kaala, Yamaghanda or Gulika.
    Kaala {
        /// Which.
        kaala: Kaala,
    },
    /// Bhadra, the vishti karana.
    Vishti {},
}

const fn whole() -> u8 {
    4
}

/// The rising sign a tradition prefers for the rite: the navamsa's sign,
/// the lagna's, or either, of the modalities named, in a lagna none of
/// the grahas `free_of` names stands in (the lagna's purity, which a
/// muhurta asks of a malefic there).
///
/// ```
/// use teistro_core::catalogue::Modality;
/// use teistro_panchanga::festival::{RisingOf, SaitRising};
///
/// let dual = SaitRising {
///     of: RisingOf::Navamsa,
///     modalities: vec![Modality::Dwiswabhava],
///     free_of: Vec::new(),
/// };
/// let json = serde_json::to_string(&dual)?;
/// assert_eq!(json, r#"{"of":"NAVAMSA","modalities":["DWISWABHAVA"]}"#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaitRising {
    /// Which sign is read.
    pub of: RisingOf,
    /// The modalities it may be of.
    pub modalities: Vec<Modality>,
    /// The grahas the lagna must not hold, read by sign; none when
    /// absent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub free_of: Vec<Graha>,
}

impl SaitRising {
    /// What the committee's printed saits hold to (`festival-sait.md`
    /// §1, cruxes C745 and C747): a navamsa of a dual sign, in a lagna
    /// the Sun does not stand in.
    ///
    /// ```
    /// use teistro_core::catalogue::{Graha, Modality};
    /// use teistro_panchanga::festival::SaitRising;
    ///
    /// let rising = SaitRising::dual_navamsa();
    /// assert_eq!(rising.modalities, [Modality::Dwiswabhava]);
    /// assert_eq!(rising.free_of, [Graha::Sun]);
    /// ```
    #[must_use]
    pub fn dual_navamsa() -> Self {
        Self {
            of: RisingOf::Navamsa,
            modalities: vec![Modality::Dwiswabhava],
            free_of: vec![Graha::Sun],
        }
    }
}

/// Which rising sign a [`SaitRising`] reads.
///
/// ```
/// use teistro_panchanga::festival::RisingOf;
///
/// assert_eq!(serde_json::to_string(&RisingOf::NavamsaOrLagna)?, r#""NAVAMSA_OR_LAGNA""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum RisingOf {
    /// The navamsa's sign.
    Navamsa,
    /// The lagna's sign.
    Lagna,
    /// Either: a navamsa counts when its own sign is of a modality named
    /// or its lagna's is.
    NavamsaOrLagna,
}

/// Which candidate is the sait: whole ones are preferred, and among them
/// the earliest or the latest (crux C746).
///
/// ```
/// use teistro_panchanga::festival::SaitChoice;
///
/// assert_eq!(SaitChoice::default(), SaitChoice::Latest);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum SaitChoice {
    /// The first.
    Earliest,
    /// The last.
    #[default]
    Latest,
}

/// The navamsas risen over a window: the one at its start, then each
/// entered inside it, by index (0, Aries' first, to 107, Pisces' last).
///
/// ```
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::festival::Navamsas;
///
/// let risen = Navamsas {
///     first: 26,
///     entered: vec![(JulianDay::literal(2_460_000.6), 27)],
///     held: Vec::new(),
/// };
/// assert_eq!(risen.entered.len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Navamsas {
    /// The navamsa rising at the window's start.
    pub first: u8,
    /// Each later one, with when it rose.
    pub entered: Vec<(JulianDay<Utc>, u8)>,
    /// The signs the rising's `free_of` grahas stand in over the window,
    /// read at its middle.
    pub held: Vec<Rashi>,
}

/// A festival day's sait: the window it was found in, every candidate
/// there, and the one chosen.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::festival::{Sait, SaitCandidate};
///
/// let one = SaitCandidate { span: Interval::literal(2_460_000.6, 2_460_000.61), whole: true, lagna: None, navamsa: None };
/// let sait = Sait { window: 0, within: one.span, candidates: vec![one], chosen: Some(0), at: one.span.at_fraction(0.5).ok() };
/// assert!(sait.at.is_some());
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Sait {
    /// Which of the rule's windows gave it, from 0; the last when none
    /// did.
    pub window: usize,
    /// That window on the day.
    pub within: Interval,
    /// Every candidate in it, in order.
    pub candidates: Vec<SaitCandidate>,
    /// The one chosen, by its index; none when the window holds none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen: Option<usize>,
    /// The chosen candidate's middle, the instant an almanac prints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<JulianDay<Utc>>,
}

/// A stretch the sait may be in.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::festival::SaitCandidate;
///
/// let kanya = SaitCandidate {
///     span: Interval::literal(2_460_000.6, 2_460_000.61),
///     whole: true,
///     lagna: Some(Rashi::Sagittarius),
///     navamsa: Some(Rashi::Virgo),
/// };
/// assert!(kanya.whole);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SaitCandidate {
    /// When.
    pub span: Interval,
    /// Whether the navamsa lies wholly inside the free window; always
    /// for a candidate without a rising sign.
    pub whole: bool,
    /// The lagna's sign, when the rule reads a rising sign.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lagna: Option<Rashi>,
    /// The navamsa's sign, when the rule reads a rising sign.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navamsa: Option<Rashi>,
}

impl SaitRule {
    /// Whether every window and avoidance is in range: a ghati count 1
    /// to 60, muhurtas 1 to 15 in order, quarters 1 to 4, and at least
    /// one window, and a key.
    ///
    /// ```
    /// use teistro_panchanga::festival::{SaitChoice, SaitRule, SaitWindow, SaitWindowRule};
    ///
    /// let rule = |within| SaitRule {
    ///     rule: String::from("GHATASTHAPANA"),
    ///     source: String::new(),
    ///     windows: vec![SaitWindowRule { within, avoid: Vec::new() }],
    ///     rising: None,
    ///     choose: SaitChoice::Latest,
    /// };
    /// assert!(rule(SaitWindow::Ghatis { ghatis: 10 }).check().is_ok());
    /// assert!(rule(SaitWindow::DayMuhurta { from: 9, to: 8 }).check().is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG`, naming the field that is out of range.
    pub fn check(&self) -> Result<(), Error> {
        if self.rule.is_empty() {
            return Err(
                Error::invalid_arg("a sait names the rule whose day it is on").with_field("rule"),
            );
        }
        if self.windows.is_empty() {
            return Err(
                Error::invalid_arg("a sait names at least one window").with_field("windows")
            );
        }
        for (k, rule) in self.windows.iter().enumerate() {
            let at = |field: &str| format!("windows[{k}].{field}");
            let bad = match rule.within {
                SaitWindow::Ghatis { ghatis } => !(1..=60).contains(&ghatis),
                SaitWindow::DayMuhurta { from, to } => {
                    !(1..=15).contains(&from) || !(from..=15).contains(&to)
                }
                SaitWindow::NightMuhurta { muhurta } => !(1..=15).contains(&muhurta),
                SaitWindow::Part { .. } | SaitWindow::Pradosha => false,
            };
            if bad {
                return Err(Error::invalid_arg(
                    "a window out of range: ghatis 1 to 60, muhurtas 1 to 15 in order",
                )
                .with_field(at("within")));
            }
            for (j, avoid) in rule.avoid.iter().enumerate() {
                let bad = match *avoid {
                    SaitAvoid::TithiStart { nadis } => !(1..=60).contains(&nadis),
                    SaitAvoid::Nakshatra { quarters, .. } | SaitAvoid::Yoga { quarters, .. } => {
                        !(1..=4).contains(&quarters)
                    }
                    SaitAvoid::Kaala { .. } | SaitAvoid::Vishti {} => false,
                };
                if bad {
                    return Err(Error::invalid_arg(
                        "an avoidance out of range: nadis 1 to 60, quarters 1 to 4",
                    )
                    .with_field(at(&format!("avoid[{j}]"))));
                }
            }
        }
        if self
            .rising
            .as_ref()
            .is_some_and(|r| r.modalities.is_empty())
        {
            return Err(
                Error::invalid_arg("a rising sign names at least one modality")
                    .with_field("rising.modalities"),
            );
        }
        Ok(())
    }

    /// The sait on `day`, the festival's day, for an `occurrence` (its
    /// tithi's span): each window in turn, less what it avoids, read
    /// against the navamsas `risen` gives over it, until one holds a
    /// candidate.
    ///
    /// ```
    /// use teistro_panchanga::festival::{SaitChoice, SaitRule, SaitWindow, SaitWindowRule};
    ///
    /// // Built in the sdk, where the day and its rising signs are found;
    /// // here only the rule's shape.
    /// let rule = SaitRule {
    ///     rule: String::from("GHATASTHAPANA"),
    ///     source: String::from("DS p. 65"),
    ///     windows: vec![SaitWindowRule { within: SaitWindow::Ghatis { ghatis: 10 }, avoid: Vec::new() }],
    ///     rising: None,
    ///     choose: SaitChoice::Earliest,
    /// };
    /// assert!(rule.reads_rising() == false);
    /// ```
    ///
    /// # Errors
    ///
    /// Whatever `risen` returns, and `INTERNAL` for a day whose windows
    /// cannot be drawn.
    pub fn elect<F>(
        &self,
        day: &Panchanga,
        occurrence: Interval,
        mut risen: F,
    ) -> Result<Sait, Error>
    where
        F: FnMut(Interval) -> Result<Navamsas, Error>,
    {
        let mut last = None;
        for (k, rule) in self.windows.iter().enumerate() {
            let Some(within) = window_on(rule.within, day)? else {
                continue;
            };
            let avoided: Vec<Interval> = rule
                .avoid
                .iter()
                .flat_map(|avoid| avoided(*avoid, day, occurrence))
                .collect();
            let free = less(within, &avoided);
            let candidates = match &self.rising {
                None => free
                    .iter()
                    .map(|&span| SaitCandidate {
                        span,
                        whole: true,
                        lagna: None,
                        navamsa: None,
                    })
                    .collect(),
                Some(rising) => rising_in(rising, &free, &risen(within)?, within),
            };
            if candidates.is_empty() {
                last = Some((k, within));
                continue;
            }
            let chosen = self.chosen(&candidates);
            let at = chosen
                .and_then(|i| candidates.get(i))
                .map(|c| c.span.at_fraction(0.5))
                .transpose()?;
            return Ok(Sait {
                window: k,
                within,
                candidates,
                chosen,
                at,
            });
        }
        let (window, within) =
            last.ok_or_else(|| Error::internal("no window of the sait could be drawn on its day"))?;
        Ok(Sait {
            window,
            within,
            candidates: Vec::new(),
            chosen: None,
            at: None,
        })
    }

    /// The saits Nepal's national panchanga prints for the `NEPAL` pack's
    /// rules (§2): Ghatasthapana (`NAVARATRA_ARAMBHA`) and Bhai Tika
    /// (`YAMA_DWITIYA`), each a navamsa of a dual sign (crux C745) inside
    /// the text's window; and Laxmi Puja's window, the pradosha, with no
    /// lagna (C824). Vijaya Dashami's tika is not among them: no rule
    /// found elects all four printed years (C823).
    ///
    /// ```
    /// use teistro_panchanga::festival::{SaitRule, SaitWindow};
    ///
    /// let saits = SaitRule::nepal();
    /// assert!(saits.iter().all(|sait| sait.check().is_ok()));
    /// assert!(saits.iter().any(|sait| sait.rule == "NAVARATRA_ARAMBHA"));
    /// let lakshmi = saits.iter().find(|sait| sait.rule == "LAKSHMI_PUJA").ok_or("shipped")?;
    /// assert_eq!((lakshmi.windows[0].within, lakshmi.rising.is_none()), (SaitWindow::Pradosha, true));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn nepal() -> Vec<SaitRule> {
        let dual = || Some(SaitRising::dual_navamsa());
        let abhijit = || SaitWindowRule {
            within: SaitWindow::DayMuhurta { from: 8, to: 8 },
            avoid: Vec::new(),
        };
        vec![
            SaitRule {
                rule: "NAVARATRA_ARAMBHA".to_owned(),
                source: "Dharmasindhu p. 65 and Nirnayasindhu pp. 120-124: within ten ghatis of sunrise, avoiding the 1st's first sixteen nadis, Chitra and Vaidhriti, else at midday in Abhijit; Rahu kaala avoided, a dual sign's navamsa in a lagna free of the Sun taken as the committee prints (C745, C747), the latest (C746)".to_owned(),
                windows: vec![
                    SaitWindowRule {
                        within: SaitWindow::Ghatis { ghatis: 10 },
                        avoid: vec![
                            SaitAvoid::TithiStart { nadis: 16 },
                            SaitAvoid::Nakshatra {
                                nakshatra: Nakshatra::Chitra,
                                quarters: 4,
                            },
                            SaitAvoid::Yoga {
                                yoga: Yoga::Vaidhriti,
                                quarters: 4,
                            },
                            SaitAvoid::Kaala {
                                kaala: Kaala::RahuKaala,
                            },
                        ],
                    },
                    abhijit(),
                ],
                rising: dual(),
                choose: SaitChoice::Latest,
            },
            SaitRule {
                rule: "LAKSHMI_PUJA".to_owned(),
                source: "Dharmasindhu p. 77 and Nirnayasindhu pp. 148-149: the puja in pradosha, the three muhurtas of the night from sunset; the text's window only, as the committee's printed lagna for it was not recovered (C824)".to_owned(),
                windows: vec![SaitWindowRule {
                    within: SaitWindow::Pradosha,
                    avoid: Vec::new(),
                }],
                rising: None,
                choose: SaitChoice::Earliest,
            },
            SaitRule {
                rule: "YAMA_DWITIYA".to_owned(),
                source: "the committee's Bhai Tika: Abhijit (Hemadri's madhyahna, Nirnayasindhu p. 150) and a dual sign's navamsa in a lagna free of the Sun (C745, C747)".to_owned(),
                windows: vec![abhijit()],
                rising: dual(),
                choose: SaitChoice::Latest,
            },
        ]
    }

    /// The `NEPAL_PANCHANGA` pack's saits beside [`SaitRule::nepal`]'s:
    /// Phulpati's in the forenoon (Nirnayasindhu pp. 127-128).
    ///
    /// ```
    /// use teistro_panchanga::festival::SaitRule;
    ///
    /// assert!(SaitRule::nepal_panchanga().iter().all(|sait| sait.rule == "PHULPATI"));
    /// ```
    #[must_use]
    pub fn nepal_panchanga() -> Vec<SaitRule> {
        vec![SaitRule {
            rule: "PHULPATI".to_owned(),
            source: "Nirnayasindhu pp. 127-128: the puja in the forenoon, the first six day muhurtas; Rahu kaala avoided, a dual sign's navamsa in a lagna free of the Sun taken as the committee prints (C745, C747), the earliest (C746)".to_owned(),
            windows: vec![SaitWindowRule {
                within: SaitWindow::DayMuhurta { from: 1, to: 6 },
                avoid: vec![SaitAvoid::Kaala {
                    kaala: Kaala::RahuKaala,
                }],
            }],
            rising: Some(SaitRising::dual_navamsa()),
            choose: SaitChoice::Earliest,
        }]
    }

    /// Whether the rule reads a rising sign, which the caller then finds
    /// over each window.
    ///
    /// ```
    /// use teistro_panchanga::festival::{SaitChoice, SaitRule};
    ///
    /// let free = SaitRule { rule: String::new(), source: String::new(), windows: Vec::new(), rising: None, choose: SaitChoice::Latest };
    /// assert!(!free.reads_rising());
    /// ```
    #[must_use]
    pub const fn reads_rising(&self) -> bool {
        self.rising.is_some()
    }

    /// The chosen candidate: a whole one by [`SaitChoice`]; when the
    /// window cuts every candidate short, the longest stretch left, as a
    /// navamsa's stub at a window's edge is no rising to perform in.
    fn chosen(&self, candidates: &[SaitCandidate]) -> Option<usize> {
        let mut whole = candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| c.whole)
            .map(|(i, _)| i);
        let picked = match self.choose {
            SaitChoice::Earliest => whole.next(),
            SaitChoice::Latest => whole.next_back(),
        };
        picked.or_else(|| {
            let length = |c: &SaitCandidate| c.span.to.get() - c.span.from.get();
            // The first of equal lengths, so the answer cannot hang on
            // the last bit of a sum.
            candidates
                .iter()
                .enumerate()
                .fold(None, |best: Option<(usize, f64)>, (i, c)| match best {
                    Some((_, longest)) if length(c) <= longest => best,
                    _ => Some((i, length(c))),
                })
                .map(|(i, _)| i)
        })
    }
}

/// A window drawn on the day; none when the day has no such stretch (no
/// night muhurtas past a polar day's end).
fn window_on(window: SaitWindow, day: &Panchanga) -> Result<Option<Interval>, Error> {
    let (sunrise, sunset) = (day.day.sunrise, day.day.sunset);
    let muhurtas = |from: u8, to: u8, of: &[Interval]| -> Option<Interval> {
        let first = of.get(usize::from(from).checked_sub(1)?)?;
        let last = of.get(usize::from(to).checked_sub(1)?)?;
        Some(Interval::literal(first.from.get(), last.to.get()))
    };
    Ok(match window {
        SaitWindow::Ghatis { ghatis } => Some(Interval::new(
            sunrise,
            JulianDay::literal(sunrise.get() + f64::from(ghatis) * GHATI_DAYS),
        )?),
        SaitWindow::DayMuhurta { from, to } => muhurtas(from, to, &day.muhurtas.daylight),
        SaitWindow::NightMuhurta { muhurta } => muhurtas(muhurta, muhurta, &day.muhurtas.night),
        SaitWindow::Pradosha => muhurtas(1, 3, &day.muhurtas.night),
        SaitWindow::Part { part } => Some(Interval::new(sunrise, sunset)?.part(part.index(), 5)?),
    })
}

/// The stretches an avoidance covers on the day.
fn avoided(avoid: SaitAvoid, day: &Panchanga, occurrence: Interval) -> Vec<Interval> {
    match avoid {
        SaitAvoid::TithiStart { nadis } => Interval::new(
            occurrence.from,
            JulianDay::literal(occurrence.from.get() + f64::from(nadis) * GHATI_DAYS),
        )
        .into_iter()
        .collect(),
        SaitAvoid::Nakshatra {
            nakshatra,
            quarters,
        } => day
            .limbs
            .nakshatra
            .iter()
            .filter(|span| span.member == nakshatra)
            .filter_map(|span| quartered(span, quarters))
            .collect(),
        SaitAvoid::Yoga { yoga, quarters } => day
            .limbs
            .yoga
            .iter()
            .filter(|span| span.member == yoga)
            .filter_map(|span| quartered(span, quarters))
            .collect(),
        SaitAvoid::Kaala { kaala } => day
            .kaalas
            .iter()
            .filter(|k| k.kaala == kaala)
            .map(|k| k.at)
            .collect(),
        SaitAvoid::Vishti {} => day
            .limbs
            .karana
            .iter()
            .filter(|span| span.member == Karana::Vishti)
            .map(|span| span.whole)
            .collect(),
    }
}

/// A span's first `quarters` quarters, from where it began.
fn quartered<T>(span: &Span<T>, quarters: u8) -> Option<Interval> {
    span.whole
        .at_fraction(f64::from(quarters) / 4.0)
        .ok()
        .and_then(|to| Interval::new(span.whole.from, to).ok())
}

/// `within` less every avoided stretch, as its free pieces in order.
fn less(within: Interval, avoided: &[Interval]) -> Vec<Interval> {
    let mut cut: Vec<Interval> = avoided
        .iter()
        .filter_map(|a| a.clipped_to(within))
        .collect();
    cut.sort_by(|a, b| a.from.get().total_cmp(&b.from.get()));
    let mut free = Vec::new();
    let mut from = within.from;
    for gap in cut {
        if gap.from.get() > from.get()
            && let Ok(piece) = Interval::new(from, gap.from)
        {
            free.push(piece);
        }
        if gap.to.get() > from.get() {
            from = gap.to;
        }
    }
    if within.to.get() > from.get()
        && let Ok(piece) = Interval::new(from, within.to)
    {
        free.push(piece);
    }
    free
}

/// Each navamsa of a preferred sign over the free pieces, as a candidate:
/// its part inside a piece, whole when it lies wholly in one.
fn rising_in(
    rising: &SaitRising,
    free: &[Interval],
    risen: &Navamsas,
    within: Interval,
) -> Vec<SaitCandidate> {
    let sign = |index: u8| Rashi::from_id(u16::from(index % 12));
    let lagna = |index: u8| Rashi::from_id(u16::from(index / 9));
    let held =
        |r: Option<Rashi>| r.is_some_and(|r| rising.modalities.contains(&r.attributes().modality));
    // Each navamsa's own stretch over the window, its edges where it rose
    // and where the next did.
    let mut edges = vec![(within.from, risen.first)];
    edges.extend(risen.entered.iter().copied());
    let mut candidates = Vec::new();
    for (k, &(from, index)) in edges.iter().enumerate() {
        let to = edges.get(k + 1).map_or(within.to, |next| next.0);
        let (navamsa, risen_in) = (sign(index), lagna(index));
        let preferred = match rising.of {
            RisingOf::Navamsa => held(navamsa),
            RisingOf::Lagna => held(risen_in),
            RisingOf::NavamsaOrLagna => held(navamsa) || held(risen_in),
        };
        let Ok(stretch) = Interval::new(from, to) else {
            continue;
        };
        if !preferred || risen_in.is_some_and(|sign| risen.held.contains(&sign)) {
            continue;
        }
        // A navamsa cut by the window's own edge is not whole either.
        let bounded = k > 0 && edges.get(k + 1).is_some();
        for piece in free {
            let Some(part) = stretch.clipped_to(*piece) else {
                continue;
            };
            candidates.push(SaitCandidate {
                span: part,
                whole: bounded && part == stretch,
                lagna: risen_in,
                navamsa,
            });
        }
    }
    candidates
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::indexing_slicing,
        reason = "a test fails by panicking and indexes what it built"
    )]

    use super::*;

    #[test]
    fn a_window_less_its_avoidances_is_its_free_pieces_in_order() {
        let within = Interval::literal(10.0, 20.0);
        let free = less(
            within,
            &[
                Interval::literal(12.0, 13.0),
                Interval::literal(9.0, 11.0),
                Interval::literal(12.5, 14.0),
            ],
        );
        assert_eq!(
            free,
            vec![Interval::literal(11.0, 12.0), Interval::literal(14.0, 20.0)]
        );
        assert_eq!(less(within, &[Interval::literal(0.0, 30.0)]), Vec::new());
        assert_eq!(less(within, &[]), vec![within]);
    }

    #[test]
    fn a_navamsa_is_whole_only_between_two_risings_inside_a_free_piece() {
        let dual = SaitRising {
            free_of: Vec::new(),
            ..SaitRising::dual_navamsa()
        };
        // Navamsas 2 (Mithuna), 3 (Karka), 4 (Simha), 5 (Kanya) over the
        // window, Mithuna at its start.
        let within = Interval::literal(0.0, 4.0);
        let risen = Navamsas {
            first: 2,
            entered: vec![
                (JulianDay::literal(1.0), 3),
                (JulianDay::literal(2.0), 4),
                (JulianDay::literal(3.0), 5),
            ],
            held: Vec::new(),
        };
        let found = rising_in(&dual, &[within], &risen, within);
        let signs: Vec<_> = found.iter().map(|c| (c.navamsa, c.whole)).collect();
        // Mithuna is cut by the window's start and Kanya by its end.
        assert_eq!(
            signs,
            vec![(Some(Rashi::Gemini), false), (Some(Rashi::Virgo), false)]
        );
        let inner = Navamsas {
            first: 1,
            entered: vec![(JulianDay::literal(1.0), 2), (JulianDay::literal(2.0), 3)],
            held: Vec::new(),
        };
        let found = rising_in(&dual, &[within], &inner, within);
        assert_eq!(found.len(), 1);
        assert!(found[0].whole);
        assert_eq!(found[0].lagna, Some(Rashi::Aries));
        // A lagna a named graha holds gives no candidate.
        let held = Navamsas {
            held: vec![Rashi::Aries],
            ..inner
        };
        assert_eq!(rising_in(&dual, &[within], &held, within), Vec::new());
    }

    #[test]
    fn a_whole_candidate_is_chosen_by_the_rule_and_else_the_longest_stretch() {
        let candidate = |from: f64, whole| SaitCandidate {
            span: Interval::literal(from, from + 1.0),
            whole,
            lagna: None,
            navamsa: None,
        };
        let all = [
            candidate(0.0, false),
            candidate(2.0, true),
            candidate(4.0, true),
            candidate(6.0, false),
        ];
        let rule = |choose| SaitRule {
            rule: String::new(),
            source: String::new(),
            windows: Vec::new(),
            rising: None,
            choose,
        };
        assert_eq!(rule(SaitChoice::Earliest).chosen(&all), Some(1));
        assert_eq!(rule(SaitChoice::Latest).chosen(&all), Some(2));
        // None whole: the longest stretch, whatever the choice, and the
        // first of two equal ones.
        let stub = SaitCandidate {
            span: Interval::literal(8.0, 8.25),
            ..candidate(8.0, false)
        };
        let parts = [candidate(0.0, false), stub];
        assert_eq!(rule(SaitChoice::Latest).chosen(&parts), Some(0));
        assert_eq!(
            rule(SaitChoice::Earliest).chosen(&[stub, candidate(0.0, false)]),
            Some(1)
        );
        let even = [candidate(0.0, false), candidate(6.0, false)];
        assert_eq!(rule(SaitChoice::Latest).chosen(&even), Some(0));
        assert_eq!(rule(SaitChoice::Latest).chosen(&[]), None);
    }
}
