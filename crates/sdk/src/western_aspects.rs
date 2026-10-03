//! The Western aspects a founded chart holds (`03-design/western-aspects.md`),
//! and those between two charts (`03-design/western-synastry.md`).

use teistro_chart::foundation::{ChartFoundation, GrahaPosition};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_serial::Document;
use teistro_western::{
    AspectRequest, Placed, SynastryPoint, SynastryRequest, SynastryRow, SynastryZodiac,
    WesternAspectRow, aspects, synastry,
};

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
        let planets: Vec<Placed> = planets(&chart.foundation)
            .map(|at| Placed::new(at.graha, at.longitude_deg, at.speed_deg_per_day))
            .collect();
        aspects(&planets, request)
    }

    /// **Synastry**: the Western aspects between two charts, every point
    /// of `first` against every point of `second`, closest first (Leo,
    /// *How to Judge a Nativity*, pp. 189, 221–223).
    ///
    /// The points are each chart's planets, as [`ChartArea::western_aspects`]
    /// reads them, and its lagna unless the request leaves it out; the
    /// lagna takes a planet's orb (C242). The two are compared in the
    /// tropical zodiac unless the request asks for each chart's own
    /// (C241).
    ///
    /// ```no_run
    /// # use teistro::{ChartRequest, Context, Ephemeris, SynastryRequest};
    /// # use teistro::quantity::{JulianDay, Utc};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (his, hers): ((JulianDay<Utc>, ChartRequest), (JulianDay<Utc>, ChartRequest)) = todo!();
    /// let first = sdk.chart().reading(his.0, &his.1.with_outer_planets())?.value;
    /// let second = sdk.chart().reading(hers.0, &hers.1.with_outer_planets())?.value;
    /// for row in sdk.chart().synastry(&first, &second, &SynastryRequest::default())? {
    ///     println!("{:?} {:?} {:?}, {:.1}° from exact", row.first, row.aspect, row.second, row.from_exact_deg);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`SynastryRequest::check`] refuses, a point the model has no
    /// orb for, and, read in each chart's own zodiac, two charts founded
    /// in different ones (`zodiac`).
    pub fn synastry(
        self,
        first: &Document,
        second: &Document,
        request: &SynastryRequest,
    ) -> Result<Vec<SynastryRow>, Error> {
        request.check()?;
        let (a, b) = (&first.foundation, &second.foundation);
        if request.zodiac == SynastryZodiac::Charts && a.zodiac.ayanamsha != b.zodiac.ayanamsha {
            return Err(Error::invalid_arg(
                "the two charts were founded in different zodiacs, so their longitudes do not compare",
            )
            .with_field("zodiac")
            .with_hint("found both under one zodiac, or compare them in the tropical one"));
        }
        synastry(&points(a, request), &points(b, request), request)
    }
}

/// The planets a Western table reads: the seven, never the nodes, and the
/// outer three when the chart placed them.
fn planets(foundation: &ChartFoundation) -> impl Iterator<Item = &GrahaPosition> {
    foundation
        .grahas
        .iter()
        .filter(|at| !matches!(at.graha, Graha::Rahu | Graha::Ketu))
        .chain(&foundation.outer)
}

/// One chart's points for a synastry, in the zodiac the request compares
/// them in, its lagna last when asked.
fn points(foundation: &ChartFoundation, request: &SynastryRequest) -> Vec<SynastryPoint> {
    let tropical = request.zodiac == SynastryZodiac::Tropical;
    let lagna = request.lagna.then(|| {
        SynastryPoint::lagna(if tropical {
            foundation.zodiac.to_tropical(foundation.lagna_deg)
        } else {
            foundation.lagna_deg
        })
    });
    planets(foundation)
        .map(|at| {
            SynastryPoint::graha(
                at.graha,
                if tropical {
                    at.tropical_deg
                } else {
                    at.longitude_deg
                },
            )
        })
        .chain(lagna)
        .collect()
}
