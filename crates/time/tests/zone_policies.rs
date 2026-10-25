//! Each daylight-saving, unknown-time and leap-second policy, on the
//! transitions the embedded tzdb records (`resolve`'s errors section).
//! The instants are worked by hand from the zone lines:
//!
//! - `Asia/Kathmandu`: `5:30 - +0530 1986` then `5:45 - +0545`, so the
//!   clocks went from 00:00 to 00:15 on 1 January 1986 and 00:05 never
//!   happened;
//! - `America/New_York` in 2024: 02:00 EST became 03:00 EDT on 10 March,
//!   and 02:00 EDT became 01:00 EST on 3 November, so 01:30 happened twice;
//! - the leap second at the end of 31 December 2016, and none at the end
//!   of 30 June 2017.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    reason = "tests fail by panicking; a fixed offset's instant is exact"
)]

use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Calendar;
use teistro_core::error::{Detail, Error, Status};
use teistro_core::quantity::{JulianDay, Longitude, Utc};
use teistro_core::settings::{DstGap, DstOverlap, UnknownTime};
use teistro_time::{
    Chosen, CivilDateTime, CivilTime, DstOutcome, EmbeddedTzdb, Policy, Resolved, UtcOffset,
    Warning, ZoneSource, ZoneSpec, civil_of, resolve,
};

/// A hundredth of a second, in days.
const TOLERANCE_DAYS: f64 = 0.01 / 86_400.0;

fn civil(y: i32, m: u8, d: u8, (hour, minute, second): (u8, u8, u8)) -> CivilDateTime {
    CivilDateTime::at(
        CalendarDate::defined(Calendar::Gregorian, y, m, d),
        CivilTime::new(hour, minute, second).unwrap(),
    )
}

fn in_zone(civil: &CivilDateTime, zone: &str, policy: Policy) -> Result<Resolved, Error> {
    resolve(
        civil,
        &ZoneSpec::iana(zone),
        &policy,
        EmbeddedTzdb::shared(),
    )
}

fn assert_at(resolved: &Resolved, jd: f64) {
    assert!(
        (resolved.instant.get() - jd).abs() < TOLERANCE_DAYS,
        "{} against {jd} ({:.3} s off)",
        resolved.instant,
        (resolved.instant.get() - jd) * 86_400.0
    );
}

fn assert_refused(error: &Error, detail: Option<Detail>, field: &str) {
    assert_eq!(error.status, Status::InvalidArg, "{error}");
    assert_eq!(error.detail, detail, "{error}");
    assert_eq!(error.field(), Some(field), "{error}");
    assert!(error.hint().is_some(), "a refusal says what to do: {error}");
}

/// Kathmandu's move to +05:45 left a quarter-hour no clock showed:
/// refused under `ERROR`, read at the old offset under `SHIFT_FORWARD`.
#[test]
fn a_time_in_kathmandus_1986_gap_is_refused_or_shifted() {
    let lost = civil(1986, 1, 1, (0, 5, 0));
    let refused = in_zone(
        &lost,
        "Asia/Kathmandu",
        Policy {
            gap: DstGap::Error,
            ..Policy::default()
        },
    )
    .unwrap_err();
    assert_refused(&refused, Some(Detail::DstGap), "time");
    assert!(
        refused.message.contains("+05:30") && refused.message.contains("+05:45"),
        "both offsets are named: {}",
        refused.message
    );

    let shifted = in_zone(
        &lost,
        "Asia/Kathmandu",
        Policy {
            gap: DstGap::ShiftForward,
            ..Policy::default()
        },
    )
    .unwrap();
    // 00:05 at +05:30 is 18:35 UTC on 31 December 1985.
    assert_at(&shifted, 2_446_431.5 - (5.0 + 25.0 / 60.0) / 24.0);
    assert_eq!(shifted.zone.offset.seconds(), 5 * 3600 + 30 * 60);
    assert_eq!(
        shifted.zone.dst,
        DstOutcome::Gap {
            shifted_by_seconds: 15 * 60
        }
    );
    assert!(shifted.zone.has(Warning::DstGapShifted));

    // A quarter-hour later the clocks show it, at the new offset.
    let after = in_zone(
        &civil(1986, 1, 1, (0, 15, 0)),
        "Asia/Kathmandu",
        Policy::default(),
    )
    .unwrap();
    assert_eq!(after.zone.offset.seconds(), 5 * 3600 + 45 * 60);
    assert_eq!(after.zone.dst, DstOutcome::None);
}

/// New York's spring-forward hour: the same two answers, an hour long.
#[test]
fn a_time_in_new_yorks_spring_gap_is_refused_or_shifted() {
    let lost = civil(2024, 3, 10, (2, 30, 0));
    let refused = in_zone(
        &lost,
        "America/New_York",
        Policy {
            gap: DstGap::Error,
            ..Policy::default()
        },
    )
    .unwrap_err();
    assert_refused(&refused, Some(Detail::DstGap), "time");

    let shifted = in_zone(
        &lost,
        "America/New_York",
        Policy {
            gap: DstGap::ShiftForward,
            ..Policy::default()
        },
    )
    .unwrap();
    // 02:30 EST is 07:30 UTC, which the clocks showed as 03:30 EDT.
    assert_at(&shifted, 2_460_379.5 + 7.5 / 24.0);
    assert_eq!(
        shifted.zone.dst,
        DstOutcome::Gap {
            shifted_by_seconds: 3600
        }
    );
}

/// New York's repeated hour: either occurrence on request, or a refusal
/// naming both.
#[test]
fn a_time_in_new_yorks_autumn_overlap_is_chosen_or_refused() {
    let twice = civil(2024, 11, 3, (1, 30, 0));
    for (overlap, chosen, utc_hours) in [
        (DstOverlap::Earlier, Chosen::Earlier, 5.5),
        (DstOverlap::Later, Chosen::Later, 6.5),
    ] {
        let read = in_zone(
            &twice,
            "America/New_York",
            Policy {
                overlap,
                ..Policy::default()
            },
        )
        .unwrap();
        assert_at(&read, 2_460_617.5 + utc_hours / 24.0);
        assert_eq!(read.zone.dst, DstOutcome::Overlap { chosen });
        assert!(read.zone.has(Warning::DstAmbiguous));
    }
    let refused = in_zone(
        &twice,
        "America/New_York",
        Policy {
            overlap: DstOverlap::Error,
            ..Policy::default()
        },
    )
    .unwrap_err();
    assert_refused(&refused, None, "time");
    assert!(
        refused.message.contains("-04:00") && refused.message.contains("-05:00"),
        "both occurrences are named: {}",
        refused.message
    );
}

/// A birth without a time takes the policy's time and says so, or is
/// refused; `SUNRISE` needs the place, which `resolve` does not have.
#[test]
fn a_date_without_a_time_follows_the_unknown_time_policy() {
    let day = CivilDateTime::date_only(CalendarDate::defined(Calendar::Gregorian, 2024, 6, 1));
    for (unknown_time, hours) in [(UnknownTime::Noon, 12.0), (UnknownTime::Midnight, 0.0)] {
        let read = in_zone(
            &day,
            "Asia/Kathmandu",
            Policy {
                unknown_time,
                ..Policy::default()
            },
        )
        .unwrap();
        // 1 June 2024 at +05:45.
        assert_at(&read, 2_460_462.5 + (hours - 5.75) / 24.0);
        assert!(!read.zone.time_known);
        assert!(read.zone.has(Warning::TimeUnknownFallback));
    }
    for unknown_time in [UnknownTime::Refuse, UnknownTime::Sunrise] {
        let refused = in_zone(
            &day,
            "Asia/Kathmandu",
            Policy {
                unknown_time,
                ..Policy::default()
            },
        )
        .unwrap_err();
        assert_refused(&refused, Some(Detail::TimeUnknown), "time");
    }
}

/// 23:59:60 on a day that ended with a leap second is that day's end,
/// in UTC and in a zone; on a day without one it is refused.
#[test]
fn a_leap_second_folds_onto_midnight_only_where_one_was_added() {
    let utc = ZoneSpec::Fixed {
        offset: UtcOffset::UTC,
    };
    let at = |civil: &CivilDateTime, zone: &ZoneSpec| {
        resolve(civil, zone, &Policy::default(), EmbeddedTzdb::shared())
    };
    let midnight_2017 = 2_457_754.5;
    let folded = at(&civil(2016, 12, 31, (23, 59, 60)), &utc).unwrap();
    assert_eq!(folded.instant.get(), midnight_2017);
    assert!(folded.zone.has(Warning::LeapSecondFolded));
    assert!(folded.civil.time.is_some_and(CivilTime::is_leap_second));

    // The same second as Kathmandu's clocks showed it.
    let local = at(
        &civil(2017, 1, 1, (5, 44, 60)),
        &ZoneSpec::iana("Asia/Kathmandu"),
    )
    .unwrap();
    assert_at(&local, midnight_2017);
    assert!(local.zone.has(Warning::LeapSecondFolded));

    let refused = at(&civil(2017, 6, 30, (23, 59, 60)), &utc).unwrap_err();
    assert_refused(&refused, Some(Detail::NonexistentDate), "time");
}

/// A fixed offset reads the clock as given, with no database behind it.
#[test]
fn a_fixed_offset_is_read_as_given() {
    let offset = UtcOffset::try_from_seconds(5 * 3600 + 45 * 60).unwrap();
    let read = resolve(
        &civil(2024, 3, 10, (12, 0, 0)),
        &ZoneSpec::Fixed { offset },
        &Policy::default(),
        EmbeddedTzdb::shared(),
    )
    .unwrap();
    assert_eq!(read.instant.get(), 2_460_379.5 + 6.25 / 24.0);
    assert_eq!(read.zone.source, ZoneSource::Manual);
    assert_eq!(read.zone.tzdb_version, "manual");
    assert_eq!(read.zone.abbreviation, None);
    assert_eq!(read.zone.dst, DstOutcome::None);
    assert_eq!(ZoneSpec::Fixed { offset }.to_string(), offset.to_string());
}

/// A calendar `resolve` cannot read without a context is refused, naming
/// the field and the call that can.
#[test]
fn a_calendar_that_needs_an_ephemeris_is_refused_by_resolve() {
    let lunisolar = CivilDateTime::at(
        CalendarDate::defined(Calendar::IndianLunisolar, 2081, 1, 1),
        CivilTime::NOON,
    );
    let refused = in_zone(&lunisolar, "Asia/Kathmandu", Policy::default()).unwrap_err();
    assert_eq!(refused.status, Status::Unsupported, "{refused}");
    assert_eq!(refused.field(), Some("date.calendar"));
    assert!(
        refused.message.contains("resolve_with"),
        "{}",
        refused.message
    );
}

/// A local mean clock is no whole number of minutes off UTC, so its
/// 23:59:60 cannot be the second a leap adds at a UTC midnight.
#[test]
fn a_leap_second_on_a_clock_off_the_minute_is_refused() {
    let kathmandu_mean = ZoneSpec::LocalMean {
        longitude: Longitude::literal(85.324),
    };
    let refused = resolve(
        &civil(2016, 12, 31, (23, 59, 60)),
        &kathmandu_mean,
        &Policy::default(),
        EmbeddedTzdb::shared(),
    )
    .unwrap_err();
    assert_refused(&refused, Some(Detail::NonexistentDate), "time");
}

/// Reading an instant back past the leap table's word warns.
#[test]
fn an_instant_read_back_past_the_leap_table_warns() {
    let utc = ZoneSpec::Fixed {
        offset: UtcOffset::UTC,
    };
    let tzdb = EmbeddedTzdb::shared();
    // 1 January 2200, 00:00 UTC.
    let far = JulianDay::<Utc>::try_new(2_524_593.5).unwrap();
    assert!(
        civil_of(far, &utc, tzdb)
            .unwrap()
            .1
            .has(Warning::LeapTableExpired)
    );
    let j2000 = JulianDay::<Utc>::try_new(2_451_545.0).unwrap();
    assert!(
        !civil_of(j2000, &utc, tzdb)
            .unwrap()
            .1
            .has(Warning::LeapTableExpired)
    );
}
