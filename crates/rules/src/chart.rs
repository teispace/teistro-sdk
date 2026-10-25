//! What a rule reads of a chart, and the choices the language leaves open.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{
    CharaKaraka, Dignity, Graha, Karana, Nakshatra, Point, Rashi, Tithi, Vara, Varga, Yoga,
};
use teistro_core::settings::NodeAspects;

use crate::language::{Body, House, Pada};
use crate::table::SignDegree;
use teistro_state::dignity;

/// The signs the grahas stand in at one instant, as a `transit-in-house`
/// condition reads them against the natal chart: the sky of gochar.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_rules::TransitSky;
///
/// let sky = TransitSky::new([(Graha::Saturn, Rashi::Pisces)]);
/// assert_eq!(sky.sign(Graha::Saturn), Some(Rashi::Pisces));
/// assert_eq!(sky.sign(Graha::Jupiter), None);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransitSky {
    signs: Vec<(Graha, Rashi)>,
}

impl TransitSky {
    /// A sky of these grahas in these signs; a graha given twice keeps the
    /// last.
    ///
    /// ```
    /// use teistro_core::catalogue::{Graha, Rashi};
    /// use teistro_rules::TransitSky;
    ///
    /// // A graha given twice keeps the last.
    /// let sky = TransitSky::new([(Graha::Saturn, Rashi::Aquarius), (Graha::Saturn, Rashi::Pisces)]);
    /// assert_eq!(sky.sign(Graha::Saturn), Some(Rashi::Pisces));
    /// ```
    #[must_use]
    pub fn new(signs: impl IntoIterator<Item = (Graha, Rashi)>) -> TransitSky {
        let mut sky = TransitSky::default();
        for (graha, sign) in signs {
            sky.signs.retain(|(held, _)| *held != graha);
            sky.signs.push((graha, sign));
        }
        sky
    }

    /// The sign a graha stands in, when the sky holds it.
    ///
    /// ```
    /// use teistro_core::catalogue::{Graha, Rashi};
    /// use teistro_rules::TransitSky;
    ///
    /// let sky = TransitSky::new([(Graha::Jupiter, Rashi::Taurus)]);
    /// assert_eq!(sky.sign(Graha::Jupiter), Some(Rashi::Taurus));
    /// assert_eq!(sky.sign(Graha::Saturn), None);
    /// ```
    #[must_use]
    pub fn sign(&self, graha: Graha) -> Option<Rashi> {
        self.signs
            .iter()
            .find_map(|(held, sign)| (*held == graha).then_some(*sign))
    }
}

/// One body as a rule reads it.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// use teistro_core::catalogue::Graha;
///
/// // Jupiter carried to the middle of Cancer, the fourth from Aries.
/// place(&mut chart, Body::Graha(Graha::Jupiter), Rashi::Cancer);
/// let jupiter = chart.placement(Body::Graha(Graha::Jupiter));
/// assert_eq!((jupiter.sign, jupiter.house, jupiter.longitude), (Rashi::Cancer, House::try_new(4)?, 105.0));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    /// Its sidereal longitude, degrees.
    pub longitude: f64,
    /// Its sign.
    pub sign: Rashi,
    /// The house it is recorded in.
    pub house: House,
    /// Its dignity.
    pub dignity: Dignity,
    /// Whether it is retrograde.
    pub retrograde: bool,
    /// Whether the Sun burns it at all.
    pub combust: bool,
    /// Its chara karaka among seven, when it holds one.
    pub karaka7: Option<CharaKaraka>,
    /// Its chara karaka among eight, when it holds one.
    pub karaka8: Option<CharaKaraka>,
    /// Its navamsha sign, under whichever scheme the chart's vargas were
    /// computed.
    pub navamsha: Rashi,
}

/// What a chart says of each body's strength.
///
/// The texts ask for strength constantly — "while the ascendant lord is
/// strong" (BPHS ch. 36), "if the Argala causing planet is stronger than the
/// obstructing one" (ch. 31 v. 4) — and give one measure for it, the six-fold
/// strength of ch. 27, with a requirement for each graha in vv. 32 and 33.
/// The kernel compares the numbers it is given and does not compute them: what
/// measure they are in, and what reading of the requirement they were taken
/// under (crux C71), belong to whoever builds the chart, and `measure` records
/// which it was so a reader knows what was compared.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// # use teistro_rules::{Condition, Evaluator, Participants, Readings};
/// #
/// # let holds = |chart: &RuleChart, readings: Readings, condition: &str| {
/// #     let condition: Condition = serde_json::from_str(condition).unwrap();
/// #     Evaluator::new(chart, readings).holds(&condition, &mut Participants::default())
/// # };
///
/// use teistro_rules::{StrengthMeasure, Strengths};
///
/// // Every body at 6 against a requirement of 5, the lagna and nodes unmeasured.
/// let mut of = [Some(6.0); 10];
/// of[7..].fill(None);
/// let strengths = Strengths { measure: StrengthMeasure::Shadbala, of, required: [Some(5.0); 10] };
///
/// let strong = r#"{"type": "planet-strong", "planet": "JUPITER"}"#;
/// assert!(!holds(&chart, Readings::TEXTS, strong));
/// chart.strengths = Some(strengths);
/// assert!(holds(&chart, Readings::TEXTS, strong));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strengths {
    /// Which measure the numbers are in.
    pub measure: StrengthMeasure,
    /// Each body's strength, in [`Body::ALL`](crate::Body::ALL)'s order; none
    /// where the measure does not reach it, as Shadbala does not reach the
    /// nodes or the lagna.
    pub of: [Option<f64>; 10],
    /// What each body must reach to be called strong, in the same unit and the
    /// same order.
    pub required: [Option<f64>; 10],
}

impl Strengths {
    /// A chart that carries no strength at all, which every rule that asks for
    /// one answers false on.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_rules::{Body, Strengths};
    ///
    /// // Silent: neither strong nor weak.
    /// let sun = Body::Graha(Graha::Sun);
    /// assert!(!Strengths::NONE.is_strong(sun) && !Strengths::NONE.is_weak(sun));
    /// ```
    pub const NONE: Strengths = Strengths {
        measure: StrengthMeasure::Shadbala,
        of: [None; 10],
        required: [None; 10],
    };

    /// Whether a body reaches what is required of it; false where either
    /// number is missing.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_rules::Body;
    /// use teistro_rules::{StrengthMeasure, Strengths};
    ///
    /// // Every body at 6 against a requirement of 5, the lagna and nodes unmeasured.
    /// let mut of = [Some(6.0); 10];
    /// of[7..].fill(None);
    /// let strengths = Strengths { measure: StrengthMeasure::Shadbala, of, required: [Some(5.0); 10] };
    ///
    /// assert!(strengths.is_strong(Body::Graha(Graha::Saturn)));
    /// // The nodes are not measured.
    /// assert!(!strengths.is_strong(Body::Graha(Graha::Rahu)));
    /// ```
    #[must_use]
    pub fn is_strong(&self, body: Body) -> bool {
        match (self.of.get(body.index()), self.required.get(body.index())) {
            (Some(Some(strength)), Some(Some(required))) => strength >= required,
            _ => false,
        }
    }

    /// Whether a body falls short of what is required of it; false where
    /// either number is missing, so "not strong" and "weak" are different
    /// questions on a chart that says nothing.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_rules::Body;
    /// use teistro_rules::{StrengthMeasure, Strengths};
    ///
    /// // Every body at 6 against a requirement of 5, the lagna and nodes unmeasured.
    /// let mut of = [Some(6.0); 10];
    /// of[7..].fill(None);
    /// let strengths = Strengths { measure: StrengthMeasure::Shadbala, of, required: [Some(5.0); 10] };
    ///
    /// let mut weak = strengths;
    /// weak.of[0] = Some(4.5);
    /// assert!(weak.is_weak(Body::Graha(Graha::Sun)));
    /// assert!(!weak.is_weak(Body::Graha(Graha::Rahu)) && !weak.is_strong(Body::Graha(Graha::Rahu)));
    /// ```
    #[must_use]
    pub fn is_weak(&self, body: Body) -> bool {
        match (self.of.get(body.index()), self.required.get(body.index())) {
            (Some(Some(strength)), Some(Some(required))) => strength < required,
            _ => false,
        }
    }

    /// Whether one body's strength exceeds another's; false where either is
    /// missing.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_rules::Body;
    /// use teistro_rules::{StrengthMeasure, Strengths};
    ///
    /// // Every body at 6 against a requirement of 5, the lagna and nodes unmeasured.
    /// let mut of = [Some(6.0); 10];
    /// of[7..].fill(None);
    /// let strengths = Strengths { measure: StrengthMeasure::Shadbala, of, required: [Some(5.0); 10] };
    ///
    /// let mut stronger = strengths;
    /// stronger.of[4] = Some(7.0);
    /// assert!(stronger.exceeds(Body::Graha(Graha::Jupiter), Body::Graha(Graha::Venus)));
    /// assert!(!stronger.exceeds(Body::Graha(Graha::Venus), Body::Graha(Graha::Venus)));
    /// ```
    #[must_use]
    pub fn exceeds(&self, one: Body, other: Body) -> bool {
        match (self.of.get(one.index()), self.of.get(other.index())) {
            (Some(Some(one)), Some(Some(other))) => one > other,
            _ => false,
        }
    }
}

/// Which measure a chart's strengths are in.
///
/// ```
/// use teistro_rules::{StrengthMeasure, Strengths};
///
/// assert_eq!(Strengths::NONE.measure, StrengthMeasure::Shadbala);
/// let own = Strengths { measure: StrengthMeasure::Caller, ..Strengths::NONE };
/// assert_ne!(own.measure, Strengths::NONE.measure);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StrengthMeasure {
    /// The six-fold strength of BPHS ch. 27, in rupas, against the requirement
    /// its vv. 32 and 33 give each graha.
    Shadbala,
    /// A measure the caller names in its own documentation. The kernel
    /// compares the numbers and says no more about them.
    Caller,
}

/// A chart as the rules read it: the nine grahas and the lagna, in
/// [`Body::ALL`]'s order, and the tithi of the birth when it is known.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// # use teistro_rules::{Condition, Evaluator, Participants, Readings};
/// #
/// # let holds = |chart: &RuleChart, readings: Readings, condition: &str| {
/// #     let condition: Condition = serde_json::from_str(condition).unwrap();
/// #     Evaluator::new(chart, readings).holds(&condition, &mut Participants::default())
/// # };
///
/// assert_eq!(chart.lagna(), Rashi::Aries);
/// assert!(holds(&chart, Readings::TEXTS, r#"{"type": "planet-in-kendra", "planet": "JUPITER"}"#));
/// // A rule reading the panchanga answers false on a chart without one.
/// assert!(!holds(&chart, Readings::TEXTS, r#"{"type": "panchanga-vara", "varas": ["SHANIVARA"]}"#));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuleChart {
    /// Each body's placement, the Sun to Ketu and then the lagna.
    pub placements: [Placement; 10],
    /// The panchanga at birth, which a rule reading one needs
    /// ([`Rule::reads_panchanga`](crate::Rule::reads_panchanga)).
    pub panchanga: Option<Panchanga>,
    /// What the chart says of each body's strength, when it says anything; a
    /// rule that asks whether a body is strong answers false without it.
    pub strengths: Option<Strengths>,
}

/// The panchanga at birth, as the rules read it.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// # use teistro_rules::{Condition, Evaluator, Participants, Readings};
/// #
/// # let holds = |chart: &RuleChart, readings: Readings, condition: &str| {
/// #     let condition: Condition = serde_json::from_str(condition).unwrap();
/// #     Evaluator::new(chart, readings).holds(&condition, &mut Participants::default())
/// # };
///
/// # use teistro_core::catalogue::{Karana, Nakshatra, Tithi, Vara, Yoga};
/// # use teistro_rules::{Pada, Panchanga, Span, Spans};
/// #
/// # let panchanga = Panchanga {
/// #     tithi: Tithi::ShuklaDvitiya,
/// #     vara: Vara::Shanivara,
/// #     nakshatra: Nakshatra::Ashwini,
/// #     pada: Pada::try_new(1)?,
/// #     yoga: Yoga::Vishkambha,
/// #     karana: Karana::Bava,
/// #     spans: Spans {
/// #         tithi: Some(Span { elapsed: 10.0, remaining: 50.0 }),
/// #         nakshatra: None,
/// #         lagna: None,
/// #         night: None,
/// #     },
/// #     by_day: Some(true),
/// #     sankranti_hours: None,
/// #     eclipse: None,
/// # };
///
/// chart.panchanga = Some(panchanga);
/// assert!(holds(&chart, Readings::TEXTS, r#"{"type": "panchanga-vara", "varas": ["SHANIVARA"]}"#));
/// assert!(holds(&chart, Readings::TEXTS, r#"{"type": "birth-by-day"}"#));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Panchanga {
    /// The tithi, whose paksha it carries.
    pub tithi: Tithi,
    /// The weekday.
    pub vara: Vara,
    /// The Moon's nakshatra.
    pub nakshatra: Nakshatra,
    /// The Moon's pada in it.
    pub pada: Pada,
    /// The panchanga yoga.
    pub yoga: Yoga,
    /// The karana.
    pub karana: Karana,
    /// How much of each limb the birth had used and had left, in ghatikas,
    /// which the gandantas of BPHS ch. 92 are measured in.
    pub spans: Spans,
    /// Whether the birth fell between sunrise and sunset, which BPHS ch. 10
    /// v. 5 reads beside the paksha; none when the chart does not say.
    pub by_day: Option<bool>,
    /// The hours from the sankranti inside the birth's day, sunrise to
    /// sunrise, to the birth: negative for a birth before it. None when the
    /// day has no sankranti, or the chart does not say.
    pub sankranti_hours: Option<f64>,
    /// The eclipse the birth falls in, if any.
    pub eclipse: Option<Eclipse>,
}

/// How far the birth stood into each limb, and how much of it was left, in
/// ghatikas of 24 minutes. A limb the chart does not measure is none, and a
/// condition reading its edge does not hold.
///
/// ```
/// # use teistro_core::catalogue::{Karana, Nakshatra, Tithi, Vara, Yoga};
/// # use teistro_rules::{Pada, Panchanga, Span, Spans};
/// #
/// # let panchanga = Panchanga {
/// #     tithi: Tithi::ShuklaDvitiya,
/// #     vara: Vara::Shanivara,
/// #     nakshatra: Nakshatra::Ashwini,
/// #     pada: Pada::try_new(1)?,
/// #     yoga: Yoga::Vishkambha,
/// #     karana: Karana::Bava,
/// #     spans: Spans {
/// #         tithi: Some(Span { elapsed: 10.0, remaining: 50.0 }),
/// #         nakshatra: None,
/// #         lagna: None,
/// #         night: None,
/// #     },
/// #     by_day: Some(true),
/// #     sankranti_hours: None,
/// #     eclipse: None,
/// # };
///
/// use teistro_rules::Limb;
///
/// assert_eq!(panchanga.spans.of(Limb::Tithi).and_then(|span| span.part(6)), Some(2));
/// assert_eq!(panchanga.spans.of(Limb::Nakshatra), None);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spans {
    /// The tithi's.
    pub tithi: Option<Span>,
    /// The Moon's nakshatra's.
    pub nakshatra: Option<Span>,
    /// The rising sign's.
    pub lagna: Option<Span>,
    /// The night's, from sunset to the next sunrise, by the day's sunrise
    /// convention; none for a birth by day or a day the Sun does not both
    /// rise and set. BPHS ch. 9 v. 14 measures each twilight from it: the
    /// first three ghatikas after the disc is half set, the last three
    /// before it is half risen.
    pub night: Option<Span>,
}

impl Spans {
    /// The span of one limb.
    ///
    /// ```
    /// use teistro_rules::{Limb, Span, Spans};
    ///
    /// let night = Span { elapsed: 1.0, remaining: 29.0 };
    /// let spans = Spans { tithi: None, nakshatra: None, lagna: None, night: Some(night) };
    /// assert_eq!(spans.of(Limb::Night), Some(night));
    /// assert_eq!(spans.of(Limb::Lagna), None);
    /// ```
    #[must_use]
    pub const fn of(&self, limb: Limb) -> Option<Span> {
        match limb {
            Limb::Tithi => self.tithi,
            Limb::Nakshatra => self.nakshatra,
            Limb::Lagna => self.lagna,
            Limb::Night => self.night,
        }
    }
}

/// How much of a limb had passed at the birth and how much was left, in
/// ghatikas.
///
/// ```
/// use teistro_rules::Span;
///
/// // Twenty of sixty ghatikas gone: the second third.
/// let span = Span { elapsed: 20.0, remaining: 40.0 };
/// assert_eq!(span.part(3), Some(2));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    /// Ghatikas since it began.
    pub elapsed: f64,
    /// Ghatikas until it ends.
    pub remaining: f64,
}

impl Span {
    /// Which of `of` equal parts of the limb the birth falls in, the first
    /// being 1; none for a span that is empty or measured backwards, or for
    /// no parts at all. The birth at a boundary falls in the later part, and
    /// the limb's last instant in the last.
    ///
    /// ```
    /// use teistro_rules::Span;
    ///
    /// let span = Span { elapsed: 10.0, remaining: 50.0 };
    /// assert_eq!(span.part(6), Some(2));
    /// assert_eq!(Span { elapsed: 0.0, remaining: 60.0 }.part(6), Some(1));
    /// assert_eq!(Span { elapsed: 60.0, remaining: 0.0 }.part(6), Some(6));
    /// assert_eq!(span.part(0), None);
    /// ```
    #[must_use]
    pub fn part(&self, of: u8) -> Option<u8> {
        let whole = self.elapsed + self.remaining;
        if of == 0 || self.elapsed < 0.0 || self.remaining < 0.0 || whole <= 0.0 {
            return None;
        }
        let at = (self.elapsed / whole * f64::from(of)).floor();
        // `at` is in 0..=of, so the cast is exact.
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a whole number from 0 to `of`, a u8"
        )]
        let at = at as u8;
        Some(at.min(of - 1) + 1)
    }
}

/// Which limb a condition measures.
///
/// ```
/// use teistro_rules::Limb;
///
/// assert_eq!(serde_json::to_string(&Limb::Nakshatra)?, r#""nakshatra""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum Limb {
    /// The tithi.
    Tithi,
    /// The Moon's nakshatra.
    Nakshatra,
    /// The rising sign.
    Lagna,
    /// The night, sunset to sunrise, whose first and last ghatikas are the
    /// twilights.
    Night,
}

/// An eclipse a birth falls in.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// # use teistro_rules::{Condition, Evaluator, Participants, Readings};
/// #
/// # let holds = |chart: &RuleChart, readings: Readings, condition: &str| {
/// #     let condition: Condition = serde_json::from_str(condition).unwrap();
/// #     Evaluator::new(chart, readings).holds(&condition, &mut Participants::default())
/// # };
///
/// # use teistro_core::catalogue::{Karana, Nakshatra, Tithi, Vara, Yoga};
/// # use teistro_rules::{Pada, Panchanga, Span, Spans};
/// #
/// # let panchanga = Panchanga {
/// #     tithi: Tithi::ShuklaDvitiya,
/// #     vara: Vara::Shanivara,
/// #     nakshatra: Nakshatra::Ashwini,
/// #     pada: Pada::try_new(1)?,
/// #     yoga: Yoga::Vishkambha,
/// #     karana: Karana::Bava,
/// #     spans: Spans {
/// #         tithi: Some(Span { elapsed: 10.0, remaining: 50.0 }),
/// #         nakshatra: None,
/// #         lagna: None,
/// #         night: None,
/// #     },
/// #     by_day: Some(true),
/// #     sankranti_hours: None,
/// #     eclipse: None,
/// # };
///
/// use teistro_rules::Eclipse;
///
/// chart.panchanga = Some(Panchanga { eclipse: Some(Eclipse::Lunar), ..panchanga });
/// assert!(holds(&chart, Readings::TEXTS, r#"{"type": "birth-during-eclipse", "kind": "lunar"}"#));
/// assert!(!holds(&chart, Readings::TEXTS, r#"{"type": "birth-during-eclipse", "kind": "solar"}"#));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Eclipse {
    /// Of the Sun.
    Solar,
    /// Of the Moon.
    Lunar,
}

/// A point the chart carries — an upagraha, a special lagna, a sphuta — where
/// it stands, as a [`Placement`] carries a body. An
/// [`Evaluator`](crate::Evaluator) given these resolves `{"point": …}`
/// references, and a reading that counts degrees, as the three pairs do of
/// the hora lagna, reads its longitude.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// use teistro_core::catalogue::Point;
/// use teistro_rules::{Evaluator, PointAt, Readings};
///
/// let gulika = [PointAt { point: Point::Gulika, longitude: 105.0, sign: Rashi::Cancer }];
/// let evaluator = Evaluator::new(&chart, Readings::TEXTS).with_points(&gulika);
/// assert_eq!(evaluator.points()[0].sign, Rashi::Cancer);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointAt {
    /// Which point.
    pub point: Point,
    /// Its sidereal longitude, degrees.
    pub longitude: f64,
    /// Its sign.
    pub sign: Rashi,
}

/// One divisional chart, as the rules read it: each body's sign in it, in
/// [`Body::ALL`]'s order. An [`Evaluator`](crate::Evaluator) given these reads
/// `in-varga` conditions in them.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// # use teistro_rules::{Condition, Evaluator, Participants, Readings};
/// #
/// # let holds = |chart: &RuleChart, readings: Readings, condition: &str| {
/// #     let condition: Condition = serde_json::from_str(condition).unwrap();
/// #     Evaluator::new(chart, readings).holds(&condition, &mut Participants::default())
/// # };
///
/// use teistro_core::catalogue::Varga;
///
/// // Every body in Leo in the navamsha.
/// let navamsha = chart.varga_signs(Varga::D9, |_| Rashi::Leo);
/// let condition: Condition = serde_json::from_str(r#"{"type": "in-varga", "varga": "D9", "condition": {"type": "lagna-in-sign", "signs": ["LEO"]}}"#)?;
/// let evaluator = Evaluator::new(&chart, Readings::TEXTS);
/// assert!(!evaluator.holds(&condition, &mut Participants::default()));
/// let vargas = [navamsha];
/// assert!(evaluator.with_vargas(&vargas).holds(&condition, &mut Participants::default()));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VargaSigns {
    /// Which division.
    pub varga: Varga,
    /// Each body's sign in it.
    pub signs: [Rashi; 10],
}

impl RuleChart {
    /// The chart a divisional one makes: each body in its varga sign, its
    /// houses counted whole-sign from the varga lagna, and its dignity read
    /// from the varga sign as the Saptavargaja does
    /// (`teistro_state::dignity::varga_dignity`). A body's longitude, motion,
    /// combustion, karakas and navamsha stay the rasi chart's, and the
    /// language refuses a condition that reads a longitude inside a varga.
    ///
    /// ```
    /// # use teistro_core::catalogue::{Dignity, Rashi};
    /// # use teistro_rules::{Body, House, Placement, RuleChart};
    /// #
    /// # let mut chart = RuleChart {
    /// #     placements: [Placement {
    /// #         longitude: 15.0,
    /// #         sign: Rashi::Aries,
    /// #         house: House::try_new(1).unwrap(),
    /// #         dignity: Dignity::Neutral,
    /// #         retrograde: false,
    /// #         combust: false,
    /// #         karaka7: None,
    /// #         karaka8: None,
    /// #         navamsha: Rashi::Aries,
    /// #     }; 10],
    /// #     panchanga: None,
    /// #     strengths: None,
    /// # };
    /// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
    /// #     let at = &mut chart.placements[body.index()];
    /// #     at.sign = sign;
    /// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
    /// #     at.house = House::between(Rashi::Aries, sign);
    /// # };
    ///
    /// use teistro_core::catalogue::{Graha, Varga};
    ///
    /// let mut signs = chart.varga_signs(Varga::D9, |_| Rashi::Leo);
    /// signs.signs[Body::Graha(Graha::Saturn).index()] = Rashi::Aquarius;
    /// let navamsha = chart.in_varga(&signs);
    /// // Houses counted from the varga lagna; the longitude stays the rasi chart's.
    /// assert_eq!(navamsha.lagna(), Rashi::Leo);
    /// let saturn = navamsha.placement(Body::Graha(Graha::Saturn));
    /// assert_eq!((saturn.house, saturn.longitude), (House::try_new(7)?, 15.0));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn in_varga(&self, signs: &VargaSigns) -> RuleChart {
        let rasi = |graha: Graha| {
            Body::ALL
                .iter()
                .position(|body| *body == Body::Graha(graha))
                .and_then(|at| self.placements.get(at))
                .map(|placement| placement.sign)
        };
        let lagna = signs.signs.get(Body::Lagna.index()).copied();
        let mut placements = self.placements;
        for (at, placement) in placements.iter_mut().enumerate() {
            let Some(sign) = signs.signs.get(at).copied() else {
                continue;
            };
            placement.sign = sign;
            if let Some(lagna) = lagna {
                placement.house = House::between(lagna, sign);
            }
            placement.dignity = match Body::ALL.get(at) {
                Some(Body::Graha(graha)) => dignity::varga_dignity(*graha, sign, rasi),
                _ => placement.dignity,
            };
        }
        RuleChart {
            placements,
            ..*self
        }
    }
}

/// Where the eight-karaka scheme puts the Pitrikaraka (crux C101).
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// use teistro_core::catalogue::{CharaKaraka, Graha};
/// use teistro_rules::EightKarakas;
///
/// // The Moon furthest into her sign, Mars next, the rest equal.
/// chart.placements[Body::Graha(Graha::Moon).index()].longitude = 25.0;
/// chart.placements[Body::Graha(Graha::Mars).index()].longitude = 20.0;
/// let least = |order| chart.with_chara_karakas(order).placement(Body::Graha(Graha::Moon)).karaka8;
/// assert_eq!(least(EightKarakas::Parashara), Some(CharaKaraka::Atmakaraka));
/// assert_eq!(EightKarakas::default(), EightKarakas::Parashara);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EightKarakas {
    /// BPHS ch. 32 vv. 13 to 17: the Amatyakaraka, Bhratri, Matri, Pitri,
    /// Putra, Gnati and Dara karakas in turn, so the Darakaraka is the least.
    #[default]
    Parashara,
    /// The Pitrikaraka after the Darakaraka, as the least: the recording
    /// engine's order, which reproduces every one of the corpus's 93 charts
    /// where Parashara's reproduces none.
    PitrikarakaLast,
}

impl EightKarakas {
    /// The karakas from the most advanced in its sign down.
    const fn order(self) -> [CharaKaraka; 8] {
        use CharaKaraka::{
            Amatyakaraka, Atmakaraka, Bhratrikaraka, Darakaraka, Gnatikaraka, Matrikaraka,
            Pitrikaraka, Putrakaraka,
        };
        match self {
            EightKarakas::Parashara => [
                Atmakaraka,
                Amatyakaraka,
                Bhratrikaraka,
                Matrikaraka,
                Pitrikaraka,
                Putrakaraka,
                Gnatikaraka,
                Darakaraka,
            ],
            EightKarakas::PitrikarakaLast => [
                Atmakaraka,
                Amatyakaraka,
                Bhratrikaraka,
                Matrikaraka,
                Putrakaraka,
                Gnatikaraka,
                Darakaraka,
                Pitrikaraka,
            ],
        }
    }
}

/// The seven karakas from the most advanced down, where the Matri and Putra
/// karakas are one (BPHS ch. 32 vv. 13 to 17).
const SEVEN_KARAKAS: [CharaKaraka; 7] = [
    CharaKaraka::Atmakaraka,
    CharaKaraka::Amatyakaraka,
    CharaKaraka::Bhratrikaraka,
    CharaKaraka::Matrikaraka,
    CharaKaraka::Putrakaraka,
    CharaKaraka::Gnatikaraka,
    CharaKaraka::Darakaraka,
];

/// The grahas the seven-karaka scheme ranks, the Sun to Saturn.
const KARAKA_GRAHAS: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

impl RuleChart {
    /// The same chart with every graha's chara karaka computed from its
    /// longitude, in both schemes (BPHS ch. 32 vv. 3 to 8 and 13 to 17): the
    /// graha furthest into its sign is the Atmakaraka, the next the
    /// Amatyakaraka, and so down; Rahu, counted only among eight, by how far
    /// he has still to go, his motion being backwards. The grahas are
    /// compared to the arc-second, as the verses compare degrees, then
    /// minutes, then seconds, and two that are equal to the second hold one
    /// karaka between them, which leaves the last unheld.
    ///
    /// ```
    /// use teistro_core::catalogue::{CharaKaraka, Dignity, Graha, Rashi};
    /// use teistro_rules::{Body, EightKarakas, House, Placement, RuleChart};
    ///
    /// // Every body at 10° Aries but the Moon, at 25°.
    /// let at = |longitude| Placement {
    ///     longitude,
    ///     sign: Rashi::Aries,
    ///     house: House::try_new(1).unwrap(),
    ///     dignity: Dignity::Neutral,
    ///     retrograde: false,
    ///     combust: false,
    ///     karaka7: None,
    ///     karaka8: None,
    ///     navamsha: Rashi::Aries,
    /// };
    /// let mut placements = [at(10.0); 10];
    /// placements[Body::Graha(Graha::Moon).index()] = at(25.0);
    /// let chart = RuleChart { placements, panchanga: None, strengths: None }
    ///     .with_chara_karakas(EightKarakas::Parashara);
    /// let karakas = |graha| {
    ///     let placement = chart.placement(Body::Graha(graha));
    ///     (placement.karaka7, placement.karaka8)
    /// };
    /// assert_eq!(karakas(Graha::Moon).0, Some(CharaKaraka::Atmakaraka));
    /// // The six at 10° are equal to the second and share the Amatyakaraka;
    /// // among eight, Rahu, with 20° still to go, takes it and they share the
    /// // Bhratrikaraka.
    /// assert_eq!(karakas(Graha::Sun), (Some(CharaKaraka::Amatyakaraka), Some(CharaKaraka::Bhratrikaraka)));
    /// assert_eq!(karakas(Graha::Rahu), (None, Some(CharaKaraka::Amatyakaraka)));
    /// ```
    #[must_use]
    pub fn with_chara_karakas(mut self, order: EightKarakas) -> RuleChart {
        let seconds = |graha: Graha| self.karaka_seconds(graha);
        let seven = KARAKA_GRAHAS.map(|graha| (graha, seconds(graha)));
        let mut with_rahu = [(Graha::Rahu, seconds(Graha::Rahu)); 8];
        for (slot, graha) in with_rahu.iter_mut().zip(seven) {
            *slot = graha;
        }
        for placement in &mut self.placements {
            placement.karaka7 = None;
            placement.karaka8 = None;
        }
        for (graha, karaka) in ranked(&seven, &SEVEN_KARAKAS) {
            if let Some(at) = self.placements.get_mut(Body::Graha(graha).index()) {
                at.karaka7 = karaka;
            }
        }
        for (graha, karaka) in ranked(&with_rahu, &order.order()) {
            if let Some(at) = self.placements.get_mut(Body::Graha(graha).index()) {
                at.karaka8 = karaka;
            }
        }
        self
    }

    /// The grahas that hold the chara karakas, the Atmakaraka's first and
    /// down (BPHS ch. 32): seven, or eight `with_rahu`. This is the order
    /// the Karaka dasha runs in (ch. 46 v. 178) under either naming of the
    /// eight, since [`EightKarakas`] names the ranks and does not reorder
    /// them. Two grahas equal to the arc-second keep the grahas' order.
    ///
    /// ```
    /// use teistro_core::catalogue::{Dignity, Graha, Rashi};
    /// use teistro_rules::{Body, House, Placement, RuleChart};
    ///
    /// let at = |longitude| Placement {
    ///     longitude,
    ///     sign: Rashi::Aries,
    ///     house: House::try_new(1).unwrap(),
    ///     dignity: Dignity::Neutral,
    ///     retrograde: false,
    ///     combust: false,
    ///     karaka7: None,
    ///     karaka8: None,
    ///     navamsha: Rashi::Aries,
    /// };
    /// let mut placements = [at(10.0); 10];
    /// placements[Body::Graha(Graha::Moon).index()] = at(25.0);
    /// // Rahu at 2° has 28° still to go, the most of all.
    /// placements[Body::Graha(Graha::Rahu).index()] = at(2.0);
    /// let chart = RuleChart { placements, panchanga: None, strengths: None };
    /// assert_eq!(chart.chara_karaka_ranking(false)[0], Graha::Moon);
    /// assert_eq!(&chart.chara_karaka_ranking(true)[..2], &[Graha::Rahu, Graha::Moon]);
    /// ```
    #[must_use]
    pub fn chara_karaka_ranking(&self, with_rahu: bool) -> Vec<Graha> {
        let mut grahas = KARAKA_GRAHAS.to_vec();
        if with_rahu {
            grahas.push(Graha::Rahu);
        }
        // Stable, so equals keep the grahas' order.
        grahas.sort_by_key(|&graha| core::cmp::Reverse(self.karaka_seconds(graha)));
        grahas
    }

    /// How far `graha` is into its sign in whole arc-seconds, or, for
    /// Rahu, whose motion is backwards, how far he has still to go: what
    /// the chara karakas rank by.
    fn karaka_seconds(&self, graha: Graha) -> u32 {
        let into = self
            .placement(Body::Graha(graha))
            .longitude
            .rem_euclid(30.0);
        let degrees = if graha == Graha::Rahu {
            30.0 - into
        } else {
            into
        };
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a place in a sign is under 108 000 arc-seconds"
        )]
        let seconds = (degrees * 3600.0).floor() as u32;
        seconds
    }

    /// A divisional chart of this one: each body's sign in the division, from
    /// its longitude by `sign_of`, the division's rule. The kernel does not
    /// choose the rule, a division having several; the SDK's bridge passes its
    /// classical scheme.
    ///
    /// ```
    /// # use teistro_core::catalogue::{Dignity, Rashi};
    /// # use teistro_rules::{Body, House, Placement, RuleChart};
    /// #
    /// # let mut chart = RuleChart {
    /// #     placements: [Placement {
    /// #         longitude: 15.0,
    /// #         sign: Rashi::Aries,
    /// #         house: House::try_new(1).unwrap(),
    /// #         dignity: Dignity::Neutral,
    /// #         retrograde: false,
    /// #         combust: false,
    /// #         karaka7: None,
    /// #         karaka8: None,
    /// #         navamsha: Rashi::Aries,
    /// #     }; 10],
    /// #     panchanga: None,
    /// #     strengths: None,
    /// # };
    /// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
    /// #     let at = &mut chart.placements[body.index()];
    /// #     at.sign = sign;
    /// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
    /// #     at.house = House::between(Rashi::Aries, sign);
    /// # };
    ///
    /// use teistro_core::catalogue::Varga;
    ///
    /// // A navamsha from Aries for a movable sign: 15° is the fifth ninth, Leo.
    /// let navamsha = chart.varga_signs(Varga::D9, |longitude| Rashi::ALL[(longitude / (30.0 / 9.0)) as usize % 12]);
    /// assert_eq!(navamsha.varga, Varga::D9);
    /// assert!(navamsha.signs.iter().all(|sign| *sign == Rashi::Leo));
    /// ```
    #[must_use]
    pub fn varga_signs(&self, varga: Varga, sign_of: impl Fn(f64) -> Rashi) -> VargaSigns {
        VargaSigns {
            varga,
            signs: self
                .placements
                .map(|placement| sign_of(placement.longitude)),
        }
    }
}

/// Each graha with the karaka its rank gives: its rank is how many distinct
/// places lie further into their signs, so grahas equal to the second share
/// one, and a rank past the last karaka holds none.
fn ranked<const N: usize>(
    grahas: &[(Graha, u32); N],
    karakas: &[CharaKaraka; N],
) -> [(Graha, Option<CharaKaraka>); N] {
    grahas.map(|(graha, seconds)| {
        let ahead = grahas
            .iter()
            .enumerate()
            .filter(|(at, (_, other))| {
                *other > seconds && !grahas.iter().take(*at).any(|(_, earlier)| earlier == other)
            })
            .count();
        (graha, karakas.get(ahead).copied())
    })
}

impl RuleChart {
    /// A body's placement.
    ///
    /// ```
    /// # use teistro_core::catalogue::{Dignity, Rashi};
    /// # use teistro_rules::{Body, House, Placement, RuleChart};
    /// #
    /// # let mut chart = RuleChart {
    /// #     placements: [Placement {
    /// #         longitude: 15.0,
    /// #         sign: Rashi::Aries,
    /// #         house: House::try_new(1).unwrap(),
    /// #         dignity: Dignity::Neutral,
    /// #         retrograde: false,
    /// #         combust: false,
    /// #         karaka7: None,
    /// #         karaka8: None,
    /// #         navamsha: Rashi::Aries,
    /// #     }; 10],
    /// #     panchanga: None,
    /// #     strengths: None,
    /// # };
    /// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
    /// #     let at = &mut chart.placements[body.index()];
    /// #     at.sign = sign;
    /// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
    /// #     at.house = House::between(Rashi::Aries, sign);
    /// # };
    ///
    /// use teistro_core::catalogue::Graha;
    ///
    /// assert_eq!(chart.placement(Body::Graha(Graha::Venus)).dignity, Dignity::Neutral);
    /// assert_eq!(chart.placement(Body::Lagna).sign, Rashi::Aries);
    /// ```
    #[must_use]
    pub fn placement(&self, body: Body) -> &Placement {
        // `Body::index` is below ten by construction.
        #[allow(clippy::indexing_slicing, reason = "a body's index is below ten")]
        &self.placements[body.index()]
    }

    /// The lagna's sign.
    ///
    /// ```
    /// # use teistro_core::catalogue::{Dignity, Rashi};
    /// # use teistro_rules::{Body, House, Placement, RuleChart};
    /// #
    /// # let mut chart = RuleChart {
    /// #     placements: [Placement {
    /// #         longitude: 15.0,
    /// #         sign: Rashi::Aries,
    /// #         house: House::try_new(1).unwrap(),
    /// #         dignity: Dignity::Neutral,
    /// #         retrograde: false,
    /// #         combust: false,
    /// #         karaka7: None,
    /// #         karaka8: None,
    /// #         navamsha: Rashi::Aries,
    /// #     }; 10],
    /// #     panchanga: None,
    /// #     strengths: None,
    /// # };
    /// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
    /// #     let at = &mut chart.placements[body.index()];
    /// #     at.sign = sign;
    /// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
    /// #     at.house = House::between(Rashi::Aries, sign);
    /// # };
    ///
    /// place(&mut chart, Body::Lagna, Rashi::Virgo);
    /// assert_eq!(chart.lagna(), Rashi::Virgo);
    /// ```
    #[must_use]
    pub fn lagna(&self) -> Rashi {
        self.placement(Body::Lagna).sign
    }
}

/// Who counts as a benefic.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// # use teistro_rules::{Condition, Evaluator, Participants, Readings};
/// #
/// # let holds = |chart: &RuleChart, readings: Readings, condition: &str| {
/// #     let condition: Condition = serde_json::from_str(condition).unwrap();
/// #     Evaluator::new(chart, readings).holds(&condition, &mut Participants::default())
/// # };
///
/// use teistro_core::catalogue::Graha;
/// use teistro_rules::Benefics;
///
/// // The Moon 210° past the Sun is waning: a malefic by company, still a benefic by nature.
/// place(&mut chart, Body::Graha(Graha::Moon), Rashi::Scorpio);
/// let benefic = r#"{"type": "planet-is", "planet": "MOON", "class": "benefic"}"#;
/// assert!(!holds(&chart, Readings::TEXTS, benefic));
/// assert!(holds(&chart, Readings { benefics: Benefics::Natural, ..Readings::TEXTS }, benefic));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Benefics {
    /// The natural lists, the Moon turned malefic when waning and Mercury when
    /// only malefics share its sign: the recording engine's reading.
    #[default]
    ByCompany,
    /// Jupiter, Venus, Mercury and the Moon, always.
    Natural,
}

/// Whether a deep dignity meets a rule asking for its plain form.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// # use teistro_rules::{Condition, Evaluator, Participants, Readings};
/// #
/// # let holds = |chart: &RuleChart, readings: Readings, condition: &str| {
/// #     let condition: Condition = serde_json::from_str(condition).unwrap();
/// #     Evaluator::new(chart, readings).holds(&condition, &mut Participants::default())
/// # };
///
/// use teistro_core::catalogue::Graha;
/// use teistro_rules::DignityMatch;
///
/// chart.placements[Body::Graha(Graha::Sun).index()].dignity = Dignity::DeepExalted;
/// let exalted = r#"{"type": "planet-dignity", "planet": "SUN", "dignities": ["EXALTED"]}"#;
/// assert!(holds(&chart, Readings::TEXTS, exalted));
/// assert!(!holds(&chart, Readings { dignity: DignityMatch::Exact, ..Readings::TEXTS }, exalted));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DignityMatch {
    /// Deep exaltation meets exaltation and deep debilitation debilitation:
    /// the recording engine's reading.
    #[default]
    DeepMeetsPlain,
    /// Only the dignity the rule names.
    Exact,
}

/// Which side of the nodal axis the seven may stand on.
///
/// ```
/// use teistro_rules::{NodeSides, Readings};
///
/// // Either side of the axis, unless a reading asks for Rahu to Ketu.
/// assert_eq!(Readings::TEXTS.node_sides, NodeSides::Either);
/// assert_ne!(NodeSides::RahuToKetu, NodeSides::default());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NodeSides {
    /// Either side: the recording engine's reading.
    #[default]
    Either,
    /// From Rahu to Ketu only.
    RahuToKetu,
}

/// Which house a body is in.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// # use teistro_rules::{Condition, Evaluator, Participants, Readings};
/// #
/// # let holds = |chart: &RuleChart, readings: Readings, condition: &str| {
/// #     let condition: Condition = serde_json::from_str(condition).unwrap();
/// #     Evaluator::new(chart, readings).holds(&condition, &mut Participants::default())
/// # };
///
/// use teistro_core::catalogue::Graha;
/// use teistro_rules::Houses;
///
/// // A chart that records Jupiter in the second though he stands in the lagna's sign.
/// chart.placements[Body::Graha(Graha::Jupiter).index()].house = House::try_new(2)?;
/// let second = r#"{"type": "planet-in-house", "planet": "JUPITER", "houses": [2]}"#;
/// assert!(holds(&chart, Readings { houses: Houses::Recorded, ..Readings::TEXTS }, second));
/// assert!(!holds(&chart, Readings { houses: Houses::WholeSign, ..Readings::TEXTS }, second));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Houses {
    /// The house the chart records: the recording engine's reading.
    #[default]
    Recorded,
    /// Whole signs from the lagna.
    WholeSign,
}

/// Whether the nodes count as retrograde.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// # use teistro_rules::{Condition, Evaluator, Participants, Readings};
/// #
/// # let holds = |chart: &RuleChart, readings: Readings, condition: &str| {
/// #     let condition: Condition = serde_json::from_str(condition).unwrap();
/// #     Evaluator::new(chart, readings).holds(&condition, &mut Participants::default())
/// # };
///
/// // "As the nodes have retrograde motions" (BPHS ch. 31 v. 6).
/// let rahu = r#"{"type": "planet-retrograde", "planet": "RAHU"}"#;
/// assert!(holds(&chart, Readings::TEXTS, rahu));
/// assert!(!holds(&chart, Readings::RECORDING_ENGINE, rahu));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NodeMotion {
    /// Never, whatever the chart records: the recording engine's reading.
    #[default]
    NeverRetrograde,
    /// Always.
    AlwaysRetrograde,
}

/// Which conditions add their bodies to a rule's participants.
///
/// ```
/// use teistro_rules::{Gathering, Readings};
///
/// assert_eq!(Readings::TEXTS.gathering, Gathering::EveryHeld);
/// let narrow = Readings { gathering: Gathering::DecidingBranch, ..Readings::TEXTS };
/// assert_ne!(narrow, Readings::TEXTS);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Gathering {
    /// Every condition that held, even inside a branch that went on to fail:
    /// the recording engine's reading.
    #[default]
    EveryHeld,
    /// Only the branches that decided.
    DecidingBranch,
}

/// How an unqualified conjunction is judged.
///
/// ```
/// # use teistro_core::catalogue::{Dignity, Rashi};
/// # use teistro_rules::{Body, House, Placement, RuleChart};
/// #
/// # let mut chart = RuleChart {
/// #     placements: [Placement {
/// #         longitude: 15.0,
/// #         sign: Rashi::Aries,
/// #         house: House::try_new(1).unwrap(),
/// #         dignity: Dignity::Neutral,
/// #         retrograde: false,
/// #         combust: false,
/// #         karaka7: None,
/// #         karaka8: None,
/// #         navamsha: Rashi::Aries,
/// #     }; 10],
/// #     panchanga: None,
/// #     strengths: None,
/// # };
/// # let place = |chart: &mut RuleChart, body: Body, sign: Rashi| {
/// #     let at = &mut chart.placements[body.index()];
/// #     at.sign = sign;
/// #     at.longitude = f64::from(sign as u8) * 30.0 + 15.0;
/// #     at.house = House::between(Rashi::Aries, sign);
/// # };
///
/// # use teistro_rules::{Condition, Evaluator, Participants, Readings};
/// #
/// # let holds = |chart: &RuleChart, readings: Readings, condition: &str| {
/// #     let condition: Condition = serde_json::from_str(condition).unwrap();
/// #     Evaluator::new(chart, readings).holds(&condition, &mut Participants::default())
/// # };
///
/// use teistro_core::catalogue::Graha;
/// use teistro_rules::Conjunction;
///
/// // Mars at 29° Aries and the Moon at 1° Taurus: two degrees apart, a sign between.
/// chart.placements[Body::Graha(Graha::Mars).index()].longitude = 29.0;
/// place(&mut chart, Body::Graha(Graha::Moon), Rashi::Taurus);
/// chart.placements[Body::Graha(Graha::Moon).index()].longitude = 31.0;
/// let together = r#"{"type": "planet-conjunct", "planets": ["MOON", "MARS"]}"#;
/// assert!(!holds(&chart, Readings::TEXTS, together));
/// assert!(holds(&chart, Readings { conjunction: Conjunction::Orb(5.0), ..Readings::TEXTS }, together));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Conjunction {
    /// In one sign: the recording engine's reading.
    #[default]
    SameSign,
    /// Within an orb, degrees, across a sign's edge too.
    Orb(f64),
}

/// Which house's pada is the upapada (BPHS ch. 30 vv. 1 to 6).
///
/// ```
/// use teistro_rules::{Readings, Upapada};
///
/// assert_eq!(Readings::TEXTS.upapada, Upapada::Twelfth);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Upapada {
    /// The twelfth's, the house "following" the lagna as the translation
    /// reads it: the text's reading, and the default, since the recording
    /// engine has no upapada.
    #[default]
    Twelfth,
    /// The twelfth's for an odd lagna and the second's for an even one, as the
    /// translation's note gives the Jaimini commentaries.
    ByLagnaParity,
}

/// Which stretch of a sign "the nth degree" of a table names.
///
/// ```
/// use teistro_rules::{Bhaga, SignDegree};
///
/// // "The fifth degree": 4° to 5° when running, 5° to 6° when completed.
/// let fifth = SignDegree::try_new(5)?;
/// assert!(Bhaga::Running.contains(fifth, 4.5) && !Bhaga::Running.contains(fifth, 5.0));
/// assert!(Bhaga::Completed.contains(fifth, 5.0));
/// assert!(Bhaga::WithinOne.contains(fifth, 6.0));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Bhaga {
    /// The degree running to n: from n − 1° to n°. Jataka Parijata's "degrees
    /// attained" (ch. 1 vv. 57 and 58) read as the ordinal degree: the
    /// default.
    #[default]
    Running,
    /// From n − ½° to n + ½°.
    Centred,
    /// The degree completed at n: from n° to n + 1°.
    Completed,
    /// Within a degree either side of n: the recording engine's reading.
    WithinOne,
}

impl Bhaga {
    /// Whether a place `in_sign` degrees into its sign is in the stretch the
    /// table's `degree` names: each stretch includes its start and excludes
    /// its end, but for the engine's, which includes both.
    ///
    /// ```
    /// use teistro_rules::{Bhaga, SignDegree};
    ///
    /// let tenth = SignDegree::try_new(10)?;
    /// assert!(Bhaga::Centred.contains(tenth, 9.5));
    /// assert!(!Bhaga::Centred.contains(tenth, 10.5));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn contains(self, degree: SignDegree, in_sign: f64) -> bool {
        let n = f64::from(degree.get());
        match self {
            Bhaga::Running => (n - 1.0..n).contains(&in_sign),
            Bhaga::Centred => (n - 0.5..n + 0.5).contains(&in_sign),
            Bhaga::Completed => (n..n + 1.0).contains(&in_sign),
            Bhaga::WithinOne => (in_sign - n).abs() <= 1.0,
        }
    }
}

/// Which bodies an aspect condition adds to a rule's participants.
///
/// ```
/// use teistro_rules::{AspectGathering, Readings};
///
/// assert_eq!(Readings::RECORDING_ENGINE.aspect_gathering, AspectGathering::Both);
/// assert_eq!(Readings::RECORDING_ENGINE_DOSHAS.aspect_gathering, AspectGathering::Neither);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AspectGathering {
    /// The body aspecting and the one aspected: the recording engine's yoga
    /// evaluator.
    #[default]
    Both,
    /// Neither: its dosha evaluator, which reads aspects itself.
    Neither,
}

/// A choice at every place the condition language leaves a meaning open
/// (`03-design/yogas-measured.md`); the default is the texts' where they
/// settle one and the recording engine's elsewhere.
///
/// ```
/// use teistro_rules::{Houses, Readings};
///
/// // The texts' reading is the default; one field changed is another reading.
/// assert_eq!(Readings::default(), Readings::TEXTS);
/// let recorded = Readings { houses: Houses::Recorded, ..Readings::default() };
/// assert_ne!(recorded, Readings::TEXTS);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Readings {
    /// Who counts as a benefic.
    pub benefics: Benefics,
    /// Whether a deep dignity meets its plain form.
    pub dignity: DignityMatch,
    /// Which side of the nodes the seven may stand on.
    pub node_sides: NodeSides,
    /// Which house a body is in.
    pub houses: Houses,
    /// Whether the nodes count as retrograde.
    pub node_motion: NodeMotion,
    /// Which conditions add participants.
    pub gathering: Gathering,
    /// How an unqualified conjunction is judged.
    pub conjunction: Conjunction,
    /// What the nodes aspect beyond the seventh.
    pub node_aspects: NodeAspects,
    /// Which house's pada is the upapada.
    pub upapada: Upapada,
    /// Which stretch of a sign a table's degree names.
    pub bhaga: Bhaga,
    /// Which bodies an aspect condition adds.
    pub aspect_gathering: AspectGathering,
}

impl Default for Readings {
    fn default() -> Readings {
        Readings::TEXTS
    }
}

impl Readings {
    /// The recording engine's reading at every place it has one, and the
    /// text's where it has none.
    ///
    /// ```
    /// use teistro_rules::{Bhaga, Houses, Readings};
    ///
    /// let engine = Readings::RECORDING_ENGINE;
    /// assert_eq!((engine.houses, engine.bhaga), (Houses::Recorded, Bhaga::WithinOne));
    /// ```
    pub const RECORDING_ENGINE: Readings = Readings {
        benefics: Benefics::ByCompany,
        dignity: DignityMatch::DeepMeetsPlain,
        node_sides: NodeSides::Either,
        houses: Houses::Recorded,
        node_motion: NodeMotion::NeverRetrograde,
        gathering: Gathering::EveryHeld,
        conjunction: Conjunction::SameSign,
        node_aspects: NodeAspects::None,
        upapada: Upapada::Twelfth,
        bhaga: Bhaga::WithinOne,
        aspect_gathering: AspectGathering::Both,
    };

    /// The recording engine's dosha evaluator: its yoga evaluator's reading
    /// but for aspects, which add no participant.
    ///
    /// ```
    /// use teistro_rules::{AspectGathering, Readings};
    ///
    /// // The yoga reading but for aspects.
    /// let doshas = Readings::RECORDING_ENGINE_DOSHAS;
    /// assert_eq!(Readings { aspect_gathering: AspectGathering::Both, ..doshas }, Readings::RECORDING_ENGINE);
    /// ```
    pub const RECORDING_ENGINE_DOSHAS: Readings = Readings {
        aspect_gathering: AspectGathering::Neither,
        ..Readings::RECORDING_ENGINE
    };

    /// The texts' reading wherever a text read settles one, the recording
    /// engine's elsewhere: the default.
    ///
    /// ```
    /// use teistro_rules::{Bhaga, Houses, NodeMotion, Readings};
    ///
    /// let texts = Readings::TEXTS;
    /// assert_eq!((texts.bhaga, texts.houses, texts.node_motion), (Bhaga::Running, Houses::WholeSign, NodeMotion::AlwaysRetrograde));
    /// ```
    pub const TEXTS: Readings = Readings {
        // The ordinal degree the texts count a bhaga by (crux C82).
        bhaga: Bhaga::Running,
        // "As the nodes have retrograde motions" — BPHS ch. 31 v. 6, which is
        // the verse the intervention's backward count already rests on. A
        // default that read the nodes as direct would contradict a verse the
        // SDK itself cites.
        node_motion: NodeMotion::AlwaysRetrograde,
        // The verses count bhavas as whole signs from the lagna, and a rule
        // that meant the chalit would have to say so. `Recorded` follows
        // whatever house the caller's chart carries, so a chart built under
        // another placement system would quietly answer a different question;
        // this makes the kernel count for itself.
        houses: Houses::WholeSign,
        ..Readings::RECORDING_ENGINE
    };
}

/// The natural benefics.
pub(crate) const NATURAL_BENEFICS: [Graha; 4] =
    [Graha::Jupiter, Graha::Venus, Graha::Mercury, Graha::Moon];
/// The natural malefics.
pub(crate) const NATURAL_MALEFICS: [Graha; 5] = [
    Graha::Sun,
    Graha::Mars,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    /// Weak is below what is required, not at it; a chart that says
    /// nothing is weak in nothing.
    #[test]
    fn a_body_is_weak_only_below_what_it_requires() {
        let strengths = |strength: f64| Strengths {
            measure: StrengthMeasure::Caller,
            of: [Some(strength); 10],
            required: [Some(5.0); 10],
        };
        assert!(strengths(4.9).is_weak(Body::Lagna));
        assert!(!strengths(5.0).is_weak(Body::Lagna));
        let silent = Strengths {
            required: [None; 10],
            ..strengths(1.0)
        };
        assert!(!silent.is_weak(Body::Lagna));
    }

    /// A span with either part below nothing has no part.
    #[test]
    fn a_span_with_a_negative_part_has_no_part() {
        let part = |elapsed: f64, remaining: f64| Span { elapsed, remaining }.part(6);
        assert_eq!(part(-1.0, 60.0), None);
        assert_eq!(part(60.0, -1.0), None);
        assert_eq!(part(30.0, 30.0), Some(4));
    }
}
