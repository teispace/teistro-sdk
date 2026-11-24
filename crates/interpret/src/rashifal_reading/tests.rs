#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro_core::catalogue::{Graha, Nakshatra, Rashi, Tithi, Yoga};
use teistro_core::quantity::JulianDay;
use teistro_core::settings::ScoreBands;
use teistro_gochar::Transit;
use teistro_gochar::hits::{Hit, HitEvent, Motion};
use teistro_intl::{ClockTime, Value};
use teistro_rashifal::baseline::{BaselineScore, Panchanga, baseline_score};
use teistro_rashifal::{Period, PeriodEvent, RashiReading, RashifalRules, rashifal};

use super::{DayScore, Fact, Frame, MoonTurn, ReadingInput, ReadingItem, ReadingPlan, reading};

const START: f64 = 2_461_000.5;

/// A sky with Saturn in Pisces, the nodes opposite and the rest in Aries,
/// and two events: Jupiter enters Taurus early, Saturn turns late.
fn sky() -> [RashiReading; 12] {
    let events = [
        PeriodEvent {
            hit: Hit {
                instant: JulianDay::literal(START + 20.0),
                graha: Graha::Jupiter,
                event: HitEvent::SignIngress {
                    into: Rashi::Cancer,
                    motion: Motion::Direct,
                },
            },
            sign: Rashi::Cancer,
        },
        PeriodEvent {
            hit: Hit {
                instant: JulianDay::literal(START + 300.0),
                graha: Graha::Saturn,
                event: HitEvent::Station {
                    turns: Motion::Retrograde,
                },
            },
            sign: Rashi::Pisces,
        },
    ];
    sky_with(&events)
}

/// The same sky with other events.
fn sky_with(events: &[PeriodEvent]) -> [RashiReading; 12] {
    let mut transits = [Transit::new(Rashi::Aries, 5.0); 9];
    transits[Graha::Saturn as usize] = Transit::new(Rashi::Pisces, 12.0);
    transits[Graha::Jupiter as usize] = Transit::new(Rashi::Gemini, 20.0);
    transits[Graha::Venus as usize] = Transit::new(Rashi::Aquarius, 2.0);
    transits[Graha::Rahu as usize] = Transit::new(Rashi::Virgo, 3.0);
    transits[Graha::Ketu as usize] = Transit::new(Rashi::Pisces, 3.0);
    rashifal(&transits, events, &RashifalRules::default())
}

fn score(reading: &RashiReading, period: Period) -> BaselineScore {
    let panchanga = Panchanga {
        tithi: Tithi::ShuklaPratipada,
        yoga: Yoga::Priti,
        muhurta_yogas: 1,
    };
    baseline_score(
        reading,
        &[],
        &[false; 9],
        panchanga,
        period,
        ScoreBands::DEFAULT,
    )
}

fn span(period: Period) -> (f64, f64) {
    let days = match period {
        Period::Daily => 1.0,
        Period::Weekly => 7.0,
        Period::Monthly => 30.0,
        Period::Yearly => 365.0,
    };
    (START, START + days)
}

const WEEK: [DayScore; 7] = [
    DayScore { overall: 40 },
    DayScore { overall: 62 },
    DayScore { overall: 80 },
    DayScore { overall: 55 },
    DayScore { overall: 20 },
    DayScore { overall: 25 },
    DayScore { overall: 50 },
];

fn plan(
    reading: &RashiReading,
    period: Period,
    first_day: i64,
    moon_turn: Option<MoonTurn>,
) -> ReadingPlan {
    let days: &[DayScore] = if period == Period::Weekly { &WEEK } else { &[] };
    dated(reading, period, first_day, moon_turn, days, &[], &[])
}

/// A plan with its days, and its days' and events' dates, given.
fn dated(
    reading: &RashiReading,
    period: Period,
    first_day: i64,
    moon_turn: Option<MoonTurn>,
    days: &[DayScore],
    day_dates: &[Value],
    event_dates: &[Value],
) -> ReadingPlan {
    let score = score(reading, period);
    super::reading(&ReadingInput {
        reading,
        score: &score,
        period,
        first_day,
        span: span(period),
        bands: ScoreBands::DEFAULT,
        moon_turn,
        days,
        day_dates,
        event_dates,
    })
}

const PERIODS: [Period; 4] = [
    Period::Daily,
    Period::Weekly,
    Period::Monthly,
    Period::Yearly,
];

const TURN: MoonTurn = MoonTurn {
    time: ClockTime::new(13, 25, 0),
    from_house: 3,
    to_house: 4,
    good_after: false,
};

fn items(plan: &ReadingPlan) -> Vec<&ReadingItem> {
    std::iter::once(&plan.headline)
        .chain(plan.parts.iter().flat_map(|part| &part.items))
        .chain(&plan.why)
        .chain(plan.areas.iter().map(|area| &area.note))
        .collect()
}

#[test]
fn every_item_names_facts_the_answer_carries() {
    for read in &sky() {
        for period in PERIODS {
            let turn = (period == Period::Daily).then_some(TURN);
            let plan = plan(read, period, 739_000, turn);
            let score = score(read, period);
            let days = if period == Period::Weekly {
                WEEK.len()
            } else {
                0
            };
            for item in items(&plan) {
                assert!(
                    !item.rests_on.is_empty(),
                    "{} rests on nothing",
                    item.intent
                );
                for fact in &item.rests_on {
                    let exists = match *fact {
                        Fact::Transit { graha } => {
                            read.gochar.grahas.iter().any(|g| g.graha == graha)
                        }
                        Fact::Event { index } => index < read.events.len(),
                        Fact::Influence { index } => index < score.key_influences.len(),
                        Fact::Day { index } => index < days,
                        Fact::MoonTurn => turn.is_some(),
                        Fact::Saturn
                        | Fact::Band
                        | Fact::AreaBand { .. }
                        | Fact::Lucky
                        | Fact::Syllables => true,
                    };
                    assert!(
                        exists,
                        "{} names {fact:?}, which the answer lacks",
                        item.intent
                    );
                }
            }
        }
    }
}

#[test]
fn the_moon_is_named_on_a_day_only() {
    for read in &sky() {
        for period in [Period::Weekly, Period::Monthly, Period::Yearly] {
            let plan = plan(read, period, 739_000, None);
            let moon = items(&plan).into_iter().any(|item| {
                item.rests_on
                    .contains(&Fact::Transit { graha: Graha::Moon })
            });
            assert!(!moon, "{period:?} names the Moon for {:?}", read.rashi);
        }
    }
}

#[test]
fn a_period_keeps_its_caution_budget_and_never_ends_on_one() {
    for read in &sky() {
        for (period, most) in [(Period::Daily, 1), (Period::Weekly, 2)] {
            let plan = plan(read, period, 739_000, None);
            let cautions = plan
                .parts
                .iter()
                .flat_map(|part| &part.items)
                .filter(|item| item.caution)
                .count();
            assert!(
                cautions <= most,
                "{period:?} for {:?} says {cautions} cautions",
                read.rashi
            );
        }
        for period in PERIODS {
            let plan = plan(read, period, 739_000, None);
            for part in &plan.parts {
                let ends_on_caution = part.items.last().is_some_and(|item| item.caution);
                let all_cautions = part.items.iter().all(|item| item.caution);
                assert!(
                    !ends_on_caution || all_cautions,
                    "{period:?} {:?} ends on a caution",
                    part.frame
                );
            }
        }
    }
}

#[test]
fn the_same_period_reads_alike_and_the_next_day_differs() {
    let read = &sky()[3];
    let today = plan(read, Period::Daily, 739_000, None);
    assert_eq!(today, plan(read, Period::Daily, 739_000, None));
    let tomorrow = plan(read, Period::Daily, 739_001, None);
    for (a, b) in items(&today).into_iter().zip(items(&tomorrow)) {
        if a.intent == b.intent {
            assert_ne!(
                a.variant(),
                b.variant(),
                "{} reads alike two days running",
                a.intent
            );
        }
    }
}

#[test]
fn an_intent_said_twice_in_a_plan_is_worded_twice() {
    for read in &sky() {
        let plan = plan(read, Period::Yearly, 739_000, None);
        let said = items(&plan);
        for (i, a) in said.iter().enumerate() {
            for b in &said[i + 1..] {
                if a.intent == b.intent {
                    assert_ne!(a.variant(), b.variant(), "{} repeats a wording", a.intent);
                }
            }
        }
    }
}

#[test]
fn a_week_names_its_best_day_only_when_one_day_leads() {
    let read = &sky()[0];
    let week = plan(read, Period::Weekly, 739_000, None);
    assert_eq!(week.best_day, Some(2));
    assert!(super::strict_best(&[DayScore { overall: 60 }, DayScore { overall: 60 }]).is_none());
    assert_eq!(
        week.parts.iter().map(|part| part.frame).collect::<Vec<_>>()[..3],
        [Frame::Start, Frame::Middle, Frame::End]
    );
}

#[test]
fn a_day_splits_at_the_moon_s_turn() {
    let plan = plan(&sky()[0], Period::Daily, 739_000, Some(TURN));
    let frames: Vec<Frame> = plan.parts.iter().map(|part| part.frame).collect();
    assert_eq!(frames, [Frame::BeforeMoon, Frame::AfterMoon]);
    assert!(
        plan.why
            .iter()
            .any(|item| item.rests_on == [Fact::MoonTurn])
    );
}

#[test]
fn a_month_splits_at_its_largest_event_and_a_year_places_events_by_quarter() {
    let month = plan(&sky()[0], Period::Monthly, 739_000, None);
    assert_eq!(month.parts[1].frame, Frame::SecondHalf);
    // Jupiter's ingress outweighs Saturn's station in the guide's order.
    assert_eq!(month.parts[1].items[0].rests_on, [Fact::Event { index: 0 }]);
    let year = plan(&sky()[0], Period::Yearly, 739_000, None);
    let frames: Vec<Frame> = year.parts.iter().map(|part| part.frame).collect();
    assert_eq!(
        frames,
        [Frame::Slow, Frame::Quarter1, Frame::Quarter4, Frame::Whole]
    );
}

#[test]
fn a_sign_s_syllables_are_its_nine_padas() {
    let aries = super::syllables(Rashi::Aries);
    assert_eq!(aries.len(), 9);
    let ashwini = &Nakshatra::Ashwini.attributes().padas;
    assert_eq!(aries[0], ashwini[0].akshara);
    assert_eq!(aries[8], Nakshatra::Krittika.attributes().padas[0].akshara);
    let pisces = super::syllables(Rashi::Pisces);
    assert_eq!(pisces[8], Nakshatra::Revati.attributes().padas[3].akshara);
}

#[test]
fn the_love_and_family_score_is_the_rounded_mean() {
    let read = &sky()[0];
    let score = score(read, Period::Daily);
    let of = |life| {
        score
            .areas
            .iter()
            .find(|(area, _)| *area == life)
            .unwrap()
            .1
    };
    let mean = (u16::from(of(teistro_core::catalogue::LifeArea::Relationships))
        + u16::from(of(teistro_core::catalogue::LifeArea::Family)))
    .div_ceil(2);
    assert_eq!(
        u16::from(super::area_score(&score, super::ReadingArea::LoveFamily)),
        mean
    );
}

#[test]
fn reading_is_reachable_by_its_public_name() {
    let read = &sky()[0];
    let score = score(read, Period::Daily);
    let plan = reading(&ReadingInput {
        reading: read,
        score: &score,
        period: Period::Daily,
        first_day: 1,
        span: span(Period::Daily),
        bands: ScoreBands::DEFAULT,
        moon_turn: None,
        days: &[],
        day_dates: &[],
        event_dates: &[],
    });
    assert_eq!(plan.rashi, Rashi::Aries);
}

#[test]
fn every_item_carries_its_hedge_and_variant_as_slots() {
    for read in &sky() {
        for period in PERIODS {
            let turn = (period == Period::Daily).then_some(TURN);
            let plan = plan(read, period, 739_000, turn);
            for item in items(&plan) {
                assert!(item.variant().is_some(), "{} has no variant", item.intent);
                assert_eq!(
                    item.params.get("hedge"),
                    Some(&teistro_intl::Value::from(item.hedge.key())),
                    "{}'s hedge slot",
                    item.intent
                );
                assert!(item.intent.starts_with("sdk.voice.rashifal."));
            }
        }
    }
}

#[test]
fn an_event_is_said_once_in_a_plan() {
    for read in &sky() {
        for period in [Period::Weekly, Period::Monthly, Period::Yearly] {
            let plan = plan(read, period, 739_000, None);
            for index in 0..read.events.len() {
                let said = items(&plan)
                    .into_iter()
                    .filter(|item| item.rests_on.contains(&Fact::Event { index }))
                    .count();
                assert!(said <= 1, "{period:?} says event {index} {said} times");
            }
        }
    }
}

#[test]
fn every_intent_said_is_one_the_list_names() {
    let known = super::intents();
    for read in &sky() {
        for period in PERIODS {
            let turn = (period == Period::Daily).then_some(TURN);
            for item in items(&plan(read, period, 739_000, turn)) {
                assert!(
                    known.contains(&item.intent),
                    "{} is not listed",
                    item.intent
                );
            }
        }
    }
}

#[test]
fn a_good_moon_turn_is_said_as_likely() {
    let turn = MoonTurn {
        good_after: true,
        ..TURN
    };
    let plan = plan(&sky()[0], Period::Daily, 739_000, Some(turn));
    let said: Vec<&ReadingItem> = items(&plan)
        .into_iter()
        .filter(|item| item.rests_on == [Fact::MoonTurn])
        .collect();
    assert_eq!(said.len(), 2, "the why and the part after her turn");
    assert!(
        said.iter()
            .all(|item| item.hedge == super::Hedge::Likely && !item.caution)
    );
}

#[test]
fn a_short_week_reads_the_days_it_has_and_dates_its_best() {
    let days = [
        DayScore { overall: 50 },
        DayScore { overall: 70 },
        DayScore { overall: 60 },
    ];
    let dates = [Value::Int(1), Value::Int(2), Value::Int(3)];
    let events = [Value::Int(10), Value::Int(11)];
    let week = dated(
        &sky()[0],
        Period::Weekly,
        739_000,
        None,
        &days,
        &dates,
        &events,
    );
    let frames: Vec<Frame> = week.parts.iter().map(|part| part.frame).collect();
    assert_eq!(frames, [Frame::Start, Frame::Middle, Frame::Whole]);
    assert_eq!(week.best_day, Some(1));
    let best = items(&week)
        .into_iter()
        .find(|item| item.rests_on == [Fact::Day { index: 1 }])
        .unwrap();
    assert_eq!(best.params.get("date"), Some(&Value::Int(2)));
    let event = items(&week)
        .into_iter()
        .find(|item| item.rests_on == [Fact::Event { index: 0 }])
        .unwrap();
    assert_eq!(event.params.get("date"), Some(&Value::Int(10)));
}

#[test]
fn a_tied_week_names_no_best_day() {
    let days = [DayScore { overall: 60 }, DayScore { overall: 60 }];
    let week = dated(&sky()[0], Period::Weekly, 739_000, None, &days, &[], &[]);
    assert_eq!(week.best_day, None);
    let frames: Vec<Frame> = week.parts.iter().map(|part| part.frame).collect();
    assert_eq!(frames, [Frame::Start, Frame::Whole]);
}

#[test]
fn a_month_without_events_reads_whole_and_a_year_s_middle_quarters_hold_theirs() {
    let quiet = &sky_with(&[])[0];
    let month = plan(quiet, Period::Monthly, 739_000, None);
    let frames: Vec<Frame> = month.parts.iter().map(|part| part.frame).collect();
    assert_eq!(frames, [Frame::Whole]);
    let station = |day: f64, graha: Graha, sign: Rashi| PeriodEvent {
        hit: Hit {
            instant: JulianDay::literal(START + day),
            graha,
            event: HitEvent::Station {
                turns: Motion::Direct,
            },
        },
        sign,
    };
    let busy = &sky_with(&[
        station(100.0, Graha::Mars, Rashi::Aries),
        station(200.0, Graha::Mercury, Rashi::Aries),
    ])[0];
    let year = plan(busy, Period::Yearly, 739_000, None);
    let frames: Vec<Frame> = year.parts.iter().map(|part| part.frame).collect();
    assert_eq!(
        frames,
        [Frame::Slow, Frame::Quarter2, Frame::Quarter3, Frame::Whole]
    );
    let direct = items(&year)
        .into_iter()
        .filter(|item| item.intent.contains(".STATION_DIRECT."))
        .count();
    assert_eq!(direct, 2);
}

#[test]
fn no_areas_say_no_lines_and_a_part_of_cautions_keeps_its_order() {
    let read = &sky()[0];
    let score = score(read, Period::Daily);
    let input = ReadingInput {
        reading: read,
        score: &score,
        period: Period::Daily,
        first_day: 1,
        span: span(Period::Daily),
        bands: ScoreBands::DEFAULT,
        moon_turn: None,
        days: &[],
        day_dates: &[],
        event_dates: &[],
    };
    let mut say = super::Sayer::new(&input);
    assert_eq!(super::area_lines(&input, &[], &mut say), []);
    let plan = plan(read, Period::Daily, 739_000, None);
    let caution = |intent: &str| ReadingItem {
        intent: String::from(intent),
        caution: true,
        ..plan.headline.clone()
    };
    let mut cautions = [caution("a"), caution("b")];
    super::keep_items_caution_off_the_end(&mut cautions);
    assert_eq!([&*cautions[0].intent, &*cautions[1].intent], ["a", "b"]);
}
