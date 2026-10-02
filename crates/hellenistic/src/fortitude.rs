//! Both halves of Lilly's table in one chart: the essential dignities and
//! the accidental fortitudes, under named rules
//! (`03-design/essential-dignities.md` §Accidental fortitudes).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

use crate::accidental::{
    AccidentalRules, AccidentalScores, AccidentalSky, PlanetAccidents, accidental_dignities,
};
use crate::reading::{ChartSky, Dignities, DignityRequest};

/// How a chart's fortitudes are read: its essential dignities' request,
/// and the accidental table's rules and scores.
///
/// The default is Lilly's throughout; every part is a knob, and the
/// answer reports each.
///
/// ```
/// use teistro_hellenistic::{AccidentalRules, FortitudeRequest, SectRule};
///
/// let asked = FortitudeRequest::from_json(
///     r#"{"dignities": {"sectRule": "DAYLIGHT"}, "rules": {"beamsDeg": 15}}"#,
/// )?;
/// assert_eq!(asked.dignities().sect_rule(), SectRule::Daylight);
/// assert_eq!(asked.rules().beams_deg, 15.0);
/// assert_eq!(asked.rules().combustion_deg, AccidentalRules::LILLY.combustion_deg);
///
/// let typo = FortitudeRequest::from_json(r#"{"rules": {"beamDeg": 15}}"#).unwrap_err();
/// assert_eq!(typo.field(), Some("fortitudes.rules.beamDeg"));
///
/// let negative = FortitudeRequest::from_json(r#"{"rules": {"beamsDeg": -1}}"#).unwrap_err();
/// assert_eq!(negative.field(), Some("fortitudes.rules.beamsDeg"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct FortitudeRequest {
    dignities: DignityRequest,
    rules: AccidentalRules,
    scores: AccidentalScores,
}

/// The record every binding writes the request as.
const FORTITUDES: &str = "fortitudes";

impl FortitudeRequest {
    /// The request as the bindings write it: `{"dignities": {...},
    /// "rules": {...}, "scores": {...}}`, every member optional and
    /// taking Lilly's. `dignities` is a [`DignityRequest`]'s record; a
    /// partile orb is `{"partile": {"WITHIN": {"orbDeg": 1}}}`.
    ///
    /// # Errors
    ///
    /// Text that is not the record, a member it does not know, a value
    /// that is not one, or an orb or mean motion out of range, each named
    /// under `fortitudes`, so a bad request is refused before a chart is
    /// read.
    pub fn from_json(text: &str) -> Result<FortitudeRequest, Error> {
        let request: FortitudeRequest = teistro_core::strict::read(text, FORTITUDES)?;
        request
            .rules
            .check()
            .map_err(|why| why.under(&format!("{FORTITUDES}.rules")))?;
        Ok(request)
    }

    /// The same request, its essential dignities read otherwise.
    #[must_use]
    pub const fn with_dignities(mut self, dignities: DignityRequest) -> FortitudeRequest {
        self.dignities = dignities;
        self
    }

    /// The same request, under other orbs and limits.
    #[must_use]
    pub const fn with_rules(mut self, rules: AccidentalRules) -> FortitudeRequest {
        self.rules = rules;
        self
    }

    /// The same request, the accidental lines scored otherwise.
    #[must_use]
    pub const fn with_scores(mut self, scores: AccidentalScores) -> FortitudeRequest {
        self.scores = scores;
        self
    }

    /// How the essential dignities are read.
    #[must_use]
    pub const fn dignities(&self) -> DignityRequest {
        self.dignities
    }

    /// The accidental orbs and limits.
    #[must_use]
    pub const fn rules(&self) -> AccidentalRules {
        self.rules
    }

    /// The accidental scores.
    #[must_use]
    pub const fn scores(&self) -> AccidentalScores {
        self.scores
    }

    /// Both halves of the table in a chart: the essential dignities from
    /// the chart's sky, and the accidental fortitudes from the same
    /// longitudes and the rest of it.
    ///
    /// # Errors
    ///
    /// As [`DignityRequest::read`] and
    /// [`accidental_dignities`](crate::accidental_dignities); a rule out of
    /// range is named under `rules`, as `rules.beamsDeg`.
    pub fn read(&self, chart: &ChartSky, sky: &AccidentalSky) -> Result<Fortitudes, Error> {
        self.rules.check().map_err(|why| why.under("rules"))?;
        let dignities = self.dignities.read(chart)?;
        let planets = accidental_dignities(&chart.longitudes(), sky, &self.rules, &self.scores)?;
        Ok(Fortitudes {
            dignities,
            sky: *sky,
            rules: self.rules,
            scores: self.scores,
            planets,
        })
    }
}

/// Both halves of Lilly's table in one chart, with everything that made
/// them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Fortitudes {
    /// The essential dignities, with their sect, rules and scores.
    pub dignities: Dignities,
    /// What the accidental fortitudes were read from: the motions, the
    /// division and its cusps, the North Node and the stars' places.
    pub sky: AccidentalSky,
    /// The accidental orbs and limits.
    pub rules: AccidentalRules,
    /// The accidental scores.
    pub scores: AccidentalScores,
    /// The seven's accidental fortitudes, in the Chaldean order.
    pub planets: [PlanetAccidents; 7],
}

impl Fortitudes {
    /// A planet's strength over the whole table, as Lilly sums it (pp.
    /// 178–180): its essential score and mutual reception, and its
    /// accidental fortitudes less its debilities. `None` for a graha
    /// outside the seven.
    #[must_use]
    pub fn net(&self, planet: Graha) -> Option<i16> {
        let essential = self
            .dignities
            .planets
            .iter()
            .find(|at| at.planet == planet)?;
        let accidental = self.planets.iter().find(|at| at.planet == planet)?;
        Some(essential.score + essential.reception + accidental.fortitude - accidental.debility)
    }
}
