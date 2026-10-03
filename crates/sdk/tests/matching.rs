//! Two charts matched through their Moons (`03-design/matching.md`): the
//! natives read from each founded chart's own sidereal Moon, the kootas
//! the kernel's, and a tropical chart refused by the role it was given.

#![allow(
    clippy::unwrap_used,
    clippy::float_cmp,
    reason = "tests fail by panicking and compare exact halves"
)]

use teistro::catalogue::{Graha, Koota};
use teistro::matching::{ashta_koota, porutham};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    ChartRequest, Context, Document, Ephemeris, KootaReading, KootaRules, MatchRole, Native,
    Partner, PartnerMatching, PoruthamRules, UtcOffset,
};

fn context(profile: Option<&str>) -> Context {
    let builder = Context::builder().ephemeris([Ephemeris::Builtin]);
    match profile {
        Some(profile) => builder.profile(profile),
        None => builder,
    }
    .build()
    .unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn founded(sdk: &Context, instant: f64) -> Document {
    sdk.chart()
        .reading(
            JulianDay::<Utc>::try_new(instant).unwrap(),
            &ChartRequest::at(kathmandu(), UtcOffset::UTC),
        )
        .unwrap()
        .value
}

fn partner(instant: f64) -> Partner {
    Partner {
        instant: JulianDay::<Utc>::try_new(instant).unwrap(),
        place: kathmandu(),
        utc_offset: UtcOffset::UTC,
    }
}

fn moon(chart: &Document) -> Native {
    Native::of_moon(chart.foundation.graha(Graha::Moon).unwrap().longitude_deg).unwrap()
}

#[test]
fn a_match_reads_each_charts_own_sidereal_moon() {
    let sdk = context(None);
    // Every other day for four weeks: the Moon through every sign.
    let bride = founded(&sdk, 2_447_892.5);
    for day in 0..14 {
        let groom = founded(&sdk, 2_451_545.0 + 2.0 * f64::from(day));
        let koota = sdk
            .chart()
            .matching(&bride, &groom, KootaRules::default())
            .unwrap();
        assert_eq!(
            koota,
            ashta_koota(moon(&bride), moon(&groom), KootaRules::default())
        );
        let ten = sdk
            .chart()
            .porutham(&bride, &groom, PoruthamRules::default())
            .unwrap();
        assert_eq!(
            ten,
            porutham(moon(&bride), moon(&groom), PoruthamRules::default())
        );
        assert!((0.0..=36.0).contains(&koota.total));
        let Some(KootaReading::Tara {
            bride_to_groom,
            groom_to_bride,
        }) = koota.row(Koota::Tara).map(|row| row.reading)
        else {
            unreachable!()
        };
        assert!((1..=9).contains(&bride_to_groom) && (1..=9).contains(&groom_to_bride));
    }
}

#[test]
fn a_tropical_chart_is_refused_by_its_role() {
    let sidereal = founded(&context(None), 2_447_892.5);
    let western = context(Some("western-tropical-default"));
    let tropical = founded(&western, 2_451_545.0);
    let refused = western
        .chart()
        .matching(&sidereal, &tropical, KootaRules::default())
        .unwrap_err();
    assert_eq!(refused.field(), Some("groom"));
    let refused = western
        .chart()
        .matching(&tropical, &sidereal, KootaRules::default())
        .unwrap_err();
    assert_eq!(refused.field(), Some("bride"));
}

#[test]
fn a_batch_stands_on_the_side_the_partner_does_not() {
    let sdk = context(None);
    let hers = founded(&sdk, 2_447_892.5);
    let charts: Vec<Document> = (0..5)
        .map(|day| founded(&sdk, 2_451_545.0 + 5.0 * f64::from(day)))
        .collect();
    let rules = KootaRules {
        nadi_dosha: teistro::matching::NadiDosha::MiddleOnly,
        ..KootaRules::default()
    };
    for role in [MatchRole::Bride, MatchRole::Groom] {
        let asked = PartnerMatching {
            partner: partner(2_447_892.5),
            partner_role: role,
            rules,
            porutham: PoruthamRules {
                deergha_beyond: teistro::matching::DeerghaBeyond::Seventh,
                ..PoruthamRules::default()
            },
        };
        let matched = sdk.chart().matching_with(&charts, &asked).unwrap();
        assert_eq!(matched.len(), charts.len());
        for (chart, both) in charts.iter().zip(&matched) {
            let (bride, groom) = match role {
                MatchRole::Bride => (&hers, chart),
                MatchRole::Groom => (chart, &hers),
            };
            assert_eq!(
                both.ashta_koota,
                sdk.chart().matching(bride, groom, rules).unwrap()
            );
            assert_eq!(
                both.porutham,
                sdk.chart().porutham(bride, groom, asked.porutham).unwrap()
            );
        }
    }
}

#[test]
fn a_tropical_partner_is_refused_as_the_partner() {
    let western = context(Some("western-tropical-default"));
    let asked = PartnerMatching {
        partner: partner(2_447_892.5),
        partner_role: MatchRole::Groom,
        rules: KootaRules::default(),
        porutham: PoruthamRules::default(),
    };
    let refused = western.chart().matching_with(&[], &asked).unwrap_err();
    assert_eq!(refused.field(), Some("partner"));
}

#[test]
fn a_record_reads_back_what_it_wrote() {
    let asked = PartnerMatching {
        partner: partner(2_447_892.5),
        partner_role: MatchRole::Bride,
        rules: KootaRules::default(),
        porutham: PoruthamRules::default(),
    };
    let text = serde_json::to_string(&asked).unwrap();
    assert_eq!(PartnerMatching::from_json(&text).unwrap(), asked);
    let north = PartnerMatching::from_json(
        r#"{"partner": {"instant": 2447892.5, "place": {"latitude": 95, "longitude": 0, "altitude": 0}}, "partnerRole": "BRIDE"}"#,
    )
    .unwrap_err();
    assert_eq!(north.field(), Some("matching.partner.place.latitude"));
    let typo = PartnerMatching::from_json(
        r#"{"partner": {"instant": 2447892.5, "place": {"latitude": 27, "longitude": 85, "altitude": 0}}, "partnerRole": "BRIDE", "porutham": {"deergha": "SEVENTH"}}"#,
    )
    .unwrap_err();
    assert_eq!(typo.field(), Some("matching.porutham.deergha"));
}
