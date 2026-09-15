//! What a rule reads of a chart, and the choices the language leaves open.

use teistro_core::catalogue::{CharaKaraka, Dignity, Graha, Rashi};
use teistro_core::settings::NodeAspects;

use crate::language::{Body, House};

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

/// A chart as the rules read it: the nine grahas and the lagna, in
/// [`Body::ALL`]'s order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuleChart {
    /// Each body's placement, the Sun to Ketu and then the lagna.
    pub placements: [Placement; 10],
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

/// A choice at every place the condition language leaves a meaning open
/// (`03-design/yogas-measured.md`); the default is the recording engine's.
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
}

impl Default for Readings {
    fn default() -> Readings {
        Readings::RECORDING_ENGINE
    }
}

impl Readings {
    /// The recording engine's reading at every place it has one, and the
    /// text's where it has none: the default.
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
