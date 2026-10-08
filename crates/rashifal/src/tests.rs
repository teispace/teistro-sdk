//! The rashifal against the gochar it is built from and the baseline's
//! formula, held to hand-computed cases.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    reason = "tests fail by panicking, index the twelve and compare exact sums"
)]

use teistro_core::catalogue::{Graha, Rashi, Tithi, Yoga};
use teistro_core::quantity::JulianDay;
use teistro_gochar::hits::{Hit, HitEvent, Motion};
use teistro_gochar::sade_sati::Phase;
use teistro_gochar::{Reference, Transit, Verdict, gochar};

use crate::baseline::{LifeArea, Panchanga, Period, baseline_score};
use crate::{PeriodEvent, RashifalRules, rashifal, reference_day};

fn rules() -> RashifalRules {
    RashifalRules::default()
}

/// A sky that can happen: the nodes opposite, Rahu in Libra, the rest
/// placed by `at`, Aries otherwise.
fn sky(at: &[(Graha, Rashi)]) -> [Transit; 9] {
    let mut transits = [Transit::new(Rashi::Aries, 15.0); 9];
    transits[Graha::Rahu as usize] = Transit::new(Rashi::Libra, 15.0);
    transits[Graha::Ketu as usize] = Transit::new(Rashi::Aries, 15.0);
    for &(graha, sign) in at {
        transits[graha as usize] = Transit::new(sign, 15.0);
    }
    transits
}

fn ingress(graha: Graha, into: Rashi) -> PeriodEvent {
    PeriodEvent {
        hit: Hit {
            instant: JulianDay::literal(2_460_676.5),
            graha,
            event: HitEvent::SignIngress {
                into,
                motion: Motion::Direct,
            },
        },
        sign: into,
    }
}

const QUIET: Panchanga = Panchanga {
    tithi: Tithi::KrishnaDvitiya,
    yoga: Yoga::Vajra,
    muhurta_yogas: 0,
};

#[test]
fn each_sign_reads_the_gochar_from_itself() {
    let transits = sky(&[
        (Graha::Saturn, Rashi::Pisces),
        (Graha::Jupiter, Rashi::Gemini),
    ]);
    let rules = rules();
    for read in rashifal(&transits, &[], &rules) {
        assert_eq!(
            read.gochar,
            gochar(Reference::moon(read.rashi), &transits, rules.gochar)
        );
    }
}

#[test]
fn saturn_reads_its_phase_and_its_spells() {
    let read = rashifal(&sky(&[(Graha::Saturn, Rashi::Pisces)]), &[], &rules());
    let by = |rashi: Rashi| read[rashi as usize].saturn;
    assert_eq!(by(Rashi::Aries).sade_sati, Some(Phase::Rising));
    assert_eq!(by(Rashi::Pisces).sade_sati, Some(Phase::Peak));
    assert_eq!(by(Rashi::Aquarius).sade_sati, Some(Phase::Setting));
    // Pisces is the 4th from Sagittarius and the 8th from Leo.
    assert!(by(Rashi::Sagittarius).spell && by(Rashi::Leo).spell);
    assert!(!by(Rashi::Libra).spell && by(Rashi::Libra).sade_sati.is_none());
    // A spell is the rules' to name (C149).
    let seventh = RashifalRules {
        spells: vec![7],
        ..rules()
    };
    let read = rashifal(&sky(&[(Graha::Saturn, Rashi::Pisces)]), &[], &seventh);
    assert!(read[Rashi::Virgo as usize].saturn.spell);
    assert!(!read[Rashi::Leo as usize].saturn.spell);
}

#[test]
fn an_event_is_counted_from_every_sign() {
    let events = [ingress(Graha::Jupiter, Rashi::Cancer)];
    let read = rashifal(&sky(&[]), &events, &rules());
    let houses: Vec<u8> = read.iter().map(|one| one.events[0].house).collect();
    assert_eq!(houses, [4, 3, 2, 1, 12, 11, 10, 9, 8, 7, 6, 5]);
    // Jupiter is good in the 2nd, 5th, 7th, 9th and 11th (v. 2).
    let good: Vec<u8> = read
        .iter()
        .filter(|one| one.events[0].good_house)
        .map(|one| one.events[0].house)
        .collect();
    assert_eq!(good, [2, 11, 9, 7, 5]);
}

#[test]
fn the_reference_day_is_the_middle_one() {
    for (days, offset) in [
        (1, 0),
        (7, 3),
        (28, 13),
        (29, 14),
        (30, 14),
        (31, 15),
        (365, 182),
        (366, 182),
    ] {
        assert_eq!(reference_day(days), offset, "{days} days");
    }
}

#[test]
fn the_baseline_scores_an_all_good_day_by_hand() {
    // From Aries: the Sun, Mars and Saturn in the 3rd, the Moon in the
    // 1st, Mercury in the 2nd, Jupiter in the 2nd, Venus in the 1st, Rahu
    // in the 3rd with Ketu opposite in the 9th.
    let transits = sky(&[
        (Graha::Sun, Rashi::Gemini),
        (Graha::Mars, Rashi::Gemini),
        (Graha::Saturn, Rashi::Gemini),
        (Graha::Moon, Rashi::Aries),
        (Graha::Mercury, Rashi::Taurus),
        (Graha::Jupiter, Rashi::Taurus),
        (Graha::Venus, Rashi::Aries),
        (Graha::Rahu, Rashi::Gemini),
        (Graha::Ketu, Rashi::Sagittarius),
    ]);
    let read = &rashifal(&transits, &[], &rules())[0];
    let verdicts: Vec<Verdict> = read.gochar.grahas.iter().map(|one| one.verdict).collect();
    // Ketu in the 9th is not good; every other transit is good and
    // nothing stands in its vedha house but where the verses exempt it.
    assert_eq!(verdicts[8], Verdict::NotGood);
    let mut retrograde = [false; 9];
    retrograde[7] = true;
    retrograde[8] = true;
    let score = baseline_score(read, &retrograde, QUIET, Period::Daily);
    // Re-derived from the formula: each verdict's contribution, a quiet
    // panchanga's -2 (bhadra) + 0 (Vajra), no events, no Saturn penalty.
    let raw: f64 = read
        .gochar
        .grahas
        .iter()
        .zip(retrograde)
        .map(|(one, back)| {
            let w = [8.0, 5.0, 8.0, 6.0, 20.0, 7.0, 20.0, 12.0, 10.0][one.graha as usize];
            match one.verdict {
                Verdict::Good => w * if back { 0.75 } else { 1.0 },
                Verdict::Obstructed => w * 0.15,
                Verdict::NotGood => -w * 0.6 * if back { 1.25 } else { 1.0 },
            }
        })
        .sum();
    let expected = ((raw - 2.0 + 82.6) / 188.6 * 100.0 + 0.5).floor();
    assert_eq!(f64::from(score.overall), expected);
    // Every graha of weight 8 or more is named, and Mars, tied with the
    // Sun, is the one the cut drops.
    let named: Vec<Graha> = score.key_influences.iter().map(|one| one.graha).collect();
    assert_eq!(
        named,
        [
            Graha::Jupiter,
            Graha::Saturn,
            Graha::Rahu,
            Graha::Ketu,
            Graha::Sun
        ]
    );
}

#[test]
fn the_baseline_counts_the_first_and_fourth_twice_in_the_whole() {
    // Everything in Aries, the 1st from Aries, but the nodes.
    let read = &rashifal(&sky(&[]), &[], &rules())[0];
    let score = baseline_score(read, &[false; 9], QUIET, Period::Yearly);
    let area = |which: LifeArea| score.areas.iter().find(|(one, _)| *one == which).unwrap().1;
    // Finance (the 2nd and the 11th) holds no transit and takes the whole.
    assert_eq!(area(LifeArea::Finance), score.overall);
    // The overall area is fed by the 1st twice and differs from the score.
    assert_ne!(area(LifeArea::Overall), score.overall);
}

#[test]
fn sade_sati_at_its_peak_lowers_the_score() {
    let calm = &rashifal(&sky(&[(Graha::Saturn, Rashi::Leo)]), &[], &rules())[0];
    let peak = &rashifal(&sky(&[(Graha::Saturn, Rashi::Aries)]), &[], &rules())[0];
    let score = |read| baseline_score(read, &[false; 9], QUIET, Period::Daily).overall;
    assert!(score(peak) < score(calm));
}

#[test]
fn the_baselines_tables_depart_where_its_report_says() {
    use crate::baseline::baseline_gochar;
    let rules = RashifalRules::default();
    let from_aries = |at: &[(Graha, Rashi)]| {
        let text = rashifal(&sky(at), &[], &rules)[0].gochar.clone();
        let baseline = baseline_gochar(&text);
        (text, baseline)
    };
    let verdict = |reading: &teistro_gochar::GocharReading, graha: Graha| {
        reading.grahas[graha as usize].verdict
    };
    // D1: Venus in the 11th; Mars in the 3rd obstructs it by the text,
    // in the 6th by the baseline.
    let (text, baseline) = from_aries(&[
        (Graha::Venus, Rashi::Aquarius),
        (Graha::Mars, Rashi::Gemini),
    ]);
    assert_eq!(verdict(&text, Graha::Venus), Verdict::Obstructed);
    assert_eq!(verdict(&baseline, Graha::Venus), Verdict::Good);
    // D3: the Sun in the 11th and Saturn in its vedha 5th; the text spares
    // father and son, the baseline does not.
    let (text, baseline) = from_aries(&[
        (Graha::Sun, Rashi::Aquarius),
        (Graha::Saturn, Rashi::Leo),
        (Graha::Moon, Rashi::Taurus),
        (Graha::Mercury, Rashi::Taurus),
        (Graha::Venus, Rashi::Taurus),
        (Graha::Mars, Rashi::Taurus),
        (Graha::Jupiter, Rashi::Taurus),
    ]);
    assert_eq!(verdict(&text, Graha::Sun), Verdict::Good);
    assert_eq!(verdict(&baseline, Graha::Sun), Verdict::Obstructed);
    // D2: Rahu in the 3rd; by the Sun's vedha the 9th obstructs it, by the
    // baseline's the 12th.
    let (text, baseline) = from_aries(&[
        (Graha::Rahu, Rashi::Gemini),
        (Graha::Ketu, Rashi::Sagittarius),
        (Graha::Mars, Rashi::Pisces),
    ]);
    assert_eq!(text.grahas[Graha::Rahu as usize].vedha_house, Some(9));
    assert_eq!(baseline.grahas[Graha::Rahu as usize].vedha_house, Some(12));
    assert_eq!(verdict(&baseline, Graha::Rahu), Verdict::Obstructed);
    // The houses, and so the good houses, are the text's.
    for (ours, theirs) in text.grahas.iter().zip(&baseline.grahas) {
        assert_eq!(
            (ours.house, ours.good_house),
            (theirs.house, theirs.good_house)
        );
    }
}
