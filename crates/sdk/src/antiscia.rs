//! A founded chart's antiscia (`03-design/western-antiscia.md`).

use teistro_core::error::Error;
use teistro_serial::Document;
use teistro_western::{Antiscia, AntisciaRequest, Reflected, antiscia};

use crate::area::ChartArea;
use crate::western_aspects::planets;

impl ChartArea<'_> {
    /// A chart's **antiscia** (Lilly, *Christian Astrology*, pp. 90–92):
    /// each planet's reflection about the solstices and about the
    /// equinoxes, and every pair standing in antiscion or contrantiscion
    /// within the request's orbs, read at the conjunction (Lilly's moieties
    /// by default, C244), closest first. The planets are reflected from
    /// their tropical longitude, so the chart's zodiac changes nothing.
    ///
    /// ```no_run
    /// # use teistro::{AntisciaRequest, ChartRequest, Context, Ephemeris};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (instant, request): (JulianDay<Utc>, ChartRequest) = todo!();
    /// let chart = sdk.chart().reading(instant, &request)?.value;
    /// let read = sdk.chart().antiscia(&chart, &AntisciaRequest::default())?;
    /// for row in &read.pairs {
    ///     let kind = if row.contrary { "contrantiscion" } else { "antiscion" };
    ///     println!("{:?} {kind} {:?}, {:.2}° apart", row.first, row.second, row.apart_deg);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`AntisciaRequest::check`] refuses.
    pub fn antiscia(self, chart: &Document, request: &AntisciaRequest) -> Result<Antiscia, Error> {
        let bodies: Vec<Reflected> = planets(&chart.foundation)
            .map(|at| Reflected::new(at.graha, at.tropical_deg))
            .collect();
        antiscia(&bodies, request)
    }
}
