//! The transit hit list through the façade: every ingress and station of a
//! window, against one chart (`03-design/transit-hit-list.md`).

use serde::{Deserialize, Serialize};
use teistro_astro::events::{Direction, Lattice, StationKind};
use teistro_chart::OUTER;
use teistro_chart::foundation::{ChartFoundation, TransitEvent, TransitEventKind};
use teistro_core::catalogue::{Catalogued as _, Graha, LunarEclipseKind, SolarEclipseKind, Varga};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_gochar::GRAHAS;
use teistro_gochar::hits::{self, Edge, Hit, HitEvent, Motion, NatalPoint};
use teistro_serial::Document;
use teistro_vargas::Scheme;

/// A kind of event a hit list reports, spelled as the event's own `kind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// A graha entering a new sign of a divisional chart the request
    /// names, the navamsha unless it names others (C418). Asked for by
    /// name only: it is not among [`HitKind::DEFAULT`].
    VargaIngress,
    /// The Moon reaching a principal phase, its elongation from the Sun at
    /// a multiple of 90° (C419): the Moon's, reported whichever grahas the
    /// request names. Asked for by name only.
    LunarPhase,
    /// The Moon's elongation from the Sun returning to the natal chart's,
    /// once a synodic month (C420): each chart's own, reported whichever
    /// grahas the request names. Asked for by name only.
    PhaseReturn,
    /// A solar eclipse whose Sun falls within the eclipse orb of a natal
    /// point the request names (C421, C422). Asked for by name only.
    SolarEclipse,
    /// A lunar eclipse whose Moon falls within the eclipse orb of a natal
    /// point the request names (C421, C422). Asked for by name only.
    LunarEclipse,
}

impl HitKind {
    /// Every kind a hit list can report.
    pub const ALL: [HitKind; 9] = [
        HitKind::SignIngress,
        HitKind::NakshatraIngress,
        HitKind::Station,
        HitKind::Aspect,
        HitKind::VargaIngress,
        HitKind::LunarPhase,
        HitKind::PhaseReturn,
        HitKind::SolarEclipse,
        HitKind::LunarEclipse,
    ];

    /// The kinds a request asks for unless it names its own: the four the
    /// hit list began with. A kind added since is asked for by name, so
    /// no list a caller already reads grows under them.
    pub const DEFAULT: [HitKind; 4] = [
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
///
/// // Jupiter's navamsha and dashamsha ingresses over the same years.
/// use teistro::catalogue::Varga;
/// let vargas = asked
///     .with_grahas([Graha::Jupiter])
///     .with_kinds([HitKind::VargaIngress])
///     .with_vargas([Varga::D9, Varga::D10]);
/// assert_eq!(vargas.vargas(), &[Varga::D9, Varga::D10]);
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
    vargas: Vec<Varga>,
    eclipse_orb_deg: f64,
}

impl HitRequest {
    /// The aspects a hit list reports unless asked otherwise: the
    /// conjunction and the opposition, which every tradition counts (crux
    /// C145).
    pub const DEFAULT_ASPECTS: [u16; 2] = [0, 180];

    /// The divisional charts a varga ingress is reported in unless asked
    /// otherwise: the navamsha, the chart a transit is most often read in
    /// after the rashi (C418).
    pub const DEFAULT_VARGAS: [Varga; 1] = [Varga::D9];

    /// How far an eclipse's luminary may stand from a natal point and
    /// still fall on it, degrees, unless asked otherwise (C422): the
    /// middle of the one to five degrees practice gives an eclipse.
    pub const DEFAULT_ECLIPSE_ORB_DEG: f64 = 3.0;

    /// The widest eclipse orb a request may name, degrees.
    pub const MOST_ECLIPSE_ORB_DEG: f64 = 15.0;

    /// Every event of all nine grahas between two instants: their
    /// ingresses, their stations and their conjunctions and oppositions to
    /// the natal grahas and lagna, exact — the kinds of
    /// [`HitKind::DEFAULT`].
    #[must_use]
    pub fn between(from: JulianDay<Utc>, to: JulianDay<Utc>) -> HitRequest {
        HitRequest {
            from,
            to,
            grahas: GRAHAS.to_vec(),
            kinds: HitKind::DEFAULT.to_vec(),
            points: GRAHAS
                .iter()
                .map(|graha| NatalPoint::Graha { graha: *graha })
                .chain([NatalPoint::Lagna])
                .collect(),
            aspects: HitRequest::DEFAULT_ASPECTS.to_vec(),
            orb_deg: None,
            vargas: HitRequest::DEFAULT_VARGAS.to_vec(),
            eclipse_orb_deg: HitRequest::DEFAULT_ECLIPSE_ORB_DEG,
        }
    }

    /// The hit list that is each graha's **returns**: its conjunction, at
    /// 0°, with its own natal place and nothing else
    /// (`03-design/western-returns.md`). The Moon's is the lunar return.
    ///
    /// Asked for several grahas, the list also holds each one's crossing
    /// of another's natal place, its `to` naming the place;
    /// [`ChartArea::returns`](crate::area::ChartArea::returns) keeps only
    /// the returns.
    ///
    /// ```
    /// use teistro::catalogue::Graha;
    /// use teistro::quantity::{JulianDay, Utc};
    /// use teistro::{HitKind, HitRequest};
    ///
    /// let lunar = HitRequest::returns(JulianDay::<Utc>::literal(2_460_676.5), JulianDay::literal(2_461_041.5), [Graha::Moon]);
    /// assert!(lunar.asks(HitKind::Aspect) && !lunar.asks(HitKind::Station));
    /// assert_eq!(lunar.aspects(), &[0]);
    /// ```
    #[must_use]
    pub fn returns(
        from: JulianDay<Utc>,
        to: JulianDay<Utc>,
        grahas: impl IntoIterator<Item = Graha>,
    ) -> HitRequest {
        let grahas: Vec<Graha> = grahas.into_iter().collect();
        HitRequest {
            from,
            to,
            points: grahas
                .iter()
                .map(|graha| NatalPoint::Graha { graha: *graha })
                .collect(),
            grahas,
            kinds: vec![HitKind::Aspect],
            aspects: vec![0],
            orb_deg: None,
            vargas: HitRequest::DEFAULT_VARGAS.to_vec(),
            eclipse_orb_deg: HitRequest::DEFAULT_ECLIPSE_ORB_DEG,
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

    /// The same request, reporting varga ingresses in these divisional
    /// charts (C418). The rashi, D1, is not one: its ingresses are
    /// [`HitKind::SignIngress`].
    #[must_use]
    pub fn with_vargas(mut self, vargas: impl IntoIterator<Item = Varga>) -> HitRequest {
        self.vargas = vargas.into_iter().collect();
        self
    }

    /// The same request, an eclipse falling on a natal point when its
    /// luminary stands within this many degrees of it (C422).
    ///
    /// ```
    /// use teistro::quantity::{JulianDay, Utc};
    /// use teistro::{HitKind, HitRequest};
    ///
    /// let year = HitRequest::between(JulianDay::<Utc>::literal(2_460_676.5), JulianDay::literal(2_461_041.5))
    ///     .with_kinds([HitKind::SolarEclipse, HitKind::LunarEclipse]);
    /// assert_eq!(year.eclipse_orb_deg(), HitRequest::DEFAULT_ECLIPSE_ORB_DEG);
    /// assert_eq!(year.with_eclipse_orb(1.0).eclipse_orb_deg(), 1.0);
    /// ```
    #[must_use]
    pub const fn with_eclipse_orb(mut self, orb_deg: f64) -> HitRequest {
        self.eclipse_orb_deg = orb_deg;
        self
    }

    /// The eclipse orb, degrees.
    #[must_use]
    pub const fn eclipse_orb_deg(&self) -> f64 {
        self.eclipse_orb_deg
    }

    /// Whether either kind of eclipse was asked for.
    #[must_use]
    pub fn asks_eclipses(&self) -> bool {
        self.asks(HitKind::SolarEclipse) || self.asks(HitKind::LunarEclipse)
    }

    /// The divisional charts varga ingresses are reported in.
    #[must_use]
    pub fn vargas(&self) -> &[Varga] {
        &self.vargas
    }

    /// The kinds of event asked for, in the order named.
    #[must_use]
    pub fn kinds(&self) -> &[HitKind] {
        &self.kinds
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
    /// `INVALID_ARG` naming `to`, `grahas`, `kinds`, `points`, `aspects`,
    /// `orb_deg`, `vargas` or `eclipse_orb_deg`.
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
        placeable(self.grahas.iter().copied(), "grahas")?;
        if self.kinds.is_empty() {
            return Err(Error::invalid_arg("no kind of event to report").with_field("kinds"));
        }
        if self.asks(HitKind::VargaIngress) {
            if self.vargas.is_empty() {
                return Err(
                    Error::invalid_arg("no divisional chart to report varga ingresses in")
                        .with_field("vargas"),
                );
            }
            repeated(&self.vargas, "vargas")?;
            if self.vargas.contains(&Varga::D1) {
                return Err(Error::invalid_arg(
                    "the rashi is no divisional chart to enter a new sign of",
                )
                .with_field("vargas")
                .with_hint("a graha entering a sign is the SIGN_INGRESS kind"));
            }
        }
        if self.asks(HitKind::Aspect) {
            if self.points.is_empty() {
                return Err(Error::invalid_arg("no natal point to aspect").with_field("points"));
            }
            if self.aspects.is_empty() {
                return Err(Error::invalid_arg("no aspect to report").with_field("aspects"));
            }
            self.check_points()?;
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
        self.check_eclipses()
    }

    /// The natal points, when a kind that reads them was asked for: at
    /// least one, none twice, each a point a chart places.
    fn check_points(&self) -> Result<(), Error> {
        if self.points.is_empty() {
            return Err(
                Error::invalid_arg("no natal point to aspect or eclipse").with_field("points")
            );
        }
        repeated(&self.points, "points")?;
        placeable(
            self.points.iter().filter_map(|point| match point {
                NatalPoint::Graha { graha } => Some(*graha),
                NatalPoint::Lagna | NatalPoint::Midheaven => None,
            }),
            "points",
        )
    }

    /// The eclipses' natal points and orb, when an eclipse was asked for.
    fn check_eclipses(&self) -> Result<(), Error> {
        if !self.asks_eclipses() {
            return Ok(());
        }
        self.check_points()?;
        let orb = self.eclipse_orb_deg;
        if !(orb > 0.0 && orb <= HitRequest::MOST_ECLIPSE_ORB_DEG) {
            return Err(Error::invalid_arg(format!(
                "an eclipse orb is more than 0 and at most {} degrees, not {orb}",
                HitRequest::MOST_ECLIPSE_ORB_DEG
            ))
            .with_field("eclipse_orb_deg"));
        }
        Ok(())
    }

    /// The request a binding writes as `hits`, read and checked: the window
    /// as UTC Julian days, and optionally the grahas, the kinds, the natal
    /// points, the aspects' angles, an orb and the divisional charts
    /// (`vargas`), each spelled as the answer spells it — a graha or a
    /// varga by its key, a kind as an event's `kind`, a point as an
    /// aspect's `to` or bare (`"LAGNA"`, `"MOON"`).
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
        if let Some(vargas) = asked.vargas {
            request = request.with_vargas(vargas);
        }
        if let Some(orb_deg) = asked.eclipse_orb_deg {
            request = request.with_eclipse_orb(orb_deg);
        }
        request.check().map_err(|why| {
            // The builder's fields are snake-cased; the record's are not.
            let why = match why.field() {
                Some("orb_deg") => why.with_field("orbDeg"),
                Some("eclipse_orb_deg") => why.with_field("eclipseOrbDeg"),
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "HitRequest"))]
#[serde(rename_all = "camelCase")]
pub(crate) struct Asked {
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
    #[serde(default)]
    vargas: Option<Vec<Varga>>,
    #[serde(default)]
    eclipse_orb_deg: Option<f64>,
}

/// A natal point as a request may name it: as an answer's `to` spells it
/// (`{"point": "GRAHA", "graha": "graha.MOON"}`, `{"point": "LAGNA"}`,
/// `{"point": "MIDHEAVEN"}`), or bare, a graha's key, `"LAGNA"` or
/// `"MIDHEAVEN"`.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub(crate) struct PointAsked(NatalPoint);

impl<'de> Deserialize<'de> for PointAsked {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        let value = serde_json::Value::deserialize(deserializer)?;
        let graha = |key: &str| {
            Graha::from_either_key(key)
                .map(|graha| PointAsked(NatalPoint::Graha { graha }))
                .map_err(D::Error::custom)
        };
        let shapes = "a point is \"LAGNA\", \"MIDHEAVEN\", a graha's key, or an aspect's `to`";
        match &value {
            serde_json::Value::String(key) if key == "LAGNA" => Ok(PointAsked(NatalPoint::Lagna)),
            serde_json::Value::String(key) if key == "MIDHEAVEN" => {
                Ok(PointAsked(NatalPoint::Midheaven))
            }
            serde_json::Value::String(key) => {
                graha(key).map_err(|why| D::Error::custom(format!("{why}; {shapes}")))
            }
            serde_json::Value::Object(fields) => {
                let field = |name: &str| fields.get(name).and_then(serde_json::Value::as_str);
                match (field("point"), field("graha"), fields.len()) {
                    (Some("LAGNA"), None, 1) => Ok(PointAsked(NatalPoint::Lagna)),
                    (Some("MIDHEAVEN"), None, 1) => Ok(PointAsked(NatalPoint::Midheaven)),
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

/// Refuses a graha no chart can place, by field: the nine, and the outer
/// planets a chart asked `with_outer_planets` places beside them
/// (`03-design/western-outer-planets.md`). Every member the catalogue
/// names today is one of the twelve; the check holds a member it adds.
fn placeable(grahas: impl IntoIterator<Item = Graha>, field: &str) -> Result<(), Error> {
    match grahas
        .into_iter()
        .find(|graha| !GRAHAS.contains(graha) && !OUTER.contains(graha))
    {
        Some(graha) => Err(Error::invalid_arg(format!(
            "{} is not a graha a chart places",
            graha.key()
        ))
        .with_field(field)
        .with_hint("a chart places the Sun to Saturn, Rahu and Ketu, and Uranus, Neptune and Pluto when asked")),
        None => Ok(()),
    }
}

/// Refuses a list that names one member twice, by field.
pub(crate) fn repeated<T: PartialEq + std::fmt::Debug>(
    members: &[T],
    field: &str,
) -> Result<(), Error> {
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
    Varga(Varga),
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
/// mean: the signs, the nakshatras and each divisional chart's lines once
/// for the group, and each chart's aspect lines, exact and at the orb's
/// two edges.
///
/// # Errors
///
/// A natal chart without a point asked for, named `points`.
pub(crate) fn lattices_of(
    request: &HitRequest,
    natals: &[&Document],
    midheavens: &[Option<f64>],
    members: &[usize],
) -> Result<Vec<(Meaning, Lattice)>, Error> {
    let mut meanings = Vec::new();
    if request.asks(HitKind::SignIngress) {
        meanings.push((Meaning::Sign, Lattice::SIGNS));
    }
    if request.asks(HitKind::NakshatraIngress) {
        meanings.push((Meaning::Nakshatra, Lattice::NAKSHATRAS));
    }
    if request.asks(HitKind::VargaIngress) {
        for varga in &request.vargas {
            let lines = teistro_vargas::change::lines(&Scheme::of(*varga)).ok_or_else(|| {
                Error::internal(format!("{} has no lattice of lines", varga.key()))
            })?;
            meanings.push((Meaning::Varga(*varga), lines));
        }
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
        for point in &request.points {
            let natal_deg = point_deg(natals, midheavens, *chart, *point)?;
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

/// One chart of the batch, by its index.
fn foundation_of<'d>(natals: &[&'d Document], chart: usize) -> Result<&'d ChartFoundation, Error> {
    natals
        .get(chart)
        .map(|natal| &natal.foundation)
        .ok_or_else(|| Error::internal("a group names a chart it was not given"))
}

/// A natal point's longitude in one chart of the batch, in the chart's
/// zodiac: a graha's or the lagna's from its foundation, the midheaven's
/// from its angles, read once for the batch.
///
/// # Errors
///
/// A chart without the point, named `points`: an outer planet the birth
/// was not founded with.
fn point_deg(
    natals: &[&Document],
    midheavens: &[Option<f64>],
    chart: usize,
    point: NatalPoint,
) -> Result<f64, Error> {
    let foundation = foundation_of(natals, chart)?;
    match point {
        NatalPoint::Graha { graha } => foundation.graha(graha).map(|at| at.longitude_deg),
        NatalPoint::Lagna => Some(foundation.lagna_deg),
        NatalPoint::Midheaven => midheavens.get(chart).copied().flatten(),
    }
    .ok_or_else(|| {
        Error::invalid_arg(format!("natal chart {chart} has no {point:?} to aspect"))
            .with_field("points")
            .with_hint(
                "an outer planet is in a chart founded `with_outer_planets`; found the natal chart with it",
            )
    })
}

/// One group's hits, each with the one chart it belongs to or `None` for
/// every chart of the group, and the stamp of the search that found them.
pub(crate) type GroupHits = (
    Vec<(Option<usize>, Hit)>,
    teistro_core::envelope::Provenance,
);

/// An eclipse of a hit list's window, as the eclipse search found it:
/// its greatest moment, UT1 read as UTC as the almanac's eclipses are,
/// and its kind (C421).
#[derive(Clone, Copy, Debug)]
pub(crate) enum SkyEclipse {
    Solar {
        greatest: JulianDay<Utc>,
        kind: SolarEclipseKind,
    },
    Lunar {
        greatest: JulianDay<Utc>,
        kind: LunarEclipseKind,
    },
}

impl SkyEclipse {
    /// The greatest eclipse.
    pub(crate) const fn instant(self) -> JulianDay<Utc> {
        match self {
            SkyEclipse::Solar { greatest, .. } | SkyEclipse::Lunar { greatest, .. } => greatest,
        }
    }

    /// The luminary eclipsed, whose place is the eclipse's: the Sun's in a
    /// solar eclipse, the Moon's in a lunar one.
    pub(crate) const fn luminary(self) -> Graha {
        match self {
            SkyEclipse::Solar { .. } => Graha::Sun,
            SkyEclipse::Lunar { .. } => Graha::Moon,
        }
    }

    /// The hit the eclipse is on a natal point.
    pub(crate) const fn on(self, to: NatalPoint) -> HitEvent {
        match self {
            SkyEclipse::Solar { kind, .. } => HitEvent::SolarEclipse { to, eclipse: kind },
            SkyEclipse::Lunar { kind, .. } => HitEvent::LunarEclipse { to, eclipse: kind },
        }
    }
}

/// Every eclipse of the window that falls on a natal point of a group's
/// charts, each with the chart it is the hit of: its luminary, read in the
/// chart's zodiac at the greatest moment (`luminaries`, a row an eclipse,
/// as `Founder::longitudes` reads the nine), within the eclipse orb of the
/// point (C421, C422).
///
/// # Errors
///
/// A natal chart without a point asked for, as [`lattices_of`] refuses.
pub(crate) fn eclipse_hits(
    request: &HitRequest,
    (natals, midheavens, members): (&[&Document], &[Option<f64>], &[usize]),
    eclipses: &[SkyEclipse],
    luminaries: &[[f64; 9]],
) -> Result<Vec<(usize, Hit)>, Error> {
    let mut found = Vec::new();
    for (eclipse, row) in eclipses.iter().zip(luminaries) {
        let graha = eclipse.luminary();
        let Some(&luminary_deg) = row.get(graha as usize) else {
            return Err(Error::internal(format!(
                "no place of {graha:?} at an eclipse"
            )));
        };
        for chart in members {
            for point in &request.points {
                let natal_deg = point_deg(natals, midheavens, *chart, *point)?;
                if hits::eclipse_reaches(luminary_deg, natal_deg, request.eclipse_orb_deg) {
                    found.push((
                        *chart,
                        Hit {
                            instant: eclipse.instant(),
                            graha,
                            event: eclipse.on(*point),
                        },
                    ));
                }
            }
        }
    }
    Ok(found)
}

/// What a crossing of the Moon's elongation from the Sun means: a phase,
/// which is the sky's, or one chart's phase return (by its index in the
/// batch).
#[derive(Clone, Copy, Debug)]
pub(crate) enum PhaseMeaning {
    Phases,
    Return(usize),
}

/// The lattices of the elongation one group's scan tests: the quarters
/// once for the group, and each chart's natal elongation once a circle.
///
/// # Errors
///
/// A natal chart without its Sun or Moon, which a founded chart always
/// places.
pub(crate) fn phase_lattices_of(
    request: &HitRequest,
    natals: &[&Document],
    members: &[usize],
) -> Result<Vec<(PhaseMeaning, Lattice)>, Error> {
    let mut meanings = Vec::new();
    if request.asks(HitKind::LunarPhase) {
        meanings.push((
            PhaseMeaning::Phases,
            Lattice {
                origin_deg: 0.0,
                step_deg: 90.0,
            },
        ));
    }
    if request.asks(HitKind::PhaseReturn) {
        for chart in members {
            let foundation = foundation_of(natals, *chart)?;
            let at = |graha: Graha| {
                foundation
                    .graha(graha)
                    .map(|place| place.longitude_deg)
                    .ok_or_else(|| Error::internal(format!("natal chart {chart} has no {graha:?}")))
            };
            meanings.push((
                PhaseMeaning::Return(*chart),
                Lattice {
                    origin_deg: natal_elongation_deg(at(Graha::Moon)?, at(Graha::Sun)?),
                    step_deg: 360.0,
                },
            ));
        }
    }
    Ok(meanings)
}

/// The phase a chart was born in: the Moon's longitude less the Sun's,
/// in `[0, 360)` (C420). Any zodiac gives the same, the ayanamsha
/// cancelling.
pub(crate) fn natal_elongation_deg(moon_deg: f64, sun_deg: f64) -> f64 {
    teistro_core::angle::normalise_deg(moon_deg - sun_deg)
}

/// The hit a crossing of the elongation is, and the one chart it belongs
/// to when it is a phase return.
pub(crate) fn phase_hit_of(
    event: &TransitEvent,
    meanings: &[(PhaseMeaning, Lattice)],
) -> Option<(HitEvent, Option<usize>)> {
    let TransitEventKind::Crossing {
        lattice,
        boundary_deg,
        ..
    } = event.kind
    else {
        return None;
    };
    match meanings.get(lattice)?.0 {
        PhaseMeaning::Phases => Some((hits::lunar_phase(boundary_deg), None)),
        PhaseMeaning::Return(chart) => Some((HitEvent::PhaseReturn, Some(chart))),
    }
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
            // Most of the trimshamsha's lines, and the hora's at each
            // sign's end, leave the sign as it was: no ingress.
            Meaning::Varga(varga) => hits::varga_ingress(
                varga,
                teistro_vargas::change::either_side(&Scheme::of(varga), boundary_deg).ok()?,
                motion(direction),
            )
            .map(|ingress| (ingress, None)),
            Meaning::Aspect(chart, point, natal_deg, edge) => hits::aspect_hit(
                (point, natal_deg),
                (boundary_deg, edge, request.orb_deg.unwrap_or(0.0)),
                motion(direction),
                &request.aspects,
            )
            .map(|aspect| (aspect, Some(chart))),
        },
        TransitEventKind::Station { kind, .. } => Some((station(kind), None)),
    }
}

/// The hit a station is: the way the graha turned.
pub(crate) const fn station(kind: StationKind) -> HitEvent {
    HitEvent::Station {
        turns: match kind {
            StationKind::Retrograde => Motion::Retrograde,
            StationKind::Direct => Motion::Direct,
        },
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::float_cmp,
        reason = "tests fail by panicking, and the elongations they compare are exact"
    )]

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
        let asked = HitRequest::from_json(&format!(
            r#"{{{WINDOW}, "points": ["MIDHEAVEN", {{"point": "MIDHEAVEN"}}]}}"#
        ));
        assert_eq!(asked.unwrap_err().field(), Some("hits.points"));
        let asked = HitRequest::from_json(&format!(
            r#"{{{WINDOW}, "points": [{{"point": "MIDHEAVEN"}}, "LAGNA"]}}"#
        ))
        .unwrap();
        assert_eq!(asked.points(), [NatalPoint::Midheaven, NatalPoint::Lagna]);
        let asked = HitRequest::from_json(&format!(
            r#"{{{WINDOW}, "points": [{{"point": "MIDHEAVEN", "graha": "SUN"}}]}}"#
        ));
        assert_eq!(asked.unwrap_err().field(), Some("hits.points[0]"));
    }

    #[test]
    fn a_new_kind_is_asked_for_by_name() {
        // The default is the four the list began with, so no list a
        // caller already reads grows.
        let asked = HitRequest::from_json(&format!("{{{WINDOW}}}")).unwrap();
        assert_eq!(asked.kinds(), HitKind::DEFAULT);
        assert!(!asked.asks(HitKind::VargaIngress));
        let asked = HitRequest::from_json(&format!(
            r#"{{{WINDOW}, "kinds": ["VARGA_INGRESS"], "vargas": ["D9", "varga.D30"]}}"#
        ))
        .unwrap();
        assert_eq!(asked.vargas(), [Varga::D9, Varga::D30]);
        // Asked for without charts, it reports the navamsha's.
        let asked =
            HitRequest::from_json(&format!(r#"{{{WINDOW}, "kinds": ["VARGA_INGRESS"]}}"#)).unwrap();
        assert_eq!(asked.vargas(), HitRequest::DEFAULT_VARGAS);
        // Charts named beside kinds that never read them are not checked.
        assert!(HitRequest::from_json(&format!(r#"{{{WINDOW}, "vargas": []}}"#)).is_ok());
        let asked = HitRequest::from_json(&format!(
            r#"{{{WINDOW}, "kinds": ["LUNAR_PHASE", "PHASE_RETURN"]}}"#
        ))
        .unwrap();
        assert!(asked.asks(HitKind::LunarPhase) && asked.asks(HitKind::PhaseReturn));
    }

    #[test]
    fn a_natal_phase_is_the_moon_less_the_sun() {
        assert_eq!(natal_elongation_deg(100.0, 10.0), 90.0);
        // A waning Moon is past 180°, not the same square from the other
        // side: the return keeps the lunation's half.
        assert_eq!(natal_elongation_deg(10.0, 100.0), 270.0);
        assert_eq!(natal_elongation_deg(5.0, 5.0), 0.0);
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
            (
                r#", "kinds": ["VARGA_INGRESS"], "vargas": []"#,
                "hits.vargas",
            ),
            (
                r#", "kinds": ["VARGA_INGRESS"], "vargas": ["D9", "D9"]"#,
                "hits.vargas",
            ),
            (
                r#", "kinds": ["VARGA_INGRESS"], "vargas": ["D1"]"#,
                "hits.vargas",
            ),
            (
                r#", "kinds": ["SOLAR_ECLIPSE"], "eclipseOrbDeg": 0"#,
                "hits.eclipseOrbDeg",
            ),
            (
                r#", "kinds": ["LUNAR_ECLIPSE"], "eclipseOrbDeg": 16"#,
                "hits.eclipseOrbDeg",
            ),
            (
                r#", "kinds": ["LUNAR_ECLIPSE"], "points": []"#,
                "hits.points",
            ),
        ] {
            assert_eq!(refused(rest).field(), Some(field), "{rest}");
        }
        assert_eq!(
            refused(r#", "grahas": ["PLUTO", "PLUTO"]"#).field(),
            Some("hits.grahas")
        );
        assert_eq!(
            refused(r#", "points": ["URANUS", "URANUS"]"#).field(),
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
