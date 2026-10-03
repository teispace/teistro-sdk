//! A founded chart's equal distances (`03-design/western-midpoints.md`).

use teistro_core::error::Error;
use teistro_serial::Document;
use teistro_western::{MidpointRequest, MidpointRow, PlanetAt, midpoints};

use crate::area::ChartArea;
use crate::western_aspects::planets;

impl ChartArea<'_> {
    /// A chart's **equal distances** (Leo, *How to Judge a Nativity*,
    /// pp. 47–48): every planet standing within the request's orb of the
    /// axis through the midpoint of two others (0.5° by default, C245), on
    /// the shorter arc's midpoint or opposite it (C246), closest first.
    /// The chart's own longitudes are read, so a sidereal chart gives the
    /// rows a tropical one of the same birth does.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, MidpointRequest};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (instant, request): (JulianDay<Utc>, ChartRequest) = todo!();
    /// let chart = sdk.chart().reading(instant, &request)?.value;
    /// for row in sdk.chart().midpoints(&chart, &MidpointRequest::default())? {
    ///     println!(
    ///         "{:?} stands {:.1}° from {:?} and {:?}",
    ///         row.middle, row.distance_deg, row.first, row.second
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`MidpointRequest::check`] refuses.
    pub fn midpoints(
        self,
        chart: &Document,
        request: &MidpointRequest,
    ) -> Result<Vec<MidpointRow>, Error> {
        let bodies: Vec<PlanetAt> = planets(&chart.foundation)
            .map(|at| PlanetAt::new(at.graha, at.longitude_deg))
            .collect();
        midpoints(&bodies, request)
    }
}
