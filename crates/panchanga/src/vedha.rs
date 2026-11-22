//! An eclipse's vedha, the sutak: the praharas before an eclipse a place
//! sees in which food is not taken (*Dharmasindhu* p. 28;
//! `03-design/muhurta.md` §4.1.1, `03-design/panchanga-reach.md`).
//!
//! The count reads only the eclipse as the place sees it and the sunrises
//! and sunsets of the almanac days around it, so it lives with the almanac
//! rather than with the muhurta search: an eclipse view reaches it in a
//! build without the muhurta family (SR-132), and the search's season
//! reads the same function.

use teistro_astro::eclipse::{EclipsesHere, LunarEclipseKind};
use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::EclipseVedha;
use teistro_core::time::LocalClock;

use crate::almanac::Panchanga;
use crate::season::{Blackout, BlackoutKind, blackout, interval};

/// Which eclipse the season reads, as far as the vedha's count asks.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::vedha::{SeenEclipse, SeenKind};
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
/// use teistro_panchanga::vedha::vedha_praharas;
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
/// use teistro_panchanga::vedha::{SeenEclipse, SeenKind};
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
/// use teistro_panchanga::vedha::vedha_praharas;
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
    /// use teistro_panchanga::vedha::SeenEclipse;
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

/// The praharas an eclipse's vedha opens before, under a rule
/// (`panchanga.eclipse_vedha`): four for a solar eclipse and for a Moon
/// that rises eclipsed, three for any other lunar eclipse, or four for a
/// total one under `FULL_LUNAR_FOUR`.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::vedha::{SeenEclipse, SeenKind};
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
/// use teistro_panchanga::vedha::vedha_praharas;
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
/// use teistro_panchanga::vedha::{SeenEclipse, SeenKind};
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
/// use teistro_panchanga::season::BlackoutKind;
/// use teistro_panchanga::vedha::eclipse_vedha;
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
/// use teistro_panchanga::vedha::{SeenEclipse, SeenKind, vedha_over_days};
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
    days: &[Panchanga],
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

/// An eclipse's vedha read off the civil days around the one that holds
/// its first moment seen on `clock`: [`VEDHA_DAYS_AROUND`] either side,
/// each asked of `day`, and counted by [`vedha_over_days`]. The one
/// reading of which days a vedha needs, which the almanac's eclipses and
/// the muhurta search's season both use, each with its own source of days.
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::Gregorian;
/// # use teistro_core::catalogue::Ayanamsha;
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
/// use teistro_core::settings::EclipseVedha;
/// use teistro_panchanga::vedha::{SeenEclipse, SeenKind, vedha_of};
///
/// // A lunar eclipse seen for an hour from 18:00 UTC on 25 November 2026.
/// let eclipse = SeenEclipse {
///     kind: SeenKind::Lunar,
///     greatest: 2_461_370.27,
///     seen: Interval::literal(2_461_370.25, 2_461_370.29),
///     rose_eclipsed: false,
///     set_eclipsed: false,
/// };
/// let mut asked = Vec::new();
/// let vedha = vedha_of(&eclipse, EclipseVedha::Dharmasindhu, &Gregorian, &clock, |date| {
///     asked.push(date.clone());
///     Ok(almanac.day(date, &kathmandu)?.value)
/// })?;
/// // Five civil days asked, and the vedha opens before the eclipse is seen.
/// assert_eq!(asked.len(), 5);
/// assert!(vedha.at.from.get() < eclipse.seen.from.get());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// `day`'s own refusal, a date the calendar cannot write, and as
/// [`vedha_over_days`].
pub fn vedha_of(
    eclipse: &SeenEclipse,
    rule: EclipseVedha,
    calendar: &dyn CalendarSystem,
    clock: &dyn LocalClock,
    mut day: impl FnMut(&CalendarDate) -> Result<Panchanga, Error>,
) -> Result<Blackout, Error> {
    let first = JulianDay::<Utc>::literal(eclipse.seen.from.get());
    let (fixed, _) = FixedDay::from_local_jd(clock.local_jd(first));
    let days = (-VEDHA_DAYS_AROUND..=VEDHA_DAYS_AROUND)
        .map(|n| day(&calendar.date_of(fixed.plus_days(n))?))
        .collect::<Result<Vec<Panchanga>, Error>>()?;
    vedha_over_days(eclipse, &days, rule)
}

/// Gives each eclipse of `found` the place sees its vedha under `rule`,
/// read by [`vedha_of`] off the days `day` answers; an eclipse the place
/// does not see keeps `None`.
///
/// ```
/// use teistro_astro::eclipse::EclipsesHere;
/// use teistro_calendar::Gregorian;
/// use teistro_core::error::Error;
/// use teistro_core::settings::EclipseVedha;
/// use teistro_core::time::UtcOffset;
/// use teistro_panchanga::vedha::with_vedhas;
///
/// // No eclipse seen, so no day is asked for and nothing is given.
/// let mut found = EclipsesHere::default();
/// with_vedhas(&mut found, EclipseVedha::Dharmasindhu, &Gregorian, &UtcOffset::UTC, |_| {
///     Err(Error::internal("no day is asked for"))
/// })?;
/// assert_eq!(found, EclipsesHere::default());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`vedha_of`].
pub fn with_vedhas(
    found: &mut EclipsesHere,
    rule: EclipseVedha,
    calendar: &dyn CalendarSystem,
    clock: &dyn LocalClock,
    mut day: impl FnMut(&CalendarDate) -> Result<Panchanga, Error>,
) -> Result<(), Error> {
    for seen in SeenEclipse::all_of(found)? {
        let vedha = Some(vedha_of(&seen, rule, calendar, clock, &mut day)?.at);
        let greatest = |at: f64| at.to_bits() == seen.greatest.to_bits();
        if seen.kind == SeenKind::Solar {
            for solar in &mut found.solar {
                if greatest(solar.eclipse.greatest.get()) {
                    solar.vedha = vedha;
                }
            }
        } else {
            for lunar in &mut found.lunar {
                if greatest(lunar.eclipse.greatest.get()) {
                    lunar.vedha = vedha;
                }
            }
        }
    }
    Ok(())
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
