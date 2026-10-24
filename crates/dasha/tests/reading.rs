//! A dasha's reading read back: what a stored document infers from its
//! rows (a graha dasha's lords, division and shares, the firdaria's nodes,
//! the decennials' order and division, a time lord's start) rebuilds the
//! kernel it was written from, and the cursor over every kernel answers as
//! the kernel it holds.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test fails by panicking and indexes what it built"
)]

use teistro_core::angle::Nas;
use teistro_core::catalogue::{DashaSystem, Graha, Nakshatra, Rashi};
use teistro_core::quantity::{Degrees, Depth, JulianDay};
use teistro_core::settings::{
    AfterCycle, AshtottariGrouping, DecennialDivision, FirdariaNodes, GrahaDivision, YearLength,
    root,
};
use teistro_dasha::{
    Birth, Dasha, DashaCursor, DashaDefinition, DashaReading, DecennialDasha, FirdariaDasha,
    GrahaDasha, KalachakraDasha, KalachakraRules, Lord, Period, PeriodRow, ProfectionDasha,
    RashiChart, RashiDasha, RashiDefinition, RashiRules, ReleasingDasha, Rules, Timeline,
    UduDefinition, decennial_order, rashi_row, row,
};

const BIRTH: f64 = 2_451_545.0;

/// The seven's longitudes, for the order the decennials run in.
const PLACES: [(Graha, f64); 7] = [
    (Graha::Sun, 280.0),
    (Graha::Moon, 45.0),
    (Graha::Mars, 120.0),
    (Graha::Mercury, 300.0),
    (Graha::Jupiter, 200.0),
    (Graha::Venus, 330.0),
    (Graha::Saturn, 10.0),
];

fn birth() -> JulianDay<teistro_core::quantity::Utc> {
    JulianDay::literal(BIRTH)
}

fn depth(levels: u8) -> Depth {
    Depth::try_new(levels).unwrap()
}

fn rules(system: DashaSystem) -> Rules {
    Rules::of(&root().dasha, system)
}

/// Every period a timeline answers to `levels`, depth first: what a
/// reading's rows are written from.
fn tree(timeline: &impl Timeline, levels: usize) -> Vec<Period> {
    fn walk(timeline: &impl Timeline, period: Period, levels: usize, out: &mut Vec<Period>) {
        out.push(period);
        if period.path.depth() < levels {
            for child in timeline.children(&period) {
                walk(timeline, child, levels, out);
            }
        }
    }
    let mut out = Vec::new();
    for maha in timeline.mahadashas() {
        walk(timeline, maha, levels, &mut out);
    }
    out
}

fn graha(lords: &[(Graha, f64)], division: GrahaDivision) -> GrahaDasha {
    GrahaDasha::new(
        lords.to_vec(),
        birth(),
        YearLength::Julian36525,
        division,
        AfterCycle::End,
    )
    .unwrap()
}

/// A reading under the Julian year every kernel here is built with.
fn time_lord(system: DashaSystem, timeline: &impl Timeline, levels: u8) -> DashaReading {
    let rules = Rules {
        year_length: YearLength::Julian36525,
        ..rules(system)
    };
    DashaReading::of_time_lord(system, timeline, rules, depth(levels))
}

/// The nine with unequal years.
const UNEQUAL: [(Graha, f64); 9] = [
    (Graha::Sun, 6.0),
    (Graha::Moon, 10.0),
    (Graha::Mars, 7.0),
    (Graha::Rahu, 18.0),
    (Graha::Jupiter, 16.0),
    (Graha::Saturn, 19.0),
    (Graha::Mercury, 17.0),
    (Graha::Ketu, 7.0),
    (Graha::Venus, 20.0),
];

#[test]
fn a_period_s_level_is_the_places_in_its_path() {
    let dasha = graha(&UNEQUAL, GrahaDivision::Proportional);
    let reading = time_lord(DashaSystem::Naisargika, &dasha, 3);
    let levels: Vec<usize> = reading.periods.iter().map(PeriodRow::level).collect();
    let first_three: Vec<usize> = levels.iter().copied().take(3).collect();
    assert_eq!(first_three, [1, 2, 3]);
    for (row, level) in reading.periods.iter().zip(&levels) {
        assert_eq!(row.path.split('/').count(), *level, "{}", row.path);
    }
    // Every period of the birth cycle to the depth, depth first.
    let walked = tree(&dasha, 3);
    assert_eq!(reading.periods.len(), walked.len());
    for (row, period) in reading.periods.iter().zip(&walked) {
        assert_eq!(*row, PeriodRow::of(period));
    }
    assert_eq!(levels.iter().filter(|level| **level == 1).count(), 9);
    assert_eq!(reading.first_lord, Graha::Sun);
}

#[test]
fn a_graha_dasha_s_lords_are_read_back_from_its_mahadashas() {
    let dasha = graha(&UNEQUAL, GrahaDivision::Proportional);
    let reading = time_lord(DashaSystem::Naisargika, &dasha, 1);
    let whole = reading.graha_lords(true).unwrap();
    assert_eq!(whole, UNEQUAL);
    let years = reading.graha_lords(false).unwrap();
    for ((lord, read), (expected, given)) in years.iter().zip(UNEQUAL) {
        assert_eq!(*lord, expected);
        assert!((read - given).abs() < 1e-9, "{lord:?}: {read} of {given}");
    }
    // A year a fraction off a count is a fraction only unrounded.
    let fractional = graha(
        &[(Graha::Sun, 2.4), (Graha::Moon, 1.6)],
        GrahaDivision::Equal,
    );
    let reading = time_lord(DashaSystem::Naisargika, &fractional, 1);
    assert_eq!(
        reading.graha_lords(true).unwrap(),
        [(Graha::Sun, 2.0), (Graha::Moon, 2.0)]
    );
    let unrounded = reading.graha_lords(false).unwrap();
    assert!((unrounded[0].1 - 2.4).abs() < 1e-9);
    let empty = DashaReading {
        periods: Vec::new(),
        ..reading
    };
    assert_eq!(empty.graha_lords(true), None);
}

#[test]
fn a_graha_dasha_s_division_is_read_back_from_its_first_antardashas() {
    for division in [GrahaDivision::Equal, GrahaDivision::Proportional] {
        let dasha = graha(&UNEQUAL, division);
        let reading = time_lord(DashaSystem::Naisargika, &dasha, 2);
        assert_eq!(reading.graha_division(), Some(division), "{division:?}");
        // Rebuilt from what it read, it is the dasha it was read from.
        let rebuilt = graha(&reading.graha_lords(true).unwrap(), division);
        assert_eq!(time_lord(DashaSystem::Naisargika, &rebuilt, 2), reading);
    }
    // One level shows no division.
    let dasha = graha(&UNEQUAL, GrahaDivision::Equal);
    assert_eq!(
        time_lord(DashaSystem::Naisargika, &dasha, 1).graha_division(),
        None
    );
    // Equal years divide alike either way.
    let alike = [(Graha::Sun, 5.0), (Graha::Moon, 5.0), (Graha::Mars, 5.0)];
    for division in [GrahaDivision::Equal, GrahaDivision::Proportional] {
        let reading = time_lord(DashaSystem::Naisargika, &graha(&alike, division), 2);
        assert_eq!(reading.graha_division(), None, "{division:?}");
    }
    // Equal years read at three levels are still the mahadashas' alone.
    let reading = time_lord(
        DashaSystem::Naisargika,
        &graha(&alike, GrahaDivision::Equal),
        3,
    );
    assert_eq!(reading.graha_division(), None);
    // A lone lord has one sub-period, which shows nothing.
    let lone = graha(&[(Graha::Sun, 5.0)], GrahaDivision::Proportional);
    assert_eq!(
        time_lord(DashaSystem::Naisargika, &lone, 2).graha_division(),
        None
    );
    // Two lords whose years differ by less than the tolerance are alike.
    let close = [(Graha::Sun, 5.0), (Graha::Moon, 5.0 + 1e-12)];
    let reading = time_lord(
        DashaSystem::Naisargika,
        &graha(&close, GrahaDivision::Equal),
        2,
    );
    assert_eq!(reading.graha_division(), None);
    // Two lords whose years differ by a day are not.
    let day = 1.0 / 365.25;
    let apart = [(Graha::Sun, 5.0), (Graha::Moon, 5.0 + day)];
    let reading = time_lord(
        DashaSystem::Naisargika,
        &graha(&apart, GrahaDivision::Equal),
        2,
    );
    assert_eq!(reading.graha_division(), Some(GrahaDivision::Equal));
    let reading = time_lord(
        DashaSystem::Naisargika,
        &graha(&apart, GrahaDivision::Proportional),
        2,
    );
    assert_eq!(reading.graha_division(), Some(GrahaDivision::Proportional));
}

#[test]
fn an_ayurdaya_dasha_s_seats_and_shares_rebuild_it() {
    // The lagna and its lord share a graha in different signs, and the
    // shares are uneven and not in seat order.
    let seats = vec![
        (Graha::Mars, Some(Rashi::Aries), 7.5),
        (Graha::Mars, Some(Rashi::Scorpio), 4.25),
        (Graha::Venus, Some(Rashi::Taurus), 11.0),
    ];
    let shares = vec![
        vec![(1, 0.5), (2, 0.3), (0, 0.2)],
        vec![(2, 1.0)],
        vec![(0, 0.25), (1, 0.75)],
    ];
    let dasha = GrahaDasha::with_shares(
        seats.clone(),
        shares.clone(),
        birth(),
        YearLength::Julian36525,
        AfterCycle::End,
    )
    .unwrap();
    let reading = time_lord(DashaSystem::Aayu, &dasha, 3);
    let read = reading.graha_shares().unwrap();
    for (got, want) in read.seats.iter().zip(&seats) {
        assert_eq!((got.0, got.1), (want.0, want.1));
        assert!((got.2 - want.2).abs() < 1e-9, "{got:?} of {want:?}");
    }
    assert_eq!(read.shares.len(), shares.len());
    for (got, want) in read.shares.iter().zip(&shares) {
        let seats_of: Vec<usize> = got.iter().map(|share| share.0).collect();
        let want_seats: Vec<usize> = want.iter().map(|share| share.0).collect();
        assert_eq!(seats_of, want_seats);
        for (part, wanted) in got.iter().zip(want) {
            assert!((part.1 - wanted.1).abs() < 1e-9, "{part:?} of {wanted:?}");
        }
    }
    let rebuilt = GrahaDasha::with_shares(
        read.seats,
        read.shares,
        birth(),
        YearLength::Julian36525,
        AfterCycle::End,
    )
    .unwrap();
    let again = time_lord(DashaSystem::Aayu, &rebuilt, 3);
    for (row, want) in again.periods.iter().zip(&reading.periods) {
        assert_eq!(
            (&row.path, row.lord, row.sign),
            (&want.path, want.lord, want.sign)
        );
        assert!((row.interval.days() - want.interval.days()).abs() < 1e-6);
    }
    // One level holds no shares, and nothing holds no seats.
    assert!(
        time_lord(DashaSystem::Aayu, &dasha, 1)
            .graha_shares()
            .is_none()
    );
    let empty = DashaReading {
        periods: Vec::new(),
        ..reading.clone()
    };
    assert!(empty.graha_shares().is_none());
    // A sub-period of no stored seat holds none.
    let mut stray = reading;
    let child = stray
        .periods
        .iter_mut()
        .find(|row| row.level() == 2)
        .unwrap();
    child.lord = Graha::Saturn;
    assert!(stray.graha_shares().is_none());
}

#[test]
fn the_firdaria_s_nodes_are_read_back_from_where_the_head_runs() {
    let firdaria = |first, nodes| {
        let dasha = FirdariaDasha::new(first, birth(), YearLength::Julian36525, nodes).unwrap();
        time_lord(DashaSystem::Firdaria, &dasha, 1)
    };
    for nodes in [FirdariaNodes::AfterMars, FirdariaNodes::End] {
        assert_eq!(
            firdaria(Graha::Moon, nodes).firdaria_nodes(),
            nodes,
            "{nodes:?}"
        );
    }
    // A day round's Mars is the seventh, so the nodes follow it at the end
    // either way, and the reading says the end.
    for nodes in [FirdariaNodes::AfterMars, FirdariaNodes::End] {
        let reading = firdaria(Graha::Sun, nodes);
        let head = reading
            .periods
            .iter()
            .position(|row| row.lord == Graha::Rahu);
        assert_eq!(head, Some(7), "{nodes:?}");
        assert_eq!(reading.firdaria_nodes(), FirdariaNodes::End, "{nodes:?}");
    }
    // A reading with no Head reads as the end.
    let mut headless = firdaria(Graha::Moon, FirdariaNodes::AfterMars);
    headless.periods.retain(|row| row.lord != Graha::Rahu);
    assert_eq!(headless.firdaria_nodes(), FirdariaNodes::End);
    // The Head is counted among the mahadashas alone: a night round's,
    // after Mars, is the fifth, where it is the thirty-third row of two
    // levels.
    let deep = {
        let dasha = FirdariaDasha::new(
            Graha::Moon,
            birth(),
            YearLength::Julian36525,
            FirdariaNodes::AfterMars,
        )
        .unwrap();
        time_lord(DashaSystem::Firdaria, &dasha, 2)
    };
    let row = deep
        .periods
        .iter()
        .position(|row| row.lord == Graha::Rahu)
        .unwrap();
    assert!(row >= 7, "the Head's row {row}");
    assert_eq!(deep.firdaria_nodes(), FirdariaNodes::AfterMars);
}

#[test]
fn the_decennials_order_and_division_are_read_back_from_their_rows() {
    let order = decennial_order(Graha::Moon, &PLACES).unwrap();
    for division in [DecennialDivision::Cycles, DecennialDivision::Proportional] {
        let dasha = DecennialDasha::new(order, birth(), YearLength::Julian36525, division).unwrap();
        let reading = time_lord(DashaSystem::Decennials, &dasha, 3);
        assert_eq!(reading.decennial_order(), Some(order));
        assert_eq!(reading.decennial_division(), Some(division), "{division:?}");
        // Two levels show no division.
        let shallow = time_lord(DashaSystem::Decennials, &dasha, 2);
        assert_eq!(shallow.decennial_division(), None);
        assert_eq!(shallow.decennial_order(), Some(order));
    }
    // Fewer than seven stored mahadashas give no order.
    let dasha = DecennialDasha::new(
        order,
        birth(),
        YearLength::Julian36525,
        DecennialDivision::Cycles,
    )
    .unwrap();
    let mut six = time_lord(DashaSystem::Decennials, &dasha, 1);
    six.periods.truncate(6);
    assert_eq!(six.decennial_order(), None);
    six.periods.clear();
    assert_eq!(six.decennial_order(), None);
}

#[test]
fn a_time_lord_s_start_is_its_first_period_s_sign() {
    let releasing = ReleasingDasha::new(Rashi::Leo, birth(), YearLength::Savana360).unwrap();
    let reading = time_lord(DashaSystem::ReleasingFortune, &releasing, 2);
    assert_eq!(reading.start_sign(), Some(Rashi::Leo));
    let profection = ProfectionDasha::new(Rashi::Pisces, birth(), YearLength::Julian36525).unwrap();
    let reading = time_lord(DashaSystem::Profection, &profection, 1);
    assert_eq!(reading.start_sign(), Some(Rashi::Pisces));
    // A graha dasha's periods carry no sign.
    let dasha = graha(&UNEQUAL, GrahaDivision::Equal);
    assert_eq!(
        time_lord(DashaSystem::Naisargika, &dasha, 1).start_sign(),
        None
    );
}

fn vimshottari() -> Dasha {
    let row = row(DashaSystem::Vimshottari, AshtottariGrouping::ThreeEach).unwrap();
    Dasha::new(row, &moon_birth(), rules(DashaSystem::Vimshottari)).unwrap()
}

fn moon_birth() -> Birth {
    Birth {
        instant: birth(),
        moon: Nas::from_degrees(Degrees::try_new(123.456).unwrap()),
        moon_span: None,
    }
}

fn rashi_chart() -> RashiChart {
    RashiChart {
        lagna: Rashi::Aquarius,
        arudha_lagna: Rashi::Aries,
        navamsa_lagna: Rashi::Aries,
        signs: [Rashi::Capricorn; 9],
        dignities: [teistro_core::catalogue::Dignity::Neutral; 9],
        brahma: None,
    }
}

fn sudarshana() -> RashiDasha {
    RashiDasha::new(
        rashi_row(DashaSystem::SudarshanaChakra).unwrap(),
        &rashi_chart(),
        birth(),
        YearLength::Julian36525,
        AfterCycle::End,
        RashiRules::RECORDING_ENGINE,
    )
    .unwrap()
}

#[test]
fn a_registered_system_s_reading_carries_its_definition() {
    let mut definition = UduDefinition::of("sdk.test.three", Nakshatra::Ashwini);
    definition.lords = vec![
        Lord {
            graha: Graha::Sun,
            years: 6,
        },
        Lord {
            graha: Graha::Moon,
            years: 10,
        },
        Lord {
            graha: Graha::Mars,
            years: 7,
        },
    ];
    definition.depth = depth(2);
    let dasha = Dasha::new(
        &definition.row(),
        &moon_birth(),
        rules(DashaSystem::Vimshottari),
    )
    .unwrap();
    let reading = DashaReading::of_registered(&dasha, &definition, None);
    assert_eq!(
        reading.definition,
        Some(DashaDefinition::Udu(definition.clone()))
    );
    assert_eq!(reading.depth, depth(2));
    assert_eq!(
        DashaReading {
            definition: None,
            ..reading
        },
        DashaReading::of(&dasha, definition.depth, None)
    );

    let definition = RashiDefinition::of("sdk.test.signs");
    let dasha = RashiDasha::new(
        &definition.row(),
        &rashi_chart(),
        birth(),
        definition.year_length,
        AfterCycle::End,
        RashiRules::RECORDING_ENGINE,
    )
    .unwrap();
    let rules = rules(DashaSystem::Chara);
    let reading = DashaReading::of_registered_rashi(&dasha, &definition, rules);
    assert_eq!(
        reading.definition,
        Some(DashaDefinition::Rashi(definition.clone()))
    );
    assert_eq!(
        DashaReading {
            definition: None,
            ..reading
        },
        DashaReading::of_rashi(&dasha, rules, definition.depth)
    );
}

/// What a timeline answers: its breadth, its second mahadasha, the first
/// of its second cycle, the mahadasha a day into the second, and that
/// one's second child.
fn answers(timeline: &impl Timeline) -> Vec<String> {
    let second = timeline.mahadasha(0, 1).unwrap();
    let instant = second.interval.from.get() + 1.0;
    vec![
        format!("{}", timeline.breadth()),
        format!("{second:?}"),
        format!("{:?}", timeline.mahadasha(1, 0)),
        format!("{:?}", timeline.mahadasha_at(instant)),
        format!("{:?}", timeline.child(&second, 1)),
    ]
}

/// The cursor over each kernel: its breadth, its mahadashas by place and
/// by instant and their children are the kernel's own, and only its own
/// accessor answers.
#[test]
fn the_cursor_answers_as_the_kernel_it_holds() {
    let kalachakra = KalachakraDasha::new(&moon_birth(), KalachakraRules::of(&root())).unwrap();
    let order = decennial_order(Graha::Sun, &PLACES).unwrap();
    let cursors = [
        DashaCursor::Nakshatra(vimshottari()),
        DashaCursor::Rashi(sudarshana()),
        DashaCursor::Kalachakra(kalachakra),
        DashaCursor::Releasing(
            ReleasingDasha::new(Rashi::Leo, birth(), YearLength::Savana360).unwrap(),
        ),
        DashaCursor::Profection(
            ProfectionDasha::new(Rashi::Pisces, birth(), YearLength::Julian36525).unwrap(),
        ),
        DashaCursor::Firdaria(
            FirdariaDasha::new(
                Graha::Sun,
                birth(),
                YearLength::Julian36525,
                FirdariaNodes::End,
            )
            .unwrap(),
        ),
        DashaCursor::Decennials(
            DecennialDasha::new(
                order,
                birth(),
                YearLength::Julian36525,
                DecennialDivision::Proportional,
            )
            .unwrap(),
        ),
        DashaCursor::Graha(graha(&UNEQUAL, GrahaDivision::Proportional)),
    ];
    for (at, cursor) in cursors.iter().enumerate() {
        let own = match cursor {
            DashaCursor::Nakshatra(dasha) => answers(dasha),
            DashaCursor::Rashi(dasha) => answers(dasha),
            DashaCursor::Kalachakra(dasha) => answers(dasha),
            DashaCursor::Releasing(dasha) => answers(dasha),
            DashaCursor::Profection(dasha) => answers(dasha),
            DashaCursor::Firdaria(dasha) => answers(dasha),
            DashaCursor::Decennials(dasha) => answers(dasha),
            DashaCursor::Graha(dasha) => answers(dasha),
        };
        assert_eq!(answers(cursor), own, "cursor {at}");
        let accessors = [
            cursor.nakshatra().is_some(),
            cursor.rashi().is_some(),
            cursor.kalachakra().is_some(),
            cursor.releasing().is_some(),
            cursor.profection().is_some(),
            cursor.firdaria().is_some(),
            cursor.decennials().is_some(),
            cursor.graha().is_some(),
        ];
        let expected: Vec<bool> = (0..accessors.len()).map(|slot| slot == at).collect();
        assert_eq!(accessors.to_vec(), expected, "cursor {at}");
    }
    // Each cursor below the profected year, whose years run no sub-periods,
    // finds a mahadasha a day into the second and that one's second child.
    for (at, cursor) in cursors.iter().enumerate().filter(|(at, _)| *at != 4) {
        let own = answers(cursor);
        assert_ne!(own[3], "None", "cursor {at}");
        assert_ne!(own[4], "None", "cursor {at}");
    }
    assert_eq!(cursors[0].nakshatra(), Some(&vimshottari()));
    assert_eq!(cursors[1].rashi(), Some(&sudarshana()));
}
