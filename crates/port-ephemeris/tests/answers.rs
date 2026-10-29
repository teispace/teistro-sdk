//! What the port's own types answer at their edges: a grid written or
//! joined outside its shape, a request the port refuses before any
//! provider sees it, counts of nothing, and the keys a report prints.

#![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

use teistro_core::quantity::Place;
use teistro_port_ephemeris::{
    Astronomy, Body, Capabilities, Cell, CellStatus, Centre, CountingProvider, EphemerisProvider,
    Frame, Horizon, PositionColumns, PositionRequest, ProviderError, SpeedModel, TestProvider,
    TimeScale, ask_positions, validate,
};

#[test]
fn a_cell_outside_the_grid_is_neither_written_nor_read() {
    let mut columns = PositionColumns::new(2, 2, Frame::CANONICAL);
    let marked = Cell {
        lon: 7.0,
        ..Cell::EMPTY
    };
    columns.set_at(2, 0, marked);
    columns.set_at(0, 2, marked);
    assert!(
        columns
            .cells()
            .all(|cell| cell.lon.to_bits() != 7.0_f64.to_bits())
    );
    assert_eq!(columns.index(2, 0), None);
    assert!(columns.at(0, 2).is_none());
    columns.set_at(1, 1, marked);
    assert_eq!(
        columns.at(1, 1).map(|cell| cell.lon.to_bits()),
        Some(7.0_f64.to_bits())
    );
}

#[test]
fn a_grid_of_other_bodies_or_another_frame_is_not_joined() {
    let mut first = PositionColumns::new(1, 2, Frame::CANONICAL);
    let before = first.clone();
    assert!(!first.append(PositionColumns::new(1, 3, Frame::CANONICAL)));
    assert!(!first.append(PositionColumns::new(
        1,
        2,
        Frame::CANONICAL.with_centre(Centre::Topocentric)
    )));
    assert!(first.bit_identical(&before), "the grid is left as it was");
    assert!(first.append(PositionColumns::new(2, 2, Frame::CANONICAL)));
    assert_eq!(first.jd_count, 3);
}

#[test]
fn two_grids_differ_by_any_column_or_status() {
    let jds = [2_451_545.0];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
    let answer = TestProvider::new().positions(&request).unwrap();
    assert!(answer.all_ok() && answer.bit_identical(&answer.clone()));
    let changed = |change: &dyn Fn(&mut PositionColumns)| {
        let mut changed = answer.clone();
        change(&mut changed);
        changed
    };
    let moved = changed(&|columns| {
        for value in &mut columns.dist_speed {
            *value = -*value - 1.0;
        }
    });
    assert!(!answer.bit_identical(&moved));
    let failed = changed(&|columns| columns.status.fill(CellStatus::OutOfRange));
    assert!(!failed.all_ok() && !answer.bit_identical(&failed));
    let negative_zero = changed(&|columns| {
        for value in &mut columns.lat {
            *value = if value.to_bits() == 0.0_f64.to_bits() {
                -0.0
            } else {
                0.0
            };
        }
    });
    assert!(!answer.bit_identical(&negative_zero), "compared by bits");
}

#[test]
fn the_port_refuses_a_request_before_any_provider_sees_it() {
    let capabilities = TestProvider::new().capabilities();
    let jds = [2_451_545.0];
    let topocentric = PositionRequest::new(
        &jds,
        TimeScale::Ut1,
        &[Body::Sun],
        Frame::CANONICAL.with_centre(Centre::Topocentric),
    );
    assert!(matches!(
        validate(&capabilities, &topocentric),
        Err(ProviderError::Invalid { detail }) if detail.contains("needs an observer")
    ));
    let placed = PositionRequest {
        observer: Some(Place::try_from_degrees(27.7, 85.3, 0.0).unwrap()),
        ..topocentric
    };
    assert!(validate(&capabilities, &placed).is_ok());
    let infinite = [f64::INFINITY];
    let request = PositionRequest::new(&infinite, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
    assert!(matches!(
        validate(&capabilities, &request),
        Err(ProviderError::Invalid { detail }) if detail == "non-finite instant inf"
    ));
}

#[test]
fn a_grid_the_provider_covers_nowhere_never_reaches_it() {
    let counting = CountingProvider::new(TestProvider::new());
    let capabilities = counting.capabilities();
    let far = [1.0e9, 2.0e9];
    let request = PositionRequest::new(&far, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
    let columns = ask_positions(&counting, &capabilities, &request).unwrap();
    assert!(
        columns
            .status
            .iter()
            .all(|status| *status == CellStatus::OutOfRange)
    );
    assert_eq!(counting.calls().total(), 0);
    // Covered everywhere, it is passed through as asked.
    let near = [2_451_545.0, 2_451_546.0];
    let request = PositionRequest::new(&near, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
    assert!(
        ask_positions(&counting, &capabilities, &request)
            .unwrap()
            .all_ok()
    );
    assert_eq!(counting.calls().total(), 1);
}

#[test]
fn counts_of_nothing_are_zero_and_a_reset_forgets_the_repeats() {
    let counting = CountingProvider::new(TestProvider::new());
    let calls = counting.calls();
    assert_eq!(calls.repeat_share().to_bits(), 0.0_f64.to_bits());
    assert_eq!(calls.mean_width().to_bits(), 0.0_f64.to_bits());
    let jds = [2_451_545.0];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
    counting.positions(&request).unwrap();
    // Not watching, no cell is told apart, so none repeats.
    assert_eq!(counting.calls().repeat_share().to_bits(), 0.0_f64.to_bits());
    counting.reset();
    assert_eq!(counting.calls().total(), 0);

    let watching = CountingProvider::new(TestProvider::new()).watching_repeats();
    watching.positions(&request).unwrap();
    watching.positions(&request).unwrap();
    watching.reset();
    watching.positions(&request).unwrap();
    assert_eq!(
        watching.calls().repeat_share().to_bits(),
        0.0_f64.to_bits(),
        "the reset forgot the cells it had seen"
    );
}

#[test]
fn a_report_line_names_every_departure_from_the_usual() {
    let usual = TestProvider::new().capabilities();
    let unusual = Capabilities {
        astronomy: Astronomy::Classical,
        speed_model: SpeedModel::Rule,
        deterministic: false,
        ..usual.clone()
    };
    let line = unusual.describe();
    assert!(
        line.ends_with(", classical, speeds by rule, non-deterministic"),
        "{line}"
    );
    assert!(!usual.describe().contains("classical"));
}

#[test]
fn every_correction_set_has_a_key() {
    use teistro_port_ephemeris::Corrections;
    let nutation = Corrections {
        nutation: true,
        ..Corrections::GEOMETRIC
    };
    assert_eq!(nutation.key(), "nutation");
    let deflected = Corrections {
        deflection: false,
        ..Corrections::APPARENT
    };
    assert_eq!(deflected.key(), "light_time+aberration+nutation");
}

#[test]
fn the_refracted_centre_is_a_named_convention() {
    use teistro_core::settings::{Sunrise, SunriseConvention};
    assert_eq!(
        Horizon::CENTRE_REFRACTION.convention(),
        Some(SunriseConvention::Named {
            which: Sunrise::CentreRefraction
        })
    );
}

#[test]
fn a_cache_of_nothing_passes_through_and_one_of_one_cell_stores_one() {
    use teistro_port_ephemeris::CachingProvider;
    let jds = [2_451_545.0];
    let request = PositionRequest::new(
        &jds,
        TimeScale::Ut1,
        &[Body::Sun, Body::Moon],
        Frame::CANONICAL,
    );

    let off = CachingProvider::with_capacity(TestProvider::new(), 0);
    assert!(!off.caching());
    assert!(off.positions(&request).unwrap().all_ok());
    assert_eq!(
        off.stats().hit_share().to_bits(),
        0.0_f64.to_bits(),
        "nothing asked of it"
    );

    let one = CachingProvider::with_capacity(TestProvider::new(), 1);
    let first = one.positions(&request).unwrap();
    let stats = one.stats();
    assert_eq!((stats.stored, stats.refused), (1, 1), "room for one cell");
    assert!(one.positions(&request).unwrap().bit_identical(&first));
    assert_eq!(one.stats().hits, 1);

    let empty = PositionRequest::new(&[], TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
    let caching = CachingProvider::new(TestProvider::new());
    assert_eq!(caching.positions(&empty).unwrap().len(), 0);
    assert_eq!(caching.stats().misses, 0, "an empty grid is not looked up");
}

#[test]
fn a_cache_remembers_a_refusal_by_the_request_s_shape() {
    use teistro_port_ephemeris::CachingProvider;
    let caching = CachingProvider::new(TestProvider::new());
    let jds = [2_451_545.0];
    let equatorial = PositionRequest::new(
        &jds,
        TimeScale::Ut1,
        &[Body::Sun],
        Frame::CANONICAL.with_coordinates(teistro_port_ephemeris::Coordinates::Equatorial),
    );
    let first = caching.positions(&equatorial).unwrap_err();
    assert!(matches!(first, ProviderError::Unsupported { .. }));
    assert_eq!(caching.stats().failed, 1);
    let again = caching.positions(&equatorial).unwrap_err();
    assert_eq!(again, first, "the same refusal, without asking");
    assert_eq!(caching.stats().remembered, 1);
    caching.clear();
    assert_eq!(caching.stats().remembered, 0);
    assert!(caching.positions(&equatorial).is_err());
    assert_eq!(caching.stats().failed, 1, "asked again after the clear");
}

#[test]
fn two_grids_differ_by_each_column_alone() {
    let jds = [2_451_545.0];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Moon], Frame::CANONICAL);
    let answer = TestProvider::new().positions(&request).unwrap();
    let columns: [fn(&mut PositionColumns) -> &mut Vec<f64>; 6] = [
        |c| &mut c.lon,
        |c| &mut c.lat,
        |c| &mut c.dist,
        |c| &mut c.lon_speed,
        |c| &mut c.lat_speed,
        |c| &mut c.dist_speed,
    ];
    for (index, column) in columns.into_iter().enumerate() {
        let mut changed = answer.clone();
        for value in column(&mut changed) {
            *value += 1.0;
        }
        assert!(!answer.bit_identical(&changed), "column {index}");
    }
    let mut shorter = answer.clone();
    shorter.lon.clear();
    assert!(
        !answer.bit_identical(&shorter),
        "a column of another length"
    );
    let mut outside = answer.clone();
    outside.set(5, Cell::failed(CellStatus::OutOfRange));
    assert!(
        outside.bit_identical(&answer),
        "a cell outside the grid is not written"
    );
}

#[test]
fn a_grid_covered_in_part_keeps_each_body_in_its_row() {
    let provider = TestProvider::new();
    let capabilities = provider.capabilities();
    let jds = [1.0e9, 2_451_545.0, 2.0e9, 2_451_546.0];
    let bodies = [Body::Sun, Body::Moon];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &bodies, Frame::CANONICAL);
    let columns = ask_positions(&provider, &capabilities, &request).unwrap();
    let whole = PositionRequest::new(&jds[1..2], TimeScale::Ut1, &bodies, Frame::CANONICAL);
    let direct = provider.positions(&whole).unwrap();
    for body in 0..2 {
        assert_eq!(
            columns.at(1, body).map(|cell| cell.lon.to_bits()),
            direct.at(0, body).map(|cell| cell.lon.to_bits()),
            "body {body}"
        );
        assert_eq!(
            columns.at(0, body).map(|cell| cell.status),
            Some(CellStatus::OutOfRange)
        );
    }
}

#[test]
fn every_named_horizon_and_frame_key_reads_back() {
    use teistro_core::catalogue::Ayanamsha;
    use teistro_core::settings::SunriseConvention;
    use teistro_port_ephemeris::{Corrections, DiscPoint, Overrides, Refraction};
    assert_eq!(Corrections::GEOMETRIC.key(), "GEOMETRIC");
    let twilight = Horizon {
        disc: DiscPoint::Centre,
        refraction: Refraction::None,
        altitude_deg: -6.0,
    };
    assert!(matches!(
        twilight.convention(),
        Some(SunriseConvention::Custom { altitude_deg }) if altitude_deg.to_bits() == (-6.0_f64).to_bits()
    ));
    let usual = TestProvider::new().capabilities();
    let declared = Capabilities {
        overrides: Overrides::AYANAMSHA,
        ayanamshas: vec![Ayanamsha::Lahiri],
        ..usual
    };
    assert!(declared.has_ayanamsha(Ayanamsha::Lahiri));
    assert!(!declared.has_ayanamsha(Ayanamsha::Raman));
}
