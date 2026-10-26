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
use teistro_astro::eclipse::{EclipsesHere, LunarEclipseKind};
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

/// Which blackout: the catalogue's `blackout_kind`, each member with its
/// source.
pub use teistro_core::catalogue::BlackoutKind;

/// A blackout and the interval it holds over, clipped to the range asked.
///
/// ```
/// # use teistro_astro::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::settings::{AyanamshaBasis, OverridePolicy};
/// # use teistro_ephemeris_builtin::provider::Builtin;
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac::of(
/// #     Some(Ayanamsha::Lahiri.into()),
/// #     AyanamshaBasis::True,
/// #     PrecessionModel::Vondrak2011,
/// #     DeltaTModel::TableThenModel,
/// # );
/// use teistro_muhurta::season::{BlackoutKind, blackouts};
///
/// // 2026-09-01 to 2026-12-01, 00:00 UTC.
/// let autumn = Interval::literal(2_461_284.5, 2_461_375.5);
/// let found = blackouts(&longitudes, zodiac, autumn)?;
/// // Pitru paksha 2026 opens on the Purnima of 26 September (civil date).
/// let pitru = found.iter().find(|b| b.kind == BlackoutKind::PitruPaksha).ok_or("found")?;
/// assert!((2_461_308.5..2_461_310.5).contains(&pitru.at.from.get()));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Blackout {
    /// Which.
    pub kind: BlackoutKind,
    /// When.
    pub at: Interval,
}

/// A lunar month: new moon to new moon, named and marked.
///
/// ```
/// # use teistro_astro::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::settings::{AyanamshaBasis, OverridePolicy};
/// # use teistro_ephemeris_builtin::provider::Builtin;
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac::of(
/// #     Some(Ayanamsha::Lahiri.into()),
/// #     AyanamshaBasis::True,
/// #     PrecessionModel::Vondrak2011,
/// #     DeltaTModel::TableThenModel,
/// # );
/// use teistro_calendar::lunisolar::MonthKind;
/// use teistro_core::catalogue::Masa;
/// use teistro_muhurta::season::months;
///
/// // 2026-05-01 to 2026-07-01, 00:00 UTC.
/// let found = months(&longitudes, zodiac, Interval::literal(2_461_161.5, 2_461_222.5))?;
/// // 2026's Jyeshtha is adhika, from the new moon of 16 May.
/// let adhika = found.iter().find(|m| m.kind == MonthKind::Adhika).ok_or("found")?;
/// assert_eq!(adhika.masa, Masa::Jyeshtha);
/// assert!((adhika.at.from.get() - 2_461_177.334).abs() < 0.01);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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

pub use teistro_calendar::lunisolar::SAMSARPA_REACH_MONTHS;

/// A synodic month, days.
const SYNODIC_MONTH_DAYS: f64 = 29.530_588_9;

/// The lunar months that open inside a window, each closed by the next
/// new moon.
///
/// ```
/// # use teistro_astro::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::settings::{AyanamshaBasis, OverridePolicy};
/// # use teistro_ephemeris_builtin::provider::Builtin;
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac::of(
/// #     Some(Ayanamsha::Lahiri.into()),
/// #     AyanamshaBasis::True,
/// #     PrecessionModel::Vondrak2011,
/// #     DeltaTModel::TableThenModel,
/// # );
/// use teistro_calendar::lunisolar::MonthKind;
/// use teistro_core::catalogue::Masa;
/// use teistro_muhurta::season::months;
///
/// // 2026-05-01 to 2026-07-01, 00:00 UTC.
/// let found = months(&longitudes, zodiac, Interval::literal(2_461_161.5, 2_461_222.5))?;
/// // The months tile the span without a gap.
/// assert!(found.windows(2).all(|pair| pair[0].at.to == pair[1].at.from));
/// # Ok::<(), teistro_core::Error>(())
/// ```
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
/// ```
/// # use teistro_astro::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::settings::{AyanamshaBasis, OverridePolicy};
/// # use teistro_ephemeris_builtin::provider::Builtin;
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac::of(
/// #     Some(Ayanamsha::Lahiri.into()),
/// #     AyanamshaBasis::True,
/// #     PrecessionModel::Vondrak2011,
/// #     DeltaTModel::TableThenModel,
/// # );
/// use teistro_muhurta::season::{BlackoutKind, blackouts};
///
/// // 2026-09-01 to 2026-12-01, 00:00 UTC.
/// let autumn = Interval::literal(2_461_284.5, 2_461_375.5);
/// let found = blackouts(&longitudes, zodiac, autumn)?;
/// // Chaturmas closes on Kartika's bright eleventh, 20 November 2026.
/// assert!(found.iter().any(|b| b.kind == BlackoutKind::Chaturmas
///     && (2_461_363.5..2_461_365.5).contains(&b.at.to.get())));
/// assert!(found.windows(2).all(|pair| pair[0].at.from.get() <= pair[1].at.from.get()));
/// # Ok::<(), teistro_core::Error>(())
/// ```
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
        range.to.get() + f64::from(SAMSARPA_REACH_MONTHS) * SYNODIC_MONTH_DAYS,
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
///
/// ```
/// # use teistro_astro::visibility::{HeliacalEvent, HeliacalKind, Motion, Side, Visibility};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_port_ephemeris::Body;
/// #
/// # let state = |at: f64, visible: bool| Visibility {
/// #     body: Body::Venus,
/// #     day_start: JulianDay::literal(at.floor() - 0.5),
/// #     instant: JulianDay::literal(at),
/// #     side: Side::East,
/// #     motion: Motion::Direct,
/// #     measure_deg: 0.0,
/// #     threshold_deg: 0.0,
/// #     visible,
/// #     evaluations: 0,
/// # };
/// # let event = |kind, at| HeliacalEvent {
/// #     kind,
/// #     day: state(at, kind == HeliacalKind::MorningFirst || kind == HeliacalKind::EveningFirst),
/// # };
/// # let events = [
/// #     event(HeliacalKind::EveningFirst, 110.2),
/// #     event(HeliacalKind::EveningLast, 150.8),
/// #     event(HeliacalKind::MorningFirst, 160.2),
/// #     event(HeliacalKind::MorningLast, 250.3),
/// # ];
/// # let range = Interval::literal(100.0, 300.0);
/// use teistro_muhurta::season::{BlackoutKind, asta};
///
/// // Unseen at the start: the first stretch opens at the range's start, and
/// // the last runs to its end.
/// let found = asta(BlackoutKind::ShukraAsta, &state(100.2, false), &events, range);
/// let spans: Vec<(f64, f64)> = found.iter().map(|b| (b.at.from.get(), b.at.to.get())).collect();
/// assert_eq!(spans, [(100.0, 110.2), (150.8, 160.2), (250.3, 300.0)]);
/// // Seen at the start, the first appearance closes nothing.
/// assert_eq!(asta(BlackoutKind::ShukraAsta, &state(100.2, true), &events, range).len(), 2);
/// ```
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
/// ```
/// # use teistro_astro::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::settings::{AyanamshaBasis, OverridePolicy};
/// # use teistro_ephemeris_builtin::provider::Builtin;
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac::of(
/// #     Some(Ayanamsha::Lahiri.into()),
/// #     AyanamshaBasis::True,
/// #     PrecessionModel::Vondrak2011,
/// #     DeltaTModel::TableThenModel,
/// # );
///
/// # use teistro_astro::visibility::{Criterion, Heliacal};
/// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// # use teistro_port_ephemeris::Horizon;
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let heliacal = Heliacal::new(&completion, kathmandu, Criterion::SURYA_SIDDHANTA, Horizon::CENTRE_NO_REFRACTION, DeltaTModel::TableThenModel);
/// use teistro_muhurta::season::{BlackoutKind, asta_over};
/// use teistro_port_ephemeris::Body;
///
/// // Venus meets the Sun once in October and November 2026, unseen from Kathmandu.
/// let autumn = Interval::literal(2_461_314.5, 2_461_375.5);
/// let found = asta_over(&heliacal, Body::Venus, autumn)?;
/// assert_eq!(found.len(), 1);
/// assert_eq!(found[0].kind, BlackoutKind::ShukraAsta);
/// // Asta is Jupiter's or Venus's alone.
/// assert_eq!(asta_over(&heliacal, Body::Mars, autumn).unwrap_err().field(), Some("body"));
/// # Ok::<(), teistro_core::Error>(())
/// ```
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
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_muhurta::season::{SeenEclipse, SeenKind};
///
/// // A lunar eclipse seen from a tenth of the way into a night: twelve-hour
/// // days, sunrise at 0.25 and sunset at 0.75 of each.
/// let eclipse = SeenEclipse {
///     kind: SeenKind::Lunar,
///     greatest: 0.85,
///     seen: Interval::literal(0.80, 0.90),
///     rose_eclipsed: false,
///     set_eclipsed: false,
/// };
/// use teistro_core::settings::EclipseVedha;
/// use teistro_muhurta::season::vedha_praharas;
///
/// // A total lunar eclipse counts four under the rule that says so.
/// let total = SeenEclipse { kind: SeenKind::TotalLunar, ..eclipse };
/// assert_eq!(vedha_praharas(&total, EclipseVedha::FullLunarFour), 4);
/// assert_eq!(vedha_praharas(&total, EclipseVedha::Dharmasindhu), 3);
/// ```
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
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_muhurta::season::{SeenEclipse, SeenKind};
///
/// // A lunar eclipse seen from a tenth of the way into a night: twelve-hour
/// // days, sunrise at 0.25 and sunset at 0.75 of each.
/// let eclipse = SeenEclipse {
///     kind: SeenKind::Lunar,
///     greatest: 0.85,
///     seen: Interval::literal(0.80, 0.90),
///     rose_eclipsed: false,
///     set_eclipsed: false,
/// };
/// use teistro_core::settings::EclipseVedha;
/// use teistro_muhurta::season::vedha_praharas;
///
/// // A Moon that rises eclipsed counts four praharas back, not three.
/// let risen = SeenEclipse { rose_eclipsed: true, ..eclipse };
/// assert_eq!(vedha_praharas(&risen, EclipseVedha::Dharmasindhu), 4);
/// ```
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
    /// ```
    /// use teistro_astro::eclipse::EclipsesHere;
    /// use teistro_muhurta::season::SeenEclipse;
    ///
    /// // No eclipse found, none seen.
    /// assert!(SeenEclipse::all_of(&EclipsesHere::default())?.is_empty());
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
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
                kind: if lunar.eclipse.kind == LunarEclipseKind::Total {
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
///
/// ```
/// use teistro_muhurta::season::{ECLIPSE_STAR_MONTHS, ECLIPSE_STAR_REACH_DAYS};
///
/// assert_eq!(ECLIPSE_STAR_MONTHS, 6.0);
/// assert!((ECLIPSE_STAR_REACH_DAYS - 177.18).abs() < 0.01);
/// ```
pub const ECLIPSE_STAR_MONTHS: f64 = 6.0;

/// How far before a range an eclipse can still bar a star inside it, days.
///
/// ```
/// use teistro_muhurta::season::ECLIPSE_STAR_REACH_DAYS;
///
/// // Six synodic months, under half a year.
/// assert!(ECLIPSE_STAR_REACH_DAYS < 182.6);
/// ```
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
/// ```
/// # use teistro_astro::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::settings::{AyanamshaBasis, OverridePolicy};
/// # use teistro_ephemeris_builtin::provider::Builtin;
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::Frame;
/// #
/// # let provider = Builtin::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame::CANONICAL);
/// # let zodiac = Zodiac::of(
/// #     Some(Ayanamsha::Lahiri.into()),
/// #     AyanamshaBasis::True,
/// #     PrecessionModel::Vondrak2011,
/// #     DeltaTModel::TableThenModel,
/// # );
/// use teistro_muhurta::season::{BlackoutKind, SeenEclipse, SeenKind, eclipse_stars};
///
/// // The partial lunar eclipse of 28 August 2026: each month after, the Moon
/// // returns to its star for about a day.
/// let eclipse = SeenEclipse {
///     kind: SeenKind::Lunar,
///     greatest: 2_461_280.676,
///     seen: Interval::literal(2_461_280.6, 2_461_280.75),
///     rose_eclipsed: false,
///     set_eclipsed: false,
/// };
/// let autumn = Interval::literal(2_461_284.5, 2_461_375.5);
/// let found = eclipse_stars(&longitudes, zodiac, &[eclipse], autumn)?;
/// assert_eq!(found.len(), 3);
/// assert!(found.iter().all(|b| b.kind == BlackoutKind::EclipseStar && (0.8..1.4).contains(&b.at.days())));
/// # Ok::<(), teistro_core::Error>(())
/// ```
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
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_muhurta::season::{SeenEclipse, SeenKind};
///
/// // A lunar eclipse seen from a tenth of the way into a night: twelve-hour
/// // days, sunrise at 0.25 and sunset at 0.75 of each.
/// let eclipse = SeenEclipse {
///     kind: SeenKind::Lunar,
///     greatest: 0.85,
///     seen: Interval::literal(0.80, 0.90),
///     rose_eclipsed: false,
///     set_eclipsed: false,
/// };
/// use teistro_core::settings::EclipseVedha;
/// use teistro_muhurta::season::vedha_praharas;
///
/// assert_eq!(vedha_praharas(&eclipse, EclipseVedha::Dharmasindhu), 3);
/// let solar = SeenEclipse { kind: SeenKind::Solar, ..eclipse };
/// assert_eq!(vedha_praharas(&solar, EclipseVedha::Dharmasindhu), 4);
/// ```
#[must_use]
pub fn vedha_praharas(eclipse: &SeenEclipse, rule: EclipseVedha) -> u8 {
    let solar = eclipse.kind == SeenKind::Solar;
    let four = match rule {
        EclipseVedha::FixedHours => solar,
        EclipseVedha::FullLunarFour => {
            solar || eclipse.rose_eclipsed || eclipse.kind == SeenKind::TotalLunar
        }
        _ => solar || eclipse.rose_eclipsed,
    };
    if four { 4 } else { 3 }
}

/// A prahara under [`EclipseVedha::FixedHours`]: three hours, days.
const FIXED_PRAHARA_DAYS: f64 = 3.0 / 24.0;

/// An eclipse's vedha (*Dharmasindhu* p. 28; `muhurta.md` §4.1.1).
///
/// `turns` are the sunrises and sunsets around the eclipse in order,
/// alternating, each pair cut into its four praharas, and must reach far
/// enough back that the praharas counted exist. The vedha opens at the
/// start of the prahara [`vedha_praharas`] before the one holding the
/// first moment seen, and closes when the eclipse ends as seen, or at
/// `next_rising` when the body set eclipsed. Under
/// [`EclipseVedha::FixedHours`] the praharas are three hours each,
/// counted back from the first moment seen itself, and `turns` is not
/// read.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_muhurta::season::{SeenEclipse, SeenKind};
///
/// // A lunar eclipse seen from a tenth of the way into a night: twelve-hour
/// // days, sunrise at 0.25 and sunset at 0.75 of each.
/// let eclipse = SeenEclipse {
///     kind: SeenKind::Lunar,
///     greatest: 0.85,
///     seen: Interval::literal(0.80, 0.90),
///     rose_eclipsed: false,
///     set_eclipsed: false,
/// };
/// let turns: Vec<f64> = (-2..=2).flat_map(|d| [f64::from(d) + 0.25, f64::from(d) + 0.75]).chain([3.25]).collect();
/// use teistro_core::settings::EclipseVedha;
/// use teistro_muhurta::season::{BlackoutKind, eclipse_vedha};
///
/// // Seen in the night's first prahara: the vedha opens at the day's second
/// // (Dharmasindhu p. 28) and closes as the eclipse ends.
/// let vedha = eclipse_vedha(&eclipse, &turns, 9.0, EclipseVedha::Dharmasindhu)?;
/// assert_eq!(vedha.kind, BlackoutKind::EclipseVedha);
/// assert_eq!((vedha.at.from.get(), vedha.at.to.get()), (0.375, 0.90));
/// # Ok::<(), teistro_core::Error>(())
/// ```
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
    let closes = if eclipse.set_eclipsed {
        next_rising
    } else {
        eclipse.seen.to.get()
    };
    if rule == EclipseVedha::FixedHours {
        let count = f64::from(vedha_praharas(eclipse, rule));
        return Ok(blackout(
            BlackoutKind::EclipseVedha,
            interval(first - count * FIXED_PRAHARA_DAYS, closes)?,
        ));
    }
    let holding = edges
        .windows(2)
        .position(|pair| matches!(pair, [a, b] if *a <= first && first < *b))
        .ok_or_else(|| {
            Error::internal(format!("no prahara around the eclipse seen from {first}"))
        })?;
    let opens = holding
        .checked_sub(usize::from(vedha_praharas(eclipse, rule)))
        .and_then(|k| edges.get(k).copied())
        .ok_or_else(|| {
            Error::internal(format!(
                "too few praharas before the eclipse seen from {first}"
            ))
        })?;
    Ok(blackout(
        BlackoutKind::EclipseVedha,
        interval(opens, closes)?,
    ))
}

/// How many civil days either side of the one holding an eclipse's first
/// moment seen [`vedha_over_days`] reads: four praharas back and a rising
/// after a body set eclipsed both lie within two.
pub const VEDHA_DAYS_AROUND: i64 = 2;

/// An eclipse's vedha read off the almanac days around it: their sunrises
/// and sunsets are the turns [`eclipse_vedha`] counts praharas in, and the
/// next sunrise, or the Moon's next rise, after the eclipse ends as seen
/// is where a body that set eclipsed closes it. `days` are the civil days
/// from [`VEDHA_DAYS_AROUND`] before the one holding the first moment seen
/// to as many after, in order; what the muhurta search and the almanac's
/// eclipses both read.
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = TestProvider;
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let date = |day| CalendarDate::defined(Calendar::Gregorian, 2026, 11, day);
/// use teistro_core::settings::EclipseVedha;
/// use teistro_muhurta::season::{SeenEclipse, SeenKind, vedha_over_days};
///
/// // Five days around 25 November 2026, and a lunar eclipse seen from the
/// // middle day's evening for an hour.
/// let days = (23..=27)
///     .map(|day| Ok(almanac.day(&date(day), &kathmandu)?.value))
///     .collect::<Result<Vec<_>, teistro_core::Error>>()?;
/// let evening = days[2].day.sunset.get() + 0.05;
/// let eclipse = SeenEclipse {
///     kind: SeenKind::Lunar,
///     greatest: evening + 0.02,
///     seen: Interval::literal(evening, evening + 0.04),
///     rose_eclipsed: false,
///     set_eclipsed: false,
/// };
/// let vedha = vedha_over_days(&eclipse, &days, EclipseVedha::Dharmasindhu)?;
/// // Three praharas before the night's first, so from the day's second.
/// assert!(vedha.at.from.get() > days[2].day.sunrise.get());
/// assert_eq!(vedha.at.to.get(), evening + 0.04);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// `INTERNAL` for days that do not reach far enough round the eclipse.
pub fn vedha_over_days(
    eclipse: &SeenEclipse,
    days: &[teistro_panchanga::Panchanga],
    rule: EclipseVedha,
) -> Result<Blackout, Error> {
    let mut turns: Vec<f64> = days
        .iter()
        .flat_map(|p| [p.day.sunrise.get(), p.day.sunset.get()])
        .collect();
    turns.extend(days.last().map(|p| p.day.next_sunrise.get()));
    let after = eclipse.seen.to.get();
    let rising = if eclipse.kind == SeenKind::Solar {
        days.iter()
            .map(|p| p.day.next_sunrise.get())
            .find(|t| *t > after)
    } else {
        days.iter()
            .flat_map(|p| p.moon.rises.iter().map(|r| r.get()))
            .find(|t| *t > after)
    };
    // Asked only when the body set eclipsed; otherwise the end is seen.
    let next_rising = match rising {
        Some(at) => at,
        None if !eclipse.set_eclipsed => after,
        None => {
            return Err(Error::internal(format!(
                "no rising in the two days after the eclipse that set at {after}"
            )));
        }
    };
    eclipse_vedha(eclipse, &turns, next_rising, rule)
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::float_cmp,
        reason = "tests fail by panicking, and compare quarters of exact halves"
    )]

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
    fn a_solar_eclipse_or_a_moon_risen_eclipsed_counts_four_under_each_count() {
        let risen = SeenEclipse {
            rose_eclipsed: true,
            ..eclipse(false, 0.76)
        };
        for rule in [EclipseVedha::Dharmasindhu, EclipseVedha::FullLunarFour] {
            assert_eq!(vedha_praharas(&eclipse(true, 0.30), rule), 4);
            assert_eq!(vedha_praharas(&risen, rule), 4);
        }
        // Fixed hours tell the Sun from the Moon and nothing else.
        assert_eq!(vedha_praharas(&risen, EclipseVedha::FixedHours), 3);
    }

    #[test]
    fn fixed_hours_count_back_from_the_first_moment_seen() {
        let rule = EclipseVedha::FixedHours;
        // Twelve hours before a solar eclipse, nine before a lunar one,
        // whichever prahara holds it.
        assert_eq!(opens(&eclipse(true, 0.30), rule), 0.30 - 0.5);
        assert_eq!(opens(&eclipse(false, 0.80), rule), 0.80 - 0.375);
        let set = SeenEclipse {
            set_eclipsed: true,
            ..eclipse(false, 0.80)
        };
        let held = eclipse_vedha(&set, &turns(), 1.8, rule).unwrap();
        assert_eq!(held.at.to.get(), 1.8);
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
