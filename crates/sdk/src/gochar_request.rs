//! Gochar through the façade: the transits at many instants, each read
//! from the natal chart's reference sign (`03-design/gochar.md`).

use serde::{Deserialize, Serialize};
use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_gochar::ashtakavarga::{self, Prastara};
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
    ashtakavarga: bool,
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
            ashtakavarga: false,
        }
    }

    /// Counted from another reference than the natal Moon.
    #[must_use]
    pub const fn counted_from(mut self, from: GocharFrom) -> GocharRequest {
        self.from = from;
        self
    }

    /// The same request, each reading also judging the seven's transits by
    /// the natal Ashtakavarga (`03-design/gochar-ashtakavarga.md`).
    #[must_use]
    pub const fn with_ashtakavarga(mut self) -> GocharRequest {
        self.ashtakavarga = true;
        self
    }

    /// Whether the Ashtakavarga's reading was asked for.
    #[must_use]
    pub const fn ashtakavarga(&self) -> bool {
        self.ashtakavarga
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
        let request = GocharRequest::over(instants).counted_from(asked.from);
        Ok(if asked.ashtakavarga {
            request.with_ashtakavarga()
        } else {
            request
        })
    }
}

/// The record every binding writes the request as.
const GOCHAR: &str = "gochar";

/// [`GocharRequest`] as the bindings write it.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "GocharRequest"))]
pub(crate) struct Asked {
    instants: Vec<f64>,
    #[serde(default)]
    from: GocharFrom,
    #[serde(default)]
    ashtakavarga: bool,
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
/// chart's zodiac, the Sun to Ketu; and, given the natal prastara, the
/// seven's transits judged by it.
pub(crate) fn reading(
    longitudes: &[f64; 9],
    reference: Reference,
    rules: GocharRules,
    prastara: Option<&Prastara>,
) -> GocharReading {
    let transits = longitudes.map(Transit::at_longitude);
    let mut read = gochar(reference, &transits, rules);
    read.ashtakavarga = prastara.map(|prastara| ashtakavarga::transits(prastara, &transits, rules));
    read
}

/// The `GOCHAR` section of a chart's dossier: each graha's transit at each
/// of the `gochar` record's instants, a row a graha an instant
/// (`03-design/dossier.md`). A chart request writes it from its `gochar`
/// record; a caller of [`crate::dossier::dossier`] passes it as an extra
/// section with the instants and the chart's own readings.
#[derive(Clone, Copy, Debug)]
pub struct GocharSection<'a> {
    instants: &'a [JulianDay<Utc>],
    readings: Option<&'a [GocharReading]>,
}

impl<'a> GocharSection<'a> {
    /// The section's key, as a dossier record names it.
    pub const KEY: &'static str = "GOCHAR";

    /// The section of `readings`, one an instant of `instants`; `None`, a
    /// chart read at no instant, whose dossier has no `GOCHAR` to write.
    #[must_use]
    pub const fn new(
        instants: &'a [JulianDay<Utc>],
        readings: Option<&'a [GocharReading]>,
    ) -> GocharSection<'a> {
        GocharSection { instants, readings }
    }
}

impl crate::dossier::DossierSection for GocharSection<'_> {
    fn key(&self) -> &'static str {
        GocharSection::KEY
    }
    fn columns(&self) -> &[crate::dossier::Column] {
        use crate::dossier::{Column, column};
        const COLUMNS: [Column; 9] = [
            column("at", "the instant, on the dossier's clock"),
            column("graha", "the graha in transit"),
            column("sign", "the sign it transits"),
            column("deg", "degrees in the sign, two decimals"),
            column(
                "house",
                "its house from the natal Moon's sign, or the reference the record names",
            ),
            column(
                "verdict",
                "GOOD, OBSTRUCTED or NOT_GOOD (Phaladeepika ch. 26)",
            ),
            column("vedha", "the house whose occupant would obstruct it"),
            column("obstructed", "the grahas there that do, `,`-separated"),
            column("fruitful", "whether it stands in its decanate of fruition"),
        ];
        &COLUMNS
    }
    fn asked_by(&self) -> &'static str {
        "the `gochar` record on the chart request"
    }
    fn rows(&self, _: &crate::Document, view: &crate::dossier::View) -> Option<Vec<Vec<String>>> {
        use crate::dossier::{civil, fixed, word};
        let dash = || String::from("-");
        Some(
            self.readings?
                .iter()
                .zip(self.instants)
                .flat_map(|(reading, instant)| {
                    let at = civil(*instant, view.utc_offset_seconds);
                    reading.grahas.iter().map(move |one| {
                        vec![
                            at.clone(),
                            one.graha.key().to_owned(),
                            one.transit.sign.key().to_owned(),
                            fixed(one.transit.degrees, 2),
                            one.house.to_string(),
                            word(&one.verdict),
                            one.vedha_house.map_or_else(dash, |house| house.to_string()),
                            if one.obstructed_by.is_empty() {
                                dash()
                            } else {
                                one.obstructed_by
                                    .iter()
                                    .map(|graha| graha.key())
                                    .collect::<Vec<_>>()
                                    .join(",")
                            },
                            one.fruitful_now.to_string(),
                        ]
                    })
                })
                .collect(),
        )
    }
}
