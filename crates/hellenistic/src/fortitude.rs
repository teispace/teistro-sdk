//! Both halves of Lilly's table in one chart: the essential dignities and
//! the accidental fortitudes, under named rules
//! (`03-design/essential-dignities.md` §Accidental fortitudes).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

use crate::accidental::{
    AccidentalRules, AccidentalScores, AccidentalSky, PlanetAccidents, accidental_dignities,
};
use crate::almuten::{AlmutenRules, AlmutenSky, Almutens};
use crate::reading::{ChartSky, Dignities, DignityRequest};

/// How a chart's fortitudes are read: its essential dignities' request,
/// and the accidental table's rules and scores.
///
/// The default is Lilly's throughout; every part is a knob, and the
/// answer reports each.
///
/// ```
/// use teistro_hellenistic::{AccidentalRules, FortitudeRequest, FortuneRule, PlaceReading, SectRule};
///
/// let asked = FortitudeRequest::from_json(
///     r#"{"dignities": {"sectRule": "DAYLIGHT"}, "rules": {"beamsDeg": 15}}"#,
/// )?;
/// assert_eq!(asked.dignities().sect_rule(), SectRule::Daylight);
/// assert_eq!(asked.rules().beams_deg, 15.0);
/// assert_eq!(asked.rules().combustion_deg, AccidentalRules::LILLY.combustion_deg);
///
/// let sign = FortitudeRequest::from_json(r#"{"almuten": {"place": "SIGN"}}"#)?;
/// assert_eq!(sign.almuten().place, PlaceReading::Sign);
/// assert_eq!(sign.almuten().fortune, FortuneRule::DayAndNight);
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
    almuten: AlmutenRules,
}

/// The record every binding writes the request as.
const FORTITUDES: &str = "fortitudes";

impl FortitudeRequest {
    /// The request as the bindings write it: `{"dignities": {...},
    /// "rules": {...}, "scores": {...}, "almuten": {...}}`, every member
    /// optional and taking Lilly's. `dignities` is a [`DignityRequest`]'s record; a
    /// partile orb is `{"partile": {"WITHIN": {"orbDeg": 1}}}`.
    ///
    /// ```
    /// use teistro_hellenistic::FortitudeRequest;
    ///
    /// assert_eq!(FortitudeRequest::from_json("{}")?, FortitudeRequest::default());
    /// let unknown = FortitudeRequest::from_json(r#"{"dignity": {}}"#).unwrap_err();
    /// assert_eq!(unknown.field(), Some("fortitudes.dignity"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
    ///
    /// ```
    /// use teistro_hellenistic::{DignityRequest, FortitudeRequest, SectRule};
    ///
    /// let asked = FortitudeRequest::default().with_dignities(DignityRequest::default().with_sect_rule(SectRule::Daylight));
    /// assert_eq!(asked.dignities().sect_rule(), SectRule::Daylight);
    /// ```
    #[must_use]
    pub const fn with_dignities(mut self, dignities: DignityRequest) -> FortitudeRequest {
        self.dignities = dignities;
        self
    }

    /// The same request, under other orbs and limits.
    ///
    /// ```
    /// # use teistro_hellenistic::ChartSky;
    /// #
    /// # let chart = ChartSky {
    /// #     saturn_deg: 285.0, jupiter_deg: 245.0, mars_deg: 190.0, sun_deg: 15.0,
    /// #     venus_deg: 25.0, mercury_deg: 40.0, moon_deg: 100.0,
    /// #     sun_altitude_deg: 30.0, moon_altitude_deg: -20.0, daylight: true,
    /// # };
    /// # use teistro_core::catalogue::HouseSystem;
    /// # use teistro_hellenistic::AccidentalSky;
    /// #
    /// # let sky = AccidentalSky {
    /// #     speeds_deg_per_day: [0.03, 0.08, 0.5, 0.99, 1.2, 1.6, 13.0],
    /// #     houses: HouseSystem::Equal,
    /// #     cusps_deg: std::array::from_fn(|k| k as f64 * 30.0),
    /// #     ascendant_deg: 0.0,
    /// #     midheaven_deg: 270.0,
    /// #     north_node_deg: 250.0,
    /// #     regulus_deg: 150.0, spica_deg: 204.0, algol_deg: 56.0,
    /// # };
    /// use teistro_hellenistic::{AccidentalRules, FortitudeRequest};
    ///
    /// // A rule out of range is refused before the chart is read.
    /// let negative = FortitudeRequest::default().with_rules(AccidentalRules::LILLY.with_cusp_orb(-1.0));
    /// assert_eq!(negative.read(&chart, &sky).unwrap_err().field(), Some("rules.cuspOrbDeg"));
    /// ```
    #[must_use]
    pub const fn with_rules(mut self, rules: AccidentalRules) -> FortitudeRequest {
        self.rules = rules;
        self
    }

    /// The same request, the accidental lines scored otherwise.
    ///
    /// ```
    /// use teistro_hellenistic::{AccidentalScores, FortitudeRequest};
    ///
    /// let flat = AccidentalScores { houses: [0; 12], ..AccidentalScores::LILLY };
    /// assert_eq!(FortitudeRequest::default().with_scores(flat).scores().houses, [0; 12]);
    /// ```
    #[must_use]
    pub const fn with_scores(mut self, scores: AccidentalScores) -> FortitudeRequest {
        self.scores = scores;
        self
    }

    /// The same request, its almutens read otherwise.
    ///
    /// ```
    /// use teistro_hellenistic::{AlmutenRules, FortitudeRequest, PlaceReading};
    ///
    /// let mut by_sign = AlmutenRules::LILLY;
    /// by_sign.place = PlaceReading::Sign;
    /// let asked = FortitudeRequest::default().with_almuten(by_sign);
    /// assert_eq!(asked.almuten().place, PlaceReading::Sign);
    /// ```
    #[must_use]
    pub const fn with_almuten(mut self, almuten: AlmutenRules) -> FortitudeRequest {
        self.almuten = almuten;
        self
    }

    /// How the essential dignities are read.
    ///
    /// ```
    /// use teistro_hellenistic::{DignityRequest, FortitudeRequest};
    ///
    /// assert_eq!(FortitudeRequest::default().dignities(), DignityRequest::default());
    /// ```
    #[must_use]
    pub const fn dignities(&self) -> DignityRequest {
        self.dignities
    }

    /// The accidental orbs and limits.
    ///
    /// ```
    /// use teistro_hellenistic::{AccidentalRules, FortitudeRequest};
    ///
    /// assert_eq!(FortitudeRequest::default().rules(), AccidentalRules::LILLY);
    /// ```
    #[must_use]
    pub const fn rules(&self) -> AccidentalRules {
        self.rules
    }

    /// The accidental scores.
    ///
    /// ```
    /// use teistro_hellenistic::{AccidentalScores, FortitudeRequest};
    ///
    /// assert_eq!(FortitudeRequest::default().scores(), AccidentalScores::LILLY);
    /// ```
    #[must_use]
    pub const fn scores(&self) -> AccidentalScores {
        self.scores
    }

    /// How the almutens are read.
    ///
    /// ```
    /// use teistro_hellenistic::{AlmutenRules, FortitudeRequest};
    ///
    /// assert_eq!(FortitudeRequest::default().almuten(), AlmutenRules::default());
    /// ```
    #[must_use]
    pub const fn almuten(&self) -> AlmutenRules {
        self.almuten
    }

    /// Both halves of the table in a chart: the essential dignities from
    /// the chart's sky, and the accidental fortitudes from the same
    /// longitudes and the rest of it.
    ///
    /// ```
    /// # use teistro_hellenistic::ChartSky;
    /// #
    /// # let chart = ChartSky {
    /// #     saturn_deg: 285.0, jupiter_deg: 245.0, mars_deg: 190.0, sun_deg: 15.0,
    /// #     venus_deg: 25.0, mercury_deg: 40.0, moon_deg: 100.0,
    /// #     sun_altitude_deg: 30.0, moon_altitude_deg: -20.0, daylight: true,
    /// # };
    /// # use teistro_core::catalogue::HouseSystem;
    /// # use teistro_hellenistic::AccidentalSky;
    /// #
    /// # let sky = AccidentalSky {
    /// #     speeds_deg_per_day: [0.03, 0.08, 0.5, 0.99, 1.2, 1.6, 13.0],
    /// #     houses: HouseSystem::Equal,
    /// #     cusps_deg: std::array::from_fn(|k| k as f64 * 30.0),
    /// #     ascendant_deg: 0.0,
    /// #     midheaven_deg: 270.0,
    /// #     north_node_deg: 250.0,
    /// #     regulus_deg: 150.0, spica_deg: 204.0, algol_deg: 56.0,
    /// # };
    /// use teistro_core::catalogue::Graha;
    /// use teistro_hellenistic::FortitudeRequest;
    ///
    /// let read = FortitudeRequest::default().read(&chart, &sky)?;
    /// // Both halves, the seven in the Chaldean order.
    /// assert_eq!(read.dignities.planets[3].planet, Graha::Sun);
    /// assert_eq!(read.planets[3].planet, Graha::Sun);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
        let mut nets = [0; 7];
        for (net, (essential, accidental)) in
            nets.iter_mut().zip(dignities.planets.iter().zip(&planets))
        {
            *net =
                essential.score + essential.reception + accidental.fortitude - accidental.debility;
        }
        let almutens = self.almuten.read(
            nets,
            &AlmutenSky {
                ascendant_deg: sky.ascendant_deg,
                midheaven_deg: sky.midheaven_deg,
                sun_deg: chart.sun_deg,
                moon_deg: chart.moon_deg,
                moon_altitude_deg: chart.moon_altitude_deg,
                cusps_deg: &sky.cusps_deg,
                sect: dignities.sect,
                rules: &dignities.rules,
                scores: &dignities.scores,
            },
        )?;
        Ok(Fortitudes {
            dignities,
            sky: *sky,
            rules: self.rules,
            scores: self.scores,
            planets,
            almutens,
        })
    }
}

/// Both halves of Lilly's table in one chart, with everything that made
/// them.
///
/// ```
/// # use teistro_hellenistic::ChartSky;
/// #
/// # let chart = ChartSky {
/// #     saturn_deg: 285.0, jupiter_deg: 245.0, mars_deg: 190.0, sun_deg: 15.0,
/// #     venus_deg: 25.0, mercury_deg: 40.0, moon_deg: 100.0,
/// #     sun_altitude_deg: 30.0, moon_altitude_deg: -20.0, daylight: true,
/// # };
/// # use teistro_core::catalogue::HouseSystem;
/// # use teistro_hellenistic::AccidentalSky;
/// #
/// # let sky = AccidentalSky {
/// #     speeds_deg_per_day: [0.03, 0.08, 0.5, 0.99, 1.2, 1.6, 13.0],
/// #     houses: HouseSystem::Equal,
/// #     cusps_deg: std::array::from_fn(|k| k as f64 * 30.0),
/// #     ascendant_deg: 0.0,
/// #     midheaven_deg: 270.0,
/// #     north_node_deg: 250.0,
/// #     regulus_deg: 150.0, spica_deg: 204.0, algol_deg: 56.0,
/// # };
/// use teistro_hellenistic::{AccidentalRules, FortitudeRequest};
///
/// let read = FortitudeRequest::default().read(&chart, &sky)?;
/// assert_eq!((read.sky, read.rules), (sky, AccidentalRules::LILLY));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
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
    /// The almutens: of the figure (each planet's net), of the five
    /// places, and of each house.
    pub almutens: Almutens,
}

impl Fortitudes {
    /// A planet's strength over the whole table, as Lilly sums it (pp.
    /// 178–180): its essential score and mutual reception, and its
    /// accidental fortitudes less its debilities. `None` for a graha
    /// outside the seven.
    ///
    /// ```
    /// # use teistro_hellenistic::ChartSky;
    /// #
    /// # let chart = ChartSky {
    /// #     saturn_deg: 285.0, jupiter_deg: 245.0, mars_deg: 190.0, sun_deg: 15.0,
    /// #     venus_deg: 25.0, mercury_deg: 40.0, moon_deg: 100.0,
    /// #     sun_altitude_deg: 30.0, moon_altitude_deg: -20.0, daylight: true,
    /// # };
    /// # use teistro_core::catalogue::HouseSystem;
    /// # use teistro_hellenistic::AccidentalSky;
    /// #
    /// # let sky = AccidentalSky {
    /// #     speeds_deg_per_day: [0.03, 0.08, 0.5, 0.99, 1.2, 1.6, 13.0],
    /// #     houses: HouseSystem::Equal,
    /// #     cusps_deg: std::array::from_fn(|k| k as f64 * 30.0),
    /// #     ascendant_deg: 0.0,
    /// #     midheaven_deg: 270.0,
    /// #     north_node_deg: 250.0,
    /// #     regulus_deg: 150.0, spica_deg: 204.0, algol_deg: 56.0,
    /// # };
    /// use teistro_core::catalogue::Graha;
    /// use teistro_hellenistic::FortitudeRequest;
    ///
    /// let read = FortitudeRequest::default().read(&chart, &sky)?;
    /// // Mars: his essential score and reception, his fortitudes less his debilities.
    /// let (essential, accidental) = (read.dignities.planets[2], &read.planets[2]);
    /// let net = essential.score + essential.reception + accidental.fortitude - accidental.debility;
    /// assert_eq!(read.net(Graha::Mars), Some(net));
    /// assert_eq!(read.net(Graha::Rahu), None);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn net(&self, planet: Graha) -> Option<i16> {
        self.almutens.figure.total(planet)
    }
}
