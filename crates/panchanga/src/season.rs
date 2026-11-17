//! The season: the stretches of the year the Sun and the Moon close to a
//! rite (`03-design/muhurta.md` §4.1, `03-design/panchanga-reach.md`).
//!
//! A blackout is not a defect weighed against a good day: while Chaturmas
//! holds, no school marries, however good the rest of the day. So the
//! season is kept apart from any judgement of a day: each blackout a kind
//! and the interval it holds over, found once for a range.
//!
//! The intervals are **instants**, not whole days: Pitru paksha begins
//! when the Purnima begins, not at a midnight. A day-level reader asks
//! "any blackout overlaps the day".
//!
//! Every blackout here is the Sun and the Moon: the lunar months from the
//! new moons, their names from the Sun's sign at the opening new moon,
//! their kinds from the sankrantis inside them (the calendar's own rule,
//! `calendar-indian-lunisolar.md` §2), the tithis that bound Chaturmas,
//! Holashtaka and Pitru paksha, and the Sun's signs that are Kharmas and
//! the sankrantis. The rule lives here, below the muhurta family, so an
//! almanac reaches it without the muhurta search (SR-138); the muhurta's
//! season re-exports it and adds what only an election reads: asta, by a
//! visibility criterion the search names, and an eclipse's star. An
//! eclipse's vedha is [`crate::vedha`]'s.

use serde::{Deserialize, Serialize};
use teistro_astro::Completion;
use teistro_astro::events::{Longitudes, Search};
use teistro_calendar::lunisolar::{MonthKind, SYNODIC_DAYS};
use teistro_core::catalogue::{Masa, Rashi};
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, Lattice, Quantity};

use crate::limb::{Sidereal, Zodiac, geocentric, signs_within};
use crate::span::Span;

/// Which blackout: the catalogue's `blackout_kind`, each member with its
/// source.
pub use teistro_core::catalogue::BlackoutKind;

/// The kinds the Sun and the Moon make, each of which [`blackouts`] finds:
/// every kind but asta, which a visibility criterion decides, and an
/// eclipse's star and vedha, which an eclipse a place sees decides.
///
/// ```
/// use teistro_panchanga::season::{BlackoutKind, SKY_KINDS};
///
/// assert!(SKY_KINDS.contains(&BlackoutKind::Chaturmas));
/// assert!(!SKY_KINDS.contains(&BlackoutKind::GuruAsta));
/// ```
pub const SKY_KINDS: [BlackoutKind; 8] = [
    BlackoutKind::Chaturmas,
    BlackoutKind::AdhikaMasa,
    BlackoutKind::Samsarpa,
    BlackoutKind::KshayaMasa,
    BlackoutKind::Kharmas,
    BlackoutKind::PitruPaksha,
    BlackoutKind::Sankranti,
    BlackoutKind::Holashtaka,
];

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
/// use teistro_panchanga::season::{BlackoutKind, blackouts};
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
/// use teistro_panchanga::season::months;
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

/// A synodic month, days: the lunisolar calendar's own.
const SYNODIC_MONTH_DAYS: f64 = SYNODIC_DAYS;

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
/// use teistro_panchanga::season::months;
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
/// use teistro_panchanga::season::{BlackoutKind, blackouts};
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

/// The blackouts the Sun and the Moon make over a range, read in the sky a
/// profile's season is read in: the chart's frame and search zodiac the
/// profile names ([`teistro_chart::zodiac::ChartZodiac::searched`]), the
/// frame made geocentric, since an almanac's season is the same wherever
/// it is read. What the muhurta search's season and
/// `sdk.almanac().blackouts` both read, so a ceremony page and an election
/// never disagree on when Chaturmas holds.
///
/// ```
/// # use teistro_astro::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_chart::zodiac::ChartZodiac;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{DEFAULT_PROFILE, Profile, SettingsPatch};
/// # use teistro_ephemeris_builtin::provider::Builtin;
/// #
/// # let provider = Builtin::new();
/// # let settings = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?
/// #     .settings;
/// # let models = (PrecessionModel::Vondrak2011, DeltaTModel::TableThenModel);
/// # let completion = Completion::new(&provider, settings.provider.overrides, models.1);
/// use teistro_panchanga::season::{BlackoutKind, sky_blackouts};
///
/// // 2026-09-01 to 2026-12-01, 00:00 UTC, the profile's zodiac taken at the
/// // middle.
/// let autumn = Interval::literal(2_461_284.5, 2_461_375.5);
/// let searched = ChartZodiac::searched(&completion, &settings, JulianDay::literal(2_461_330.0), models)?;
/// let found = sky_blackouts(&completion, searched.frame, searched.zodiac, autumn)?;
/// assert!(found.iter().any(|b| b.kind == BlackoutKind::Chaturmas));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// The provider's own refusal.
pub fn sky_blackouts<P: EphemerisProvider + ?Sized>(
    completion: &Completion<'_, P>,
    frame: Frame,
    zodiac: Zodiac,
    range: Interval,
) -> Result<Vec<Blackout>, Error> {
    let geocentric = completion.longitudes(geocentric(frame));
    blackouts(&geocentric, zodiac, range)
}

impl Blackout {
    /// A blackout of `kind` between two raw instants.
    ///
    /// ```
    /// use teistro_panchanga::season::{Blackout, BlackoutKind};
    ///
    /// let kharmas = Blackout::between(BlackoutKind::Kharmas, 2_461_389.9, 2_461_419.4)?;
    /// assert!((kharmas.at.days() - 29.5).abs() < 1e-9);
    /// assert!(Blackout::between(BlackoutKind::Kharmas, 2.0, 1.0).is_err());
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for an instant that is not finite or an interval the
    /// wrong way round.
    pub fn between(kind: BlackoutKind, from: f64, to: f64) -> Result<Blackout, Error> {
        Ok(blackout(kind, interval(from, to)?))
    }
}

pub(crate) const fn blackout(kind: BlackoutKind, at: Interval) -> Blackout {
    Blackout { kind, at }
}

/// Keeps what overlaps the range, clipped to it, in order of start.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::season::{Blackout, BlackoutKind, clip_and_order};
///
/// let found = vec![
///     Blackout { kind: BlackoutKind::Kharmas, at: Interval::literal(5.0, 12.0) },
///     Blackout { kind: BlackoutKind::Chaturmas, at: Interval::literal(0.0, 3.0) },
///     Blackout { kind: BlackoutKind::Sankranti, at: Interval::literal(20.0, 21.0) },
/// ];
/// let kept = clip_and_order(found, Interval::literal(1.0, 10.0));
/// assert_eq!(kept.len(), 2);
/// assert_eq!((kept[0].at.from.get(), kept[1].at.to.get()), (1.0, 10.0));
/// ```
#[must_use]
pub fn clip_and_order(found: Vec<Blackout>, range: Interval) -> Vec<Blackout> {
    let mut kept: Vec<Blackout> = found
        .into_iter()
        .filter_map(|b| b.at.clipped_to(range).map(|at| Blackout { at, ..b }))
        .filter(|b| !b.at.is_empty())
        .collect();
    kept.sort_by(|a, b| a.at.from.get().total_cmp(&b.at.from.get()));
    kept
}

/// An interval from two raw instants.
pub(crate) fn interval(from: f64, to: f64) -> Result<Interval, Error> {
    Interval::new(JulianDay::try_new(from)?, JulianDay::try_new(to)?)
}
