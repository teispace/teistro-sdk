//! The transit hit list through the façade: every ingress and station of a
//! window, against one chart (`03-design/transit-hit-list.md`).

use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_gochar::GRAHAS;
use teistro_gochar::hits::NatalPoint;

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
