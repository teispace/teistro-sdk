//! Placement, over every lagna in every shipped layout
//! (`03-design/chart-geometry.md` §4).
//!
//! Placement only chooses, so its whole behaviour is a finite table: twelve
//! lagnas times four layouts. Every row of it is checked, not a sample.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index their own fixtures"
)]

use teistro_core::catalogue::{Graha, Rashi};
use teistro_geometry::Point;
use teistro_geometry::layout::{Direction, Holds};
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
        let Some(grid) = layout.shape.as_grid() else {
            continue;
        };
        for lagna in Rashi::ALL {
            let chart = chart(lagna);
            let placed = place(&layout, &chart).unwrap();
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

            for (cell, row) in placed.cells.iter().zip(&grid.cells) {
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
                // The lagna is in house 1 and nowhere else, and a grid has
                // one ring.
                assert_eq!(cell.lagna, cell.house == 1);
                assert_eq!(cell.ring, 0);
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
            assert_eq!(placed.frame, grid.frame);
        }
    }
}

#[test]
fn the_fixed_layouts_keep_what_they_fix_whatever_the_lagna() {
    // South Indian keeps Pisces top-left for every lagna; North Indian
    // keeps house 1 in the top diamond, whose sign is the lagna's.
    for lagna in Rashi::ALL {
        let south = place(&rows::south_indian(), &chart(lagna)).unwrap();
        let top_left = south
            .cells
            .iter()
            .find(|cell| cell.outline.contains(Point::new(0.1, 0.1)))
            .unwrap();
        assert_eq!(top_left.sign, Rashi::Pisces);

        let north = place(&rows::north_indian(), &chart(lagna)).unwrap();
        let top = north
            .cells
            .iter()
            .find(|cell| cell.outline.contains(Point::new(0.5, 0.2)))
            .unwrap();
        assert_eq!((top.house, top.sign), (1, lagna));
    }
}

#[test]
fn a_placed_chart_reads_back_from_its_own_json() {
    let placed = place(&rows::nepali_lotus(), &chart(Rashi::Scorpio)).unwrap();
    let json = serde_json::to_string(&placed).unwrap();
    assert!(
        json.contains("\"graha.SUN\""),
        "a body is its catalogue key"
    );
    let read: teistro_geometry::Placed = serde_json::from_str(&json).unwrap();
    assert_eq!(read, placed);
}

/// The sector of `ring` that holds the point `minutes` minutes clockwise of
/// twelve o'clock, midway across the ring.
fn sector_at(
    placed: &teistro_geometry::Placed,
    radial: &teistro_geometry::Radial,
    ring: u8,
    minutes: f64,
) -> (u8, Rashi) {
    let band = radial.rings[usize::from(ring)];
    let radius = f64::midpoint(band.inner, band.outer);
    let angle = minutes / 60.0 * 30f64.to_radians();
    let point = Point::new(0.5 + radius * angle.sin(), 0.5 - radius * angle.cos());
    let cell = placed
        .cells
        .iter()
        .find(|cell| cell.ring == ring && cell.outline.contains(point))
        .unwrap_or_else(|| panic!("nothing of ring {ring} at {minutes} minutes"));
    (cell.house, cell.sign)
}

#[test]
fn the_chakra_counts_each_ring_from_its_own_place() {
    // Lagna in Leo, the Moon in Taurus, the Sun in Scorpio.
    let chart = Placements {
        lagna: Rashi::Leo,
        bodies: vec![
            (Graha::Sun.key_id(), Rashi::Scorpio),
            (Graha::Moon.key_id(), Rashi::Taurus),
        ],
    };
    let layout = rows::sudarshan_chakra();
    let radial = layout.shape.as_radial().unwrap();
    let placed = place(&layout, &chart).unwrap();
    assert_eq!(placed.cells.len(), 36);

    // Just after twelve o'clock is house 1 of every ring, just before it is
    // house 12, and each ring's first house is its own reference's sign.
    for (ring, first) in [(0, Rashi::Leo), (1, Rashi::Taurus), (2, Rashi::Scorpio)] {
        assert_eq!(sector_at(&placed, radial, ring, 5.0), (1, first));
        let twelfth = Rashi::from_id((first.id() + 11) % 12).unwrap();
        assert_eq!(sector_at(&placed, radial, ring, -5.0), (12, twelfth));
        // And the houses run clockwise: an hour on is house 2.
        assert_eq!(sector_at(&placed, radial, ring, 65.0).0, 2);
    }

    // The Sun stands in the Sun ring's first house and the lagna in the
    // lagna ring's.
    let sun_ring_first = placed
        .cells
        .iter()
        .find(|c| c.ring == 2 && c.house == 1)
        .unwrap();
    assert_eq!(sun_ring_first.bodies, vec![Graha::Sun.key_id()]);
    assert!(
        placed
            .cells
            .iter()
            .find(|c| c.ring == 0 && c.house == 1)
            .unwrap()
            .lagna
    );
}

#[test]
fn every_chakra_sector_holds_its_anchors_and_overlaps_no_other() {
    let placed = place(&rows::sudarshan_chakra(), &chart(Rashi::Aries)).unwrap();
    for cell in &placed.cells {
        assert!(
            cell.outline.contains(cell.label),
            "ring {} house {}",
            cell.ring,
            cell.house
        );
        assert!(
            cell.outline.contains(cell.anchor),
            "ring {} house {}",
            cell.ring,
            cell.house
        );
    }
    // No point on a fine grid is inside two sectors, and the sectors cover
    // the three rings' area.
    for i in 0..61 {
        for j in 0..61 {
            let point = Point::new((f64::from(i) + 0.37) / 61.0, (f64::from(j) + 0.37) / 61.0);
            let holders = placed
                .cells
                .iter()
                .filter(|c| c.outline.contains(point))
                .count();
            assert!(
                holders <= 1,
                "{holders} sectors hold ({}, {})",
                point.x,
                point.y
            );
        }
    }
    let covered: f64 = placed.cells.iter().map(|c| c.outline.area()).sum();
    let (inner, outer) = (50.0 / 480.0, 220.0 / 480.0);
    let annulus = std::f64::consts::PI * (outer * outer - inner * inner);
    assert!(
        (covered - annulus).abs() / annulus < 1e-2,
        "{covered} of {annulus}"
    );
    // The frame draws the four circles the three rings are bounded by, the
    // two they share once each.
    assert_eq!(placed.frame.len(), 4);
}

#[test]
fn a_ring_the_chart_cannot_count_is_refused_by_name() {
    let chart = Placements {
        lagna: Rashi::Leo,
        bodies: vec![(Graha::Sun.key_id(), Rashi::Scorpio)],
    };
    let error = place(&rows::sudarshan_chakra(), &chart).expect_err("no Moon");
    assert_eq!(error.field(), Some("bodies"));
    assert!(error.message.contains("graha.MOON"), "{error}");
}

#[test]
fn an_anticlockwise_chakra_runs_the_other_way() {
    let mut layout = rows::sudarshan_chakra();
    if let teistro_geometry::Shape::Radial(radial) = &mut layout.shape {
        radial.direction = Direction::Anticlockwise;
    }
    let radial = layout.shape.as_radial().unwrap().clone();
    let placed = place(&layout, &chart(Rashi::Aries)).unwrap();
    assert_eq!(sector_at(&placed, &radial, 0, -5.0).0, 1);
    assert_eq!(sector_at(&placed, &radial, 0, 5.0).0, 12);
    assert_eq!(sector_at(&placed, &radial, 0, -65.0).0, 2);
}
