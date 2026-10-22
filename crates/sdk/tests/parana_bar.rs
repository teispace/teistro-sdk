//! The parana's nakshatra bar (`03-design/festival-rules.md` §8.5, C700)
//! over a real sky: *Dharmasindhu*'s three shares of a nakshatra, the
//! first quarter of Anuradha, the middle third of Shravana and the last
//! quarter of Revati, after the bright 11ths of Ashadha, Bhadrapada and
//! Kartika, 2000 to 2035 at Delhi and Kathmandu.
//!
//! Each barred share is searched at its own bounds. Its edges stand where
//! pyswisseph (Moshier, Lahiri) puts the Moon's crossings, within 40
//! seconds, the time the Moon takes over the 20″ to 30″ the built-in's
//! theories stand from a modern ephemeris
//! (`03-design/builtin-ephemeris-speed.md`); a whole nakshatra's bar
//! shares its start and end with the quarters' searches, to the
//! millisecond.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Calendar, Nakshatra};
use teistro::festival::{EkadashiFast, EkadashiRule, NakshatraBar, ParanaEdge};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};

fn delhi() -> (Place, UtcOffset) {
    (
        Place::new(
            Latitude::literal(28.6139),
            Longitude::literal(77.209),
            Altitude::literal(216.0),
        ),
        UtcOffset::literal(5, 30, 0),
    )
}

fn kathmandu() -> (Place, UtcOffset) {
    (
        Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        ),
        UtcOffset::literal(5, 45, 0),
    )
}

/// Every fast of the shipped Ekadashi rules from 2000 to 2035, each
/// rule's parana barred by `bars`.
fn fasts((place, clock): (Place, UtcOffset), bars: &[NakshatraBar]) -> Vec<EkadashiFast> {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let request = FestivalRequest::from(FestivalPack::Dharmasindhu);
    let barred: Vec<EkadashiRule> = request
        .ekadashis()
        .iter()
        .cloned()
        .map(|mut rule| {
            rule.parana.as_mut().unwrap().bars = bars.to_vec();
            rule
        })
        .collect();
    let request = request.with_ekadashis(barred);
    (2000..2036)
        .flat_map(|year| {
            sdk.almanac()
                .festivals(
                    &CalendarDate::defined(Calendar::Gregorian, year, 1, 1),
                    &CalendarDate::defined(Calendar::Gregorian, year, 12, 31),
                    &place,
                    clock,
                    &request,
                )
                .unwrap()
                .value
                .ekadashis
        })
        .collect()
}

/// The Smarta fast whose parana falls on a day, and its barred share.
fn on(fasts: &[EkadashiFast], (year, month, day): (i32, u8, u8)) -> &EkadashiFast {
    fasts
        .iter()
        .filter(|fast| fast.rule == "EKADASHI_SMARTA")
        .find(|fast| {
            let parana = fast.parana.as_ref().unwrap();
            (parana.day.year, parana.day.month, parana.day.day) == (year, month, day)
        })
        .unwrap()
}

fn seconds(a: f64, b: f64) -> f64 {
    (a - b).abs() * 86_400.0
}

/// The parana day, the nakshatra, and pyswisseph's crossings of the
/// share's two bounds, UTC Julian days.
type Row = ((i32, u8, u8), Nakshatra, f64, f64);

const ROWS: [Row; 5] = [
    // Revati's share outlasts a short 12th: set aside.
    (
        (2007, 11, 22),
        Nakshatra::Revati,
        2_454_426.388_282_8,
        2_454_426.611_581_7,
    ),
    // The meal waits past Shravana's middle third.
    (
        (2008, 9, 12),
        Nakshatra::Shravana,
        2_454_721.483_207_3,
        2_454_721.836_526_1,
    ),
    // The morning ends where Revati's last quarter begins.
    (
        (2020, 11, 26),
        Nakshatra::Revati,
        2_459_179.878_391_6,
        2_459_180.160_029_5,
    ),
    (
        (2028, 7, 3),
        Nakshatra::Anuradha,
        2_461_955.586_875_1,
        2_461_955.836_271_5,
    ),
    // Kathmandu, 2025-07-07, where the published reference keeps no bar
    // and opens the meal at sunrise (C700): Anuradha's first quarter
    // holds it to 05:36.
    (
        (2025, 7, 7),
        Nakshatra::Anuradha,
        2_460_863.216_670_7,
        2_460_863.494_237_4,
    ),
];

#[test]
fn each_barred_share_is_where_a_modern_ephemeris_puts_the_moon() {
    let (at_delhi, at_kathmandu) = (
        fasts(delhi(), &NakshatraBar::dharmasindhu()),
        fasts(kathmandu(), &NakshatraBar::dharmasindhu()),
    );
    for (k, &(day, nakshatra, from, to)) in ROWS.iter().enumerate() {
        let fasts = if k == 4 { &at_kathmandu } else { &at_delhi };
        let parana = on(fasts, day).parana.as_ref().unwrap();
        let barred = parana.barred.unwrap();
        assert_eq!(barred.bar.nakshatra, nakshatra, "{day:?}");
        assert!(seconds(barred.span.from.get(), from) < 40.0, "{day:?}");
        assert!(seconds(barred.span.to.get(), to) < 40.0, "{day:?}");
        let kept = parana.kept.unwrap();
        match k {
            0 => assert!(barred.set_aside && kept.to_by == ParanaEdge::TithiEnd),
            1 | 4 => assert_eq!(
                (kept.from_by, kept.span.from),
                (ParanaEdge::Nakshatra, barred.span.to)
            ),
            _ => assert_eq!(
                (kept.to_by, kept.span.to),
                (ParanaEdge::Nakshatra, barred.span.from)
            ),
        }
    }
}

/// Over 36 years each place keeps 91 fasts a share meets, 6 of them with
/// the bar set aside; every kept window lies outside a bar not set aside,
/// and a bar is set aside only when it outlasts the tithi the meal is
/// kept in. One parana keeps no window: on 2032-09-16 the hari vasara
/// outlasts the morning, Shravana's middle third holds the afternoon, and
/// the 12th outlasts the share, so the bar stands and the rule's windows
/// leave nothing.
#[test]
fn a_bar_keeps_the_meal_outside_its_share_unless_it_outlasts_the_tithi() {
    for place in [delhi(), kathmandu()] {
        let fasts = fasts(place, &NakshatraBar::dharmasindhu());
        let barred: Vec<_> = fasts
            .iter()
            .filter_map(|fast| {
                let parana = fast.parana.as_ref().unwrap();
                Some((parana, parana.barred?))
            })
            .collect();
        let aside = barred.iter().filter(|(_, barred)| barred.set_aside).count();
        assert_eq!((barred.len(), aside), (91, 6));
        let none: Vec<_> = barred
            .iter()
            .filter(|(parana, _)| parana.kept.is_none())
            .map(|(parana, barred)| {
                let day = &parana.day;
                ((day.year, day.month, day.day), barred.set_aside)
            })
            .collect();
        assert_eq!(none, [((2032, 9, 16), false)]);
        for (parana, barred) in barred {
            let Some(kept) = parana.kept else { continue };
            if barred.set_aside {
                assert!(
                    kept.span.to.get() <= barred.span.to.get(),
                    "{:?}",
                    parana.day
                );
            } else {
                assert!(
                    kept.span.to.get() <= barred.span.from.get()
                        || kept.span.from.get() >= barred.span.to.get(),
                    "{:?}",
                    parana.day
                );
            }
        }
    }
}

/// The verse's whole nakshatras and *Dharmasindhu*'s shares are searched
/// apart, and meet where they must: Anuradha's whole stay starts with its
/// first quarter, Revati's ends with its last.
#[test]
fn a_whole_nakshatra_s_bar_meets_its_quarters_at_their_shared_bounds() {
    let (whole, shares) = (
        fasts(delhi(), &NakshatraBar::verse()),
        fasts(delhi(), &NakshatraBar::dharmasindhu()),
    );
    let stays = |fasts: &[EkadashiFast], nakshatra: Nakshatra| -> Vec<(f64, f64)> {
        fasts
            .iter()
            .filter_map(|fast| fast.parana.as_ref()?.barred)
            .filter(|barred| barred.bar.nakshatra == nakshatra)
            .map(|barred| (barred.span.from.get(), barred.span.to.get()))
            .collect()
    };
    let mut met = 0;
    for (nakshatra, start) in [(Nakshatra::Anuradha, true), (Nakshatra::Revati, false)] {
        let whole = stays(&whole, nakshatra);
        for (from, to) in stays(&shares, nakshatra) {
            let found = whole.iter().find(|(a, b)| *a <= from && to <= *b).unwrap();
            let (theirs, ours) = if start {
                (found.0, from)
            } else {
                (found.1, to)
            };
            assert!(seconds(theirs, ours) < 1e-3, "{nakshatra:?} at {from}");
            met += 1;
        }
    }
    assert!(met > 20, "{met} shares compared");
}

/// No shipped rule bars: the default parana answers as the published
/// reference does, and names no bar.
#[test]
fn the_shipped_rules_bar_nothing() {
    let fasts = fasts(delhi(), &[]);
    assert!(fasts.len() > 36 * 24 * 3);
    assert!(
        fasts
            .iter()
            .all(|fast| fast.parana.as_ref().unwrap().barred.is_none())
    );
    assert!(
        EkadashiRule::dharmasindhu().iter().all(|rule| rule
            .parana
            .as_ref()
            .unwrap()
            .bars
            .is_empty())
    );
}
