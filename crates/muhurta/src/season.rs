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
//! Everything but asta and the eclipses is the Sun and the Moon: the lunar
//! months from the new moons, their names from the Sun's sign at the
//! opening new moon, their kinds from the sankrantis inside them (the
//! calendar's own rule, `calendar-indian-lunisolar.md` §2), and the tithis
//! that bound Chaturmas and Pitru paksha. Asta is the heliacal events of
//! Jupiter and Venus (`astro::visibility`), paired last-seen to
//! first-seen. The eclipse blackouts read the eclipses the place sees
//! (`muhurta.md` §4.1.1): the eclipse's star for six months after it, and
//! the vedha before it.

use serde::{Deserialize, Serialize};
use teistro_astro::eclipse::{EclipsesHere, LunarKind};
use teistro_astro::events::{Longitudes, Search, value_of};
use teistro_astro::visibility::{Heliacal, HeliacalEvent, Visibility};
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Masa, Rashi};
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_core::settings::EclipseVedha;
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
    /// also called Purushottam masa (C177). The adhika month before a
    /// kshaya month is not one of these; it is [`BlackoutKind::Samsarpa`].
    AdhikaMasa,
    /// The adhika month before a kshaya month, which *Dharmasindhu* calls
    /// samsarpa and holds fit for every rite (p. 3, C179). It is its own
    /// kind so that a rule which closes every adhika month heeds this one
    /// and [`BlackoutKind::AdhikaMasa`] together.
    Samsarpa,
    /// A month holding two sankrantis, whose second name the year skips:
    /// *Dharmasindhu*'s amhaspati, avoided in every rite (p. 3, C179).
    KshayaMasa,
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
    /// The eight days before Holi: from the start of the bright eighth of
    /// (nija, amanta) Phalguna to the full moon (*Shighrabodha* I.137–138).
    /// The text bars marriage and the like in it only on the Shutudri,
    /// the Vipasha and the Iravati and at Tripushkara, and calls it
    /// auspicious elsewhere, so no shipped activity heeds it (C193).
    Holashtaka,
    /// Jupiter unseen, from its last sighting to its next.
    GuruAsta,
    /// Venus unseen: twice a synodic cycle, about its inferior and its
    /// superior conjunction, and both are windows.
    ShukraAsta,
    /// The Moon in the star an eclipse the place saw fell in, for six
    /// synodic months after it (Raman, ch. V, Mahadosha 16: grahanotpatha;
    /// cruxes C189 to C191).
    EclipseStar,
    /// An eclipse's vedha, the almanacs' sutak: from the prahara
    /// *Dharmasindhu* counts back from the eclipse to its end as seen, or
    /// to the body's next rising when it set eclipsed (C192).
    EclipseVedha,
}

impl BlackoutKind {
    /// Every kind, in declaration order. [`BlackoutKind::position`] is
    /// an exhaustive match, so a kind added to the enum and not here fails
    /// to compile there or fails the test that reads this back.
    pub const ALL: [BlackoutKind; 12] = [
        BlackoutKind::Chaturmas,
        BlackoutKind::AdhikaMasa,
        BlackoutKind::Samsarpa,
        BlackoutKind::KshayaMasa,
        BlackoutKind::Kharmas,
        BlackoutKind::PitruPaksha,
        BlackoutKind::Sankranti,
        BlackoutKind::Holashtaka,
        BlackoutKind::GuruAsta,
        BlackoutKind::ShukraAsta,
        BlackoutKind::EclipseStar,
        BlackoutKind::EclipseVedha,
    ];

    /// The kind's place in [`BlackoutKind::ALL`].
    #[must_use]
    pub const fn position(self) -> usize {
        match self {
            BlackoutKind::Chaturmas => 0,
            BlackoutKind::AdhikaMasa => 1,
            BlackoutKind::Samsarpa => 2,
            BlackoutKind::KshayaMasa => 3,
            BlackoutKind::Kharmas => 4,
            BlackoutKind::PitruPaksha => 5,
            BlackoutKind::Sankranti => 6,
            BlackoutKind::Holashtaka => 7,
            BlackoutKind::GuruAsta => 8,
            BlackoutKind::ShukraAsta => 9,
            BlackoutKind::EclipseStar => 10,
            BlackoutKind::EclipseVedha => 11,
        }
    }
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
const MONTHS_BEFORE_DAYS: f64 = 5.0 * SYNODIC_MONTH_DAYS;

/// How far after a range the months are found, days: one month, so the
/// last month the range touches is closed.
const MONTHS_AFTER_DAYS: f64 = 31.0;

/// How many synodic months past a range the season looks for a kshaya
/// month, so that an adhika month inside the range is known to be or not
/// to be the samsarpa before one. The lunisolar pass measures the gap
/// from a kshaya month back to the adhika before it over a millennium
/// (at most five months) and fails if this does not cover it
/// (`calendar-indian-lunisolar-measured.md` §5).
pub const SAMSARPA_REACH_MONTHS: f64 = 6.0;

/// A synodic month, days.
const SYNODIC_MONTH_DAYS: f64 = 29.530_588_9;

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
    let lead = interval(
        range.from.get() - MONTHS_BEFORE_DAYS,
        range.to.get() + SAMSARPA_REACH_MONTHS * SYNODIC_MONTH_DAYS,
    )?;
    let months = months(tropical, zodiac, lead)?;
    let source = Sidereal::over(tropical, zodiac);
    let mut found = Vec::new();
    for (i, month) in months.iter().enumerate() {
        match (month.masa, month.kind) {
            (_, MonthKind::Adhika) => {
                let next = months
                    .iter()
                    .skip(i + 1)
                    .find(|m| m.kind != MonthKind::Nija);
                let kind = if next.is_some_and(|m| m.kind == MonthKind::Kshaya) {
                    BlackoutKind::Samsarpa
                } else {
                    BlackoutKind::AdhikaMasa
                };
                found.push(blackout(kind, month.at));
            }
            (_, MonthKind::Kshaya) => found.push(blackout(BlackoutKind::KshayaMasa, month.at)),
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
            (Masa::Phalguna, MonthKind::Nija) => {
                let from = elongation_in(&source, month.at, BRIGHT_EIGHTH_DEG)?;
                let to = elongation_in(&source, month.at, FULL_MOON_DEG)?;
                found.push(blackout(BlackoutKind::Holashtaka, interval(from, to)?));
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

/// The elongation at the start of the bright eighth tithi, degrees.
const BRIGHT_EIGHTH_DEG: f64 = 84.0;

/// The elongation at the full moon, where the Purnima ends, degrees.
const FULL_MOON_DEG: f64 = 180.0;

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

/// Which eclipse the season reads, as far as the vedha's count asks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeenKind {
    /// A solar eclipse.
    Solar,
    /// A lunar eclipse the umbra covers in part.
    Lunar,
    /// A lunar eclipse the umbra covers whole.
    TotalLunar,
}

/// An eclipse as the season reads it: one the place sees, a lunar one by
/// its umbral phase (`muhurta.md` §4.1.1, crux C190).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeenEclipse {
    /// Which.
    pub kind: SeenKind,
    /// The greatest eclipse, a UT1 Julian day read as UTC.
    pub greatest: f64,
    /// From the first moment the place sees to the last.
    pub seen: Interval,
    /// Whether the eclipse had begun when the body rose.
    pub rose_eclipsed: bool,
    /// Whether it had not ended when the body set.
    pub set_eclipsed: bool,
}

/// How much later than a contact a first or last seen moment must be to be
/// a rising or a setting rather than the contact itself, days: a second.
const SAME_MOMENT_DAYS: f64 = 1.0 / 86_400.0;

impl SeenEclipse {
    /// The eclipses of a window that the place sees, in order of their
    /// greatest moment; a penumbral eclipse, which the eye does not see,
    /// is none of them.
    ///
    /// # Errors
    ///
    /// None in practice: an interval from two finite instants.
    pub fn all_of(found: &EclipsesHere) -> Result<Vec<SeenEclipse>, Error> {
        let mut seen = Vec::new();
        for lunar in &found.lunar {
            let (Some(v), Some(u1), Some(u4)) =
                (lunar.here.umbral_seen, lunar.here.u1, lunar.here.u4)
            else {
                continue;
            };
            seen.push(SeenEclipse {
                kind: if lunar.eclipse.kind == LunarKind::Total {
                    SeenKind::TotalLunar
                } else {
                    SeenKind::Lunar
                },
                greatest: lunar.eclipse.greatest.get(),
                seen: interval(v.from.get(), v.to.get())?,
                rose_eclipsed: v.from.get() > u1.at.get() + SAME_MOMENT_DAYS,
                set_eclipsed: v.to.get() < u4.at.get() - SAME_MOMENT_DAYS,
            });
        }
        for solar in &found.solar {
            let Some(here) = solar.here else { continue };
            let Some(v) = here.seen else { continue };
            seen.push(SeenEclipse {
                kind: SeenKind::Solar,
                greatest: solar.eclipse.greatest.get(),
                seen: interval(v.from.get(), v.to.get())?,
                rose_eclipsed: v.from.get() > here.first.at.get() + SAME_MOMENT_DAYS,
                set_eclipsed: v.to.get() < here.fourth.at.get() - SAME_MOMENT_DAYS,
            });
        }
        seen.sort_by(|a, b| a.greatest.total_cmp(&b.greatest));
        Ok(seen)
    }
}

/// How long an eclipse's star stays barred, in synodic months: Raman's
/// six for a marriage (crux C191).
pub const ECLIPSE_STAR_MONTHS: f64 = 6.0;

/// How far before a range an eclipse can still bar a star inside it, days.
pub const ECLIPSE_STAR_REACH_DAYS: f64 = ECLIPSE_STAR_MONTHS * SYNODIC_MONTH_DAYS;

/// A nakshatra's width, degrees.
const NAKSHATRA_DEG: f64 = 360.0 / 27.0;

/// The stretches the Moon stands in each seen eclipse's star, from the
/// greatest eclipse to [`ECLIPSE_STAR_MONTHS`] after it, clipped to the
/// range (Raman, ch. V, Mahadosha 16; cruxes C189 to C191).
///
/// The star is the Moon's sidereal nakshatra at the greatest eclipse. The
/// passage the eclipse falls in is barred from the eclipse on, and every
/// later passage whole.
///
/// # Errors
///
/// The source's own refusal.
pub fn eclipse_stars<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    eclipses: &[SeenEclipse],
    range: Interval,
) -> Result<Vec<Blackout>, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let moon = Quantity::Longitude(Body::Moon);
    let mut found = Vec::new();
    for eclipse in eclipses {
        let (from, to) = (eclipse.greatest, eclipse.greatest + ECLIPSE_STAR_REACH_DAYS);
        if to <= range.from.get() || from >= range.to.get() {
            continue;
        }
        let at = value_of(moon, &source, JulianDay::<Ut1>::literal(from))?;
        let star = (at.rem_euclid(360.0) / NAKSHATRA_DEG).floor();
        let crossings = |deg: f64| -> Result<Vec<f64>, Error> {
            Ok(Search::new(&source, moon, Lattice::single(deg))
                .between(
                    JulianDay::<Ut1>::literal(from),
                    JulianDay::<Ut1>::literal(to),
                )?
                .iter()
                .map(|event| event.instant.get())
                .collect())
        };
        let entries = crossings(star * NAKSHATRA_DEG)?;
        let exits = crossings(((star + 1.0) * NAKSHATRA_DEG).rem_euclid(360.0))?;
        for start in core::iter::once(from).chain(entries) {
            let end = exits
                .iter()
                .copied()
                .find(|exit| *exit > start)
                .unwrap_or(to);
            found.push(blackout(BlackoutKind::EclipseStar, interval(start, end)?));
        }
    }
    Ok(clip_and_order(found, range))
}

/// The praharas an eclipse's vedha opens before, under a rule
/// (`panchanga.eclipse_vedha`): four for a solar eclipse and for a Moon
/// that rises eclipsed, three for any other lunar eclipse, or four for a
/// total one under `FULL_LUNAR_FOUR`.
#[must_use]
pub fn vedha_praharas(eclipse: &SeenEclipse, rule: EclipseVedha) -> usize {
    let four = eclipse.kind == SeenKind::Solar
        || eclipse.rose_eclipsed
        || (rule == EclipseVedha::FullLunarFour && eclipse.kind == SeenKind::TotalLunar);
    if four { 4 } else { 3 }
}

/// An eclipse's vedha (*Dharmasindhu* p. 28; `muhurta.md` §4.1.1).
///
/// `turns` are the sunrises and sunsets around the eclipse in order,
/// alternating, each pair cut into its four praharas, and must reach far
/// enough back that the praharas counted exist. The vedha opens at the
/// start of the prahara [`vedha_praharas`] before the one holding the
/// first moment seen, and closes when the eclipse ends as seen, or at
/// `next_rising` when the body set eclipsed.
///
/// # Errors
///
/// `INTERNAL` for turns that do not hold the first moment seen, or that
/// begin too late to count back from it.
pub fn eclipse_vedha(
    eclipse: &SeenEclipse,
    turns: &[f64],
    next_rising: f64,
    rule: EclipseVedha,
) -> Result<Blackout, Error> {
    let mut edges = Vec::with_capacity(turns.len() * 4);
    for pair in turns.windows(2) {
        if let [a, b] = pair {
            edges.extend((0..4).map(|q| a + (b - a) * f64::from(q) / 4.0));
        }
    }
    edges.extend(turns.last());
    let first = eclipse.seen.from.get();
    let holding = edges
        .windows(2)
        .position(|pair| matches!(pair, [a, b] if *a <= first && first < *b))
        .ok_or_else(|| {
            Error::internal(format!("no prahara around the eclipse seen from {first}"))
        })?;
    let opens = holding
        .checked_sub(vedha_praharas(eclipse, rule))
        .and_then(|k| edges.get(k).copied())
        .ok_or_else(|| {
            Error::internal(format!(
                "too few praharas before the eclipse seen from {first}"
            ))
        })?;
    let closes = if eclipse.set_eclipsed {
        next_rising
    } else {
        eclipse.seen.to.get()
    };
    Ok(blackout(
        BlackoutKind::EclipseVedha,
        interval(opens, closes)?,
    ))
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

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::float_cmp,
        reason = "tests fail by panicking, and compare quarters of exact halves"
    )]

    #[test]
    fn every_blackout_kind_is_listed_once_in_its_place() {
        for (at, kind) in BlackoutKind::ALL.iter().enumerate() {
            assert_eq!(kind.position(), at, "{kind:?}");
        }
    }

    use super::{BlackoutKind, EclipseVedha, SeenEclipse, SeenKind, eclipse_vedha, vedha_praharas};
    use teistro_core::interval::Interval;

    /// Sunrise at 0.25 and sunset at 0.75 of every day from two days
    /// before day 0 to two after: twelve-hour days, three-hour praharas.
    fn turns() -> Vec<f64> {
        (-2..=2)
            .flat_map(|d| [f64::from(d) + 0.25, f64::from(d) + 0.75])
            .chain([3.25])
            .collect()
    }

    fn eclipse(solar: bool, from: f64) -> SeenEclipse {
        SeenEclipse {
            kind: if solar {
                SeenKind::Solar
            } else {
                SeenKind::Lunar
            },
            greatest: from + 0.05,
            seen: Interval::literal(from, from + 0.1),
            rose_eclipsed: false,
            set_eclipsed: false,
        }
    }

    fn opens(eclipse: &SeenEclipse, rule: EclipseVedha) -> f64 {
        eclipse_vedha(eclipse, &turns(), 9.0, rule)
            .unwrap()
            .at
            .from
            .get()
    }

    /// *Dharmasindhu* p. 28's own examples, each counted back from the
    /// prahara that holds the eclipse.
    #[test]
    fn the_vedha_opens_where_the_texts_examples_open_it() {
        let rule = EclipseVedha::Dharmasindhu;
        // A solar eclipse in the day's first prahara: the night before.
        assert_eq!(opens(&eclipse(true, 0.30), rule), -0.25);
        // In its second: from the night's second prahara.
        assert_eq!(opens(&eclipse(true, 0.40), rule), -0.125);
        // A lunar eclipse in the night's first prahara: from the day's
        // second; in the night's second, from the day's third.
        assert_eq!(opens(&eclipse(false, 0.80), rule), 0.375);
        assert_eq!(opens(&eclipse(false, 0.90), rule), 0.5);
        // A Moon that rises eclipsed: four praharas, the whole day before.
        let risen = SeenEclipse {
            rose_eclipsed: true,
            ..eclipse(false, 0.76)
        };
        assert_eq!(opens(&risen, rule), 0.25);
    }

    #[test]
    fn some_count_four_for_a_total_lunar_eclipse() {
        let total = SeenEclipse {
            kind: SeenKind::TotalLunar,
            ..eclipse(false, 0.80)
        };
        assert_eq!(vedha_praharas(&total, EclipseVedha::Dharmasindhu), 3);
        assert_eq!(vedha_praharas(&total, EclipseVedha::FullLunarFour), 4);
        assert_eq!(opens(&total, EclipseVedha::FullLunarFour), 0.25);
        // A partial one stays at three under either rule.
        assert_eq!(
            opens(&eclipse(false, 0.80), EclipseVedha::FullLunarFour),
            0.375
        );
    }

    #[test]
    fn the_vedha_closes_as_seen_or_at_the_next_rising() {
        let seen = eclipse(false, 0.80);
        let closed = eclipse_vedha(&seen, &turns(), 9.0, EclipseVedha::Dharmasindhu).unwrap();
        assert_eq!(closed.kind, BlackoutKind::EclipseVedha);
        assert_eq!(closed.at.to.get(), seen.seen.to.get());
        let set = SeenEclipse {
            set_eclipsed: true,
            ..seen
        };
        let held = eclipse_vedha(&set, &turns(), 1.8, EclipseVedha::Dharmasindhu).unwrap();
        assert_eq!(held.at.to.get(), 1.8);
    }

    #[test]
    fn turns_that_cannot_count_back_are_refused() {
        // The days begin at the eclipse's own day: no night before it.
        let late: Vec<f64> = turns().into_iter().skip(4).collect();
        assert!(
            eclipse_vedha(&eclipse(true, 0.30), &late, 9.0, EclipseVedha::Dharmasindhu).is_err()
        );
        assert!(
            eclipse_vedha(
                &eclipse(true, 5.0),
                &turns(),
                9.0,
                EclipseVedha::Dharmasindhu
            )
            .is_err()
        );
    }
}
