//! The **lots**: points a chart does not place, each the distance between
//! two of its points counted from a third (`03-design/hellenistic-lots.md`).
//!
//! Every lot is that one shape, so a lot is data here: a [`LotFormula`]
//! over a closed set of [`LotPoint`]s, and the fourteen Valens gives are a
//! table of them ([`Lot::formula`]). A caller's own lot, or another
//! author's reading of one of these, is a formula handed to the same
//! evaluator ([`lot_place`]): nothing about Valens's fourteen is
//! privileged but their names.

use core::cell::Cell;

use serde::{Deserialize, Serialize};
use teistro_core::angle::normalise_deg;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::house::House;

use crate::dignity::{Sect, SectRule, exaltation_degree};
use crate::reading::ChartSky;

/// How the Part of Fortune is taken by night (cruxes C220 and C221).
///
/// Daimon is Fortune reflected in the ascendant, so the rule that
/// reverses one reverses the other.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum FortuneRule {
    /// Lilly's (pp. 143–144): the ascendant plus the Moon less the Sun, by
    /// day or night, which he gives as Ptolemy's. The almuten's default.
    #[default]
    DayAndNight,
    /// By night, the ascendant plus the Sun less the Moon: the rule Lilly
    /// reports and sets aside, and Valens's at *Anthologies* II.22, where
    /// Daimon is counted the other way. The lots' default. The Tajika
    /// Punya saham reverses the same way.
    ReversedByNight,
    /// Valens's own preference (*Anthologies* III.11): by night, reversed
    /// while the Moon is above the earth, "until the time it sets", and
    /// counted from the Sun to the Moon once it has set. The Moon is read
    /// by its centre's geometric altitude, as [`SectRule::Horizon`] reads
    /// the Sun's, and on the horizon it has set.
    ReversedWhileMoonUp,
}

impl FortuneRule {
    /// Whether Fortune is counted from the Moon to the Sun in a chart of
    /// this sect, the Moon's centre this many degrees above the true
    /// horizon. Only [`FortuneRule::ReversedWhileMoonUp`] reads the Moon.
    ///
    /// ```
    /// use teistro_hellenistic::{FortuneRule, Sect};
    ///
    /// assert!(!FortuneRule::DayAndNight.reverses(Sect::Night, 30.0));
    /// assert!(FortuneRule::ReversedByNight.reverses(Sect::Night, -30.0));
    /// assert!(FortuneRule::ReversedWhileMoonUp.reverses(Sect::Night, 30.0));
    /// assert!(!FortuneRule::ReversedWhileMoonUp.reverses(Sect::Night, -30.0));
    /// assert!(!FortuneRule::ReversedWhileMoonUp.reverses(Sect::Day, 30.0));
    /// ```
    #[must_use]
    pub fn reverses(self, sect: Sect, moon_altitude_deg: f64) -> bool {
        match (self, sect) {
            (_, Sect::Day) | (FortuneRule::DayAndNight, Sect::Night) => false,
            (FortuneRule::ReversedByNight, Sect::Night) => true,
            (FortuneRule::ReversedWhileMoonUp, Sect::Night) => moon_altitude_deg > 0.0,
        }
    }
}

/// The Part of Fortune's longitude, in degrees in `[0, 360)`: the Moon's
/// distance from the Sun counted from the ascendant, or the Sun's from
/// the Moon when `reversed` ([`FortuneRule::reverses`] says when).
///
/// ```
/// use teistro_hellenistic::{FortuneRule, Sect, part_of_fortune};
///
/// // Lilly, pp. 143–144: the Moon at 21°18′ Virgo, the Sun at 4°18′
/// // Aries, 23°27′ Leo rising: Fortune at 10°27′ Aquarius.
/// let at = |sign: f64, deg: f64, min: f64| sign * 30.0 + deg + min / 60.0;
/// let reversed = FortuneRule::DayAndNight.reverses(Sect::Night, -10.0);
/// let fortune = part_of_fortune(at(4.0, 23.0, 27.0), at(0.0, 4.0, 18.0), at(5.0, 21.0, 18.0), reversed);
/// assert!((fortune - at(10.0, 10.0, 27.0)).abs() < 1e-9);
/// ```
#[must_use]
pub fn part_of_fortune(ascendant_deg: f64, sun_deg: f64, moon_deg: f64, reversed: bool) -> f64 {
    if reversed {
        counted(moon_deg, sun_deg, ascendant_deg, Distance::Forward)
    } else {
        counted(sun_deg, moon_deg, ascendant_deg, Distance::Forward)
    }
}

/// The distance from `from` to `to`, counted from `counted_from`.
fn counted(from: f64, to: f64, counted_from: f64, distance: Distance) -> f64 {
    let forward = normalise_deg(to - from);
    let arc = match distance {
        Distance::Forward => forward,
        Distance::Shorter => forward.min(360.0 - forward),
    };
    normalise_deg(counted_from + arc)
}

/// One of the lots Valens gives (*Anthologies*, Riley's translation), each
/// from its own chapter; [`Lot::formula`] is each as data.
///
/// Serialised as its key in screaming snake case (`FOREIGN_LANDS`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Lot {
    /// **Fortune** (II.22, III.11): from the Sun to the Moon, from the
    /// ascendant; by night as [`FortuneRule`] says.
    Fortune,
    /// **Daimon** (II.22): from the Moon to the Sun by day, the Sun to the
    /// Moon by night; always Fortune reflected in the ascendant.
    Daimon,
    /// **Basis** (II.22): the shorter arc between Fortune and Daimon, from
    /// the ascendant, by day and night alike: "from the nearest Lot to the
    /// other", never more than seven signs.
    Basis,
    /// **Love** (IV.25, a marginal note): from Fortune to Daimon by day,
    /// reversed by night.
    Love,
    /// **Necessity** (IV.25, a marginal note): from Daimon to Fortune by
    /// day, reversed by night.
    Necessity,
    /// **Exaltation** (II.18): from the Sun to its exaltation by day, from
    /// the Moon to its exaltation by night (C222).
    Exaltation,
    /// **Debt** (II.23): from Mercury to Saturn, by day and night.
    Debt,
    /// **Theft** (II.24): from Mercury to Mars by day, reversed by night,
    /// counted from **Saturn**.
    Theft,
    /// **Deceit** (II.25): from the Sun to Mars by day, reversed by night.
    Deceit,
    /// **Foreign lands** (II.29): from Saturn to Mars, by day and night.
    ForeignLands,
    /// **The father** (II.31, after Timaios): from the Sun to Saturn by
    /// day, and from **Venus to the Moon** by night. The text adds that
    /// "some" count the Sun to Jupiter.
    Father,
    /// **Marriage** (II.37): from Jupiter to Venus by day, reversed by
    /// night. II.38 gives another, by the native's sex.
    Marriage,
    /// **Brothers** (II.40): from Saturn to Jupiter by day, reversed by
    /// night.
    Brothers,
    /// **Crisis**, the crisis-producing place (V.1): from Saturn to Mars by
    /// day, reversed by night.
    Crisis,
}

impl Lot {
    /// Every lot, in Valens's order as this module lists them.
    pub const ALL: [Lot; 14] = [
        Lot::Fortune,
        Lot::Daimon,
        Lot::Basis,
        Lot::Love,
        Lot::Necessity,
        Lot::Exaltation,
        Lot::Debt,
        Lot::Theft,
        Lot::Deceit,
        Lot::ForeignLands,
        Lot::Father,
        Lot::Marriage,
        Lot::Brothers,
        Lot::Crisis,
    ];

    /// The lot's formula, as the text gives it. Fortune's and Daimon's
    /// night arcs are each other's day arcs; [`FortuneRule`] decides when
    /// those two take them, and every other lot takes its night arc by
    /// night.
    ///
    /// ```
    /// use teistro_hellenistic::{Lot, LotPoint};
    /// use teistro_core::catalogue::Graha;
    ///
    /// let theft = Lot::Theft.formula();
    /// assert_eq!(theft.day.counted_from, LotPoint::Planet(Graha::Saturn));
    /// assert_eq!(theft.night.from, LotPoint::Planet(Graha::Mars));
    /// ```
    #[must_use]
    pub const fn formula(self) -> LotFormula {
        use Graha::{Jupiter, Mars, Mercury, Moon, Saturn, Sun, Venus};
        use LotPoint::{Ascendant, Planet};
        const FORTUNE: LotPoint = LotPoint::Lot(Lot::Fortune);
        const DAIMON: LotPoint = LotPoint::Lot(Lot::Daimon);
        match self {
            Lot::Fortune => LotFormula::reversed_by_night(Planet(Sun), Planet(Moon), Ascendant),
            Lot::Daimon => LotFormula::reversed_by_night(Planet(Moon), Planet(Sun), Ascendant),
            Lot::Basis => LotFormula::same(LotArc::new(FORTUNE, DAIMON, Ascendant).shorter()),
            Lot::Love => LotFormula::reversed_by_night(FORTUNE, DAIMON, Ascendant),
            Lot::Necessity => LotFormula::reversed_by_night(DAIMON, FORTUNE, Ascendant),
            Lot::Exaltation => LotFormula {
                day: LotArc::new(Planet(Sun), LotPoint::Exaltation(Sun), Ascendant),
                night: LotArc::new(Planet(Moon), LotPoint::Exaltation(Moon), Ascendant),
            },
            Lot::Debt => LotFormula::same(LotArc::new(Planet(Mercury), Planet(Saturn), Ascendant)),
            Lot::Theft => {
                LotFormula::reversed_by_night(Planet(Mercury), Planet(Mars), Planet(Saturn))
            }
            Lot::Deceit => LotFormula::reversed_by_night(Planet(Sun), Planet(Mars), Ascendant),
            Lot::ForeignLands => {
                LotFormula::same(LotArc::new(Planet(Saturn), Planet(Mars), Ascendant))
            }
            Lot::Father => LotFormula {
                day: LotArc::new(Planet(Sun), Planet(Saturn), Ascendant),
                night: LotArc::new(Planet(Venus), Planet(Moon), Ascendant),
            },
            Lot::Marriage => {
                LotFormula::reversed_by_night(Planet(Jupiter), Planet(Venus), Ascendant)
            }
            Lot::Brothers => {
                LotFormula::reversed_by_night(Planet(Saturn), Planet(Jupiter), Ascendant)
            }
            Lot::Crisis => LotFormula::reversed_by_night(Planet(Saturn), Planet(Mars), Ascendant),
        }
    }

    /// Whether [`FortuneRule`], not the sect, decides when this lot takes
    /// its night arc.
    const fn follows_fortune(self) -> bool {
        matches!(self, Lot::Fortune | Lot::Daimon)
    }
}

/// One point a lot is counted between or from.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LotPoint {
    /// The ascendant.
    Ascendant,
    /// The midheaven.
    Midheaven,
    /// One of the seven; the nodes and the outer planets are refused.
    Planet(Graha),
    /// Another lot, as the request reads it.
    Lot(Lot),
    /// One of the seven's exaltation, at Lilly's degree
    /// ([`exaltation_degree`]): the Sun's is Aries 19°.
    Exaltation(Graha),
    /// A fixed longitude, degrees: `Degrees(0.0)` is the first degree of
    /// Aries, the other reading of C222.
    Degrees(f64),
}

/// Which way the distance between two points is taken.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Distance {
    /// In the order of the signs from the first point to the second: every
    /// lot but Basis.
    #[default]
    Forward,
    /// The shorter way round, never more than half the circle: Basis.
    Shorter,
}

/// One arc of a lot: the distance from one point to another, counted from
/// a third.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LotArc {
    /// The point the distance is taken from.
    pub from: LotPoint,
    /// The point the distance is taken to.
    pub to: LotPoint,
    /// The point the distance is counted from, in the order of the signs.
    pub counted_from: LotPoint,
    /// Which way the distance is taken.
    #[serde(default)]
    pub distance: Distance,
}

impl LotArc {
    /// The distance from `from` to `to` in the order of the signs, counted
    /// from `counted_from`.
    #[must_use]
    pub const fn new(from: LotPoint, to: LotPoint, counted_from: LotPoint) -> LotArc {
        LotArc {
            from,
            to,
            counted_from,
            distance: Distance::Forward,
        }
    }

    /// The same arc, its distance taken the shorter way round.
    #[must_use]
    pub const fn shorter(mut self) -> LotArc {
        self.distance = Distance::Shorter;
        self
    }
}

/// How a lot is counted by day and by night.
///
/// ```
/// use teistro_hellenistic::{LotArc, LotFormula, LotPoint};
/// use teistro_core::catalogue::Graha;
///
/// // Valens's lot of children (II.39) for a woman: from Jupiter to Venus,
/// // by day and night. It is the native's, not the sect's, so it is a
/// // formula a caller writes and not a member of `Lot`.
/// let children = LotFormula::same(LotArc::new(
///     LotPoint::Planet(Graha::Jupiter),
///     LotPoint::Planet(Graha::Venus),
///     LotPoint::Ascendant,
/// ));
/// assert_eq!(children.at(true), children.day);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LotFormula {
    /// For a day chart.
    pub day: LotArc,
    /// For a night chart.
    pub night: LotArc,
}

impl LotFormula {
    /// The same arc by day and by night.
    #[must_use]
    pub const fn same(arc: LotArc) -> LotFormula {
        LotFormula {
            day: arc,
            night: arc,
        }
    }

    /// From `from` to `to` by day and from `to` to `from` by night, each
    /// counted from `counted_from`: the shape most lots take.
    #[must_use]
    pub const fn reversed_by_night(
        from: LotPoint,
        to: LotPoint,
        counted_from: LotPoint,
    ) -> LotFormula {
        LotFormula {
            day: LotArc::new(from, to, counted_from),
            night: LotArc::new(to, from, counted_from),
        }
    }

    /// The arc a chart reads: the night arc when `night`.
    #[must_use]
    pub const fn at(&self, night: bool) -> LotArc {
        if night { self.night } else { self.day }
    }
}

/// How a chart's lots are read: its sect's rule and Fortune's by night.
///
/// The default is Valens's: his horizon for the sect, and Fortune
/// reversed by night as at *Anthologies* II.22.
///
/// ```
/// use teistro_hellenistic::{FortuneRule, LotRequest, SectRule};
///
/// let asked = LotRequest::from_json(r#"{"fortune": "REVERSED_WHILE_MOON_UP"}"#)?;
/// assert_eq!(asked.fortune(), FortuneRule::ReversedWhileMoonUp);
/// assert_eq!(asked.sect_rule(), SectRule::Horizon);
///
/// let typo = LotRequest::from_json(r#"{"fortuna": "DAY_AND_NIGHT"}"#).unwrap_err();
/// assert_eq!(typo.field(), Some("lots.fortuna"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct LotRequest {
    sect_rule: SectRule,
    fortune: FortuneRule,
}

impl Default for LotRequest {
    fn default() -> LotRequest {
        LotRequest::VALENS
    }
}

impl LotRequest {
    /// Valens's: his horizon, and Fortune reversed by night.
    pub const VALENS: LotRequest = LotRequest {
        sect_rule: SectRule::Horizon,
        fortune: FortuneRule::ReversedByNight,
    };

    /// The request as the bindings write it, `{"sectRule": ..., "fortune":
    /// ...}`, every member optional and taking Valens's.
    ///
    /// # Errors
    ///
    /// Text that is not the record, a member it does not know, or a value
    /// that is not one, each named under `lots`.
    pub fn from_json(text: &str) -> Result<LotRequest, Error> {
        teistro_core::strict::read(text, "lots")
    }

    /// The same request, the sect read otherwise.
    #[must_use]
    pub const fn with_sect_rule(mut self, sect_rule: SectRule) -> LotRequest {
        self.sect_rule = sect_rule;
        self
    }

    /// The same request, Fortune taken otherwise by night.
    #[must_use]
    pub const fn with_fortune(mut self, fortune: FortuneRule) -> LotRequest {
        self.fortune = fortune;
        self
    }

    /// How the sect is read.
    #[must_use]
    pub const fn sect_rule(&self) -> SectRule {
        self.sect_rule
    }

    /// How Fortune is taken by night.
    #[must_use]
    pub const fn fortune(&self) -> FortuneRule {
        self.fortune
    }
}

/// What a chart's lots are read from: its seven and the two altitudes
/// ([`ChartSky`]), and its angles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LotSky {
    /// The seven, the Sun's and the Moon's altitudes, and the daylight.
    pub chart: ChartSky,
    /// The ascendant, degrees in the chart's zodiac.
    pub ascendant_deg: f64,
    /// The midheaven, degrees in the chart's zodiac.
    pub midheaven_deg: f64,
}

/// Where a lot fell.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LotPlace {
    /// The lot, degrees in the chart's zodiac, in `[0, 360)`.
    pub longitude_deg: f64,
    /// The sign it falls in.
    pub sign: Rashi,
    /// That sign's lord, the lot's ruler, which Valens reads it by.
    pub lord: Graha,
    /// Its place counted in whole signs from the ascendant's sign, as
    /// Valens counts "the XI Place".
    pub house: House,
}

/// One of the lots asked for, where it fell.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PlacedLot {
    /// Which.
    pub lot: Lot,
    /// Where it fell.
    pub place: LotPlace,
}

/// The lots a chart was asked for, and how it was read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LotReading {
    /// The chart's sect, as the request's rule reads it.
    pub sect: Sect,
    /// The request it was read under.
    pub request: LotRequest,
    /// Whether Fortune was counted from the Moon to the Sun (and Daimon
    /// from the Sun to the Moon).
    pub fortune_reversed: bool,
    /// One per lot asked, in the order asked.
    pub lots: Vec<PlacedLot>,
}

/// The lots asked for, in a chart.
///
/// Each is computed once however many read it: Basis, Love and Necessity
/// each read Fortune and Daimon.
///
/// ```
/// use teistro_hellenistic::{ChartSky, Lot, LotRequest, LotSky, lots};
///
/// // Valens III.11's first example, by night: the Sun in Cancer, the Moon
/// // and the ascendant in Pisces, "the Lot of Fortune in Cancer".
/// let sky = LotSky {
///     chart: ChartSky {
///         saturn_deg: 255.0, jupiter_deg: 285.0, mars_deg: 225.0, sun_deg: 105.0,
///         venus_deg: 100.0, mercury_deg: 135.0, moon_deg: 345.0,
///         sun_altitude_deg: -40.0, moon_altitude_deg: 2.0, daylight: false,
///     },
///     ascendant_deg: 340.0,
///     midheaven_deg: 250.0,
/// };
/// let read = lots(&sky, &[Lot::Fortune, Lot::Daimon], LotRequest::VALENS)?;
/// assert!(read.fortune_reversed);
/// assert_eq!(read.lots[0].place.longitude_deg, 100.0);
/// assert_eq!(read.lots[0].place.house.get(), 5);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for a longitude or an altitude that is not a number,
/// named by its field.
pub fn lots(sky: &LotSky, which: &[Lot], request: LotRequest) -> Result<LotReading, Error> {
    let reading = Evaluation::of(sky, request)?;
    let lots = which
        .iter()
        .map(|&lot| PlacedLot {
            lot,
            place: reading.place(reading.lot(lot)),
        })
        .collect();
    Ok(LotReading {
        sect: reading.sect,
        request,
        fortune_reversed: reading.fortune_reversed,
        lots,
    })
}

/// Where a lot of the caller's own falls: any formula over the same
/// points, read under the same request. Its night arc is taken by night;
/// a [`LotPoint::Lot`] in it reads that lot as [`lots`] would.
///
/// ```
/// use teistro_hellenistic::{ChartSky, Lot, LotArc, LotFormula, LotPoint, LotRequest, LotSky, lot_place};
/// use teistro_core::catalogue::Graha;
///
/// let sky = LotSky {
///     chart: ChartSky {
///         saturn_deg: 300.0, jupiter_deg: 250.0, mars_deg: 200.0, sun_deg: 10.0,
///         venus_deg: 40.0, mercury_deg: 20.0, moon_deg: 100.0,
///         sun_altitude_deg: 30.0, moon_altitude_deg: 10.0, daylight: true,
///     },
///     ascendant_deg: 50.0,
///     midheaven_deg: 320.0,
/// };
/// // Exaltation counted to the first degree of Aries, the other reading
/// // of C222: 0 − 10 + 50.
/// let to_the_sign = LotFormula::same(LotArc::new(
///     LotPoint::Planet(Graha::Sun),
///     LotPoint::Degrees(0.0),
///     LotPoint::Ascendant,
/// ));
/// assert_eq!(lot_place(&sky, &to_the_sign, LotRequest::VALENS)?.longitude_deg, 40.0);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`lots`], and `INVALID_ARG` for a planet outside the seven or a
/// fixed degree that is not a number, named by the point it stands in
/// (`formula.night.to`).
pub fn lot_place(
    sky: &LotSky,
    formula: &LotFormula,
    request: LotRequest,
) -> Result<LotPlace, Error> {
    let reading = Evaluation::of(sky, request)?;
    let night = reading.sect == Sect::Night;
    let arc = formula.at(night);
    let half = if night { "night" } else { "day" };
    for (name, point) in [
        ("from", arc.from),
        ("to", arc.to),
        ("countedFrom", arc.counted_from),
    ] {
        check_point(sky, point).map_err(|why| why.with_field(format!("formula.{half}.{name}")))?;
    }
    Ok(reading.place(reading.arc(arc)))
}

/// Where one point a lot is counted from falls, with its sign, lord and
/// whole-sign house: the start a time lord is counted from when it is not
/// the Ascendant (Valens IV.11 profects from "every point": the Sun, the
/// Moon, Fortune, Daimon). A lot named as the point is read under
/// `request`, as [`lots`] reads it.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_hellenistic::{
///     ChartSky, Lot, LotPoint, LotRequest, LotSky, lots, point_place,
/// };
///
/// let sky = LotSky {
///     chart: ChartSky {
///         saturn_deg: 300.0,
///         jupiter_deg: 250.0,
///         mars_deg: 100.0,
///         sun_deg: 10.0,
///         venus_deg: 40.0,
///         mercury_deg: 20.0,
///         moon_deg: 130.0,
///         sun_altitude_deg: 30.0,
///         daylight: true,
///         moon_altitude_deg: 20.0,
///     },
///     ascendant_deg: 50.0,
///     midheaven_deg: 320.0,
/// };
/// let moon = point_place(&sky, LotPoint::Planet(Graha::Moon), LotRequest::VALENS)?;
/// assert_eq!(moon.sign, Rashi::Leo);
/// let fortune = point_place(&sky, LotPoint::Lot(Lot::Fortune), LotRequest::VALENS)?;
/// let read = lots(&sky, &[Lot::Fortune], LotRequest::VALENS)?;
/// assert_eq!(fortune, read.lots[0].place);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`lots`], and `INVALID_ARG` for a planet outside the seven or a
/// fixed degree that is not a number, named `point`.
pub fn point_place(sky: &LotSky, point: LotPoint, request: LotRequest) -> Result<LotPlace, Error> {
    let reading = Evaluation::of(sky, request)?;
    check_point(sky, point).map_err(|why| why.with_field("point"))?;
    Ok(reading.place(reading.point(point)))
}

/// A point the sky cannot supply, refused.
fn check_point(sky: &LotSky, point: LotPoint) -> Result<(), Error> {
    match point {
        LotPoint::Planet(planet) | LotPoint::Exaltation(planet)
            if sky.chart.of(planet).is_none() =>
        {
            Err(Error::invalid_arg(format!(
                "{planet:?} is not one of the seven a lot is counted from"
            )))
        }
        LotPoint::Degrees(deg) if !deg.is_finite() => Err(Error::invalid_arg(format!(
            "{deg} is not a number of degrees"
        ))),
        _ => Ok(()),
    }
}

/// One chart's lots, each computed the first time it is read.
struct Evaluation<'a> {
    sky: &'a LotSky,
    sect: Sect,
    fortune_reversed: bool,
    memo: [Cell<Option<f64>>; Lot::ALL.len()],
}

impl<'a> Evaluation<'a> {
    fn of(sky: &'a LotSky, request: LotRequest) -> Result<Evaluation<'a>, Error> {
        check_sky(sky)?;
        let chart = &sky.chart;
        let sect = request
            .sect_rule
            .sect(chart.sun_altitude_deg, chart.daylight);
        Ok(Evaluation {
            sky,
            sect,
            fortune_reversed: request.fortune.reverses(sect, chart.moon_altitude_deg),
            memo: [const { Cell::new(None) }; Lot::ALL.len()],
        })
    }

    fn lot(&self, lot: Lot) -> f64 {
        // `memo` holds one slot per member of `Lot::ALL`, in its order.
        let slot = Lot::ALL
            .iter()
            .position(|&member| member == lot)
            .and_then(|index| self.memo.get(index));
        if let Some(at) = slot.and_then(Cell::get) {
            return at;
        }
        let night = if lot.follows_fortune() {
            self.fortune_reversed
        } else {
            self.sect == Sect::Night
        };
        let at = self.arc(lot.formula().at(night));
        if let Some(slot) = slot {
            slot.set(Some(at));
        }
        at
    }

    fn arc(&self, arc: LotArc) -> f64 {
        counted(
            self.point(arc.from),
            self.point(arc.to),
            self.point(arc.counted_from),
            arc.distance,
        )
    }

    /// A point's longitude; a point the sky cannot supply is refused
    /// before it is read, so it reads as 0° here.
    fn point(&self, point: LotPoint) -> f64 {
        match point {
            LotPoint::Ascendant => self.sky.ascendant_deg,
            LotPoint::Midheaven => self.sky.midheaven_deg,
            LotPoint::Planet(planet) => self.sky.chart.of(planet).map_or(0.0, |(at, _)| at),
            LotPoint::Lot(lot) => self.lot(lot),
            LotPoint::Exaltation(planet) => exaltation_degree(planet)
                .map_or(0.0, |(sign, deg)| sign.start_deg() + f64::from(deg)),
            LotPoint::Degrees(deg) => deg,
        }
    }

    fn place(&self, longitude_deg: f64) -> LotPlace {
        let sign = Rashi::of_longitude(longitude_deg);
        LotPlace {
            longitude_deg,
            sign,
            lord: sign.attributes().lord,
            house: House::between(Rashi::of_longitude(self.sky.ascendant_deg), sign),
        }
    }
}

/// The chart's longitudes and altitudes and its two angles each a finite
/// number, naming the first that is not.
fn check_sky(sky: &LotSky) -> Result<(), Error> {
    sky.chart.check()?;
    for (field, value) in [
        ("ascendant_deg", sky.ascendant_deg),
        ("midheaven_deg", sky.midheaven_deg),
    ] {
        if !value.is_finite() {
            return Err(
                Error::invalid_arg(format!("{value} is not a number of degrees")).with_field(field),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own results"
    )]

    use teistro_core::catalogue::{Graha, Rashi};

    use super::{
        ChartSky, FortuneRule, Lot, LotArc, LotFormula, LotPoint, LotRequest, LotSky, Sect,
        lot_place, lots, part_of_fortune,
    };

    /// A sky with the Sun, the Moon and the ascendant where a test puts
    /// them, the rest fixed, by night unless the Sun is raised.
    fn sky(sun: f64, moon: f64, ascendant: f64, sun_up: bool, moon_up: bool) -> LotSky {
        LotSky {
            chart: ChartSky {
                saturn_deg: 255.0,
                jupiter_deg: 285.0,
                mars_deg: 225.0,
                sun_deg: sun,
                venus_deg: 100.0,
                mercury_deg: 135.0,
                moon_deg: moon,
                sun_altitude_deg: if sun_up { 30.0 } else { -30.0 },
                moon_altitude_deg: if moon_up { 3.0 } else { -3.0 },
                daylight: sun_up,
            },
            ascendant_deg: ascendant,
            midheaven_deg: (ascendant + 270.0) % 360.0,
        }
    }

    fn at(sky: &LotSky, lot: Lot, request: LotRequest) -> f64 {
        lots(sky, &[lot], request).unwrap().lots[0]
            .place
            .longitude_deg
    }

    #[test]
    fn by_night_the_reversal_takes_the_suns_distance_from_the_moon() {
        let (asc, sun, moon) = (100.0, 10.0, 40.0);
        assert_eq!(part_of_fortune(asc, sun, moon, false), 130.0);
        assert_eq!(part_of_fortune(asc, sun, moon, true), 70.0);
        // Lilly's check (p. 144): at the full Moon, Fortune in the seventh.
        assert_eq!(part_of_fortune(0.0, 0.0, 180.0, false), 180.0);
        assert_eq!(part_of_fortune(10.0, 350.0, 0.0, false), 20.0);
    }

    #[test]
    fn valens_iii_11_reverses_fortune_by_night_and_lilly_does_not() {
        // "Sun, Venus in Cancer, moon, Ascendant in Pisces ... the Lot of
        // Fortune in Cancer"; "sun in Aquarius, moon, Ascendant in Virgo
        // ... the Lot of Fortune in Aquarius", a nocturnal nativity.
        let lilly = LotRequest::VALENS.with_fortune(FortuneRule::DayAndNight);
        for (sun, moon, ascendant, valens, other) in [
            (105.0, 345.0, 340.0, Rashi::Cancer, Rashi::Scorpio),
            (315.0, 160.0, 155.0, Rashi::Aquarius, Rashi::Aries),
        ] {
            let chart = sky(sun, moon, ascendant, false, true);
            for request in [
                LotRequest::VALENS,
                LotRequest::VALENS.with_fortune(FortuneRule::ReversedWhileMoonUp),
            ] {
                let read = lots(&chart, &[Lot::Fortune], request).unwrap();
                assert!(read.fortune_reversed);
                assert_eq!(read.lots[0].place.sign, valens);
            }
            assert_eq!(Rashi::of_longitude(at(&chart, Lot::Fortune, lilly)), other);
        }
    }

    #[test]
    fn the_moon_set_counts_fortune_from_the_sun_under_valens_own_rule() {
        let rule = LotRequest::VALENS.with_fortune(FortuneRule::ReversedWhileMoonUp);
        let set = sky(105.0, 345.0, 340.0, false, false);
        let read = lots(&set, &[Lot::Fortune], rule).unwrap();
        assert!(!read.fortune_reversed);
        assert_eq!(
            read.lots[0].place.longitude_deg,
            340.0 + 345.0 - 105.0 - 360.0
        );
        // By day no rule reverses it, whatever the Moon.
        let day = sky(105.0, 345.0, 340.0, true, true);
        assert!(!lots(&day, &[Lot::Fortune], rule).unwrap().fortune_reversed);
    }

    #[test]
    fn daimon_is_fortune_reflected_in_the_ascendant_under_every_rule() {
        for rule in [
            FortuneRule::DayAndNight,
            FortuneRule::ReversedByNight,
            FortuneRule::ReversedWhileMoonUp,
        ] {
            for (sun_up, moon_up) in [(true, true), (false, true), (false, false)] {
                let chart = sky(105.0, 345.0, 340.0, sun_up, moon_up);
                let request = LotRequest::VALENS.with_fortune(rule);
                let fortune = at(&chart, Lot::Fortune, request);
                let daimon = at(&chart, Lot::Daimon, request);
                let mirrored = (fortune + daimon - 2.0 * 340.0).rem_euclid(360.0);
                assert!(
                    mirrored.abs() < 1e-9 || (mirrored - 360.0).abs() < 1e-9,
                    "{rule:?}"
                );
            }
        }
    }

    #[test]
    fn basis_is_the_shorter_arc_and_has_no_sect() {
        for (sun, moon) in [(10.0, 100.0), (10.0, 300.0), (200.0, 20.0)] {
            let day = sky(sun, moon, 40.0, true, true);
            let night = sky(sun, moon, 40.0, false, true);
            let basis = at(&day, Lot::Basis, LotRequest::VALENS);
            assert_eq!(basis, at(&night, Lot::Basis, LotRequest::VALENS));
            assert!((basis - 40.0).rem_euclid(360.0) <= 180.0);
        }
        // The Moon 90° past the Sun: Fortune at +90°, Daimon at −90°, and
        // the two 180° apart either way.
        assert_eq!(
            at(
                &sky(10.0, 100.0, 40.0, true, true),
                Lot::Basis,
                LotRequest::VALENS
            ),
            220.0
        );
        // 30° past: Fortune +30°, Daimon −30°, the shorter arc 60°.
        assert_eq!(
            at(
                &sky(10.0, 40.0, 40.0, true, true),
                Lot::Basis,
                LotRequest::VALENS
            ),
            100.0
        );
    }

    #[test]
    fn love_and_necessity_are_counted_between_fortune_and_daimon() {
        // By day Fortune is at 70 and Daimon at 10: from Fortune to Daimon
        // is 300, from Daimon to Fortune 60, each counted from 40.
        let day = sky(10.0, 40.0, 40.0, true, true);
        assert_eq!(at(&day, Lot::Love, LotRequest::VALENS), 340.0);
        assert_eq!(at(&day, Lot::Necessity, LotRequest::VALENS), 100.0);
        // By night Fortune and Daimon change places and each arc reverses,
        // so under Valens's Fortune the two reversals cancel.
        let night = sky(10.0, 40.0, 40.0, false, true);
        assert_eq!(at(&night, Lot::Love, LotRequest::VALENS), 340.0);
        assert_eq!(at(&night, Lot::Necessity, LotRequest::VALENS), 100.0);
        // Under Lilly's only the arc reverses, and they swap.
        let lilly = LotRequest::VALENS.with_fortune(FortuneRule::DayAndNight);
        assert_eq!(at(&night, Lot::Love, lilly), 100.0);
        assert_eq!(at(&night, Lot::Necessity, lilly), 340.0);
    }

    #[test]
    fn the_odd_lots_are_counted_as_the_text_gives_them() {
        let day = sky(10.0, 40.0, 40.0, true, true);
        let night = sky(10.0, 40.0, 40.0, false, true);
        // Theft from Saturn (255): Mercury 135 to Mars 225 by day.
        assert_eq!(at(&day, Lot::Theft, LotRequest::VALENS), 345.0);
        assert_eq!(at(&night, Lot::Theft, LotRequest::VALENS), 165.0);
        // The father by night: Venus 100 to the Moon 40, from the ascendant.
        assert_eq!(at(&night, Lot::Father, LotRequest::VALENS), 340.0);
        // Exaltation by day: the Sun 10 to Aries 19; by night the Moon 40
        // to Taurus 3.
        assert_eq!(at(&day, Lot::Exaltation, LotRequest::VALENS), 49.0);
        assert_eq!(at(&night, Lot::Exaltation, LotRequest::VALENS), 33.0);
        // Debt is the same both ways: Mercury 135 to Saturn 255.
        assert_eq!(at(&day, Lot::Debt, LotRequest::VALENS), 160.0);
        assert_eq!(at(&night, Lot::Debt, LotRequest::VALENS), 160.0);
    }

    #[test]
    fn every_lot_read_together_is_each_read_alone() {
        let chart = sky(123.4, 287.6, 201.3, false, true);
        let together = lots(&chart, &Lot::ALL, LotRequest::VALENS).unwrap();
        assert_eq!(together.sect, Sect::Night);
        for (placed, lot) in together.lots.iter().zip(Lot::ALL) {
            assert_eq!(placed.lot, lot);
            assert_eq!(
                placed.place.longitude_deg,
                at(&chart, lot, LotRequest::VALENS)
            );
            assert_eq!(placed.place.sign.attributes().lord, placed.place.lord);
        }
    }

    #[test]
    fn a_formula_of_the_callers_own_reads_the_catalogue() {
        let chart = sky(10.0, 40.0, 40.0, true, true);
        let fortune_again = LotFormula::same(LotArc::new(
            LotPoint::Ascendant,
            LotPoint::Lot(Lot::Fortune),
            LotPoint::Ascendant,
        ));
        let place = lot_place(&chart, &fortune_again, LotRequest::VALENS).unwrap();
        assert_eq!(
            place.longitude_deg,
            at(&chart, Lot::Fortune, LotRequest::VALENS)
        );
    }

    #[test]
    fn a_point_the_sky_cannot_supply_is_refused_by_its_place() {
        let chart = sky(10.0, 40.0, 40.0, false, true);
        let rahu = LotFormula::reversed_by_night(
            LotPoint::Planet(Graha::Rahu),
            LotPoint::Degrees(f64::NAN),
            LotPoint::Ascendant,
        );
        let why = lot_place(&chart, &rahu, LotRequest::VALENS).unwrap_err();
        assert_eq!(why.field(), Some("formula.night.from"));
        let day = sky(10.0, 40.0, 40.0, true, true);
        let why = lot_place(&day, &rahu, LotRequest::VALENS).unwrap_err();
        assert_eq!(why.field(), Some("formula.day.from"));
        let mut broken = chart;
        broken.chart.moon_altitude_deg = f64::NAN;
        let why = lots(&broken, &[Lot::Fortune], LotRequest::VALENS).unwrap_err();
        assert_eq!(why.field(), Some("moon_altitude_deg"));
    }

    #[test]
    fn a_request_reads_and_writes_its_own_record() {
        let asked = LotRequest::VALENS.with_fortune(FortuneRule::ReversedWhileMoonUp);
        let text = serde_json::to_string(&asked).unwrap();
        assert_eq!(
            text,
            r#"{"sectRule":"HORIZON","fortune":"REVERSED_WHILE_MOON_UP"}"#
        );
        assert_eq!(LotRequest::from_json(&text).unwrap(), asked);
        assert_eq!(LotRequest::from_json("{}").unwrap(), LotRequest::VALENS);
    }
}
