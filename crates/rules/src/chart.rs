//! What a rule reads of a chart, and the choices the language leaves open.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{
    CharaKaraka, Dignity, Graha, Karana, Nakshatra, Point, Rashi, Tithi, Vara, Varga, Yoga,
};
use teistro_core::settings::NodeAspects;

use crate::language::{Body, House, Pada};
use crate::table::SignDegree;
use teistro_state::dignity;

/// One body as a rule reads it.
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
    pub const NONE: Strengths = Strengths {
        measure: StrengthMeasure::Shadbala,
        of: [None; 10],
        required: [None; 10],
    };

    /// Whether a body reaches what is required of it; false where either
    /// number is missing.
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
    #[must_use]
    pub fn is_weak(&self, body: Body) -> bool {
        match (self.of.get(body.index()), self.required.get(body.index())) {
            (Some(Some(strength)), Some(Some(required))) => strength < required,
            _ => false,
        }
    }

    /// Whether one body's strength exceeds another's; false where either is
    /// missing.
    #[must_use]
    pub fn exceeds(&self, one: Body, other: Body) -> bool {
        match (self.of.get(one.index()), self.of.get(other.index())) {
            (Some(Some(one)), Some(Some(other))) => one > other,
            _ => false,
        }
    }
}

/// Which measure a chart's strengths are in.
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
    /// Whether the birth falls on a sankranti, under whatever window the
    /// chart's maker reads.
    pub on_sankranti: bool,
    /// The eclipse the birth falls in, if any.
    pub eclipse: Option<Eclipse>,
}

/// How far the birth stood into each limb, and how much of it was left, in
/// ghatikas of 24 minutes. A limb the chart does not measure is none, and a
/// condition reading its edge does not hold.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spans {
    /// The tithi's.
    pub tithi: Option<Span>,
    /// The Moon's nakshatra's.
    pub nakshatra: Option<Span>,
    /// The rising sign's.
    pub lagna: Option<Span>,
}

impl Spans {
    /// The span of one limb.
    #[must_use]
    pub const fn of(&self, limb: Limb) -> Option<Span> {
        match limb {
            Limb::Tithi => self.tithi,
            Limb::Nakshatra => self.nakshatra,
            Limb::Lagna => self.lagna,
        }
    }
}

/// How much of a limb had passed at the birth and how much was left, in
/// ghatikas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    /// Ghatikas since it began.
    pub elapsed: f64,
    /// Ghatikas until it ends.
    pub remaining: f64,
}

/// Which limb a condition measures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Limb {
    /// The tithi.
    Tithi,
    /// The Moon's nakshatra.
    Nakshatra,
    /// The rising sign.
    Lagna,
}

/// An eclipse a birth falls in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Eclipse {
    /// Of the Sun.
    Solar,
    /// Of the Moon.
    Lunar,
}

/// A point the chart carries — an upagraha, a special lagna, a sphuta — and
/// the sign it stands in. An [`Evaluator`](crate::Evaluator) given these
/// resolves `{"point": …}` references.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointAt {
    /// Which point.
    pub point: Point,
    /// Its sign.
    pub sign: Rashi,
}

/// One divisional chart, as the rules read it: each body's sign in it, in
/// [`Body::ALL`]'s order. An [`Evaluator`](crate::Evaluator) given these reads
/// `in-varga` conditions in them.
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

impl RuleChart {
    /// A body's placement.
    #[must_use]
    pub fn placement(&self, body: Body) -> &Placement {
        // `Body::index` is below ten by construction.
        #[allow(clippy::indexing_slicing, reason = "a body's index is below ten")]
        &self.placements[body.index()]
    }

    /// The lagna's sign.
    #[must_use]
    pub fn lagna(&self) -> Rashi {
        self.placement(Body::Lagna).sign
    }
}

/// Who counts as a benefic.
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NodeSides {
    /// Either side: the recording engine's reading.
    #[default]
    Either,
    /// From Rahu to Ketu only.
    RahuToKetu,
}

/// Which house a body is in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Houses {
    /// The house the chart records: the recording engine's reading.
    #[default]
    Recorded,
    /// Whole signs from the lagna.
    WholeSign,
}

/// Whether the nodes count as retrograde.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NodeMotion {
    /// Never, whatever the chart records: the recording engine's reading.
    #[default]
    NeverRetrograde,
    /// Always.
    AlwaysRetrograde,
}

/// Which conditions add their bodies to a rule's participants.
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
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Conjunction {
    /// In one sign: the recording engine's reading.
    #[default]
    SameSign,
    /// Within an orb, degrees, across a sign's edge too.
    Orb(f64),
}

/// Which house's pada is the upapada (BPHS ch. 30 vv. 1 to 6).
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
    pub const RECORDING_ENGINE_DOSHAS: Readings = Readings {
        aspect_gathering: AspectGathering::Neither,
        ..Readings::RECORDING_ENGINE
    };

    /// The texts' reading wherever a text read settles one, the recording
    /// engine's elsewhere: the default.
    pub const TEXTS: Readings = Readings {
        bhaga: Bhaga::Running,
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
