//! The varga schemes and karaka namings a context's settings choose, through
//! the façade (`docs/03-design/varga-schemes.md`): a named variant moves
//! the chart it is chosen for and nothing else, a consumer's own rows are
//! read like the shipped ones, a broken row is refused by its field, and
//! the seven-karaka namings reach the rules.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they found"
)]

use teistro::catalogue::{CharaKaraka, Graha, Rashi, Varga};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::settings::VargaVariant;
use teistro::{ChartRequest, Context, Document, Ephemeris, RuleInputs, UtcOffset, VargaChart};

/// A Kathmandu birth, 14 April 1990, the corpus's first.
const BIRTH: f64 = 2_447_995.489_583_333_5;

/// Kashinatha's hora: each sign's first and second half (Narasimha Rao,
/// *Personal Finances from Hora Chart*, 2003), as sign indices.
const KASHINATHA: [[usize; 2]; 12] = [
    [7, 0],
    [1, 6],
    [5, 2],
    [3, 4],
    [4, 3],
    [2, 5],
    [6, 1],
    [0, 7],
    [11, 8],
    [9, 10],
    [10, 9],
    [8, 11],
];

fn context(patch: &str) -> Result<Context, teistro::Error> {
    Context::builder()
        .settings_json(patch)
        .ephemeris([Ephemeris::Builtin])
        .build()
}

fn chart(sdk: &Context, at: f64, vargas: &[Varga]) -> Document {
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let request = ChartRequest::at(place, UtcOffset::literal(5, 45, 0))
        .with_state()
        .with_vargas(vargas.iter().copied());
    sdk.chart()
        .reading(JulianDay::<Utc>::literal(at), &request)
        .unwrap()
        .value
}

fn varga(document: &Document, which: Varga) -> &VargaChart {
    document
        .vargas
        .iter()
        .find(|one| one.axis.grahas.varga == Some(which))
        .unwrap()
}

#[test]
fn a_named_hora_places_every_graha_by_its_table_and_moves_no_other_chart() {
    let shipped = context("{}").unwrap();
    let chosen =
        context(r#"{"vargas": {"schemes": {"D2": {"kind": "NAMED", "which": "KASHINATHA"}}}}"#)
            .unwrap();
    let mut moved = 0;
    for step in 0..24 {
        let at = BIRTH + f64::from(step) * 1.7;
        let ours = chart(&chosen, at, &[Varga::D2, Varga::D9]);
        let theirs = chart(&shipped, at, &[Varga::D2, Varga::D9]);
        let hora = varga(&ours, Varga::D2);
        assert_eq!(hora.axis.grahas.variant, VargaVariant::Kashinatha);
        for placed in &hora.grahas {
            let at = placed.at;
            let expected = KASHINATHA[at.rashi as usize][usize::from(at.part)];
            assert_eq!(at.sign, Rashi::ALL[expected], "{placed:?}");
        }
        // The navamsha is the shipped one, byte for byte.
        assert_eq!(varga(&ours, Varga::D9), varga(&theirs, Varga::D9));
        if hora != varga(&theirs, Varga::D2) {
            moved += 1;
        }
    }
    assert!(moved > 0, "Kashinatha's hora never parted from Parashara's");
}

#[test]
fn a_chosen_navamsha_reaches_the_rules() {
    let chosen =
        context(r#"{"vargas": {"schemes": {"D9": {"kind": "NAMED", "which": "KRISHNA_MISHRA"}}}}"#)
            .unwrap();
    let document = chart(&chosen, BIRTH, &[Varga::D9]);
    let d9 = varga(&document, Varga::D9);
    let ruled = RuleInputs::of(&document)
        .unwrap()
        .under(chosen.settings())
        .unwrap();
    for placed in &d9.grahas {
        let body = teistro::rules::Body::Graha(placed.graha);
        assert_eq!(ruled.chart.placement(body).navamsha, placed.at.sign);
    }
}

#[test]
fn a_consumers_rows_read_like_the_shipped_ones() {
    // Parashara's navamsha written as rows: the same chart as the shipped one.
    let own = r#"{"vargas": {"schemes": {"D9": {"kind": "CUSTOM", "classifier": "ALL",
        "groups": [{"spans": {"spans": "EQUAL"},
                    "map": {"map": "STEP", "base": "CYCLIC", "step": 1, "offset": 0}}]}}}}"#;
    let custom = context(own).unwrap();
    let shipped = context("{}").unwrap();
    let ours = chart(&custom, BIRTH, &[Varga::D9]);
    let theirs = chart(&shipped, BIRTH, &[Varga::D9]);
    let (ours, theirs) = (varga(&ours, Varga::D9), varga(&theirs, Varga::D9));
    assert_eq!(ours.axis.grahas.variant, VargaVariant::Custom);
    assert_eq!(ours.grahas, theirs.grahas);
    assert_eq!(ours.lagna, theirs.lagna);
}

#[test]
fn a_broken_row_or_a_stray_variant_is_refused_by_its_field() {
    for (patch, field) in [
        (
            r#"{"vargas": {"schemes": {"D9": {"kind": "CUSTOM", "classifier": "PARITY",
                "groups": [{"spans": {"spans": "EQUAL"},
                            "map": {"map": "LISTED", "signs": [0]}}]}}}}"#,
            "vargas.schemes.D9.groups",
        ),
        (
            r#"{"vargas": {"schemes": {"D9": {"kind": "CUSTOM", "classifier": "ALL",
                "groups": [{"spans": {"spans": "EQUAL"},
                            "map": {"map": "LISTED", "signs": [0]}}]}}}}"#,
            "vargas.schemes.D9.groups[0].map.signs",
        ),
        (
            r#"{"vargas": {"schemes": {"D10": {"kind": "NAMED", "which": "KASHINATHA"}}}}"#,
            "vargas.schemes.D10.which",
        ),
    ] {
        let refused = context(patch).unwrap_err().to_string();
        assert!(refused.contains(field), "{field}: {refused}");
    }
}

#[test]
fn the_putrakaraka_is_the_matrikarakas_graha_only_where_the_settings_say() {
    let jaimini = context(r#"{"jaimini": {"chara_karakas": "SEVEN"}}"#).unwrap();
    let merged = context(
        r#"{"jaimini": {"chara_karakas": "SEVEN", "seven_karaka_order": "MATRI_IS_PUTRA"}}"#,
    )
    .unwrap();
    for step in 0..12 {
        let at = BIRTH + f64::from(step) * 3.1;
        let document = chart(&jaimini, at, &[]);
        let plain = RuleInputs::of(&document)
            .unwrap()
            .under(jaimini.settings())
            .unwrap()
            .chart;
        let named = RuleInputs::of(&document)
            .unwrap()
            .under(merged.settings())
            .unwrap()
            .chart;
        // Jaimini 1.1.13-18: seven karakas and no Pitrikaraka.
        assert_eq!(plain.holder(CharaKaraka::Pitrikaraka, false), None);
        // BPHS ch. 32 v. 16: the fifth rank is the father's, and the
        // Matrikaraka's graha also signifies the children.
        let fifth = plain.holder(CharaKaraka::Putrakaraka, false).unwrap();
        assert_eq!(named.holder(CharaKaraka::Pitrikaraka, false), Some(fifth));
        assert_eq!(
            named.holder(CharaKaraka::Putrakaraka, false),
            named.holder(CharaKaraka::Matrikaraka, false)
        );
        // The ranking is a sky's, not a naming's.
        for karaka in [CharaKaraka::Atmakaraka, CharaKaraka::Darakaraka] {
            assert_eq!(
                plain.holder(karaka, false),
                named.holder(karaka, false),
                "{karaka:?}"
            );
        }
        assert_ne!(fifth, Graha::Rahu);
    }
}

#[test]
fn eight_on_a_tie_counts_seven_on_a_sky_without_one() {
    let on_tie = context(r#"{"jaimini": {"chara_karakas": "EIGHT_ON_A_TIE"}}"#).unwrap();
    let seven = context(r#"{"jaimini": {"chara_karakas": "SEVEN"}}"#).unwrap();
    let mut untied = 0;
    for step in 0..12 {
        let at = BIRTH + f64::from(step) * 2.3;
        let document = chart(&on_tie, at, &[]);
        let ruled = RuleInputs::of(&document)
            .unwrap()
            .under(on_tie.settings())
            .unwrap()
            .chart;
        if ruled.seven_tie() {
            continue;
        }
        untied += 1;
        let a = on_tie.chart().jaimini(&document).unwrap().karakamsha;
        let b = seven.chart().jaimini(&document).unwrap().karakamsha;
        assert_eq!(a.atmakaraka, b.atmakaraka);
    }
    assert!(untied > 0);
}
