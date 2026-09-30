//! A KP reading whole: the chart, its significators and the ruling
//! planets of its moment.

use serde::{Deserialize, Serialize};

use crate::chart::KpChart;
use crate::ruling::RulingPlanets;
use crate::significators::Significators;

/// A chart read as KP, with its significators and the ruling planets at
/// its moment: what a binding receives for each chart.
///
/// For a horary chart the cusps are the number's and the ruling planets
/// still the moment's own, which the Reader takes "at the moment of
/// judgment" (`03-design/kp.md`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct KpReading {
    /// The chart.
    pub chart: KpChart,
    /// Its significators.
    pub significators: Significators,
    /// The ruling planets at its moment.
    pub ruling: RulingPlanets,
}

impl KpReading {
    /// The three together.
    #[must_use]
    pub const fn new(
        chart: KpChart,
        significators: Significators,
        ruling: RulingPlanets,
    ) -> KpReading {
        KpReading {
            chart,
            significators,
            ruling,
        }
    }
}
