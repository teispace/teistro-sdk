//! The named schemes and a consumer's own rows (`03-design/varga-schemes.md`,
//! SR-114), held to their sources' worked examples and tables, and to the
//! invariants every shipped row keeps.
//!
//! The acceptance values are the sources' own: Narasimha Rao's 2003
//! handout works Kashinatha's hora for two bodies and spells out the
//! parivritti-dwaya hora's first four signs; his 2012 article spells out the
//! Uma-Shambhu hora's first seven halves and Krishna Mishra's navamshas of
//! Taurus; Iranganti Rangacharya tabulates the parivritti-traya drekkana
//! whole and gives three signs of Somanatha's.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own tables"
)]

mod common;

use common::founded;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Rashi, Varga};
use teistro_core::settings::{VargaChoice, VargaVariant};
use teistro_core::varga_rows::{Classifier, RowGroup, RowMap, RowSpans, Rows, SignBase};
use teistro_vargas::{Axis, SCHEMES, Scheme, VARIANTS, chart, sign, table};

/// A longitude given in a sign's degrees and minutes.
fn at(rashi: Rashi, degrees: u32, minutes: u32) -> Nas {
    Nas::try_from_degrees(
        f64::from(rashi as u8) * 30.0 + f64::from(degrees) + f64::from(minutes) / 60.0,
    )
    .expect("inside the zodiac")
}

/// The signs each part of `rashi` goes to.
fn row(scheme: &Scheme, rashi: Rashi) -> Vec<Rashi> {
    table(scheme)[rashi as usize].clone()
}

fn named(varga: Varga, which: VargaVariant) -> Scheme {
    Scheme::named(varga, which).expect("a scheme of the chart")
}

use Rashi::{
    Aquarius, Aries, Cancer, Capricorn, Gemini, Leo, Libra, Pisces, Sagittarius, Scorpio, Taurus,
    Virgo,
};

#[test]
fn kashinathas_hora_reproduces_the_handouts_worked_examples() {
    let hora = named(Varga::D2, VargaVariant::Kashinatha);
    // "Jupiter is at 25°16' in Taurus ... placed in Libra"; "Mars is at
    // 18°53' in Sagittarius ... placed in Sagittarius" (Narasimha Rao 2003).
    assert_eq!(sign(&hora, at(Taurus, 25, 16)), Libra);
    assert_eq!(sign(&hora, at(Sagittarius, 18, 53)), Sagittarius);
    // The 2012 article's second view: Aries's halves go to Scorpio and Aries.
    assert_eq!(row(&hora, Aries), [Scorpio, Aries]);
    // Every lord's two signs, day-strong for the Sun's half: each sign's
    // hora signs are its lord's (Leo and Cancer a pair).
    for rashi in Rashi::ALL {
        let signs = row(&hora, rashi);
        let lord = rashi.attributes().lord;
        for target in &signs {
            let paired = matches!((rashi, target), (Cancer | Leo, Cancer | Leo));
            assert!(
                paired || target.attributes().lord == lord,
                "{rashi:?} goes to {target:?}, not its lord's"
            );
        }
        assert_ne!(signs[0], signs[1], "{rashi:?}'s halves part");
    }
}

#[test]
fn the_parivritti_hora_goes_twice_round_the_zodiac() {
    let hora = named(Varga::D2, VargaVariant::Parivritti);
    // Narasimha Rao 2003: "the two parts of Aries go into Aries and Taurus;
    // ... Taurus into Gemini and Cancer; ... Gemini into Leo and Virgo; ...
    // Cancer into Libra and Scorpio".
    assert_eq!(row(&hora, Aries), [Aries, Taurus]);
    assert_eq!(row(&hora, Taurus), [Gemini, Cancer]);
    assert_eq!(row(&hora, Gemini), [Leo, Virgo]);
    assert_eq!(row(&hora, Cancer), [Libra, Scorpio]);
}

#[test]
fn the_uma_shambhu_hora_reverses_an_even_signs_halves() {
    let hora = named(Varga::D2, VargaVariant::UmaShambhu);
    // Narasimha Rao 2012: Ar's halves Ar and Ta; Ta's second half Ge and
    // first Cn; Ge's Le and Vi; Cn's second half Li.
    assert_eq!(row(&hora, Aries), [Aries, Taurus]);
    assert_eq!(row(&hora, Taurus), [Cancer, Gemini]);
    assert_eq!(row(&hora, Gemini), [Leo, Virgo]);
    assert_eq!(row(&hora, Cancer)[1], Libra);
    // The Sun's halves fall in odd signs and the Moon's in even, which is
    // what the article's reading is for.
    for rashi in Rashi::ALL {
        let signs = row(&hora, rashi);
        let sun_half = if rashi as u8 % 2 == 0 { 0 } else { 1 };
        assert_eq!(signs[sun_half] as u8 % 2, 0, "{rashi:?}'s Sun half is odd");
        assert_eq!(signs[1 - sun_half] as u8 % 2, 1, "{rashi:?}'s Moon half is even");
    }
}

#[test]
fn the_parivritti_drekkana_is_the_tabulated_one() {
    // Iranganti Rangacharya's table, sign by sign.
    let expected = [
        [Aries, Taurus, Gemini],
        [Cancer, Leo, Virgo],
        [Libra, Scorpio, Sagittarius],
        [Capricorn, Aquarius, Pisces],
    ];
    let drekkana = named(Varga::D3, VargaVariant::Parivritti);
    for rashi in Rashi::ALL {
        assert_eq!(row(&drekkana, rashi), expected[rashi as usize % 4], "{rashi:?}");
    }
}

#[test]
fn somanathas_drekkana_runs_on_from_aries_and_back_from_pisces() {
    let drekkana = named(Varga::D3, VargaVariant::Somanatha);
    // The examples Iranganti Rangacharya gives.
    assert_eq!(row(&drekkana, Aries), [Aries, Taurus, Gemini]);
    assert_eq!(row(&drekkana, Taurus), [Pisces, Aquarius, Capricorn]);
    assert_eq!(row(&drekkana, Cancer), [Sagittarius, Scorpio, Libra]);
    // And the rule itself: the odd signs' eighteen drekkanas run on round
    // the zodiac one and a half times, the even signs' back.
    let odd: Vec<Rashi> = Rashi::ALL.iter().step_by(2).flat_map(|r| row(&drekkana, *r)).collect();
    let even: Vec<Rashi> = Rashi::ALL[1..].iter().step_by(2).flat_map(|r| row(&drekkana, *r)).collect();
    for (index, (on, back)) in odd.iter().zip(&even).enumerate() {
        let index = u16::try_from(index).unwrap();
        assert_eq!(*on, Rashi::from_id(index % 12).unwrap());
        assert_eq!(*back, Rashi::from_id(11 - index % 12).unwrap());
    }
}

#[test]
fn krishna_mishras_navamsha_reverses_an_even_sign() {
    let navamsha = named(Varga::D9, VargaVariant::KrishnaMishra);
    // Narasimha Rao 2012: Taurus's nine run Vi, Le, Cn, Ge, Ta, Ar, Pi, Aq, Cp.
    assert_eq!(
        row(&navamsha, Taurus),
        [Virgo, Leo, Cancer, Gemini, Taurus, Aries, Pisces, Aquarius, Capricorn]
    );
    // An odd sign's are Parashara's.
    assert_eq!(row(&navamsha, Leo), row(&Scheme::of(Varga::D9), Leo));
    // An even sign's are Parashara's reversed.
    let mut reversed = row(&Scheme::of(Varga::D9), Scorpio);
    reversed.reverse();
    assert_eq!(row(&navamsha, Scorpio), reversed);
}

#[test]
fn a_cyclic_variant_covers_the_signs_evenly() {
    for varga in [Varga::D2, Varga::D3, Varga::D4, Varga::D5] {
        let scheme = named(varga, VargaVariant::Parivritti);
        let mut seen = [0_usize; 12];
        for cells in table(&scheme) {
            for cell in cells {
                seen[cell as usize] += 1;
            }
        }
        let each = usize::from(scheme.divisions);
        assert!(seen.iter().all(|count| *count == each), "{varga:?} {seen:?}");
    }
}

#[test]
fn every_variant_has_a_row_and_every_row_a_variant() {
    for varga in Varga::ALL {
        for which in VargaVariant::ALL {
            let rows = VARIANTS
                .iter()
                .chain(SCHEMES.iter())
                .filter(|row| row.varga == Some(varga) && row.variant == *which)
                .count();
            assert_eq!(rows, usize::from(which.serves(varga)), "{varga:?} {which:?}");
        }
    }
    for scheme in VARIANTS {
        let varga = scheme.varga.expect("a catalogued chart");
        assert_eq!(scheme.divisions, varga.attributes().divisions, "{varga:?}");
        // Every variant passes the check a consumer's rows pass.
        scheme
            .rows()
            .check(scheme.divisions, "rows")
            .unwrap_or_else(|error| panic!("{varga:?} {:?}: {error}", scheme.variant));
        // And differs from the chart's Parashara row somewhere.
        assert_ne!(table(&scheme), table(&Scheme::of(varga)), "{varga:?} {:?}", scheme.variant);
    }
}

#[test]
fn a_written_scheme_reads_back_whatever_computed_it() {
    // BPHS ch. 6's writes as it always has: no variant.
    let d9 = serde_json::to_value(Scheme::of(Varga::D9)).unwrap();
    assert!(d9.get("variant").is_none(), "{d9}");
    for scheme in VARIANTS {
        let written = serde_json::to_value(scheme).unwrap();
        assert_eq!(written["variant"], scheme.variant.key());
        let read: Scheme = serde_json::from_value(written).unwrap();
        assert_eq!(read, scheme);
    }
    // A consumer's rows read back as the rows, not as any shipped row.
    let rows = Rows {
        classifier: Classifier::Parity,
        groups: vec![
            RowGroup {
                spans: RowSpans::Degrees {
                    widths: vec![10, 20],
                },
                map: RowMap::Listed { signs: vec![4, 3] },
            },
            RowGroup {
                spans: RowSpans::Equal,
                map: RowMap::Step {
                    base: SignBase::Same,
                    step: 6,
                    offset: 1,
                },
            },
        ],
    };
    let own = Scheme::from_rows(Varga::D2, &rows).unwrap();
    let written = serde_json::to_value(own).unwrap();
    assert_eq!(written["variant"], "CUSTOM");
    let read: Scheme = serde_json::from_value(written.clone()).unwrap();
    assert_eq!(read, own);
    assert_eq!(read.rows(), rows);
    // A written rule broken after the fact is refused, not read as D2's.
    let mut broken = written;
    broken["groups"][0]["map"]["signs"][1] = serde_json::json!(13);
    let refused = serde_json::from_value::<Scheme>(broken).unwrap_err();
    assert!(refused.to_string().contains("not 13"), "{refused}");
    // As is a named scheme the chart does not have.
    let mut stray = serde_json::to_value(Scheme::of(Varga::D9)).unwrap();
    stray["variant"] = serde_json::json!("SOMANATHA");
    assert!(serde_json::from_value::<Scheme>(stray).is_err());
}

#[test]
fn a_chosen_scheme_moves_a_founded_chart_and_nothing_else() {
    let foundation = founded();
    let mut settings = teistro_chart::sample::resolved().settings;
    let shipped = chart(&foundation, Axis::chosen(Varga::D3, &settings).unwrap()).unwrap();
    assert_eq!(shipped, chart(&foundation, Axis::of(Varga::D3)).unwrap());
    settings.vargas.schemes.insert(
        Varga::D3,
        VargaChoice::Named {
            which: VargaVariant::Somanatha,
        },
    );
    let chosen = chart(&foundation, Axis::chosen(Varga::D3, &settings).unwrap()).unwrap();
    assert_eq!(chosen.axis.grahas.variant, VargaVariant::Somanatha);
    let somanatha = named(Varga::D3, VargaVariant::Somanatha);
    for placement in &chosen.grahas {
        let shipped_at = shipped.graha(placement.graha).unwrap();
        // The same body in the same part of the same sign, sent elsewhere
        // only by the rule.
        assert_eq!((placement.at.rashi, placement.at.part), (shipped_at.rashi, shipped_at.part));
        assert_eq!(placement.at.sign, table(&somanatha)[placement.at.rashi as usize][usize::from(placement.at.part)]);
    }
    // Another chart is untouched.
    assert_eq!(Scheme::chosen(Varga::D9, &settings).unwrap(), Scheme::of(Varga::D9));
}
