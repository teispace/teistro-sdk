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
use teistro_core::catalogue::{Masa, Nakshatra, Tithi};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};

use super::{
    FestivalDay, MoonArcs, Observances, Unjudged, Which, Window, check_window, occurrences,
};

mod mahadvadashi;

pub use mahadvadashi::{
    JoinedNakshatra, Mahadvadashi, MahadvadashiKind, MahadvadashiRule, NakshatraJoining,
    TrisprishaAt, VyanjuliPast,
};

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
    /// The mahadvadashis that name its fast or move it onto the 12th's
    /// day (§8.9); a rule without one keeps its table's day.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mahadvadashi: Option<MahadvadashiRule>,
}

impl EkadashiRule {
    /// Refuses what could not name an answer: an empty key, a parana
    /// with no window or one outside the daylight, and a mahadvadashi
    /// rule keeping no kind or one kind twice.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `key`, on the parana's `at`, or on
    /// `mahadvadashi.kinds`.
    pub fn check(&self) -> Result<(), Error> {
        if self.key.trim().is_empty() {
            return Err(Error::invalid_arg(
                "an Ekadashi rule needs a key: an answer is named by it",
            )
            .with_field("key"));
        }
        if let Some(parana) = &self.parana {
            parana.check(&self.key)?;
        }
        self.mahadvadashi
            .as_ref()
            .map_or(Ok(()), |mahadvadashi| mahadvadashi.check(&self.key))
    }

    /// How many days past the 11th's own the rule reads: the fortnight's
    /// end and the sunrise after it, for a rule keeping mahadvadashis.
    #[must_use]
    pub const fn reach(&self) -> u8 {
        if self.mahadvadashi.is_some() { 6 } else { 0 }
    }

    /// The Gaudiya Vaishnavas' Ekadashi, after *Hari Bhakti Vilasa*: the
    /// 10th pierces at arunodaya (12.352–357), the pure 11th is left for
    /// growth ahead as the Vaishnava's table has it (12.359's
    /// commentary), and the eight mahadvadashis name the fast or move it
    /// (13.265–292). The parana keeps *Dharmasindhu*'s windows (Q41).
    ///
    /// ```
    /// use teistro_panchanga::festival::{EkadashiRule, MahadvadashiKind};
    ///
    /// let gaudiya = EkadashiRule::hari_bhakti_vilasa();
    /// assert_eq!(gaudiya.key, "EKADASHI_GAUDIYA");
    /// let kinds = &gaudiya.mahadvadashi.unwrap().kinds;
    /// assert_eq!(kinds.len(), MahadvadashiKind::ALL.len());
    /// ```
    #[must_use]
    pub fn hari_bhakti_vilasa() -> EkadashiRule {
        EkadashiRule {
            key: "EKADASHI_GAUDIYA".to_owned(),
            source: "Hari Bhakti Vilasa 12.352–394, 13.265–292: pierced at arunodaya the 11th is left; pure, it is left for growth ahead, the 12th's day fasted unless neither stands in excess; and the eight mahadvadashis, Unmilani, Vyanjuli, Trisprisha, Pakshavardhini, Jaya, Vijaya, Jayanti and Papanashini, name the fast or move it onto the 12th's day (C707 to C711)".to_owned(),
            vedha: Vedha::Arunodaya,
            table: VAISHNAVA,
            parana: Some(ParanaRule::dharmasindhu()),
            mahadvadashi: Some(MahadvadashiRule::hari_bhakti_vilasa()),
        }
    }

    /// The arcs of the Moon its parana's bars read, in sidereal degrees,
    /// which the days must carry ([`FestivalDay::moon_arcs`]).
    #[must_use]
    pub fn moon_arcs(&self) -> Vec<(Nakshatra, (f64, f64))> {
        self.parana
            .iter()
            .flat_map(|parana| &parana.bars)
            .map(|bar| (bar.nakshatra, bar.arc_deg()))
            .collect()
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
                table: VAISHNAVA,
                parana: Some(ParanaRule::dharmasindhu()),
                mahadvadashi: None,
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
                mahadvadashi: None,
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
                mahadvadashi: None,
            },
        ]
    }
}

/// The Vaishnava's table (*Dharmasindhu* pp. 11–12): pierced at
/// arunodaya, the 11th is left and the 12th's day fasted in every kind
/// (S1 to S8); pure, the later day unless neither stands in excess (V1 to
/// V4). *Hari Bhakti Vilasa* 12.359's commentary gives the same cells.
const VAISHNAVA: EkadashiTable = EkadashiTable {
    pure: EkadashiKinds {
        neither: Which::Earlier,
        ..EkadashiKinds::all(Which::Later)
    },
    pierced: EkadashiKinds::all(Which::Later),
};

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
    /// The nakshatras the meal is not taken in after one 11th's fast
    /// (C700): none in the default, as the published reference keeps
    /// none; [`NakshatraBar::dharmasindhu`] for the *Dharmasindhu*'s.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bars: Vec<NakshatraBar>,
}

/// A share of a nakshatra's arc: the `which`th of `parts` equal parts,
/// `{1, 1}` the whole, `{4, 1}` its first quarter (pada).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Portion {
    /// The equal parts the arc is cut in, 1 to 12.
    pub parts: u8,
    /// Which of them, 1 to `parts`.
    pub which: u8,
}

impl Portion {
    /// The whole arc.
    pub const WHOLE: Portion = Portion { parts: 1, which: 1 };

    /// This share of `nakshatra`'s arc in sidereal degrees, from its start
    /// to its end.
    ///
    /// ```
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_panchanga::festival::Portion;
    ///
    /// // Shravana's last quarter (Dharmasindhu p. 70), 290° to 293°20′.
    /// let (from, to) = Portion { parts: 4, which: 4 }.arc_deg(Nakshatra::Shravana);
    /// assert!((from - 290.0).abs() < 1e-9 && (to - 293.0 - 1.0 / 3.0).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn arc_deg(self, nakshatra: Nakshatra) -> (f64, f64) {
        let span = 360.0 / 27.0;
        let start = f64::from(nakshatra.id()) * span;
        let part = span / f64::from(self.parts);
        let from = part.mul_add(f64::from(self.which.saturating_sub(1)), start);
        (from, from + part)
    }

    /// Refuses a cut outside 1 to 12 parts or a part that does not exist,
    /// named under `field`.
    pub(crate) fn check(self, key: &str, field: &str) -> Result<(), Error> {
        let Portion { parts, which } = self;
        if !(1..=12).contains(&parts) {
            return Err(Error::invalid_arg(format!(
                "{key}: a nakshatra is cut in 1 to 12 parts, not {parts}"
            ))
            .with_field(format!("{field}.parts")));
        }
        if !(1..=parts).contains(&which) {
            return Err(Error::invalid_arg(format!(
                "{key}: part {which} of {parts} does not exist; count them from 1"
            ))
            .with_field(format!("{field}.which")));
        }
        Ok(())
    }
}

impl Default for Portion {
    fn default() -> Portion {
        Portion::WHOLE
    }
}

/// A nakshatra the meal is not taken in, on the parana after the fast of
/// one 11th in one month (C700).
///
/// ```
/// use teistro_core::catalogue::{Masa, Nakshatra};
/// use teistro_panchanga::festival::{NakshatraBar, Portion};
///
/// // Vishnu sleeps in Anuradha's first quarter (Dharmasindhu pp. 41–42).
/// let [ashadha, ..] = NakshatraBar::dharmasindhu();
/// assert_eq!((ashadha.month, ashadha.nakshatra), (Masa::Ashadha, Nakshatra::Anuradha));
/// assert_eq!(ashadha.portion, Portion { parts: 4, which: 1 });
/// let (from, to) = ashadha.arc_deg();
/// assert!((from - 213.0 - 1.0 / 3.0).abs() < 1e-9 && (to - 216.0 - 2.0 / 3.0).abs() < 1e-9);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct NakshatraBar {
    /// The 11th whose fast's parana it bars.
    pub tithi: Tithi,
    /// That 11th's amanta month. A bar holds in the nija month only: the
    /// rites it guards are not kept in an adhika one (*Dharmasindhu* p. 41).
    pub month: Masa,
    /// The nakshatra.
    pub nakshatra: Nakshatra,
    /// The share of its arc that bars; the whole when left out.
    #[serde(default, skip_serializing_if = "is_whole")]
    pub portion: Portion,
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde's skip_serializing_if passes a reference"
)]
fn is_whole(portion: &Portion) -> bool {
    *portion == Portion::WHOLE
}

impl NakshatraBar {
    /// The *Dharmasindhu*'s reading (pp. 41–42, 56–57 and 80–81, after the
    /// *Vishnudharma*): the bright 12ths of Ashadha, Bhadrapada and
    /// Kartika bar the first quarter of Anuradha, where Vishnu sleeps, the
    /// middle third of Shravana, where he turns, and the last quarter of
    /// Revati, where he wakes.
    #[must_use]
    pub const fn dharmasindhu() -> [NakshatraBar; 3] {
        [
            NakshatraBar {
                tithi: Tithi::ShuklaEkadashi,
                month: Masa::Ashadha,
                nakshatra: Nakshatra::Anuradha,
                portion: Portion { parts: 4, which: 1 },
            },
            NakshatraBar {
                tithi: Tithi::ShuklaEkadashi,
                month: Masa::Bhadrapada,
                nakshatra: Nakshatra::Shravana,
                portion: Portion { parts: 3, which: 2 },
            },
            NakshatraBar {
                tithi: Tithi::ShuklaEkadashi,
                month: Masa::Kartika,
                nakshatra: Nakshatra::Revati,
                portion: Portion { parts: 4, which: 4 },
            },
        ]
    }

    /// The verse's own reading (*Dharmasindhu* p. 81, note: "आभाका
    /// सितपक्षेषु मैत्रश्रवणरेवती"): the three nakshatras whole.
    #[must_use]
    pub const fn verse() -> [NakshatraBar; 3] {
        let [mut ashadha, mut bhadrapada, mut kartika] = NakshatraBar::dharmasindhu();
        ashadha.portion = Portion::WHOLE;
        bhadrapada.portion = Portion::WHOLE;
        kartika.portion = Portion::WHOLE;
        [ashadha, bhadrapada, kartika]
    }

    /// The barred arc in sidereal degrees, from its start to its end.
    #[must_use]
    pub fn arc_deg(&self) -> (f64, f64) {
        self.portion.arc_deg(self.nakshatra)
    }

    fn check(self, key: &str, index: usize) -> Result<(), Error> {
        let at = format!("parana.bars[{index}]");
        if !matches!(self.tithi, Tithi::ShuklaEkadashi | Tithi::KrishnaEkadashi) {
            return Err(Error::invalid_arg(format!(
                "{key}: a parana bar names the 11th whose fast it follows, not {:?}",
                self.tithi
            ))
            .with_field(format!("{at}.tithi")));
        }
        self.portion.check(key, &format!("{at}.portion"))
    }
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
            bars: Vec::new(),
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
        for (index, bar) in self.bars.iter().enumerate() {
            bar.check(key, index)?;
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
    /// An edge of the barred share of a nakshatra (C700).
    Nakshatra,
    /// The end of a mahadvadashi's joined nakshatra (HBV 13.290–291): the
    /// meal kept after it, or inside it.
    JoinedNakshatra,
}

/// A barred share of a nakshatra met on the parana day (C700).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ParanaBarred {
    /// The rule's bar.
    pub bar: NakshatraBar,
    /// The Moon's stay in the barred arc.
    pub span: Interval,
    /// Whether the bar was set aside, as the barred share outlasts the
    /// tithi the meal is kept in, so that keeping it would leave no meal
    /// inside the 12th (*Dharmasindhu* p. 42, after the *Kaustubha*).
    pub set_aside: bool,
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
    /// The rule's nakshatra bar, when one met the parana day.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub barred: Option<ParanaBarred>,
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
    /// The fast: `choice`'s day, or the day a mahadvadashi sets, which
    /// may be the day after both (the 12th's, when the 11th holds two
    /// sunrises).
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
    /// What a published entry did to it; absent when none touched it,
    /// so the day is the rule's (`03-design/published-overrides.md`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<super::Published>,
    /// The mahadvadashi that set `day`, when the rule keeps them and one
    /// held; `choice` stays the table's, so a reader sees what the table
    /// alone gave.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mahadvadashi: Option<Mahadvadashi>,
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
                // The fortnight's end, which a Pakshavardhini reads.
                let end = if eleventh.member == Tithi::ShuklaEkadashi {
                    Tithi::Purnima
                } else {
                    Tithi::Amavasya
                };
                let paksha_end = run
                    .iter()
                    .skip(at + 1)
                    .take(5)
                    .find(|held| held.member == end)
                    .map(|held| held.whole);
                facts(
                    days,
                    [tenth.whole, eleventh.whole, twelfth.whole],
                    twelfth.member,
                    paksha_end,
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
    /// The days from the 11th's own on, as far as they were founded: the
    /// parana's is the one after the fast's.
    run: &'d [FestivalDay],
    /// The fortnight's new or full moon, whole, when the days hold it.
    paksha_end: Option<Interval>,
    /// The 12th's member, bright or dark.
    twelfth: Tithi,
    arunodaya: JulianDay<Utc>,
    pierced_at: Option<Vedha>,
    excess: Excess,
}

impl<'d> Facts<'d> {
    /// The day `at` past the 11th's own.
    fn day(&self, at: usize) -> Result<&'d FestivalDay, String> {
        self.run.get(at).ok_or_else(|| {
            format!("the days do not reach the {at}th past the 11th's own, which the rule reads")
        })
    }

    /// The 12th's day, as an index past the 11th's own: the first whose
    /// sunrise the 12th holds; none for a 12th holding no sunrise.
    fn twelfth_day(&self) -> Result<Option<usize>, String> {
        let [_, eleventh, twelfth] = self.tithis;
        let second = self.days[1];
        if twelfth.contains(second.sunrise) {
            return Ok(Some(1));
        }
        if !eleventh.contains(second.sunrise) {
            return Ok(None);
        }
        Ok(twelfth.contains(self.day(2)?.sunrise).then_some(2))
    }

    /// Whether the fortnight's end holds two sunrises (HBV 13.270).
    fn paksha_end_doubled(&self) -> Result<bool, String> {
        let end = self
            .paksha_end
            .ok_or("the days do not hold the fortnight's end, which a Pakshavardhini reads")?;
        if self
            .run
            .iter()
            .any(|day| end.contains(day.sunrise) && end.contains(day.next_sunrise))
        {
            return Ok(true);
        }
        if self
            .run
            .last()
            .is_some_and(|last| last.next_sunrise.get() >= end.to.get())
        {
            Ok(false)
        } else {
            Err("the days do not reach the sunrise after the fortnight's end".to_owned())
        }
    }

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
        let held = rule
            .mahadvadashi
            .as_ref()
            .map(|mahadvadashi| mahadvadashi.judge(self))
            .transpose()?
            .flatten();
        let at = held.as_ref().map_or(
            match choice {
                Which::Earlier => 0,
                Which::Later => 1,
            },
            |held| held.at,
        );
        let chosen = self.day(at)?;
        let joined = held
            .as_ref()
            .and_then(|held| held.mahadvadashi.joined.as_ref());
        let parana = rule
            .parana
            .as_ref()
            .map(|parana| self.parana(parana, at, tithi, joined))
            .transpose()?;
        let margin = rule
            .mahadvadashi
            .as_ref()
            .map_or(f64::INFINITY, |mahadvadashi| mahadvadashi.margin(self))
            .min(self.margin(rule.vedha));
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
            margin_seconds: margin * SECONDS_PER_DAY,
            parana,
            mahadvadashi: held.map(|held| held.mahadvadashi),
            published: None,
        })
    }

    /// The parana of a fast on `eleventh`, kept on the day `at` past the
    /// 11th's own (§8.5), after or inside a mahadvadashi's `joined`
    /// nakshatra (HBV 13.290–291).
    fn parana(
        &self,
        rule: &ParanaRule,
        at: usize,
        eleventh: Tithi,
        joined: Option<&JoinedNakshatra>,
    ) -> Result<Parana, String> {
        let day = self
            .run
            .get(at + 1)
            .ok_or("the days do not hold the day after the fast, its parana")?;
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
        let mut closes = vec![(held.to, ParanaEdge::TithiEnd)];
        // 13.290–291, a nakshatra that grows into the parana day: with the
        // 12th standing, after it when the 12th outlasts it ("पारणं ततः")
        // and inside the 12th otherwise ("तिथिमध्ये"); with the 12th gone,
        // inside Rohini or Shravana ("तन्मध्ये") and after the other two
        // ("तदतिक्रमे").
        if let Some(joined) = joined.filter(|joined| joined.span.to.get() > day.sunrise.get()) {
            let end = (joined.span.to, ParanaEdge::JoinedNakshatra);
            if twelfth.to.get() > day.sunrise.get() {
                if twelfth.to.get() > joined.span.to.get() {
                    bars.push(end);
                }
            } else if matches!(joined.nakshatra, Nakshatra::Rohini | Nakshatra::Shravana) {
                closes.push(end);
            } else {
                bars.push(end);
            }
        }
        let mut barred = self.barred(rule, eleventh, day)?;
        let mut kept = kept_in(rule, day, &bars, &closes, barred.map(|barred| barred.span))?;
        // Kept, a bar whose share outlasts the tithi would leave no meal
        // inside it: the bar is set aside and the meal kept as though
        // there were none (*Dharmasindhu* p. 42, after the *Kaustubha*).
        if kept.is_none() {
            if let Some(barred) = barred
                .as_mut()
                .filter(|barred| barred.span.to.get() >= held.to.get())
            {
                barred.set_aside = true;
                kept = kept_in(rule, day, &bars, &closes, None)?;
            }
        }
        Ok(Parana {
            day: day.date.clone(),
            tithi,
            hari_vasara_ends,
            barred,
            kept,
        })
    }

    /// The rule's bar for a fast on `eleventh` in this nija month, when the
    /// Moon's stay in its arc meets `day`'s daylight.
    fn barred(
        &self,
        rule: &ParanaRule,
        eleventh: Tithi,
        day: &FestivalDay,
    ) -> Result<Option<ParanaBarred>, String> {
        let month = self.days[0].month;
        let Some(&bar) = rule.bars.iter().find(|bar| {
            bar.tithi == eleventh && bar.month == month.amanta && month.kind == MonthKind::Nija
        }) else {
            return Ok(None);
        };
        let MoonArcs::Found(arcs) = &day.moon_arcs else {
            return Err(format!(
                "a parana bar reads the Moon's arcs on {}, which no search found; fill them first (`Almanac::moon_arcs`)",
                day.date
            ));
        };
        let (from_deg, to_deg) = bar.arc_deg();
        let daylight = Interval::literal(day.sunrise.get(), day.sunset.get());
        Ok(arcs
            .iter()
            .filter(|arc| arc.is_of((from_deg, to_deg)))
            .find(|arc| {
                arc.span
                    .clipped_to(daylight)
                    .is_some_and(|both| both.days() > 0.0)
            })
            .map(|arc| ParanaBarred {
                bar,
                span: arc.span,
                set_aside: false,
            }))
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

/// The first of the rule's windows on `day` that leaves time past `bars`
/// (the latest after a window's start sets it) and before `closes` (the
/// earliest before its end, the end of the tithi the meal is kept in
/// among them), outside `barred`, a barred share of a nakshatra: the meal
/// waits past that share when it holds the start, and is kept before it
/// when it falls later in the window.
fn kept_in(
    rule: &ParanaRule,
    day: &FestivalDay,
    bars: &[(JulianDay<Utc>, ParanaEdge)],
    closes: &[(JulianDay<Utc>, ParanaEdge)],
    barred: Option<Interval>,
) -> Result<Option<ParanaKept>, String> {
    for &at in &rule.at {
        let span = day.window(at).map_err(|error| error.to_string())?;
        let (mut from, mut from_by) = bars
            .iter()
            .copied()
            .filter(|(bar, _)| bar.get() > span.from.get())
            .max_by(|a, b| a.0.get().total_cmp(&b.0.get()))
            .unwrap_or((span.from, ParanaEdge::Window));
        let (mut to, mut to_by) = closes
            .iter()
            .copied()
            .filter(|(close, _)| close.get() < span.to.get())
            .min_by(|a, b| a.0.get().total_cmp(&b.0.get()))
            .unwrap_or((span.to, ParanaEdge::Window));
        if let Some(bar) = barred {
            if bar.from.get() <= from.get() && from.get() < bar.to.get() {
                (from, from_by) = (bar.to, ParanaEdge::Nakshatra);
            } else if from.get() < bar.from.get() && bar.from.get() < to.get() {
                (to, to_by) = (bar.from, ParanaEdge::Nakshatra);
            }
        }
        if from.get() < to.get() {
            return Ok(Some(ParanaKept {
                span: Interval::literal(from.get(), to.get()),
                at,
                from_by,
                to_by,
            }));
        }
    }
    Ok(None)
}

/// The 11th's two days and what stands on them (§8.2, C181).
fn facts(
    days: &[FestivalDay],
    tithis: [Interval; 3],
    member: Tithi,
    paksha_end: Option<Interval>,
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
        run: days.get(own..).unwrap_or_default(),
        paksha_end,
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
