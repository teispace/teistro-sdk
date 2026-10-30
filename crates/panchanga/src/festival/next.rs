//! The next day a rule or a kind of day falls on, with no year asked
//! (`03-design/next-occurrence.md`, SR-06): the kinds' own rules, the
//! look-ahead limit and the windows a search reckons, each one the
//! festival reckoning the range would have had.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Tithi;
use teistro_core::error::Error;

use super::{Choice, Convention, DatedRule, EkadashiRule, FestivalRule, InAdhika, Window};
use crate::almanac::MOST_DAYS;

/// How far a search looks by default, in days: past the longest gap
/// between two days of a rule kept once a year. A lunar month's festival
/// recurs within thirteen lunations, 383.9 days, and its day may move by
/// a day either side; a solar one within 366 days.
pub const LIMIT_DAYS: u32 = 400;

/// The most a search may look, in days: ten years, past the longest gap
/// between two adhika months (about 35 lunations, 1,034 days), which a
/// rule kept only in a doubled month waits.
pub const MOST_LIMIT_DAYS: u32 = 3_660;

/// The days the first window reckons: past a fortnight, so the next
/// Ekadashi, Purnima or Amavasya is found in it.
pub const FIRST_WINDOW_DAYS: u32 = 32;

/// A kind of day a search may ask for without naming a pack, each read by
/// rules the SDK states (`03-design/next-occurrence.md` §2).
///
/// ```
/// use teistro_panchanga::festival::NextKind;
///
/// assert_eq!(NextKind::ALL.map(NextKind::key), ["EKADASHI", "PURNIMA", "AMAVASYA", "SANKRANTI"]);
/// assert_eq!(serde_json::to_string(&NextKind::Sankranti)?, r#""SANKRANTI""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NextKind {
    /// An Ekadashi fast, by *Dharmasindhu*'s Smarta and Vaishnava rules
    /// ([`EkadashiRule::dharmasindhu`]); the earlier when they part.
    Ekadashi,
    /// The day whose sunrise holds the full moon, or the day an
    /// unrisen one falls in, every month (C197).
    Purnima,
    /// The same, for the new moon.
    Amavasya,
    /// The first day of a Bikram Sambat month, the day the Sun enters its
    /// sign under the punya convention (C619), as Nepal's national
    /// panchanga prints the twelve sankrantis.
    Sankranti,
}

/// The rules a kind is read by, of each sort a festival request carries.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KindRules {
    /// Its karmakala rules.
    pub rules: Vec<FestivalRule>,
    /// Its Ekadashi rules.
    pub ekadashis: Vec<EkadashiRule>,
    /// Its rules kept on a date.
    pub dated: Vec<DatedRule>,
}

impl NextKind {
    /// Every kind.
    pub const ALL: [NextKind; 4] = [
        NextKind::Ekadashi,
        NextKind::Purnima,
        NextKind::Amavasya,
        NextKind::Sankranti,
    ];

    /// Its key, as a request names it.
    ///
    /// ```
    /// use teistro_panchanga::festival::NextKind;
    ///
    /// assert_eq!(NextKind::Purnima.key(), "PURNIMA");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            NextKind::Ekadashi => "EKADASHI",
            NextKind::Purnima => "PURNIMA",
            NextKind::Amavasya => "AMAVASYA",
            NextKind::Sankranti => "SANKRANTI",
        }
    }

    /// The rules it is read by.
    ///
    /// ```
    /// use teistro_panchanga::festival::NextKind;
    ///
    /// let purnima = NextKind::Purnima.rules();
    /// assert_eq!(purnima.rules[0].key, "PURNIMA");
    /// assert!(purnima.rules[0].source.contains("C197"));
    /// assert_eq!(NextKind::Sankranti.rules().dated.len(), 12);
    /// assert_eq!(NextKind::Ekadashi.rules().ekadashis.len(), 3);
    /// ```
    #[must_use]
    pub fn rules(self) -> KindRules {
        match self {
            NextKind::Ekadashi => KindRules {
                ekadashis: EkadashiRule::dharmasindhu(),
                ..KindRules::default()
            },
            NextKind::Purnima => KindRules {
                rules: vec![every_month("PURNIMA", Tithi::Purnima, "the full moon")],
                ..KindRules::default()
            },
            NextKind::Amavasya => KindRules {
                rules: vec![every_month("AMAVASYA", Tithi::Amavasya, "the new moon")],
                ..KindRules::default()
            },
            NextKind::Sankranti => KindRules {
                dated: DatedRule::sankrantis(),
                ..KindRules::default()
            },
        }
    }
}

/// A tithi's day every month, the adhika month too, read by the sunrise
/// it holds.
fn every_month(key: &str, tithi: Tithi, name: &str) -> FestivalRule {
    FestivalRule {
        key: key.to_owned(),
        source: format!(
            "the day of {name} in every month, the adhika month too, as a patro printed in Nepal names a day by the tithi at its sunrise"
        ),
        month: None,
        convention: Convention::Amanta,
        occurs: tithi.into(),
        in_adhika: InAdhika::Also,
        at: Window::Sunrise,
        decide: Vec::new(),
        otherwise: Choice::Earlier,
        parana: Vec::new(),
    }
    .udaya()
}

/// A look-ahead limit, checked: 1 to [`MOST_LIMIT_DAYS`] days.
///
/// ```
/// use teistro_panchanga::festival::{LIMIT_DAYS, limit_days};
///
/// assert_eq!(limit_days(None)?, LIMIT_DAYS);
/// assert_eq!(limit_days(Some(30))?, 30);
/// assert_eq!(limit_days(Some(0)).unwrap_err().field(), Some("limitDays"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `OUT_OF_RANGE` naming `limitDays` for zero or past
/// [`MOST_LIMIT_DAYS`].
pub fn limit_days(asked: Option<u32>) -> Result<u32, Error> {
    match asked {
        None => Ok(LIMIT_DAYS),
        Some(days) if (1..=MOST_LIMIT_DAYS).contains(&days) => Ok(days),
        Some(days) => Err(Error::new(
            teistro_core::error::Status::OutOfRange,
            format!("a search looks 1 to {MOST_LIMIT_DAYS} days ahead, not {days}"),
        )
        .with_field("limitDays")),
    }
}

/// The windows a search over `limit` days reckons, each as the day it
/// starts on counted from the first, and its days: [`FIRST_WINDOW_DAYS`],
/// then each twice the last, at most a range's [`MOST_DAYS`], until the
/// limit. A day found early costs one short window; a yearly festival a
/// few, about the limit in all.
///
/// ```
/// use teistro_panchanga::festival::windows;
///
/// assert_eq!(windows(400), [(0, 32), (32, 64), (96, 128), (224, 176)]);
/// assert_eq!(windows(10), [(0, 10)]);
/// assert_eq!(windows(1000).iter().map(|(_, days)| days).sum::<u32>(), 1000);
/// ```
#[must_use]
pub fn windows(limit: u32) -> Vec<(u32, u32)> {
    let most = u32::try_from(MOST_DAYS).unwrap_or(u32::MAX);
    let mut found = Vec::new();
    let (mut start, mut days) = (0, FIRST_WINDOW_DAYS);
    while start < limit {
        let taken = days.min(limit - start).min(most);
        found.push((start, taken));
        start += taken;
        days = days.saturating_mul(2);
    }
    found
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn every_kind_reads_by_rules_that_check() {
        for kind in NextKind::ALL {
            let read = kind.rules();
            assert!(
                read.rules.len() + read.ekadashis.len() + read.dated.len() > 0,
                "{kind:?}"
            );
            for rule in &read.rules {
                rule.check().unwrap();
                assert_eq!(rule.at, Window::Sunrise);
            }
            for rule in &read.ekadashis {
                rule.check().unwrap();
            }
            for rule in &read.dated {
                rule.check().unwrap();
            }
        }
    }

    #[test]
    fn the_windows_cover_the_limit_once_and_none_is_past_a_range() {
        for limit in [1, 31, 32, 33, 400, 1_034, MOST_LIMIT_DAYS] {
            let plan = windows(limit);
            let mut next = 0;
            for (start, days) in &plan {
                assert_eq!(*start, next);
                assert!(*days >= 1 && usize::try_from(*days).unwrap() <= MOST_DAYS);
                next += days;
            }
            assert_eq!(next, limit);
        }
        assert_eq!(windows(0), []);
        assert!(limit_days(Some(MOST_LIMIT_DAYS)).is_ok());
        assert!(limit_days(Some(MOST_LIMIT_DAYS + 1)).is_err());
    }
}
