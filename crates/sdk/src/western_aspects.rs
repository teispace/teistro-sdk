//! The Western aspects a founded chart holds (`03-design/western-aspects.md`),
//! and those between two charts (`03-design/western-synastry.md`).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use teistro_chart::foundation::{ChartFoundation, GrahaPosition};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_serial::Document;
use teistro_western::{
    AntiscionRow, AspectRequest, Composite, Placed, SynastryParallelRow, SynastryPoint,
    SynastryRequest, SynastryRow, SynastryZodiac, WesternAspectRow, aspects, synastry,
    synastry_antiscia,
};

use crate::area::ChartArea;
use crate::reading::ChartRequest;

/// The record's name where a binding sends it, which a refusal is named
/// under.
const SYNASTRY: &str = "synastry";

/// A second birth, which every chart of a batch is read against.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Partner {
    /// The birth's instant.
    pub instant: JulianDay<Utc>,
    /// Where it happened.
    pub place: Place,
    /// Its civil clock, which a chart's day is reckoned by; UTC when left
    /// out.
    #[serde(rename = "utcOffsetSeconds", default)]
    pub utc_offset: UtcOffset,
}

/// A synastry against a partner's birth, as a binding asks it: the
/// partner, and the [`SynastryRequest`] laid flat beside it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "Map<String, Value>")]
pub struct PartnerSynastry {
    /// Whose chart every chart is read against.
    pub partner: Partner,
    /// The aspects, orbs, lagna and zodiac.
    #[serde(flatten)]
    pub request: SynastryRequest,
    /// Whether each chart's Davison birth with the partner is answered
    /// too ([`PartnerSynastry::davisons`], C248). False by default.
    #[serde(skip_serializing_if = "core::ops::Not::not")]
    pub davison: bool,
}

impl PartnerSynastry {
    /// A partner's synastry, under a request.
    #[must_use]
    pub const fn new(partner: Partner, request: SynastryRequest) -> PartnerSynastry {
        PartnerSynastry {
            partner,
            request,
            davison: false,
        }
    }

    /// Answers each chart's Davison birth with the partner too, or not.
    #[must_use]
    pub const fn with_davison(mut self, davison: bool) -> PartnerSynastry {
        self.davison = davison;
        self
    }

    /// Each chart's **Davison birth** with the partner ([`Partner::davison`],
    /// C248), the chart's birth read on `clock`, one a chart in the order
    /// given; `None` unless [`PartnerSynastry::davison`] asked. A document
    /// does not keep the clock it was founded on, so the caller gives it,
    /// as the boundary gives the batch's.
    ///
    /// ```
    /// use teistro::{Partner, PartnerSynastry, SynastryRequest, UtcOffset};
    /// use teistro::quantity::{JulianDay, Place, Utc};
    ///
    /// let hers = Partner {
    ///     instant: JulianDay::<Utc>::literal(2_403_113.499_305_556),
    ///     place: Place::try_from_degrees(51.5058, -0.1878, 0.0)?,
    ///     utc_offset: UtcOffset::UTC,
    /// };
    /// let asked = PartnerSynastry::new(hers, SynastryRequest::default());
    /// assert_eq!(asked.davisons(&[], UtcOffset::UTC)?, None);
    /// assert_eq!(asked.with_davison(true).davisons(&[], UtcOffset::UTC)?, Some(vec![]));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// What [`Partner::davison`] refuses, hinted with the chart's place in
    /// the list.
    pub fn davisons(
        &self,
        charts: &[Document],
        clock: UtcOffset,
    ) -> Result<Option<Vec<Partner>>, Error> {
        if !self.davison {
            return Ok(None);
        }
        charts
            .iter()
            .enumerate()
            .map(|(at, chart)| {
                Partner {
                    instant: chart.foundation.instant,
                    place: chart.foundation.place,
                    utc_offset: clock,
                }
                .davison(&self.partner)
                .map_err(|why| why.with_field("davison").with_hint(format!("chart {at}")))
            })
            .collect::<Result<_, _>>()
            .map(Some)
    }

    /// The record a binding sends, as JSON: `partner`, `{"instant": jd,
    /// "place": {"latitude", "longitude", "altitude"}, "utcOffsetSeconds"}`,
    /// and every field of [`SynastryRequest::from_json`] beside it.
    ///
    /// ```
    /// use teistro::PartnerSynastry;
    ///
    /// let asked = PartnerSynastry::from_json(
    ///     r#"{"partner": {"instant": 2403113.4993, "place": {"latitude": 51.5058, "longitude": -0.1878, "altitude": 0}}, "lagna": false}"#,
    /// )?;
    /// assert!(!asked.request.lagna);
    /// // A partner's place is held to the same bounds as any place.
    /// let north = PartnerSynastry::from_json(
    ///     r#"{"partner": {"instant": 2403113.4993, "place": {"latitude": 95, "longitude": 0, "altitude": 0}}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(north.field(), Some("synastry.partner.place.latitude"));
    /// // A request field is named as the caller wrote it, beside the partner.
    /// let zodiac = PartnerSynastry::from_json(
    ///     r#"{"partner": {"instant": 2403113.4993, "place": {"latitude": 51.5, "longitude": 0, "altitude": 0}}, "zodiac": "SIDEREAL"}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(zodiac.field(), Some("synastry.zodiac"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a value out of its bounds, and whatever
    /// [`SynastryRequest::check`] refuses, each named under `synastry`.
    pub fn from_json(text: &str) -> Result<PartnerSynastry, Error> {
        let given: Map<String, Value> = teistro_core::strict::read(text, SYNASTRY)?;
        let asked = PartnerSynastry::split(given, SYNASTRY)?;
        asked.request.check().map_err(|why| why.under(SYNASTRY))?;
        Ok(asked)
    }

    /// The partner taken out, and the rest read as the request, each by
    /// its own path under `root`: `flatten` would buffer the request and
    /// name a refusal inside it by the record alone.
    fn split(mut given: Map<String, Value>, root: &str) -> Result<PartnerSynastry, Error> {
        let at = format!("{root}.partner");
        let partner = given.remove("partner").ok_or_else(|| {
            Error::invalid_arg("a synastry needs the partner's birth")
                .with_field(at.clone())
                .with_hint("give `partner`: its `instant` and `place`")
        })?;
        let davison = given
            .remove("davison")
            .map(|davison| teistro_core::strict::read_value(&davison, &format!("{root}.davison")))
            .transpose()?
            .unwrap_or(false);
        Ok(PartnerSynastry {
            partner: teistro_core::strict::read_value(&partner, &at)?,
            request: teistro_core::strict::read_value(&Value::Object(given), root)?,
            davison,
        })
    }
}

impl TryFrom<Map<String, Value>> for PartnerSynastry {
    type Error = Error;

    fn try_from(given: Map<String, Value>) -> Result<PartnerSynastry, Error> {
        PartnerSynastry::split(given, SYNASTRY)
    }
}

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
        crate::composites::refuse_mixed_zodiacs(a, b, request.zodiac)?;
        synastry(&points(a, request), &points(b, request), request)
    }
}

impl ChartArea<'_> {
    /// Every chart's **synastry with one partner**: the partner's birth is
    /// founded once, with the outer planets when any chart placed them,
    /// and each chart is read against it as [`ChartArea::synastry`] reads
    /// two, the chart first, with the parallels across the two
    /// ([`ChartArea::synastry_parallels`]) when the request asks for them.
    /// One reading a chart, in the order given.
    ///
    /// # Errors
    ///
    /// What [`SynastryRequest::check`] refuses, a partner that cannot be
    /// founded, and what [`ChartArea::synastry`] and
    /// [`ChartArea::synastry_parallels`] refuse, hinted with the chart's
    /// place in the list.
    pub fn synastry_with(
        self,
        charts: &[Document],
        asked: &PartnerSynastry,
    ) -> Result<Vec<PartnerReading>, Error> {
        asked.request.check()?;
        let Partner {
            instant,
            place,
            utc_offset,
        } = asked.partner;
        let request = ChartRequest::at(place, utc_offset);
        let request = if charts
            .iter()
            .any(|chart| !chart.foundation.outer.is_empty())
        {
            request.with_outer_planets()
        } else {
            request
        };
        let partner = self
            .reading(instant, &request)
            .map_err(|why| why.with_field("partner"))?
            .value;
        let reflections = crate::antiscia::reflected(&partner.foundation);
        let theirs = asked
            .request
            .parallels
            .map(|_| self.declinations(&partner))
            .transpose()
            .map_err(|why| why.with_field("partner"))?;
        charts
            .iter()
            .enumerate()
            .map(|(at, chart)| {
                let read = || -> Result<PartnerReading, Error> {
                    let parallels = match (asked.request.parallels, &theirs) {
                        (Some(orb), Some(theirs)) => {
                            Some(self.declinations(chart)?.parallels_across(
                                theirs,
                                asked.request.lagna,
                                orb,
                            )?)
                        }
                        _ => None,
                    };
                    let antiscia = asked
                        .request
                        .antiscia
                        .as_ref()
                        .map(|orbs| {
                            synastry_antiscia(
                                &crate::antiscia::reflected(&chart.foundation),
                                &reflections,
                                orbs,
                            )
                        })
                        .transpose()?;
                    let composite = asked
                        .request
                        .composite
                        .then(|| self.composite(chart, &partner, asked.request.zodiac))
                        .transpose()
                        .map_err(|why| why.under("composite"))?;
                    Ok(PartnerReading {
                        aspects: self.synastry(chart, &partner, &asked.request)?,
                        parallels,
                        antiscia,
                        composite,
                    })
                };
                read().map_err(|why| why.with_hint(format!("chart {at}")))
            })
            .collect()
    }
}

/// One chart read against a partner: the aspects across the two, and the
/// parallels and the antiscia across them when the request asked.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerReading {
    /// The aspects, the chart's point first, closest first.
    pub aspects: Vec<SynastryRow>,
    /// The parallels, the chart's point first, closest first; `None`
    /// unless the request's `parallels` asked.
    pub parallels: Option<Vec<SynastryParallelRow>>,
    /// The antiscia, the chart's planet first, closest first; `None`
    /// unless the request's `antiscia` asked.
    pub antiscia: Option<Vec<AntiscionRow>>,
    /// The composite of the chart and the partner's, the chart first;
    /// `None` unless the request's `composite` asked.
    pub composite: Option<Composite>,
}

/// The planets a Western table reads: the seven, never the nodes, and the
/// outer three when the chart placed them.
pub(crate) fn planets(foundation: &ChartFoundation) -> impl Iterator<Item = &GrahaPosition> {
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
