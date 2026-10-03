//! A founded chart's equal distances (`03-design/western-midpoints.md`).

use teistro_core::error::Error;
use teistro_serial::Document;
use teistro_western::{
    MidpointRequest, MidpointRow, PlanetAt, SynastryMidpointRow, SynastryRequest, midpoints,
    synastry_midpoints,
};

use crate::area::ChartArea;
use crate::composites::refuse_mixed_zodiacs;
use crate::western_aspects::{planets, planets_in};

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

    /// The **equal distances across two charts** (decision 9): every
    /// planet of `second` within the orb of the axis through two of
    /// `first`'s, and every planet of `first` on an axis of two of
    /// `second`'s, under the request's `midpoints` (0.5° when it names
    /// none), closest first. Read in the request's `zodiac`, as the
    /// aspects across are: two births years apart stand in different
    /// sidereal frames.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, MidpointRequest, SynastryRequest};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (his, hers): ((JulianDay<Utc>, ChartRequest), (JulianDay<Utc>, ChartRequest)) = todo!();
    /// let first = sdk.chart().reading(his.0, &his.1)?.value;
    /// let second = sdk.chart().reading(hers.0, &hers.1)?.value;
    /// let asked = SynastryRequest::default().with_midpoints(MidpointRequest::default());
    /// for row in sdk.chart().synastry_midpoints(&first, &second, &asked)? {
    ///     let whose = if row.partners_pair { "her" } else { "his" };
    ///     println!("{:?} on {whose} {:?} and {:?}", row.middle, row.first, row.second);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`SynastryRequest::check`] refuses, and in each chart's own
    /// zodiac two charts founded in different ones (`zodiac`).
    pub fn synastry_midpoints(
        self,
        first: &Document,
        second: &Document,
        request: &SynastryRequest,
    ) -> Result<Vec<SynastryMidpointRow>, Error> {
        request.check()?;
        refuse_mixed_zodiacs(&first.foundation, &second.foundation, request.zodiac)?;
        let bodies = |chart: &Document| -> Vec<PlanetAt> {
            planets_in(&chart.foundation, request.zodiac)
                .map(|(at, longitude)| PlanetAt::new(at.graha, longitude))
                .collect()
        };
        synastry_midpoints(
            &bodies(first),
            &bodies(second),
            &request.midpoints.unwrap_or_default(),
        )
    }
}
