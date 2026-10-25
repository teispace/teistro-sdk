//! Windows: the pieces of a day over which no clause changes
//! (`03-design/muhurta.md` §4.3).
//!
//! A day is cut at every instant a clause can change at — the lagna's
//! sign and navamsa, a graha's ingress, a limb's end, a kaala's edge —
//! and each piece between two cuts is judged once. So a judgement is
//! **exact** over its window rather than read at the window's start.
//!
//! The lagna is the one quantity here found by search, in
//! `teistro_panchanga::rising` and re-exported here. It is sampled along
//! the clock and each change of navamsa is bisected; a change of sign is a
//! change of navamsa. Beyond the polar circle the lagna jumps across the
//! arcs that never rise, and the bisection then closes on the jump: one
//! cut, where several navamsas are skipped at once, which is what the sky
//! does.

use teistro_core::catalogue::Rashi;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_panchanga::Panchanga;
use teistro_panchanga::span::Span;

use crate::clause::{Clause, ClauseKind};
pub use teistro_panchanga::rising::{
    LagnaCut, SAMPLE_DAYS, TOLERANCE_DAYS, lagna_cuts, lagna_cuts_forward, navamsa_of,
};

/// Half a ghati, the twelve minutes Raman's lagna tyajya spans (ch. II).
const HALF_GHATI_DAYS: f64 = 12.0 / 1440.0;

/// The part of a sign's rising Raman's lagna tyajya rejects (ch. II,
/// p. 14): the first half ghati for Aries, Taurus, Sagittarius and Virgo,
/// the last for Pisces, Capricorn, Cancer and Scorpio, and the middle for
/// Gemini, Libra, Leo and Aquarius.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_core::interval::Interval;
/// use teistro_muhurta::window::tyajya_of;
///
/// // Libra rising for two hours: the tyajya is the half ghati at its middle.
/// let rising = Interval::literal(2_460_000.5, 2_460_000.5 + 2.0 / 24.0);
/// let middle = tyajya_of(Rashi::Libra, rising).ok_or("inside")?;
/// let minutes = |t: f64| (t - 2_460_000.5) * 1440.0;
/// assert!((minutes(middle.from.get()) - 54.0).abs() < 1e-6);
/// assert!((minutes(middle.to.get()) - 66.0).abs() < 1e-6);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn tyajya_of(sign: Rashi, rising: Interval) -> Option<Interval> {
    let (from, to) = (rising.from.get(), rising.to.get());
    let (a, b) = match sign {
        Rashi::Aries | Rashi::Taurus | Rashi::Sagittarius | Rashi::Virgo => {
            (from, from + HALF_GHATI_DAYS)
        }
        Rashi::Pisces | Rashi::Capricorn | Rashi::Cancer | Rashi::Scorpio => {
            (to - HALF_GHATI_DAYS, to)
        }
        _ => {
            let middle = f64::midpoint(from, to);
            (
                middle - 0.5 * HALF_GHATI_DAYS,
                middle + 0.5 * HALF_GHATI_DAYS,
            )
        }
    };
    Interval::new(
        JulianDay::literal(a.max(from)),
        JulianDay::literal(b.min(to)),
    )
    .ok()
}

/// The lagna tyajya clauses of the signs that rise wholly between two of
/// the cuts, clipped to `day`.
///
/// A sign whose entry or exit is not among the cuts — one already rising
/// when the search began, or still rising when it ended — is not judged,
/// since where its first or last half ghati falls is not known; a caller
/// searches a margin either side of the day so the day's own signs are
/// whole.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_core::quantity::JulianDay;
/// use teistro_muhurta::window::lagna_cuts;
///
/// // A lagna turning a whole circle a day, at 0° Aries at T.
/// const T: f64 = 2_460_000.5;
/// let lagna_at = |at: JulianDay<_>| Ok(((at.get() - T) * 360.0).rem_euclid(360.0));
/// use teistro_core::catalogue::Rashi;
/// use teistro_muhurta::ClauseKind;
/// use teistro_muhurta::window::tyajya;
///
/// // Aries, Taurus and Gemini rise whole; Aries's tyajya is its first half ghati.
/// let day = Interval::literal(T - 0.001, T + 0.26);
/// let clauses = tyajya(&lagna_cuts(day, lagna_at)?, day);
/// assert_eq!(clauses.len(), 3);
/// assert_eq!(clauses[0].kind, ClauseKind::LagnaTyajya { sign: Rashi::Aries });
/// assert!((clauses[0].at.to.get() - clauses[0].at.from.get() - 12.0 / 1440.0).abs() < 1e-6);
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[must_use]
pub fn tyajya(cuts: &[LagnaCut], day: Interval) -> Vec<Clause> {
    let entries: Vec<&LagnaCut> = cuts.iter().filter(|c| c.sign_changed).collect();
    entries
        .windows(2)
        .filter_map(|pair| {
            let [entry, exit] = pair else { return None };
            let sign = Rashi::from_id(u16::from(entry.navamsa / 9))?;
            let rising = Interval::new(entry.at, exit.at).ok()?;
            let at = tyajya_of(sign, rising)?.clipped_to(day)?;
            Some(Clause {
                kind: ClauseKind::LagnaTyajya { sign },
                at,
            })
        })
        .collect()
}

/// Every instant an almanac day's own clauses can change at: the ends of
/// its four limbs, of its panchaka and varjyam, of its kaalas, choghadiya, horas and
/// muhurtas, its sunrise, midday and sunset, and the Sun's and the Moon's
/// ingresses.
///
/// The other grahas' ingresses and the lagna's cuts are not in an almanac
/// day; a search adds them. Unsorted and possibly repeated, which
/// [`windows`] accepts.
///
/// ```
/// # use teistro_astro::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_astro::visibility::{Criterion, Heliacal};
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_chart::foundation::Founder;
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_ephemeris_builtin::provider::Builtin;
/// # use teistro_muhurta::sources::Over;
/// # use teistro_muhurta::{ProviderSources, Sources};
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::Horizon;
/// #
/// # // Kathmandu, on the Nepal clock, over the built-in ephemeris.
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = Builtin::new();
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let founder = Founder::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let completion = Completion::new(&provider, resolved.settings.provider.overrides, delta_t);
/// # let heliacal = Heliacal::new(&completion, kathmandu, Criterion::SURYA_SIDDHANTA, Horizon::CENTRE_NO_REFRACTION, delta_t);
/// # let over = Over {
/// #     almanac: &almanac,
/// #     founder: &founder,
/// #     completion: &completion,
/// #     heliacal: &heliacal,
/// #     calendar: &Gregorian,
/// #     clock: &clock,
/// #     settings: &resolved.settings,
/// #     precession,
/// #     delta_t,
/// # };
/// # // The chart zodiac taken at 2026-09-01 00:00 UTC.
/// # let sources = ProviderSources::new(&over, kathmandu, JulianDay::literal(2_461_284.5))?;
/// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
/// // 25 November 2026, after Devuthani.
/// let day = sources.day(&date(11, 25))?;
/// use teistro_muhurta::window::{day_cuts, windows};
///
/// // Sunrise is among the cuts, and the day falls into pieces at them.
/// let cuts = day_cuts(&day);
/// assert!(cuts.contains(&day.day.sunrise));
/// let pieces = windows(day.window, cuts);
/// assert!(pieces.len() > 20);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn day_cuts(day: &Panchanga) -> Vec<JulianDay<Utc>> {
    let arc = &day.day;
    let muhurtas = &day.muhurtas;
    let mut cuts = vec![
        arc.sunrise,
        arc.sunset,
        JulianDay::literal(f64::midpoint(arc.sunrise.get(), arc.sunset.get())),
    ];
    let limbs = &day.limbs;
    edges_of(&limbs.tithi, &mut cuts);
    edges_of(&limbs.nakshatra, &mut cuts);
    edges_of(&limbs.yoga, &mut cuts);
    edges_of(&limbs.karana, &mut cuts);
    edges_of(&day.omens.panchaka, &mut cuts);
    edges_of(&day.omens.varjyam, &mut cuts);
    edges_of(&day.sun.signs, &mut cuts);
    edges_of(&day.moon.signs, &mut cuts);
    let intervals = day
        .kaalas
        .iter()
        .map(|k| k.at)
        .chain(day.choghadiya.iter().map(|c| c.at))
        .chain(muhurtas.daylight.iter().copied())
        .chain(muhurtas.night.iter().copied())
        .chain(muhurtas.abhijit)
        .chain(muhurtas.brahma);
    for at in intervals {
        cuts.extend([at.from, at.to]);
    }
    for hora in &day.horas {
        cuts.extend([hora.start, hora.end]);
    }
    cuts
}

/// Pushes where each span begins and ends.
fn edges_of<T>(spans: &[Span<T>], cuts: &mut Vec<JulianDay<Utc>>) {
    for span in spans {
        cuts.extend([span.whole.from, span.whole.to]);
    }
}

/// The pieces of `day` between consecutive cuts, in order.
///
/// Cuts outside the day are ignored and repeats collapse, so a caller
/// hands in every instant any clause changes at without sorting them.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_core::quantity::JulianDay;
/// use teistro_muhurta::window::windows;
///
/// // Unsorted, repeated and outside cuts are all accepted.
/// const T: f64 = 2_460_000.5;
/// let cuts = [T + 0.5, T + 0.25, T + 0.5, T + 2.0].map(JulianDay::literal);
/// let pieces = windows(Interval::literal(T, T + 1.0), cuts);
/// let edges: Vec<f64> = pieces.iter().map(|w| w.to.get() - T).collect();
/// assert_eq!(edges, [0.25, 0.5, 1.0]);
/// ```
#[must_use]
pub fn windows(day: Interval, cuts: impl IntoIterator<Item = JulianDay<Utc>>) -> Vec<Interval> {
    let (from, to) = (day.from.get(), day.to.get());
    let mut at: Vec<f64> = cuts
        .into_iter()
        .map(JulianDay::get)
        .filter(|t| *t > from && *t < to)
        .collect();
    at.sort_by(f64::total_cmp);
    at.dedup();
    let edges: Vec<f64> = core::iter::once(from)
        .chain(at)
        .chain(core::iter::once(to))
        .collect();
    edges
        .windows(2)
        .filter_map(|pair| match pair {
            [a, b] => Interval::new(JulianDay::literal(*a), JulianDay::literal(*b)).ok(),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]
mod tests {
    use super::{lagna_cuts, lagna_cuts_forward, tyajya, tyajya_of, windows};
    use crate::clause::ClauseKind;
    use teistro_core::catalogue::Rashi;
    use teistro_core::interval::Interval;
    use teistro_core::quantity::JulianDay;

    const T0: f64 = 2_460_000.5;

    /// A lagna turning steadily once a day from 0°.
    #[allow(clippy::unnecessary_wraps, reason = "the shape `lagna_cuts` takes")]
    fn steady(
        at: JulianDay<teistro_core::quantity::Utc>,
    ) -> Result<f64, teistro_core::error::Error> {
        Ok((at.get() - T0) * 360.0)
    }

    #[test]
    fn a_steady_lagna_crosses_every_navamsa_once_a_day() {
        let day = Interval::literal(T0, T0 + 1.0);
        let cuts = lagna_cuts(day, steady).unwrap();
        // 108 boundaries 1/108 day apart. The one at T0 is where the
        // search starts and is not a crossing; the one at the day's end,
        // where Aries rises again, is.
        assert_eq!(cuts.len(), 108);
        for (k, (cut, step)) in cuts.iter().zip(1_u32..).enumerate() {
            let expected = T0 + f64::from(step) / 108.0;
            assert!(
                (cut.at.get() - expected).abs() < 2e-8,
                "{k}: {}",
                cut.at.get()
            );
            assert_eq!(usize::from(cut.navamsa), (k + 1) % 108);
        }
        assert_eq!(cuts.iter().filter(|c| c.sign_changed).count(), 12);
    }

    #[test]
    fn a_forward_lagna_is_cut_bit_for_bit_as_the_minute_scan_cuts_it() {
        // An uneven forward lagna, fast and slow as a temperate one is,
        // over a day and a quarter that starts and ends off the grid.
        let uneven = |at: JulianDay<teistro_core::quantity::Utc>| {
            let x = (at.get() - T0) * std::f64::consts::TAU;
            Ok((at.get() - T0) * 360.0 + 40.0 * x.sin() + 7.3)
        };
        let day = Interval::literal(T0 + 0.013, T0 + 1.271);
        let mut scanned = 0_u32;
        let all = lagna_cuts(day, |at| {
            scanned += 1;
            uneven(at)
        })
        .unwrap();
        let mut halved = 0_u32;
        let forward = lagna_cuts_forward(day, |at| {
            halved += 1;
            uneven(at)
        })
        .unwrap();
        assert_eq!(forward, all);
        assert!(all.len() > 130, "{}", all.len());
        for (f, a) in forward.iter().zip(&all) {
            assert_eq!(f.at.get().to_bits(), a.at.get().to_bits());
        }
        // 4,312 against 3,152 when written: the 1,812 minute samples
        // become some 650, and the refinements are the same.
        assert!(halved * 4 < scanned * 3, "{halved} of {scanned}");
    }

    #[test]
    fn a_forward_scan_of_less_than_a_minute_is_the_minute_scan() {
        let day = Interval::literal(T0 + 0.004_6, T0 + 0.004_9);
        assert_eq!(
            lagna_cuts_forward(day, steady).unwrap(),
            lagna_cuts(day, steady).unwrap()
        );
    }

    #[test]
    fn a_jump_is_one_cut() {
        // A lagna that leaps from 10° to 100° at the day's middle, as a
        // polar one leaps the arc that never rises.
        let jump = |at: JulianDay<teistro_core::quantity::Utc>| {
            Ok(if at.get() < T0 + 0.5 { 10.0 } else { 100.0 })
        };
        let cuts = lagna_cuts(Interval::literal(T0, T0 + 1.0), jump).unwrap();
        assert_eq!(cuts.len(), 1);
        assert!((cuts[0].at.get() - (T0 + 0.5)).abs() < 2e-8);
        assert!(cuts[0].sign_changed);
        assert_eq!(cuts[0].navamsa, 30);
    }

    #[test]
    fn the_tyajya_falls_where_ramans_rule_puts_it() {
        let rising = Interval::literal(T0, T0 + 0.1);
        let twelve = 12.0 / 1440.0;
        let first = tyajya_of(Rashi::Aries, rising).unwrap();
        assert!((first.from.get() - T0).abs() < 1e-12);
        assert!((first.to.get() - (T0 + twelve)).abs() < 1e-12);
        let last = tyajya_of(Rashi::Scorpio, rising).unwrap();
        assert!((last.from.get() - (T0 + 0.1 - twelve)).abs() < 1e-9);
        let middle = tyajya_of(Rashi::Leo, rising).unwrap();
        assert!((f64::midpoint(middle.from.get(), middle.to.get()) - (T0 + 0.05)).abs() < 1e-9);
        assert!((middle.days() - twelve).abs() < 1e-9);
    }

    #[test]
    fn only_a_sign_risen_whole_between_cuts_is_judged() {
        let day = Interval::literal(T0, T0 + 1.0);
        let cuts = lagna_cuts(day, steady).unwrap();
        let clauses = tyajya(&cuts, day);
        // Twelve sign entries make eleven whole signs, Taurus to Pisces;
        // the first Aries began before the search and the second rises
        // at its end.
        assert_eq!(clauses.len(), 11);
        assert_eq!(
            clauses[0].kind,
            ClauseKind::LagnaTyajya {
                sign: Rashi::Taurus
            }
        );
        assert_eq!(
            clauses[10].kind,
            ClauseKind::LagnaTyajya {
                sign: Rashi::Pisces
            }
        );
        // Pisces's is its last half ghati, ending as Aries rises.
        assert!((clauses[10].at.to.get() - (T0 + 1.0)).abs() < 2e-8);
        // Taurus's is its first half ghati, from its entry.
        assert!((clauses[0].at.from.get() - (T0 + 1.0 / 12.0)).abs() < 2e-8);
    }

    #[test]
    fn windows_cover_the_day_between_sorted_unique_cuts() {
        let day = Interval::literal(T0, T0 + 1.0);
        let pieces = windows(
            day,
            [0.7, 0.2, 0.7, -0.1, 1.5, 0.0]
                .into_iter()
                .map(|x| JulianDay::literal(T0 + x)),
        );
        let expected = [(0.0, 0.2), (0.2, 0.7), (0.7, 1.0)];
        assert_eq!(pieces.len(), expected.len());
        for (w, (a, b)) in pieces.iter().zip(expected) {
            assert!((w.from.get() - (T0 + a)).abs() < 1e-9, "{w:?}");
            assert!((w.to.get() - (T0 + b)).abs() < 1e-9, "{w:?}");
        }
    }
}
