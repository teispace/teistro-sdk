//! When a vrata's fast is broken (`festival-rules.md` §8.7): the readings
//! a rule lists, each answered for the day the meal falls on.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Nakshatra;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};

use super::{FestivalDay, Window, check_window};

/// One reading of when a fast kept on a rule's day is broken (§8.7).
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
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// In the tithi after the fast's, from its first daylight: the fast
    /// tithi's end when it falls by day, on the fast day or the next, else
    /// the sunrise after it (*Nirnayasindhu* p. 139, "सा च दशम्यां कार्या",
    /// the Navaratri's parana on the 10th).
    NextTithi,
    /// After a closing rite another rule keeps, on that rule's next day
    /// from the fast's, from the rite's first instant: its day's sunrise,
    /// its tithi's start, and the start of the share of a nakshatra that
    /// chose its day, whichever is last (*Dharmasindhu* pp. 70–71: the
    /// vow is given up on the visarjana's day, after it). With `remnant`,
    /// when the rite falls later than the day after the fast and the
    /// fast's tithi still stands at that day's sunrise for at least that
    /// many ghatis, the meal is kept inside the tithi on that day instead,
    /// sunrise to its end (DS p. 71, the remaining 9th).
    AfterRite {
        /// The closing rite's rule key, which the same request must ask
        /// for, as `"DURGA_VISARJANA"`.
        rite: String,
        /// Ghatis of the fast's tithi, 0 to 60, past the next sunrise that
        /// keep the meal inside it; none to wait for the rite always.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        remnant: Option<u8>,
    },
}

impl VrataParana {
    /// Whether the reading waits for another rule's rite.
    #[must_use]
    pub const fn follows_a_rite(&self) -> bool {
        matches!(self, VrataParana::AfterRite { .. })
    }

    /// Refuses a yama count outside 1 to 4, a window its own checks
    /// refuse, a rite named by no key or by the rule's own, or a remnant
    /// past a day's sixty ghatis, named under `parana[index]`.
    pub(super) fn check(&self, key: &str, index: usize) -> Result<(), Error> {
        let field = format!("parana[{index}]");
        let yamas = match *self {
            VrataParana::TithiEnd { yamas, .. } => yamas,
            VrataParana::Within { yamas } => Some(yamas),
            VrataParana::Sunrise | VrataParana::NextTithi => None,
            VrataParana::AfterWindow { at } => {
                return check_window(key, at, &format!("{field}.at"));
            }
            VrataParana::AfterRite { ref rite, remnant } => {
                if rite.trim().is_empty() || rite == key {
                    return Err(Error::invalid_arg(format!(
                        "{key}: a parana after a rite names another rule's key, not {rite:?}"
                    ))
                    .with_field(format!("{field}.rite")));
                }
                if let Some(remnant) = remnant.filter(|ghatis| *ghatis > 60) {
                    return Err(Error::invalid_arg(format!(
                        "{key}: a day has sixty ghatis, so a remnant counts 0 to 60 of them, not {remnant}"
                    ))
                    .with_field(format!("{field}.remnant")));
                }
                return Ok(());
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
    /// The closing rite's first instant.
    Rite,
}

/// When a fast is broken under one reading.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct VrataParanaTime {
    /// The reading.
    pub reading: VrataParana,
    /// The day the meal falls on, sunrise to the next: the day after the
    /// observance for most readings, the fast day itself for one that ends
    /// on it, and the rite's day after a rite.
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

/// A closing rite's day and first instant, as [`VrataParana::AfterRite`]
/// reads it.
#[derive(Clone, Copy, Debug)]
pub(super) struct RiteAt<'a> {
    pub(super) day: &'a FestivalDay,
    pub(super) at: f64,
}

/// Each reading answered for a fast whose tithi is `tithi`, over `from`,
/// the fast's day and the days after it, with `rite_at` the next day of a
/// rite a reading names; or why it cannot be.
pub(super) fn paranas<'a>(
    readings: &[VrataParana],
    tithi: Interval,
    from: &[FestivalDay],
    rite_at: &dyn Fn(&str) -> Option<RiteAt<'a>>,
) -> Result<Vec<VrataParanaTime>, String> {
    let [fast, next, ..] = from else {
        return Err("the days do not hold the day after the observance, its parana".to_owned());
    };
    if !next.normal {
        return Err(format!(
            "{} is a polar day, which has no windows",
            next.date
        ));
    }
    let (rise, set) = (next.sunrise.get(), next.sunset.get());
    let yamas_end = |yamas: u8| (set - rise).mul_add(f64::from(yamas) / 4.0, rise);
    let days = from;
    let time = |reading: &VrataParana, from: f64, from_by, to: Option<(f64, VrataEdge)>| {
        time(days, fast, reading, from, from_by, to)
    };
    readings
        .iter()
        .map(|reading| {
            Ok(match *reading {
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
                VrataParana::NextTithi => {
                    let (from, by) = next_tithi(days, tithi)?;
                    time(reading, from, by, None)
                }
                VrataParana::AfterRite {
                    ref rite,
                    remnant,
                } => {
                    let Some(kept) = rite_at(rite) else {
                        return Err(format!(
                            "the days hold no day of {rite} after the fast, the rite its parana follows"
                        ));
                    };
                    after_rite(next, reading, kept, tithi, remnant)
                }
            })
        })
        .collect()
}

/// A reading answered from `from`: its day is the sunrise day that
/// instant falls in, among `days`, the fast's day and those after it.
fn time(
    days: &[FestivalDay],
    fast: &FestivalDay,
    reading: &VrataParana,
    from: f64,
    from_by: VrataEdge,
    to: Option<(f64, VrataEdge)>,
) -> VrataParanaTime {
    let day = days
        .iter()
        .rfind(|day| day.sunrise.get() <= from)
        .unwrap_or(fast);
    VrataParanaTime {
        reading: reading.clone(),
        day: day.date.clone(),
        from: JulianDay::literal(from),
        from_by,
        to: to.map(|(to, _)| JulianDay::literal(to)),
        to_by: to.map(|(_, by)| by),
    }
}

/// [`VrataParana::NextTithi`]: the first daylight after the fast's tithi
/// ends, over `days` from the fast's.
fn next_tithi(days: &[FestivalDay], tithi: Interval) -> Result<(f64, VrataEdge), String> {
    let end = tithi.to.get();
    let ended_in = days
        .iter()
        .find(|day| day.sunrise.get() <= end && end < day.next_sunrise.get());
    match ended_in {
        // By day, its end; at night, the next sunrise, which the days
        // after must hold.
        Some(day) if end <= day.sunset.get() => Ok((end, VrataEdge::TithiEnd)),
        Some(day) if days.last().is_some_and(|last| !std::ptr::eq(last, day)) => {
            Ok((day.next_sunrise.get(), VrataEdge::Sunrise))
        }
        None => match days.first() {
            Some(fast) if end < fast.sunrise.get() => Ok((fast.sunrise.get(), VrataEdge::Sunrise)),
            _ => Err("the days do not reach the sunrise after the fast's tithi ends".to_owned()),
        },
        Some(_) => Err("the days do not reach the sunrise after the fast's tithi ends".to_owned()),
    }
}

/// [`VrataParana::AfterRite`] once the rite's day is `kept`, `next` the
/// day after the fast: after the rite, or
/// inside the fast's tithi left at the next sunrise when the rite falls
/// later and `remnant` ghatis of it stand.
fn after_rite(
    next: &FestivalDay,
    reading: &VrataParana,
    kept: RiteAt<'_>,
    tithi: Interval,
    remnant: Option<u8>,
) -> VrataParanaTime {
    if let Some(ghatis) = remnant {
        let rise = next.sunrise.get();
        let left = tithi.to.get() - rise;
        let ghati = (next.next_sunrise.get() - rise) / 60.0;
        if kept.day.sunrise.get() > rise && left > 0.0 && left >= f64::from(ghatis) * ghati {
            return time(
                &[],
                next,
                reading,
                rise,
                VrataEdge::Sunrise,
                Some((tithi.to.get(), VrataEdge::TithiEnd)),
            );
        }
    }
    VrataParanaTime {
        reading: reading.clone(),
        day: kept.day.date.clone(),
        from: JulianDay::literal(kept.at),
        from_by: VrataEdge::Rite,
        to: None,
        to_by: None,
    }
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
