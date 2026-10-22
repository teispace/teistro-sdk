//! When a vrata's fast is broken (`festival-rules.md` §8.7): the readings
//! a rule lists, each answered for the day after its observance.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Nakshatra;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};

use super::{FestivalDay, Window, check_window};

/// One reading of when a fast kept on a rule's day is broken, on the day
/// after it (§8.7).
///
/// ```
/// use teistro_core::catalogue::Nakshatra;
/// use teistro_panchanga::festival::VrataParana;
///
/// // Janmashtami (Dharmasindhu p. 52): the end of the 8th, and of Rohini
/// // when it joins the 8th.
/// let main = VrataParana::TithiEnd { nakshatra: Some(Nakshatra::Rohini), yamas: None };
/// let json = serde_json::to_string(&main)?;
/// assert_eq!(json, r#"{"kind":"TITHI_END","nakshatra":"ROHINI"}"#);
/// assert_eq!(serde_json::from_str::<VrataParana>(&json)?, main);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum VrataParana {
    /// After the tithi ends, and `nakshatra` too when it joins the tithi
    /// within the fast day, sunrise to sunrise (*Dharmasindhu* p. 52; for
    /// the tithi alone, p. 7). The meal follows the last end when every
    /// one falls by day (before sunset, or already before sunrise), else
    /// the last of those that do, else the first; never before sunrise.
    /// With `yamas`, an end past that many yamas of the daylight keeps
    /// the meal from sunrise to their end instead (p. 7, after Madhava).
    TithiEnd {
        /// The nakshatra whose end also counts when it joins the tithi.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        nakshatra: Option<Nakshatra>,
        /// Yamas of the daylight, 1 to 4, past which the meal is kept in
        /// the morning instead.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        yamas: Option<u8>,
    },
    /// Inside the tithi, from sunrise to its end or to `yamas` yamas of
    /// the daylight, whichever is first: the *Nirnayasindhu*'s Shivaratri,
    /// capped by Madhava's three yamas (*Dharmasindhu* pp. 90–91). A tithi
    /// ended by sunrise leaves the meal open from sunrise.
    Within {
        /// Yamas of the daylight, 1 to 4.
        yamas: u8,
    },
    /// From the day's sunrise: the morning after the festival's puja and
    /// visarjana, for one who cannot wait (*Dharmasindhu* p. 52).
    Sunrise,
    /// From the end of a window on the fast day, as after the nishitha
    /// muhurta. No rank-1 text in hand states it; no shipped rule
    /// carries it.
    AfterWindow {
        /// The window, on the fast day.
        at: Window,
    },
}

impl VrataParana {
    /// Refuses a yama count outside 1 to 4, or a window its own checks
    /// refuse, named under `parana[index]`.
    pub(super) fn check(self, key: &str, index: usize) -> Result<(), Error> {
        let field = format!("parana[{index}]");
        let yamas = match self {
            VrataParana::TithiEnd { yamas, .. } => yamas,
            VrataParana::Within { yamas } => Some(yamas),
            VrataParana::Sunrise => None,
            VrataParana::AfterWindow { at } => {
                return check_window(key, at, &format!("{field}.at"));
            }
        };
        if let Some(yamas) = yamas.filter(|yamas| !(1..=4).contains(yamas)) {
            return Err(Error::invalid_arg(format!(
                "{key}: a daylight has four yamas, so a parana counts 1 to 4 of them, not {yamas}"
            ))
            .with_field(format!("{field}.yamas")));
        }
        Ok(())
    }
}

/// What set an edge of a vrata's parana.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VrataEdge {
    /// The parana day's sunrise.
    Sunrise,
    /// The tithi's end.
    TithiEnd,
    /// The joined nakshatra's end.
    NakshatraEnd,
    /// The end of the yamas the reading counts.
    Yamas,
    /// The end of the reading's window on the fast day.
    WindowEnd,
}

/// When a fast is broken under one reading.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct VrataParanaTime {
    /// The reading.
    pub reading: VrataParana,
    /// The day after the observance, whose sunrise and tithi the reading
    /// reads.
    pub day: CalendarDate,
    /// When the meal may begin.
    pub from: JulianDay<Utc>,
    /// What set it.
    pub from_by: VrataEdge,
    /// When it must be taken by, for a reading with a window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<JulianDay<Utc>>,
    /// What set that.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_by: Option<VrataEdge>,
}

/// Each reading answered for a fast kept on `fast`, its tithi `tithi`,
/// broken on `next`, or why it cannot be.
pub(super) fn paranas(
    readings: &[VrataParana],
    tithi: Interval,
    fast: &FestivalDay,
    next: &FestivalDay,
) -> Result<Vec<VrataParanaTime>, String> {
    if !next.normal {
        return Err(format!(
            "{} is a polar day, which has no windows",
            next.date
        ));
    }
    let (rise, set) = (next.sunrise.get(), next.sunset.get());
    let yamas_end = |yamas: u8| (set - rise).mul_add(f64::from(yamas) / 4.0, rise);
    let time = |reading, from: f64, from_by, to: Option<(f64, VrataEdge)>| VrataParanaTime {
        reading,
        day: next.date.clone(),
        from: JulianDay::literal(from),
        from_by,
        to: to.map(|(to, _)| JulianDay::literal(to)),
        to_by: to.map(|(_, by)| by),
    };
    readings
        .iter()
        .map(|&reading| {
            Ok(match reading {
                VrataParana::TithiEnd { nakshatra, yamas } => {
                    let mut ends = vec![(tithi.to.get(), VrataEdge::TithiEnd)];
                    if let Some(end) =
                        nakshatra.and_then(|nakshatra| joined(nakshatra, tithi, fast, next))
                    {
                        ends.push((end, VrataEdge::NakshatraEnd));
                    }
                    let (from, by) = after_ends(&ends, rise, set);
                    match yamas.map(yamas_end) {
                        Some(cap) if from > cap => time(
                            reading,
                            rise,
                            VrataEdge::Sunrise,
                            Some((cap, VrataEdge::Yamas)),
                        ),
                        _ => time(reading, from, by, None),
                    }
                }
                VrataParana::Within { yamas } => {
                    let end = tithi.to.get();
                    if end <= rise {
                        time(reading, rise, VrataEdge::Sunrise, None)
                    } else {
                        let cap = yamas_end(yamas);
                        let to = if end < cap {
                            (end, VrataEdge::TithiEnd)
                        } else {
                            (cap, VrataEdge::Yamas)
                        };
                        time(reading, rise, VrataEdge::Sunrise, Some(to))
                    }
                }
                VrataParana::Sunrise => time(reading, rise, VrataEdge::Sunrise, None),
                VrataParana::AfterWindow { at } => {
                    let end = fast.window(at).map_err(|error| error.to_string())?.to.get();
                    time(reading, end, VrataEdge::WindowEnd, None)
                }
            })
        })
        .collect()
}

/// When the meal may begin after these ends (§8.7): the last, when every
/// one falls by day; the last of those that do, when some do; the first,
/// when none does; never before sunrise.
fn after_ends(ends: &[(f64, VrataEdge)], rise: f64, set: f64) -> (f64, VrataEdge) {
    let by_day: Vec<_> = ends
        .iter()
        .copied()
        .filter(|&(end, _)| end <= set)
        .collect();
    let latest =
        |ends: &[(f64, VrataEdge)]| ends.iter().copied().max_by(|a, b| a.0.total_cmp(&b.0));
    let chosen = if by_day.is_empty() {
        ends.iter().copied().min_by(|a, b| a.0.total_cmp(&b.0))
    } else {
        latest(&by_day)
    };
    match chosen {
        Some((end, by)) if end > rise => (end, by),
        _ => (rise, VrataEdge::Sunrise),
    }
}

/// The end of `nakshatra`'s span when it joins the tithi within the fast
/// day, its sunrise to the next.
fn joined(
    nakshatra: Nakshatra,
    tithi: Interval,
    fast: &FestivalDay,
    next: &FestivalDay,
) -> Option<f64> {
    let day = Interval::literal(fast.sunrise.get(), fast.next_sunrise.get());
    let within = tithi.clipped_to(day)?;
    fast.nakshatra
        .iter()
        .chain(&next.nakshatra)
        .filter(|span| span.member == nakshatra)
        .find(|span| {
            span.whole
                .clipped_to(within)
                .is_some_and(|both| both.days() > 0.0)
        })
        .map(|span| span.whole.to.get())
}
