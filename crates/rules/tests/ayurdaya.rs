//! BPHS ch. 43's Pindayu, Nisargayu and Amsayu over the 93 recorded charts:
//! what the verses' arithmetic says must hold of every chart, and where the
//! spans fall.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they read"
)]

mod common;

use common::{chart_at, files_in};
use teistro_rules::longevity::{AyurdayaRules, Giver, Method, Nisarga, full_years};
use teistro_rules::{Evaluator, Readings};

#[test]
fn every_span_is_the_sum_of_what_its_givers_give_within_their_bounds() {
    let mut spans: [Vec<f64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS);
        let listed = evaluator.ayurdaya(AyurdayaRules {
            nisarga: Nisarga::Listed,
            ..AyurdayaRules::default()
        });
        assert!(
            (listed.nisargayu.years - 120.0).abs() < 1e-9,
            "{}",
            path.display()
        );
        let reading = evaluator.ayurdaya(AyurdayaRules::default());
        for (at, span) in [reading.pindayu, reading.nisargayu, reading.amsayu]
            .iter()
            .enumerate()
        {
            let sum: f64 = span.contributions.iter().map(|given| given.net).sum();
            assert!((sum - span.years).abs() < 1e-9);
            for given in &span.contributions {
                assert!(
                    given.net >= 0.0 && given.net <= given.basic + 1e-9,
                    "{given:?}"
                );
                match (span.method, given.giver) {
                    // Half at deep debilitation, the whole at deep exaltation.
                    (Method::Pindayu | Method::Nisargayu, Giver::Graha(graha)) => {
                        let full = full_years(span.method, graha).unwrap();
                        assert!(given.basic >= full / 2.0 - 1e-9 && given.basic <= full + 1e-9);
                    }
                    // A year a navamsha, less twelves; a year a sign.
                    (_, _) => assert!(given.basic >= 0.0 && given.basic < 12.0 + 1e-9),
                }
            }
            spans[at].push(span.years);
        }
    }
    let summary: Vec<(f64, f64, f64)> = spans
        .iter()
        .map(|years| {
            let low = years.iter().copied().fold(f64::INFINITY, f64::min);
            let high = years.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let mean = years.iter().sum::<f64>() / 93.0;
            let round = |value: f64| (value * 100.0).round() / 100.0;
            (round(low), round(mean), round(high))
        })
        .collect();
    // Lowest, mean and highest over the 93, in years of 360 days: Pindayu and
    // Nisargayu near their full 127 and 120 less the reductions; Amsayu near
    // the 48 that eight givers of 0 to 12 years average, less the same.
    assert_eq!(
        summary,
        [
            (56.84, 85.01, 109.09),
            (38.43, 78.94, 99.29),
            (20.37, 38.92, 66.14)
        ]
    );
}

/// The chart of *Jataka Parijata*'s worked example, from its printed
/// longitudes (p. 238).
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a longitude under 360 over thirty is a sign index, and over a navamsha's span under 108"
)]
fn parijata_1853() -> teistro_rules::RuleChart {
    use teistro_core::catalogue::{Dignity, Graha, Rashi};
    use teistro_rules::{Body, House, Placement, RuleChart, StrengthMeasure, Strengths};

    let at =
        |sign: u8, degrees: f64, minutes: f64| f64::from(sign) * 30.0 + degrees + minutes / 60.0;
    let lagna = at(0, 14.0, 32.0);
    // The Sun to Saturn, then the nodes the book does not print, then the lagna.
    let longitudes = [
        at(0, 17.0, 43.0),
        at(9, 14.0, 30.0),
        at(11, 27.0, 53.0),
        at(11, 24.0, 14.0),
        at(8, 1.0, 25.0),
        at(0, 14.0, 3.0),
        at(0, 27.0, 56.0),
        at(1, 15.0, 0.0),
        at(7, 15.0, 0.0),
        lagna,
    ];
    let sign_of = |longitude: f64| Rashi::from_id((longitude / 30.0) as u16).unwrap();
    let placements = longitudes.map(|longitude| Placement {
        longitude,
        sign: sign_of(longitude),
        house: House::between(Rashi::Aries, sign_of(longitude)),
        dignity: Dignity::Neutral,
        retrograde: false,
        combust: false,
        karaka7: None,
        karaka8: None,
        navamsha: Rashi::from_id((longitude * 3.0 / 10.0) as u16 % 12).unwrap(),
    });
    let mut chart = RuleChart {
        placements,
        panchanga: None,
        strengths: Some(Strengths {
            measure: StrengthMeasure::Caller,
            of: [
                Some(9.0),
                Some(6.0),
                Some(8.0),
                Some(5.0),
                Some(6.0),
                Some(6.0),
                Some(6.0),
                None,
                None,
                None,
            ],
            required: [None; 10],
        }),
    };
    // "Venus and Saturn are eclipsed" (p. 239).
    for graha in [Graha::Venus, Graha::Saturn] {
        chart.placements[Body::Graha(graha).index()].combust = true;
    }
    chart
}

/// *Jataka Parijata* ch. 5's worked example (Sastri's 1932 translation,
/// pp. 237 to 243): a birth half a ghatika before sunrise on 30 April 1853,
/// the longitudes as printed, read under [`AyurdayaRules::PARIJATA`].
///
/// The book gives the strengths only as "Mars is stronger than Mercury";
/// the Sun is taken stronger than Mars, which makes the lagna's years its
/// navamshas, as the book counts them (C304).
#[test]
fn jataka_parijata_s_worked_example_reads_as_the_book_reduces_it() {
    use teistro_rules::longevity::{Combine, EnemyExempt, Enmity, RisingTakes};

    let chart = parijata_1853();
    let evaluator = Evaluator::new(&chart, Readings::TEXTS);
    let read = evaluator.ayurdaya(AyurdayaRules::PARIJATA);
    let pindayu = &read.pindayu.contributions;

    // The basic years from the printed longitudes. Four reproduce the
    // book's figures; three differ by its own arithmetic: Mars from 120°
    // where the longitudes give 120° 7′, Jupiter from 33° for 33° 35′, and
    // Venus from 162° 45′ for 162° 57′.
    let printed = [
        (18.5923, 1e-3),
        (17.465, 1e-3),
        (12.5, 1e-2),
        (6.31, 5e-3),
        (8.875, 3e-2),
        (19.97, 4e-2),
        (10.44, 1e-3),
    ];
    for (given, (book, tolerance)) in pindayu.iter().zip(printed) {
        assert!(
            (given.basic - book).abs() < tolerance,
            "{given:?} against {book}"
        );
    }
    let [
        sun,
        moon,
        mars,
        mercury,
        jupiter,
        venus,
        saturn,
        lagna_years,
    ] = *pindayu;
    // The visible half: Mars, the stronger of the two in the 12th, loses
    // all and Mercury nothing; the Moon, a benefic in the 10th, a sixth,
    // 14.554 as printed; Jupiter, in the 9th, an eighth.
    assert!(mars.net.abs() < 1e-12);
    assert!(mercury.reductions.visible_half.abs() < 1e-12);
    assert!((moon.net - 14.554).abs() < 1e-3, "{moon:?}");
    assert!((jupiter.net - jupiter.basic * 7.0 / 8.0).abs() < 1e-12);
    // Eclipsed, but Venus and Saturn keep their years.
    assert!(
        venus.reductions.combustion.abs() < 1e-12 && saturn.reductions.combustion.abs() < 1e-12
    );
    // "No planet is quartered in the house of its enemy": Saturn in Aries,
    // Mars in the 12th from him, is neutral by the compound friendship; by
    // natural enmity alone he would lose a third.
    assert!(saturn.reductions.enemy_sign.abs() < 1e-12);
    let natural = evaluator.ayurdaya(AyurdayaRules {
        enmity: Enmity::Natural,
        ..AyurdayaRules::PARIJATA
    });
    assert!(
        (natural.pindayu.contributions[6].reductions.enemy_sign - saturn.basic / 3.0).abs() < 1e-12
    );
    // Saturn and the Sun rise and Jupiter aspects the lagna: every giver
    // loses 4.36/108 of its years, halved (p. 243), the lagna's own excepted.
    let share = 4.36 / 108.0 / 2.0;
    for given in [sun, moon, venus] {
        assert!(
            (given.reductions.rising - given.basic * share).abs() < 1e-9,
            "{given:?}"
        );
    }
    assert!((sun.net - sun.basic * (1.0 - share)).abs() < 1e-9);
    // The lagna gives its navamshas from Aries, 4.36 years.
    assert!((lagna_years.net - 4.36).abs() < 1e-9);

    // Under BPHS's readings, only the rising malefics lose by rising, and
    // Mars in an enemy's sign would be exempt only retrograde.
    let bphs = evaluator.ayurdaya(AyurdayaRules::default());
    assert!(bphs.pindayu.contributions[1].reductions.rising.abs() < 1e-12);
    assert!(bphs.pindayu.contributions[0].reductions.rising > 0.0);
    assert_eq!(
        (
            AyurdayaRules::default().enemy_exempt,
            AyurdayaRules::default().rising,
            AyurdayaRules::PARIJATA.combine
        ),
        (
            EnemyExempt::Retrograde,
            RisingTakes::Malefic,
            Combine::Largest
        )
    );
}
