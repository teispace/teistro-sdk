//! The transit hit list through the façade: every ingress and station of a
//! window, against one chart (`03-design/transit-hit-list.md`).

use serde::{Deserialize, Serialize};
use teistro_astro::events::{Direction, Lattice, StationKind};
use teistro_chart::foundation::{TransitEvent, TransitEventKind};
use teistro_core::catalogue::{Catalogued as _, Graha};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_gochar::GRAHAS;
use teistro_gochar::hits::{self, Edge, HitEvent, Motion, NatalPoint};
use teistro_serial::Document;

/// A kind of event a hit list reports, spelled as the event's own `kind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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

    /// The same request, at these aspects' angles: whole degrees from 0 to
    /// 180, each meaning both sides (C145).
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
    /// A graha, a point or an angle named twice is refused rather than
    /// answered twice.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `to`, `grahas`, `kinds`, `points`, `aspects`
    /// or `orb_deg`.
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
        repeated(&self.grahas, "grahas")?;
        nine(self.grahas.iter().copied(), "grahas")?;
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
            repeated(&self.points, "points")?;
            nine(
                self.points.iter().filter_map(|point| match point {
                    NatalPoint::Graha { graha } => Some(*graha),
                    NatalPoint::Lagna => None,
                }),
                "points",
            )?;
            repeated(&self.aspects, "aspects")?;
            if let Some(angle) = self.aspects.iter().find(|a| **a > 180) {
                return Err(Error::invalid_arg(format!(
                    "an aspect is a whole degree from 0 to 180, not {angle}"
                ))
                .with_field("aspects")
                .with_hint("the angle past 180 is the same aspect from the other side"));
            }
            if let Some(orb) = self.orb_deg {
                // Every line of the aspects' lattice opens a window, so two
                // meet once the orb reaches half the step between lines.
                let limit = (hits::aspect_step_deg(&self.aspects) / 2.0).min(15.0);
                if !(orb > 0.0 && orb < limit) {
                    return Err(Error::invalid_arg(format!(
                        "an orb is more than 0 and less than {limit} degrees for these aspects, so no two aspects' windows meet, not {orb}"
                    ))
                    .with_field("orb_deg"));
                }
            }
        }
        Ok(())
    }

    /// The request a binding writes as `hits`, read and checked: the window
    /// as UTC Julian days, and optionally the grahas, the kinds, the natal
    /// points, the aspects' angles and an orb, each spelled as the answer
    /// spells it — a graha by its key, a kind as an event's `kind`, a point
    /// as an aspect's `to` or bare (`"LAGNA"`, `"MOON"`).
    ///
    /// ```
    /// use teistro::{HitKind, HitRequest};
    ///
    /// let asked = HitRequest::from_json(
    ///     r#"{"from": 2460676.5, "to": 2461041.5, "grahas": ["SATURN"],
    ///         "kinds": ["ASPECT"], "points": ["MOON", {"point": "LAGNA"}], "orbDeg": 2}"#,
    /// )?;
    /// assert!(asked.asks(HitKind::Aspect) && !asked.asks(HitKind::Station));
    /// assert_eq!(asked.points().len(), 2);
    /// // A refusal names the field the caller wrote.
    /// let late = HitRequest::from_json(r#"{"from": 2460676.5, "to": 2460000.5}"#).unwrap_err();
    /// assert_eq!(late.field(), Some("hits.to"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, an unknown graha, kind or point, and whatever
    /// [`HitRequest::check`] refuses, each named under `hits`.
    pub fn from_json(text: &str) -> Result<HitRequest, Error> {
        let asked: Asked = teistro_core::strict::read(text, HITS)?;
        let instant = |jd: f64, field: &str| {
            JulianDay::<Utc>::try_new(jd)
                .map_err(|why| Error::from(why).with_field(format!("{HITS}.{field}")))
        };
        let mut request =
            HitRequest::between(instant(asked.from, "from")?, instant(asked.to, "to")?);
        if let Some(grahas) = asked.grahas {
            request = request.with_grahas(grahas);
        }
        if let Some(kinds) = asked.kinds {
            request = request.with_kinds(kinds);
        }
        if let Some(points) = asked.points {
            request = request.with_points(points.into_iter().map(NatalPoint::from));
        }
        if let Some(aspects) = asked.aspects {
            request = request.with_aspects(aspects);
        }
        if let Some(orb_deg) = asked.orb_deg {
            request = request.with_orb(orb_deg);
        }
        request.check().map_err(|why| {
            // The builder's field is `orb_deg`; the record's is `orbDeg`.
            let why = match why.field() {
                Some("orb_deg") => why.with_field("orbDeg"),
                _ => why,
            };
            why.under(HITS)
        })?;
        Ok(request)
    }
}

/// The record every binding writes the request as.
const HITS: &str = "hits";

/// [`HitRequest`] as the bindings write it, camel-cased as every request
/// record is; every field but the window is optional, and an absent one is
/// the default.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Asked {
    from: f64,
    to: f64,
    #[serde(default)]
    grahas: Option<Vec<Graha>>,
    #[serde(default)]
    kinds: Option<Vec<HitKind>>,
    #[serde(default)]
    points: Option<Vec<PointAsked>>,
    #[serde(default)]
    aspects: Option<Vec<u16>>,
    #[serde(default)]
    orb_deg: Option<f64>,
}

/// A natal point as a request may name it: as an answer's `to` spells it
/// (`{"point": "GRAHA", "graha": "graha.MOON"}`, `{"point": "LAGNA"}`), or
/// bare, a graha's key or `"LAGNA"`.
#[derive(Serialize)]
#[serde(transparent)]
struct PointAsked(NatalPoint);

impl<'de> Deserialize<'de> for PointAsked {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        let value = serde_json::Value::deserialize(deserializer)?;
        let graha = |key: &str| {
            Graha::from_either_key(key)
                .map(|graha| PointAsked(NatalPoint::Graha { graha }))
                .map_err(D::Error::custom)
        };
        let shapes = "a point is \"LAGNA\", a graha's key, or an aspect's `to`";
        match &value {
            serde_json::Value::String(key) if key == "LAGNA" => Ok(PointAsked(NatalPoint::Lagna)),
            serde_json::Value::String(key) => {
                graha(key).map_err(|why| D::Error::custom(format!("{why}; {shapes}")))
            }
            serde_json::Value::Object(fields) => {
                let field = |name: &str| fields.get(name).and_then(serde_json::Value::as_str);
                match (field("point"), field("graha"), fields.len()) {
                    (Some("LAGNA"), None, 1) => Ok(PointAsked(NatalPoint::Lagna)),
                    (Some("GRAHA"), Some(key), 2) => graha(key),
                    _ => Err(D::Error::custom(format!(
                        "{value} is not a natal point; {shapes}"
                    ))),
                }
            }
            _ => Err(D::Error::custom(format!(
                "{value} is not a natal point; {shapes}"
            ))),
        }
    }
}

impl From<PointAsked> for NatalPoint {
    fn from(asked: PointAsked) -> NatalPoint {
        asked.0
    }
}

/// Refuses a graha outside the nine a chart places, by field: the
/// catalogue names more (Uranus, Pluto), and neither a transit's search
/// nor a natal chart has them.
fn nine(grahas: impl IntoIterator<Item = Graha>, field: &str) -> Result<(), Error> {
    match grahas.into_iter().find(|graha| !GRAHAS.contains(graha)) {
        Some(graha) => Err(Error::invalid_arg(format!(
            "{} is not one of the nine grahas a chart places",
            graha.key()
        ))
        .with_field(field)
        .with_hint("the nine are the Sun to Saturn, Rahu and Ketu")),
        None => Ok(()),
    }
}

/// Refuses a list that names one member twice, by field.
fn repeated<T: PartialEq + std::fmt::Debug>(members: &[T], field: &str) -> Result<(), Error> {
    for (at, member) in members.iter().enumerate() {
        if members
            .get(..at)
            .is_some_and(|before| before.contains(member))
        {
            return Err(Error::invalid_arg(format!(
                "{member:?} is named twice, which would report its events twice"
            ))
            .with_field(field));
        }
    }
    Ok(())
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

/// The motion a crossing's direction is: a falling longitude is retrograde.
pub(crate) const fn motion(direction: Direction) -> Motion {
    match direction {
        Direction::Falling => Motion::Retrograde,
        Direction::Rising => Motion::Direct,
    }
}

/// The hit a searched event is, and the one chart it belongs to when it is
/// an aspect; `None` for a line no aspect asked for lies on.
pub(crate) fn hit_of(
    event: &TransitEvent,
    meanings: &[(Meaning, Lattice)],
    request: &HitRequest,
) -> Option<(HitEvent, Option<usize>)> {
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    const WINDOW: &str = r#""from": 2460676.5, "to": 2461041.5"#;

    fn refused(rest: &str) -> Error {
        HitRequest::from_json(&format!("{{{WINDOW}{rest}}}")).unwrap_err()
    }

    #[test]
    fn a_point_is_named_as_the_answer_names_it_or_bare() {
        let asked = HitRequest::from_json(&format!(
            r#"{{{WINDOW}, "points": ["LAGNA", "SUN", {{"point": "GRAHA", "graha": "MOON"}}, {{"point": "LAGNA"}}]}}"#
        ));
        // The lagna named twice, once bare and once tagged, is one point.
        assert_eq!(asked.unwrap_err().field(), Some("hits.points"));
        let asked = HitRequest::from_json(&format!(
            r#"{{{WINDOW}, "points": ["LAGNA", "SUN", {{"point": "GRAHA", "graha": "MOON"}}]}}"#
        ))
        .unwrap();
        assert_eq!(
            asked.points(),
            [
                NatalPoint::Lagna,
                NatalPoint::Graha { graha: Graha::Sun },
                NatalPoint::Graha { graha: Graha::Moon }
            ]
        );
    }

    #[test]
    fn a_graha_is_named_bare_or_full() {
        let asked = HitRequest::from_json(&format!(
            r#"{{{WINDOW}, "grahas": ["SUN", "graha.SATURN"], "points": ["graha.MOON", {{"point": "GRAHA", "graha": "graha.MARS"}}]}}"#
        ))
        .unwrap();
        assert_eq!(asked.grahas(), [Graha::Sun, Graha::Saturn]);
        assert_eq!(
            asked.points(),
            [
                NatalPoint::Graha { graha: Graha::Moon },
                NatalPoint::Graha { graha: Graha::Mars }
            ]
        );
    }

    #[test]
    fn a_refusal_names_the_field_under_hits() {
        for (rest, field) in [
            (r#", "grahas": ["SUN", "SUN"]"#, "hits.grahas"),
            (r#", "grahas": []"#, "hits.grahas"),
            (r#", "aspects": [0, 0]"#, "hits.aspects"),
            (r#", "aspects": [181]"#, "hits.aspects"),
            (r#", "aspects": [0, 10], "orbDeg": 6"#, "hits.orbDeg"),
            (r#", "orbDeg": 20"#, "hits.orbDeg"),
            (r#", "kinds": []"#, "hits.kinds"),
        ] {
            assert_eq!(refused(rest).field(), Some(field), "{rest}");
        }
        assert_eq!(
            refused(r#", "grahas": ["PLUTO"]"#).field(),
            Some("hits.grahas")
        );
        assert_eq!(
            refused(r#", "points": ["URANUS"]"#).field(),
            Some("hits.points")
        );
        for rest in [
            r#", "kinds": ["ECLIPSE"]"#,
            r#", "points": ["ASCENDANT"]"#,
            r#", "points": [{"point": "LAGNA", "graha": "SUN"}]"#,
            r#", "points": [7]"#,
            r#", "extra": 1"#,
        ] {
            let why = refused(rest);
            assert_eq!(
                why.status,
                teistro_core::error::Status::InvalidArg,
                "{rest}"
            );
            // Each says what it wanted, never serde's "did not match".
            assert!(!why.to_string().contains("did not match"), "{why}");
        }
    }
}
