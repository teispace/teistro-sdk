//! A founded chart's antiscia (`03-design/western-antiscia.md`).

use teistro_chart::foundation::ChartFoundation;
use teistro_core::error::Error;
use teistro_serial::Document;
use teistro_western::{
    Antiscia, AntisciaRequest, AntiscionRow, PlanetAt, SynastryRequest, antiscia, synastry_antiscia,
};

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
        antiscia(&reflected(&chart.foundation), request)
    }

    /// The **antiscia across two charts**: every planet of `first` whose
    /// reflection falls within the orb of a planet of `second`, under the
    /// request's `antiscia` (Lilly's moieties when it names none), closest
    /// first, the chart's planet first in each row. The planets are
    /// reflected from their tropical longitude, so neither the charts'
    /// zodiacs nor the request's `zodiac` change anything.
    ///
    /// ```no_run
    /// # use teistro::{AntisciaRequest, ChartRequest, Context, Ephemeris, SynastryRequest};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (his, hers): ((JulianDay<Utc>, ChartRequest), (JulianDay<Utc>, ChartRequest)) = todo!();
    /// let first = sdk.chart().reading(his.0, &his.1)?.value;
    /// let second = sdk.chart().reading(hers.0, &hers.1)?.value;
    /// let asked = SynastryRequest::default().with_antiscia(AntisciaRequest::default());
    /// for row in sdk.chart().synastry_antiscia(&first, &second, &asked)? {
    ///     println!("{:?} and {:?}, {:.2}° apart", row.first, row.second, row.apart_deg);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`SynastryRequest::check`] refuses.
    pub fn synastry_antiscia(
        self,
        first: &Document,
        second: &Document,
        request: &SynastryRequest,
    ) -> Result<Vec<AntiscionRow>, Error> {
        request.check()?;
        synastry_antiscia(
            &reflected(&first.foundation),
            &reflected(&second.foundation),
            &request.antiscia.clone().unwrap_or_default(),
        )
    }
}

/// A chart's planets as the antiscia read them: each at its tropical
/// longitude.
pub(crate) fn reflected(foundation: &ChartFoundation) -> Vec<PlanetAt> {
    planets(foundation)
        .map(|at| PlanetAt::new(at.graha, at.tropical_deg))
        .collect()
}
