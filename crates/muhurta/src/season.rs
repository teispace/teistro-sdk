//! The season: the stretches of the year in which a rite is not held at
//! all (`03-design/muhurta.md` §4.1).
//!
//! A blackout is not a defect weighed against a good day. While Venus is
//! combust no school marries, however good the rest of the day, and a
//! score with enough favourable clauses would hand back that day anyway.
//! So the season is kept apart from the clauses: each blackout a kind and
//! the interval it holds over, found once for a range, and an activity's
//! rules say which kinds it heeds.
//!
//! The intervals are **instants**, not whole days: Pitru paksha begins
//! when the Purnima begins, not at a midnight. A day-level gate reads
//! "any blackout overlaps the day".
//!
//! Everything but asta is the Sun and the Moon: the lunar months from the
//! new moons, their names from the Sun's sign at the opening new moon,
//! their kinds from the sankrantis inside them (the calendar's own rule,
//! `calendar-indian-lunisolar.md` §2), and the tithis that bound Chaturmas
//! and Pitru paksha. Asta is the heliacal events of Jupiter and Venus
//! (`astro::visibility`), paired last-seen to first-seen.

use serde::{Deserialize, Serialize};
use teistro_astro::events::{Longitudes, Search};
use teistro_astro::visibility::{Heliacal, HeliacalEvent, Visibility};
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Masa, Rashi};
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_panchanga::limb::{Sidereal, Zodiac, signs_within};
use teistro_panchanga::span::Span;
use teistro_port_ephemeris::{Body, EphemerisProvider, Lattice, Quantity};

/// Which blackout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum BlackoutKind {
    /// Devshayani to Prabodhini: from the start of the bright eleventh of
    /// (nija) Ashadha to the start of the bright eleventh of (nija)
    /// Kartika, the four months Vishnu sleeps.
    Chaturmas,
    /// An intercalary month, which holds no sankranti: Nepal's Malmas,
    /// also called Purushottam masa (C177).
    AdhikaMasa,
    /// The Sun in Sagittarius or Pisces. Some north Indian calendars call
    /// it Malmas, a word that in Nepal names the adhika month instead
    /// (C177).
    Kharmas,
    /// The dark fortnight of (nija, amanta) Bhadrapada, from the start of
    /// the Purnima it is counted from to the Mahalaya new moon.
    PitruPaksha,
    /// Sixteen ghatis either side of a sankranti (Raman, ch. V, Mahadosha
    /// 2: Surya sankramana).
    Sankranti,
    /// Jupiter unseen, from its last sighting to its next.
    GuruAsta,
    /// Venus unseen: twice a synodic cycle, about its inferior and its
    /// superior conjunction, and both are windows.
    ShukraAsta,
}

/// A blackout and the interval it holds over, clipped to the range asked.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Blackout {
    /// Which.
    pub kind: BlackoutKind,
    /// When.
    pub at: Interval,
}

/// A lunar month: new moon to new moon, named and marked.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Month {
    /// The amanta month the Sun's sign at the opening new moon names.
    pub masa: Masa,
    /// Ordinary, intercalary or omitted.
    pub kind: MonthKind,
    /// From the new moon that opens it to the one that closes it.
    pub at: Interval,
}

/// Sixteen ghatis, days: Raman's span either side of a sankranti.
const SIXTEEN_GHATIS_DAYS: f64 = 16.0 / 60.0;

/// How far before a range the months are found, days. Chaturmas is the
/// longest blackout the months bound, about 118 days from its start to
/// its end, and a range that opens inside it must see the Ashadha that
/// began it: five synodic months reach it with a month to spare.
const MONTHS_BEFORE_DAYS: f64 = 5.0 * 29.530_588_9;

/// How far after a range the months are found, days: one month, so the
/// last month the range touches is closed.
const MONTHS_AFTER_DAYS: f64 = 31.0;

/// The lunar months that open inside a window, each closed by the next
/// new moon.
///
/// # Errors
///
/// The source's own refusal; a sign no masa claims, which the catalogue
/// makes impossible.
pub fn months<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    within: Interval,
) -> Result<Vec<Month>, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let new_moons: Vec<f64> = Search::new(&source, Quantity::ELONGATION, Lattice::single(0.0))
        .between(
            JulianDay::<Ut1>::literal(within.from.get()),
            JulianDay::<Ut1>::literal(within.to.get() + MONTHS_AFTER_DAYS),
        )?
        .iter()
        .map(|event| event.instant.get())
        .collect();
    let (Some(first), Some(last)) = (new_moons.first(), new_moons.last()) else {
        return Ok(Vec::new());
    };
    let sun = signs_within(
        tropical,
        Body::Sun,
        Interval::literal(*first, *last),
        zodiac,
        Some(MONTHS_AFTER_DAYS),
    )?;
    new_moons
        .windows(2)
        .filter(|pair| pair.first().is_some_and(|from| *from < within.to.get()))
        .filter_map(|pair| match pair {
            [from, to] => Some((*from, *to)),
            _ => None,
        })
        .map(|(from, to)| month_of(&sun, from, to))
        .collect()
}

/// The month between two new moons, from the Sun's sign spans.
fn month_of(sun: &[Span<Rashi>], from: f64, to: f64) -> Result<Month, Error> {
    let opening = sun
        .iter()
        .find(|span| span.whole.from.get() <= from && from < span.whole.to.get())
        .ok_or_else(|| Error::internal(format!("no Sun sign spans the new moon at {from}")))?;
    let sankrantis = sun
        .iter()
        .filter(|span| from < span.whole.from.get() && span.whole.from.get() < to)
        .count();
    let kind = MonthKind::of(sankrantis).ok_or_else(|| {
        Error::new(
            Status::Internal,
            format!("the lunar month from {from} to {to} holds {sankrantis} sankrantis"),
        )
    })?;
    let masa = Masa::ALL
        .iter()
        .copied()
        .find(|masa| masa.attributes().solar_sign == opening.member)
        .ok_or_else(|| Error::internal(format!("no masa for {:?}", opening.member)))?;
    Ok(Month {
        masa,
        kind,
        at: interval(from, to)?,
    })
}

/// The blackouts the Sun and the Moon make over a range, in order of
/// their start, each clipped to the range.
///
/// # Errors
///
/// The source's own refusal.
pub fn blackouts<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    range: Interval,
) -> Result<Vec<Blackout>, Error> {
    let lead = interval(range.from.get() - MONTHS_BEFORE_DAYS, range.to.get())?;
    let months = months(tropical, zodiac, lead)?;
    let source = Sidereal::over(tropical, zodiac);
    let mut found = Vec::new();
    for (i, month) in months.iter().enumerate() {
        match (month.masa, month.kind) {
            (_, MonthKind::Adhika) => found.push(blackout(BlackoutKind::AdhikaMasa, month.at)),
            (Masa::Ashadha, MonthKind::Nija) => {
                let kartika = months
                    .iter()
                    .skip(i + 1)
                    .find(|m| m.masa == Masa::Kartika && m.kind == MonthKind::Nija);
                if let Some(kartika) = kartika {
                    let from = elongation_in(&source, month.at, BRIGHT_ELEVENTH_DEG)?;
                    let to = elongation_in(&source, kartika.at, BRIGHT_ELEVENTH_DEG)?;
                    found.push(blackout(BlackoutKind::Chaturmas, interval(from, to)?));
                }
            }
            (Masa::Bhadrapada, MonthKind::Nija) => {
                let from = elongation_in(&source, month.at, PURNIMA_DEG)?;
                found.push(blackout(
                    BlackoutKind::PitruPaksha,
                    interval(from, month.at.to.get())?,
                ));
            }
            _ => {}
        }
    }
    let sun = signs_within(tropical, Body::Sun, range, zodiac, Some(MONTHS_AFTER_DAYS))?;
    for span in &sun {
        if matches!(span.member, Rashi::Sagittarius | Rashi::Pisces) {
            found.push(blackout(BlackoutKind::Kharmas, span.whole));
        }
        let entry = span.whole.from.get();
        found.push(blackout(
            BlackoutKind::Sankranti,
            interval(entry - SIXTEEN_GHATIS_DAYS, entry + SIXTEEN_GHATIS_DAYS)?,
        ));
    }
    Ok(clip_and_order(found, range))
}

/// The elongation at the start of the bright eleventh, degrees.
const BRIGHT_ELEVENTH_DEG: f64 = 120.0;

/// The elongation at the start of the Purnima, the fifteenth tithi.
const PURNIMA_DEG: f64 = 168.0;

/// When the elongation reaches a value inside a month, which it does once.
fn elongation_in<S: Longitudes + ?Sized>(
    source: &Sidereal<'_, S>,
    month: Interval,
    deg: f64,
) -> Result<f64, Error> {
    Search::new(source, Quantity::ELONGATION, Lattice::single(deg))
        .between(
            JulianDay::<Ut1>::literal(month.from.get()),
            JulianDay::<Ut1>::literal(month.to.get()),
        )?
        .first()
        .map(|event| event.instant.get())
        .ok_or_else(|| {
            Error::new(
                Status::NotConverged,
                format!(
                    "the elongation did not reach {deg}° in the month from {}",
                    month.from
                ),
            )
        })
}

/// The asta windows of one body from its heliacal events: each stretch
/// from a last sighting to the next first one, clipped to the range.
///
/// `start` is the body's state on the range's first day; when it was
/// unseen then, it was unseen before the range began too, so the first
/// stretch opens at the range's start. A last sighting with no first
/// after it holds to the range's end.
///
/// Each end is the instant the sighting was read at — the dawn or the
/// dusk the criterion looks at — so a window runs from the last dawn or
/// dusk the body was seen at to the first it is seen at again.
#[must_use]
pub fn asta(
    kind: BlackoutKind,
    start: &Visibility,
    events: &[HeliacalEvent],
    range: Interval,
) -> Vec<Blackout> {
    let mut found = Vec::new();
    let mut hidden_since = (!start.visible).then_some(range.from.get());
    for event in events {
        let read = event.day.instant.get();
        if event.kind.appears() {
            if let Some(from) = hidden_since.take() {
                if let Ok(at) = interval(from, read) {
                    found.push(blackout(kind, at));
                }
            }
        } else if hidden_since.is_none() {
            hidden_since = Some(read);
        }
    }
    if let Some(from) = hidden_since {
        if let Ok(at) = interval(from, range.to.get().max(from)) {
            found.push(blackout(kind, at));
        }
    }
    clip_and_order(found, range)
}

/// Guru or Shukra asta over a range: the body's state on the range's
/// first day and its heliacal events inside it, paired by [`asta`].
///
/// # Errors
///
/// A body other than Jupiter and Venus (`INVALID_ARG`), and whatever the
/// visibility reckoner refuses: a day without a dawn or a dusk, the
/// provider's own refusal.
pub fn asta_over<P: EphemerisProvider + ?Sized>(
    heliacal: &Heliacal<'_, P>,
    body: Body,
    range: Interval,
) -> Result<Vec<Blackout>, Error> {
    let kind = match body {
        Body::Jupiter => BlackoutKind::GuruAsta,
        Body::Venus => BlackoutKind::ShukraAsta,
        other => {
            return Err(Error::invalid_arg(format!(
                "asta is Jupiter's or Venus's, not {other:?}'s"
            ))
            .with_field("body"));
        }
    };
    let from = JulianDay::<Ut1>::literal(range.from.get());
    let to = JulianDay::<Ut1>::literal(range.to.get());
    let start = heliacal.state(body, heliacal.day_start(from))?;
    let events = heliacal.events(body, from, to)?;
    Ok(asta(kind, &start, &events, range))
}

const fn blackout(kind: BlackoutKind, at: Interval) -> Blackout {
    Blackout { kind, at }
}

/// Keeps what overlaps the range, clipped to it, in order of start.
fn clip_and_order(found: Vec<Blackout>, range: Interval) -> Vec<Blackout> {
    let mut kept: Vec<Blackout> = found
        .into_iter()
        .filter_map(|b| b.at.clipped_to(range).map(|at| Blackout { at, ..b }))
        .filter(|b| !b.at.is_empty())
        .collect();
    kept.sort_by(|a, b| a.at.from.get().total_cmp(&b.at.from.get()));
    kept
}

/// An interval from two raw instants.
fn interval(from: f64, to: f64) -> Result<Interval, Error> {
    Interval::new(JulianDay::try_new(from)?, JulianDay::try_new(to)?)
}
