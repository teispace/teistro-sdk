//! Windows: the pieces of a day over which no clause changes
//! (`03-design/muhurta.md` §4.3).
//!
//! A day is cut at every instant a clause can change at — the lagna's
//! sign and navamsa, a graha's ingress, a limb's end, a kaala's edge —
//! and each piece between two cuts is judged once. So a judgement is
//! **exact** over its window rather than read at the window's start.
//!
//! The lagna is the one quantity here found by search. It is sampled along
//! the clock and each change of navamsa is bisected; a change of sign is a
//! change of navamsa. Beyond the polar circle the lagna jumps across the
//! arcs that never rise, and the bisection then closes on the jump: one
//! cut, where several navamsas are skipped at once, which is what the sky
//! does.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Rashi;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_panchanga::Panchanga;
use teistro_panchanga::span::Span;

use crate::clause::{Clause, ClauseKind};

/// How far apart the lagna is sampled, in days: one minute. The lagna
/// crosses a navamsa (3°20′) in about thirteen minutes on average and in
/// no less than about four where it rises fastest outside the polar
/// circles, so a minute sees every crossing there.
pub const SAMPLE_DAYS: f64 = 1.0 / 1440.0;

/// How close a cut is found, in days: about a millisecond.
pub const TOLERANCE_DAYS: f64 = 1e-8;

/// Half a ghati, the twelve minutes Raman's lagna tyajya spans (ch. II).
const HALF_GHATI_DAYS: f64 = 12.0 / 1440.0;

/// An instant the lagna enters a navamsa.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LagnaCut {
    /// When.
    pub at: JulianDay<Utc>,
    /// The navamsa entered, 0 (Aries' first) to 107 (Pisces' last).
    pub navamsa: u8,
    /// Whether the lagna entered a new sign here.
    pub sign_changed: bool,
}

/// Every instant in a window at which the lagna enters a navamsa.
///
/// `lagna_at` gives the lagna's longitude, degrees, at an instant; it is
/// asked once a minute and then at each bisection step.
///
/// # Errors
///
/// Whatever `lagna_at` returns.
pub fn lagna_cuts<F>(within: Interval, mut lagna_at: F) -> Result<Vec<LagnaCut>, Error>
where
    F: FnMut(JulianDay<Utc>) -> Result<f64, Error>,
{
    let mut cuts = Vec::new();
    let (from, to) = (within.from.get(), within.to.get());
    let mut before = (from, navamsa_of(lagna_at(within.from)?));
    let mut t = from;
    while t < to {
        t = (t + SAMPLE_DAYS).min(to);
        let now = (t, navamsa_of(lagna_at(JulianDay::literal(t))?));
        if now.1 != before.1 {
            refine(before, now, &mut lagna_at, &mut cuts)?;
        }
        before = now;
    }
    Ok(cuts)
}

/// Finds every change of navamsa between two samples that differ.
fn refine<F>(
    a: (f64, u8),
    b: (f64, u8),
    lagna_at: &mut F,
    cuts: &mut Vec<LagnaCut>,
) -> Result<(), Error>
where
    F: FnMut(JulianDay<Utc>) -> Result<f64, Error>,
{
    let mid = f64::midpoint(a.0, b.0);
    // Converged, or below the last bit at this magnitude (a midpoint that
    // rounds onto an end cannot split the interval further).
    if b.0 - a.0 <= TOLERANCE_DAYS || mid <= a.0 || mid >= b.0 {
        cuts.push(LagnaCut {
            at: JulianDay::literal(b.0),
            navamsa: b.1,
            sign_changed: b.1 / 9 != a.1 / 9,
        });
        return Ok(());
    }
    let m = (mid, navamsa_of(lagna_at(JulianDay::literal(mid))?));
    if m.1 != a.1 {
        refine(a, m, lagna_at, cuts)?;
    }
    if m.1 != b.1 {
        refine(m, b, lagna_at, cuts)?;
    }
    Ok(())
}

/// The navamsa a longitude falls in, 0 to 107: the one floor every
/// sign and navamsa in the crate is read through.
pub(crate) fn navamsa_of(deg: f64) -> u8 {
    let index = (deg.rem_euclid(360.0) * 0.3).floor();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a floor in 0..108"
    )]
    let id = index as u8;
    id.min(107)
}

/// The part of a sign's rising Raman's lagna tyajya rejects (ch. II,
/// p. 14): the first half ghati for Aries, Taurus, Sagittarius and Virgo,
/// the last for Pisces, Capricorn, Cancer and Scorpio, and the middle for
/// Gemini, Libra, Leo and Aquarius.
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
/// its four limbs, of its panchaka, of its kaalas, choghadiya, horas and
/// muhurtas, its sunrise, midday and sunset, and the Sun's and the Moon's
/// ingresses.
///
/// The other grahas' ingresses and the lagna's cuts are not in an almanac
/// day; a search adds them. Unsorted and possibly repeated, which
/// [`windows`] accepts.
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
    use super::{lagna_cuts, tyajya, tyajya_of, windows};
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
