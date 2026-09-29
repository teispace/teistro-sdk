//! Sade Sati through the façade: Saturn's spells from the natal Moon over a
//! window, each whole (`03-design/sade-sati.md`).

use serde::{Deserialize, Serialize};
use teistro_astro::events::Lattice;
use teistro_chart::foundation::ChartFoundation;
use teistro_chart::foundation::{TransitEvent, TransitEventKind};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_gochar::sade_sati::{self, Crossing, DEFAULT_SPELLS, Reckoning};
use teistro_gochar::{GocharFrom, Reference};
use teistro_serial::Document;

use crate::hit_request::motion;

/// The window to look in, what to count Saturn's houses from, how to
/// reckon them and which smaller spells beside the Sade Sati to report.
///
/// ```
/// use teistro::quantity::{JulianDay, Utc};
/// use teistro::sade_sati::Reckoning;
/// use teistro::{GocharFrom, SadeSatiRequest};
///
/// // Is it now? The periods Saturn is in at one instant, whole.
/// let now = SadeSatiRequest::at(JulianDay::<Utc>::literal(2_461_000.5));
/// assert_eq!((now.reckoning(), now.from()), (Reckoning::Sign, GocharFrom::Moon));
/// assert_eq!(now.spells(), [4, 8]);
/// // Every one of a life, by the Moon's degree, with the 7th as well.
/// let life = SadeSatiRequest::between(JulianDay::literal(2_447_995.5), JulianDay::literal(2_477_216.5))
///     .reckoned(Reckoning::Degree)
///     .with_spells([4, 7, 8]);
/// assert!(life.check().is_ok());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SadeSatiRequest {
    from: JulianDay<Utc>,
    to: JulianDay<Utc>,
    counted_from: GocharFrom,
    reckoning: Reckoning,
    spells: Vec<u8>,
}

impl SadeSatiRequest {
    /// The periods Saturn is in at one instant: whether a Sade Sati is on,
    /// which phase, and when it began and ends.
    #[must_use]
    pub fn at(instant: JulianDay<Utc>) -> SadeSatiRequest {
        SadeSatiRequest::between(instant, instant)
    }

    /// Every period reaching into a window, each whole: from the natal
    /// Moon's sign, reckoned in whole signs, with the 4th and 8th as the
    /// smaller spells (C147, C149).
    #[must_use]
    pub fn between(from: JulianDay<Utc>, to: JulianDay<Utc>) -> SadeSatiRequest {
        SadeSatiRequest {
            from,
            to,
            counted_from: GocharFrom::Moon,
            reckoning: Reckoning::Sign,
            spells: DEFAULT_SPELLS.to_vec(),
        }
    }

    /// Counted from another reference than the natal Moon (C139).
    #[must_use]
    pub const fn counted_from(mut self, from: GocharFrom) -> SadeSatiRequest {
        self.counted_from = from;
        self
    }

    /// Reckoned otherwise than in whole signs (C147).
    #[must_use]
    pub const fn reckoned(mut self, reckoning: Reckoning) -> SadeSatiRequest {
        self.reckoning = reckoning;
        self
    }

    /// These houses as the smaller spells, 3 to 11 from the reference; none
    /// for the Sade Sati alone (C149).
    #[must_use]
    pub fn with_spells(mut self, houses: impl IntoIterator<Item = u8>) -> SadeSatiRequest {
        self.spells = houses.into_iter().collect();
        self
    }

    /// The window's start.
    #[must_use]
    pub const fn start(&self) -> JulianDay<Utc> {
        self.from
    }

    /// The window's end.
    #[must_use]
    pub const fn end(&self) -> JulianDay<Utc> {
        self.to
    }

    /// What the houses are counted from.
    #[must_use]
    pub const fn from(&self) -> GocharFrom {
        self.counted_from
    }

    /// What they are reckoned in.
    #[must_use]
    pub const fn reckoning(&self) -> Reckoning {
        self.reckoning
    }

    /// The smaller spells' houses.
    #[must_use]
    pub fn spells(&self) -> &[u8] {
        &self.spells
    }

    /// Refuses a window that runs backward, and a smaller spell outside
    /// houses 3 to 11 or named twice; a refusal names the field.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `to` or `spells`.
    pub fn check(&self) -> Result<(), Error> {
        if self.to.get() < self.from.get() {
            return Err(Error::invalid_arg(format!(
                "a window must not run backward, and {} is before {}",
                self.to.get(),
                self.from.get()
            ))
            .with_field("to"));
        }
        sade_sati::check_houses(&self.spells).map_err(|why| why.with_field("spells"))
    }

    /// The request a binding writes as `sadeSati`, read and checked: the
    /// window as UTC Julian days (`to` defaulting to `from`, one instant),
    /// and optionally the reference, the reckoning and the smaller spells.
    ///
    /// ```
    /// use teistro::SadeSatiRequest;
    /// use teistro::sade_sati::Reckoning;
    ///
    /// let asked = SadeSatiRequest::from_json(r#"{"from": 2461000.5, "reckoning": "DEGREE", "spells": []}"#)?;
    /// assert_eq!((asked.reckoning(), asked.spells().len()), (Reckoning::Degree, 0));
    /// // A refusal names the field the caller wrote.
    /// let first = SadeSatiRequest::from_json(r#"{"from": 2461000.5, "spells": [1]}"#).unwrap_err();
    /// assert_eq!(first.field(), Some("sadeSati.spells"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, an unknown reference or reckoning, and whatever
    /// [`SadeSatiRequest::check`] refuses, each named under `sadeSati`.
    pub fn from_json(text: &str) -> Result<SadeSatiRequest, Error> {
        let asked: Asked = teistro_core::strict::read(text, SADE_SATI)?;
        let instant = |jd: f64, field: &str| {
            JulianDay::<Utc>::try_new(jd)
                .map_err(|why| Error::from(why).with_field(format!("{SADE_SATI}.{field}")))
        };
        let from = instant(asked.from, "from")?;
        let to = asked.to.map_or(Ok(from), |to| instant(to, "to"))?;
        let mut request = SadeSatiRequest::between(from, to)
            .counted_from(asked.counted_from)
            .reckoned(asked.reckoning);
        if let Some(spells) = asked.spells {
            request = request.with_spells(spells);
        }
        request.check().map_err(|why| why.under(SADE_SATI))?;
        Ok(request)
    }
}

/// The record every binding writes the request as.
const SADE_SATI: &str = "sadeSati";

/// [`SadeSatiRequest`] as the bindings write it, camel-cased as every
/// request record is; every field but `from` is optional.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Asked {
    from: f64,
    #[serde(default)]
    to: Option<f64>,
    #[serde(default)]
    counted_from: GocharFrom,
    #[serde(default)]
    reckoning: Reckoning,
    #[serde(default)]
    spells: Option<Vec<u8>>,
}

/// One chart of a search: where it stands in the batch, what its houses
/// are counted from, the line its first house begins at, and which of the
/// scan's lattices holds its lines.
pub(crate) struct Charted {
    pub(crate) index: usize,
    pub(crate) reference: Reference,
    pub(crate) origin_deg: f64,
    pub(crate) lattice: usize,
}

/// The lattices one scan of Saturn tests, and each chart's part in it:
/// under whole signs every chart shares one lattice, and under the degree
/// reckoning charts whose lines coincide share theirs.
pub(crate) fn lattices_of(
    request: &SadeSatiRequest,
    natals: &[&Document],
    members: &[usize],
) -> Result<(Vec<Lattice>, Vec<Charted>), Error> {
    let mut lattices: Vec<Lattice> = Vec::new();
    let mut charts = Vec::with_capacity(members.len());
    for index in members {
        let foundation = &natals
            .get(*index)
            .ok_or_else(|| Error::internal("a group names a chart it was not given"))?
            .foundation;
        let origin_deg = request
            .reckoning()
            .origin_deg(reference_deg(foundation, request.from())?);
        let lattice = Lattice {
            origin_deg: origin_deg.rem_euclid(30.0),
            step_deg: 30.0,
        };
        let lattice = lattices
            .iter()
            .position(|known| known.origin_deg.to_bits() == lattice.origin_deg.to_bits())
            .unwrap_or_else(|| {
                lattices.push(lattice);
                lattices.len() - 1
            });
        charts.push(Charted {
            index: *index,
            reference: crate::gochar_request::reference(foundation, request.from())?,
            origin_deg,
            lattice,
        });
    }
    Ok((lattices, charts))
}

/// Saturn's crossings of each lattice, in time order.
pub(crate) fn crossings_of(events: &[TransitEvent], lattices: usize) -> Vec<Vec<Crossing>> {
    let mut crossings = vec![Vec::new(); lattices];
    for event in events {
        if let TransitEventKind::Crossing {
            lattice,
            boundary_deg,
            direction,
        } = event.kind
            && let Some(list) = crossings.get_mut(lattice)
        {
            list.push(Crossing::new(
                event.instant,
                boundary_deg,
                motion(direction),
            ));
        }
    }
    crossings
}

/// The natal reference's sidereal longitude, degrees: the Moon's or the
/// lagna's, in the chart's own zodiac.
pub(crate) fn reference_deg(natal: &ChartFoundation, from: GocharFrom) -> Result<f64, Error> {
    match from {
        GocharFrom::Lagna => Ok(natal.lagna_deg),
        _ => natal
            .graha(Graha::Moon)
            .map(|moon| moon.longitude_deg)
            .ok_or_else(|| Error::internal("a founded chart places the Moon")),
    }
}

/// How far past the window the search reaches at first, and how much
/// further each widening takes it, days: about three of Saturn's houses,
/// so a period in progress at the window is usually settled by the first
/// search. It decides the cost and never the answer, which
/// [`sade_sati::periods`] refuses to give until no wider search could
/// change it.
const REACH_DAYS: f64 = 365.25 * 7.5;

/// The most widenings a search takes: Saturn crosses a house in under
/// three years, so a search this wide that still cannot settle a period
/// has met a defect, not the sky.
const MOST_WIDENINGS: u8 = 8;

/// The search's window around the asked one, widened on the side a period
/// still runs past, never past what the ephemeris covers.
pub(crate) struct Widening {
    asked: (f64, f64),
    reach: (f64, f64),
    coverage: (f64, f64),
    widenings: u8,
}

impl Widening {
    /// The first search around `asked`, inside `coverage` (a provider's
    /// Julian days, a day inside each end).
    pub(crate) fn new(
        (from, to): (JulianDay<Utc>, JulianDay<Utc>),
        coverage: (f64, f64),
    ) -> Widening {
        Widening {
            asked: (from.get(), to.get()),
            reach: (REACH_DAYS, REACH_DAYS),
            coverage: (coverage.0 + 1.0, coverage.1 - 1.0),
            widenings: 0,
        }
    }

    /// The window to search now.
    pub(crate) fn window(&self) -> (JulianDay<Utc>, JulianDay<Utc>) {
        let (start, end) = self.bounds();
        (JulianDay::literal(start), JulianDay::literal(end))
    }

    /// Whether the search begins and ends at the ephemeris's own edge.
    pub(crate) fn at_coverage(&self) -> (bool, bool) {
        let (start, end) = self.bounds();
        (start <= self.coverage.0, end >= self.coverage.1)
    }

    /// Reaches further on the sides asked.
    ///
    /// # Errors
    ///
    /// `INTERNAL` past [`MOST_WIDENINGS`].
    pub(crate) fn widen(&mut self, before: bool, after: bool) -> Result<(), Error> {
        self.widenings += 1;
        if self.widenings > MOST_WIDENINGS {
            return Err(Error::internal(format!(
                "Saturn's spells were not settled by a search {} years either side of the window",
                (self.reach.0.max(self.reach.1) / 365.25).round()
            )));
        }
        if before {
            self.reach.0 += REACH_DAYS;
        }
        if after {
            self.reach.1 += REACH_DAYS;
        }
        Ok(())
    }

    /// On whole Julian days, so the scan samples the same instants however
    /// the window was drawn and a bound refines to the same bits: the
    /// window chooses which periods are reported, never where they end.
    fn bounds(&self) -> (f64, f64) {
        (
            (self.asked.0 - self.reach.0).floor().max(self.coverage.0),
            (self.asked.1 + self.reach.1).ceil().min(self.coverage.1),
        )
    }
}
