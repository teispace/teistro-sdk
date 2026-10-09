//! `sdk.research()` over founded births (`03-design/research.md` §3,
//! tests 2, 6, 8 and 9 at the façade).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    clippy::float_cmp,
    reason = "tests fail by panicking, index their own tables, and compare bits they expect to be equal"
)]

use std::num::NonZeroU16;

use teistro::catalogue::DashaSystem;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::research::{
    AfterBirth, Birth, Contrast, Design, EventShuffle, EventStudy, EventTest, GroupTest, Holds,
    Parallelism, Recombine, ReplicateTest, SplitMix64, Study, Subject,
};
use teistro::{ChartRequest, Context, Ephemeris, RuleRequest, ShippedRules, UtcOffset};

fn sdk() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("the built-in ephemeris")
}

fn place(latitude: f64, longitude: f64) -> Place {
    Place::new(
        Latitude::try_new(latitude).unwrap(),
        Longitude::try_new(longitude).unwrap(),
        Altitude::try_new(0.0).unwrap(),
    )
}

/// Births spread over 1950 to 2010, every hour of the day and three
/// places, drawn from `seed`: a sky that can happen.
fn births(count: usize, seed: u64) -> Vec<Birth> {
    let places = [
        (place(27.7172, 85.324), 20_700),
        (place(28.6139, 77.209), 19_800),
        (place(51.5074, -0.1278), 0),
    ];
    let mut rng = SplitMix64::new(seed);
    (0..count)
        .map(|_| {
            let day = 2_433_282.5 + rng.below(21_915) as f64;
            let hour = rng.below(1_440) as f64 / 1_440.0;
            let (at, seconds) = places[usize::try_from(rng.below(3)).unwrap()];
            Birth::new(
                JulianDay::<Utc>::literal(day + hour),
                at,
                UtcOffset::try_from_seconds(seconds).unwrap(),
            )
        })
        .collect()
}

fn yogas() -> teistro::RuleSet {
    RuleRequest::shipped([ShippedRules::Yogas])
        .rule_set()
        .unwrap()
}

fn template() -> ChartRequest {
    ChartRequest::at(place(0.0, 0.0), UtcOffset::try_from_seconds(0).unwrap())
}

#[test]
fn a_study_counts_what_each_chart_answers_alone() {
    let sdk = sdk();
    let births = births(24, 1);
    let set = yogas();
    let request = template();
    let design = Design::new((0..24).map(|i| u16::from(i % 2 == 0)).collect());
    let counted = sdk
        .research()
        .counts(Study::new(&births, &request, &set), &design)
        .unwrap()
        .value;
    assert_eq!(counted.rows.len(), set.rules().len());
    // The other road: every birth read alone at its own place.
    let cells = standing_alone(&sdk, &births, &set);
    let mut present = vec![[0_u32; 2]; set.rules().len()];
    for (chart, &group) in cells.iter().zip(&design.groups) {
        for (j, &holds) in chart.iter().enumerate() {
            present[j][usize::from(group)] += u32::from(holds);
        }
    }
    for (row, (rule, expected)) in counted.rows.iter().zip(set.rules().iter().zip(&present)) {
        assert_eq!(row.predicate, rule.key);
        assert_eq!(
            [row.counts[0].present, row.counts[1].present],
            *expected,
            "{}",
            rule.key
        );
        assert_eq!(row.counts[0].present + row.counts[0].absent, 12);
    }
}

#[test]
fn a_rule_that_flips_inside_the_uncertainty_is_unstable() {
    let sdk = sdk();
    let set = yogas();
    let request = template();
    let trusted = births(16, 2);
    let design = Design::new((0..16).map(|i| u16::from(i < 8)).collect());
    let unstable = |births: &[Birth]| -> u32 {
        sdk.research()
            .counts(Study::new(births, &request, &set), &design)
            .unwrap()
            .value
            .rows
            .iter()
            .flat_map(|row| row.counts.iter().map(|c| c.unstable))
            .sum()
    };
    assert_eq!(unstable(&trusted), 0);
    // Two hours either side moves every lagna at least a sign, and some
    // yoga counted from it with it.
    let rounded: Vec<Birth> = trusted.iter().map(|b| b.uncertain_by(120.0)).collect();
    assert!(unstable(&rounded) > 0);
}

#[test]
fn a_rule_on_a_point_is_unreadable_on_a_day_with_no_sunrise() {
    // The baseline's profile founds a polar day by its nearest events; the
    // special lagnas still have no sunrise to count from.
    let sdk = Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let set = RuleRequest::shipped([ShippedRules::Nabhasas, ShippedRules::Readings])
        .rule_set()
        .unwrap();
    let Some(on_points) = set.rules().iter().position(teistro::rules::Rule::reads_points) else {
        panic!("a shipped set the test can use names a point");
    };
    let tromso = place(69.6492, 18.9553);
    let utc = UtcOffset::try_from_seconds(3_600).unwrap();
    let mut births = births(4, 3);
    // Tromsø's polar night, 21 December 1988.
    births.push(Birth::new(
        JulianDay::<Utc>::literal(2_447_516.5),
        tromso,
        utc,
    ));
    let request = template();
    let design = Design::new(vec![0, 1, 0, 1, 1]);
    let counted = sdk
        .research()
        .counts(Study::new(&births, &request, &set), &design)
        .unwrap()
        .value;
    let row = &counted.rows[on_points];
    assert_eq!(row.counts[1].unreadable, 1, "{}", row.predicate);
    assert_eq!(row.counts[0].unreadable, 0);
    // The other charts in the batch keep their reading of it.
    assert_eq!(
        row.counts.iter().map(|c| c.present + c.absent).sum::<u32>(),
        4
    );
}

#[test]
fn a_comparison_is_the_same_at_every_thread_count_and_seals_its_study() {
    let sdk = sdk();
    let births = births(30, 4);
    let set = yogas();
    let request = template();
    let design = Design::new((0..30).map(|i| u16::from(i % 3 == 0)).collect());
    let mut test = GroupTest::new(9, 999, Contrast::CaseVsRest { case: 1 });
    let study = Study::new(&births, &request, &set);
    let one = sdk.research().compare(study, &design, &test).unwrap();
    test.parallelism = Parallelism::Threads(NonZeroU16::new(4).unwrap());
    let four = sdk.research().compare(study, &design, &test).unwrap();
    assert_eq!(one.value, four.value);
    // The thread count is how, not what: the study is the same study.
    assert_eq!(one.provenance.input_hash, four.provenance.input_hash);
    // The input hash is the pre-registration: a different seed, rule
    // reading or label is a different study.
    test.seed = 10;
    let reseeded = sdk.research().compare(study, &design, &test).unwrap();
    assert_ne!(one.provenance.input_hash, reseeded.provenance.input_hash);
    let formed = sdk
        .research()
        .compare(study.holding(Holds::Formed), &design, &test)
        .unwrap();
    assert_ne!(reseeded.provenance.input_hash, formed.provenance.input_hash);
    for row in &one.value.rows {
        assert!(row.p.value >= one.value.resolution);
        assert!(row.adjusted.holm >= row.p.value);
    }
}

/// Whether each rule of `set` stands on each birth, every chart read alone
/// at its own place: the road a study's batches must agree with.
fn standing_alone(sdk: &Context, births: &[Birth], set: &teistro::RuleSet) -> Vec<Vec<bool>> {
    births
        .iter()
        .map(|birth| {
            let alone = ChartRequest::at(birth.place, birth.offset);
            let read = sdk
                .chart()
                .readings_with_rules(&[birth.instant], &alone, set)
                .unwrap()
                .value;
            set.rules()
                .iter()
                .map(|rule| {
                    read[0].1.present.iter().any(|p| {
                        p.rule.key == rule.key
                            && p.result.status != Some(teistro::rules::NetStatus::FullyCancelled)
                    })
                })
                .collect()
        })
        .collect()
}

/// Pearson's phi between two rules over the same charts.
fn phi(cells: &[Vec<bool>], a: usize, b: usize) -> f64 {
    let (mut n11, mut n10, mut n01, mut n00) = (0.0, 0.0, 0.0, 0.0);
    for chart in cells {
        match (chart[a], chart[b]) {
            (true, true) => n11 += 1.0,
            (true, false) => n10 += 1.0,
            (false, true) => n01 += 1.0,
            (false, false) => n00 += 1.0,
        }
    }
    let spread: f64 = (n11 + n10) * (n01 + n00) * (n11 + n01) * (n10 + n00);
    if spread == 0.0 {
        0.0
    } else {
        (n11 * n00 - n10 * n01) / spread.sqrt()
    }
}

#[test]
fn a_planted_effect_is_found_and_only_what_shares_it_follows() {
    // Test 2 at the façade: label a chart a case far more often when one
    // rule holds on it, and the rule's adjusted p falls under 0.05; another
    // rule may follow it only by sharing its charts.
    let sdk = sdk();
    let births = births(240, 5);
    let set = yogas();
    let request = template();
    let cells = standing_alone(&sdk, &births, &set);
    // The commonest rule that is neither everywhere nor nowhere.
    let target = (0..set.rules().len())
        .map(|j| (j, cells.iter().filter(|chart| chart[j]).count()))
        .filter(|&(_, present)| (60..=180).contains(&present))
        .max_by_key(|&(_, present)| present)
        .map(|(j, _)| j)
        .expect("a rule of middling prevalence");
    let mut rng = SplitMix64::new(77);
    let groups: Vec<u16> = cells
        .iter()
        .map(|chart| {
            let share = if chart[target] { 0.8 } else { 0.2 };
            u16::from((rng.next_u64() as f64 / u64::MAX as f64) < share)
        })
        .collect();
    let design = Design::new(groups);
    // The batches count what each chart answers alone.
    let counted = sdk
        .research()
        .counts(Study::new(&births, &request, &set), &design)
        .unwrap()
        .value;
    for (j, row) in counted.rows.iter().enumerate() {
        let alone = cells
            .iter()
            .zip(&design.groups)
            .filter(|(chart, group)| chart[j] && **group == 1)
            .count();
        assert_eq!(row.counts[1].present as usize, alone, "{}", row.predicate);
    }
    let mut test = GroupTest::new(31, 4_999, Contrast::CaseVsRest { case: 1 });
    test.alpha = Some(0.05);
    let tested = sdk
        .research()
        .compare(Study::new(&births, &request, &set), &design, &test)
        .unwrap()
        .value;
    let row = &tested.rows[target];
    assert!(row.under_alpha.unwrap().max_t, "{row:?}");
    assert!(row.effect.unwrap().risk_ratio.unwrap().low > 1.0);
    for (j, other) in tested.rows.iter().enumerate() {
        if j != target && other.under_alpha.is_some_and(|u| u.holm) {
            let shared = phi(&cells, target, j);
            assert!(
                shared.abs() > 0.2,
                "{} follows {} with phi {shared}",
                other.predicate,
                row.predicate
            );
        }
    }
}

#[test]
fn every_study_refusal_names_its_field() {
    let sdk = sdk();
    let set = yogas();
    let request = template();
    let mut births = births(4, 6);
    let design = Design::new(vec![0, 1, 0, 1]);
    let field = |births: &[Birth], design: &Design| {
        sdk.research()
            .counts(Study::new(births, &request, &set), design)
            .unwrap_err()
            .field()
            .map(str::to_owned)
    };
    assert_eq!(field(&[], &Design::new(vec![])).as_deref(), Some("births"));
    births[2] = births[2].uncertain_by(721.0);
    assert_eq!(
        field(&births, &design).as_deref(),
        Some("births[2].uncertaintyMinutes")
    );
    births[2] = births[2].uncertain_by(-1.0);
    assert_eq!(
        field(&births, &design).as_deref(),
        Some("births[2].uncertaintyMinutes")
    );
    births[2] = births[2].uncertain_by(0.0);
    assert_eq!(
        field(&births, &Design::new(vec![0, 1])).as_deref(),
        Some("groups")
    );
    // A refusal after a refusal names its own.
    let test = GroupTest::new(1, 0, Contrast::CaseVsRest { case: 1 });
    let error = sdk
        .research()
        .compare(Study::new(&births, &request, &set), &design, &test)
        .unwrap_err();
    assert_eq!(error.field(), Some("permutations"));
}

/// Subjects born over the `span` years from 1950 whose events fall `from`
/// to `to` years later, drawn from `seed`.
fn subjects(count: usize, seed: u64, span: u64, from: u64, to: u64) -> Vec<Subject> {
    let mut rng = SplitMix64::new(seed);
    births(count, seed)
        .into_iter()
        .map(|birth| {
            let born = 2_433_282.5 + rng.below(span * 365) as f64 + 0.25;
            let birth = Birth {
                instant: JulianDay::<Utc>::literal(born),
                ..birth
            };
            let age = (from * 365 + rng.below((to - from) * 365)) as f64;
            Subject {
                birth,
                event: JulianDay::<Utc>::literal(born + age),
            }
        })
        .collect()
}

#[test]
fn an_event_study_shuffles_ages_or_dates_and_refuses_a_pairing_before_a_birth() {
    let sdk = sdk();
    let set = yogas();
    let request = template();
    let subjects = subjects(12, 7, 30, 5, 15);
    let study = EventStudy::new(&subjects, &request, &set, DashaSystem::Vimshottari)
        .shuffled(EventShuffle::AgesAtEvent);
    let mut test = EventTest::new(3, 199);
    let one = sdk.research().timed(study, None, &test).unwrap();
    assert_eq!(one.value.rows.len(), set.rules().len());
    for row in &one.value.rows {
        let expected = row.expected.unwrap();
        assert!(
            (0.0..=1.0).contains(&expected.expected),
            "{}",
            row.predicate
        );
        assert_eq!(row.counts.len(), 1);
    }
    test.parallelism = Parallelism::Threads(NonZeroU16::new(3).unwrap());
    let three = sdk.research().timed(study, None, &test).unwrap();
    assert_eq!(one.value, three.value);
    assert_eq!(one.provenance.input_hash, three.provenance.input_hash);
    // Births up to thirty years apart and events at 5 to 15: some date
    // shuffles put an event before a birth, which is refused unless
    // restricted.
    let crossing = subjects
        .iter()
        .flat_map(|i| {
            subjects
                .iter()
                .map(move |j| j.event.get() < i.birth.instant.get())
        })
        .filter(|&crossed| crossed)
        .count();
    assert!(crossing > 0, "the premise: some pairing crosses a birth");
    let dated = study.shuffled(EventShuffle::EventDates);
    let refused = sdk.research().timed(dated, None, &EventTest::new(3, 199));
    assert_eq!(refused.unwrap_err().field(), Some("afterBirth"));
    let restricted = EventTest {
        after_birth: AfterBirth::RestrictPairings,
        ..EventTest::new(3, 199)
    };
    let answered = sdk.research().timed(dated, None, &restricted).unwrap();
    assert_ne!(answered.provenance.input_hash, one.provenance.input_hash);
    // An event before its own birth is refused by its subject.
    let mut early = subjects.clone();
    early[4].event = JulianDay::<Utc>::literal(early[4].birth.instant.get() - 1.0);
    let study = EventStudy::new(&early, &request, &set, DashaSystem::Vimshottari);
    let error = sdk.research().timed(study, None, &test).unwrap_err();
    assert_eq!(error.field(), Some("subjects[4].event"));
}

#[test]
fn a_sample_is_read_against_its_own_recombined_population() {
    let sdk = sdk();
    let set = yogas();
    let request = template();
    let births = births(12, 8);
    let study = Study::new(&births, &request, &set);
    let control = Recombine {
        seed: 4,
        replicates: 19,
        strata: None,
    };
    let test = ReplicateTest::default();
    let first = sdk.research().expected(study, &control, &test).unwrap();
    let again = sdk.research().expected(study, &control, &test).unwrap();
    assert_eq!(first.value, again.value);
    assert_eq!(first.value.permutations, 19);
    assert_eq!(first.value.rows.len(), set.rules().len());
    for row in &first.value.rows {
        assert!(row.p.value >= first.value.resolution, "{}", row.predicate);
    }
    let reseeded = Recombine {
        seed: 5,
        ..control.clone()
    };
    let other = sdk.research().expected(study, &reseeded, &test).unwrap();
    assert_ne!(first.provenance.input_hash, other.provenance.input_hash);
    // Refusals name the control's own fields.
    let none = Recombine {
        replicates: 0,
        ..control.clone()
    };
    let error = sdk.research().expected(study, &none, &test).unwrap_err();
    assert_eq!(error.field(), Some("control.replicates"));
    let uneven = Recombine {
        strata: Some(vec![0; 11]),
        ..control
    };
    let error = sdk.research().expected(study, &uneven, &test).unwrap_err();
    assert_eq!(error.field(), Some("control.strata"));
}

#[test]
fn a_birth_the_ephemeris_cannot_reach_is_named() {
    let sdk = sdk();
    let set = yogas();
    let request = template();
    let mut births = births(5, 9);
    births[3].instant = JulianDay::<Utc>::literal(2_000_000.5);
    let design = Design::new(vec![0, 1, 0, 1, 0]);
    let error = sdk
        .research()
        .counts(Study::new(&births, &request, &set), &design)
        .unwrap_err();
    assert!(
        error
            .hint()
            .is_some_and(|hint| hint.starts_with("births[3]")),
        "{error:?}"
    );
}
