//! The Sarvatobhadra chakra through the façade: gochar's second layer, the
//! vedhas the transiting grahas strike on the native's points
//! (`03-design/gochar.md` §8).

use serde::{Deserialize, Serialize};
use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::{ChartKind, Graha};
use teistro_core::error::Error;
use teistro_core::house::House;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_gochar::Transit;
use teistro_gochar::sarvatobhadra::{
    Motion, NameSounds, NativePoints, SarvatobhadraReading, SarvatobhadraRules, StarPlace,
    sarvatobhadra,
};
use teistro_serial::Document;
use teistro_siddhanta::params::{Parameters, Planet};

use crate::Envelope;
use crate::area::ChartArea;

/// Where the five from Mars take the motion v. 15 sorts their sight by
/// (C430). The Sun, the Moon and the nodes have theirs from v. 19 under
/// every one.
///
/// ```
/// use teistro::SarvatobhadraMotion;
/// use teistro::gochar::sarvatobhadra::Motion;
///
/// assert_eq!(SarvatobhadraMotion::default(), SarvatobhadraMotion::Sky);
/// // Saturn retrograde, the rest at their mean.
/// let given = SarvatobhadraMotion::Given([Motion::Mean, Motion::Mean, Motion::Mean, Motion::Mean, Motion::Retrograde]);
/// # let _ = given;
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SarvatobhadraMotion {
    /// The sky's: each graha's speed in the transit chart against its mean
    /// daily motion ([`Motion::by_speed`]), the mean the *Surya
    /// Siddhanta*'s revolutions give (I.29 to 34), Mercury's and Venus's
    /// the Sun's. The tika gives vv. 16–18 as the motion "without
    /// computing the grahas", so the computed motion is the one they stand
    /// in for.
    #[default]
    Sky,
    /// vv. 16–18's, read off the Sun's sign counted from the graha's
    /// ([`Motion::by_the_sun`]). A graha in the Sun's own sign is unseen
    /// (v. 18) and the verses give it no motion, so it takes the sky's.
    /// The rule makes Mercury and Venus never retrograde (§8.2).
    Sun,
    /// The caller's, for Mars, Mercury, Jupiter, Venus and Saturn in that
    /// order, at every instant asked.
    Given([Motion; 5]),
}

/// The instants to read the chakra at, the name, if one is read, how the
/// five's motion is taken and the readings the texts leave open.
///
/// ```
/// use teistro::quantity::{JulianDay, Utc};
/// use teistro::{SarvatobhadraMotion, SarvatobhadraRequest};
/// use teistro::gochar::sarvatobhadra::{NameSounds, SarvatobhadraRules, Vowel};
///
/// let today = SarvatobhadraRequest::at(JulianDay::<Utc>::literal(2_461_000.5));
/// assert_eq!(today.motion(), SarvatobhadraMotion::Sky);
/// assert_eq!(today.rules(), SarvatobhadraRules::TEXT);
/// // Rama: the letter ra, the vowel a.
/// let named = today.named(NameSounds { letter: 'र', vowel: Vowel::A });
/// assert!(named.name().is_some());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SarvatobhadraRequest {
    instants: Vec<JulianDay<Utc>>,
    name: Option<NameSounds>,
    motion: SarvatobhadraMotion,
    rules: SarvatobhadraRules,
}

impl SarvatobhadraRequest {
    /// The chakra at one instant.
    #[must_use]
    pub fn at(instant: JulianDay<Utc>) -> SarvatobhadraRequest {
        SarvatobhadraRequest::over([instant])
    }

    /// The chakra at each of many instants, in the order given, every
    /// transit chart founded in one batch: without a name, the five's
    /// motion the sky's and the texts as §8 reads them.
    #[must_use]
    pub fn over(instants: impl IntoIterator<Item = JulianDay<Utc>>) -> SarvatobhadraRequest {
        SarvatobhadraRequest {
            instants: instants.into_iter().collect(),
            name: None,
            motion: SarvatobhadraMotion::Sky,
            rules: SarvatobhadraRules::TEXT,
        }
    }

    /// The same request, reading the name's letter and vowel as points.
    #[must_use]
    pub const fn named(mut self, name: NameSounds) -> SarvatobhadraRequest {
        self.name = Some(name);
        self
    }

    /// The same request, the five's motion taken another way.
    #[must_use]
    pub const fn with_motion(mut self, motion: SarvatobhadraMotion) -> SarvatobhadraRequest {
        self.motion = motion;
        self
    }

    /// The same request under other readings of the texts (C429 to C431).
    #[must_use]
    pub const fn with_rules(mut self, rules: SarvatobhadraRules) -> SarvatobhadraRequest {
        self.rules = rules;
        self
    }

    /// The instants asked for.
    #[must_use]
    pub fn instants(&self) -> &[JulianDay<Utc>] {
        &self.instants
    }

    /// The name, when one is read.
    #[must_use]
    pub const fn name(&self) -> Option<NameSounds> {
        self.name
    }

    /// How the five's motion is taken.
    #[must_use]
    pub const fn motion(&self) -> SarvatobhadraMotion {
        self.motion
    }

    /// The readings of the texts.
    #[must_use]
    pub const fn rules(&self) -> SarvatobhadraRules {
        self.rules
    }

    /// The request a binding writes as `sarvatobhadra`, read and checked:
    /// the instants as UTC Julian days and, optionally, `name`
    /// (`{letter, vowel}`), `motion` and `rules`.
    ///
    /// ```
    /// use teistro::{SarvatobhadraMotion, SarvatobhadraRequest};
    ///
    /// let asked = SarvatobhadraRequest::from_json(r#"{"instants": [2460676.5], "motion": "SUN"}"#)?;
    /// assert_eq!(asked.motion(), SarvatobhadraMotion::Sun);
    /// // A refusal names the field the caller wrote.
    /// let empty = SarvatobhadraRequest::from_json(r#"{"instants": []}"#).unwrap_err();
    /// assert_eq!(empty.field(), Some("sarvatobhadra.instants"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or no instant at all.
    pub fn from_json(text: &str) -> Result<SarvatobhadraRequest, Error> {
        let asked: Asked = teistro_core::strict::read(text, SARVATOBHADRA)?;
        let instants = asked
            .instants
            .iter()
            .enumerate()
            .map(|(at, jd)| {
                JulianDay::<Utc>::try_new(*jd).map_err(|why| {
                    Error::from(why).with_field(format!("{SARVATOBHADRA}.instants[{at}]"))
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let request = SarvatobhadraRequest {
            instants,
            name: asked.name,
            motion: asked.motion,
            rules: asked.rules,
        };
        request.check().map_err(|why| why.under(SARVATOBHADRA))?;
        Ok(request)
    }

    /// Whether the request can be answered: at least one instant, since
    /// the chakra at no time is a mistake rather than an empty answer.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `instants`.
    pub fn check(&self) -> Result<(), Error> {
        if self.instants.is_empty() {
            return Err(Error::invalid_arg("no instant to read the chakra at")
                .with_field("instants")
                .with_hint("name at least one UTC Julian day"));
        }
        Ok(())
    }
}

/// The record every binding writes the request as.
const SARVATOBHADRA: &str = "sarvatobhadra";

/// [`SarvatobhadraRequest`] as the bindings write it.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "SarvatobhadraRequest"))]
#[serde(deny_unknown_fields)]
pub(crate) struct Asked {
    instants: Vec<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<NameSounds>,
    #[serde(default)]
    motion: SarvatobhadraMotion,
    #[serde(default)]
    rules: SarvatobhadraRules,
}

/// The five from Mars, in the catalogue's order, as the kernel takes their
/// motions.
const FIVE: [Graha; 5] = [
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// A graha's mean daily motion, degrees: the *Surya Siddhanta*'s
/// revolutions over the civil days of an age (I.29 to 37), Mercury's and
/// Venus's being the Sun's (I.29).
fn mean_motion(graha: Graha) -> f64 {
    let planet = match graha {
        Graha::Mars => Planet::Mars,
        Graha::Jupiter => Planet::Jupiter,
        Graha::Saturn => Planet::Saturn,
        _ => Planet::Sun,
    };
    let text = &Parameters::TEXT;
    text.motion(planet).degrees_per_day(text)
}

/// The five's motions in a transit chart, taken as `asked`.
fn motions_of(transit: &ChartFoundation, asked: SarvatobhadraMotion) -> Result<[Motion; 5], Error> {
    if let SarvatobhadraMotion::Given(given) = asked {
        return Ok(given);
    }
    let sun = transit
        .graha(Graha::Sun)
        .map(|at| Transit::at_longitude(at.longitude_deg).sign)
        .ok_or_else(|| Error::internal("a founded chart places the Sun"))?;
    let mut motions = [Motion::Mean; 5];
    for (slot, graha) in motions.iter_mut().zip(FIVE) {
        let at = transit
            .graha(graha)
            .ok_or_else(|| Error::internal(format!("a founded chart places {}", graha.key())))?;
        let sky = Motion::by_speed(at.speed_deg_per_day, mean_motion(graha));
        *slot = match asked {
            SarvatobhadraMotion::Sun => Motion::by_the_sun(
                House::between(Transit::at_longitude(at.longitude_deg).sign, sun).get(),
            )
            .unwrap_or(sky),
            _ => sky,
        };
    }
    Ok(motions)
}

/// A transit chart as the kernel reads it: the nine's places and the
/// five's motions.
type Sky = ([Transit; 9], [Motion; 5]);

/// The nine's places in a transit chart, the Sun to Ketu, as the kernel
/// takes them.
fn sky_of(transit: &ChartFoundation) -> Result<[Transit; 9], Error> {
    let mut sky = [Transit::at_longitude(0.0); 9];
    // The catalogue's first nine are the nine.
    for (slot, graha) in sky.iter_mut().zip(Graha::ALL) {
        let at = transit
            .graha(graha)
            .ok_or_else(|| Error::internal(format!("a founded chart places {}", graha.key())))?;
        *slot = Transit::at_longitude(at.longitude_deg);
    }
    Ok(sky)
}

/// The native's points from a natal document: the Moon's star and sign,
/// and the tithi and weekday of birth from its panchanga.
fn native_of(natal: &Document, name: Option<NameSounds>) -> Result<NativePoints, Error> {
    let day = natal.panchanga.as_ref().ok_or_else(|| {
        Error::invalid_arg(
            "the natal document carries no panchanga, which the native's tithi and weekday \
             are read from; ask for it with `ChartRequest::with_panchanga`",
        )
        .with_field("panchanga")
    })?;
    let instant = natal.foundation.instant;
    let tithi = day
        .tithi_at(instant)
        .ok_or_else(|| Error::internal("the birth's almanac runs no tithi at the birth"))?
        .member;
    let moon = natal
        .foundation
        .graha(Graha::Moon)
        .ok_or_else(|| Error::internal("a founded chart places the Moon"))?;
    Ok(NativePoints {
        nakshatra: StarPlace::at(moon.longitude_deg).nakshatra,
        rashi: Transit::at_longitude(moon.longitude_deg).sign,
        tithi,
        vara: day.vara(),
        name,
    })
}

impl ChartArea<'_> {
    /// The **Sarvatobhadra chakra** at each instant asked, read against a
    /// natal chart: where each transiting graha stands among the chakra's
    /// 28 stars, the lines it looks along, every cell it strikes, and
    /// which of the native's points the malefics and the benefics strike
    /// (Phaladeepika ch. 26 v. 48, *Narapati Jayacharya Svarodaya* vv. 2 to
    /// 30; `03-design/gochar.md` §8). It reports clauses and scores
    /// nothing; [`gochar`](Self::gochar)'s verdicts are its own.
    ///
    /// The native's star and sign are the natal Moon's, the tithi and
    /// weekday the birth's from the document's panchanga, and the name's
    /// letter and vowel the request's. The transit charts are founded at
    /// the natal place in one batch, their grahas in the chart's zodiac.
    ///
    /// ```no_run
    /// # use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
    /// # use teistro::{ChartRequest, Context, Ephemeris, SarvatobhadraRequest, UtcOffset};
    /// use teistro::gochar::sarvatobhadra::VedhaPoint;
    /// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let place = Place::new(Latitude::try_new(27.7)?, Longitude::try_new(85.3)?, Altitude::try_new(1400.0)?);
    /// let asked = ChartRequest::at(place, UtcOffset::literal(5, 45, 0)).with_panchanga();
    /// let natal = sdk.chart().reading(JulianDay::literal(2_447_995.489_583_333_5), &asked)?.value;
    /// let today = SarvatobhadraRequest::at(JulianDay::<Utc>::literal(2_461_000.5));
    /// let read = &sdk.chart().sarvatobhadra(&natal, &today)?.value[0];
    /// // v. 28: the native's points a malefic strikes, star, sign, tithi,
    /// // and the letter and vowel when a name was given.
    /// let star = read.points.iter().find(|p| p.point == VedhaPoint::Nakshatra);
    /// println!("{} of five struck", read.malefic_points);
    /// # let _ = star;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// No instant, naming `instants`; a natal document without its
    /// panchanga, naming `panchanga`; a name's letter the chakra does not
    /// read, naming `native.name.letter`; whatever founding a transit chart
    /// refuses, such as an instant the ephemeris does not cover.
    pub fn sarvatobhadra(
        self,
        natal: &Document,
        request: &SarvatobhadraRequest,
    ) -> Result<Envelope<Vec<SarvatobhadraReading>>, Error> {
        let found = self.sarvatobhadra_many([natal], request)?;
        let provenance = found.provenance.clone();
        let one = found
            .value
            .into_iter()
            .next()
            .ok_or_else(|| Error::internal("one chart was asked about and none answered"))?;
        Ok(Envelope::sealing(one, provenance))
    }

    /// [`ChartArea::sarvatobhadra`] for many natal charts at the same
    /// instants, the transit charts founded **once** for every natal chart
    /// at one place and of one kind, which a batch's charts are. One list
    /// of readings per chart, in the order given.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::sarvatobhadra`], a refusal of one chart's points
    /// hinting which chart.
    pub fn sarvatobhadra_many<'d>(
        self,
        natals: impl IntoIterator<Item = &'d Document>,
        request: &SarvatobhadraRequest,
    ) -> Result<Envelope<Vec<Vec<SarvatobhadraReading>>>, Error> {
        request.check()?;
        // Each place and kind's transit charts, read into the kernel's
        // sky and motions once, and every chart there read against them.
        let mut skies: Vec<(Place, ChartKind, Vec<Sky>)> = Vec::new();
        let mut provenance = None;
        let mut answers = Vec::new();
        for (at, natal) in natals.into_iter().enumerate() {
            let native = native_of(natal, request.name())
                .map_err(|error| error.with_hint(format!("chart {at}")))?;
            let (place, kind) = (natal.foundation.place, natal.foundation.kind);
            let known = skies.iter().position(|(p, k, _)| *p == place && *k == kind);
            let index = if let Some(index) = known {
                index
            } else {
                let found = self.founding(UtcOffset::UTC, |founder| {
                    founder.found(request.instants(), &place, kind)
                })?;
                let read = found
                    .value
                    .iter()
                    .map(|transit| Ok((sky_of(transit)?, motions_of(transit, request.motion())?)))
                    .collect::<Result<Vec<_>, Error>>()?;
                provenance.get_or_insert(found.provenance);
                skies.push((place, kind, read));
                skies.len() - 1
            };
            let read = skies
                .get(index)
                .ok_or_else(|| Error::internal("a sky was read and not kept"))?;
            answers.push(
                read.2
                    .iter()
                    .map(|(sky, motions)| sarvatobhadra(sky, motions, &native, request.rules()))
                    .collect::<Result<Vec<_>, Error>>()?,
            );
        }
        let provenance = provenance.ok_or_else(|| {
            Error::invalid_arg("no natal chart to read the chakra against").with_field("natals")
        })?;
        Ok(Envelope::sealing(answers, provenance))
    }
}
