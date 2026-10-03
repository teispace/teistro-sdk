//! A founded chart's harmonics (`03-design/western-harmonics.md`).

use teistro_core::error::Error;
use teistro_serial::Document;
use teistro_western::{HarmonicChart, HarmonicRequest, PlanetAt, harmonic_chart};

use crate::area::ChartArea;
use crate::western_aspects::planets;

impl ChartArea<'_> {
    /// A chart's **harmonic chart** (Addey, *Harmonics in Astrology*):
    /// each planet the chart places, the ascendant and the midheaven at
    /// its longitude multiplied by the request's whole number, in its
    /// equal house from the harmonic ascendant (C254), and every pair
    /// meeting within the orb, 12° by default (C252), closest first. Read
    /// in the chart's own zodiac (C253), so the 9th of a sidereal chart
    /// puts every point in its navamsa's sign.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, HarmonicRequest};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (instant, request): (JulianDay<Utc>, ChartRequest) = todo!();
    /// let chart = sdk.chart().reading(instant, &request)?.value;
    /// let fifth = sdk.chart().harmonic(&chart, &HarmonicRequest::of(5))?;
    /// for row in &fifth.rows {
    ///     println!("{:?} and {:?}, {:.1}° apart in the 5th", row.first, row.second, row.apart_deg);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`HarmonicRequest::check`] refuses, and a chart whose angles
    /// cannot be read.
    pub fn harmonic(
        self,
        chart: &Document,
        request: &HarmonicRequest,
    ) -> Result<HarmonicChart, Error> {
        let angles = self.angles(chart)?;
        let bodies: Vec<PlanetAt> = planets(&chart.foundation)
            .map(|at| PlanetAt::new(at.graha, at.longitude_deg))
            .collect();
        harmonic_chart(&bodies, angles.ascendant_deg, angles.midheaven_deg, request)
    }
}
