//! The transit hit list through the façade: every ingress and station of a
//! window, against one chart (`03-design/transit-hit-list.md`).

use teistro_astro::events::{Direction, Lattice, StationKind};
use teistro_chart::foundation::{TransitEvent, TransitEventKind};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_gochar::GRAHAS;
use teistro_gochar::hits::{self, Edge, HitEvent, Motion, NatalPoint};
use teistro_serial::Document;

/// A kind of event a hit list reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum HitKind {
    /// A graha entering a sign.
    SignIngress,
    /// A graha entering a nakshatra.
    NakshatraIngress,
    /// A graha standing still, turning retrograde or direct.
    Station,
    /// A graha aspecting a natal point, at the angles asked for (C145),
    /// with the orb's edges when an orb was asked for (C146).
    Aspect,
}

impl HitKind {
    /// Every kind, the default a request asks for.
    pub const ALL: [HitKind; 4] = [
        HitKind::SignIngress,
        HitKind::NakshatraIngress,
        HitKind::Station,
        HitKind::Aspect,
    ];
}

/// The window to search, whose events and which grahas'.
///
/// ```
/// use teistro::catalogue::Graha;
/// use teistro::quantity::{JulianDay, Utc};
/// use teistro::{HitKind, HitRequest};
///
/// // Saturn's and Jupiter's sign changes and stations over ten years.
/// let asked = HitRequest::between(JulianDay::<Utc>::literal(2_460_676.5), JulianDay::literal(2_464_329.0))
///     .with_grahas([Graha::Saturn, Graha::Jupiter])
///     .with_kinds([HitKind::SignIngress, HitKind::Station]);
/// assert_eq!(asked.grahas().len(), 2);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct HitRequest {
    from: JulianDay<Utc>,
    to: JulianDay<Utc>,
    grahas: Vec<Graha>,
    kinds: Vec<HitKind>,
    points: Vec<NatalPoint>,
    aspects: Vec<u16>,
    orb_deg: Option<f64>,
}

impl HitRequest {
    /// The aspects a hit list reports unless asked otherwise: the
    /// conjunction and the opposition, which every tradition counts (crux
    /// C145).
    pub const DEFAULT_ASPECTS: [u16; 2] = [0, 180];

    /// Every event of all nine grahas between two instants: their
    /// ingresses, their stations and their conjunctions and oppositions to
    /// the natal grahas and lagna, exact.
    #[must_use]
    pub fn between(from: JulianDay<Utc>, to: JulianDay<Utc>) -> HitRequest {
        HitRequest {
            from,
            to,
            grahas: GRAHAS.to_vec(),
            kinds: HitKind::ALL.to_vec(),
            points: GRAHAS
                .iter()
                .map(|graha| NatalPoint::Graha { graha: *graha })
                .chain([NatalPoint::Lagna])
                .collect(),
            aspects: HitRequest::DEFAULT_ASPECTS.to_vec(),
            orb_deg: None,
        }
    }

    /// The same request, for these grahas only.
    #[must_use]
    pub fn with_grahas(mut self, grahas: impl IntoIterator<Item = Graha>) -> HitRequest {
        self.grahas = grahas.into_iter().collect();
        self
    }

    /// The same request, for these kinds of event only.
    #[must_use]
    pub fn with_kinds(mut self, kinds: impl IntoIterator<Item = HitKind>) -> HitRequest {
        self.kinds = kinds.into_iter().collect();
        self
    }

    /// The same request, aspecting these natal points only.
    #[must_use]
    pub fn with_points(mut self, points: impl IntoIterator<Item = NatalPoint>) -> HitRequest {
        self.points = points.into_iter().collect();
        self
    }

    /// The same request, at these aspects' angles: multiples of 30 from 0
    /// to 180, each meaning both sides (C145).
    #[must_use]
    pub fn with_aspects(mut self, angles: impl IntoIterator<Item = u16>) -> HitRequest {
        self.aspects = angles.into_iter().collect();
        self
    }

    /// The same request, with each aspect's window this many degrees
    /// either side of exact, reported as it opens and closes (C146).
    #[must_use]
    pub const fn with_orb(mut self, orb_deg: f64) -> HitRequest {
        self.orb_deg = Some(orb_deg);
        self
    }

    /// The natal points aspected.
    #[must_use]
    pub fn points(&self) -> &[NatalPoint] {
        &self.points
    }

    /// The aspects' angles.
    #[must_use]
    pub fn aspects(&self) -> &[u16] {
        &self.aspects
    }

    /// The orb, when one was asked for.
    #[must_use]
    pub const fn orb_deg(&self) -> Option<f64> {
        self.orb_deg
    }

    /// The window's start.
    #[must_use]
    pub const fn from(&self) -> JulianDay<Utc> {
        self.from
    }

    /// The window's end.
    #[must_use]
    pub const fn to(&self) -> JulianDay<Utc> {
        self.to
    }

    /// The grahas asked about.
    #[must_use]
    pub fn grahas(&self) -> &[Graha] {
        &self.grahas
    }

    /// Whether a kind of event was asked for.
    #[must_use]
    pub fn asks(&self, kind: HitKind) -> bool {
        self.kinds.contains(&kind)
    }

    /// The request checked before any search: a window that runs forward,
    /// and at least one graha and one kind; a refusal names the field.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `to`, `grahas` or `kinds`.
    pub fn check(&self) -> Result<(), Error> {
        if self.to.get() <= self.from.get() {
            return Err(Error::invalid_arg(format!(
                "a hit list's window must run forward, and {} is not after {}",
                self.to.get(),
                self.from.get()
            ))
            .with_field("to"));
        }
        if self.grahas.is_empty() {
            return Err(
                Error::invalid_arg("no graha to search the transits of").with_field("grahas")
            );
        }
        if self.kinds.is_empty() {
            return Err(Error::invalid_arg("no kind of event to report").with_field("kinds"));
        }
        if self.asks(HitKind::Aspect) {
            if self.points.is_empty() {
                return Err(Error::invalid_arg("no natal point to aspect").with_field("points"));
            }
            if self.aspects.is_empty() {
                return Err(Error::invalid_arg("no aspect to report").with_field("aspects"));
            }
            if let Some(angle) = self.aspects.iter().find(|a| **a > 180 || **a % 30 != 0) {
                return Err(Error::invalid_arg(format!(
                    "an aspect is a multiple of 30 degrees from 0 to 180, not {angle}"
                ))
                .with_field("aspects")
                .with_hint("the angle past 180 is the same aspect from the other side"));
            }
            if let Some(orb) = self.orb_deg
                && !(orb > 0.0 && orb < 15.0)
            {
                return Err(Error::invalid_arg(format!(
                    "an orb is more than 0 and less than 15 degrees, so no two aspects' windows meet, not {orb}"
                ))
                .with_field("orb_deg"));
            }
        }
        Ok(())
    }
}

/// What a lattice's crossings mean, and for which chart: an ingress is
/// every chart's, an aspect one chart's (by its index in the batch).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Meaning {
    Sign,
    Nakshatra,
    Aspect(usize, NatalPoint, f64, Edge),
}

/// The charts searched together, each group with the place it is searched
/// from: all of them at once, unless the sky depends on where it is seen
/// from, when each place is its own group.
pub(crate) fn groups(natals: &[&Document], topocentric: bool) -> Vec<(Place, Vec<usize>)> {
    let mut groups: Vec<(Place, Vec<usize>)> = Vec::new();
    for (index, natal) in natals.iter().enumerate() {
        let place = natal.foundation.place;
        match groups
            .iter_mut()
            .find(|(seen, _)| !topocentric || *seen == place)
        {
            Some((_, members)) => members.push(index),
            None => groups.push((place, vec![index])),
        }
    }
    groups
}

/// The lattices one group's scan tests, and what each one's crossings
/// mean: the signs and the nakshatras once for the group, and each chart's
/// aspect lines, exact and at the orb's two edges.
///
/// # Errors
///
/// A natal chart without a point asked for, named `points`.
pub(crate) fn lattices_of(
    request: &HitRequest,
    natals: &[&Document],
    members: &[usize],
) -> Result<Vec<(Meaning, Lattice)>, Error> {
    let mut meanings = Vec::new();
    if request.asks(HitKind::SignIngress) {
        meanings.push((Meaning::Sign, Lattice::SIGNS));
    }
    if request.asks(HitKind::NakshatraIngress) {
        meanings.push((Meaning::Nakshatra, Lattice::NAKSHATRAS));
    }
    if !request.asks(HitKind::Aspect) {
        return Ok(meanings);
    }
    let orb = request.orb_deg.unwrap_or(0.0);
    let edges: &[(Edge, f64)] = if request.orb_deg.is_some() {
        &[(Edge::Exact, 0.0), (Edge::Before, -orb), (Edge::Past, orb)]
    } else {
        &[(Edge::Exact, 0.0)]
    };
    let step_deg = hits::aspect_step_deg(&request.aspects);
    for chart in members {
        let foundation = &natals
            .get(*chart)
            .ok_or_else(|| Error::internal("a group names a chart it was not given"))?
            .foundation;
        for point in &request.points {
            let natal_deg = match point {
                NatalPoint::Graha { graha } => foundation.graha(*graha).map(|at| at.longitude_deg),
                NatalPoint::Lagna => Some(foundation.lagna_deg),
            }
            .ok_or_else(|| {
                Error::invalid_arg(format!("natal chart {chart} has no {point:?} to aspect"))
                    .with_field("points")
            })?;
            for (edge, shift) in edges {
                meanings.push((
                    Meaning::Aspect(*chart, *point, natal_deg, *edge),
                    Lattice {
                        origin_deg: natal_deg + shift,
                        step_deg,
                    },
                ));
            }
        }
    }
    Ok(meanings)
}

/// The hit a searched event is, and the one chart it belongs to when it is
/// an aspect; `None` for a line no aspect asked for lies on.
pub(crate) fn hit_of(
    event: &TransitEvent,
    meanings: &[(Meaning, Lattice)],
    request: &HitRequest,
) -> Option<(HitEvent, Option<usize>)> {
    let motion = |direction: Direction| match direction {
        Direction::Falling => Motion::Retrograde,
        Direction::Rising => Motion::Direct,
    };
    match event.kind {
        TransitEventKind::Crossing {
            lattice,
            boundary_deg,
            direction,
        } => match meanings.get(lattice)?.0 {
            Meaning::Sign => Some((hits::sign_ingress(boundary_deg, motion(direction)), None)),
            Meaning::Nakshatra => Some((
                hits::nakshatra_ingress(boundary_deg, motion(direction)),
                None,
            )),
            Meaning::Aspect(chart, point, natal_deg, edge) => hits::aspect_hit(
                (point, natal_deg),
                (boundary_deg, edge, request.orb_deg.unwrap_or(0.0)),
                motion(direction),
                &request.aspects,
            )
            .map(|aspect| (aspect, Some(chart))),
        },
        TransitEventKind::Station { kind, .. } => Some((
            HitEvent::Station {
                turns: match kind {
                    StationKind::Retrograde => Motion::Retrograde,
                    StationKind::Direct => Motion::Direct,
                },
            },
            None,
        )),
    }
}
