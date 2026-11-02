//! How far a birth time can move before what its chart states changes
//! (`03-design/rectification.md`, "Birth-time sensitivity", A2-5).
//!
//! For one birth, every fact a chart states from its clock (the lagna's
//! sign and pada, each varga's lagna, the Moon's pada, the dasha running at
//! a reference instant and the chalit's bhava middles) with its nearest
//! change either side: the instant, the minutes from the birth, and what
//! the fact becomes. A report: nothing here bars, and no text is read.
//!
//! The search is the purifier's (`edges.rs`): one window around the birth
//! sampled on the seed grid, every pair of samples that differ bisected
//! until the change is pinned to [`EDGE_TOLERANCE_DAYS`]. The value sampled
//! is the birth's whole set of facts, so each fact's nearest change is read
//! off the runs between the edges.
//!
//! ```
//! # use teistro_core::catalogue::Vara;
//! # use teistro_core::quantity::{JulianDay, Utc};
//! # use teistro_rectification::{Day, Sky};
//! # struct Line;
//! # impl Sky for Line {
//! #     fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, teistro_core::Error> {
//! #         Ok(((at.get() - 2_460_000.25) * 360.0).rem_euclid(360.0))
//! #     }
//! #     fn sun_deg(&self, _: JulianDay<Utc>) -> Result<f64, teistro_core::Error> { Ok(10.0) }
//! #     fn moon_deg(&self, _: JulianDay<Utc>) -> Result<f64, teistro_core::Error> { Ok(100.0) }
//! #     fn day(&self, _: JulianDay<Utc>) -> Result<Day, teistro_core::Error> {
//! #         Ok(Day::literal(2_460_000.25, 2_460_000.75, 2_460_001.25, Vara::Budhavara))
//! #     }
//! # }
//! use teistro_core::catalogue::Rashi;
//! use teistro_rectification::Window;
//! use teistro_rectification::sensitivity::{SensitivityRules, SensitivitySky, sensitivity};
//!
//! impl SensitivitySky for Line {}
//!
//! // A lagna turning a degree every four minutes, 12° into Aries at the birth.
//! let birth = JulianDay::literal(2_460_000.25 + 12.0 / 360.0);
//! let window = Window::between(JulianDay::literal(birth.get() - 0.1), JulianDay::literal(birth.get() + 0.1))?;
//! let found = sensitivity(birth, window, &Line, &SensitivityRules::default())?;
//! assert_eq!(found.lagna_sign.value, Rashi::Aries);
//! // Eighteen degrees to Taurus: seventy-two minutes later.
//! let later = found.lagna_sign.later.as_ref().ok_or("a change")?;
//! assert!((later.minutes - 72.0).abs() < 1e-3);
//! assert_eq!(later.to, Rashi::Taurus);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::cell::RefCell;

use serde::Serialize;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{DashaSystem, Graha, Nakshatra, Rashi, Varga};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{Depth, JulianDay, Utc};
use teistro_core::settings::Balance;
use teistro_dasha::{Birth, Dasha, Timeline, UduRow};
use teistro_vargas::{Scheme, sign};

use crate::edges::{self, Piece};
use crate::{DEFAULT_SEED_MINUTES, EDGE_TOLERANCE_DAYS, FINEST_SEED_MINUTES, Grid, Sky, Window};

/// What the sensitivity asks of an ephemeris beyond the purifier's
/// [`Sky`]: the chart's bhava middles at an instant.
///
/// ```
/// # use teistro_core::catalogue::Vara;
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_rectification::{Day, Sky};
/// # struct Line;
/// # impl Sky for Line {
/// #     fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, teistro_core::Error> {
/// #         Ok(((at.get() - 2_460_000.25) * 360.0).rem_euclid(360.0))
/// #     }
/// #     fn sun_deg(&self, _: JulianDay<Utc>) -> Result<f64, teistro_core::Error> { Ok(10.0) }
/// #     fn moon_deg(&self, _: JulianDay<Utc>) -> Result<f64, teistro_core::Error> { Ok(100.0) }
/// #     fn day(&self, _: JulianDay<Utc>) -> Result<Day, teistro_core::Error> {
/// #         Ok(Day::literal(2_460_000.25, 2_460_000.75, 2_460_001.25, Vara::Budhavara))
/// #     }
/// # }
/// use teistro_rectification::sensitivity::SensitivitySky;
///
/// impl SensitivitySky for Line {}
///
/// // A sky that reads no chalit answers none, and the answer has no cusps.
/// assert_eq!(Line.middles_deg(JulianDay::literal(2_460_000.3))?, None);
/// # Ok::<(), teistro_core::Error>(())
/// ```
pub trait SensitivitySky: Sky {
    /// The twelve bhava middles of the chart's chalit at an instant, in the
    /// chart's zodiac, degrees; `None` where they are not worth reading
    /// because their signs only count on from the lagna's (a whole-sign
    /// chalit, C1141). `None` by default.
    ///
    /// ```
    /// # use teistro_core::catalogue::Vara;
    /// # use teistro_core::quantity::{JulianDay, Utc};
    /// # use teistro_rectification::{Day, Sky};
    /// # struct Line;
    /// # impl Sky for Line {
    /// #     fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, teistro_core::Error> {
    /// #         Ok(((at.get() - 2_460_000.25) * 360.0).rem_euclid(360.0))
    /// #     }
    /// #     fn sun_deg(&self, _: JulianDay<Utc>) -> Result<f64, teistro_core::Error> { Ok(10.0) }
    /// #     fn moon_deg(&self, _: JulianDay<Utc>) -> Result<f64, teistro_core::Error> { Ok(100.0) }
    /// #     fn day(&self, _: JulianDay<Utc>) -> Result<Day, teistro_core::Error> {
    /// #         Ok(Day::literal(2_460_000.25, 2_460_000.75, 2_460_001.25, Vara::Budhavara))
    /// #     }
    /// # }
    /// use teistro_rectification::sensitivity::SensitivitySky;
    ///
    /// impl SensitivitySky for Line {}
    /// assert!(Line.middles_deg(JulianDay::literal(2_460_000.3))?.is_none());
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Whatever the implementation cannot answer, unchanged.
    fn middles_deg(&self, at: JulianDay<Utc>) -> Result<Option<[f64; 12]>, Error> {
        let _ = at;
        Ok(None)
    }
}

/// The dasha whose running periods the sensitivity follows: the periods
/// running at `at`, to `depth`, of a nakshatra-seeded `system` under
/// `rules` (C1142).
///
/// ```
/// use teistro_core::catalogue::DashaSystem;
/// use teistro_core::quantity::{Depth, JulianDay};
/// use teistro_core::settings::root;
/// use teistro_rectification::sensitivity::DashaAt;
///
/// let asked = DashaAt {
///     at: JulianDay::literal(2_470_000.5),
///     system: DashaSystem::Vimshottari,
///     depth: Depth::try_new(3)?,
///     rules: teistro_dasha::Rules::of(&root().dasha, DashaSystem::Vimshottari),
/// };
/// assert_eq!(asked.depth.get(), 3);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DashaAt {
    /// The reference instant the running periods are read at.
    pub at: JulianDay<Utc>,
    /// The system: a nakshatra-seeded one (Vimshottari, Ashtottari, Yogini
    /// and the rest of the rows).
    pub system: DashaSystem,
    /// How many levels: 1 the mahadasha, 3 the pratyantardasha.
    pub depth: Depth,
    /// The rules it is computed under.
    pub rules: teistro_dasha::Rules,
}

/// What the sensitivity reads, and how finely it searches.
///
/// ```
/// use teistro_core::catalogue::Varga;
/// use teistro_rectification::DEFAULT_SEED_MINUTES;
/// use teistro_rectification::sensitivity::SensitivityRules;
///
/// let rules = SensitivityRules::default();
/// assert_eq!(rules.vargas, [Varga::D9, Varga::D10]);
/// assert_eq!(rules.seed_minutes, DEFAULT_SEED_MINUTES);
/// assert!(rules.dasha.is_none());
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SensitivityRules {
    /// The seed step, minutes, from a second to an hour: the purifier's
    /// `seedMinutes`. A fact that changes and changes back inside one step
    /// is not seen.
    pub seed_minutes: f64,
    /// The vargas whose lagnas are followed, in the order answered (C1140).
    pub vargas: Vec<Varga>,
    /// The dasha followed, or none.
    pub dasha: Option<DashaAt>,
}

impl Default for SensitivityRules {
    fn default() -> SensitivityRules {
        SensitivityRules {
            seed_minutes: DEFAULT_SEED_MINUTES,
            vargas: vec![Varga::D9, Varga::D10],
            dasha: None,
        }
    }
}

impl SensitivityRules {
    /// Refuses rules no search can run under.
    ///
    /// ```
    /// use teistro_core::catalogue::DashaSystem;
    /// use teistro_core::quantity::{Depth, JulianDay};
    /// use teistro_core::settings::root;
    /// use teistro_rectification::sensitivity::{DashaAt, SensitivityRules};
    ///
    /// let coarse = SensitivityRules { seed_minutes: 90.0, ..SensitivityRules::default() };
    /// assert_eq!(coarse.check().unwrap_err().field(), Some("rectification.seedMinutes"));
    /// // Chara is sign-based: it has no Moon to follow.
    /// let chara = DashaAt {
    ///     at: JulianDay::literal(2_470_000.5),
    ///     system: DashaSystem::Chara,
    ///     depth: Depth::MIN,
    ///     rules: teistro_dasha::Rules::of(&root().dasha, DashaSystem::Chara),
    /// };
    /// let refused = SensitivityRules { dasha: Some(chara), ..SensitivityRules::default() };
    /// assert_eq!(refused.check().unwrap_err().field(), Some("rectification.dasha.system"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a seed step outside [`FINEST_SEED_MINUTES`] to an
    /// hour, a varga named twice, or a dasha system that is not
    /// nakshatra-seeded, each naming its field under `rectification`.
    pub fn check(&self) -> Result<(), Error> {
        self.row().map(|_| ())
    }

    /// The dasha's row, once the rules are checked.
    fn row(&self) -> Result<Option<&'static UduRow>, Error> {
        if !self.seed_minutes.is_finite()
            || !(FINEST_SEED_MINUTES..=60.0).contains(&self.seed_minutes)
        {
            return Err(Error::invalid_arg(format!(
                "the seed step is {FINEST_SEED_MINUTES:.4} to 60 minutes, not {}",
                self.seed_minutes
            ))
            .with_field("rectification.seedMinutes"));
        }
        for (index, varga) in self.vargas.iter().enumerate() {
            if self.vargas.iter().take(index).any(|seen| seen == varga) {
                return Err(Error::invalid_arg(format!("{varga} is named twice"))
                    .with_field("rectification.vargas"));
            }
        }
        let Some(dasha) = self.dasha else {
            return Ok(None);
        };
        teistro_dasha::row(dasha.system, dasha.rules.ashtottari_grouping)
            .map(Some)
            .ok_or_else(|| {
                Error::invalid_arg(format!(
                    "{} is not seeded by the Moon's nakshatra, so a birth's minutes do not move it",
                    dasha.system
                ))
                .with_field("rectification.dasha.system")
            })
    }
}

/// A nakshatra's quarter: which nakshatra, and which of its four padas.
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_rectification::sensitivity::Pada;
///
/// // 280°30′: half a degree into Shravana.
/// assert_eq!(Pada::of(280.5)?, Pada { nakshatra: Nakshatra::Shravana, pada: 1 });
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "SensitivityPada"))]
#[serde(rename_all = "camelCase")]
pub struct Pada {
    /// The nakshatra, of the twenty-seven.
    pub nakshatra: Nakshatra,
    /// The pada, 1 to 4.
    pub pada: u8,
}

impl Pada {
    /// The pada a longitude stands in.
    ///
    /// ```
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_rectification::sensitivity::Pada;
    ///
    /// assert_eq!(Pada::of(0.0)?, Pada { nakshatra: Nakshatra::Ashwini, pada: 1 });
    /// assert_eq!(Pada::of(359.9)?, Pada { nakshatra: Nakshatra::Revati, pada: 4 });
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for a longitude that is not finite.
    pub fn of(longitude_deg: f64) -> Result<Pada, Error> {
        let at = Nas::try_from_degrees(longitude_deg.rem_euclid(360.0))?;
        Ok(Pada {
            nakshatra: at.nakshatra(),
            pada: at.pada().get() + 1,
        })
    }
}

/// Where a fact changes, nearest the birth on one side.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_core::quantity::JulianDay;
/// use teistro_rectification::sensitivity::Change;
///
/// let change = Change { at: JulianDay::literal(2_460_000.3), minutes: 4.5, to: Rashi::Leo };
/// assert_eq!(change.to, Rashi::Leo);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "Sensitivity{T}Change"))]
#[serde(rename_all = "camelCase")]
pub struct Change<T> {
    /// The instant it changes, a UTC Julian day, to within
    /// [`EDGE_TOLERANCE_DAYS`].
    pub at: JulianDay<Utc>,
    /// How far it is from the birth, minutes, positive on either side
    /// (C1143).
    pub minutes: f64,
    /// What the fact becomes.
    pub to: T,
}

/// One fact at the birth and its nearest change either side; a side with
/// none holds the fact over the whole searched span.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_rectification::sensitivity::Fact;
///
/// let steady = Fact { value: Rashi::Leo, earlier: None, later: None };
/// assert!(steady.earlier.is_none() && steady.later.is_none());
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "Sensitivity{T}Fact"))]
#[serde(rename_all = "camelCase")]
pub struct Fact<T> {
    /// The fact at the birth.
    pub value: T,
    /// Its nearest change before the birth.
    pub earlier: Option<Change<T>>,
    /// Its nearest change after the birth.
    pub later: Option<Change<T>>,
}

/// One varga's lagna and its changes.
///
/// ```
/// use teistro_core::catalogue::{Rashi, Varga};
/// use teistro_rectification::sensitivity::{Fact, VargaLagna};
///
/// let d9 = VargaLagna { varga: Varga::D9, lagna: Fact { value: Rashi::Aries, earlier: None, later: None } };
/// assert_eq!(d9.varga, Varga::D9);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct VargaLagna {
    /// Which varga.
    pub varga: Varga,
    /// Its lagna's sign.
    pub lagna: Fact<Rashi>,
}

/// One bhava middle's sign and its changes.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_rectification::sensitivity::{Cusp, Fact};
///
/// let tenth = Cusp { house: 10, sign: Fact { value: Rashi::Capricorn, earlier: None, later: None } };
/// assert_eq!(tenth.house, 10);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "SensitivityCusp"))]
#[serde(rename_all = "camelCase")]
pub struct Cusp {
    /// The house, 1 to 12.
    pub house: u8,
    /// Its middle's sign under the chart's chalit.
    pub sign: Fact<Rashi>,
}

/// The dasha's running lords and their changes.
///
/// ```
/// use teistro_core::catalogue::{DashaSystem, Graha};
/// use teistro_core::quantity::JulianDay;
/// use teistro_rectification::sensitivity::{Fact, Running};
///
/// let running = Running {
///     at: JulianDay::literal(2_470_000.5),
///     system: DashaSystem::Vimshottari,
///     lords: Fact { value: vec![Graha::Venus, Graha::Sun], earlier: None, later: None },
/// };
/// assert_eq!(running.lords.value.len(), 2);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "RunningDasha"))]
#[serde(rename_all = "camelCase")]
pub struct Running {
    /// The reference instant.
    pub at: JulianDay<Utc>,
    /// The system.
    pub system: DashaSystem,
    /// The lords running at the reference instant, the mahadasha first;
    /// none where it is before the birth or past the cycle's end.
    pub lords: Fact<Vec<Graha>>,
}

/// How far a birth time can move before each fact its chart states
/// changes.
///
/// ```
/// # use teistro_core::catalogue::Vara;
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_rectification::{Day, Sky};
/// # struct Line;
/// # impl Sky for Line {
/// #     fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, teistro_core::Error> {
/// #         Ok(((at.get() - 2_460_000.25) * 360.0).rem_euclid(360.0))
/// #     }
/// #     fn sun_deg(&self, _: JulianDay<Utc>) -> Result<f64, teistro_core::Error> { Ok(10.0) }
/// #     fn moon_deg(&self, _: JulianDay<Utc>) -> Result<f64, teistro_core::Error> { Ok(100.0) }
/// #     fn day(&self, _: JulianDay<Utc>) -> Result<Day, teistro_core::Error> {
/// #         Ok(Day::literal(2_460_000.25, 2_460_000.75, 2_460_001.25, Vara::Budhavara))
/// #     }
/// # }
/// # use teistro_rectification::Window;
/// # use teistro_rectification::sensitivity::{SensitivityRules, SensitivitySky, sensitivity};
/// # impl SensitivitySky for Line {}
/// # let birth = JulianDay::literal(2_460_000.25 + 12.0 / 360.0);
/// # let window = Window::between(JulianDay::literal(birth.get() - 0.1), JulianDay::literal(birth.get() + 0.1))?;
/// let found = sensitivity(birth, window, &Line, &SensitivityRules::default())?;
/// // The Moon stands still, so its pada never changes.
/// assert!(found.moon_pada.earlier.is_none() && found.moon_pada.later.is_none());
/// // A navamsha is 3°20′, thirteen minutes and twenty seconds of this lagna.
/// let d9 = &found.vargas[0].lagna;
/// let span = d9.earlier.as_ref().ok_or("a change")?.minutes + d9.later.as_ref().ok_or("a change")?.minutes;
/// assert!((span - 40.0 / 3.0).abs() < 1e-3);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Sensitivity {
    /// The birth.
    pub birth: JulianDay<Utc>,
    /// The span searched either side of it.
    pub span: Window,
    /// How closely each change is placed, seconds: a hundredth.
    pub precision_seconds: f64,
    /// The seed grid the span was sampled on.
    pub grid: Grid,
    /// The lagna's sign.
    pub lagna_sign: Fact<Rashi>,
    /// The lagna's nakshatra and pada.
    pub lagna_pada: Fact<Pada>,
    /// Each varga's lagna, in the order the rules name them.
    pub vargas: Vec<VargaLagna>,
    /// The Moon's nakshatra and pada.
    pub moon_pada: Fact<Pada>,
    /// The dasha running at the reference instant, when one was asked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dasha: Option<Running>,
    /// Each bhava middle's sign, houses 1 to 12; none under a chalit whose
    /// middles only count on from the lagna's sign (C1141).
    pub cusps: Vec<Cusp>,
}

/// How a birth time's facts change around it, over `span`.
///
/// ```
/// # use teistro_core::catalogue::Vara;
/// # use teistro_core::quantity::{JulianDay, Utc};
/// # use teistro_rectification::{Day, Sky};
/// # struct Line;
/// # impl Sky for Line {
/// #     fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, teistro_core::Error> {
/// #         Ok(((at.get() - 2_460_000.25) * 360.0).rem_euclid(360.0))
/// #     }
/// #     fn sun_deg(&self, _: JulianDay<Utc>) -> Result<f64, teistro_core::Error> { Ok(10.0) }
/// #     fn moon_deg(&self, _: JulianDay<Utc>) -> Result<f64, teistro_core::Error> { Ok(100.0) }
/// #     fn day(&self, _: JulianDay<Utc>) -> Result<Day, teistro_core::Error> {
/// #         Ok(Day::literal(2_460_000.25, 2_460_000.75, 2_460_001.25, Vara::Budhavara))
/// #     }
/// # }
/// use teistro_rectification::Window;
/// use teistro_rectification::sensitivity::{SensitivityRules, SensitivitySky, sensitivity};
///
/// impl SensitivitySky for Line {}
///
/// let birth = JulianDay::literal(2_460_000.3);
/// let outside = Window::between(JulianDay::literal(2_460_000.4), JulianDay::literal(2_460_000.5))?;
/// let refused = sensitivity(birth, outside, &Line, &SensitivityRules::default()).unwrap_err();
/// assert_eq!(refused.field(), Some("rectification.span"));
/// # Ok::<(), teistro_core::Error>(())
/// ```
///
/// # Errors
///
/// What [`SensitivityRules::check`] refuses; `INVALID_ARG` for a span that
/// does not hold the birth inside it, named `rectification.span`; and
/// whatever the sky refuses.
pub fn sensitivity(
    birth: JulianDay<Utc>,
    span: Window,
    sky: &dyn SensitivitySky,
    rules: &SensitivityRules,
) -> Result<Sensitivity, Error> {
    let row = rules.row()?;
    let at = birth.get();
    if !(span.from.get() < at && at < span.to.get()) {
        return Err(Error::invalid_arg(format!(
            "the span searched holds the birth inside it, and {at} is not inside {} to {}",
            span.from.get(),
            span.to.get()
        ))
        .with_field("rectification.span"));
    }
    let reader = Reader {
        sky,
        schemes: rules
            .vargas
            .iter()
            .map(|varga| Scheme::of(*varga))
            .collect(),
        dasha: rules.dasha.zip(row),
        spans: RefCell::new(Vec::new()),
    };
    let step_days = rules.seed_minutes / (24.0 * 60.0);
    let (pieces, cells) = edges::runs(span, step_days, &[], |at| reader.facts(at))?;
    let own = pieces
        .iter()
        .position(|piece| piece.from <= at && at < piece.to)
        .ok_or_else(|| Error::internal("the runs tile the span, and none holds the birth"))?;
    let houses = pieces.get(own).map_or(0, |piece| piece.value.middles.len());
    let read = Sides {
        pieces: &pieces,
        own,
        at,
    };
    Ok(Sensitivity {
        birth,
        span,
        precision_seconds: EDGE_TOLERANCE_DAYS * 86_400.0,
        grid: Grid { step_days, cells },
        lagna_sign: read.fact(|facts| facts.lagna_sign),
        lagna_pada: read.fact(|facts| facts.lagna_pada),
        vargas: rules
            .vargas
            .iter()
            .enumerate()
            .map(|(index, varga)| VargaLagna {
                varga: *varga,
                lagna: read.fact(|facts| facts.vargas.get(index).copied().unwrap_or(Rashi::Aries)),
            })
            .collect(),
        moon_pada: read.fact(|facts| facts.moon_pada),
        dasha: rules.dasha.map(|dasha| Running {
            at: dasha.at,
            system: dasha.system,
            lords: read.fact(|facts| facts.lords.clone()),
        }),
        cusps: (0..houses)
            .map(|index| Cusp {
                house: u8::try_from(index + 1).unwrap_or(u8::MAX),
                sign: read.fact(|facts| facts.middles.get(index).copied().unwrap_or(Rashi::Aries)),
            })
            .collect(),
    })
}

/// Every fact a chart states from its clock, at one instant.
#[derive(Clone, Debug, PartialEq)]
struct Facts {
    lagna_sign: Rashi,
    lagna_pada: Pada,
    vargas: Vec<Rashi>,
    moon_pada: Pada,
    lords: Vec<Graha>,
    middles: Vec<Rashi>,
}

impl Default for Facts {
    fn default() -> Facts {
        let first = Pada {
            nakshatra: Nakshatra::Ashwini,
            pada: 1,
        };
        Facts {
            lagna_sign: Rashi::Aries,
            lagna_pada: first,
            vargas: Vec::new(),
            moon_pada: first,
            lords: Vec::new(),
            middles: Vec::new(),
        }
    }
}

/// What reads the facts at an instant.
struct Reader<'s> {
    sky: &'s dyn SensitivitySky,
    schemes: Vec<Scheme>,
    dasha: Option<(DashaAt, &'static UduRow)>,
    /// The Moon's stays in the segments met so far, by segment, which a
    /// temporal balance reads: every birth in one segment shares its stay.
    spans: RefCell<Vec<(u8, Interval)>>,
}

impl Reader<'_> {
    fn facts(&self, at: f64) -> Result<Facts, Error> {
        let instant = JulianDay::literal(at);
        let lagna_deg = self.sky.ascendant_deg(instant)?;
        let lagna = Nas::try_from_degrees(lagna_deg.rem_euclid(360.0))?;
        let moon_deg = self.sky.moon_deg(instant)?;
        let moon = Nas::try_from_degrees(moon_deg.rem_euclid(360.0))?;
        let lords = match self.dasha {
            Some((dasha, row)) => {
                let moon_span = match dasha.rules.balance {
                    Balance::Temporal => Some(self.stay(row, moon, at)?),
                    _ => None,
                };
                let birth = Birth {
                    instant,
                    moon,
                    moon_span,
                };
                Dasha::new(row, &birth, dasha.rules)?
                    .at(dasha.at, dasha.depth)
                    .iter()
                    .map(|period| period.lord)
                    .collect()
            }
            None => Vec::new(),
        };
        let middles = match self.sky.middles_deg(instant)? {
            Some(middles) => middles
                .iter()
                .map(|deg| Rashi::of_longitude(*deg))
                .collect(),
            None => Vec::new(),
        };
        Ok(Facts {
            lagna_sign: Rashi::of_longitude(lagna_deg),
            lagna_pada: Pada::of(lagna_deg)?,
            vargas: self
                .schemes
                .iter()
                .map(|scheme| sign(scheme, lagna))
                .collect(),
            moon_pada: Pada::of(moon_deg)?,
            lords,
            middles,
        })
    }

    /// The Moon's stay in the segment of `row`'s wheel she stands in at
    /// `at`: from where the segment changes before to where it changes
    /// after, bisected as every other edge is.
    fn stay(&self, row: &UduRow, moon: Nas, at: f64) -> Result<Interval, Error> {
        let place = row.wheel.segment(moon).place;
        if let Some((_, held)) = self.spans.borrow().iter().find(|(seen, _)| *seen == place) {
            return Ok(*held);
        }
        let segment = |at: f64| -> Result<u8, Error> {
            let moon = self.sky.moon_deg(JulianDay::literal(at))?;
            Ok(row
                .wheel
                .segment(Nas::try_from_degrees(moon.rem_euclid(360.0))?)
                .place)
        };
        let edge = |direction: f64| -> Result<f64, Error> {
            let mut inside = at;
            for _ in 0..STAY_STEPS {
                let next = inside + direction * STAY_STEP_DAYS;
                let found = segment(next)?;
                if found != place {
                    let mut cuts = Vec::new();
                    let (before, after) = if direction > 0.0 {
                        ((inside, place), (next, found))
                    } else {
                        ((next, found), (inside, place))
                    };
                    edges::bisect(&segment, before, after, &mut cuts)?;
                    let nearest = if direction > 0.0 {
                        cuts.first()
                    } else {
                        cuts.last()
                    };
                    return nearest.copied().ok_or_else(|| {
                        Error::internal("a segment that differs at both ends changes between them")
                    });
                }
                inside = next;
            }
            Err(Error::internal(
                "the Moon stayed in one segment for longer than a segment lasts",
            ))
        };
        let stay = Interval::new(
            JulianDay::literal(edge(-1.0)?),
            JulianDay::literal(edge(1.0)?),
        )?;
        self.spans.borrow_mut().push((place, stay));
        Ok(stay)
    }
}

/// How far a temporal balance's search steps from the birth to find the
/// Moon's segment changed: three hours, under the 13°20′ a segment spans at
/// the Moon's fastest.
const STAY_STEP_DAYS: f64 = 0.125;

/// How many steps a stay's edge is looked for over: two days, longer than
/// the Moon stays in any segment of either wheel.
const STAY_STEPS: u32 = 16;

/// The runs of a span and the one that holds the birth.
struct Sides<'p> {
    pieces: &'p [Piece<Facts>],
    own: usize,
    at: f64,
}

impl Sides<'_> {
    /// One fact at the birth and its nearest change either side: the first
    /// run outward whose value of it differs.
    fn fact<T: Clone + PartialEq>(&self, read: impl Fn(&Facts) -> T) -> Fact<T> {
        // `own` was found among the pieces, so it is one of them.
        let birth = self.pieces.get(self.own).map(|piece| read(&piece.value));
        let value = birth.unwrap_or_else(|| read(&Facts::default()));
        let later = self
            .pieces
            .iter()
            .skip(self.own + 1)
            .find(|piece| read(&piece.value) != value)
            .map(|piece| Change {
                at: JulianDay::literal(piece.from),
                minutes: (piece.from - self.at) * 1440.0,
                to: read(&piece.value),
            });
        let earlier = self
            .pieces
            .iter()
            .take(self.own)
            .rev()
            .find(|piece| read(&piece.value) != value)
            .map(|piece| Change {
                at: JulianDay::literal(piece.to),
                minutes: (self.at - piece.to) * 1440.0,
                to: read(&piece.value),
            });
        Fact {
            value,
            earlier,
            later,
        }
    }
}
