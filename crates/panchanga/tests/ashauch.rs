//! The ashauch kernel held to the texts' cases (`03-design/ashauch.md`
//! §5.1, T1–T29), on synthetic days: day `k` rises at Julian day `k`,
//! sets at `k + 0.5`, and is 1 January 2024 plus `k` days. Its night's
//! thirds turn at `k + 0.8333`, its middle is `k + 0.75`, and its last
//! quarter begins at `k + 0.875`.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index fixed lists"
)]

use teistro_calendar::CalendarDate;
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Calendar, Masa, Tithi, Vara};
use teistro_core::error::Status;
use teistro_core::quantity::JulianDay;
use teistro_core::settings::LunarMonth as Convention;
use teistro_core::time::UtcOffset;
use teistro_panchanga::ashauch::{
    Ashauch, AshauchEvent, AshauchKind, AshauchRequest, AshauchTable, Clause, DayOneBy, Deceased,
    Length, Nearness, NightRule, Overlapped, PakshiniNight, Relation, Running, Sex, Upanayana,
    ashauch,
};
use teistro_panchanga::festival::{FestivalDay, MoonArcs, Moonrise, SunEntry};
use teistro_panchanga::month;

const DAYS: usize = 500;

/// 2024's months, a leap year, then 2025's.
const MONTHS: [u8; 24] = [
    31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
];

fn date(k: usize) -> CalendarDate {
    let mut left = k;
    for (index, &length) in MONTHS.iter().enumerate() {
        if left < usize::from(length) {
            return CalendarDate::defined(
                Calendar::Gregorian,
                2024 + i32::try_from(index / 12).unwrap(),
                u8::try_from(index % 12 + 1).unwrap(),
                u8::try_from(left + 1).unwrap(),
            );
        }
        left -= usize::from(length);
    }
    panic!("day {k} is past 2025");
}

fn day(k: usize) -> FestivalDay {
    #[allow(clippy::cast_precision_loss, reason = "a few hundred days")]
    let rise = k as f64;
    FestivalDay {
        date: date(k),
        vara: Vara::ALL[(k + 1) % 7],
        sunrise: JulianDay::literal(rise),
        sunset: JulianDay::literal(rise + 0.5),
        next_sunrise: JulianDay::literal(rise + 1.0),
        normal: true,
        month: month::of(
            Masa::Pausha,
            Tithi::ShuklaPratipada,
            Convention::Amanta,
            MonthKind::Nija,
        ),
        tithi: Vec::new(),
        nakshatra: Vec::new(),
        moonrise: Moonrise::Unsearched,
        sun_entry: SunEntry::Unsearched,
        moon_arcs: MoonArcs::Unsearched,
    }
}

fn days() -> Vec<FestivalDay> {
    (0..DAYS).map(day).collect()
}

const fn jd(at: f64) -> JulianDay<teistro_core::quantity::Utc> {
    JulianDay::literal(at)
}

fn death(at: f64) -> AshauchEvent {
    AshauchEvent::Death {
        at: jd(at),
        cremated_at: None,
        ahitagni: false,
        deceased: Deceased::default(),
    }
}

fn of(at: f64, deceased: Deceased) -> AshauchEvent {
    AshauchEvent::Death {
        at: jd(at),
        cremated_at: None,
        ahitagni: false,
        deceased,
    }
}

fn birth(at: f64, child: Option<Sex>) -> AshauchEvent {
    AshauchEvent::Birth { at: jd(at), child }
}

fn asked(key: &str, event: AshauchEvent, relation: Relation) -> AshauchRequest {
    AshauchRequest {
        key: key.to_owned(),
        event,
        relation,
        heard_at: None,
        abroad: false,
        nearness: Nearness::SameVillage,
        full_days: None,
        during: Vec::new(),
    }
}

fn under(table: &AshauchTable, request: &AshauchRequest) -> Ashauch {
    ashauch(table, request, &days(), &UtcOffset::UTC).unwrap()
}

fn answer(request: &AshauchRequest) -> Ashauch {
    under(&AshauchTable::dharmasindhu(), request)
}

fn heard(mut request: AshauchRequest, at: f64) -> AshauchRequest {
    request.heard_at = Some(jd(at));
    request
}

fn during(mut request: AshauchRequest, earlier: &[Ashauch]) -> AshauchRequest {
    request.during = earlier.iter().map(Running::from).collect();
    request
}

/// A child born `age` days before a death at 50.2, as the bands read it.
fn child(sex: Sex, age: f64) -> Deceased {
    Deceased {
        born: Some(jd(50.2 - age)),
        sex: Some(sex),
        ..Deceased::default()
    }
}

const YEAR: f64 = 365.2425;

fn ends(found: &Ashauch) -> f64 {
    found.end.get()
}

#[test]
fn t1_to_t3_a_sapinda_keeps_the_full_period_a_samanodaka_three_a_sagotra_one() {
    let full = answer(&asked("t1", death(1.2), Relation::Sapinda));
    assert_eq!(full.length, Length::Full);
    assert_eq!(full.days, Some(10));
    assert_eq!(full.day_one, date(1));
    assert_eq!(full.day_one_by, DayOneBy::Daylight);
    // Ten days from day 1 end at the sunrise beginning day 11.
    assert_eq!(ends(&full), 11.0);
    assert_eq!(full.last_day, Some(date(10)));
    assert!(matches!(full.clause, Clause::Relation { .. }));
    assert_eq!(
        ends(&answer(&asked("t2", death(1.2), Relation::Samanodaka))),
        4.0
    );
    assert_eq!(
        ends(&answer(&asked("t3", death(1.2), Relation::Sagotra))),
        2.0
    );
    let mut twelve = asked("t1-12", death(1.2), Relation::Sapinda);
    twelve.full_days = Some(12);
    assert_eq!(ends(&answer(&twelve)), 13.0);
}

#[test]
fn each_night_rule_gives_a_night_death_its_day() {
    let table = |night| AshauchTable {
        night,
        ..AshauchTable::dharmasindhu()
    };
    let first = |night, at, offset| {
        let found = ashauch(
            &table(night),
            &asked("k", death(at), Relation::Sapinda),
            &days(),
            &offset,
        )
        .unwrap();
        (found.day_one.clone(), ends(&found))
    };
    // K1: past the night's middle, inside its first two thirds.
    let utc = UtcOffset::UTC;
    assert_eq!(first(NightRule::Thirds, 1.80, utc), (date(1), 11.0));
    assert_eq!(first(NightRule::HalfNight, 1.80, utc), (date(2), 12.0));
    assert_eq!(first(NightRule::Sunrise, 1.80, utc), (date(1), 11.0));
    // K2: in the last third.
    assert_eq!(first(NightRule::Thirds, 1.90, utc), (date(2), 12.0));
    assert_eq!(first(NightRule::Sunrise, 1.90, utc), (date(1), 11.0));
    // K3: before the night's middle.
    assert_eq!(first(NightRule::HalfNight, 1.70, utc), (date(1), 11.0));
    // Six hours west, the clock's midnight is the Julian day's .75.
    let west = UtcOffset::literal(-6, 0, 0);
    assert_eq!(first(NightRule::CivilMidnight, 1.70, west), (date(1), 11.0));
    assert_eq!(first(NightRule::CivilMidnight, 1.80, west), (date(2), 12.0));
    // Six hours east, midnight falls before sunset: the whole night is
    // the day before's.
    let east = UtcOffset::literal(6, 0, 0);
    assert_eq!(first(NightRule::CivilMidnight, 1.95, east), (date(1), 11.0));
    let night = answer(&asked("by", death(1.80), Relation::Sapinda)).day_one_by;
    assert_eq!(
        night,
        DayOneBy::Night {
            rule: NightRule::Thirds
        }
    );
}

#[test]
fn a_sajyotih_ends_at_nightfall_or_daybreak_and_a_pakshini_on_the_next_day() {
    let at = |relation, at| ends(&answer(&asked("s", death(at), relation)));
    assert_eq!(at(Relation::Subject, 1.2), 1.5);
    assert_eq!(at(Relation::Subject, 1.6), 2.0);
    assert_eq!(at(Relation::MaternalGrandmother, 1.2), 2.5);
    assert_eq!(at(Relation::MaternalGrandmother, 1.6), 3.0);
    let nightfall = AshauchTable {
        pakshini_at_night: PakshiniNight::NextNightfall,
        ..AshauchTable::dharmasindhu()
    };
    let found = under(
        &nightfall,
        &asked("s", death(1.6), Relation::MaternalGrandmother),
    );
    assert_eq!(ends(&found), 2.5);
    assert_eq!(found.last_day, Some(date(2)));
    let bath = answer(&asked("b", death(1.2), Relation::OfAscetic));
    assert_eq!((ends(&bath), bath.last_day), (1.2, None));
}

#[test]
fn a_relation_row_reads_the_placement() {
    let at = |placement| {
        let mut request = asked("p", death(1.2), Relation::MarriedSibling);
        request.nearness = placement;
        answer(&request).length
    };
    assert_eq!(at(Nearness::ObserversHouse), Length::Days { days: 3 });
    assert_eq!(at(Nearness::SameVillage), Length::Pakshini);
    assert_eq!(at(Nearness::OtherVillage), Length::Days { days: 1 });
}

#[test]
fn t4_t6_a_son_before_the_upanayana_takes_his_band() {
    let son = |age, upanayana| Deceased {
        upanayana,
        ..child(Sex::Male, age)
    };
    let on_30th = son(29.5, None);
    assert_eq!(
        answer(&asked("t4", of(50.2, on_30th), Relation::Sapinda)).length,
        Length::Days { days: 1 }
    );
    assert_eq!(
        answer(&asked("t4", of(50.2, on_30th), Relation::Father)).length,
        Length::Days { days: 3 }
    );
    let buried = Deceased {
        buried: true,
        ..on_30th
    };
    assert_eq!(
        answer(&asked("t4", of(50.2, buried), Relation::Sapinda)).length,
        Length::Bath
    );
    let four = son(4.0 * YEAR, Some(Upanayana::NotDone));
    let found = answer(&asked("t6", of(50.2, four), Relation::Sapinda));
    assert_eq!(found.length, Length::Days { days: 3 });
    assert!(matches!(found.clause, Clause::Band { row: 4, .. }));
    // Past the upanayana, the full period.
    let done = son(9.0 * YEAR, Some(Upanayana::Done));
    assert_eq!(
        answer(&asked("t6", of(50.2, done), Relation::Sapinda)).length,
        Length::Full
    );
    // Where it is not performed, the band runs to the 16th year.
    let none = |age| son(age, Some(Upanayana::NotPerformed));
    assert_eq!(
        answer(&asked(
            "t6",
            of(
                50.2,
                Deceased {
                    married: Some(false),
                    ..none(12.0 * YEAR)
                }
            ),
            Relation::Sapinda
        ))
        .length,
        Length::Days { days: 3 }
    );
    assert_eq!(
        answer(&asked("t6", of(50.2, none(17.0 * YEAR)), Relation::Sapinda)).length,
        Length::Full
    );
}

#[test]
fn t5_a_child_dead_inside_its_birth_period_keeps_the_birth_period() {
    let found = answer(&asked(
        "t5",
        of(50.2, child(Sex::Male, 7.0)),
        Relation::Sapinda,
    ));
    assert_eq!(found.clause, Clause::BirthPeriodRuns);
    assert_eq!(found.kind, AshauchKind::Birth);
    // Born on day 43, so the birth period ends at day 53's sunrise.
    assert_eq!(found.day_one, date(43));
    assert_eq!(ends(&found), 53.0);
}

#[test]
fn t7_t8_a_daughter_takes_her_band() {
    let two = child(Sex::Female, 2.0 * YEAR);
    assert_eq!(
        answer(&asked("t7", of(50.2, two), Relation::Father)).length,
        Length::Days { days: 3 }
    );
    assert_eq!(
        answer(&asked("t7", of(50.2, two), Relation::Sapinda)).length,
        Length::Bath
    );
    let infant = child(Sex::Female, 100.0);
    assert_eq!(
        answer(&asked("t7", of(50.2, infant), Relation::Mother)).length,
        Length::Days { days: 1 }
    );
    let five = Deceased {
        betrothed: Some(false),
        married: Some(false),
        ..child(Sex::Female, 5.0 * YEAR)
    };
    assert_eq!(
        answer(&asked("t8", of(50.2, five), Relation::Sapinda)).length,
        Length::Days { days: 1 }
    );
    assert_eq!(
        answer(&asked("t8", of(50.2, five), Relation::Father)).length,
        Length::Days { days: 3 }
    );
    let betrothed = Deceased {
        betrothed: Some(true),
        ..five
    };
    assert_eq!(
        answer(&asked("t8", of(50.2, betrothed), Relation::Sapinda)).length,
        Length::Days { days: 3 }
    );
}

#[test]
fn a_fact_a_band_turns_on_is_asked_for_not_guessed() {
    let refused = |deceased, field| {
        let error = ashauch(
            &AshauchTable::dharmasindhu(),
            &asked("q", of(50.2, deceased), Relation::Sapinda),
            &days(),
            &UtcOffset::UTC,
        )
        .unwrap_err();
        assert_eq!(error.status, Status::InvalidArg, "{error}");
        assert_eq!(error.field(), Some(field), "{error}");
    };
    refused(child(Sex::Female, 5.0 * YEAR), "event.deceased.married");
    refused(
        Deceased {
            sex: None,
            ..child(Sex::Female, 2.0 * YEAR)
        },
        "event.deceased.sex",
    );
    refused(
        Deceased {
            upanayana: None,
            ..child(Sex::Male, 9.0 * YEAR)
        },
        "event.deceased.upanayana",
    );
    // An adult with no birth given had every rite.
    assert_eq!(
        answer(&asked("q", death(1.2), Relation::Sapinda)).length,
        Length::Full
    );
}

#[test]
fn t9_t10_a_loss_by_its_month() {
    let loss = |month, relation| {
        answer(&asked(
            "loss",
            AshauchEvent::Loss { at: jd(1.2), month },
            relation,
        ))
        .length
    };
    assert_eq!(loss(2, Relation::Mother), Length::Days { days: 3 });
    assert_eq!(loss(2, Relation::Father), Length::Bath);
    assert_eq!(loss(4, Relation::Mother), Length::Days { days: 4 });
    assert_eq!(loss(5, Relation::Mother), Length::Days { days: 5 });
    assert_eq!(loss(5, Relation::Sapinda), Length::Days { days: 3 });
    assert_eq!(loss(6, Relation::Mother), Length::Days { days: 6 });
}

#[test]
fn t11_news_inside_the_period_keeps_the_days_left() {
    let found = answer(&heard(asked("t11", death(1.2), Relation::Sapinda), 6.2));
    assert_eq!(found.clause, Clause::Remainder);
    assert_eq!(found.start.get(), 6.2);
    assert_eq!(found.day_one, date(1));
    assert_eq!(ends(&found), 11.0);
    // Heard the same day, nothing is left out.
    let same = answer(&heard(asked("t11", death(1.2), Relation::Sapinda), 1.4));
    assert!(matches!(same.clause, Clause::Relation { .. }));
}

#[test]
fn t12_t15_news_past_the_full_period_by_the_late_table() {
    let late = |after: f64, abroad| {
        let mut request = heard(asked("late", death(1.2), Relation::Sapinda), 1.2 + after);
        request.abroad = abroad;
        let found = under(&AshauchTable::dharmasindhu(), &request);
        (
            found.length,
            found.clause.clone(),
            ends(&found) - found.start.get(),
        )
    };
    let (length, clause, _) = late(50.0, false);
    assert_eq!(length, Length::Days { days: 3 });
    assert_eq!(
        clause,
        Clause::Late {
            within_days: 90,
            abroad: false
        }
    );
    assert!((late(50.0, false).2 - 2.8).abs() < 1e-9);
    assert_eq!(late(150.0, false).0, Length::Pakshini);
    assert_eq!(late(240.0, false).0, Length::Days { days: 1 });
    assert_eq!(late(300.0, false).0, Length::Sajyotih);
    let (length, clause, _) = late(420.0, false);
    assert_eq!((length, clause), (Length::Bath, Clause::LateBath));
    // Mādhava's abroad: three days only within three fortnights.
    assert_eq!(late(40.0, true).0, Length::Days { days: 3 });
    assert_eq!(late(50.0, true).0, Length::Pakshini);
}

#[test]
fn t16_a_parents_death_heard_however_late_is_the_full_period_from_the_hearing() {
    let found = answer(&heard(asked("t16", death(1.2), Relation::Child), 101.2));
    assert!(matches!(found.clause, Clause::RowHeardLate { .. }));
    assert_eq!(found.day_one, date(101));
    assert_eq!(ends(&found), 111.0);
}

#[test]
fn t17_a_birth_or_a_short_period_heard_past_it_is_a_bath() {
    let found = answer(&heard(
        asked("t17", birth(1.2, None), Relation::Father),
        20.2,
    ));
    assert_eq!(
        (found.length, found.clause),
        (Length::Bath, Clause::BirthHeardLate)
    );
    let short = answer(&heard(asked("s", death(1.2), Relation::Samanodaka), 8.2));
    assert_eq!(
        (short.length, short.clause),
        (Length::Bath, Clause::ShortHeardLate)
    );
}

/// A sapiṇḍa's death at 1.2, its period ending at day 11's sunrise.
fn first(relation: Relation, deceased: Deceased) -> Ashauch {
    answer(&asked("first", of(1.2, deceased), relation))
}

#[test]
fn t18_to_t21_a_death_during_another() {
    let earlier = first(Relation::Sapinda, Deceased::default());
    let second = |at, relation| {
        answer(&during(
            asked("second", death(at), relation),
            std::slice::from_ref(&earlier),
        ))
    };
    let t18 = second(4.2, Relation::Sapinda);
    assert_eq!(ends(&t18), 11.0);
    assert_eq!(
        t18.overlap,
        Some(Overlapped::EndsWithEarlier {
            earlier: "first".to_owned()
        })
    );
    let t19 = second(10.6, Relation::Sapinda);
    assert_eq!(ends(&t19), 13.0);
    assert_eq!(
        t19.overlap,
        Some(Overlapped::LastNight {
            earlier: "first".to_owned(),
            days: 2
        })
    );
    assert_eq!(t19.last_day, Some(date(12)));
    let t20 = second(10.9, Relation::Sapinda);
    assert_eq!(ends(&t20), 14.0);
    let t21 = second(10.6, Relation::Samanodaka);
    assert_eq!(ends(&t21), 11.0);
    // K8: after the first has ended, its own period.
    let k8 = second(11.2, Relation::Sapinda);
    assert_eq!((ends(&k8), k8.overlap), (21.0, None));
    // `[2, 2]` adds two in the last quarter too.
    let two = AshauchTable {
        overlap: teistro_panchanga::ashauch::Overlap {
            last_night: [2, 2],
            ..AshauchTable::dharmasindhu().overlap
        },
        ..AshauchTable::dharmasindhu()
    };
    let request = during(asked("second", death(10.9), Relation::Sapinda), &[earlier]);
    assert_eq!(ends(&under(&two, &request)), 13.0);
}

#[test]
fn t22_t23_a_birth_never_takes_in_a_death_but_a_death_takes_in_a_birth() {
    let born = answer(&asked("born", birth(1.2, None), Relation::Sapinda));
    let t22 = answer(&during(
        asked("t22", death(4.2), Relation::Sapinda),
        &[born],
    ));
    assert_eq!((ends(&t22), t22.overlap), (14.0, None));
    let died = first(Relation::Sapinda, Deceased::default());
    let t23 = answer(&during(
        asked("t23", birth(4.2, None), Relation::Sapinda),
        std::slice::from_ref(&died),
    ));
    assert_eq!(ends(&t23), 11.0);
    // T27: the birthing mother's own period is never taken in.
    let t27 = answer(&during(
        asked("t27", birth(4.2, None), Relation::Mother),
        &[died],
    ));
    assert_eq!((ends(&t27), t27.overlap), (14.0, None));
}

#[test]
fn t24_to_t26_a_son_for_his_father_and_his_mother() {
    let parent = |sex| Deceased {
        sex: Some(sex),
        ..Deceased::default()
    };
    let mother = first(Relation::Child, parent(Sex::Female));
    let father = first(Relation::Child, parent(Sex::Male));
    let then = |at, sex, earlier: &Ashauch| {
        answer(&during(
            asked("then", of(at, parent(sex)), Relation::Child),
            std::slice::from_ref(earlier),
        ))
    };
    let t24 = then(4.2, Sex::Male, &mother);
    assert_eq!((ends(&t24), t24.overlap), (14.0, None));
    let t25 = then(4.2, Sex::Female, &father);
    // The father's end at day 11's sunrise, and a pakṣiṇī begun by day.
    assert_eq!(ends(&t25), 12.5);
    assert_eq!(
        t25.overlap,
        Some(Overlapped::MotherAfterFather {
            earlier: "first".to_owned()
        })
    );
    let t26 = then(10.6, Sex::Female, &father);
    assert_eq!(ends(&t26), 13.0);
    // A son's period for a parent is not taken in by a sapiṇḍa's.
    let sapinda = first(Relation::Sapinda, Deceased::default());
    let own = then(4.2, Sex::Male, &sapinda);
    assert_eq!((ends(&own), own.overlap), (14.0, None));
}

#[test]
fn t28_a_married_daughter_for_her_parent() {
    let found = answer(&asked("t28", death(1.2), Relation::MarriedDaughter));
    assert_eq!(found.length, Length::Days { days: 3 });
    let late = answer(&heard(
        asked("t28", death(1.2), Relation::MarriedDaughter),
        40.2,
    ));
    assert_eq!(late.length, Length::Pakshini);
    assert!(matches!(late.clause, Clause::RowHeardLate { .. }));
    // A married daughter dying in her father's house: her parents three.
    let mut request = asked(
        "home",
        of(
            1.2,
            Deceased {
                sex: Some(Sex::Female),
                married: Some(true),
                ..Deceased::default()
            },
        ),
        Relation::Father,
    );
    request.nearness = Nearness::ObserversHouse;
    assert_eq!(answer(&request).length, Length::Days { days: 3 });
    // Elsewhere the table names no length, and says so.
    request.nearness = Nearness::OtherVillage;
    let error = ashauch(
        &AshauchTable::dharmasindhu(),
        &request,
        &days(),
        &UtcOffset::UTC,
    )
    .unwrap_err();
    assert_eq!(error.status, Status::Unsupported);
    assert_eq!(error.field(), Some("relation"));
}

#[test]
fn t29_a_birth_period_ends_at_the_naming_days_sunrise() {
    let ten = answer(&asked("t29", birth(1.2, None), Relation::Father));
    assert_eq!((ends(&ten), ten.last_day), (11.0, Some(date(10))));
    let mut request = asked("t29", birth(1.2, None), Relation::Father);
    request.full_days = Some(12);
    assert_eq!(ends(&answer(&request)), 13.0);
}

#[test]
fn the_mother_is_told_when_rites_open_after_a_son_or_a_daughter() {
    let rites = |child| {
        answer(&asked("m", birth(1.2, Some(child)), Relation::Mother))
            .rites_from
            .map(JulianDay::get)
    };
    assert_eq!(rites(Sex::Male), Some(31.0));
    assert_eq!(rites(Sex::Female), Some(41.0));
    assert_eq!(
        answer(&asked("f", birth(1.2, Some(Sex::Male)), Relation::Father)).rites_from,
        None
    );
}

#[test]
fn an_ahitagni_counts_from_the_cremation() {
    let found = answer(&asked(
        "agni",
        AshauchEvent::Death {
            at: jd(1.2),
            cremated_at: Some(jd(2.3)),
            ahitagni: true,
            deceased: Deceased::default(),
        },
        Relation::Sapinda,
    ));
    assert_eq!((found.day_one.clone(), ends(&found)), (date(2), 12.0));
}

#[test]
fn every_part_of_the_table_is_data_a_consumer_can_change() {
    let table = AshauchTable::dharmasindhu();
    let json = serde_json::to_string(&table).unwrap();
    let back: AshauchTable = serde_json::from_str(&json).unwrap();
    assert_eq!(back, table);
    // A community keeping twelve days for a maternal uncle changes that
    // row alone.
    let mut changed = table.clone();
    for row in &mut changed.relations {
        if row.relation == Relation::MaternalUncle {
            row.length = Length::Days { days: 12 };
        }
    }
    let uncle = asked("u", death(1.2), Relation::MaternalUncle);
    assert_eq!(ends(&under(&changed, &uncle)), 13.0);
    let sapinda = asked("s", death(1.2), Relation::Sapinda);
    assert_eq!(under(&changed, &sapinda), under(&table, &sapinda));
    let request = during(
        heard(asked("r", death(1.2), Relation::Sapinda), 3.0),
        &[answer(&sapinda)],
    );
    let json = serde_json::to_string(&request).unwrap();
    assert_eq!(
        serde_json::from_str::<AshauchRequest>(&json).unwrap(),
        request
    );
}

#[test]
fn a_table_or_a_request_that_cannot_be_counted_is_refused() {
    let mut table = AshauchTable::dharmasindhu();
    table.relations[0].length = Length::Days { days: 0 };
    let error = ashauch(
        &table,
        &asked("x", death(1.2), Relation::Sapinda),
        &days(),
        &UtcOffset::UTC,
    )
    .unwrap_err();
    assert_eq!(error.field(), Some("relations[0].length"));
    let refused = |request: AshauchRequest, field| {
        let error = ashauch(
            &AshauchTable::dharmasindhu(),
            &request,
            &days(),
            &UtcOffset::UTC,
        )
        .unwrap_err();
        assert_eq!(error.field(), Some(field), "{error}");
    };
    refused(
        heard(asked("x", death(5.2), Relation::Sapinda), 4.0),
        "heardAt",
    );
    let mut zero = asked("x", death(1.2), Relation::Sapinda);
    zero.full_days = Some(0);
    refused(zero, "fullDays");
    refused(
        asked(
            "x",
            AshauchEvent::Loss {
                at: jd(1.2),
                month: 7,
            },
            Relation::Mother,
        ),
        "event.month",
    );
    refused(
        asked(
            "x",
            AshauchEvent::Death {
                at: jd(1.2),
                cremated_at: None,
                ahitagni: true,
                deceased: Deceased::default(),
            },
            Relation::Sapinda,
        ),
        "event.crematedAt",
    );
    // A run with a gap inside the count.
    let mut gapped = days();
    gapped.remove(5);
    let error = ashauch(
        &AshauchTable::dharmasindhu(),
        &asked("x", death(1.2), Relation::Sapinda),
        &gapped,
        &UtcOffset::UTC,
    )
    .unwrap_err();
    assert_eq!(error.status, Status::Internal);
}

#[test]
fn a_request_names_the_instants_its_run_must_hold() {
    let request = during(
        heard(
            asked("a", of(50.2, child(Sex::Male, 7.0)), Relation::Sapinda),
            52.0,
        ),
        &[first(Relation::Sapinda, Deceased::default())],
    );
    let anchors: Vec<f64> = request.anchors().into_iter().map(JulianDay::get).collect();
    assert_eq!(anchors, vec![11.0, 43.2, 50.2, 52.0]);
    assert_eq!(request.reach(&AshauchTable::dharmasindhu()), 44);
}

fn mit(request: &AshauchRequest) -> Ashauch {
    under(&AshauchTable::mitakshara(), request)
}

#[test]
fn m1_the_mitakshara_divides_the_night_at_its_middle() {
    // Past the middle (k + 0.75), before the last third (k + 0.8333).
    let night = asked("m1", death(40.78), Relation::Sapinda);
    assert_eq!(answer(&night).day_one, date(40));
    assert_eq!(mit(&night).day_one, date(41));
    assert_eq!(AshauchTable::mitakshara().name, AshauchTable::MITAKSHARA);
}

#[test]
fn m2_the_parents_of_a_child_under_two_keep_the_full_period() {
    let under_mit = |deceased, relation| mit(&asked("m2", of(50.2, deceased), relation)).length;
    let infant = child(Sex::Male, 29.5);
    assert_eq!(under_mit(infant, Relation::Father), Length::Full);
    assert_eq!(
        under_mit(infant, Relation::Sapinda),
        Length::Days { days: 1 }
    );
    // DS gives the parents three.
    assert_eq!(
        answer(&asked("m2", of(50.2, infant), Relation::Father)).length,
        Length::Days { days: 3 }
    );
    let girl = child(Sex::Female, 1.5 * YEAR);
    assert_eq!(under_mit(girl, Relation::Mother), Length::Full);
    assert_eq!(under_mit(girl, Relation::Sapinda), Length::Bath);
    // Past two, the parents three.
    assert_eq!(
        under_mit(child(Sex::Male, 2.5 * YEAR), Relation::Father),
        Length::Days { days: 3 }
    );
}

#[test]
fn m3_a_childs_band_turns_on_its_chudakarana() {
    let son = |age, chudakarana| Deceased {
        chudakarana,
        ..child(Sex::Male, age)
    };
    let sapindas = |deceased| mit(&asked("m3", of(50.2, deceased), Relation::Sapinda)).length;
    // With teeth, before the cūḍā, a day; after it, three.
    assert_eq!(
        sapindas(son(1.5 * YEAR, Some(false))),
        Length::Days { days: 1 }
    );
    assert_eq!(
        sapindas(son(1.5 * YEAR, Some(true))),
        Length::Days { days: 3 }
    );
    // Left out, read from the age: not yet done before three.
    assert_eq!(sapindas(son(1.5 * YEAR, None)), Length::Days { days: 1 });
    assert_eq!(sapindas(son(2.5 * YEAR, None)), Length::Days { days: 1 });
    let three = Deceased {
        upanayana: Some(Upanayana::NotDone),
        ..son(3.5 * YEAR, None)
    };
    assert_eq!(sapindas(three), Length::Days { days: 3 });
    let daughter = |chudakarana| Deceased {
        chudakarana: Some(chudakarana),
        ..child(Sex::Female, 1.5 * YEAR)
    };
    assert_eq!(sapindas(daughter(false)), Length::Bath);
    assert_eq!(sapindas(daughter(true)), Length::Days { days: 1 });
}

#[test]
fn m4_late_news_by_vriddha_vasishthas_months_and_a_bath_from_another_land() {
    let late = |after: f64, abroad| {
        let mut request = heard(asked("m4", death(1.2), Relation::Sapinda), 1.2 + after);
        request.abroad = abroad;
        mit(&request).length
    };
    assert_eq!(late(50.0, false), Length::Days { days: 3 });
    assert_eq!(late(150.0, false), Length::Pakshini);
    assert_eq!(late(240.0, false), Length::Days { days: 1 });
    // DS keeps a sajyotiḥ here; the Mitakshara a bath.
    assert_eq!(late(300.0, false), Length::Bath);
    assert_eq!(late(40.0, true), Length::Bath);
}

#[test]
fn m5_the_teacher_the_maternal_kin_and_the_in_laws_by_the_commentary() {
    let length = |relation, nearness| {
        let mut request = asked("m5", death(1.2), relation);
        request.nearness = nearness;
        mit(&request).length
    };
    assert_eq!(
        length(Relation::MaternalUncle, Nearness::OtherVillage),
        Length::Days { days: 1 }
    );
    assert_eq!(
        length(Relation::MaternalUncle, Nearness::ObserversHouse),
        Length::Pakshini
    );
    assert_eq!(
        length(Relation::Pupil, Nearness::OtherVillage),
        Length::Days { days: 3 }
    );
    assert_eq!(
        length(Relation::SonInLaw, Nearness::SameVillage),
        Length::Pakshini
    );
    assert_eq!(
        length(Relation::ParentInLaw, Nearness::SameVillage),
        Length::Bath
    );
    assert_eq!(
        length(Relation::Adoption, Nearness::OtherVillage),
        Length::Days { days: 1 }
    );
    assert_eq!(
        length(Relation::Adoption, Nearness::SameVillage),
        Length::Days { days: 3 }
    );
    // A row the commentary leaves to others is the Dharmasindhu's.
    assert_eq!(
        length(Relation::Subject, Nearness::SameVillage),
        Length::Sajyotih
    );
}

#[test]
fn m6_a_married_daughter_away_from_her_fathers_house_is_none_on_his_side() {
    let married = Deceased {
        sex: Some(Sex::Female),
        married: Some(true),
        ..Deceased::default()
    };
    let mut away = asked("m6", of(1.2, married), Relation::Father);
    away.nearness = Nearness::OtherVillage;
    assert_eq!(mit(&away).length, Length::Bath);
    away.nearness = Nearness::ObserversHouse;
    assert_eq!(mit(&away).length, Length::Days { days: 3 });
}

#[test]
fn m7_the_mother_takes_part_in_rites_on_the_20th_or_the_30th_day() {
    let rites = |child| {
        mit(&asked("m7", birth(1.2, Some(child)), Relation::Mother))
            .rites_from
            .map(JulianDay::get)
    };
    assert_eq!(rites(Sex::Male), Some(21.0));
    assert_eq!(rites(Sex::Female), Some(31.0));
}

#[test]
fn m8_the_mitakshara_table_is_counted() {
    AshauchTable::mitakshara().check().unwrap();
    let json = serde_json::to_string(&AshauchTable::mitakshara()).unwrap();
    let back: AshauchTable = serde_json::from_str(&json).unwrap();
    assert_eq!(back, AshauchTable::mitakshara());
}
