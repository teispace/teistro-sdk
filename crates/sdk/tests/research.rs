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

use teistro::catalogue::{DashaSystem, Nakshatra};
use teistro::dasha::VIMSHOTTARI;
use teistro::quantity::{Altitude, Depth, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::research::{
    AfterBirth, Birth, Contrast, Design, EventShuffle, EventStudy, EventTest, GroupTest, Holds,
    Parallelism, Recombine, Referents, ReplicateTest, SplitMix64, Study, Subject,
};
use teistro::{
    ChartRequest, Context, Ephemeris, ResearchAnswer, ResearchRequest, RuleRequest, ShippedRules,
    UduDefinition, UtcOffset,
};

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

/// A consumer's rule: the lagna rises in a dual sign.
fn lagna_dual() -> teistro::RuleSet {
    let rule: teistro::rules::Rule = serde_json::from_value(serde_json::json!({
        "key": "LAGNA_DUAL", "category": "test", "source": {"text": "test"},
        "conditions": [{"type": "lagna-in-sign",
                        "signs": ["GEMINI", "VIRGO", "SAGITTARIUS", "PISCES"]}]
    }))
    .unwrap();
    RuleRequest::shipped([])
        .with_rules([rule])
        .rule_set()
        .unwrap()
}

/// How many births a single-rule study reads as unstable.
fn unstable_births(sdk: &Context, births: &[Birth], set: &teistro::RuleSet, step: f64) -> u32 {
    let request = template();
    let design = Design::new((0..births.len()).map(|i| u16::from(i % 2 == 0)).collect());
    sdk.research()
        .counts(Study::new(births, &request, set).stepping(step), &design)
        .unwrap()
        .value
        .rows[0]
        .counts
        .iter()
        .map(|count| count.unstable)
        .sum()
}

#[test]
fn an_uncertain_birth_is_read_across_its_window_not_at_its_edges() {
    // The lagna turns the zodiac in a day, a sign in about two hours, so a
    // birth uncertain by twelve hours either side passes every sign. Its
    // two edges are a day apart, so rise within a sign or so of each
    // other and about half a turn from the centre, where a dual sign's
    // opposite is dual too. Read at its edges only, a dual-lagna birth
    // often looks stable; read across the window, none is.
    let sdk = sdk();
    let set = lagna_dual();
    let births: Vec<Birth> = births(24, 3)
        .into_iter()
        .map(|b| b.uncertain_by(720.0))
        .collect();
    assert_eq!(
        unstable_births(&sdk, &births, &set, 2.0),
        24,
        "every birth passes through a movable sign"
    );
    // A step as wide as the window reads the edges alone, which is how
    // every window was read before, and calls some of them stable.
    let edges = unstable_births(&sdk, &births, &set, 720.0);
    assert!(edges < 24, "the edges alone found {edges} of 24 unstable");
}

#[test]
fn a_slow_predicate_stays_stable_across_a_narrow_window() {
    // The Sun keeps its sign for a month: ten minutes either side moves it
    // a fortieth of a degree, so only a birth that close to an ingress
    // could read unstable, and none of these is.
    let sdk = sdk();
    let rule: teistro::rules::Rule = serde_json::from_value(serde_json::json!({
        "key": "SUN_FIRE", "category": "test", "source": {"text": "test"},
        "conditions": [{"type": "planet-in-sign", "planet": "SUN",
                        "signs": ["ARIES", "LEO", "SAGITTARIUS"]}]
    }))
    .unwrap();
    let set = RuleRequest::shipped([])
        .with_rules([rule])
        .rule_set()
        .unwrap();
    let births: Vec<Birth> = births(24, 4)
        .into_iter()
        .map(|b| b.uncertain_by(10.0))
        .collect();
    assert_eq!(unstable_births(&sdk, &births, &set, 1.0), 0);
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
    let Some(on_points) = set
        .rules()
        .iter()
        .position(teistro::rules::Rule::reads_points)
    else {
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
    let study = EventStudy::new(
        &subjects,
        &request,
        &set,
        DashaSystem::Vimshottari,
        EventShuffle::AgesAtEvent,
    );
    let mut test = EventTest::new(3, 199);
    let one = sdk.research().timed(&study, None, &test).unwrap();
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
    let three = sdk.research().timed(&study, None, &test).unwrap();
    assert_eq!(one.value, three.value);
    assert_eq!(one.provenance.input_hash, three.provenance.input_hash);
    // Read to the mahadasha alone, the study is another registration.
    let shallow = study.clone().to_depth(Depth::MIN);
    let mahadashas = sdk.research().timed(&shallow, None, &test).unwrap();
    assert_ne!(mahadashas.provenance.input_hash, one.provenance.input_hash);
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
    let dated = study.clone().shuffled(EventShuffle::EventDates);
    let refused = sdk.research().timed(&dated, None, &EventTest::new(3, 199));
    assert_eq!(refused.unwrap_err().field(), Some("afterBirth"));
    let restricted = EventTest {
        after_birth: AfterBirth::RestrictPairings,
        ..EventTest::new(3, 199)
    };
    let answered = sdk.research().timed(&dated, None, &restricted).unwrap();
    assert_ne!(answered.provenance.input_hash, one.provenance.input_hash);
    // An event before its own birth is refused by its subject.
    let mut early = subjects.clone();
    early[4].event = JulianDay::<Utc>::literal(early[4].birth.instant.get() - 1.0);
    let study = EventStudy::new(
        &early,
        &request,
        &set,
        DashaSystem::Vimshottari,
        EventShuffle::AgesAtEvent,
    );
    let error = sdk.research().timed(&study, None, &test).unwrap_err();
    assert_eq!(error.field(), Some("subjects[4].event"));
}

#[test]
fn a_self_matched_study_reads_each_person_against_their_own_life() {
    let sdk = sdk();
    let set = yogas();
    let request = template();
    // Births thirty years apart: a date shuffle would cross them, and a
    // self-matched study moves nobody's event to anybody else.
    let subjects = subjects(12, 7, 30, 5, 15);
    let year = Referents::new(365.25, 30.0);
    let study = EventStudy::new(
        &subjects,
        &request,
        &set,
        DashaSystem::Vimshottari,
        EventShuffle::EventDates,
    )
    .self_matched(year);
    assert_eq!(study.shuffle, EventShuffle::SelfMatched);
    let mut test = EventTest::new(3, 199);
    let one = sdk.research().timed(&study, None, &test).unwrap();
    assert_eq!(one.value.rows.len(), set.rules().len());
    for row in &one.value.rows {
        assert_eq!(row.counts.len(), 1);
        let read = row.counts[0].present + row.counts[0].absent;
        assert_eq!(read + row.counts[0].unreadable, 12, "{}", row.predicate);
        let expected = row.expected.unwrap();
        assert!(
            (0.0..=1.0).contains(&expected.expected),
            "{}",
            row.predicate
        );
    }
    test.parallelism = Parallelism::Threads(NonZeroU16::new(3).unwrap());
    let three = sdk.research().timed(&study, None, &test).unwrap();
    assert_eq!(one.value, three.value);
    assert_eq!(one.provenance.input_hash, three.provenance.input_hash);
    // Other referents are another registration.
    let weekly = study.clone().self_matched(Referents::new(365.25, 7.0));
    let other = sdk.research().timed(&weekly, None, &test).unwrap();
    assert_ne!(other.provenance.input_hash, one.provenance.input_hash);
    // Without referents, with strata, or referents under another shuffle:
    // each refused by its field.
    let bare = EventStudy {
        referents: None,
        ..study.clone()
    };
    let refused = |study: &EventStudy<'_>, strata: Option<&[u32]>| {
        sdk.research()
            .timed(study, strata, &test)
            .unwrap_err()
            .field()
            .map(str::to_owned)
    };
    assert_eq!(refused(&bare, None).as_deref(), Some("referents"));
    assert_eq!(refused(&study, Some(&[0; 12])).as_deref(), Some("strata"));
    let aged = study.clone().shuffled(EventShuffle::AgesAtEvent);
    assert_eq!(refused(&aged, None).as_deref(), Some("referents"));
    let tight = study.clone().self_matched(Referents::new(365.25, 0.25));
    assert_eq!(refused(&tight, None).as_deref(), Some("referents.stepDays"));
    let restricted = EventTest {
        after_birth: AfterBirth::RestrictPairings,
        ..EventTest::new(3, 199)
    };
    let error = sdk.research().timed(&study, None, &restricted).unwrap_err();
    assert_eq!(error.field(), Some("afterBirth"));
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

/// A birth as a binding writes it.
fn birth_json(birth: &Birth) -> serde_json::Value {
    serde_json::json!({
        "instant": birth.instant.get(),
        "latitudeDeg": birth.place.latitude.get(),
        "longitudeDeg": birth.place.longitude.get(),
        "utcOffsetSeconds": birth.offset.seconds(),
        "uncertaintyMinutes": birth.uncertainty_minutes,
    })
}

fn sent(sdk: &Context, request: &serde_json::Value) -> Result<ResearchAnswer, teistro::Error> {
    sdk.research()
        .request(&ResearchRequest::from_json(&request.to_string())?)
}

#[test]
fn a_study_sent_as_a_record_answers_as_the_call_does() {
    let sdk = sdk();
    let set = yogas();
    let request = template();
    let births = births(12, 9);
    let design = Design::new((0..12).map(|i| u16::from(i < 4)).collect());
    let study = Study::new(&births, &request, &set).holding(Holds::Formed);
    let listed: Vec<serde_json::Value> = births.iter().map(birth_json).collect();
    let rules = serde_json::json!({"shipped": ["YOGAS"]});

    let counted = sent(
        &sdk,
        &serde_json::json!({"study": "COUNTS", "rules": rules, "holds": "FORMED",
            "births": listed, "design": {"groups": design.groups}}),
    )
    .unwrap();
    let direct = sdk.research().counts(study, &design).unwrap();
    assert_eq!(counted, ResearchAnswer::Counts(direct));

    let uncertain: Vec<Birth> = births.iter().map(|b| b.uncertain_by(30.0)).collect();
    let stepped = sent(
        &sdk,
        &serde_json::json!({"study": "COUNTS", "rules": rules, "holds": "FORMED",
            "births": uncertain.iter().map(birth_json).collect::<Vec<_>>(),
            "design": {"groups": design.groups}, "uncertaintyStepMinutes": 10}),
    )
    .unwrap();
    let direct = sdk
        .research()
        .counts(
            Study::new(&uncertain, &request, &set)
                .holding(Holds::Formed)
                .stepping(10.0),
            &design,
        )
        .unwrap();
    assert_eq!(stepped, ResearchAnswer::Counts(direct));

    let test = GroupTest::new(11, 199, Contrast::CaseVsRest { case: 1 });
    let compared = sent(
        &sdk,
        &serde_json::json!({"study": "COMPARE", "rules": rules, "holds": "FORMED",
            "births": listed, "design": {"groups": design.groups},
            "test": {"seed": "11", "permutations": 199,
                     "contrast": {"kind": "CASE_VS_REST", "case": 1}}}),
    )
    .unwrap();
    let direct = sdk.research().compare(study, &design, &test).unwrap();
    assert_eq!(compared, ResearchAnswer::Tested(direct));

    let control = Recombine {
        seed: 4,
        replicates: 9,
        strata: None,
    };
    let expected = sent(
        &sdk,
        &serde_json::json!({"study": "EXPECTED", "rules": rules, "holds": "FORMED",
            "births": listed, "control": {"seed": 4, "replicates": 9}}),
    )
    .unwrap();
    let direct = sdk
        .research()
        .expected(study, &control, &ReplicateTest::default())
        .unwrap();
    assert_eq!(expected, ResearchAnswer::Tested(direct));
}

#[test]
fn an_event_study_sent_as_a_record_answers_as_the_call_does() {
    let sdk = sdk();
    let set = yogas();
    let request = template();
    let rules = serde_json::json!({"shipped": ["YOGAS"]});
    let lives = subjects(10, 7, 10, 20, 30);
    let events: Vec<serde_json::Value> = lives
        .iter()
        .map(|s| serde_json::json!({"birth": birth_json(&s.birth), "event": s.event.get()}))
        .collect();
    let timed = sent(
        &sdk,
        &serde_json::json!({"study": "TIMED", "rules": rules, "subjects": events,
            "dasha": "dasha_system.VIMSHOTTARI", "depth": 1, "shuffle": "AGES_AT_EVENT",
            "test": {"seed": 3, "permutations": 99}}),
    )
    .unwrap();
    let study = EventStudy::new(
        &lives,
        &request,
        &set,
        DashaSystem::Vimshottari,
        EventShuffle::AgesAtEvent,
    )
    .to_depth(Depth::MIN);
    let direct = sdk
        .research()
        .timed(&study, None, &EventTest::new(3, 99))
        .unwrap();
    assert_eq!(timed, ResearchAnswer::Tested(direct));
    let matched = sent(
        &sdk,
        &serde_json::json!({"study": "TIMED", "rules": rules, "subjects": events,
            "dasha": "dasha_system.VIMSHOTTARI", "shuffle": "SELF_MATCHED",
            "referents": {"windowDays": 365.25, "stepDays": 30},
            "test": {"seed": 3, "permutations": 99}}),
    )
    .unwrap();
    let study = EventStudy::new(
        &lives,
        &request,
        &set,
        DashaSystem::Vimshottari,
        EventShuffle::SelfMatched,
    )
    .self_matched(Referents::new(365.25, 30.0));
    let direct = sdk
        .research()
        .timed(&study, None, &EventTest::new(3, 99))
        .unwrap();
    assert_eq!(matched, ResearchAnswer::Tested(direct));
}

/// A context's own dasha system delivers as the catalogue's does: here
/// Vimshottari's table under the consumer's key, so every row agrees with
/// the catalogued study's and only the registration the hash seals differs.
#[test]
fn an_event_study_reads_a_dasha_the_context_registered() {
    let twin = UduDefinition {
        lords: VIMSHOTTARI.lords.to_vec(),
        sources: vec![String::from("Vimshottari's table, for the test")],
        ..UduDefinition::of("ACME_VIMSHOTTARI", Nakshatra::Ashwini)
    };
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .dasha_system(twin)
        .build()
        .unwrap();
    let rules = serde_json::json!({"shipped": ["YOGAS"]});
    let lives = subjects(8, 11, 10, 20, 30);
    let events: Vec<serde_json::Value> = lives
        .iter()
        .map(|s| serde_json::json!({"birth": birth_json(&s.birth), "event": s.event.get()}))
        .collect();
    let timed = |sdk: &Context, dasha: &str| {
        sent(
            sdk,
            &serde_json::json!({"study": "TIMED", "rules": rules, "subjects": events,
                "dasha": dasha, "shuffle": "AGES_AT_EVENT",
                "test": {"seed": 5, "permutations": 99}}),
        )
    };
    let Ok(ResearchAnswer::Tested(catalogued)) = timed(&sdk, "VIMSHOTTARI") else {
        panic!("the catalogue's system reads");
    };
    for key in ["ACME_VIMSHOTTARI", "dasha_system.ACME_VIMSHOTTARI"] {
        let Ok(ResearchAnswer::Tested(registered)) = timed(&sdk, key) else {
            panic!("{key} reads");
        };
        assert_eq!(registered.value, catalogued.value, "{key}");
        assert_ne!(
            registered.provenance.input_hash,
            catalogued.provenance.input_hash
        );
    }
    // The same call as the record makes.
    let set = yogas();
    let request = template();
    let direct = sdk
        .research()
        .timed(
            &EventStudy::new(
                &lives,
                &request,
                &set,
                "ACME_VIMSHOTTARI",
                EventShuffle::AgesAtEvent,
            ),
            None,
            &EventTest::new(5, 99),
        )
        .unwrap();
    assert_eq!(direct.value, catalogued.value);

    // A context without it refuses the key where it was asked, and a key
    // of another kind is refused before any context reads it.
    let unknown = timed(&self::sdk(), "ACME_VIMSHOTTARI").unwrap_err();
    assert_eq!(unknown.field(), Some("dasha"), "{unknown:?}");
    let other = timed(&sdk, "graha.SUN").unwrap_err();
    assert_eq!(other.field(), Some("research.dasha"), "{other:?}");
}

#[test]
fn a_record_is_refused_where_it_stands() {
    let sdk = sdk();
    let births: Vec<serde_json::Value> = births(4, 2).iter().map(birth_json).collect();
    let refused = |request: serde_json::Value| sent(&sdk, &request).unwrap_err();
    let base = serde_json::json!({"study": "COUNTS", "rules": {"shipped": ["YOGAS"]},
        "births": births, "design": {"groups": [0, 0, 1, 1]}});
    let with = |field: &str, value: serde_json::Value| {
        let mut request = base.clone();
        request[field] = value;
        request
    };
    let without = |field: &str| {
        let mut request = base.clone();
        request.as_object_mut().unwrap().remove(field);
        request
    };
    for (request, field) in [
        (without("design"), "research.design"),
        (without("births"), "research.births"),
        (with("births", serde_json::json!([])), "research.births"),
        (with("test", serde_json::json!({})), "research.test"),
        (with("depth", serde_json::json!(2)), "research.depth"),
        (with("study", serde_json::json!("SURVEY")), "research.study"),
        (
            with("rules", serde_json::json!({"shipped": ["NONE"]})),
            "research.rules.shipped[0]",
        ),
        (with("holds", serde_json::json!("ALWAYS")), "research.holds"),
        (with("unasked", serde_json::json!(1)), "research.unasked"),
        (
            with(
                "referents",
                serde_json::json!({"windowDays": 28, "stepDays": 7}),
            ),
            "research.referents",
        ),
    ] {
        assert_eq!(refused(request).field(), Some(field), "{field}");
    }
    let mut far = base.clone();
    far["births"][2]["latitudeDeg"] = serde_json::json!(91.0);
    assert_eq!(refused(far).field(), Some("research.births[2].latitudeDeg"));
    let mut late = base.clone();
    late["births"][1]["uncertaintyMinutes"] = serde_json::json!(721.0);
    assert_eq!(
        refused(late).field(),
        Some("births[1].uncertaintyMinutes"),
        "a façade refusal names the field the façade does",
    );
    for step in [0.0, -1.0, 721.0] {
        assert_eq!(
            refused(with("uncertaintyStepMinutes", serde_json::json!(step))).field(),
            Some("uncertaintyStepMinutes"),
            "{step}",
        );
    }
    let compare = |test: serde_json::Value| {
        let mut request = with("study", serde_json::json!("COMPARE"));
        request["test"] = test;
        refused(request)
    };
    let bad_seed = compare(serde_json::json!({"seed": "-1", "permutations": 9,
        "contrast": {"kind": "ANY_DIFFERENCE"}}));
    assert_eq!(bad_seed.field(), Some("research.test.seed"));
    let no_contrast = compare(serde_json::json!({"seed": 1, "permutations": 9}));
    assert_eq!(no_contrast.field(), Some("research.test"));
    let timed = serde_json::json!({"study": "TIMED", "rules": {"shipped": ["YOGAS"]},
        "subjects": [{"birth": base["births"][0], "event": 2_460_000.5}],
        "dasha": "VIMSHOTTARI", "depth": 7, "shuffle": "EVENT_DATES",
        "test": {"seed": 1, "permutations": 9}});
    assert_eq!(refused(timed.clone()).field(), Some("research.depth"));
    // A timed study names its null; neither shuffle is a default (C364).
    let mut unshuffled = timed;
    unshuffled["depth"] = serde_json::json!(2);
    unshuffled.as_object_mut().unwrap().remove("shuffle");
    assert_eq!(
        refused(unshuffled.clone()).field(),
        Some("research.shuffle")
    );
    // A timed study reads each birth at its instant; a step is not its own.
    let mut stepped = unshuffled;
    stepped["shuffle"] = serde_json::json!("EVENT_DATES");
    stepped["uncertaintyStepMinutes"] = serde_json::json!(5);
    assert_eq!(
        refused(stepped).field(),
        Some("research.uncertaintyStepMinutes")
    );
}

#[test]
fn a_self_matched_record_names_its_referents_where_they_are_wrong() {
    let sdk = sdk();
    let birth = birth_json(&births(1, 2)[0]);
    let matched = serde_json::json!({"study": "TIMED", "rules": {"shipped": ["YOGAS"]},
        "subjects": [{"birth": birth, "event": 2_460_000.5}],
        "dasha": "VIMSHOTTARI", "shuffle": "SELF_MATCHED",
        "test": {"seed": 1, "permutations": 9}});
    // Referents are a record of two lengths, each named where it is wrong.
    for (referents, field) in [
        (serde_json::json!({"windowDays": 28}), "research.referents"),
        (
            serde_json::json!({"windowDays": 28, "stepDays": 7, "anchor": 0}),
            "research.referents.anchor",
        ),
        (
            serde_json::json!({"windowDays": "month", "stepDays": 7}),
            "research.referents.windowDays",
        ),
    ] {
        let mut request = matched.clone();
        request["referents"] = referents;
        assert_eq!(
            sent(&sdk, &request).unwrap_err().field(),
            Some(field),
            "{field}"
        );
    }
}

/// A rule of the sky in transit, read at each event's instant: Saturn back
/// in his natal sign, the Saturn return near the thirtieth year.
fn saturn_return() -> teistro::RuleSet {
    let rule: teistro::rules::Rule = serde_json::from_str(
        r#"{"key": "SATURN_RETURN", "category": "transit", "source": {"text": "a test"},
            "conditions": [{"type": "transit-in-house", "planet": "SATURN", "houses": [1], "from": "SATURN"}]}"#,
    )
    .unwrap();
    RuleRequest::default()
        .with_rules([rule])
        .rule_set()
        .unwrap()
}

#[test]
fn a_transit_is_read_at_each_event_and_its_cost_is_chosen() {
    // Events planted at the Saturn return, 10 759 days (his period) after
    // each birth, give or take two months. Moving the dates among people
    // breaks it; keeping the ages cannot, since every age is the same
    // return: the date shuffle finds what the age shuffle by design keeps.
    let sdk = sdk();
    let set = saturn_return();
    let request = template();
    let mut rng = SplitMix64::new(11);
    let subjects: Vec<Subject> = births(40, 11)
        .into_iter()
        .map(|birth| {
            let born = 2_433_282.5 + rng.below(30 * 365) as f64 + 0.25;
            let age = 10_759.0 - 60.0 + rng.below(120) as f64;
            Subject {
                birth: Birth {
                    instant: JulianDay::<Utc>::literal(born),
                    ..birth
                },
                event: JulianDay::<Utc>::literal(born + age),
            }
        })
        .collect();
    let study = EventStudy::new(
        &subjects,
        &request,
        &set,
        DashaSystem::Vimshottari,
        EventShuffle::EventDates,
    );
    let test = EventTest {
        after_birth: AfterBirth::RestrictPairings,
        ..EventTest::new(5, 999)
    };
    let dated = sdk.research().timed(&study, None, &test).unwrap().value;
    let row = &dated.rows[0];
    let seen = row.expected.unwrap();
    // Most, not all: a natal Saturn near a sign's edge can stand a sign
    // over two months either side of his exact return.
    assert!(seen.observed > 0.6, "{row:?}");
    assert!(seen.expected < 0.3, "{row:?}");
    assert!(row.p.value < 0.01, "{row:?}");
    let aged = sdk
        .research()
        .timed(
            &study.clone().shuffled(EventShuffle::AgesAtEvent),
            None,
            &test,
        )
        .unwrap()
        .value;
    let kept = aged.rows[0].expected.unwrap();
    assert_eq!(kept.observed, seen.observed);
    assert!((kept.observed - kept.expected).abs() < 0.1, "{kept:?}");

    // The age shuffle reads the sky at every pairing's instant, up to 40²
    // of them (fewer where two pairings fall on one instant); a cap below
    // that is refused with the count it would need.
    let capped = EventTest {
        max_pairs: 1_000,
        ..test.clone()
    };
    let refused = sdk
        .research()
        .timed(
            &study.clone().shuffled(EventShuffle::AgesAtEvent),
            None,
            &capped,
        )
        .unwrap_err();
    assert_eq!(refused.field(), Some("test.maxPairs"));
    let needed: u64 = refused
        .message
        .split_once("at ")
        .and_then(|(_, rest)| rest.split_once(" instants"))
        .and_then(|(count, _)| count.parse().ok())
        .unwrap();
    assert!((1_001..=1_600).contains(&needed), "{}", refused.message);
    // The date shuffle reads it at each event a place at a time: 40 events
    // at each of three places at most.
    sdk.research().timed(&study, None, &capped).unwrap();
    // The cap does not change an answer, so it is not registered.
    let again = sdk.research().timed(&study, None, &capped).unwrap();
    let first = sdk.research().timed(&study, None, &test).unwrap();
    assert_eq!(again.provenance.input_hash, first.provenance.input_hash);

    // Where no instant is read, a transit rule is refused rather than
    // silently absent.
    let births: Vec<Birth> = subjects.iter().map(|s| s.birth).collect();
    let natal = sdk
        .chart()
        .readings_with_rules(&[births[0].instant], &request, &set)
        .unwrap_err();
    assert_eq!(natal.field(), Some("rules"));
    let counted = sdk
        .research()
        .counts(
            Study::new(&births, &request, &set),
            &Design::new(vec![0; births.len()]),
        )
        .unwrap_err();
    assert_eq!(counted.field(), Some("rules"));
}
