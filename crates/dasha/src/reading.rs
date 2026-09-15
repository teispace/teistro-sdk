//! A dasha as a chart document carries it: what it was computed from, its
//! balance, and its periods to the depth the settings ask for.
//!
//! The document holds rows and not the kernel, so a consumer in any
//! language reads the timeline as data. A caller who wants deeper periods,
//! or the chain at an instant, rebuilds the [`DashaCursor`] the reading came
//! from and asks it: the reading carries everything that takes.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{DashaSystem, Graha, Nakshatra, Rashi};
use teistro_core::interval::Interval;
use teistro_core::quantity::Depth;

use crate::balance::BalanceAtBirth;
use crate::kalachakra::{KalachakraDasha, KalachakraRules};
use crate::rashi::RashiDasha;
use crate::tree::{Dasha, Period, Rules, Timeline};

/// One dasha of a chart.
///
/// A nakshatra-seeded dasha carries the nakshatra that seeds it and its
/// balance at birth; a sign-based one has neither, and its periods carry
/// their signs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DashaReading {
    /// Which system.
    pub system: DashaSystem,
    /// The choices it was computed under. A sign-based dasha reads only the
    /// year length and what follows the cycle.
    pub rules: Rules,
    /// The Kalachakra's own choices, when it is the Kalachakra.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub kalachakra: Option<KalachakraRules>,
    /// The nakshatra the Moon stood in, which seeds a nakshatra-seeded
    /// dasha; nothing for a sign-based one.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub seed: Option<Nakshatra>,
    /// The lord it starts with.
    pub first_lord: Graha,
    /// Whether the seed lay outside a conditional system's nakshatras, and
    /// started at the first lord because the rules let it.
    pub overflow: bool,
    /// The Moon's stay in its nakshatra, when the balance was temporal.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub moon_span: Option<Interval>,
    /// What remained of the first period at birth; nothing for a sign-based
    /// dasha, whose first period runs whole from birth.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub balance: Option<BalanceAtBirth>,
    /// How many levels the periods go down.
    pub depth: Depth,
    /// Every period of the birth cycle to `depth`, depth first in time
    /// order: a mahadasha, then its antardashas and theirs, then the next.
    pub periods: Vec<PeriodRow>,
}

/// One period, as a row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PeriodRow {
    /// Where it sits: the place at each level, from the mahadasha down,
    /// joined by `/` (`2/5/3`).
    pub path: String,
    /// The sign it is the period of, in a sign-based dasha.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sign: Option<Rashi>,
    /// Its lord.
    pub lord: Graha,
    /// When it runs.
    pub interval: Interval,
}

impl PeriodRow {
    /// A period as a row.
    #[must_use]
    pub fn of(period: &Period) -> PeriodRow {
        PeriodRow {
            path: period.path.to_string(),
            sign: period.sign,
            lord: period.lord,
            interval: period.interval,
        }
    }
}

impl DashaReading {
    /// A nakshatra-seeded dasha's reading, its periods to `depth` levels.
    ///
    /// `moon_span` is the Moon's stay the balance read, when it read one.
    #[must_use]
    pub fn of(dasha: &Dasha, depth: Depth, moon_span: Option<Interval>) -> DashaReading {
        let row = dasha.row();
        let seat = dasha.seat();
        DashaReading {
            system: row.system,
            rules: dasha.rules(),
            kalachakra: None,
            seed: Some(dasha.seed()),
            first_lord: row
                .lords
                .get(seat.lord)
                .map_or(Graha::Ketu, |lord| lord.graha),
            overflow: seat.overflow,
            moon_span,
            balance: Some(dasha.balance()),
            depth,
            periods: rows(dasha, depth),
        }
    }

    /// A sign-based dasha's reading under `rules`, its periods to `depth`
    /// levels.
    #[must_use]
    pub fn of_rashi(dasha: &RashiDasha, rules: Rules, depth: Depth) -> DashaReading {
        let periods = rows(dasha, depth);
        DashaReading {
            system: dasha.row().system,
            rules,
            kalachakra: None,
            seed: None,
            first_lord: periods.first().map_or(Graha::Sun, |row| row.lord),
            overflow: false,
            moon_span: None,
            balance: None,
            depth,
            periods,
        }
    }
}

impl DashaReading {
    /// The Kalachakra's reading under the dasha group's `rules`, its periods
    /// to `depth` levels or to the antardashas it stops at, whichever is
    /// shallower, and the depth it carries says which (crux C58).
    #[must_use]
    pub fn of_kalachakra(
        dasha: &KalachakraDasha,
        rules: Rules,
        depth: Depth,
        moon_span: Option<Interval>,
    ) -> DashaReading {
        let carried = Depth::try_new(depth.get().min(2)).unwrap_or(depth);
        let periods = rows(dasha, carried);
        DashaReading {
            system: DashaSystem::Kalachakra,
            rules,
            kalachakra: Some(dasha.rules()),
            seed: Some(dasha.seed()),
            first_lord: periods.first().map_or(Graha::Sun, |row| row.lord),
            overflow: false,
            moon_span,
            balance: Some(dasha.balance()),
            depth: carried,
            periods,
        }
    }
}

/// Every period of the birth cycle to `depth`, depth first, as rows.
fn rows(timeline: &impl Timeline, depth: Depth) -> Vec<PeriodRow> {
    fn collect(timeline: &impl Timeline, period: Period, depth: usize, out: &mut Vec<PeriodRow>) {
        out.push(PeriodRow::of(&period));
        if period.path.depth() < depth {
            for child in timeline.children(&period) {
                collect(timeline, child, depth, out);
            }
        }
    }
    let mut out = Vec::new();
    let levels = usize::from(depth.get());
    for maha in timeline.mahadashas() {
        collect(timeline, maha, levels, &mut out);
    }
    out
}

/// The cursor behind any dasha a document carries: nakshatra-seeded or
/// sign-based, read the same way through [`Timeline`].
#[derive(Clone, Debug, PartialEq)]
pub enum DashaCursor {
    /// A nakshatra-seeded dasha.
    Nakshatra(Dasha),
    /// A sign-based dasha.
    Rashi(RashiDasha),
    /// The Kalachakra.
    Kalachakra(KalachakraDasha),
}

impl Timeline for DashaCursor {
    fn breadth(&self) -> usize {
        match self {
            DashaCursor::Nakshatra(dasha) => dasha.breadth(),
            DashaCursor::Rashi(dasha) => dasha.breadth(),
            DashaCursor::Kalachakra(dasha) => dasha.breadth(),
        }
    }

    fn mahadasha(&self, cycle: u32, index: usize) -> Option<Period> {
        match self {
            DashaCursor::Nakshatra(dasha) => dasha.mahadasha(cycle, index),
            DashaCursor::Rashi(dasha) => dasha.mahadasha(cycle, index),
            DashaCursor::Kalachakra(dasha) => dasha.mahadasha(cycle, index),
        }
    }

    fn mahadasha_at(&self, instant: f64) -> Option<Period> {
        match self {
            DashaCursor::Nakshatra(dasha) => dasha.mahadasha_at(instant),
            DashaCursor::Rashi(dasha) => dasha.mahadasha_at(instant),
            DashaCursor::Kalachakra(dasha) => dasha.mahadasha_at(instant),
        }
    }

    fn child(&self, parent: &Period, index: usize) -> Option<Period> {
        match self {
            DashaCursor::Nakshatra(dasha) => dasha.child(parent, index),
            DashaCursor::Rashi(dasha) => dasha.child(parent, index),
            DashaCursor::Kalachakra(dasha) => dasha.child(parent, index),
        }
    }
}

impl DashaCursor {
    /// The nakshatra-seeded dasha, when it is one.
    #[must_use]
    pub const fn nakshatra(&self) -> Option<&Dasha> {
        match self {
            DashaCursor::Nakshatra(dasha) => Some(dasha),
            _ => None,
        }
    }

    /// The sign-based dasha, when it is one.
    #[must_use]
    pub const fn rashi(&self) -> Option<&RashiDasha> {
        match self {
            DashaCursor::Rashi(dasha) => Some(dasha),
            _ => None,
        }
    }

    /// The Kalachakra, when it is the Kalachakra.
    #[must_use]
    pub const fn kalachakra(&self) -> Option<&KalachakraDasha> {
        match self {
            DashaCursor::Kalachakra(dasha) => Some(dasha),
            _ => None,
        }
    }
}
