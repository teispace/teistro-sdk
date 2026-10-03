//! The Western aspects a founded chart holds (`03-design/western-aspects.md`).

use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_serial::Document;
use teistro_western::{AspectRequest, Placed, WesternAspectRow, aspects};

use crate::area::ChartArea;

impl ChartArea<'_> {
    /// The **Western aspects** a chart holds: every pair of its planets
    /// at one of the request's aspects, inside the orb its model allows,
    /// closest first.
    ///
    /// The planets are the seven, and Uranus, Neptune and Pluto when the
    /// chart placed them ([`ChartRequest::with_outer_planets`]); the nodes
    /// are not read. The default request is Leo's nine aspects under his
    /// orbs (C240); [`AspectRequest::lilly`] is the Ptolemaic five under
    /// Lilly's moieties.
    ///
    /// ```no_run
    /// # use teistro::{AspectRequest, ChartRequest, Context, Ephemeris};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (instant, request): (JulianDay<Utc>, ChartRequest) = todo!();
    /// let chart = sdk.chart().reading(instant, &request.with_outer_planets())?.value;
    /// for row in sdk.chart().western_aspects(&chart, &AspectRequest::default())? {
    ///     println!("{:?} {:?} {:?}, {:.1}° from exact", row.first, row.aspect, row.second, row.from_exact_deg);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`AspectRequest::check`] refuses, and a planet the model has
    /// no orb for, such as Uranus under Lilly's moieties.
    ///
    /// [`ChartRequest::with_outer_planets`]: crate::ChartRequest::with_outer_planets
    pub fn western_aspects(
        self,
        chart: &Document,
        request: &AspectRequest,
    ) -> Result<Vec<WesternAspectRow>, Error> {
        let foundation = &chart.foundation;
        let planets: Vec<Placed> = foundation
            .grahas
            .iter()
            .filter(|at| !matches!(at.graha, Graha::Rahu | Graha::Ketu))
            .chain(&foundation.outer)
            .map(|at| Placed::new(at.graha, at.longitude_deg, at.speed_deg_per_day))
            .collect();
        aspects(&planets, request)
    }
}
