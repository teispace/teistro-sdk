//! The ishṭa-devatā through the façade: the 12th from the kārakāṁśa the
//! Jaimini reading names, read in both charts (`docs/03-design/remedies.md`
//! step 4).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the nine they read"
)]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::remedies::{DevataRules, IshtaDevata, NINE, SunWithKetu};
use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};

#[test]
fn every_graha_in_the_twelfth_is_read_and_no_other() {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let request = ChartRequest::at(place, UtcOffset::literal(5, 45, 0));
    // A month of charts, a day apart: the Moon walks through every sign.
    let mut read_any = 0;
    for day in 0..30 {
        let instant = JulianDay::<Utc>::literal(2_447_995.489_583_333_5 + f64::from(day));
        let chart = sdk.chart().reading(instant, &request).unwrap().value;
        let jaimini = sdk.chart().jaimini(&chart).unwrap().karakamsha;
        let read = sdk
            .chart()
            .ishta_devata(&chart, DevataRules::default())
            .unwrap();
        assert_eq!(read.karakamsha, jaimini.sign);
        assert_eq!(read.atmakaraka, jaimini.atmakaraka);
        for (houses, devata) in [
            (jaimini.in_rasi, &read.in_rasi),
            (jaimini.in_navamsha, &read.in_navamsha),
        ] {
            let twelfth: Vec<Graha> = NINE
                .into_iter()
                .zip(houses)
                .filter(|&(_, house)| house == 12)
                .map(|(graha, _)| graha)
                .collect();
            let read: Vec<Graha> = devata.devotions.iter().map(|one| one.graha).collect();
            assert_eq!(read, twelfth, "day {day}");
            read_any += usize::from(!twelfth.is_empty());
            assert_nodes_apart(devata);
        }
        let surya = sdk
            .chart()
            .ishta_devata(
                &chart,
                DevataRules {
                    sun_with_ketu: SunWithKetu::Surya,
                },
            )
            .unwrap();
        assert_eq!(surya.in_rasi.rules.sun_with_ketu, SunWithKetu::Surya);
    }
    // An empty 12th answers nothing; the month must reach full ones.
    assert!(read_any > 10, "{read_any} of 60 readings had a graha there");
}

/// The nodes never stand in one sign, so neither carries `with_ketu`.
fn assert_nodes_apart(devata: &IshtaDevata) {
    let nodes = devata
        .devotions
        .iter()
        .filter(|one| matches!(one.graha, Graha::Rahu | Graha::Ketu))
        .count();
    assert!(nodes <= 1);
    assert!(
        devata
            .devotions
            .iter()
            .filter(|one| matches!(one.graha, Graha::Rahu | Graha::Ketu))
            .all(|one| !one.with_ketu)
    );
}
