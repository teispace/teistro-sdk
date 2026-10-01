//! The seven planets' dignities in one chart, under named rules
//! (`03-design/essential-dignities.md` §The façade).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

use crate::dignity::{
    CHALDEAN_ORDER, DignityRules, EssentialDignity, Scores, Sect, SectRule, essential_dignity,
};

/// What a chart's dignities are read from: the seven planets' longitudes
/// in the chart's zodiac, the Sun's geometric altitude, and whether the
/// chart's instant falls between its sunrise and its sunset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartSky {
    /// Saturn's longitude, in degrees.
    pub saturn_deg: f64,
    /// Jupiter's longitude, in degrees.
    pub jupiter_deg: f64,
    /// Mars's longitude, in degrees.
    pub mars_deg: f64,
    /// The Sun's longitude, in degrees.
    pub sun_deg: f64,
    /// Venus's longitude, in degrees.
    pub venus_deg: f64,
    /// Mercury's longitude, in degrees.
    pub mercury_deg: f64,
    /// The Moon's longitude, in degrees.
    pub moon_deg: f64,
    /// The Sun's centre above the true horizon, in degrees, which
    /// [`SectRule::Horizon`] reads.
    pub sun_altitude_deg: f64,
    /// Whether the instant is between the chart's sunrise and its sunset,
    /// which [`SectRule::Daylight`] reads.
    pub daylight: bool,
}

impl ChartSky {
    /// One of the seven's longitude and the field it is read from; `None`
    /// for a graha that holds no essential dignity.
    fn of(&self, planet: Graha) -> Option<(f64, &'static str)> {
        Some(match planet {
            Graha::Saturn => (self.saturn_deg, "saturn_deg"),
            Graha::Jupiter => (self.jupiter_deg, "jupiter_deg"),
            Graha::Mars => (self.mars_deg, "mars_deg"),
            Graha::Sun => (self.sun_deg, "sun_deg"),
            Graha::Venus => (self.venus_deg, "venus_deg"),
            Graha::Mercury => (self.mercury_deg, "mercury_deg"),
            Graha::Moon => (self.moon_deg, "moon_deg"),
            _ => return None,
        })
    }
}

/// How a chart's dignities are read: the sect's rule, the terms and
/// triplicities, and the scores.
///
/// The default is Valens's horizon for the sect and Lilly for the rest,
/// the one complete, scored, cited reading; every part is a knob, and the
/// answer reports each.
///
/// ```
/// use teistro_hellenistic::{DignityRequest, DignityRules, SectRule, Terms};
///
/// let asked = DignityRequest::default()
///     .with_sect_rule(SectRule::Daylight)
///     .with_rules(DignityRules::LILLY.with_terms(Terms::Egyptian));
/// assert_eq!(asked.sect_rule(), SectRule::Daylight);
/// assert_eq!(asked.rules().terms, Terms::Egyptian);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct DignityRequest {
    sect_rule: SectRule,
    rules: DignityRules,
    scores: Scores,
}

impl Default for DignityRequest {
    fn default() -> DignityRequest {
        DignityRequest {
            sect_rule: SectRule::Horizon,
            rules: DignityRules::LILLY,
            scores: Scores::LILLY,
        }
    }
}

/// The record every binding writes the request as.
const DIGNITIES: &str = "dignities";

impl DignityRequest {
    /// The request as the bindings write it, the shape the answer reports
    /// it in: `{"sectRule": "DAYLIGHT", "rules": {"terms": "EGYPTIAN",
    /// "triplicities": "PTOLEMY"}, "scores": {"peregrine": 0}}`. Every
    /// member is optional and takes the default's; a table of the
    /// caller's own is `{"terms": {"TABLE": [[{"lord": "MARS", "end": 6},
    /// …], …]}}`, five terms a sign from Aries.
    ///
    /// ```
    /// use teistro_hellenistic::{DignityRequest, SectRule, Terms, Triplicities};
    ///
    /// let asked = DignityRequest::from_json(
    ///     r#"{"sectRule": "DAYLIGHT", "rules": {"terms": "EGYPTIAN"}, "scores": {"peregrine": 0}}"#,
    /// )?;
    /// assert_eq!(asked.sect_rule(), SectRule::Daylight);
    /// assert_eq!(asked.rules().terms, Terms::Egyptian);
    /// assert_eq!(asked.rules().triplicities, Triplicities::Lilly);
    /// assert_eq!((asked.scores().peregrine, asked.scores().house), (0, 5));
    /// assert_eq!(DignityRequest::from_json("{}")?, DignityRequest::default());
    ///
    /// let typo = DignityRequest::from_json(r#"{"scores": {"peregrin": 0}}"#).unwrap_err();
    /// assert_eq!(typo.field(), Some("dignities.scores.peregrin"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Text that is not the record, a member it does not know, or a value
    /// that is not one, such as a malformed table of terms, each named
    /// under `dignities`.
    pub fn from_json(text: &str) -> Result<DignityRequest, Error> {
        teistro_core::strict::read(text, DIGNITIES)
    }

    /// The same request, reading the sect by another rule.
    #[must_use]
    pub const fn with_sect_rule(mut self, sect_rule: SectRule) -> DignityRequest {
        self.sect_rule = sect_rule;
        self
    }

    /// The same request, under other terms and triplicities.
    #[must_use]
    pub const fn with_rules(mut self, rules: DignityRules) -> DignityRequest {
        self.rules = rules;
        self
    }

    /// The same request, scored otherwise.
    #[must_use]
    pub const fn with_scores(mut self, scores: Scores) -> DignityRequest {
        self.scores = scores;
        self
    }

    /// How the sect is read.
    #[must_use]
    pub const fn sect_rule(&self) -> SectRule {
        self.sect_rule
    }

    /// The terms and triplicities.
    #[must_use]
    pub const fn rules(&self) -> DignityRules {
        self.rules
    }

    /// The scores.
    #[must_use]
    pub const fn scores(&self) -> Scores {
        self.scores
    }

    /// The seven's dignities in a chart, in the Chaldean order.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_hellenistic::{ChartSky, DignityRequest, Sect};
    ///
    /// // The Sun at 5° Capricorn, high in the sky.
    /// let sky = ChartSky {
    ///     saturn_deg: 285.0, jupiter_deg: 100.0, mars_deg: 5.0, sun_deg: 275.0,
    ///     venus_deg: 35.0, mercury_deg: 260.0, moon_deg: 33.0,
    ///     sun_altitude_deg: 40.0, daylight: true,
    /// };
    /// let read = DignityRequest::default().read(&sky)?;
    /// assert_eq!(read.sect, Sect::Day);
    /// let mars = read.planets.iter().find(|at| at.planet == Graha::Mars).unwrap();
    /// assert!(mars.dignity.house && mars.dignity.face);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on a longitude or altitude that is not a finite
    /// number, naming its field.
    pub fn read(&self, sky: &ChartSky) -> Result<Dignities, Error> {
        if !sky.sun_altitude_deg.is_finite() {
            return Err(Error::invalid_arg(format!(
                "a solar altitude of {}",
                sky.sun_altitude_deg
            ))
            .with_field("sun_altitude_deg"));
        }
        let sect = self.sect_rule.sect(sky.sun_altitude_deg, sky.daylight);
        let mut planets = Vec::with_capacity(CHALDEAN_ORDER.len());
        for planet in CHALDEAN_ORDER {
            let Some((longitude_deg, field)) = sky.of(planet) else {
                continue;
            };
            let dignity = essential_dignity(planet, longitude_deg, sect, &self.rules)
                .map_err(|why| why.with_field(field))?;
            planets.push(PlanetDignity {
                planet,
                longitude_deg,
                dignity,
                score: dignity.score(&self.scores),
            });
        }
        let planets = planets
            .try_into()
            .map_err(|_| Error::internal("the seven are seven"))?;
        Ok(Dignities {
            sect,
            sect_rule: self.sect_rule,
            rules: self.rules,
            scores: self.scores,
            planets,
        })
    }
}

/// One planet's dignities and its score.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PlanetDignity {
    /// The planet.
    pub planet: Graha,
    /// Where it stands, in the chart's zodiac.
    pub longitude_deg: f64,
    /// What it holds there.
    pub dignity: EssentialDignity,
    /// Its score under the request's [`Scores`].
    pub score: i16,
}

/// The seven's dignities in one chart, with everything that made them.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Dignities {
    /// The sect the chart was read in.
    pub sect: Sect,
    /// The rule that chose it.
    pub sect_rule: SectRule,
    /// The terms and triplicities.
    pub rules: DignityRules,
    /// The scores.
    pub scores: Scores,
    /// The seven, in the Chaldean order, Saturn first.
    pub planets: [PlanetDignity; 7],
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own results"
    )]

    use super::{ChartSky, DignityRequest};
    use crate::{CHALDEAN_ORDER, Scores, Sect, SectRule, Terms};

    /// The Sun just under the western horizon: set by the horizon, still
    /// lit by the chart's own sunset.
    const DUSK: ChartSky = ChartSky {
        saturn_deg: 200.0,
        jupiter_deg: 100.0,
        mars_deg: 5.0,
        sun_deg: 179.5,
        venus_deg: 35.0,
        mercury_deg: 170.0,
        moon_deg: 33.0,
        sun_altitude_deg: -0.3,
        daylight: true,
    };

    #[test]
    fn the_answer_reports_the_sect_and_the_rule_that_chose_it() {
        let horizon = DignityRequest::default().read(&DUSK).unwrap();
        assert_eq!(
            (horizon.sect, horizon.sect_rule),
            (Sect::Night, SectRule::Horizon)
        );
        let daylight = DignityRequest::default()
            .with_sect_rule(SectRule::Daylight)
            .read(&DUSK)
            .unwrap();
        assert_eq!(
            (daylight.sect, daylight.sect_rule),
            (Sect::Day, SectRule::Daylight)
        );
        // Saturn in Libra rules the air triplicity by day only.
        assert!(!horizon.planets[0].dignity.triplicity);
        assert!(daylight.planets[0].dignity.triplicity);
    }

    #[test]
    fn the_seven_come_in_the_chaldean_order_with_their_scores() {
        let scores = Scores::new(1, 1, 1, 1, 1, 0, 0, 0);
        let read = DignityRequest::default()
            .with_scores(scores)
            .read(&DUSK)
            .unwrap();
        let order: Vec<_> = read.planets.iter().map(|at| at.planet).collect();
        assert_eq!(order, CHALDEAN_ORDER);
        for at in read.planets {
            assert_eq!(at.score, at.dignity.score(&scores), "{:?}", at.planet);
        }
        assert_eq!(read.scores, scores);
    }

    #[test]
    fn a_longitude_that_is_not_a_number_is_refused_by_its_field() {
        let sky = ChartSky {
            venus_deg: f64::NAN,
            ..DUSK
        };
        let why = DignityRequest::default().read(&sky).unwrap_err();
        assert_eq!(why.field(), Some("venus_deg"));
        let sky = ChartSky {
            sun_altitude_deg: f64::INFINITY,
            ..DUSK
        };
        let why = DignityRequest::default().read(&sky).unwrap_err();
        assert_eq!(why.field(), Some("sun_altitude_deg"));
    }

    /// The Egyptian table as a binding writes it, with the first term of
    /// Aries in the spelling every binding reads a graha back in.
    fn egyptian_as_written(first_lord: &str) -> String {
        let mut table = serde_json::to_value(crate::TermsTable::EGYPTIAN).unwrap();
        table[0][0]["lord"] = serde_json::Value::from(first_lord);
        serde_json::json!({"rules": {"terms": {"TABLE": table}}}).to_string()
    }

    #[test]
    fn a_table_of_the_callers_own_crosses_in_either_spelling() {
        for lord in ["JUPITER", "graha.JUPITER"] {
            let asked = DignityRequest::from_json(&egyptian_as_written(lord)).unwrap();
            assert_eq!(
                asked.rules().terms,
                Terms::Table(crate::TermsTable::EGYPTIAN),
                "{lord}"
            );
        }
        // Mars twice in Aries is no table, and the refusal says where.
        let why = DignityRequest::from_json(&egyptian_as_written("MARS")).unwrap_err();
        assert_eq!(why.field(), Some("dignities.rules.terms.TABLE"), "{why}");
    }

    #[test]
    fn the_request_round_trips_by_name() {
        let asked = DignityRequest::default().with_sect_rule(SectRule::Night);
        let text = serde_json::to_string(&asked).unwrap();
        assert!(text.contains("\"sectRule\":\"NIGHT\""), "{text}");
        assert_eq!(
            serde_json::from_str::<DignityRequest>(&text).unwrap(),
            asked
        );
    }
}
