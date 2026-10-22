//! The eight mahadvadashis of *Hari Bhakti Vilasa* (`03-design/festival-rules.md`
//! §8.9): twelfths whose joining names a Gaudiya Ekadashi's fast, or moves
//! it off the 11th onto the 12th's day. Verses are Haridas Shastri's
//! edition, vol. 2 (Vrindavan 1986); "13.270" is vilasa 13, verse 270.
//!
//! Four arise by the tithis' joining and four by a bright 12th's
//! nakshatra (13.265–266). Where the text leaves a measure open the rule
//! carries a parameter, defaulting to the reading every published list
//! follows (C707 to C711).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Nakshatra, Tithi};
use teistro_core::error::Error;
use teistro_core::interval::Interval;

use super::{Facts, arunodaya};

/// The eight great twelfths (HBV 13.265: "उन्मीलनी वञ्जुली च त्रिस्पृशा
/// पक्षवर्धिनी । जया च विजया चैव जयन्ती पापनाशिनी").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MahadvadashiKind {
    /// 13.267: a *sampūrṇā* 11th grows into the next day and the 12th
    /// does not; the fast is that next day.
    Unmilani,
    /// 13.268: the 12th alone grows past the sunrise after its own.
    Vyanjuli,
    /// 13.269: the 11th at the day's start, the 12th the whole day, the
    /// 13th at its end.
    Trisprisha,
    /// 13.270: the fortnight's new or full moon holds two sunrises; the
    /// 11th is left and the 12th fasted.
    Pakshavardhini,
    /// 13.485: a bright 12th joined with Punarvasu.
    Jaya,
    /// 13.489–490: with Shravana.
    Vijaya,
    /// 13.506, 13.508: with Rohini, *prājāpatya*.
    Jayanti,
    /// 13.534: with Pushya.
    Papanashini,
}

impl MahadvadashiKind {
    /// Every kind, in the order of 13.265.
    pub const ALL: [MahadvadashiKind; 8] = [
        MahadvadashiKind::Unmilani,
        MahadvadashiKind::Vyanjuli,
        MahadvadashiKind::Trisprisha,
        MahadvadashiKind::Pakshavardhini,
        MahadvadashiKind::Jaya,
        MahadvadashiKind::Vijaya,
        MahadvadashiKind::Jayanti,
        MahadvadashiKind::Papanashini,
    ];

    /// The nakshatra a bright 12th is joined with, for the four kinds
    /// that arise by one.
    ///
    /// ```
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_panchanga::festival::MahadvadashiKind;
    ///
    /// assert_eq!(MahadvadashiKind::Vijaya.nakshatra(), Some(Nakshatra::Shravana));
    /// assert_eq!(MahadvadashiKind::Unmilani.nakshatra(), None);
    /// ```
    #[must_use]
    pub const fn nakshatra(self) -> Option<Nakshatra> {
        match self {
            MahadvadashiKind::Jaya => Some(Nakshatra::Punarvasu),
            MahadvadashiKind::Vijaya => Some(Nakshatra::Shravana),
            MahadvadashiKind::Jayanti => Some(Nakshatra::Rohini),
            MahadvadashiKind::Papanashini => Some(Nakshatra::Pushya),
            MahadvadashiKind::Unmilani
            | MahadvadashiKind::Vyanjuli
            | MahadvadashiKind::Trisprisha
            | MahadvadashiKind::Pakshavardhini => None,
        }
    }

    /// The verse that defines it, as an answer quotes it.
    #[must_use]
    pub const fn clause(self) -> &'static str {
        match self {
            MahadvadashiKind::Unmilani => {
                "HBV 13.267: एकादशी तु सम्पूर्णा वर्धते पुनरेव सा । द्वादशी च न वर्धेत कथितोन्मीलनीति सा"
            }
            MahadvadashiKind::Vyanjuli => "HBV 13.268: द्वादश्येव विवर्धेत न चैवैकादशी यदा",
            MahadvadashiKind::Trisprisha => {
                "HBV 13.269: अरुणोदय आद्या स्याद्द्वादशी सकलं दिनम् । अन्ते त्रयोदशी प्रातस्त्रिस्पृशा सा हरेः प्रिया"
            }
            MahadvadashiKind::Pakshavardhini => {
                "HBV 13.270: कुहूराके यदा वृद्धिः प्रयाते पक्षवर्धिनी । विहायैकादशीं तत्र द्वादशीं समुपोषयेत्"
            }
            MahadvadashiKind::Jaya => {
                "HBV 13.485: द्वादश्यां तु सिते पक्षे ऋक्षं यदि पुनर्वसुः । नाम्ना सा तु जया ख्याता"
            }
            MahadvadashiKind::Vijaya => {
                "HBV 13.489: यदा तु शुक्लद्वादश्यां नक्षत्रं श्रवणं भवेत् । विजया सा तिथिः प्रोक्ता"
            }
            MahadvadashiKind::Jayanti => {
                "HBV 13.506: यदा तु शुक्लद्वादश्यां प्राजापत्यं प्रजायते । जयन्ती नाम सा"
            }
            MahadvadashiKind::Papanashini => {
                "HBV 13.534: यदा तु शुक्लद्वादश्यां पुष्या भवति कर्हिचित् । तदा सा तु महापुण्या कथिता पापनाशिनी"
            }
        }
    }
}

/// Where the 11th must stand on a Trisprisha's day (C707).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrisprishaAt {
    /// At arunodaya and at sunrise: the Hindi gloss of 13.269, and every
    /// published Trisprisha row.
    Sunrise,
    /// At arunodaya, even when it ends before sunrise: 13.269's letter,
    /// "अरुणोदय आद्या".
    Arunodaya,
}

/// What 13.286–287's nakshatra "longer, equal or shorter" is measured
/// against (C708). Every reading keeps 13.288–289: a nakshatra other than
/// Shravana needs the 12th to last to sunset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NakshatraJoining {
    /// The day: it holds the 12th's sunrise and the next.
    TwoSunrises,
    /// The tithi: it holds the 12th's sunrise and lasts to the 12th's end
    /// or past it (the Hindi gloss).
    OutlastsTwelfth,
    /// As `OUTLASTS_TWELFTH`, or it begins after the 12th's sunrise while
    /// the 12th stands, of any length: 13.286's "from sunrise".
    BegunInDaylight,
}

/// How much of the 12th must stand past the sunrise after its own for a
/// Vyanjuli (C709).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VyanjuliPast {
    /// Any: "a kala or even a vikala" (12.374, कला च विकलापि वा).
    Any,
    /// Half a muhurta, a ghati of that day's daylight: 13.356's
    /// "त्रयोदश्यां मुहूर्तार्धं".
    HalfMuhurta,
}

/// Which mahadvadashis a Gaudiya Ekadashi keeps, and how each is read.
/// Every field defaults to [`MahadvadashiRule::hari_bhakti_vilasa`]'s, so
/// `{}` is the text's rule.
///
/// ```
/// use teistro_panchanga::festival::{MahadvadashiKind, MahadvadashiRule};
///
/// let rule: MahadvadashiRule = serde_json::from_str(r#"{"kinds": ["PAKSHAVARDHINI"]}"#).unwrap();
/// assert_eq!(rule.kinds, [MahadvadashiKind::Pakshavardhini]);
/// assert!(rule.sampurna);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct MahadvadashiRule {
    /// The kinds kept, the first that holds setting the fast (C711); a
    /// kind left out is not kept.
    pub kinds: Vec<MahadvadashiKind>,
    /// Where the 11th stands on a Trisprisha's day.
    pub trisprisha: TrisprishaAt,
    /// What a nakshatra kind's joining is measured against.
    pub joining: NakshatraJoining,
    /// How much of the 12th a Vyanjuli needs past the second sunrise.
    pub vyanjuli: VyanjuliPast,
    /// Whether Unmilani and Vyanjuli need the 11th pure at arunodaya,
    /// *sampūrṇā* (13.356, C710).
    pub sampurna: bool,
}

impl Default for MahadvadashiRule {
    fn default() -> MahadvadashiRule {
        MahadvadashiRule::hari_bhakti_vilasa()
    }
}

impl MahadvadashiRule {
    /// HBV 13.265–292 read as every published list reads it: the 11th at
    /// sunrise on a Trisprisha's day, a nakshatra joined for two
    /// sunrises, any of the 12th past a Vyanjuli's second sunrise, a
    /// *sampūrṇā* 11th, and the 12th's day before Unmilani's when two
    /// hold (13.270: "विहायैकादशीं … द्वादशीं समुपोषयेत्").
    #[must_use]
    pub fn hari_bhakti_vilasa() -> MahadvadashiRule {
        use MahadvadashiKind::{
            Jaya, Jayanti, Pakshavardhini, Papanashini, Trisprisha, Unmilani, Vijaya, Vyanjuli,
        };
        MahadvadashiRule {
            kinds: vec![
                Vijaya,
                Jaya,
                Jayanti,
                Papanashini,
                Vyanjuli,
                Pakshavardhini,
                Unmilani,
                Trisprisha,
            ],
            trisprisha: TrisprishaAt::Sunrise,
            joining: NakshatraJoining::TwoSunrises,
            vyanjuli: VyanjuliPast::Any,
            sampurna: true,
        }
    }

    /// Refuses no kind, which would keep nothing, and a kind named twice,
    /// whose order would be two precedences.
    pub(super) fn check(&self, key: &str) -> Result<(), Error> {
        if self.kinds.is_empty() {
            return Err(Error::invalid_arg(format!(
                "{key}: a mahadvadashi rule keeps at least one kind; leave the rule out to keep none"
            ))
            .with_field("mahadvadashi.kinds"));
        }
        for (index, kind) in self.kinds.iter().enumerate() {
            if self.kinds.iter().take(index).any(|held| held == kind) {
                return Err(Error::invalid_arg(format!(
                    "{key}: {kind:?} is named twice; the kinds are an order of precedence"
                ))
                .with_field(format!("mahadvadashi.kinds[{index}]")));
            }
        }
        Ok(())
    }

    /// The kinds that hold over an 11th, and the index into the 11th's
    /// run of days of the fast the first sets.
    pub(super) fn judge(&self, facts: &Facts<'_>) -> Result<Option<Held>, String> {
        let mut held = Vec::new();
        let mut first = None;
        for &kind in &self.kinds {
            if let Some(found) = self.holds(kind, facts)? {
                first.get_or_insert(found);
                held.push(kind);
            }
        }
        Ok(first
            .zip(held.first().copied())
            .map(|((at, joined), kind)| Held {
                at,
                mahadvadashi: Mahadvadashi {
                    kind,
                    held,
                    clause: kind.clause().to_owned(),
                    joined,
                },
            }))
    }

    /// Whether `kind` holds, with the fast's index and its joined
    /// nakshatra.
    fn holds(
        &self,
        kind: MahadvadashiKind,
        facts: &Facts<'_>,
    ) -> Result<Option<(usize, Option<JoinedNakshatra>)>, String> {
        let [_, eleventh, twelfth] = facts.tithis;
        let [first, second] = facts.days;
        let sampurna = !self.sampurna || facts.pierced_at.is_none();
        Ok(match kind {
            MahadvadashiKind::Unmilani => {
                let grows = sampurna
                    && eleventh.contains(first.sunrise)
                    && eleventh.contains(second.sunrise);
                if grows {
                    let after = facts.day(2)?;
                    let twelfth_grows =
                        twelfth.contains(after.sunrise) && twelfth.contains(after.next_sunrise);
                    (!twelfth_grows).then_some((1, None))
                } else {
                    None
                }
            }
            MahadvadashiKind::Vyanjuli => {
                let grows = sampurna
                    && eleventh.contains(first.sunrise)
                    && !eleventh.contains(second.sunrise)
                    && twelfth.contains(second.sunrise)
                    && twelfth.contains(second.next_sunrise);
                let enough = grows
                    && match self.vyanjuli {
                        VyanjuliPast::Any => true,
                        VyanjuliPast::HalfMuhurta => {
                            let after = facts.day(2)?;
                            twelfth.to.get() - after.sunrise.get() >= ghati_of(after)
                        }
                    };
                enough.then_some((1, None))
            }
            MahadvadashiKind::Trisprisha => [(facts.before, first), (first, second)]
                .into_iter()
                .position(|(before, day)| {
                    let opens = eleventh.contains(arunodaya(before, day))
                        && (self.trisprisha == TrisprishaAt::Arunodaya
                            || eleventh.contains(day.sunrise));
                    opens && twelfth.to.get() < day.next_sunrise.get()
                })
                .map(|at| (at, None)),
            MahadvadashiKind::Pakshavardhini => match facts.twelfth_day()? {
                Some(at) if facts.paksha_end_doubled()? => Some((at, None)),
                _ => None,
            },
            MahadvadashiKind::Jaya
            | MahadvadashiKind::Vijaya
            | MahadvadashiKind::Jayanti
            | MahadvadashiKind::Papanashini => {
                let Some(nakshatra) = kind.nakshatra() else {
                    return Ok(None);
                };
                if facts.twelfth != Tithi::ShuklaDwadashi {
                    return Ok(None);
                }
                let Some(at) = facts.twelfth_day()? else {
                    return Ok(None);
                };
                let day = facts.day(at)?;
                // 13.288: the 12th to sunset; 13.289: not for Shravana.
                if nakshatra != Nakshatra::Shravana && !twelfth.contains(day.sunset) {
                    return Ok(None);
                }
                day.nakshatra
                    .iter()
                    .filter(|span| span.member == nakshatra)
                    .map(|span| span.whole)
                    .find(|&stay| {
                        let at_sunrise = stay.contains(day.sunrise);
                        match self.joining {
                            NakshatraJoining::TwoSunrises => {
                                at_sunrise && stay.contains(day.next_sunrise)
                            }
                            NakshatraJoining::OutlastsTwelfth => {
                                at_sunrise && stay.to.get() >= twelfth.to.get()
                            }
                            NakshatraJoining::BegunInDaylight => {
                                (at_sunrise && stay.to.get() >= twelfth.to.get())
                                    || (stay.from.get() > day.sunrise.get()
                                        && stay.from.get() < twelfth.to.get())
                            }
                        }
                    })
                    .map(|span| (at, Some(JoinedNakshatra { nakshatra, span })))
            }
        })
    }

    /// The least time, in days, between an edge a kept kind reads and an
    /// instant it is read at; infinite when none is read.
    pub(super) fn margin(&self, facts: &Facts<'_>) -> f64 {
        let [_, eleventh, twelfth] = facts.tithis;
        let mut pairs: Vec<(f64, f64)> = Vec::new();
        let mut near = |edges: &[f64], instants: &[f64]| {
            for &edge in edges {
                pairs.extend(instants.iter().map(|&instant| (edge, instant)));
            }
        };
        let rises: Vec<f64> = facts.run.iter().map(|day| day.sunrise.get()).collect();
        for &kind in &self.kinds {
            match kind {
                MahadvadashiKind::Unmilani | MahadvadashiKind::Vyanjuli => {
                    near(&[twelfth.to.get()], &rises);
                }
                MahadvadashiKind::Trisprisha => {
                    let [first, second] = facts.days;
                    near(
                        &[eleventh.to.get()],
                        &[arunodaya(first, second).get(), second.sunrise.get()],
                    );
                }
                MahadvadashiKind::Pakshavardhini => {
                    if let Some(end) = facts.paksha_end {
                        near(&[end.from.get(), end.to.get()], &rises);
                    }
                }
                MahadvadashiKind::Jaya
                | MahadvadashiKind::Vijaya
                | MahadvadashiKind::Jayanti
                | MahadvadashiKind::Papanashini => {
                    let day = facts
                        .twelfth_day()
                        .ok()
                        .flatten()
                        .and_then(|at| facts.run.get(at));
                    if let (Some(day), Some(nakshatra)) = (day, kind.nakshatra()) {
                        near(&[twelfth.to.get()], &[day.sunset.get()]);
                        for span in day.nakshatra.iter().filter(|span| span.member == nakshatra) {
                            near(
                                &[span.whole.from.get(), span.whole.to.get()],
                                &[day.sunrise.get(), day.next_sunrise.get(), twelfth.to.get()],
                            );
                        }
                    }
                }
            }
        }
        if self.vyanjuli == VyanjuliPast::HalfMuhurta {
            if let Some(after) = facts.run.get(2) {
                pairs.push((twelfth.to.get() - after.sunrise.get(), ghati_of(after)));
            }
        }
        pairs
            .iter()
            .map(|(edge, instant)| (edge - instant).abs())
            .fold(f64::INFINITY, f64::min)
    }
}

/// A ghati of a day's daylight, half a muhurta.
fn ghati_of(day: &super::FestivalDay) -> f64 {
    (day.sunset.get() - day.sunrise.get()) / 30.0
}

/// A mahadvadashi that held, and the fast it set.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Mahadvadashi {
    /// The kind that set the fast: the first of the rule's kinds that
    /// held.
    pub kind: MahadvadashiKind,
    /// Every kind that held, in the rule's order.
    pub held: Vec<MahadvadashiKind>,
    /// The verse that defines `kind`.
    pub clause: String,
    /// The joined nakshatra's stay, for the four nakshatra kinds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joined: Option<JoinedNakshatra>,
}

/// The nakshatra a bright 12th is joined with, and its stay.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct JoinedNakshatra {
    /// Punarvasu, Shravana, Rohini or Pushya.
    pub nakshatra: Nakshatra,
    /// Its whole stay.
    pub span: Interval,
}

/// What [`MahadvadashiRule::judge`] found: the answer and where its fast
/// falls in the 11th's run of days.
pub(super) struct Held {
    pub(super) at: usize,
    pub(super) mahadvadashi: Mahadvadashi,
}
