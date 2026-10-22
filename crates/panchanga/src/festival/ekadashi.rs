//! Ekadashi: which of two days the eleventh's fast falls on
//! (`03-design/festival-rules.md` §8; *Dharmasindhu*, pp. 11–12).
//!
//! An Ekadashi is not a karmakala rule. Its two days are the 11th's own
//! day and the day after (C181), and what decides between them is not
//! how the tithi holds a window but two facts about it. Is it **pierced**
//! (*viddha*) by the 10th: at arunodaya, four ghatis before sunrise, as
//! Vaishnavas reckon, or at sunrise itself, as Smartas do? And which of
//! the 11th and the 12th stand **in excess** (*adhikya*), holding the
//! sunrise after their own? The text answers each of the eight kinds for
//! each observer, so a rule carries its answers as a table.
//!
//! The fast is broken the day after, inside the 12th and past its first
//! quarter, in the windows a [`ParanaRule`] names (§8.5).

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Masa, Tithi};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};

use super::{FestivalDay, Observances, Unjudged, Which, Window, check_window, occurrences};

/// Where the 10th pierces the 11th's day (p. 11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Vedha {
    /// At arunodaya, four ghatis of the night before sunrise: the 10th
    /// "entering even a pala past 56 ghatis". The Vaishnavas' vedha.
    Arunodaya,
    /// At sunrise: the 10th standing "a pala past the sixty-ghati
    /// sunrise". The Smartas'. Pierced at sunrise is pierced at arunodaya
    /// too.
    Sunrise,
}

/// Which of the 11th and the 12th stand past the sunrise after their own
/// day (*adhikya*, p. 11: "सूर्योदयोत्तरसत्त्वम्").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Excess {
    /// The 11th alone holds the later day's sunrise.
    Eleventh,
    /// The 12th alone holds the sunrise after the later day.
    Twelfth,
    /// Both do.
    Both,
    /// Neither does.
    Neither,
}

impl Excess {
    /// The kind, from whether each stands in excess.
    #[must_use]
    pub const fn of(eleventh: bool, twelfth: bool) -> Excess {
        match (eleventh, twelfth) {
            (true, false) => Excess::Eleventh,
            (false, true) => Excess::Twelfth,
            (true, true) => Excess::Both,
            (false, false) => Excess::Neither,
        }
    }
}

/// The day each of the four kinds takes: the 11th's own (`EARLIER`) or
/// the day after (`LATER`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct EkadashiKinds {
    /// The 11th alone in excess.
    pub eleventh: Which,
    /// The 12th alone in excess.
    pub twelfth: Which,
    /// Both in excess.
    pub both: Which,
    /// Neither in excess.
    pub neither: Which,
}

impl EkadashiKinds {
    /// The day this kind takes.
    #[must_use]
    pub const fn of(self, excess: Excess) -> Which {
        match excess {
            Excess::Eleventh => self.eleventh,
            Excess::Twelfth => self.twelfth,
            Excess::Both => self.both,
            Excess::Neither => self.neither,
        }
    }

    const fn all(day: Which) -> EkadashiKinds {
        EkadashiKinds {
            eleventh: day,
            twelfth: day,
            both: day,
            neither: day,
        }
    }
}

/// An observer's answers, for the 11th pure and pierced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct EkadashiTable {
    /// When the 10th does not pierce the 11th's day.
    pub pure: EkadashiKinds,
    /// When it does.
    pub pierced: EkadashiKinds,
}

/// Whose fast, and by what: a vedha and the table of eight answers.
///
/// ```
/// use teistro_panchanga::festival::{EkadashiRule, Excess, Vedha, Which};
///
/// let rules = EkadashiRule::dharmasindhu();
/// let smarta = rules.iter().find(|rule| rule.key == "EKADASHI_SMARTA").unwrap();
/// assert_eq!(smarta.vedha, Vedha::Sunrise);
/// // p. 12: "in both-excess and 12th-only excess Smartas leave the
/// // pierced, and not otherwise".
/// assert_eq!(smarta.table.pierced.of(Excess::Eleventh), Which::Earlier);
/// assert_eq!(smarta.table.pierced.of(Excess::Twelfth), Which::Later);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct EkadashiRule {
    /// The rule's key in its pack, `"EKADASHI_SMARTA"`.
    pub key: String,
    /// Where the rule is stated.
    pub source: String,
    /// Where the 10th pierces.
    pub vedha: Vedha,
    /// The day each kind takes.
    pub table: EkadashiTable,
    /// When the fast is broken (§8.5); a rule without one answers no
    /// parana.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parana: Option<ParanaRule>,
}

impl EkadashiRule {
    /// Refuses what could not name an answer: an empty key, and a
    /// parana with no window or one outside the daylight.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `key` or on the parana's `at`.
    pub fn check(&self) -> Result<(), Error> {
        if self.key.trim().is_empty() {
            return Err(Error::invalid_arg(
                "an Ekadashi rule needs a key: an answer is named by it",
            )
            .with_field("key"));
        }
        self.parana
            .as_ref()
            .map_or(Ok(()), |parana| parana.check(&self.key))
    }

    /// *Dharmasindhu*'s three observers (§8.3), each citing its rows.
    #[must_use]
    pub fn dharmasindhu() -> Vec<EkadashiRule> {
        use Which::{Earlier, Later};
        vec![
            EkadashiRule {
                key: "EKADASHI_VAISHNAVA".to_owned(),
                source: "Dharmasindhu pp. 11–12: pierced at arunodaya, the 11th is left and the 12th's day fasted, in every kind (S1 to S8); pure, the later day unless neither stands in excess (V1 to V4)".to_owned(),
                vedha: Vedha::Arunodaya,
                table: EkadashiTable {
                    pure: EkadashiKinds {
                        neither: Earlier,
                        ..EkadashiKinds::all(Later)
                    },
                    pierced: EkadashiKinds::all(Later),
                },
                parana: Some(ParanaRule::dharmasindhu()),
            },
            EkadashiRule {
                key: "EKADASHI_SMARTA".to_owned(),
                source: "Dharmasindhu pp. 11–12, the householder: pierced at sunrise, the 11th is left only when the 12th stands in excess (S2, S4, S6, S8); pure, the earlier day unless both stand in excess (S1, S3, S5, S7), S5 by Madhava, whom the closing says present practice follows (C182)".to_owned(),
                vedha: Vedha::Sunrise,
                table: EkadashiTable {
                    pure: EkadashiKinds {
                        eleventh: Earlier,
                        twelfth: Earlier,
                        both: Later,
                        neither: Earlier,
                    },
                    pierced: EkadashiKinds {
                        eleventh: Earlier,
                        twelfth: Later,
                        both: Later,
                        neither: Earlier,
                    },
                },
                parana: Some(ParanaRule::dharmasindhu()),
            },
            EkadashiRule {
                key: "EKADASHI_SMARTA_RENUNCIANT".to_owned(),
                source: "Dharmasindhu pp. 11–12: yatis, the desireless householder, forest-dwellers, widows and those seeking release take the later day with the 11th in excess (S1, S2) and pierced with neither (S8), and the householder's day otherwise; the later in S5 is only what \"some say\"".to_owned(),
                vedha: Vedha::Sunrise,
                table: EkadashiTable {
                    pure: EkadashiKinds {
                        eleventh: Later,
                        twelfth: Earlier,
                        both: Later,
                        neither: Earlier,
                    },
                    pierced: EkadashiKinds::all(Later),
                },
                parana: Some(ParanaRule::dharmasindhu()),
            },
        ]
    }
}

/// When an Ekadashi fast is broken (*parana*, §8.5): on the day after
/// the fast, inside the 12th, in the first of these daylight windows that
/// leaves time once the 12th's first quarter, the *hari vasara*, is past.
///
/// *Dharmasindhu* p. 19 and *Hari Bhakti Vilasa* 13.238–259 put the meal
/// inside the 12th and past its first quarter, and on the 13th when the
/// 12th is gone by the parana day's sunrise (HBV 13.264–287, a fast kept
/// on the 12th). The windows are a reading of p. 19's "the first three
/// muhurtas of the morning, say many; the afternoon, say some", which is
/// the consumer's to change: [`ParanaRule::dharmasindhu`] keeps the
/// many's morning and the some's afternoon when the morning is lost.
///
/// ```
/// use teistro_panchanga::festival::{DayPart, ParanaRule, Window};
///
/// let rule = ParanaRule::dharmasindhu();
/// assert_eq!(rule.at[0], Window::Part { part: DayPart::Pratah });
/// assert!(rule.hari_vasara);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParanaRule {
    /// The daylight windows the meal is kept in, in order of preference:
    /// each a span of the daylight (`PART` or `DAY_MUHURTA`).
    pub at: Vec<Window>,
    /// Whether the 12th's first quarter is passed over before eating.
    pub hari_vasara: bool,
}

impl ParanaRule {
    /// *Dharmasindhu* p. 19: pratah, the daylight's first fifth ("the
    /// first three muhurtas of the morning, not at midday, say many"),
    /// then aparahna, its fourth, when the hari vasara outlasts the
    /// morning ("the afternoon, say some"); past the hari vasara.
    #[must_use]
    pub fn dharmasindhu() -> ParanaRule {
        use super::DayPart::{Aparahna, Pratah};
        ParanaRule {
            at: vec![
                Window::Part { part: Pratah },
                Window::Part { part: Aparahna },
            ],
            hari_vasara: true,
        }
    }

    /// Refuses a parana with no window, or one that is not a span of the
    /// daylight.
    fn check(&self, key: &str) -> Result<(), Error> {
        if self.at.is_empty() {
            return Err(Error::invalid_arg(format!(
                "{key}: a parana needs a window to keep the meal in"
            ))
            .with_field("parana.at"));
        }
        for (index, &window) in self.at.iter().enumerate() {
            let at = format!("parana.at[{index}]");
            if !matches!(window, Window::Part { .. } | Window::DayMuhurta { .. }) {
                return Err(Error::invalid_arg(format!(
                    "{key}: a parana is kept in a span of the daylight, PART or DAY_MUHURTA, not {window:?}"
                ))
                .with_field(at));
            }
            check_window(key, window, &at)?;
        }
        Ok(())
    }
}

/// What set an edge of the window a parana was kept in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ParanaEdge {
    /// The rule's window's own edge.
    Window,
    /// The end of the hari vasara, the 12th's first quarter.
    HariVasara,
    /// The start of the tithi the meal is kept in: the 12th, when the
    /// 11th stands past the parana day's sunrise.
    TithiStart,
    /// The end of the tithi the meal is kept in.
    TithiEnd,
}

/// The window a parana was kept in.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ParanaKept {
    /// When the meal may be taken.
    pub span: Interval,
    /// The rule's window it lies in.
    pub at: Window,
    /// What set its start.
    pub from_by: ParanaEdge,
    /// What set its end.
    pub to_by: ParanaEdge,
}

/// When a fast is broken, under its rule's [`ParanaRule`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Parana {
    /// The day after the fast.
    pub day: CalendarDate,
    /// The tithi the meal is kept in: the 12th when it stands past that
    /// day's sunrise, and otherwise the tithi at the sunrise.
    pub tithi: Tithi,
    /// When the hari vasara ends, if after that day's sunrise, whether or
    /// not the rule passes it over.
    pub hari_vasara_ends: Option<JulianDay<Utc>>,
    /// The window kept; null when none of the rule's leaves time, as the
    /// morning alone does when the hari vasara outlasts it.
    pub kept: Option<ParanaKept>,
}

/// An Ekadashi's fast: the day a rule gives it, and the facts that gave
/// it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct EkadashiFast {
    /// The rule's key.
    pub rule: String,
    /// The bright or the dark 11th.
    pub tithi: Tithi,
    /// Its amanta month.
    pub month: Masa,
    /// Whether that month is adhika.
    pub adhika: bool,
    /// The 10th, the 11th and the 12th, whole.
    pub tithis: [Interval; 3],
    /// The 11th's own day and the day after (C181).
    pub days: [CalendarDate; 2],
    /// Where the 10th pierces the 11th's day, whatever the rule reckons:
    /// at sunrise, at arunodaya only, or nowhere.
    pub pierced_at: Option<Vedha>,
    /// Whether that pierces by the rule's vedha.
    pub pierced: bool,
    /// Which stand in excess.
    pub excess: Excess,
    /// The day the rule's table gives.
    pub choice: Which,
    /// The fast.
    pub day: CalendarDate,
    /// **How far the place can move**, in seconds of time, as an
    /// observance's [`margin_seconds`](super::Observance::margin_seconds):
    /// the least time between an edge of the 10th, the 11th or the 12th
    /// and an instant the fast was decided from. Those are the sunsets
    /// that give the 11th its own day, that day's sunrise (and its
    /// arunodaya, for a rule pierced there), and the two sunrises after
    /// it, which say what stands in excess.
    pub margin_seconds: f64,
    /// When it is broken, under the rule's parana; left out for a rule
    /// without one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parana: Option<Parana>,
}

/// Every Ekadashi over consecutive sunrise days, under each rule.
///
/// An 11th is judged when the days hold the day before its own and the
/// day after; the caller widens the run for a range's edges, as for
/// [`observances`](super::observances).
///
/// # Errors
///
/// A rule's own refusal ([`EkadashiRule::check`]).
pub fn ekadashis(rules: &[EkadashiRule], days: &[FestivalDay]) -> Result<Observances, Error> {
    for rule in rules {
        rule.check()?;
    }
    let mut answer = Observances::default();
    if rules.is_empty() {
        return Ok(answer);
    }
    let run = occurrences(days);
    for (at, eleventh) in run.iter().enumerate() {
        if !matches!(
            eleventh.member,
            Tithi::ShuklaEkadashi | Tithi::KrishnaEkadashi
        ) {
            continue;
        }
        let found = at
            .checked_sub(1)
            .and_then(|before| run.get(before))
            .zip(run.get(at + 1))
            .ok_or("the days do not hold the 10th and the 12th beside it")
            .and_then(|(tenth, twelfth)| {
                facts(
                    days,
                    [tenth.whole, eleventh.whole, twelfth.whole],
                    twelfth.member,
                )
            });
        for rule in rules {
            match found
                .as_ref()
                .map_err(|why| (*why).to_owned())
                .and_then(|facts| facts.fast(rule, eleventh.member))
            {
                Ok(fast) => answer.ekadashis.push(fast),
                Err(why) => answer.unjudged.push(Unjudged {
                    rule: rule.key.clone(),
                    tithi: eleventh.whole,
                    why,
                }),
            }
        }
    }
    Ok(answer)
}

/// What an 11th's fast is decided by, whoever keeps it.
struct Facts<'d> {
    tithis: [Interval; 3],
    /// The day before the 11th's own: its sunset is one the own day
    /// was chosen by.
    before: &'d FestivalDay,
    days: [&'d FestivalDay; 2],
    /// The day after the later, a parana's when the fast is the later's.
    after: Option<&'d FestivalDay>,
    /// The 12th's member, bright or dark.
    twelfth: Tithi,
    arunodaya: JulianDay<Utc>,
    pierced_at: Option<Vedha>,
    excess: Excess,
}

impl Facts<'_> {
    fn fast(&self, rule: &EkadashiRule, tithi: Tithi) -> Result<EkadashiFast, String> {
        // Pierced at sunrise is pierced at arunodaya, the earlier instant.
        let pierced = self.pierced_at.is_some_and(|at| at >= rule.vedha);
        let kinds = if pierced {
            rule.table.pierced
        } else {
            rule.table.pure
        };
        let choice = kinds.of(self.excess);
        let [first, second] = self.days;
        let chosen = match choice {
            Which::Earlier => first,
            Which::Later => second,
        };
        let parana = rule
            .parana
            .as_ref()
            .map(|parana| self.parana(parana, choice))
            .transpose()?;
        Ok(EkadashiFast {
            rule: rule.key.clone(),
            tithi,
            month: first.month.amanta,
            adhika: first.month.kind == MonthKind::Adhika,
            tithis: self.tithis,
            days: [first.date.clone(), second.date.clone()],
            pierced_at: self.pierced_at,
            pierced,
            excess: self.excess,
            choice,
            day: chosen.date.clone(),
            margin_seconds: self.margin(rule.vedha) * SECONDS_PER_DAY,
            parana,
        })
    }

    /// The parana of a fast kept on `choice`'s day (§8.5).
    fn parana(&self, rule: &ParanaRule, choice: Which) -> Result<Parana, String> {
        let day = match choice {
            Which::Earlier => self.days[1],
            Which::Later => self
                .after
                .ok_or("the days do not hold the day after the fast, its parana")?,
        };
        if !day.normal {
            return Err(format!("{} is a polar day, which has no windows", day.date));
        }
        let twelfth = self.tithis[2];
        let (tithi, held) = if twelfth.to.get() > day.sunrise.get() {
            (self.twelfth, twelfth)
        } else {
            let at = day
                .tithi
                .iter()
                .find(|span| span.whole.contains(day.sunrise))
                .ok_or_else(|| format!("no tithi holds {}'s sunrise", day.date))?;
            (at.member, at.whole)
        };
        let quarter = (twelfth.to.get() - twelfth.from.get()).mul_add(0.25, twelfth.from.get());
        let hari_vasara_ends = (quarter > day.sunrise.get()).then(|| JulianDay::literal(quarter));
        // What bars the start: the tithi's own, and the hari vasara's end
        // when the rule passes it over; the latest past the window's
        // start sets it.
        let mut bars = vec![(held.from, ParanaEdge::TithiStart)];
        if let Some(end) = hari_vasara_ends.filter(|_| rule.hari_vasara) {
            bars.push((end, ParanaEdge::HariVasara));
        }
        for &at in &rule.at {
            let span = day.window(at).map_err(|error| error.to_string())?;
            let (from, from_by) = bars
                .iter()
                .copied()
                .filter(|(bar, _)| bar.get() > span.from.get())
                .max_by(|a, b| a.0.get().total_cmp(&b.0.get()))
                .unwrap_or((span.from, ParanaEdge::Window));
            let (to, to_by) = if held.to.get() < span.to.get() {
                (held.to, ParanaEdge::TithiEnd)
            } else {
                (span.to, ParanaEdge::Window)
            };
            if from.get() < to.get() {
                return Ok(Parana {
                    day: day.date.clone(),
                    tithi,
                    hari_vasara_ends,
                    kept: Some(ParanaKept {
                        span: Interval::literal(from.get(), to.get()),
                        at,
                        from_by,
                        to_by,
                    }),
                });
            }
        }
        Ok(Parana {
            day: day.date.clone(),
            tithi,
            hari_vasara_ends,
            kept: None,
        })
    }

    /// The least time, in days, between an edge of the three tithis and
    /// an instant a rule pierced at `vedha` decides by.
    fn margin(&self, vedha: Vedha) -> f64 {
        let [tenth, eleventh, twelfth] = self.tithis;
        let [first, second] = self.days;
        let edges = [tenth.from, eleventh.from, eleventh.to, twelfth.to].map(JulianDay::get);
        let mut instants = vec![
            self.before.sunset,
            first.sunset,
            first.sunrise,
            second.sunrise,
            second.next_sunrise,
        ];
        if vedha == Vedha::Arunodaya {
            instants.push(self.arunodaya);
        }
        instants
            .iter()
            .flat_map(|instant| edges.map(|edge| (instant.get() - edge).abs()))
            .fold(f64::INFINITY, f64::min)
    }
}

const SECONDS_PER_DAY: f64 = 86_400.0;

/// The 11th's two days and what stands on them (§8.2, C181).
fn facts(
    days: &[FestivalDay],
    tithis: [Interval; 3],
    member: Tithi,
) -> Result<Facts<'_>, &'static str> {
    let [tenth, eleventh, twelfth] = tithis;
    // The day in whose daylight the 11th begins; one beginning at night
    // is the next day's. A kshaya 11th always begins in daylight, so this
    // is the day it runs in.
    let own = days
        .iter()
        .position(|day| day.sunset.get() > eleventh.from.get())
        .ok_or("the days do not hold the 11th's own day")?;
    let before = own
        .checked_sub(1)
        .and_then(|index| days.get(index))
        .ok_or("the days do not hold the day before the 11th's")?;
    let (first, second) = days
        .get(own)
        .zip(days.get(own + 1))
        .ok_or("the days do not hold the day after the 11th's")?;
    let arunodaya = arunodaya(before, first);
    let pierced_at = if tenth.contains(first.sunrise) {
        Some(Vedha::Sunrise)
    } else if tenth.contains(arunodaya) {
        Some(Vedha::Arunodaya)
    } else {
        None
    };
    let excess = Excess::of(
        eleventh.contains(second.sunrise),
        twelfth.contains(second.next_sunrise),
    );
    Ok(Facts {
        tithis,
        before,
        days: [first, second],
        after: days.get(own + 2),
        twelfth: member,
        arunodaya,
        pierced_at,
        excess,
    })
}

/// Four ghatis of the night before a sunrise, a ghati a thirtieth of the
/// night (C176): on a day of sixty, ghati 56.
fn arunodaya(before: &FestivalDay, day: &FestivalDay) -> JulianDay<Utc> {
    let night = day.sunrise.get() - before.sunset.get();
    JulianDay::literal((night / 30.0).mul_add(-4.0, day.sunrise.get()))
}
