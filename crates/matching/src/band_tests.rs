//! The three printed tables held to the thresholds their sources state,
//! and a consumer's table held to the shape a grade needs.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    reason = "tests fail by panicking, index their own results and compare the totals they passed in"
)]

use teistro_core::catalogue::{Koota, Nakshatra};

use crate::{
    BandRow, BhakootStanding, Clears, GunaBand, GunaBands, KootaReading, KootaRules, Native,
    ashta_koota,
};

/// A Moon in the middle of a nakshatra's first pada.
fn moon(star: Nakshatra) -> Native {
    Native::of_moon(f64::from(star.id()) * 360.0 / 27.0 + 1.0).unwrap()
}

/// The band each total falls in, with the signs standing well.
fn bands(table: &GunaBands, totals: &[f64]) -> Vec<GunaBand> {
    totals
        .iter()
        .map(|&total| table.grade_total(total, false).band)
        .collect()
}

#[test]
fn muhurta_martanda_asks_for_more_than_eighteen() {
    let table = GunaBands::muhurta_martanda();
    assert_eq!(
        bands(&table, &[0.0, 18.0, 18.5, 36.0]),
        [
            GunaBand::Unfit,
            GunaBand::Unfit,
            GunaBand::Fit,
            GunaBand::Fit
        ]
    );
    // A dushkuta does not move this floor: the verse names none.
    assert_eq!(table.grade_total(18.5, true).band, GunaBand::Fit);
    assert_eq!(GunaBands::default(), table);
}

#[test]
fn the_nepali_almanac_lowers_the_floor_to_sixteen_when_the_signs_stand_well() {
    let table = GunaBands::nepali_almanac();
    assert_eq!(
        bands(&table, &[16.0, 16.5, 18.0, 20.0, 20.5, 30.0, 30.5]),
        [
            GunaBand::Unfit,
            GunaBand::Middling,
            GunaBand::Middling,
            GunaBand::Middling,
            GunaBand::Good,
            GunaBand::Good,
            GunaBand::Excellent,
        ]
    );
    // With a dushkuta the 16 row does not hold, and 18 is the floor.
    let graded = table.grade_total(17.0, true);
    assert_eq!(
        (graded.band, graded.row, graded.dushkuta),
        (GunaBand::Unfit, 0, true)
    );
    let graded = table.grade_total(18.0, true);
    assert_eq!((graded.band, graded.row), (GunaBand::Middling, 2));
}

#[test]
fn aryavarta_reads_its_grid() {
    let table = GunaBands::aryavarta();
    assert_eq!(
        bands(&table, &[16.0, 17.0, 20.0, 21.0, 30.0, 31.0]),
        [
            GunaBand::Unfit,
            GunaBand::Middling,
            GunaBand::Middling,
            GunaBand::Good,
            GunaBand::Good,
            GunaBand::Excellent,
        ]
    );
    assert_eq!(table.grade_total(17.0, true).band, GunaBand::Middling);
}

#[test]
fn every_shipped_table_is_found_by_its_name_and_checks() {
    for name in GunaBands::SHIPPED {
        let table = GunaBands::shipped(name).unwrap();
        assert_eq!(table.name, name);
        table.check().unwrap();
        let round = GunaBands::from_json(&serde_json::to_string(&table).unwrap()).unwrap();
        assert_eq!(round, table);
    }
    assert_eq!(GunaBands::shipped("PARASHARA"), None);
}

#[test]
fn a_grade_names_its_table_total_and_row() {
    let graded = GunaBands::aryavarta().grade_total(25.5, false);
    assert_eq!(graded.bands, "ARYAVARTA");
    assert_eq!(graded.total, 25.5);
    assert_eq!(graded.row, 2);
    assert!(!graded.dushkuta);
}

#[test]
fn a_koota_is_graded_by_its_bhakoot_dosha() {
    let mut seen = [false; 2];
    for bride in Nakshatra::ALL {
        for groom in Nakshatra::ALL {
            let koota = ashta_koota(moon(bride), moon(groom), KootaRules::default());
            let dosha = matches!(
                koota.row(Koota::Bhakoot).unwrap().reading,
                KootaReading::Bhakoot { dosha: Some(_), .. }
            );
            let graded = GunaBands::nepali_almanac().grade(&koota);
            assert_eq!(graded.dushkuta, dosha, "{bride:?} and {groom:?}");
            assert_eq!(graded.total, koota.total);
            assert_eq!(
                graded,
                GunaBands::nepali_almanac().grade_total(koota.total, dosha)
            );
            seen[usize::from(dosha)] = true;
        }
    }
    assert_eq!(
        seen,
        [true, true],
        "both standings are met among the 729 pairs"
    );
}

#[test]
fn a_comparison_clears_at_or_above_its_threshold_as_its_row_says() {
    assert!(Clears::AtLeast.cleared(18.0, 18.0));
    assert!(!Clears::Above.cleared(18.0, 18.0));
    assert!(Clears::Above.cleared(18.5, 18.0));
    assert!(BhakootStanding::Any.holds(true) && BhakootStanding::Any.holds(false));
    assert!(BhakootStanding::Satkuta.holds(false) && !BhakootStanding::Satkuta.holds(true));
    assert!(BhakootStanding::Dushkuta.holds(true) && !BhakootStanding::Dushkuta.holds(false));
}

/// A consumer's table whose rows are `rows`.
fn table(rows: Vec<BandRow>) -> GunaBands {
    GunaBands {
        name: String::from("MINE"),
        source: String::new(),
        rows,
    }
}

fn row(band: GunaBand, from: f64, clears: Clears, bhakoot: BhakootStanding) -> BandRow {
    BandRow {
        band,
        from,
        clears,
        bhakoot,
    }
}

#[test]
fn a_table_that_cannot_grade_every_total_is_refused_by_field() {
    let floor = row(GunaBand::Unfit, 0.0, Clears::AtLeast, BhakootStanding::Any);
    let refused = |bands: GunaBands| bands.check().unwrap_err().field().map(str::to_owned);
    let mut unnamed = table(vec![floor]);
    unnamed.name = String::from("  ");
    assert_eq!(refused(unnamed).as_deref(), Some("bands.name"));
    assert_eq!(refused(table(Vec::new())).as_deref(), Some("bands.rows"));
    for first in [
        row(GunaBand::Unfit, 1.0, Clears::AtLeast, BhakootStanding::Any),
        row(GunaBand::Unfit, 0.0, Clears::Above, BhakootStanding::Any),
        row(
            GunaBand::Unfit,
            0.0,
            Clears::AtLeast,
            BhakootStanding::Satkuta,
        ),
    ] {
        assert_eq!(
            refused(table(vec![first])).as_deref(),
            Some("bands.rows[0]")
        );
    }
    for from in [-1.0, 36.5, f64::NAN, f64::INFINITY] {
        let bands = table(vec![
            floor,
            row(GunaBand::Good, from, Clears::Above, BhakootStanding::Any),
        ]);
        assert_eq!(
            refused(bands).as_deref(),
            Some("bands.rows[1].from"),
            "{from}"
        );
    }
    table(vec![
        floor,
        row(GunaBand::Good, 36.0, Clears::AtLeast, BhakootStanding::Any),
    ])
    .check()
    .unwrap();
}

#[test]
fn a_table_read_from_json_takes_the_defaults_and_is_checked() {
    let read = GunaBands::from_json(
        r#"{"name":"MINE","source":"","rows":[{"band":"UNFIT","from":0},{"band":"GOOD","from":20,"clears":"ABOVE"}]}"#,
    )
    .unwrap();
    assert_eq!(read.rows[0].clears, Clears::AtLeast);
    assert_eq!(read.rows[1].bhakoot, BhakootStanding::Any);
    assert_eq!(read.grade_total(20.5, true).band, GunaBand::Good);
    let refused = GunaBands::from_json(r#"{"name":"MINE","source":"","rows":[]}"#).unwrap_err();
    assert_eq!(refused.field(), Some("bands.rows"));
    assert!(GunaBands::from_json(r#"{"name":"MINE"}"#).is_err());
}
