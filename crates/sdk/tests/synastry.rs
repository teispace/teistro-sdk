//! Synastry, the Western aspects between two charts
//! (`03-design/western-synastry.md`), on the married pair whose births
//! Leo gives in *How to Judge a Nativity* (p. 130): King George V and
//! Queen Mary.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    AntisciaRequest, AspectRequest, ChartRequest, Context, Document, Ephemeris, NatalPoint,
    ParallelRequest, Partner, PartnerSynastry, SynastryRequest, SynastryRow, SynastryZodiac,
    UtcOffset, WesternAspect,
};

/// "born 1-18 a.m., 3rd June, 1865, London", at Marlborough House.
const GEORGE: (f64, f64, f64) = (2_402_390.554_166_667, 51.5045, -0.1366);
/// "born 11-59 p.m., 26th May, 1867, London", at Kensington Palace.
const MARY: (f64, f64, f64) = (2_403_113.499_305_556, 51.5058, -0.1878);

fn context(profile: Option<&str>) -> Context {
    let builder = Context::builder().ephemeris([Ephemeris::Builtin]);
    match profile {
        Some(profile) => builder.profile(profile),
        None => builder,
    }
    .build()
    .unwrap()
}

fn western() -> Context {
    context(Some("western-tropical-default"))
}

fn born(sdk: &Context, (jd, latitude, longitude): (f64, f64, f64)) -> Document {
    let place = Place::new(
        Latitude::literal(latitude),
        Longitude::literal(longitude),
        Altitude::literal(0.0),
    );
    sdk.chart()
        .reading(
            JulianDay::<Utc>::try_new(jd).unwrap(),
            &ChartRequest::at(place, UtcOffset::UTC).with_outer_planets(),
        )
        .unwrap()
        .value
}

const fn graha(graha: Graha) -> NatalPoint {
    NatalPoint::Graha { graha }
}

#[test]
fn leos_remarks_on_each_chart_hold() {
    // "Neptune rising in Aries in close sextile with the Sun", and
    // Jupiter "in Pisces intercepted in the ascendant in dexter square to
    // the Sun" (p. 130): what makes the two births a fixture.
    let sdk = western();
    let george = born(&sdk, GEORGE);
    let mary = born(&sdk, MARY);
    let aries = 0.0..30.0;
    let neptune = george
        .foundation
        .graha(Graha::Neptune)
        .unwrap()
        .longitude_deg;
    assert!(aries.contains(&neptune), "Neptune in Aries: {neptune}");
    assert!(
        aries.contains(&george.foundation.lagna_deg),
        "Aries rising: {}",
        george.foundation.lagna_deg
    );
    let jupiter = mary.foundation.graha(Graha::Jupiter).unwrap().longitude_deg;
    assert!(
        (330.0..360.0).contains(&jupiter),
        "Jupiter in Pisces: {jupiter}"
    );
    let holds = |chart: &Document, a: Graha, aspect: WesternAspect, b: Graha| {
        sdk.chart()
            .western_aspects(chart, &AspectRequest::default())
            .unwrap()
            .iter()
            .any(|row| row.aspect == aspect && [row.first, row.second] == [a, b])
    };
    assert!(holds(
        &george,
        Graha::Sun,
        WesternAspect::Sextile,
        Graha::Neptune
    ));
    assert!(holds(
        &mary,
        Graha::Sun,
        WesternAspect::Square,
        Graha::Jupiter
    ));
}

#[test]
fn the_cross_contacts_agree_with_the_recast() {
    // The Ptolemaic five within 2.5° of exact, from a Moshier recast of the
    // two births (pyswisseph, rank 3); the nearest contact outside the cut
    // stands 2.75° from exact and the widest inside 2.18°.
    let recast: [(NatalPoint, WesternAspect, NatalPoint, f64); 9] = [
        (
            graha(Graha::Mars),
            WesternAspect::Opposition,
            NatalPoint::Lagna,
            0.32,
        ),
        (
            graha(Graha::Mars),
            WesternAspect::Sextile,
            graha(Graha::Sun),
            0.39,
        ),
        (
            graha(Graha::Mercury),
            WesternAspect::Opposition,
            graha(Graha::Saturn),
            1.23,
        ),
        (
            graha(Graha::Venus),
            WesternAspect::Sextile,
            graha(Graha::Moon),
            1.23,
        ),
        (
            graha(Graha::Pluto),
            WesternAspect::Conjunction,
            graha(Graha::Pluto),
            1.69,
        ),
        (
            graha(Graha::Moon),
            WesternAspect::Trine,
            graha(Graha::Mercury),
            1.70,
        ),
        (
            graha(Graha::Sun),
            WesternAspect::Sextile,
            graha(Graha::Neptune),
            1.91,
        ),
        (
            graha(Graha::Pluto),
            WesternAspect::Square,
            graha(Graha::Mars),
            2.03,
        ),
        (
            graha(Graha::Mars),
            WesternAspect::Square,
            graha(Graha::Venus),
            2.18,
        ),
    ];
    let sdk = western();
    let rows = sdk
        .chart()
        .synastry(
            &born(&sdk, GEORGE),
            &born(&sdk, MARY),
            &SynastryRequest::default(),
        )
        .unwrap();
    let close: Vec<&SynastryRow> = rows
        .iter()
        .filter(|row| WesternAspect::PTOLEMAIC.contains(&row.aspect) && row.from_exact_deg <= 2.5)
        .collect();
    assert_eq!(close.len(), recast.len(), "{close:#?}");
    for (first, aspect, second, from_exact_deg) in recast {
        let row = close
            .iter()
            .find(|row| (row.first, row.aspect, row.second) == (first, aspect, second))
            .unwrap_or_else(|| panic!("{first:?} {aspect:?} {second:?} in {close:#?}"));
        assert!(
            (row.from_exact_deg - from_exact_deg).abs() < 0.01,
            "{first:?} {aspect:?} {second:?}: {} against {from_exact_deg}",
            row.from_exact_deg
        );
    }
    // Closest first, inside the orb, and every row across: his point first.
    assert!(
        rows.windows(2)
            .all(|pair| pair[0].from_exact_deg <= pair[1].from_exact_deg)
    );
    assert!(rows.iter().all(|row| row.from_exact_deg <= row.orb_deg));
}

#[test]
fn the_lagna_can_be_left_out_and_lillys_moieties_need_it_out() {
    let sdk = western();
    let (george, mary) = (born(&sdk, GEORGE), born(&sdk, MARY));
    let without = sdk
        .chart()
        .synastry(
            &george,
            &mary,
            &SynastryRequest::default().with_lagna(false),
        )
        .unwrap();
    assert!(
        without
            .iter()
            .all(|row| row.first != NatalPoint::Lagna && row.second != NatalPoint::Lagna)
    );
    let refused = sdk
        .chart()
        .synastry(&george, &mary, &SynastryRequest::lilly().with_lagna(true))
        .unwrap_err();
    assert_eq!(refused.field(), Some("lagna"));
    // Lilly's moieties have no orb for Uranus either, which both charts place.
    let refused = sdk
        .chart()
        .synastry(&george, &mary, &SynastryRequest::lilly())
        .unwrap_err();
    assert!(refused.to_string().contains("URANUS"), "{refused}");
}

#[test]
fn each_charts_own_zodiac_parts_from_the_tropical_by_the_precession_between() {
    // Under a sidereal profile, every longitude carries its own instant's
    // ayanamsha, so a separation across the two charts moves by the
    // difference of the two (C241): about 50″ a year over the two years.
    let sdk = context(None);
    let (george, mary) = (born(&sdk, GEORGE), born(&sdk, MARY));
    assert!(george.foundation.zodiac.is_sidereal());
    let drift = mary.foundation.zodiac.offset_deg - george.foundation.zodiac.offset_deg;
    assert!(
        (drift * 3600.0 - 100.0).abs() < 5.0,
        "{drift}° over two years"
    );
    let request = SynastryRequest::default().with_lagna(false);
    let between = |zodiac: SynastryZodiac| -> f64 {
        sdk.chart()
            .synastry(&george, &mary, &request.clone().with_zodiac(zodiac))
            .unwrap()
            .into_iter()
            .find(|row| {
                (row.first, row.aspect, row.second)
                    == (
                        graha(Graha::Mars),
                        WesternAspect::Sextile,
                        graha(Graha::Sun),
                    )
            })
            .unwrap()
            .apart_deg
    };
    let moved = between(SynastryZodiac::Charts) - between(SynastryZodiac::Tropical);
    assert!(
        (moved.abs() - drift.abs()).abs() < 1e-9,
        "{moved} against {drift}"
    );

    // Two charts founded in different zodiacs compare only tropically.
    let tropical = born(&western(), MARY);
    let refused = sdk
        .chart()
        .synastry(
            &george,
            &tropical,
            &request.clone().with_zodiac(SynastryZodiac::Charts),
        )
        .unwrap_err();
    assert_eq!(refused.field(), Some("zodiac"));
    assert!(sdk.chart().synastry(&george, &tropical, &request).is_ok());
}

#[test]
fn a_batch_read_against_one_partner_is_each_chart_read_against_it() {
    // The partner is founded once, with the outer planets the charts
    // carry, and each chart is read against it as two charts are.
    let sdk = western();
    let (jd, latitude, longitude) = MARY;
    let asked = PartnerSynastry::from_json(&format!(
        r#"{{"partner": {{"instant": {jd}, "place": {{"latitude": {latitude}, "longitude": {longitude}, "altitude": 0}}}}, "aspects": ["SEXTILE", "OPPOSITION"]}}"#
    ))
    .unwrap();
    let george = born(&sdk, GEORGE);
    let earlier = born(&sdk, (GEORGE.0 - 3000.25, GEORGE.1, GEORGE.2));
    let charts = [george.clone(), earlier.clone()];
    let read = sdk.chart().synastry_with(&charts, &asked).unwrap();
    let mary = born(&sdk, MARY);
    for (chart, one) in charts.iter().zip(&read) {
        assert_eq!(
            one.aspects,
            sdk.chart().synastry(chart, &mary, &asked.request).unwrap()
        );
        assert_eq!(one.parallels, None, "not asked");
    }
    assert!(
        read[0]
            .aspects
            .iter()
            .any(|row| (row.first, row.aspect, row.second)
                == (
                    graha(Graha::Mars),
                    WesternAspect::Opposition,
                    NatalPoint::Lagna
                ))
    );
    // A partner past the ephemeris is refused as the partner.
    let far = PartnerSynastry::new(
        Partner {
            instant: JulianDay::<Utc>::try_new(9_000_000.0).unwrap(),
            ..asked.partner
        },
        asked.request.clone(),
    );
    let refused = sdk.chart().synastry_with(&charts, &far).unwrap_err();
    assert_eq!(refused.field(), Some("partner"), "{refused}");
}

#[test]
fn the_parallels_across_agree_with_the_recast() {
    // The pairs within 0.95° of one distance from the equator in the
    // Moshier recast; the next, his lagna and her Pluto, stands at 0.98°,
    // too near the orb's edge to pin.
    let recast: [(NatalPoint, NatalPoint, bool, f64); 8] = [
        (graha(Graha::Uranus), graha(Graha::Uranus), false, 0.049),
        (graha(Graha::Mars), graha(Graha::Mercury), false, 0.439),
        (graha(Graha::Saturn), graha(Graha::Moon), false, 0.609),
        (graha(Graha::Jupiter), graha(Graha::Uranus), true, 0.661),
        (graha(Graha::Pluto), graha(Graha::Pluto), false, 0.695),
        (graha(Graha::Neptune), graha(Graha::Pluto), false, 0.831),
        (graha(Graha::Moon), graha(Graha::Pluto), true, 0.854),
        (graha(Graha::Mars), graha(Graha::Sun), false, 0.907),
    ];
    let sdk = western();
    let (george, mary) = (born(&sdk, GEORGE), born(&sdk, MARY));
    let asked = SynastryRequest::default().with_parallels(ParallelRequest::default());
    let rows = sdk
        .chart()
        .synastry_parallels(&george, &mary, &asked)
        .unwrap();
    let close: Vec<_> = rows.iter().filter(|row| row.apart_deg < 0.95).collect();
    assert_eq!(close.len(), recast.len(), "{close:#?}");
    for ((first, second, contrary, apart_deg), row) in recast.iter().zip(&close) {
        assert_eq!(
            (row.first, row.second, row.contrary),
            (*first, *second, *contrary)
        );
        assert!(
            (row.apart_deg - apart_deg).abs() < 0.01,
            "{row:?} against {apart_deg}"
        );
    }

    // The batch reads them the same, beside the aspects.
    let (jd, latitude, longitude) = MARY;
    let partner = PartnerSynastry::from_json(&format!(
        r#"{{"partner": {{"instant": {jd}, "place": {{"latitude": {latitude}, "longitude": {longitude}, "altitude": 0}}}}, "parallels": {{}}}}"#
    ))
    .unwrap();
    let read = sdk.chart().synastry_with(&[george], &partner).unwrap();
    assert_eq!(read[0].parallels.as_deref(), Some(rows.as_slice()));

    // Without the lagna, no row names it.
    let without = sdk
        .chart()
        .synastry_parallels(&born(&sdk, GEORGE), &mary, &asked.with_lagna(false))
        .unwrap();
    assert!(
        without
            .iter()
            .all(|row| row.first != NatalPoint::Lagna && row.second != NatalPoint::Lagna)
    );
}

#[test]
fn the_antiscia_across_agree_with_the_recast() {
    // Every pair across under Lilly's moieties in the Moshier recast, his
    // planet first; the outer three, which the moieties give no orb, stand
    // in none.
    let recast: [(Graha, Graha, bool, f64); 7] = [
        (Graha::Saturn, Graha::Jupiter, false, 0.089),
        (Graha::Saturn, Graha::Moon, false, 2.462),
        (Graha::Mercury, Graha::Mars, false, 3.914),
        (Graha::Mars, Graha::Saturn, true, 4.701),
        (Graha::Venus, Graha::Mars, false, 4.923),
        (Graha::Mars, Graha::Mercury, false, 4.940),
        (Graha::Mars, Graha::Sun, false, 10.800),
    ];
    let sdk = western();
    let (george, mary) = (born(&sdk, GEORGE), born(&sdk, MARY));
    let asked = SynastryRequest::default().with_antiscia(AntisciaRequest::default());
    let rows = sdk
        .chart()
        .synastry_antiscia(&george, &mary, &asked)
        .unwrap();
    assert_eq!(rows.len(), recast.len(), "{rows:#?}");
    for ((first, second, contrary, apart_deg), row) in recast.iter().zip(&rows) {
        assert_eq!(
            (row.first, row.second, row.contrary),
            (*first, *second, *contrary)
        );
        assert!(
            (row.apart_deg - apart_deg).abs() < 0.02,
            "{row:?} against {apart_deg}"
        );
    }

    // The batch reads them the same, beside the aspects.
    let (jd, latitude, longitude) = MARY;
    let partner = PartnerSynastry::from_json(&format!(
        r#"{{"partner": {{"instant": {jd}, "place": {{"latitude": {latitude}, "longitude": {longitude}, "altitude": 0}}}}, "antiscia": {{}}}}"#
    ))
    .unwrap();
    let read = sdk.chart().synastry_with(&[george], &partner).unwrap();
    assert_eq!(read[0].antiscia.as_deref(), Some(rows.as_slice()));
    assert_eq!(read[0].parallels, None);

    // A refusal inside the record is named by its field.
    let wide = PartnerSynastry::from_json(&format!(
        r#"{{"partner": {{"instant": {jd}, "place": {{"latitude": {latitude}, "longitude": {longitude}, "altitude": 0}}}}, "antiscia": {{"orbs": {{"model": "BY_ASPECT", "orbs": [{{"aspect": "TRINE", "orbDeg": 3}}]}}}}}}"#
    ))
    .unwrap_err();
    assert_eq!(wide.field(), Some("synastry.antiscia.orbs.orbs"));
}
