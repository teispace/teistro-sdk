//! Synastry: the Western aspects between two charts
//! (`03-design/western-synastry.md`).
//!
//! Alan Leo reads two nativities by the aspects across them: the
//! luminaries interchanged (*How to Judge a Nativity*, p. 189), the
//! ascendants in good aspect (p. 221), and one planet "on the place of"
//! another (pp. 222–223). Every point of the first chart is read against
//! every point of the second, on the engine a chart's own table reads, so
//! the orbs are his single-chart orbs under whichever model is asked.
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_gochar::hits::NatalPoint;
//! use teistro_western::{SynastryPoint, SynastryRequest, WesternAspect, synastry};
//!
//! // His Sun at 10° and her Moon at 12°: the luminaries interchanged.
//! let his = [SynastryPoint::graha(Graha::Sun, 10.0)];
//! let hers = [SynastryPoint::graha(Graha::Moon, 12.0), SynastryPoint::lagna(85.0)];
//! let found = synastry(&his, &hers, &SynastryRequest::default())?;
//! assert_eq!(found.len(), 1);
//! assert_eq!(found[0].second, NatalPoint::Graha { graha: Graha::Moon });
//! assert_eq!(found[0].aspect, WesternAspect::Conjunction);
//! assert!((found[0].from_exact_deg - 2.0).abs() < 1e-9);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_aspect::orb::Moving;
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_gochar::hits::NatalPoint;

use crate::antiscia::AntisciaRequest;
use crate::aspects::{AspectRequest, OrbModel, Station, WesternAspect, holding, refuse_repeats};
use crate::declination::{ParallelRequest, paired, refuse_past_a_pole};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const ROOT: &str = "synastry";

/// Which zodiac two charts are compared in (C241).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum SynastryZodiac {
    /// Both charts' tropical longitudes, Leo's frame: the default, and the
    /// one in which two births years apart stand where their seasons put
    /// them.
    #[default]
    Tropical,
    /// Each chart's longitudes as founded, for a sidereal reader; two
    /// charts founded in different zodiacs are refused.
    Charts,
}

/// What a synastry is asked: the aspects and orbs a chart's own table
/// reads, whether each chart's lagna joins its planets, and the zodiac.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", from = "Asked")]
pub struct SynastryRequest {
    /// The aspects, and the orbs they are read under: Leo's nine under his
    /// orbs unless a caller says otherwise (C240).
    #[serde(flatten)]
    pub table: AspectRequest,
    /// Whether each chart's lagna is read beside its planets (Leo, p.
    /// 221); it stands as a planet in Leo's orbs (C242).
    pub lagna: bool,
    /// The zodiac the two are compared in (C241).
    pub zodiac: SynastryZodiac,
    /// The parallels across the two charts, when asked: each point of one
    /// the same distance from the equator as a point of the other
    /// (`03-design/western-declinations.md`). None by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallels: Option<ParallelRequest>,
    /// The antiscia across the two charts, when asked: each planet of one
    /// whose reflection falls on a planet of the other
    /// (`03-design/western-antiscia.md`). None by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub antiscia: Option<AntisciaRequest>,
}

/// [`SynastryRequest`] as it is read: the aspect table's fields laid
/// flat, without `flatten`, which buffers what it reads and so names a
/// refusal inside by the record alone, not by the field that failed.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct Asked {
    aspects: Vec<WesternAspect>,
    orbs: OrbModel,
    lagna: bool,
    zodiac: SynastryZodiac,
    parallels: Option<ParallelRequest>,
    antiscia: Option<AntisciaRequest>,
}

impl Default for Asked {
    fn default() -> Asked {
        let SynastryRequest {
            table: AspectRequest { aspects, orbs },
            lagna,
            zodiac,
            parallels,
            antiscia,
        } = SynastryRequest::default();
        Asked {
            aspects,
            orbs,
            lagna,
            zodiac,
            parallels,
            antiscia,
        }
    }
}

impl From<Asked> for SynastryRequest {
    fn from(asked: Asked) -> SynastryRequest {
        SynastryRequest {
            table: AspectRequest {
                aspects: asked.aspects,
                orbs: asked.orbs,
            },
            lagna: asked.lagna,
            zodiac: asked.zodiac,
            parallels: asked.parallels,
            antiscia: asked.antiscia,
        }
    }
}

impl Default for SynastryRequest {
    fn default() -> SynastryRequest {
        SynastryRequest {
            table: AspectRequest::default(),
            lagna: true,
            zodiac: SynastryZodiac::Tropical,
            parallels: None,
            antiscia: None,
        }
    }
}

impl SynastryRequest {
    /// Lilly's reading: the Ptolemaic five under his moieties, which give
    /// the lagna no orb, so it is left out.
    #[must_use]
    pub fn lilly() -> SynastryRequest {
        SynastryRequest {
            table: AspectRequest::lilly(),
            lagna: false,
            zodiac: SynastryZodiac::Tropical,
            parallels: None,
            antiscia: None,
        }
    }

    /// Reads the parallels across the two charts too, under this request.
    #[must_use]
    pub const fn with_parallels(mut self, parallels: ParallelRequest) -> Self {
        self.parallels = Some(parallels);
        self
    }

    /// Reads the antiscia across the two charts too, under this request.
    #[must_use]
    pub fn with_antiscia(mut self, antiscia: AntisciaRequest) -> Self {
        self.antiscia = Some(antiscia);
        self
    }

    /// Looks for these aspects only.
    #[must_use]
    pub fn with_aspects(mut self, aspects: impl IntoIterator<Item = WesternAspect>) -> Self {
        self.table = self.table.with_aspects(aspects);
        self
    }

    /// Reads them under this model of orbs.
    #[must_use]
    pub fn with_orbs(mut self, orbs: OrbModel) -> Self {
        self.table = self.table.with_orbs(orbs);
        self
    }

    /// Reads each chart's lagna beside its planets, or not.
    #[must_use]
    pub const fn with_lagna(mut self, lagna: bool) -> Self {
        self.lagna = lagna;
        self
    }

    /// Compares the two in this zodiac.
    #[must_use]
    pub const fn with_zodiac(mut self, zodiac: SynastryZodiac) -> Self {
        self.zodiac = zodiac;
        self
    }

    /// The request a binding sends, as JSON: the aspect table's `aspects`
    /// and `orbs` ([`AspectRequest::from_json`]), `lagna` (true when left
    /// out) and `zodiac`, `"TROPICAL"` (the default) or `"CHARTS"`.
    ///
    /// ```
    /// use teistro_western::{SynastryRequest, SynastryZodiac, WesternAspect};
    ///
    /// let asked = SynastryRequest::from_json(r#"{"aspects": ["TRINE"], "zodiac": "CHARTS"}"#)?;
    /// assert_eq!(asked.table.aspects, [WesternAspect::Trine]);
    /// assert!(asked.lagna);
    /// assert_eq!(asked.zodiac, SynastryZodiac::Charts);
    /// // A key it does not read is refused by name, under the record's root.
    /// let typo = SynastryRequest::from_json(r#"{"lagnas": false}"#).unwrap_err();
    /// assert_eq!(typo.field(), Some("synastry.lagnas"));
    /// // And a value it cannot read, by the field that holds it.
    /// let zodiac = SynastryRequest::from_json(r#"{"zodiac": "SIDEREAL"}"#).unwrap_err();
    /// assert_eq!(zodiac.field(), Some("synastry.zodiac"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, and whatever [`SynastryRequest::check`] refuses, each named
    /// under `synastry`.
    pub fn from_json(text: &str) -> Result<SynastryRequest, Error> {
        let asked: SynastryRequest = teistro_core::strict::read(text, ROOT)?;
        asked.check().map_err(|why| why.under(ROOT))?;
        Ok(asked)
    }

    /// Refuses a request no synastry could answer, by field: what
    /// [`AspectRequest::check`] refuses, and a lagna asked under moieties,
    /// which give it no orb (C242).
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the field.
    pub fn check(&self) -> Result<(), Error> {
        self.table.check()?;
        if self.lagna && matches!(self.table.orbs, OrbModel::Moieties { .. }) {
            return Err(Error::invalid_arg("the moieties give the lagna no orb")
                .with_field("lagna")
                .with_hint("ask with `lagna: false`, or read the lagna under Leo's orbs"));
        }
        if let Some(parallels) = &self.parallels {
            parallels.check().map_err(|why| why.under("parallels"))?;
        }
        if let Some(antiscia) = &self.antiscia {
            antiscia.check().map_err(|why| why.under("antiscia"))?;
        }
        Ok(())
    }
}

/// A point of one chart as a synastry reads it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynastryPoint {
    /// Which: a planet, or the lagna.
    pub point: NatalPoint,
    /// Its longitude, degrees, in the zodiac both charts are read in.
    pub longitude_deg: f64,
}

impl SynastryPoint {
    /// A planet at a longitude.
    #[must_use]
    pub const fn graha(graha: Graha, longitude_deg: f64) -> SynastryPoint {
        SynastryPoint {
            point: NatalPoint::Graha { graha },
            longitude_deg,
        }
    }

    /// The lagna at a longitude.
    #[must_use]
    pub const fn lagna(longitude_deg: f64) -> SynastryPoint {
        SynastryPoint {
            point: NatalPoint::Lagna,
            longitude_deg,
        }
    }

    /// Its key, for a refusal: `SUN`, or `LAGNA`.
    const fn key(self) -> &'static str {
        point_key(self.point)
    }
}

/// One aspect between a point of the first chart and a point of the
/// second. Two births do not move against each other, so it has no
/// applying.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynastryRow {
    /// The point of the first chart.
    pub first: NatalPoint,
    /// The point of the second.
    pub second: NatalPoint,
    /// Which aspect.
    pub aspect: WesternAspect,
    /// How far apart the two stand, degrees, 0 to 180.
    pub apart_deg: f64,
    /// How far from exact, degrees; the smaller, the stronger (p. 47).
    pub from_exact_deg: f64,
    /// The orb the model allowed this pair at this aspect, degrees.
    pub orb_deg: f64,
}

/// Every aspect between a point of `first` and a point of `second`,
/// closest first.
///
/// The request's `lagna` and `zodiac` say which points a caller gathers
/// and in which zodiac; this reads the points it is given.
///
/// # Errors
///
/// What [`AspectRequest::check`] refuses; a point given twice on one side;
/// a point the model has no orb for; a longitude that is not finite.
pub fn synastry(
    first: &[SynastryPoint],
    second: &[SynastryPoint],
    request: &SynastryRequest,
) -> Result<Vec<SynastryRow>, Error> {
    request.table.check()?;
    for (side, points) in [("first", first), ("second", second)] {
        refuse_repeats(points.iter().map(|one| one.key()), "a point")
            .map_err(|why| why.with_field(side))?;
    }
    let station = |one: &SynastryPoint| Station {
        label: one.point,
        point: one.point,
        moving: Moving::new(one.longitude_deg, 0.0),
    };
    let pairs = first
        .iter()
        .flat_map(|a| second.iter().map(move |b| (station(a), station(b))));
    Ok(holding(pairs, &request.table)?
        .into_iter()
        .map(|held| SynastryRow {
            first: held.first,
            second: held.second,
            aspect: held.aspect,
            apart_deg: held.apart_deg,
            from_exact_deg: held.from_exact_deg,
            orb_deg: held.orb_deg,
        })
        .collect())
}

/// A point of one chart as a synastry's parallels read it: its distance
/// from the equator.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeclinedPoint {
    /// Which: a planet, or the lagna.
    pub point: NatalPoint,
    /// Its declination, degrees, north positive.
    pub declination_deg: f64,
}

impl DeclinedPoint {
    /// A planet at a declination.
    #[must_use]
    pub const fn graha(graha: Graha, declination_deg: f64) -> DeclinedPoint {
        DeclinedPoint {
            point: NatalPoint::Graha { graha },
            declination_deg,
        }
    }

    /// The lagna at a declination.
    #[must_use]
    pub const fn lagna(declination_deg: f64) -> DeclinedPoint {
        DeclinedPoint {
            point: NatalPoint::Lagna,
            declination_deg,
        }
    }
}

/// A point of one chart and a point of the other the same distance from
/// the equator, within the orb.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynastryParallelRow {
    /// The first chart's point.
    pub first: NatalPoint,
    /// The second chart's.
    pub second: NatalPoint,
    /// Whether the two stand on opposite sides of the equator (C243).
    pub contrary: bool,
    /// How far apart their distances from the equator are, degrees.
    pub apart_deg: f64,
    /// The orb the request allowed, degrees.
    pub orb_deg: f64,
}

/// Every point of `first` the same distance from the equator as a point of
/// `second`, within the orb, closest first: the parallels across two
/// charts (`03-design/western-declinations.md`).
///
/// ```
/// use teistro_gochar::hits::NatalPoint;
/// use teistro_western::{DeclinedPoint, ParallelRequest, synastry_parallels};
/// use teistro_core::catalogue::Graha;
///
/// let his = [DeclinedPoint::graha(Graha::Sun, 22.3)];
/// let hers = [DeclinedPoint::graha(Graha::Moon, -21.8), DeclinedPoint::lagna(5.0)];
/// let found = synastry_parallels(&his, &hers, &ParallelRequest::default())?;
/// assert_eq!(found.len(), 1);
/// assert_eq!(found[0].second, NatalPoint::Graha { graha: Graha::Moon });
/// assert!(found[0].contrary);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// What [`ParallelRequest::check`] refuses; a point given twice on one
/// side (`first` or `second`); a declination past a pole.
pub fn synastry_parallels(
    first: &[DeclinedPoint],
    second: &[DeclinedPoint],
    request: &ParallelRequest,
) -> Result<Vec<SynastryParallelRow>, Error> {
    request.check()?;
    for (side, points) in [("first", first), ("second", second)] {
        refuse_repeats(points.iter().map(|one| point_key(one.point)), "a point")
            .map_err(|why| why.with_field(side))?;
        refuse_past_a_pole(points.iter().map(|one| one.declination_deg), |at| {
            format!("{side}[{at}].declinationDeg")
        })?;
    }
    let pairs = first.iter().flat_map(|a| {
        second
            .iter()
            .map(move |b| ((a.point, a.declination_deg), (b.point, b.declination_deg)))
    });
    Ok(paired(pairs, request.orb_deg)
        .into_iter()
        .map(|pair| SynastryParallelRow {
            first: pair.first,
            second: pair.second,
            contrary: pair.contrary,
            apart_deg: pair.apart_deg,
            orb_deg: request.orb_deg,
        })
        .collect())
}

/// A natal point's key, for a refusal: `SUN`, or `LAGNA`.
const fn point_key(point: NatalPoint) -> &'static str {
    match point {
        NatalPoint::Graha { graha } => graha.key(),
        NatalPoint::Lagna => "LAGNA",
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;
    use crate::aspects::BodyOrb;

    #[test]
    fn every_point_of_one_is_read_against_every_point_of_the_other() {
        // Both Suns at 0°, both Moons at 90°: the Sun–Sun and Moon–Moon
        // conjunctions, and the two Sun–Moon squares, each its own row.
        let his = [
            SynastryPoint::graha(Graha::Sun, 0.0),
            SynastryPoint::graha(Graha::Moon, 90.0),
        ];
        let hers = his;
        let found = synastry(&his, &hers, &SynastryRequest::default()).unwrap();
        let pairs: Vec<(NatalPoint, NatalPoint, WesternAspect)> = found
            .iter()
            .map(|row| (row.first, row.second, row.aspect))
            .collect();
        let sun = NatalPoint::Graha { graha: Graha::Sun };
        let moon = NatalPoint::Graha { graha: Graha::Moon };
        assert_eq!(pairs.len(), 4, "{pairs:?}");
        for pair in [
            (sun, sun, WesternAspect::Conjunction),
            (moon, moon, WesternAspect::Conjunction),
            (sun, moon, WesternAspect::Square),
            (moon, sun, WesternAspect::Square),
        ] {
            assert!(pairs.contains(&pair), "{pair:?} in {pairs:?}");
        }
    }

    #[test]
    fn the_lagna_takes_a_planets_orb_and_none_among_the_moieties() {
        // Nine degrees from the other's Sun the lagna holds a conjunction,
        // since a luminary and a planet take 10°; nine degrees from the
        // other's lagna it does not, since two planets take 8° and the
        // lagna is no luminary (C242).
        let his = [SynastryPoint::lagna(0.0)];
        let leo = SynastryRequest::default();
        let by_sun = synastry(&his, &[SynastryPoint::graha(Graha::Sun, 9.0)], &leo).unwrap();
        assert_eq!(by_sun.len(), 1);
        assert!((by_sun[0].orb_deg - 10.0).abs() < 1e-12);
        let by_lagna = synastry(&his, &[SynastryPoint::lagna(9.0)], &leo).unwrap();
        assert!(by_lagna.is_empty(), "{by_lagna:?}");
        let near = [SynastryPoint::graha(Graha::Sun, 7.5)];

        let refused = SynastryRequest::lilly()
            .with_lagna(true)
            .check()
            .unwrap_err();
        assert_eq!(refused.field(), Some("lagna"));
        let moieties = OrbModel::Moieties {
            orbs: vec![BodyOrb {
                graha: Graha::Sun,
                orb_deg: 17.0,
            }],
        };
        let refused = synastry(
            &his,
            &near,
            &SynastryRequest::default()
                .with_lagna(false)
                .with_orbs(moieties),
        )
        .unwrap_err();
        assert!(refused.to_string().contains("lagna"), "{refused}");
    }

    #[test]
    fn a_point_given_twice_on_one_side_is_refused_by_side() {
        let twice = [
            SynastryPoint::graha(Graha::Sun, 0.0),
            SynastryPoint::graha(Graha::Sun, 1.0),
        ];
        let once = [SynastryPoint::graha(Graha::Moon, 0.0)];
        let request = SynastryRequest::default();
        let field = |a: &[SynastryPoint], b: &[SynastryPoint]| {
            synastry(a, b, &request)
                .unwrap_err()
                .field()
                .map(str::to_owned)
        };
        assert_eq!(field(&twice, &once).as_deref(), Some("first"));
        assert_eq!(field(&once, &twice).as_deref(), Some("second"));
    }

    #[test]
    fn the_record_reads_the_table_flat_and_refuses_by_name() {
        let asked = SynastryRequest::from_json(
            r#"{"aspects": ["SQUARE"], "orbs": {"model": "BY_ASPECT", "orbs": [{"aspect": "SQUARE", "orbDeg": 3}]}, "lagna": false}"#,
        )
        .unwrap();
        assert_eq!(asked.table.aspects, [WesternAspect::Square]);
        assert!(!asked.lagna);
        assert_eq!(asked.zodiac, SynastryZodiac::Tropical);
        let field = |text: &str| {
            SynastryRequest::from_json(text)
                .unwrap_err()
                .field()
                .map(str::to_owned)
        };
        assert_eq!(
            field(r#"{"aspects": []}"#).as_deref(),
            Some("synastry.aspects")
        );
        assert_eq!(
            field(r#"{"zodiac": "SIDEREAL"}"#).as_deref(),
            Some("synastry.zodiac")
        );
        assert_eq!(
            field(r#"{"orbs": {"model": "MOIETIES", "orbs": [{"graha": "SUN", "orbDeg": 17}]}}"#)
                .as_deref(),
            Some("synastry.lagna")
        );
        // The record round-trips as it was read.
        let again: SynastryRequest =
            teistro_core::strict::read(&serde_json::to_string(&asked).unwrap(), ROOT).unwrap();
        assert_eq!(again, asked);
    }

    #[test]
    fn the_parallels_are_asked_in_the_record_and_named_under_it() {
        let asked = SynastryRequest::from_json(r#"{"parallels": {}}"#).unwrap();
        assert_eq!(asked.parallels, Some(ParallelRequest::default()));
        assert_eq!(SynastryRequest::from_json("{}").unwrap().parallels, None);
        let wide = SynastryRequest::from_json(r#"{"parallels": {"orbDeg": 20}}"#).unwrap_err();
        assert_eq!(wide.field(), Some("synastry.parallels.orbDeg"));
        let typo = SynastryRequest::from_json(r#"{"parallels": {"orb": 1}}"#).unwrap_err();
        assert_eq!(typo.field(), Some("synastry.parallels.orb"));
    }

    #[test]
    fn the_parallels_across_read_every_point_against_every_point() {
        let his = [
            DeclinedPoint::graha(Graha::Sun, 10.0),
            DeclinedPoint::lagna(-4.0),
        ];
        let hers = [
            DeclinedPoint::graha(Graha::Sun, -10.5),
            DeclinedPoint::graha(Graha::Moon, 4.2),
        ];
        let found = synastry_parallels(&his, &hers, &ParallelRequest::default()).unwrap();
        let pairs: Vec<(NatalPoint, NatalPoint, bool)> = found
            .iter()
            .map(|row| (row.first, row.second, row.contrary))
            .collect();
        assert_eq!(
            pairs,
            [
                (
                    NatalPoint::Lagna,
                    NatalPoint::Graha { graha: Graha::Moon },
                    true
                ),
                (
                    NatalPoint::Graha { graha: Graha::Sun },
                    NatalPoint::Graha { graha: Graha::Sun },
                    true
                ),
            ]
        );
        let twice = [DeclinedPoint::lagna(1.0), DeclinedPoint::lagna(2.0)];
        let refused = synastry_parallels(&his, &twice, &ParallelRequest::default()).unwrap_err();
        assert_eq!(refused.field(), Some("second"));
        let past = [DeclinedPoint::lagna(95.0)];
        let refused = synastry_parallels(&past, &hers, &ParallelRequest::default()).unwrap_err();
        assert_eq!(refused.field(), Some("first[0].declinationDeg"));
    }
}
