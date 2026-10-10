//! A founded chart's declinations, and the parallels among its planets
//! (`03-design/western-declinations.md`).

use serde::{Deserialize, Serialize};
use teistro_core::angle::declination_deg;
use teistro_core::error::Error;
use teistro_serial::Document;
use teistro_western::{
    Declined, DeclinedPoint, ParallelRequest, ParallelRow, SynastryParallelRow, SynastryRequest,
    parallels, synastry_parallels,
};

use crate::area::ChartArea;
use crate::western_aspects::planets;

/// A chart's distances from the equator: each planet's, and its two
/// angles', all degrees north of it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Declinations {
    /// The true obliquity at the chart's instant, which every declination
    /// was turned by.
    pub obliquity_deg: f64,
    /// The seven planets, and the outer three when the chart placed them,
    /// in the catalogue's order.
    pub grahas: Vec<Declined>,
    /// The lagna's: the Sun's at that degree (Leo, p. 141).
    pub lagna_deg: f64,
    /// The midheaven's, read the same way.
    pub midheaven_deg: f64,
}

impl Declinations {
    /// One planet's declination, when the chart placed it.
    #[must_use]
    pub fn graha(&self, graha: teistro_core::catalogue::Graha) -> Option<f64> {
        self.grahas
            .iter()
            .find(|one| one.graha == graha)
            .map(|one| one.declination_deg)
    }
}

impl ChartArea<'_> {
    /// A chart's **declinations**: each planet's distance from the
    /// equator, from its tropical longitude, its ecliptic latitude and the
    /// true obliquity at the chart's instant, and the lagna's and the
    /// midheaven's as degrees of the ecliptic. None depends on the chart's
    /// zodiac.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris};
    /// # use teistro::catalogue::Graha;
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (instant, request): (JulianDay<Utc>, ChartRequest) = todo!();
    /// let chart = sdk.chart().reading(instant, &request)?.value;
    /// let read = sdk.chart().declinations(&chart)?;
    /// println!("the Sun {:+.2}°, the lagna {:+.2}°", read.graha(Graha::Sun).unwrap_or_default(), read.lagna_deg);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`ChartArea::angles`] refuses, which gives the obliquity.
    pub fn declinations(self, chart: &Document) -> Result<Declinations, Error> {
        let foundation = &chart.foundation;
        let angles = self.angles(chart)?;
        let obliquity = angles.obliquity_deg;
        let of_degree = |longitude_deg: f64| {
            declination_deg(foundation.zodiac.to_tropical(longitude_deg), 0.0, obliquity)
        };
        Ok(Declinations {
            obliquity_deg: obliquity,
            grahas: planets(foundation)
                .map(|at| {
                    Declined::new(
                        at.graha,
                        declination_deg(at.tropical_deg, at.latitude_deg, obliquity),
                    )
                })
                .collect(),
            lagna_deg: of_degree(angles.ascendant_deg),
            midheaven_deg: of_degree(angles.midheaven_deg),
        })
    }

    /// The **parallels** among a chart's planets: every pair the same
    /// distance from the equator within the orb, on either side of it
    /// (Leo, *How to Judge a Nativity*, pp. 42, 47; C243), closest first.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, ParallelRequest};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (instant, request): (JulianDay<Utc>, ChartRequest) = todo!();
    /// let chart = sdk.chart().reading(instant, &request)?.value;
    /// for row in sdk.chart().parallels(&chart, &ParallelRequest::default())? {
    ///     let kind = if row.contrary { "contra-parallel" } else { "parallel" };
    ///     println!("{:?} {kind} {:?}, {:.2}° apart", row.first, row.second, row.apart_deg);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`ParallelRequest::check`] refuses, and what
    /// [`ChartArea::declinations`] does.
    pub fn parallels(
        self,
        chart: &Document,
        request: &ParallelRequest,
    ) -> Result<Vec<ParallelRow>, Error> {
        request.check()?;
        parallels(&self.declinations(chart)?.grahas, request)
    }

    /// The **parallels across two charts**: every point of `first` the
    /// same distance from the equator as a point of `second`, within the
    /// orb of the request's `parallels` (Leo's 1° when it names none), on
    /// either side of the equator (C243), closest first. The points are
    /// each chart's planets and, unless the request leaves it out, its
    /// lagna. A declination does not depend on the zodiac, so the
    /// request's `zodiac` changes nothing here.
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, ParallelRequest, SynastryRequest};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (his, hers): ((JulianDay<Utc>, ChartRequest), (JulianDay<Utc>, ChartRequest)) = todo!();
    /// let first = sdk.chart().reading(his.0, &his.1)?.value;
    /// let second = sdk.chart().reading(hers.0, &hers.1)?.value;
    /// let asked = SynastryRequest::default().with_parallels(ParallelRequest::default());
    /// for row in sdk.chart().synastry_parallels(&first, &second, &asked)? {
    ///     println!("{:?} and {:?}, {:.2}° apart", row.first, row.second, row.apart_deg);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`SynastryRequest::check`] refuses, and what
    /// [`ChartArea::declinations`] does for either chart.
    pub fn synastry_parallels(
        self,
        first: &Document,
        second: &Document,
        request: &SynastryRequest,
    ) -> Result<Vec<SynastryParallelRow>, Error> {
        request.check()?;
        self.declinations(first)?.parallels_across(
            &self.declinations(second)?,
            request.lagna,
            request.parallels.unwrap_or_default(),
        )
    }
}

impl Declinations {
    /// The parallels across this chart's declinations and `theirs`, each
    /// chart's lagna beside its planets when `lagna`.
    pub(crate) fn parallels_across(
        &self,
        theirs: &Declinations,
        lagna: bool,
        request: ParallelRequest,
    ) -> Result<Vec<SynastryParallelRow>, Error> {
        synastry_parallels(&self.points(lagna), &theirs.points(lagna), &request)
    }

    /// The chart's points as a synastry's parallels read them: its planets,
    /// then its lagna when asked.
    fn points(&self, lagna: bool) -> Vec<DeclinedPoint> {
        self.grahas
            .iter()
            .map(|one| DeclinedPoint::graha(one.graha, one.declination_deg))
            .chain(lagna.then(|| DeclinedPoint::lagna(self.lagna_deg)))
            .collect()
    }
}
