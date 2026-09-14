//! Placement, over every lagna in every shipped layout
//! (`03-design/chart-geometry.md` §4).
//!
//! Placement only chooses, so its whole behaviour is a finite table: twelve
//! lagnas times four layouts. Every row of it is checked, not a sample.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own fixtures"
)]

use teistro_core::catalogue::{Graha, Rashi};
use teistro_geometry::layout::Holds;
use teistro_geometry::{Placements, place, rows};

/// A chart with the lagna in `lagna` and one graha in every sign, so each
/// cell must receive exactly the graha of its own sign.
fn chart(lagna: Rashi) -> Placements {
    let grahas = [
        Graha::Sun,
        Graha::Moon,
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
        Graha::Rahu,
        Graha::Ketu,
    ];
    Placements {
        lagna,
        bodies: Rashi::ALL
            .iter()
            .zip(grahas.iter().cycle())
            .map(|(sign, graha)| (graha.key_id(), *sign))
            .collect(),
    }
}

#[test]
fn every_lagna_places_consistently_in_every_layout() {
    for layout in rows::shipped() {
        for lagna in Rashi::ALL {
            let chart = chart(lagna);
            let placed = place(&layout, &chart);
            assert_eq!(placed.layout, layout.key);
            assert_eq!(placed.cells.len(), 12);

            let mut signs: Vec<u16> = placed.cells.iter().map(|cell| cell.sign.id()).collect();
            let mut houses: Vec<u8> = placed.cells.iter().map(|cell| cell.house).collect();
            signs.sort_unstable();
            houses.sort_unstable();
            assert_eq!(
                signs,
                (0..12).collect::<Vec<_>>(),
                "{} {lagna:?}",
                layout.key
            );
            assert_eq!(
                houses,
                (1..=12).collect::<Vec<_>>(),
                "{} {lagna:?}",
                layout.key
            );

            for (cell, row) in placed.cells.iter().zip(&layout.grid.cells) {
                // A house counts inclusively from the lagna's sign.
                assert_eq!(
                    u16::from(cell.house - 1),
                    (cell.sign.id() + 12 - lagna.id()) % 12,
                    "{} {lagna:?}",
                    layout.key
                );
                // What the row fixes is what the placed cell shows.
                match row.holds {
                    Holds::Sign(sign) => assert_eq!(cell.sign, sign),
                    Holds::House(house) => assert_eq!(cell.house, house),
                }
                // The lagna is in house 1 and nowhere else.
                assert_eq!(cell.lagna, cell.house == 1);
                // Each cell holds exactly the graha put in its sign.
                let expected: Vec<_> = chart
                    .bodies
                    .iter()
                    .filter(|(_, at)| *at == cell.sign)
                    .map(|(key, _)| *key)
                    .collect();
                assert_eq!(cell.bodies, expected);
                assert_eq!(cell.outline, row.outline);
                assert_eq!((cell.label, cell.anchor), (row.label, row.bodies));
            }
            assert_eq!(placed.frame, layout.grid.frame);
        }
    }
}

#[test]
fn the_fixed_layouts_keep_what_they_fix_whatever_the_lagna() {
    // South Indian keeps Pisces top-left for every lagna; North Indian
    // keeps house 1 in the top diamond, whose sign is the lagna's.
    for lagna in Rashi::ALL {
        let south = place(&rows::south_indian(), &chart(lagna));
        let top_left = south
            .cells
            .iter()
            .find(|cell| {
                cell.outline
                    .contains(teistro_geometry::Point::new(0.1, 0.1))
            })
            .unwrap();
        assert_eq!(top_left.sign, Rashi::Pisces);

        let north = place(&rows::north_indian(), &chart(lagna));
        let top = north
            .cells
            .iter()
            .find(|cell| {
                cell.outline
                    .contains(teistro_geometry::Point::new(0.5, 0.2))
            })
            .unwrap();
        assert_eq!((top.house, top.sign), (1, lagna));
    }
}

#[test]
fn a_placed_chart_reads_back_from_its_own_json() {
    let placed = place(&rows::nepali_lotus(), &chart(Rashi::Scorpio));
    let json = serde_json::to_string(&placed).unwrap();
    assert!(
        json.contains("\"graha.SUN\""),
        "a body is its catalogue key"
    );
    let read: teistro_geometry::Placed = serde_json::from_str(&json).unwrap();
    assert_eq!(read, placed);
}
