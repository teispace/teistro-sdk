//! A dasha as a chart document carries it: what it was computed from, its
//! balance, and its periods to the depth the settings ask for.
//!
//! The document holds rows and not the kernel, so a consumer in any
//! language reads the timeline as data. A caller who wants deeper periods,
//! or the chain at an instant, rebuilds the [`Dasha`] the reading came from
//! and asks it: the reading carries everything that takes.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{DashaSystem, Graha, Nakshatra};
use teistro_core::interval::Interval;
use teistro_core::quantity::Depth;

use crate::balance::BalanceAtBirth;
use crate::tree::{Dasha, Period, Rules};

/// One dasha of a chart.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DashaReading {
    /// Which system.
    pub system: DashaSystem,
    /// The choices it was computed under.
    pub rules: Rules,
    /// The nakshatra the Moon stood in, which seeds it.
    pub seed: Nakshatra,
    /// The lord it starts with.
    pub first_lord: Graha,
    /// Whether the seed lay outside a conditional system's nakshatras, and
    /// started at the first lord because the rules let it.
    pub overflow: bool,
    /// The Moon's stay in its nakshatra, when the balance was temporal.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub moon_span: Option<Interval>,
    /// What remained of the first period at birth.
    pub balance: BalanceAtBirth,
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
            lord: period.lord,
            interval: period.interval,
        }
    }
}

impl DashaReading {
    /// A dasha's reading, its periods to `depth` levels.
    ///
    /// `moon_span` is the Moon's stay the balance read, when it read one.
    #[must_use]
    pub fn of(dasha: &Dasha, depth: Depth, moon_span: Option<Interval>) -> DashaReading {
        let row = dasha.row();
        let seat = dasha.seat();
        let mut periods = Vec::new();
        let levels = usize::from(depth.get());
        for maha in dasha.mahadashas() {
            collect(dasha, maha, levels, &mut periods);
        }
        DashaReading {
            system: row.system,
            rules: dasha.rules(),
            seed: dasha.seed(),
            first_lord: row
                .lords
                .get(seat.lord)
                .map_or(Graha::Ketu, |lord| lord.graha),
            overflow: seat.overflow,
            moon_span,
            balance: dasha.balance(),
            depth,
            periods,
        }
    }
}

fn collect(dasha: &Dasha, period: Period, depth: usize, out: &mut Vec<PeriodRow>) {
    out.push(PeriodRow::of(&period));
    if period.path.depth() < depth {
        for child in dasha.children(&period) {
            collect(dasha, child, depth, out);
        }
    }
}
