//! Gochar through the façade: the transits at many instants, each read
//! from the natal chart's reference sign (`03-design/gochar.md`).

use serde::{Deserialize, Serialize};
use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_gochar::{GocharFrom, GocharReading, GocharRules, Reference, Transit, gochar};

/// The instants to read the transits at, and what to count them from.
///
/// ```
/// use teistro::quantity::{JulianDay, Utc};
/// use teistro::{GocharFrom, GocharRequest};
///
/// // A year of daily snapshots, counted from the natal Moon by default.
/// let days: Vec<_> = (0..365).map(|d| JulianDay::<Utc>::literal(2_460_676.5 + f64::from(d))).collect();
/// let year = GocharRequest::over(days);
/// assert_eq!(year.from(), GocharFrom::Moon);
/// let from_the_lagna = GocharRequest::at(JulianDay::literal(2_460_676.5)).counted_from(GocharFrom::Lagna);
/// assert_eq!(from_the_lagna.instants().len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct GocharRequest {
    instants: Vec<JulianDay<Utc>>,
    from: GocharFrom,
}

impl GocharRequest {
    /// The transits at one instant.
    #[must_use]
    pub fn at(instant: JulianDay<Utc>) -> GocharRequest {
        GocharRequest::over([instant])
    }

    /// The transits at each of many instants, in the order given: the
    /// grahas placed at every instant in one request, without founding a
    /// transit chart, and the natal reference read once.
    #[must_use]
    pub fn over(instants: impl IntoIterator<Item = JulianDay<Utc>>) -> GocharRequest {
        GocharRequest {
            instants: instants.into_iter().collect(),
            from: GocharFrom::Moon,
        }
    }

    /// Counted from another reference than the natal Moon.
    #[must_use]
    pub const fn counted_from(mut self, from: GocharFrom) -> GocharRequest {
        self.from = from;
        self
    }

    /// The instants asked for.
    #[must_use]
    pub fn instants(&self) -> &[JulianDay<Utc>] {
        &self.instants
    }

    /// What the transits are counted from.
    #[must_use]
    pub const fn from(&self) -> GocharFrom {
        self.from
    }

    /// The request a binding writes as `gochar`, read and checked: the
    /// instants as UTC Julian days and, optionally, what to count from.
    ///
    /// ```
    /// use teistro::{GocharFrom, GocharRequest};
    ///
    /// let asked = GocharRequest::from_json(r#"{"instants": [2460676.5, 2460677.5], "from": "LAGNA"}"#)?;
    /// assert_eq!((asked.instants().len(), asked.from()), (2, GocharFrom::Lagna));
    /// // A refusal names the field the caller wrote.
    /// let empty = GocharRequest::from_json(r#"{"instants": []}"#).unwrap_err();
    /// assert_eq!(empty.field(), Some("gochar.instants"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, an unknown reference, or no instant at all: a request for the
    /// transits at no time is a mistake rather than an empty answer.
    pub fn from_json(text: &str) -> Result<GocharRequest, Error> {
        let asked: Asked = teistro_core::strict::read(text, GOCHAR)?;
        if asked.instants.is_empty() {
            return Err(Error::invalid_arg("no instant to read the transits at")
                .with_field(format!("{GOCHAR}.instants"))
                .with_hint("name at least one UTC Julian day"));
        }
        let instants = asked
            .instants
            .iter()
            .enumerate()
            .map(|(at, jd)| {
                JulianDay::<Utc>::try_new(*jd)
                    .map_err(|why| Error::from(why).with_field(format!("{GOCHAR}.instants[{at}]")))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(GocharRequest::over(instants).counted_from(asked.from))
    }
}

/// The record every binding writes the request as.
const GOCHAR: &str = "gochar";

/// [`GocharRequest`] as the bindings write it.
#[derive(Serialize, Deserialize)]
struct Asked {
    instants: Vec<f64>,
    #[serde(default)]
    from: GocharFrom,
}

/// A sign from a sidereal longitude.
fn sign_of(longitude_deg: f64) -> Rashi {
    Transit::at_longitude(longitude_deg).sign
}

/// The natal chart's reference sign for `from`.
pub(crate) fn reference(natal: &ChartFoundation, from: GocharFrom) -> Result<Reference, Error> {
    match from {
        GocharFrom::Lagna => Rashi::from_id(u16::from(natal.lagna_sign_index()))
            .map(Reference::lagna)
            .ok_or_else(|| Error::internal("a lagna in no sign")),
        _ => natal
            .graha(Graha::Moon)
            .map(|moon| Reference::moon(sign_of(moon.longitude_deg)))
            .ok_or_else(|| Error::internal("a founded chart places the Moon")),
    }
}

/// One instant's gochar from `reference`, over the grahas' places in the
/// chart's zodiac, the Sun to Ketu.
pub(crate) fn reading(
    longitudes: &[f64; 9],
    reference: Reference,
    rules: GocharRules,
) -> GocharReading {
    gochar(reference, &longitudes.map(Transit::at_longitude), rules)
}
