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
use teistro_core::catalogue::Graha;
use teistro_rules::longevity::{
    AyurdayaRules, Giver, JEEVASARMAN_YEARS, Method, Nisarga, full_years,
};
use teistro_rules::{Body, Evaluator, Readings};

#[test]
fn every_span_is_the_sum_of_what_its_givers_give_within_their_bounds() {
    let mut spans: [Vec<f64>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
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
        // Jeevasarman's span is Pindayu's worked with a seventh of 120 years
        // and 5 days for every graha: the same share of its full years, and
        // the same lagna.
        for (pinda, jeeva) in reading
            .pindayu
            .contributions
            .iter()
            .zip(&reading.jeevasarman.contributions)
        {
            let share = match pinda.giver {
                Giver::Graha(graha) => full_years(Method::Pindayu, graha).unwrap(),
                Giver::Lagna => JEEVASARMAN_YEARS,
            };
            assert!((pinda.basic / share - jeeva.basic / JEEVASARMAN_YEARS).abs() < 1e-12);
        }
        for (at, span) in [
            reading.pindayu,
            reading.nisargayu,
            reading.amsayu,
            reading.jeevasarman,
        ]
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
                    (
                        Method::Pindayu | Method::Nisargayu | Method::Jeevasarman,
                        Giver::Graha(graha),
                    ) => {
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
    // the 48 that eight givers of 0 to 12 years average, less the same; and
    // Jeevasarman's near its full 120 and 5 days, less them.
    assert_eq!(
        summary,
        [
            (56.84, 85.01, 109.09),
            (38.43, 78.94, 99.29),
            (20.37, 38.92, 66.14),
            (52.03, 80.1, 101.44)
        ]
    );
}

/// A printed longitude: signs, degrees, minutes and seconds.
fn at(sign: u8, degrees: f64, minutes: f64, seconds: f64) -> f64 {
    f64::from(sign) * 30.0 + degrees + minutes / 60.0 + seconds / 3600.0
}

/// A chart from a book's printed longitudes, the Sun to Saturn, the nodes
/// and the lagna, with every graha direct, unburnt and of neutral dignity.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a longitude under 360 over thirty is a sign index, and over a navamsha's span under 108"
)]
fn printed_chart(longitudes: [f64; 10]) -> teistro_rules::RuleChart {
    use teistro_core::catalogue::{Dignity, Rashi};
    use teistro_rules::{House, Placement, RuleChart};

    let sign_of = |longitude: f64| Rashi::from_id((longitude / 30.0) as u16).unwrap();
    let lagna = sign_of(longitudes[9]);
    let placements = longitudes.map(|longitude| Placement {
        longitude,
        sign: sign_of(longitude),
        house: House::between(lagna, sign_of(longitude)),
        dignity: Dignity::Neutral,
        retrograde: false,
        combust: false,
        karaka7: None,
        karaka8: None,
        navamsha: Rashi::from_id((longitude * 3.0 / 10.0) as u16 % 12).unwrap(),
    });
    RuleChart {
        placements,
        panchanga: None,
        strengths: None,
    }
}

/// The chart of *Jataka Parijata*'s worked example, from its printed
/// longitudes (p. 238).
fn parijata_1853() -> teistro_rules::RuleChart {
    use teistro_core::catalogue::Graha;
    use teistro_rules::{Body, StrengthMeasure, Strengths};

    // The Sun to Saturn, then the nodes the book does not print, then the lagna.
    let mut chart = printed_chart([
        at(0, 17.0, 43.0, 0.0),
        at(9, 14.0, 30.0, 0.0),
        at(11, 27.0, 53.0, 0.0),
        at(11, 24.0, 14.0, 0.0),
        at(8, 1.0, 25.0, 0.0),
        at(0, 14.0, 3.0, 0.0),
        at(0, 27.0, 56.0, 0.0),
        at(1, 15.0, 0.0, 0.0),
        at(7, 15.0, 0.0, 0.0),
        at(0, 14.0, 32.0, 0.0),
    ]);
    chart.strengths = Some(Strengths {
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
    });
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

/// The translator's figure of the rays (*Jataka Parijata* ch. 5, notes to
/// v. 22, pp. 251 to 254), "a distinguished personage": Jupiter retrograde,
/// Saturn eclipsed.
///
/// The book's table prints Jupiter at 6s 25° 13′ 23″ and its working at
/// 6s 25° 43′ 23″; the working's is taken, which its rays follow. Its Mars
/// multiplies .132537 / 6 × 5 as .1103475 for .110448, so his rays are the
/// product, .2209 for the printed .2207, and the total 31.3234 for 31.3232.
#[test]
fn the_rays_reproduce_the_translator_s_figure_to_its_last_place() {
    use teistro_core::catalogue::Graha;
    use teistro_rules::Body;
    use teistro_rules::longevity::{Facing, RasmiRules};
    use teistro_rules::rule::LifeClass;

    let mut chart = printed_chart([
        at(1, 2.0, 55.0, 30.0),
        at(11, 23.0, 35.0, 24.0),
        at(3, 24.0, 1.0, 26.0),
        at(0, 13.0, 10.0, 48.0),
        at(6, 25.0, 43.0, 23.0),
        at(2, 18.0, 15.0, 50.0),
        at(0, 17.0, 59.0, 38.0),
        // Rahu with Venus and Ketu opposite, as the chakra draws them.
        at(2, 15.0, 0.0, 0.0),
        at(8, 15.0, 0.0, 0.0),
        at(7, 15.0, 47.0, 24.0),
    ]);
    chart.placements[Body::Graha(Graha::Jupiter).index()].retrograde = true;
    chart.placements[Body::Graha(Graha::Saturn).index()].combust = true;
    let rasmi = Evaluator::new(&chart, Readings::TEXTS).rasmi(RasmiRules::default());

    let printed = [
        (8.7264, Facing::Away, false, 0.0),
        (6.5902, Facing::Towards, false, 1.0 / 16.0),
        (0.2209, Facing::Away, true, 0.0),
        (1.5655, Facing::Towards, true, 0.0),
        (5.3882, Facing::Away, true, 0.0),
        (8.7765, Facing::Away, true, 0.0),
        (0.0557, Facing::Away, false, 0.0),
    ];
    for (graha, (rays, facing, doubled, share)) in rasmi.grahas.iter().zip(printed) {
        // The book truncates to its fourth place.
        assert!((graha.rays - rays).abs() < 1e-4, "{graha:?} against {rays}");
        assert_eq!(
            (
                graha.facing,
                graha.doubled,
                graha.enemy_share,
                graha.eclipsed
            ),
            (facing, doubled, share, false),
            "{graha:?}"
        );
    }
    // Long by Jatakadesa's bands.
    assert!((rasmi.total - 31.3234).abs() < 1e-4, "{}", rasmi.total);
    assert_eq!(rasmi.class, LifeClass::Long);

    // v. 23's years keep half at debilitation: the Sun's 9.363 (p. 254),
    // under the same doublings as his rays.
    let sun = rasmi.grahas[0];
    assert!((sun.basic_years - 9.363).abs() < 1e-3 && (sun.years - sun.basic_years).abs() < 1e-12);
    for graha in rasmi.grahas {
        assert!((graha.years / graha.basic_years - graha.rays / graha.basic).abs() < 1e-9);
    }
    // By the sign, Mercury in Aries is only in a friend's (Mars, neutral by
    // nature and in the fourth from him): the doubling was the dwadasamsa's.
    let by_sign = Evaluator::new(&chart, Readings::TEXTS).rasmi(RasmiRules::VERSE);
    assert!(!by_sign.grahas[3].doubled);
}

/// Jeevasarman's full years are the note's "17 years, 1 month, 22 days, 8
/// ghatikas and 34.3 vighatikas" (*Jataka Parijata* ch. 5 v. 17, p. 247),
/// a seventh of 120 years and 5 days of 360.
#[test]
fn jeevasarman_gives_a_seventh_of_120_years_and_5_days() {
    let printed = 17.0 + 1.0 / 12.0 + (22.0 + (8.0 + 34.3 / 60.0) / 60.0) / 360.0;
    assert!((JEEVASARMAN_YEARS - printed).abs() < 1e-7);
    assert_eq!(full_years(Method::Jeevasarman, Graha::Rahu), None);
}

/// Balabhadra's reductions leave a whole, two thirds or a half of the same
/// basic years v. 46 starts from, and the note's 7 over 27 counts nakshatra
/// years, which the solar sum converts by 324 over 365.
#[test]
fn balabhadra_takes_only_the_greatest_reduction_from_the_same_years() {
    use teistro_core::settings::{Ekadhipatya, MoonBinduFromJupiter};
    use teistro_rules::longevity::{AshtakaReductions, AshtakavargaAyusRules, Bindus, Divisor};
    use teistro_strength::ashtakavarga::{AshtakavargaChart, bindus};

    let seven = [
        Graha::Sun,
        Graha::Moon,
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
    ];
    let mut kept = [0_u32; 3];
    for (path, file) in files_in("doshas") {
        let chart = chart_at(&path, &file["inputs"]);
        let evaluator = Evaluator::new(&chart, Readings::TEXTS);
        let sign = |body: Body| chart.placement(body).sign;
        let ashtakavarga = AshtakavargaChart {
            lagna: sign(Body::Lagna),
            signs: seven.map(|graha| sign(Body::Graha(graha))),
        };
        let raw = Bindus {
            grahas: bindus(&ashtakavarga, MoonBinduFromJupiter::Twelfth),
            ekadhipatya: Ekadhipatya::Bphs,
        };
        let verse = evaluator.ashtakavarga_ayus(&raw, AshtakavargaAyusRules::default());
        let note = AshtakavargaAyusRules {
            divisor: Divisor::SevenOverTwentySeven,
            reductions: AshtakaReductions::Balabhadra,
            ..AshtakavargaAyusRules::default()
        };
        let balabhadra = evaluator.ashtakavarga_ayus(&raw, note);
        let thirty = evaluator.ashtakavarga_ayus(
            &raw,
            AshtakavargaAyusRules {
                reductions: AshtakaReductions::Balabhadra,
                ..AshtakavargaAyusRules::default()
            },
        );
        for (with_verse, with_note) in verse.grahas.iter().zip(&thirty.grahas) {
            assert_eq!(with_verse.basic, with_note.basic, "{}", path.display());
            let at = [1.0, 2.0 / 3.0, 0.5]
                .iter()
                .position(|kept| (with_note.factor - kept).abs() < 1e-12);
            assert!(at.is_some(), "{}: {with_note:?}", path.display());
            if let Some(count) = at.and_then(|at| kept.get_mut(at)) {
                *count += 1;
            }
        }
        assert!((verse.bhinna_solar * 365.0 - verse.bhinna * 360.0).abs() < 1e-9);
        assert!((balabhadra.bhinna_solar * 365.0 - balabhadra.bhinna * 324.0).abs() < 1e-9);
    }
    // Over the 93 charts' 651 grahas, how many keep all, two thirds and
    // half: two in three are halved, most for company in their sign, which
    // Mercury and Venus beside the Sun nearly always have.
    assert_eq!(kept, [99, 121, 431]);
}
