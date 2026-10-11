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
    assert_eq!(json["shuffle"], "research/shuffle/2");
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
        assert!(row.observed.is_some_and(|observed| observed >= 0.0));
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
    // No dates make this board (subject 0 would be born after event 1 and
    // before event 0, subject 1 the other way round), so the exact sampler
    // refuses it; rejection, which asks nothing of the board, draws the
    // identity a sixth of the time.
    assert_eq!(
        timed(&stuck, None, &test).unwrap_err().field(),
        Some("afterBirth")
    );
    test.shuffle = ShuffleVersion::V1;
    assert!(timed(&stuck, None, &test).is_ok());
}

/// A board whose event `j` may go to the subjects `0..eligible[j]`, as
/// subjects born in order and events dated after the first `eligible[j]`
/// births make it.
fn board_of(eligible: &[usize]) -> PairMatrix {
    let n = eligible.len();
    let mut matrix = PairMatrix::new(n).unwrap();
    for (j, &count) in eligible.iter().enumerate() {
        for i in count..n {
            matrix.forbid(i, j);
        }
    }
    matrix
}

/// How often each pairing of `board` is drawn in `draws` draws, by the
/// exact sampler or by rejection.
fn drawn(board: &PairMatrix, draws: u64, exact: bool) -> BTreeMap<Vec<u16>, u64> {
    let n = board.subjects();
    let identity: Vec<u16> = (0..u16::try_from(n).unwrap()).collect();
    let layout = Layout::new(&identity, 0, None);
    let exact_board = events::Board::of(board, &layout).unwrap();
    let (mut pairing, mut pool) = (vec![0; n], Vec::new());
    let mut seen = BTreeMap::new();
    for k in 0..draws {
        let mut rng = SplitMix64::for_permutation(77, k);
        if exact {
            exact_board.draw(&mut rng, &mut pairing, &mut pool);
        } else {
            loop {
                layout.permute_with(&mut rng, &mut pairing, &mut pool);
                let allowed = pairing
                    .iter()
                    .enumerate()
                    .all(|(i, &j)| board.allows(i, usize::from(j)));
                if allowed {
                    break;
                }
            }
        }
        *seen.entry(pairing.clone()).or_default() += 1;
    }
    seen
}

#[test]
fn a_board_of_one_pairing_draws_it_every_time() {
    // Eligible counts 1, 2, 3: one pairing, 1·1·1.
    let seen = drawn(&board_of(&[1, 2, 3]), 1_000, true);
    assert_eq!(seen.len(), 1);
    assert_eq!(seen.get(&vec![0, 1, 2]), Some(&1_000));
}

#[test]
fn the_exact_sampler_draws_every_allowed_pairing_alike() {
    // Eligible counts 2, 3, 4, 4: 2·2·2·1 = 8 pairings, each within its
    // binomial over 80 000 draws, and rejection agrees cell for cell.
    let board = board_of(&[2, 3, 4, 4]);
    for exact in [true, false] {
        let seen = drawn(&board, 80_000, exact);
        assert_eq!(seen.len(), 8, "exact {exact}: {seen:?}");
        for (pairing, &count) in &seen {
            assert!(
                near(count, 80_000, 1.0 / 8.0),
                "exact {exact}: {pairing:?} {count}"
            );
            for (i, &j) in pairing.iter().enumerate() {
                assert!(board.allows(i, usize::from(j)), "{pairing:?}");
            }
        }
    }
}

#[test]
fn a_cohort_rejection_cannot_pair_is_paired_exactly() {
    // Sixty subjects born in order, each event twenty births after its
    // own: a random pairing almost never keeps every event after its
    // birth, so rejection gives up, and the exact sampler does not.
    let n = 60;
    let mut matrix = board_of(&(0..n).map(|j| (j + 21).min(n)).collect::<Vec<_>>());
    matrix
        .push("p", |i, j| {
            if (i + j) % 3 == 0 {
                Cell::Present
            } else {
                Cell::Absent
            }
        })
        .unwrap();
    let mut test = EventTest::new(5, 199);
    test.after_birth = AfterBirth::RestrictPairings;
    test.shuffle = ShuffleVersion::V1;
    assert_eq!(
        timed(&matrix, None, &test).unwrap_err().field(),
        Some("afterBirth")
    );
    test.shuffle = ShuffleVersion::V2;
    let tested = timed(&matrix, None, &test).unwrap();
    assert_eq!(tested.shuffle, ShuffleVersion::V2);
    // An unrestricted study draws what /1 drew, bit for bit.
    let free = events(40, 0.5, 0.4, 9);
    let mut plain = EventTest::new(5, 299);
    let second = timed(&free, None, &plain).unwrap();
    plain.shuffle = ShuffleVersion::V1;
    let first = timed(&free, None, &plain).unwrap();
    assert_eq!(first.rows, second.rows);
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
    assert_eq!(tested.rows[1].observed, Some(0.0));
    // Replicates that all agree, a sample beyond them: no finite statistic
    // says how far, and the p-value still ranks it past every one.
    let agreeing: Vec<Vec<(u32, u32)>> = (0..19).map(|_| vec![(20, 100), (50, 100)]).collect();
    let beyond = replicated(&names, &observed, &agreeing, &test).unwrap();
    assert_eq!(beyond.rows[0].observed, None);
    assert_eq!(beyond.rows[0].p.exceed, 0);
    assert_eq!(beyond.rows[1].observed, Some(0.0));
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

#[test]
fn the_hint_is_the_fewest_permutations_the_check_accepts() {
    // The float estimate ⌈M/alpha⌉ − 1 sits one either side of the
    // boundary for some pairs (alpha 0.3 over three, 0.01 over 73): the
    // hint is stepped onto it, so it is accepted and one fewer is not.
    for alpha in [0.3, 0.2, 0.15, 0.1, 0.07, 0.05, 0.025, 0.01, 0.005, 0.001] {
        for family in 1..400 {
            let least = crate::least_reaching(family, alpha);
            assert!(
                crate::reaches_alpha(least, family, alpha),
                "{alpha} {family}"
            );
            assert!(
                least == 1 || !crate::reaches_alpha(least - 1, family, alpha),
                "{alpha} {family}"
            );
        }
    }
    // And the refusal's hint is what the guard then takes.
    assert_eq!(crate::least_reaching(3, 0.3), 10);
    assert!(crate::check_permutations(10, 0.95, Some(0.3), 3).is_ok());
    let refused = crate::check_permutations(9, 0.95, Some(0.3), 3).unwrap_err();
    assert_eq!(refused.hint(), Some("at least 10 permutations"));
    assert_eq!(crate::least_reaching(73, 0.01), 7299);
}

/// A batch built from two-by-two tables, one per stratum, each
/// `(case_present, case_read, rest_present, rest_read)` and keyed by its
/// index; the charts interleaved (chart `i` is the `37·i mod n`-th
/// drawn) so that no stratum arrives contiguous.
fn stratified_batch(tables: &[(u32, u32, u32, u32)]) -> (Matrix, Design) {
    let mut drawn: Vec<(u32, u16, bool)> = Vec::new();
    for (key, &(case_present, case_read, rest_present, rest_read)) in tables.iter().enumerate() {
        let key = u32::try_from(key).unwrap();
        drawn.extend((0..case_read).map(|i| (key, 1, i < case_present)));
        drawn.extend((0..rest_read).map(|i| (key, 0, i < rest_present)));
    }
    let n = drawn.len();
    assert_ne!(n % 37, 0, "37 must be coprime to the batch");
    let charts: Vec<(u32, u16, bool)> = (0..n).map(|i| drawn[(37 * i) % n]).collect();
    let cells: Vec<Cell> = charts
        .iter()
        .map(|&(_, _, present)| if present { Cell::Present } else { Cell::Absent })
        .collect();
    let mut matrix = Matrix::new(n);
    matrix.push("p", &cells).unwrap();
    let design = Design::new(charts.iter().map(|&(_, group, _)| group).collect())
        .with_strata(charts.iter().map(|&(key, _, _)| key).collect());
    (matrix, design)
}

#[test]
fn the_chi_square_tail_meets_its_table() {
    // scipy.stats.chi2.sf at each point.
    for (x, df, sf) in [
        (3.841_458_820_694_124, 1, 0.05),
        (5.991_464_547_107_979, 2, 0.05),
        (7.814_727_903_251_178, 3, 0.05),
        (9.487_729_036_781_154, 4, 0.05),
        (18.307_038_053_275_146, 10, 0.05),
        (0.5, 7, 0.999_446_481_390_424_9),
        (123.0, 99, 0.051_444_916_997_488_76),
    ] {
        let got = special::chi_square_upper(x, df);
        assert!(
            (got - sf).abs() < 1e-12,
            "chi2({x}, {df}) = {got}, not {sf}"
        );
    }
    assert_eq!(special::chi_square_upper(0.0, 3), 1.0);
}

#[test]
fn the_strata_meet_a_reference_implementation() {
    // Three strata whose odds ratios differ (2.97, 0.20, 1.29). The
    // figures are statsmodels 0.15's `StratifiedTable` over the same
    // tables: the pooled odds ratio and its Robins–Breslow–Greenland
    // interval, the Mantel–Haenszel risk ratio, Breslow–Day with
    // Tarone's correction, and the CMH chi-square, which is the square
    // of this test's statistic when every chart is read.
    let (matrix, design) = stratified_batch(&[(12, 30, 5, 40), (3, 25, 9, 20), (7, 18, 7, 22)]);
    let tested = compare(&matrix, &design, &case_test(1, 999)).unwrap();
    let row = &tested.rows[0];
    let observed = row.observed.unwrap();
    assert!(
        (observed * observed - 0.395_101_645_492_143_3).abs() < 1e-12,
        "{observed}"
    );
    let pooled = row.effect.unwrap().pooled.unwrap();
    let odds = pooled.odds_ratio.unwrap();
    assert!((odds.estimate - 1.229_626_153_605_506_2).abs() < 1e-12);
    assert!(
        (odds.low - 0.626_057_285_298_923_7).abs() < 1e-9,
        "{odds:?}"
    );
    assert!(
        (odds.high - 2.415_083_272_944_817_5).abs() < 1e-9,
        "{odds:?}"
    );
    let risk = pooled.risk_ratio.unwrap();
    assert!((risk.estimate - 1.169_789_498_033_772_9).abs() < 1e-12);
    assert!(risk.low < risk.estimate && risk.estimate < risk.high);
    let homogeneity = pooled.homogeneity_p.unwrap();
    assert!(
        (homogeneity - 0.001_432_938_224_786_850_5).abs() < 1e-9,
        "{homogeneity}"
    );
}

#[test]
fn a_pooled_ratio_removes_what_the_strata_confound() {
    // Simpson's table: within each stratum the cases and the rest share
    // one rate (0.5, then 0.2), so every stratum's ratio is 1; the cases
    // crowd into the low stratum, so the crude ratio is 0.59.
    let (matrix, design) = stratified_batch(&[(5, 10, 20, 40), (8, 40, 2, 10)]);
    let stratified = compare(&matrix, &design, &case_test(4, 999)).unwrap();
    let row = &stratified.rows[0];
    let effect = row.effect.unwrap();
    let pooled = effect.pooled.unwrap();
    assert_eq!(pooled.risk_ratio.unwrap().estimate, 1.0);
    assert_eq!(pooled.odds_ratio.unwrap().estimate, 1.0);
    assert_eq!(pooled.homogeneity_p, Some(1.0));
    assert!((effect.risk_ratio.unwrap().estimate - 13.0 / 22.0).abs() < 1e-12);
    // Every stratum's count is its mean, so the statistic is zero and
    // every permutation reaches it; the crude statistic is not zero.
    assert_eq!(row.observed, Some(0.0));
    assert_eq!(row.p.exceed, 999);
    let crude = compare(
        &matrix,
        &Design::new(design.groups.clone()),
        &case_test(4, 999),
    )
    .unwrap();
    assert!(crude.rows[0].observed.unwrap() > 1.0);
    assert!(crude.rows[0].effect.unwrap().pooled.is_none());
}

#[test]
fn one_stratum_is_no_strata_to_the_bit() {
    // The regression lock: a design whose strata are all one key answers
    // exactly as the design without strata, under either contrast.
    let (matrix, design) = study(83, 29, 7, 13);
    let one = design.clone().with_strata(vec![42; 83]);
    for contrast in [Contrast::CaseVsRest { case: 1 }, Contrast::AnyDifference] {
        let mut test = GroupTest::new(9, 1_999, contrast);
        test.alpha = Some(0.05);
        assert_eq!(
            compare(&matrix, &design, &test).unwrap(),
            compare(&matrix, &one, &test).unwrap(),
            "{contrast:?}"
        );
    }
}

#[test]
fn a_contiguous_layout_draws_the_labels_it_drew_before() {
    // Laying the strata out contiguously must not change which label any
    // chart gets under any permutation.
    let (matrix, design) = study(71, 23, 1, 17);
    let keys: Vec<u32> = (0..71_u32).map(|i| (i * 7 + i / 5) % 6).collect();
    let laid = Contiguous::new(&matrix, &design.groups, &keys);
    let mut order: Vec<usize> = (0..71).collect();
    order.sort_by_key(|&at| keys[at]);
    let scattered = Layout::new(&design.groups, 2, Some(&keys));
    let contiguous = Layout::new(&laid.groups, 2, Some(&laid.keys));
    let (mut before, mut after, mut scratch) = (vec![0; 71], vec![0; 71], Vec::new());
    for k in 0..200 {
        scattered.permute(5, k, &mut before, &mut scratch);
        contiguous.permute(5, k, &mut after, &mut scratch);
        for (to, &from) in order.iter().enumerate() {
            assert_eq!(after[to], before[from], "permutation {k}, chart {from}");
        }
    }
}

#[test]
fn the_stratified_exact_p_meets_an_enumeration() {
    // Test 3 under strata: two strata of six charts, every arrangement of
    // the cases inside each enumerated, against the convolution.
    let (matrix, design) = stratified_batch(&[(3, 3, 1, 3), (1, 2, 3, 4)]);
    let strata: [(u32, u32, u32); 2] = [(6, 4, 3), (6, 4, 2)]; // (charts, present, cases)
    let z_of = |counts: [u32; 2]| {
        let (mut departure, mut variance) = (0.0, 0.0);
        for (&(n, m, c), &a) in strata.iter().zip(&counts) {
            let (mean, spread) = engine::moments(c, m, n).unwrap();
            departure += f64::from(a) - mean;
            variance += spread;
        }
        departure / variance.sqrt()
    };
    // The present charts are the first `m` of each stratum's six; an
    // arrangement is a six-bit mask with `c` bits set.
    let arrangements = |n: u32, c: u32| (0..1_u32 << n).filter(move |mask| mask.count_ones() == c);
    for alternative in [
        Alternative::TwoSided,
        Alternative::Greater,
        Alternative::Less,
    ] {
        let mut test = case_test(23, 20_000);
        test.alternative = alternative;
        test.level = 0.999;
        let row = compare(&matrix, &design, &test).unwrap().rows.remove(0);
        let observed = row.observed.unwrap();
        let (mut reached, mut total) = (0_u32, 0_u32);
        for first in arrangements(6, 3) {
            for second in arrangements(6, 2) {
                let z = z_of([
                    (first & 0b1111).count_ones(),
                    (second & 0b1111).count_ones(),
                ]);
                let stat = match alternative {
                    Alternative::Greater => z,
                    Alternative::Less => -z,
                    Alternative::TwoSided => z.abs(),
                };
                total += 1;
                reached += u32::from(engine::reaches(stat, observed));
            }
        }
        let enumerated = f64::from(reached) / f64::from(total);
        let exact = row.exact.unwrap();
        assert!(
            (exact - enumerated).abs() < 1e-12,
            "{alternative:?}: {exact} against {enumerated}"
        );
        assert!(
            row.p.low <= exact && exact <= row.p.high,
            "{alternative:?}: {:?}",
            row.p
        );
    }
}

/// A self-matched study of one predicate: each subject `(instants,
/// present, at_event)` read at its event and `instants − 1` referents,
/// the predicate holding at `present` of them, the event among them when
/// `at_event`.
fn matched_of(people: &[(u16, u16, bool)]) -> MatchedMatrix {
    let counts: Vec<u16> = people.iter().map(|&(instants, _, _)| instants).collect();
    let mut matrix = MatchedMatrix::new(&counts).unwrap();
    matrix
        .push("p", |i, k| {
            let (_, present, at_event) = people[i];
            let referents = usize::from(present) - usize::from(at_event);
            let holds = if k == 0 { at_event } else { k <= referents };
            if holds { Cell::Present } else { Cell::Absent }
        })
        .unwrap();
    matrix
}

#[test]
fn a_self_matched_null_meets_its_enumeration() {
    // Three subjects, one referent each, exposed (at event, at referent)
    // (1,0), (1,0), (0,1): the 8 labellings are equally likely and a sum
    // of 2 or more is reached by 4, so the exact one-sided p is 0.5.
    let matrix = matched_of(&[(2, 1, true), (2, 1, true), (2, 1, false)]);
    let mut test = EventTest::new(31, 20_000);
    test.alternative = Alternative::Greater;
    test.level = 0.999;
    let row = matched(&matrix, &test).unwrap().rows.remove(0);
    assert!((row.exact.unwrap() - 0.5).abs() < 1e-12, "{:?}", row.exact);
    assert!(row.p.low <= 0.5 && 0.5 <= row.p.high, "{:?}", row.p);
    assert_eq!(row.counts[0].present, 2);
}

#[test]
fn a_self_matched_study_meets_a_reference_implementation() {
    // Eight subjects as strata of one event and its referents; statsmodels
    // 0.15's `StratifiedTable` over the same tables gives the CMH
    // chi-square, which is this statistic squared, and the pooled odds
    // ratio with its Robins–Breslow–Greenland interval, the
    // case-crossover's conditional odds ratio.
    let matrix = matched_of(&[
        (4, 1, true),
        (4, 2, true),
        (5, 1, false),
        (3, 2, true),
        (5, 3, true),
        (4, 1, true),
        (6, 2, false),
        (3, 1, true),
    ]);
    let row = matched(&matrix, &EventTest::new(2, 999))
        .unwrap()
        .rows
        .remove(0);
    let observed = row.observed.unwrap();
    assert!(
        (observed * observed - 4.857_799_671_592_775).abs() < 1e-12,
        "{observed}"
    );
    let pooled = row.effect.unwrap().pooled.unwrap();
    let odds = pooled.odds_ratio.unwrap();
    assert!((odds.estimate - 6.375).abs() < 1e-12);
    assert!(
        (odds.low - 1.165_308_723_912_862_6).abs() < 1e-9,
        "{odds:?}"
    );
    assert!((odds.high - 34.875_414_700_009_53).abs() < 1e-7, "{odds:?}");
    assert!((pooled.risk_ratio.unwrap().estimate - 2.890_109_890_109_89).abs() < 1e-12);
    assert_eq!(pooled.homogeneity_p, None);
}

#[test]
fn a_self_matched_null_holds_its_level() {
    // 300 null studies of 80 subjects, each read at its event and 11
    // referents, a predicate holding at each instant with a chance of
    // the subject's own: the event is just another instant, so the raw p
    // is under 0.05 within its binomial of 5%.
    let mut under = 0_u64;
    let studies = 300;
    for study in 0..studies {
        let mut rng = SplitMix64::new(4_000 + study);
        let people: Vec<(u16, Vec<bool>)> = (0..80)
            .map(|_| {
                let share = (rng.next_u64() >> 11) as f64 / (1_u64 << 53) as f64;
                let cells = (0..12)
                    .map(|_| ((rng.next_u64() >> 11) as f64 / (1_u64 << 53) as f64) < share)
                    .collect();
                (12, cells)
            })
            .collect();
        let counts: Vec<u16> = people.iter().map(|(c, _)| *c).collect();
        let mut matrix = MatchedMatrix::new(&counts).unwrap();
        matrix
            .push("p", |i, k| {
                if people[i].1[k] {
                    Cell::Present
                } else {
                    Cell::Absent
                }
            })
            .unwrap();
        let row = matched(&matrix, &EventTest::new(study, 199))
            .unwrap()
            .rows
            .remove(0);
        under += u64::from(row.p.value <= 0.05);
    }
    // Valid means no more often than the level: under 5% plus three of
    // its binomial standard deviations.
    let ceiling = 0.05 * studies as f64 + 3.0 * (0.05 * 0.95 * studies as f64).sqrt();
    assert!((under as f64) <= ceiling, "{under} of {studies}");
}

#[test]
fn a_self_matched_study_is_the_same_at_every_thread_count() {
    let matrix = matched_of(&[(4, 1, true), (4, 2, true), (5, 1, false), (3, 2, true)]);
    let mut test = EventTest::new(8, 1_001);
    let one = matched(&matrix, &test).unwrap();
    test.parallelism = Parallelism::Threads(NonZeroU16::new(3).unwrap());
    assert_eq!(one, matched(&matrix, &test).unwrap());
    test.after_birth = AfterBirth::RestrictPairings;
    assert_eq!(
        matched(&matrix, &test).unwrap_err().field(),
        Some("afterBirth")
    );
    assert_eq!(
        MatchedMatrix::new(&[3]).unwrap_err().field(),
        Some("subjects")
    );
    assert_eq!(
        MatchedMatrix::new(&[3, 0]).unwrap_err().field(),
        Some("subjects[1]")
    );
}
