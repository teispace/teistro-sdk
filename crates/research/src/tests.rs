//! The kernel's promises (`research.md` §3, tests 3, 6, 7 and 8's kernel
//! half), each with a fixed seed so its counts are exact.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    clippy::float_cmp,
    reason = "tests fail by panicking, index their own tables, and compare bits they expect to be equal"
)]

use std::num::NonZeroU16;

use super::*;

/// A batch of `charts` charts, the first `cases` of them group 1, and
/// predicates drawn from `seed`, predicate `j` holding on a chart with
/// probability `(j + 1) / (predicates + 1)`.
fn study(charts: usize, cases: usize, predicates: usize, seed: u64) -> (Matrix, Design) {
    let mut rng = SplitMix64::new(seed);
    let mut matrix = Matrix::new(charts);
    for j in 0..predicates {
        let threshold = (j as u64 + 1) * (u64::MAX / (predicates as u64 + 1));
        let cells: Vec<Cell> = (0..charts)
            .map(|_| {
                if rng.next_u64() < threshold {
                    Cell::Present
                } else {
                    Cell::Absent
                }
            })
            .collect();
        matrix.push(format!("p{j}"), &cells).unwrap();
    }
    let groups = (0..charts).map(|i| u16::from(i < cases)).collect();
    (matrix, Design::new(groups))
}

fn case_test(seed: u64, permutations: u32) -> GroupTest {
    GroupTest::new(seed, permutations, Contrast::CaseVsRest { case: 1 })
}

/// Whether `count` of `draws` lies within five standard deviations of a
/// binomial's mean at `share`.
fn near(count: u64, draws: u64, share: f64) -> bool {
    let mean = draws as f64 * share;
    let sd = (draws as f64 * share * (1.0 - share)).sqrt();
    (count as f64 - mean).abs() <= 5.0 * sd
}

#[test]
fn splitmix_is_the_published_generator() {
    // Vigna's reference splitmix64.c, seeded with 0.
    let mut rng = SplitMix64::new(0);
    assert_eq!(rng.next_u64(), 0xE220_A839_7B1D_CDAF);
    assert_eq!(rng.next_u64(), 0x6E78_9E6A_A1B9_65F4);
}

#[test]
fn a_permutation_stream_depends_on_its_index_alone() {
    let a: Vec<u64> = (0..4)
        .map(|_| SplitMix64::for_permutation(9, 3).next_u64())
        .collect();
    assert!(a.windows(2).all(|w| w[0] == w[1]));
    assert_ne!(stream_seed(9, 3), stream_seed(9, 4));
    assert_ne!(stream_seed(9, 3), stream_seed(10, 3));
}

#[test]
fn fisher_yates_hits_every_order_of_four_fairly() {
    // Test 7: 24 orders, 240 000 shuffles, each order within its binomial.
    let draws = 240_000_u64;
    let mut seen: BTreeMap<[u8; 4], u64> = BTreeMap::new();
    let mut rng = SplitMix64::new(0x5EED);
    for _ in 0..draws {
        let mut items = [0_u8, 1, 2, 3];
        rng.shuffle(&mut items);
        *seen.entry(items).or_default() += 1;
    }
    assert_eq!(seen.len(), 24);
    for (order, &count) in &seen {
        assert!(
            near(count, draws, 1.0 / 24.0),
            "{order:?} drawn {count} times"
        );
    }
}

#[test]
fn lemire_is_unbiased_off_a_power_of_two() {
    let mut rng = SplitMix64::new(42);
    for bound in [3_u64, 5, 7, 12, 1000] {
        let draws = 60_000 * bound.min(12);
        let mut counts = vec![0_u64; usize::try_from(bound).unwrap()];
        for _ in 0..draws {
            counts[usize::try_from(rng.below(bound)).unwrap()] += 1;
        }
        let share = 1.0 / bound as f64;
        // A thousand cells each hold too few draws for five deviations to
        // mean much, so that bound is checked by its extreme cells' sum.
        if bound <= 12 {
            for (value, &count) in counts.iter().enumerate() {
                assert!(near(count, draws, share), "{value} of {bound}: {count}");
            }
        } else {
            assert_eq!(counts.iter().sum::<u64>(), draws);
            assert!(counts.iter().all(|&c| c > 0));
        }
    }
    // A bound just over half the range rejects nearly half its products,
    // and every draw still lands below it.
    let huge = (1_u64 << 63) + 1;
    assert!((0..1000).all(|_| rng.below(huge) < huge));
}

#[test]
fn the_distributions_meet_their_tables() {
    assert!((special::normal_quantile(0.975) - 1.959_963_984_540_054).abs() < 1e-12);
    assert!((special::normal_quantile(0.5)).abs() < 1e-15);
    // 0 of 10: the upper end is 1 − 0.025^(1/10).
    let (low, high) = special::clopper_pearson(0, 10, 0.95);
    assert_eq!(low, 0.0);
    assert!((high - 0.308_497_2).abs() < 1e-6);
    // 5 of 10, the textbook's symmetric case.
    let (low, high) = special::clopper_pearson(5, 10, 0.95);
    assert!((low - 0.187_086_4).abs() < 1e-6, "{low}");
    assert!((high - 0.812_913_6).abs() < 1e-6, "{high}");
    let total: f64 = special::hypergeometric(10, 5, 5)
        .iter()
        .map(|(_, p)| p)
        .sum();
    assert!((total - 1.0).abs() < 1e-12);
}

#[test]
fn the_corrections_meet_a_hand_count() {
    let p = [0.01, 0.04, 0.03, 0.005];
    let close = |a: &[f64], b: &[f64]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-12);
    assert!(close(&correct::bonferroni(&p), &[0.04, 0.16, 0.12, 0.02]));
    assert!(close(&correct::holm(&p), &[0.03, 0.06, 0.06, 0.02]));
    assert!(close(
        &correct::benjamini_hochberg(&p),
        &[0.02, 0.04, 0.04, 0.02]
    ));
    let c = 1.0 + 0.5 + 1.0 / 3.0 + 0.25;
    assert!(close(
        &correct::benjamini_yekutieli(&p),
        &[0.02 * c, 0.04 * c, 0.04 * c, 0.02 * c]
    ));
}

#[test]
fn the_permutation_p_meets_the_exact_p() {
    // Test 3: every chart read, two groups, no strata. Each predicate's
    // exact p lies inside the permutation p's own Monte Carlo interval.
    let (matrix, design) = study(60, 20, 12, 3);
    for alternative in [
        Alternative::TwoSided,
        Alternative::Greater,
        Alternative::Less,
    ] {
        let mut test = case_test(11, 20_000);
        test.alternative = alternative;
        test.level = 0.999;
        let tested = compare(&matrix, &design, &test).unwrap();
        for (j, row) in tested.rows.iter().enumerate() {
            let exact = row.exact.unwrap();
            assert!(
                row.p.low <= exact && exact <= row.p.high,
                "{alternative:?} predicate {j}: exact {exact}, permutation {:?}",
                row.p
            );
        }
    }
}

#[test]
fn every_thread_count_gives_the_same_bits() {
    // Test 6, the kernel's half: 1, 2 and 8 threads, and an uneven split.
    let (matrix, mut design) = study(97, 31, 9, 5);
    design.strata = Some((0..97).map(|i| i % 4).collect());
    let mut test = case_test(2026, 3_001);
    test.alpha = Some(0.05);
    let one = serde_json::to_string(&compare(&matrix, &design, &test).unwrap()).unwrap();
    for threads in [2_u16, 3, 8] {
        test.parallelism = Parallelism::Threads(NonZeroU16::new(threads).unwrap());
        let many = serde_json::to_string(&compare(&matrix, &design, &test).unwrap()).unwrap();
        assert_eq!(one, many, "{threads} threads");
    }
    test.seed = 2027;
    test.parallelism = Parallelism::One;
    let other = serde_json::to_string(&compare(&matrix, &design, &test).unwrap()).unwrap();
    assert_ne!(one, other);
}

#[test]
fn max_t_uses_the_family_s_dependence() {
    // Two copies of one predicate are one test: max-T leaves their p as it
    // is, where Holm and Bonferroni double it.
    let (single, design) = study(40, 15, 1, 8);
    let mut cells = Vec::new();
    let tested = compare(&single, &design, &case_test(1, 4_999)).unwrap();
    let base = tested.rows[0].p.value;
    for index in 0..40 {
        cells.push(if single.columns[0].present.contains(index) {
            Cell::Present
        } else {
            Cell::Absent
        });
    }
    let mut twice = Matrix::new(40);
    twice.push("once", &cells).unwrap();
    twice.push("again", &cells).unwrap();
    let tested = compare(&twice, &design, &case_test(1, 4_999)).unwrap();
    for row in &tested.rows {
        assert_eq!(row.p.value, base);
        assert_eq!(row.adjusted.max_t, base);
        assert!((row.adjusted.bonferroni - (2.0 * base).min(1.0)).abs() < 1e-15);
    }
}

#[test]
fn an_unreadable_chart_leaves_the_denominator() {
    let mut matrix = Matrix::new(6);
    matrix
        .push(
            "edge",
            &[
                Cell::Present,
                Cell::Unreadable,
                Cell::Unstable,
                Cell::Absent,
                Cell::Present,
                Cell::Absent,
            ],
        )
        .unwrap();
    let design = Design::new(vec![1, 1, 1, 0, 0, 0]);
    let tested = compare(&matrix, &design, &case_test(4, 999)).unwrap();
    let row = &tested.rows[0];
    assert_eq!(
        row.counts[1],
        GroupCount {
            present: 1,
            absent: 0,
            unreadable: 1,
            unstable: 1
        }
    );
    assert_eq!(row.counts[0].present + row.counts[0].absent, 3);
    // One case read, so the cases' share is 1 of 1, not 1 of 3.
    assert_eq!(row.effect.unwrap().risk_case.estimate, 1.0);
    // Not every chart was read, so the exact p is not offered.
    assert_eq!(row.exact, None);
}

/// Refuses any key with an underscore anywhere in `value`.
fn snake_free(value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, inner) in map {
                assert!(!key.contains('_'), "{key}");
                snake_free(inner);
            }
        }
        serde_json::Value::Array(items) => items.iter().for_each(snake_free),
        _ => {}
    }
}

#[test]
fn the_answer_speaks_camel_case() {
    let (matrix, design) = study(20, 8, 2, 1);
    let mut test = case_test(3, 99);
    test.alpha = Some(0.5);
    let json = serde_json::to_value(compare(&matrix, &design, &test).unwrap()).unwrap();
    snake_free(&json);
    assert_eq!(json["shuffle"], "research/shuffle/1");
    let back: GroupTest = serde_json::from_str(
        r#"{"seed":3,"permutations":99,"contrast":{"kind":"CASE_VS_REST","case":1},"alpha":0.5}"#,
    )
    .unwrap();
    assert_eq!(back, test);
}

/// The field a refused request names.
fn refused(matrix: &Matrix, design: &Design, test: &GroupTest) -> String {
    compare(matrix, design, test)
        .unwrap_err()
        .field()
        .unwrap()
        .to_owned()
}

#[test]
fn every_refusal_names_its_field() {
    let (matrix, design) = study(10, 4, 3, 2);
    let good = case_test(1, 999);
    assert_eq!(
        refused(&Matrix::new(1), &Design::new(vec![0]), &good),
        "groups"
    );
    assert_eq!(refused(&matrix, &Design::new(vec![0, 1]), &good), "groups");
    assert_eq!(refused(&matrix, &Design::new(vec![0; 10]), &good), "groups");
    let gap = Design::new(vec![0, 0, 2, 2, 2, 0, 0, 2, 2, 2]);
    assert_eq!(refused(&matrix, &gap, &good), "groups");
    assert_eq!(refused(&Matrix::new(10), &design, &good), "predicates");
    let short = design.clone().with_strata(vec![0; 3]);
    assert_eq!(refused(&matrix, &short, &good), "strata");
    let frozen = design
        .clone()
        .with_strata(design.groups.iter().map(|&g| u32::from(g)).collect());
    assert_eq!(refused(&matrix, &frozen, &good), "strata");
    let stranger = GroupTest::new(1, 999, Contrast::CaseVsRest { case: 5 });
    assert_eq!(refused(&matrix, &design, &stranger), "contrast.case");
    let mut directed = GroupTest::new(1, 999, Contrast::AnyDifference);
    directed.alternative = Alternative::Greater;
    assert_eq!(refused(&matrix, &design, &directed), "alternative");
    assert_eq!(refused(&matrix, &design, &case_test(1, 0)), "permutations");
    assert_eq!(
        refused(&matrix, &design, &case_test(1, MAX_PERMUTATIONS + 1)),
        "permutations"
    );
    let mut level = good.clone();
    level.level = 1.0;
    assert_eq!(refused(&matrix, &design, &level), "level");
    let mut alpha = good.clone();
    alpha.alpha = Some(0.0);
    assert_eq!(refused(&matrix, &design, &alpha), "alpha");
    // 49 permutations resolve 0.02, above 0.05 / 3; 59 resolve 1/60.
    let mut coarse = case_test(1, 49);
    coarse.alpha = Some(0.05);
    let error = compare(&matrix, &design, &coarse).unwrap_err();
    assert_eq!(error.field(), Some("permutations"));
    assert_eq!(error.hint(), Some("at least 59 permutations"));
    let mut wide = Matrix::new(10);
    assert_eq!(
        wide.push("short", &[Cell::Present; 3]).unwrap_err().field(),
        Some("predicates[0]")
    );
}

#[test]
fn a_chi_square_reads_every_group() {
    let (matrix, _) = study(90, 30, 4, 6);
    let design = Design::new((0..90).map(|i| u16::try_from(i % 3).unwrap()).collect());
    let tested = compare(
        &matrix,
        &design,
        &GroupTest::new(5, 1_999, Contrast::AnyDifference),
    )
    .unwrap();
    for row in &tested.rows {
        assert_eq!(row.counts.len(), 3);
        assert!(row.observed >= 0.0);
        assert!(row.effect.is_none() && row.exact.is_none());
        assert!(row.adjusted.max_t >= row.p.value);
    }
}

#[test]
fn counts_and_rows_carry_their_predicates_names() {
    let (matrix, design) = study(12, 5, 3, 9);
    let counted = counts(&matrix, &design).unwrap();
    let names: Vec<&str> = counted
        .rows
        .iter()
        .map(|row| row.predicate.as_str())
        .collect();
    assert_eq!(names, ["p0", "p1", "p2"]);
    for row in &counted.rows {
        let total: u32 = row.counts.iter().map(|c| c.present + c.absent).sum();
        assert_eq!(total, 12);
    }
    let tested = compare(&matrix, &design, &case_test(1, 99)).unwrap();
    assert_eq!(tested.rows[2].predicate, "p2");
    assert_eq!(tested.rows[2].counts, counted.rows[2].counts);
    let mut twice = Matrix::new(2);
    twice.push("same", &[Cell::Present, Cell::Absent]).unwrap();
    let error = twice
        .push("same", &[Cell::Present, Cell::Absent])
        .unwrap_err();
    assert_eq!(error.field(), Some("predicates[1]"));
}

/// An event study over `n` subjects, one predicate holding at each
/// subject's own event with probability `own` and elsewhere with `base`.
fn events(n: usize, own: f64, base: f64, seed: u64) -> PairMatrix {
    let mut rng = SplitMix64::new(seed);
    let mut draws: Vec<bool> = Vec::with_capacity(n * n);
    for i in 0..n {
        for j in 0..n {
            let share = if i == j { own } else { base };
            draws.push((rng.next_u64() as f64 / u64::MAX as f64) < share);
        }
    }
    let mut matrix = PairMatrix::new(n).unwrap();
    matrix
        .push("delivers", |i, j| {
            if draws[i * n + j] {
                Cell::Present
            } else {
                Cell::Absent
            }
        })
        .unwrap();
    matrix
}

#[test]
fn a_planted_delivery_is_found_and_a_null_is_not() {
    // Test 5's kernel half.
    let mut test = EventTest::new(5, 1_999);
    test.alternative = Alternative::Greater;
    let planted = timed(&events(120, 0.6, 0.3, 1), None, &test).unwrap();
    let row = &planted.rows[0];
    assert!(row.p.value < 0.001, "{row:?}");
    let expected = row.expected.unwrap();
    assert!(expected.ratio.unwrap() > 1.5);
    assert!((expected.expected - 0.3).abs() < 0.03);
    let null = timed(&events(120, 0.3, 0.3, 2), None, &test).unwrap();
    assert!(null.rows[0].p.value > 0.01, "{:?}", null.rows[0]);
}

#[test]
fn a_pairing_before_a_birth_is_refused_or_never_drawn() {
    let n = 30;
    let mut matrix = PairMatrix::new(n).unwrap();
    // Subjects 0 to 9 were born after events 20 to 29.
    for i in 0..10 {
        for j in 20..n {
            matrix.forbid(i, j);
        }
    }
    // A predicate present exactly on the forbidden pairs.
    matrix
        .push("forbidden", |i, j| {
            if i < 10 && j >= 20 {
                Cell::Present
            } else {
                Cell::Absent
            }
        })
        .unwrap();
    let mut test = EventTest::new(3, 499);
    let error = timed(&matrix, None, &test).unwrap_err();
    assert_eq!(error.field(), Some("afterBirth"));
    test.after_birth = AfterBirth::RestrictPairings;
    test.alternative = Alternative::Less;
    // Never drawn: every permuted count is zero, as the observed one is,
    // so every permutation reaches it.
    let tested = timed(&matrix, None, &test).unwrap();
    assert_eq!(tested.rows[0].p.exceed, 499);
    // Where no pairing is possible, rejection gives up and says so.
    let mut stuck = PairMatrix::new(3).unwrap();
    stuck.forbid(0, 1);
    stuck.forbid(0, 2);
    stuck.forbid(1, 2);
    stuck.forbid(1, 0);
    stuck.push("any", |_, _| Cell::Absent).unwrap();
    // Only the identity remains, which a shuffle draws a sixth of the time.
    assert!(timed(&stuck, None, &test).is_ok());
}

#[test]
fn an_event_study_is_the_same_at_every_thread_count() {
    let matrix = events(64, 0.5, 0.4, 3);
    let strata: Vec<u32> = (0..64).map(|i| i % 3).collect();
    let mut test = EventTest::new(8, 1_001);
    let one = timed(&matrix, Some(&strata), &test).unwrap();
    test.parallelism = Parallelism::Threads(NonZeroU16::new(5).unwrap());
    assert_eq!(one, timed(&matrix, Some(&strata), &test).unwrap());
    assert_eq!(
        timed(&matrix, Some(&[0; 3]), &test).unwrap_err().field(),
        Some("strata")
    );
    let singletons: Vec<u32> = (0..64).collect();
    assert_eq!(
        timed(&matrix, Some(&singletons), &test)
            .unwrap_err()
            .field(),
        Some("strata")
    );
    assert_eq!(PairMatrix::new(1).unwrap_err().field(), Some("subjects"));
}

#[test]
fn a_sample_is_read_against_its_own_replicates() {
    // Twenty replicates whose share of predicate 0 wanders about 0.3, an
    // observed 0.6 beyond all of them, and predicate 1 at their mean.
    let mut rng = SplitMix64::new(4);
    let replicates: Vec<Vec<(u32, u32)>> = (0..199)
        .map(|_| {
            let wander = u32::try_from(rng.below(21)).unwrap();
            vec![(20 + wander, 100), (50, 100)]
        })
        .collect();
    let names = vec!["rare".to_owned(), "even".to_owned()];
    let observed = [
        GroupCount {
            present: 60,
            absent: 40,
            unreadable: 2,
            unstable: 1,
        },
        GroupCount {
            present: 50,
            absent: 50,
            unreadable: 0,
            unstable: 0,
        },
    ];
    let test = ReplicateTest {
        alternative: Alternative::Greater,
        ..ReplicateTest::default()
    };
    let tested = replicated(&names, &observed, &replicates, &test).unwrap();
    let rare = &tested.rows[0];
    assert_eq!(rare.p.exceed, 0);
    assert_eq!(rare.p.value, 1.0 / 200.0);
    let expectation = rare.expected.unwrap();
    assert!((expectation.expected - 0.3).abs() < 0.01);
    assert!((expectation.ratio.unwrap() - 2.0).abs() < 0.1);
    // Every replicate equals the observed share, so every one reaches it.
    assert_eq!(tested.rows[1].p.exceed, 199);
    assert_eq!(tested.rows[1].observed, 0.0);
    assert_eq!(rare.counts[0].unstable, 1);
    // Refusals name their fields.
    let field = |replicates: &[Vec<(u32, u32)>], names: &[String]| {
        replicated(names, &observed, replicates, &test)
            .unwrap_err()
            .field()
            .map(str::to_owned)
    };
    assert_eq!(field(&[], &names).as_deref(), Some("replicates"));
    assert_eq!(
        field(&[vec![(1, 2)]], &names).as_deref(),
        Some("replicates[0]")
    );
    assert_eq!(
        field(&replicates, &names[..1]).as_deref(),
        Some("predicates")
    );
    // Ten replicates cannot reach 0.05 over a family of two.
    let strict = ReplicateTest {
        alpha: Some(0.05),
        ..test
    };
    let short = replicated(&names, &observed, &replicates[..10], &strict).unwrap_err();
    assert_eq!(short.field(), Some("replicates"));
}
