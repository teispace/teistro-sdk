//! The periods of a day: three inauspicious eighths, sixteen choghadiya,
//! twenty-four horas and thirty muhurtas.
//!
//! Not one of them divides a clock hour. Each is an equal division of the
//! daylight or of the night, so a period's length changes with the season
//! and the latitude, and the corpus reproduces every one of them exactly
//! (`03-design/panchanga-day-conventions.md` §3). So there is one divider
//! — [`teistro_core::interval::Interval::divided`] — and this module is
//! the tables that say which part is which.
//!
//! The choghadiya is the surprise. It is normally printed as a grid of
//! fifty-six names; it is not a grid. The lord of the *k*th eighth of the
//! daylight is the lord of the *k*th hora — the vara's lord walked five
//! weekdays on per part, which `time::hora::lord_of` already computes —
//! and the night's walk starts four weekdays on from the vara and steps
//! four. Measured over 1320 horas and 880 choghadiya with no exceptions
//! (§4). So the whole of it is one seven-row table over a function the
//! SDK already has.

use serde::Serialize;
use teistro_core::catalogue::{Choghadiya, Graha, Kaala, Vara};
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};

/// The eighths an inauspicious period may take.
const EIGHTHS: u16 = 8;
/// The muhurtas each half of a day holds.
const MUHURTAS_PER_HALF: u16 = 15;
/// Which muhurta of the daylight Abhijit is, counted from zero.
const ABHIJIT_MUHURTA: u16 = 7;
/// Which muhurta of the night before sunrise Brahma muhurta is, counted
/// from zero: the fourteenth of fifteen.
const BRAHMA_MUHURTA: u16 = 13;
/// How many weekdays a hora, and so a day choghadiya, steps.
const DAY_STEP: u8 = 5;
/// How many weekdays a night choghadiya steps.
const NIGHT_STEP: u8 = 4;
/// How far on from the vara a night choghadiya's walk starts.
const NIGHT_START: u8 = 4;

/// One inauspicious eighth of the daylight.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Kaalas {
    /// Which one.
    pub kaala: Kaala,
    /// When it runs.
    pub at: Interval,
}

/// One choghadiya, of the daylight or of the night.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Part {
    /// Which choghadiya, which carries its lord and whether it is
    /// auspicious.
    pub choghadiya: Choghadiya,
    /// The graha that rules it.
    pub lord: Graha,
    /// When it runs.
    pub at: Interval,
    /// Whether it is one of the eight of the daylight.
    pub is_daytime: bool,
}

/// The thirty muhurtas of a day, with the two that have names.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Muhurtas {
    /// The fifteen of the daylight, in order.
    pub daylight: Vec<Interval>,
    /// The fifteen of the night that follows it, in order.
    pub night: Vec<Interval>,
    /// Abhijit: the eighth muhurta of the daylight, straddling noon.
    /// Absent on a day with no daylight.
    pub abhijit: Option<Interval>,
    /// Whether Abhijit is effective, which it is on every day but a
    /// Wednesday — measured on all 55 recorded days
    /// (`03-design/panchanga-day-conventions.md` §5).
    pub abhijit_effective: bool,
    /// Brahma muhurta: the fourteenth of the fifteen muhurtas of the
    /// night that **ends** at this day's sunrise. Absent when that night
    /// is not known, which is the polar case.
    pub brahma: Option<Interval>,
}

/// The lord of the weekday `steps` on from a vara.
///
/// This is `time::hora`'s own walk, reached through its public
/// `lord_of`: the *n*th hora's lord is the lord of the weekday `5(n-1)`
/// on, so `steps` of five is one hora and `steps` of four is one night
/// choghadiya.
#[must_use]
fn walk(vara: Vara, steps: u8) -> Graha {
    let weekday = (u16::from(vara.attributes().weekday) + u16::from(steps)) % 7;
    Vara::from_id(weekday).map_or_else(|| vara.attributes().lord, |v| v.attributes().lord)
}

/// The vara `steps` weekdays on from another.
#[must_use]
fn walk_vara(vara: Vara, steps: u8) -> Vara {
    let weekday = (u16::from(vara.attributes().weekday) + u16::from(steps)) % 7;
    Vara::from_id(weekday).unwrap_or(vara)
}

/// The choghadiya a graha gives its name to.
#[must_use]
pub fn choghadiya_of(lord: Graha) -> Option<Choghadiya> {
    Choghadiya::ALL
        .iter()
        .copied()
        .find(|choghadiya| choghadiya.attributes().lord == lord)
}

/// The three inauspicious eighths of a daylight arc, on a day of a vara.
///
/// An arc with no daylight — a polar night — has none of them, and the
/// list is empty rather than holding three intervals of no length. An
/// empty interval is not the absence of a period; it is a period claiming
/// to begin and end at one instant, and code that asks whether rahu kaal
/// is running gets an answer out of it
/// (`03-design/panchanga-day-conventions.md` §11).
#[must_use]
pub fn kaalas(daylight: Interval, vara: Vara) -> Vec<Kaalas> {
    if daylight.is_empty() {
        return Vec::new();
    }
    Kaala::ALL
        .iter()
        .filter_map(|kaala| {
            let eighth = kaala
                .attributes()
                .eighth_by_vara
                .get(usize::from(vara.attributes().weekday))
                .copied()?;
            // The table counts from one, as an almanac prints it.
            let at = daylight
                .part(u16::from(eighth).checked_sub(1)?, EIGHTHS)
                .ok()?;
            Some(Kaalas { kaala: *kaala, at })
        })
        .collect()
}

/// The sixteen choghadiya of a day: eight of the daylight from the vara's
/// own lord, then eight of the night from the lord four weekdays on.
///
/// A half with no arc contributes none, which is what a polar day and a
/// polar night mean.
#[must_use]
pub fn choghadiya(daylight: Interval, night: Interval, vara: Vara) -> Vec<Part> {
    let mut parts = Vec::with_capacity(16);
    parts.extend(half(daylight, vara, DAY_STEP, true));
    parts.extend(half(night, walk_vara(vara, NIGHT_START), NIGHT_STEP, false));
    parts
}

/// One half's eight choghadiya, walking `step` weekdays per part.
fn half(arc: Interval, from: Vara, step: u8, is_daytime: bool) -> Vec<Part> {
    if arc.is_empty() {
        return Vec::new();
    }
    arc.divided(EIGHTHS)
        .enumerate()
        .filter_map(|(index, at)| {
            let steps = u8::try_from(index).ok()?.checked_mul(step)?;
            let lord = walk(from, steps);
            Some(Part {
                choghadiya: choghadiya_of(lord)?,
                lord,
                at,
                is_daytime,
            })
        })
        .collect()
}

/// The thirty muhurtas of a day, with Abhijit and Brahma muhurta named.
///
/// `previous_night` is the night that **ends** at this day's sunrise,
/// which is the night Brahma muhurta belongs to. The recording engine
/// sizes it from the night that follows the day instead — a median ten
/// seconds out, and entry 17 of the deliberate-difference registry.
#[must_use]
pub fn muhurtas(
    daylight: Interval,
    night: Interval,
    previous_night: Option<Interval>,
    vara: Vara,
) -> Muhurtas {
    let daylight_parts: Vec<Interval> = if daylight.is_empty() {
        Vec::new()
    } else {
        daylight.divided(MUHURTAS_PER_HALF).collect()
    };
    let night_parts: Vec<Interval> = if night.is_empty() {
        Vec::new()
    } else {
        night.divided(MUHURTAS_PER_HALF).collect()
    };
    Muhurtas {
        abhijit: daylight_parts.get(usize::from(ABHIJIT_MUHURTA)).copied(),
        // Void on a Wednesday, effective on every other day.
        abhijit_effective: vara != Vara::Budhavara,
        brahma: previous_night
            .filter(|arc| !arc.is_empty())
            .and_then(|arc| arc.part(BRAHMA_MUHURTA, MUHURTAS_PER_HALF).ok()),
        daylight: daylight_parts,
        night: night_parts,
    }
}

/// The choghadiya running at an instant, if one is.
#[must_use]
pub fn choghadiya_at(parts: &[Part], instant: JulianDay<Utc>) -> Option<&Part> {
    parts
        .iter()
        .find(|part| part.at.contains(instant))
        .or_else(|| {
            parts
                .last()
                .filter(|part| part.at.contains_inclusive(instant))
        })
}

/// Whether an instant falls inside one of the day's inauspicious eighths.
#[must_use]
pub fn is_inauspicious(kaalas: &[Kaalas], instant: JulianDay<Utc>) -> bool {
    kaalas.iter().any(|kaala| kaala.at.contains(instant))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use super::{
        DAY_STEP, NIGHT_START, NIGHT_STEP, choghadiya, choghadiya_at, choghadiya_of,
        is_inauspicious, kaalas, muhurtas, walk, walk_vara,
    };
    use teistro_core::catalogue::{Choghadiya, Graha, Kaala, Vara};
    use teistro_core::interval::Interval;
    use teistro_core::quantity::{JulianDay, Utc};

    fn daylight() -> Interval {
        Interval::literal(100.0, 100.5)
    }

    fn night() -> Interval {
        Interval::literal(100.5, 101.0)
    }

    #[test]
    fn the_walk_is_the_horas_own() {
        // Saturday's first hora is Saturn's and its second Jupiter's.
        assert_eq!(walk(Vara::Shanivara, 0), Graha::Saturn);
        assert_eq!(walk(Vara::Shanivara, DAY_STEP), Graha::Jupiter);
        assert_eq!(walk(Vara::Shanivara, 2 * DAY_STEP), Graha::Mars);
        // Monday's night begins with Venus, four weekdays on.
        assert_eq!(walk_vara(Vara::Somavara, NIGHT_START), Vara::Shukravara);
        assert_eq!(walk(Vara::Shukravara, 0), Graha::Venus);
        assert_eq!(walk(Vara::Shukravara, NIGHT_STEP), Graha::Mars);
    }

    #[test]
    fn every_graha_names_one_choghadiya() {
        for graha in [
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
        ] {
            assert!(choghadiya_of(graha).is_some(), "{graha:?}");
        }
        // The nodes and the outer planets rule no choghadiya.
        assert!(choghadiya_of(Graha::Rahu).is_none());
        assert_eq!(choghadiya_of(Graha::Moon), Some(Choghadiya::Amrit));
        assert_eq!(choghadiya_of(Graha::Saturn), Some(Choghadiya::Kaal));
    }

    #[test]
    fn a_monday_walks_the_measured_sequence() {
        let parts = choghadiya(daylight(), night(), Vara::Somavara);
        assert_eq!(parts.len(), 16);
        let day: Vec<Choghadiya> = parts
            .iter()
            .filter(|part| part.is_daytime)
            .map(|part| part.choghadiya)
            .collect();
        // The sequence the corpus records for a Monday's daylight.
        assert_eq!(
            day,
            vec![
                Choghadiya::Amrit,
                Choghadiya::Kaal,
                Choghadiya::Shubha,
                Choghadiya::Rog,
                Choghadiya::Udveg,
                Choghadiya::Char,
                Choghadiya::Laabh,
                Choghadiya::Amrit,
            ]
        );
        let dark: Vec<Choghadiya> = parts
            .iter()
            .filter(|part| !part.is_daytime)
            .map(|part| part.choghadiya)
            .collect();
        assert_eq!(
            dark,
            vec![
                Choghadiya::Char,
                Choghadiya::Rog,
                Choghadiya::Kaal,
                Choghadiya::Laabh,
                Choghadiya::Udveg,
                Choghadiya::Shubha,
                Choghadiya::Amrit,
                Choghadiya::Char,
            ]
        );
    }

    #[test]
    fn the_eighths_partition_each_half() {
        let parts = choghadiya(daylight(), night(), Vara::Ravivara);
        for pair in parts.windows(2) {
            assert_eq!(pair[0].at.to, pair[1].at.from, "no gap between eighths");
        }
        assert_eq!(parts[0].at.from, daylight().from);
        assert_eq!(parts[15].at.to, night().to);
    }

    #[test]
    fn a_half_with_no_arc_contributes_no_periods() {
        let none = Interval::literal(100.5, 100.5);
        let polar_day = choghadiya(Interval::literal(100.0, 101.0), none, Vara::Ravivara);
        assert_eq!(polar_day.len(), 8, "the night has none, not eight empties");
        assert!(polar_day.iter().all(|part| part.is_daytime));
        assert!(kaalas(none, Vara::Ravivara).is_empty(), "and no rahu kaal");
    }

    #[test]
    fn the_inauspicious_eighths_are_the_measured_table() {
        // Saturday: gulika the first eighth, rahu kaal the third,
        // yamaghanda the sixth.
        let found = kaalas(daylight(), Vara::Shanivara);
        assert_eq!(found.len(), 3);
        let eighth = |kaala: Kaala| {
            found
                .iter()
                .find(|found| found.kaala == kaala)
                .map(|found| daylight().part_at(found.at.from, 8))
        };
        assert_eq!(eighth(Kaala::GulikaKaala), Some(Some(0)));
        assert_eq!(eighth(Kaala::RahuKaala), Some(Some(2)));
        assert_eq!(eighth(Kaala::Yamaghanda), Some(Some(5)));
        // And on a Sunday rahu kaal is the last eighth of the day.
        let sunday = kaalas(daylight(), Vara::Ravivara);
        let rahu = sunday
            .iter()
            .find(|found| found.kaala == Kaala::RahuKaala)
            .expect("a Sunday has one");
        assert_eq!(daylight().part_at(rahu.at.from, 8), Some(7));
    }

    #[test]
    fn abhijit_is_the_eighth_muhurta_and_void_on_a_wednesday() {
        let found = muhurtas(daylight(), night(), Some(night()), Vara::Somavara);
        assert_eq!(found.daylight.len(), 15);
        assert_eq!(found.night.len(), 15);
        let abhijit = found.abhijit.expect("a day with daylight has one");
        assert_eq!(abhijit, found.daylight[7]);
        assert!(found.abhijit_effective);
        let wednesday = muhurtas(daylight(), night(), Some(night()), Vara::Budhavara);
        assert!(!wednesday.abhijit_effective, "void on a Wednesday");
        assert!(wednesday.abhijit.is_some(), "but still a muhurta");
    }

    #[test]
    fn brahma_muhurta_is_of_the_night_that_ends_at_sunrise() {
        let before = Interval::literal(99.5, 100.0);
        let found = muhurtas(daylight(), night(), Some(before), Vara::Somavara);
        let brahma = found.brahma.expect("the previous night is known");
        // The fourteenth of fifteen: it ends one muhurta before sunrise.
        let muhurta = before.days() / 15.0;
        assert!((brahma.to.get() - (before.to.get() - muhurta)).abs() < 1e-12);
        assert!((brahma.from.get() - (before.to.get() - 2.0 * muhurta)).abs() < 1e-12);
        // Without that night there is no Brahma muhurta rather than a wrong one.
        assert!(
            muhurtas(daylight(), night(), None, Vara::Somavara)
                .brahma
                .is_none()
        );
    }

    #[test]
    fn a_period_can_be_asked_what_is_running() {
        let parts = choghadiya(daylight(), night(), Vara::Ravivara);
        let at = JulianDay::<Utc>::literal(100.01);
        assert_eq!(
            choghadiya_at(&parts, at).map(|part| part.choghadiya),
            Some(Choghadiya::Udveg),
            "a Sunday's daylight opens with Udveg"
        );
        assert!(choghadiya_at(&parts, JulianDay::literal(50.0)).is_none());
        let found = kaalas(daylight(), Vara::Ravivara);
        assert!(!is_inauspicious(&found, at), "the first eighth is clear");
        assert!(is_inauspicious(&found, JulianDay::literal(100.44)));
    }
}
